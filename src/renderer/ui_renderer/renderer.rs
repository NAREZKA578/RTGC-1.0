use crate::renderer::ui_renderer::batch::{Color, DrawBatch, UiVertex};
use anyhow::Result;
use bytemuck::cast_slice;
use fontdue::{Font, FontSettings};
use glow::HasContext;
use std::collections::HashMap;
use std::sync::Arc;

const ATLAS_W: u32 = 2048;
const ATLAS_H: u32 = 1024;

const UI_VERT_SRC: &str = r#"
#version 330 core
layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_uv;
layout(location = 2) in vec4 a_color;
layout(location = 3) in float a_mode;

uniform vec2 u_screen_size;

out vec2 v_uv;
out vec4 v_color;
out float v_mode;

void main() {
    vec2 ndc = (a_pos / u_screen_size) * 2.0 - 1.0;
    ndc.y = -ndc.y;
    gl_Position = vec4(ndc, 0.0, 1.0);
    v_uv = a_uv;
    v_color = a_color;
    v_mode = a_mode;
}
"#;

const UI_FRAG_SRC: &str = r#"
#version 330 core
in vec2 v_uv;
in vec4 v_color;
in float v_mode;
out vec4 out_color;

uniform sampler2D u_font_tex;

void main() {
    if (v_mode > 0.5) {
        float alpha = texture(u_font_tex, v_uv).r;
        out_color = vec4(v_color.rgb, v_color.a * alpha);
    } else {
        out_color = v_color;
    }
}
"#;

#[derive(Clone, Copy, Debug)]
pub struct GlyphInfo {
    pub atlas_x: u32,
    pub atlas_y: u32,
    pub w: u32,
    pub h: u32,
    pub advance: f32,
    pub bitmap_top_x: f32,
    pub bitmap_top_y: f32,
}

pub struct FontAtlas {
    pub gl_texture: glow::NativeTexture,
    pub glyphs: HashMap<char, GlyphInfo>,
    pub line_height: f32,
}

impl FontAtlas {
    pub fn new(gl: &glow::Context, font: &Font, size: f32) -> Result<Self> {
        let mut pixels = vec![0u8; (ATLAS_W * ATLAS_H) as usize];
        let mut glyphs: HashMap<char, GlyphInfo> = HashMap::new();
        let mut cursor_x: u32 = 2;
        let mut cursor_y: u32 = 2;
        let mut row_h: u32 = 0;

        // ASCII printable + Cyrillic
        let mut chars_to_rasterize: Vec<char> = (32u8..=126).map(|c| c as char).collect();
        // Cyrillic range: Ѐ (U+0400) to џ (U+045F) + extended
        for cp in 0x0400u32..=0x04FF {
            if let Some(ch) = char::from_u32(cp) {
                chars_to_rasterize.push(ch);
            }
        }
        // Add common Russian-specific chars that might be outside
        for cp in [0x0401, 0x0451, 0x2014, 0x2013, 0x2026, 0x00B0] {
            if let Some(ch) = char::from_u32(cp) {
                if !chars_to_rasterize.contains(&ch) {
                    chars_to_rasterize.push(ch);
                }
            }
        }

        let mut max_h: f32 = 0.0;

        for ch in chars_to_rasterize {
            let (metrics, bitmap) = font.rasterize(ch, size);
            if bitmap.is_empty() {
                continue;
            }
            let w = metrics.width as u32;
            let h = metrics.height as u32;

            if cursor_x + w + 1 >= ATLAS_W {
                cursor_x = 1;
                cursor_y += row_h + 1;
                row_h = 0;
            }
            if cursor_y + h + 1 >= ATLAS_H {
                break;
            }

            // Copy bitmap into atlas
            for py in 0..h {
                for px in 0..w {
                    let src = (py * w + px) as usize;
                    let dst = ((cursor_y + py) * ATLAS_W + (cursor_x + px)) as usize;
                    if src < bitmap.len() && dst < pixels.len() {
                        pixels[dst] = bitmap[src];
                    }
                }
            }

            let line_height = font.horizontal_line_metrics(size).map(|m| m.ascent - m.descent + m.line_gap).unwrap_or(size * 1.4);
            max_h = max_h.max(line_height);

            glyphs.insert(ch, GlyphInfo {
                atlas_x: cursor_x,
                atlas_y: cursor_y,
                w,
                h,
                advance: metrics.advance_width,
                bitmap_top_x: metrics.xmin as f32,
                bitmap_top_y: -metrics.ymin as f32,
            });

            cursor_x += w + 1;
            if h > row_h {
                row_h = h;
            }
        }

        let gl_texture = unsafe {
            let tex = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::R8 as i32,
                ATLAS_W as i32,
                ATLAS_H as i32,
                0,
                glow::RED,
                glow::UNSIGNED_BYTE,
                Some(&pixels),
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            gl.bind_texture(glow::TEXTURE_2D, None);
            tex
        };

        let lh = max_h.max(30.0);

        Ok(Self {
            gl_texture,
            glyphs,
            line_height: lh,
        })
    }
}

pub struct UiRenderer {
    gl: Arc<glow::Context>,
    program: glow::NativeProgram,
    vao: glow::NativeVertexArray,
    vbo: glow::NativeBuffer,
    screen_size_loc: Option<glow::UniformLocation>,
    font_tex_loc: Option<glow::UniformLocation>,
    pub font_atlas: Option<FontAtlas>,
}

impl UiRenderer {
    pub fn new(gl: Arc<glow::Context>) -> Result<Self> {
        let program = unsafe {
            let vs = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vs, UI_VERT_SRC);
            gl.compile_shader(vs);

            let fs = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(fs, UI_FRAG_SRC);
            gl.compile_shader(fs);

            let prog = gl.create_program().unwrap();
            gl.attach_shader(prog, vs);
            gl.attach_shader(prog, fs);
            gl.link_program(prog);
            gl.delete_shader(vs);
            gl.delete_shader(fs);
            prog
        };

        let vao = unsafe { gl.create_vertex_array().unwrap() };
        let vbo = unsafe { gl.create_buffer().unwrap() };

        let screen_size_loc = unsafe { gl.get_uniform_location(program, "u_screen_size") };
        let font_tex_loc = unsafe { gl.get_uniform_location(program, "u_font_tex") };

        let mut renderer = Self {
            gl: gl.clone(),
            program,
            vao,
            vbo,
            screen_size_loc,
            font_tex_loc,
            font_atlas: None,
        };

        if let Ok(data) = std::fs::read("assets/fonts/main_font.ttf") {
            match Font::from_bytes(data.as_slice(), FontSettings::default()) {
                Ok(font) => {
                    match FontAtlas::new(&gl, &font, 28.0) {
                        Ok(atlas) => {
                            tracing::info!("Font atlas: {} glyphs, line_height={:.1}", atlas.glyphs.len(), atlas.line_height);
                            renderer.font_atlas = Some(atlas);
                        }
                        Err(e) => tracing::error!("Failed to create font atlas: {}", e),
                    }
                }
                Err(e) => tracing::error!("Failed to load font: {}", e),
            }
        } else {
            tracing::warn!("Font not found at assets/fonts/main_font.ttf");
        }

        Ok(renderer)
    }

    pub fn render(&self, batch: &DrawBatch, screen_w: f32, screen_h: f32) {
        if batch.vertex_count() == 0 {
            return;
        }

        let gl = &self.gl;

        unsafe {
            gl.disable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func_separate(
                glow::SRC_ALPHA,
                glow::ONE_MINUS_SRC_ALPHA,
                glow::ONE,
                glow::ONE_MINUS_SRC_ALPHA,
            );

            gl.viewport(0, 0, screen_w as i32, screen_h as i32);
            gl.use_program(Some(self.program));

            if let Some(loc) = &self.screen_size_loc {
                gl.uniform_2_f32_slice(Some(loc), &[screen_w, screen_h]);
            }

            if let Some(ref atlas) = self.font_atlas {
                gl.active_texture(glow::TEXTURE0);
                gl.bind_texture(glow::TEXTURE_2D, Some(atlas.gl_texture));
                if let Some(loc) = &self.font_tex_loc {
                    gl.uniform_1_i32(Some(loc), 0);
                }
            }

            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, cast_slice(&batch.vertices), glow::STREAM_DRAW);

            let stride = std::mem::size_of::<UiVertex>() as i32;
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 8);
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 4, glow::FLOAT, false, stride, 16);
            gl.enable_vertex_attrib_array(3);
            gl.vertex_attrib_pointer_f32(3, 1, glow::FLOAT, false, stride, 32);

            gl.draw_arrays(glow::TRIANGLES, 0, batch.vertex_count() as i32);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.use_program(None);
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.disable(glow::BLEND);
        }
    }

    /// Push text. `y` is the baseline of the text line.
    pub fn push_text(
        &self,
        batch: &mut DrawBatch,
        text: &str,
        x: f32,
        y: f32,
        _size: f32,
        color: Color,
    ) {
        let atlas = match &self.font_atlas {
            Some(a) => a,
            None => {
                tracing::warn!("Font atlas not loaded, cannot render text: '{}'", text);
                return;
            }
        };

        let atlas_w = ATLAS_W as f32;
        let atlas_h = ATLAS_H as f32;
        let mut cursor_x = x;
        let c = [color.r, color.g, color.b, color.a];

        for ch in text.chars() {
            if let Some(info) = atlas.glyphs.get(&ch) {
                if info.w == 0 || info.h == 0 {
                    cursor_x += info.advance;
                    continue;
                }

                let u0 = info.atlas_x as f32 / atlas_w;
                let v0 = info.atlas_y as f32 / atlas_h;
                let u1 = (info.atlas_x + info.w) as f32 / atlas_w;
                let v1 = (info.atlas_y + info.h) as f32 / atlas_h;

                let draw_x = cursor_x + info.bitmap_top_x;
                let draw_y = y - info.bitmap_top_y;
                let gw = info.w as f32;
                let gh = info.h as f32;

                batch.vertices.push(UiVertex { pos: [draw_x, draw_y], uv: [u0, v0], color: c, mode: 1.0 });
                batch.vertices.push(UiVertex { pos: [draw_x + gw, draw_y], uv: [u1, v0], color: c, mode: 1.0 });
                batch.vertices.push(UiVertex { pos: [draw_x + gw, draw_y + gh], uv: [u1, v1], color: c, mode: 1.0 });
                batch.vertices.push(UiVertex { pos: [draw_x, draw_y], uv: [u0, v0], color: c, mode: 1.0 });
                batch.vertices.push(UiVertex { pos: [draw_x + gw, draw_y + gh], uv: [u1, v1], color: c, mode: 1.0 });
                batch.vertices.push(UiVertex { pos: [draw_x, draw_y + gh], uv: [u0, v1], color: c, mode: 1.0 });

                cursor_x += info.advance;
            } else if ch == ' ' {
                cursor_x += 14.0;
            } else {
                tracing::trace!("Missing glyph '{}' (U+{:04X})", ch, ch as u32);
            }
        }
    }

    pub fn measure_text_width(&self, text: &str) -> f32 {
        let atlas = match &self.font_atlas {
            Some(a) => a,
            None => return 0.0,
        };

        let mut width = 0.0;
        for ch in text.chars() {
            if let Some(info) = atlas.glyphs.get(&ch) {
                width += info.advance;
            } else if ch == ' ' {
                width += 14.0;
            }
        }
        width
    }
}
