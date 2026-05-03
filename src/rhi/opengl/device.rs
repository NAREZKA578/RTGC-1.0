use crate::rhi::command_buffer::CommandBuffer;
use crate::rhi::opengl::buffer::GlBuffer;
use crate::rhi::opengl::framebuffer::GlFramebuffer;
use crate::rhi::opengl::pipeline::GlPipeline;
use crate::rhi::opengl::shader::GlShader;
use crate::rhi::opengl::texture::GlTexture;
use crate::rhi::traits::*;
use crate::rhi::types::*;
use anyhow::Result;
use glow::HasContext;
use std::sync::atomic::{AtomicU64, Ordering};

use std::sync::Arc;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

pub struct GlDevice {
    gl: Arc<glow::Context>,
}

impl GlDevice {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        Self { gl }
    }
}

impl RhiDevice for GlDevice {
    fn create_buffer(&self, desc: &BufferDesc) -> Box<dyn RhiBuffer> {
        let gl = &self.gl;
        let gl_buffer = unsafe { gl.create_buffer().unwrap() };
        let id = next_id();
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(gl_buffer));
            match desc.access {
                BufferAccess::Static => {
                    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &vec![0u8; desc.size], glow::STATIC_DRAW);
                }
                BufferAccess::Dynamic => {
                    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &vec![0u8; desc.size], glow::DYNAMIC_DRAW);
                }
                BufferAccess::Stream => {
                    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &vec![0u8; desc.size], glow::STREAM_DRAW);
                }
            }
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
        Box::new(GlBuffer {
            id,
            gl_buffer,
            size: desc.size,
            usage: desc.usage,
        })
    }

    fn create_texture(&self, desc: &TextureDesc) -> Box<dyn RhiTexture> {
        let gl = &self.gl;
        let gl_texture = unsafe { gl.create_texture().unwrap() };
        let id = next_id();

        let internal_format = match desc.format {
            TextureFormat::Rgba8 => glow::RGBA8,
            TextureFormat::R8 => glow::R8,
            TextureFormat::Depth24 => glow::DEPTH_COMPONENT24,
            TextureFormat::Depth24Stencil8 => glow::DEPTH24_STENCIL8,
        };

        let format = match desc.format {
            TextureFormat::Rgba8 => glow::RGBA,
            TextureFormat::R8 => glow::RED,
            TextureFormat::Depth24 => glow::DEPTH_COMPONENT,
            TextureFormat::Depth24Stencil8 => glow::DEPTH_STENCIL,
        };

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(gl_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                internal_format as i32,
                desc.width as i32,
                desc.height as i32,
                0,
                format,
                glow::UNSIGNED_BYTE,
                None,
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            gl.bind_texture(glow::TEXTURE_2D, None);
        }

        Box::new(GlTexture {
            id,
            gl_texture,
            width: desc.width,
            height: desc.height,
        })
    }

    fn create_shader(
        &self,
        vert_src: &str,
        frag_src: &str,
        label: &str,
    ) -> Result<Box<dyn RhiShader>> {
        let gl = &self.gl;
        let id = next_id();

        let program = unsafe {
            let program = gl.create_program()
                .map_err(|e| anyhow::anyhow!("Failed to create program: {:?}", e))?;

            let vert_shader = gl
                .create_shader(glow::VERTEX_SHADER)
                .map_err(|e| anyhow::anyhow!("Failed to create vertex shader: {:?}", e))?;
            gl.shader_source(vert_shader, vert_src);
            gl.compile_shader(vert_shader);
            if !gl.get_shader_compile_status(vert_shader) {
                let log = gl.get_shader_info_log(vert_shader);
                return Err(anyhow::anyhow!("Vertex shader compile error: {}", log));
            }
            gl.attach_shader(program, vert_shader);

            let frag_shader = gl
                .create_shader(glow::FRAGMENT_SHADER)
                .map_err(|e| anyhow::anyhow!("Failed to create fragment shader: {:?}", e))?;
            gl.shader_source(frag_shader, frag_src);
            gl.compile_shader(frag_shader);
            if !gl.get_shader_compile_status(frag_shader) {
                let log = gl.get_shader_info_log(frag_shader);
                return Err(anyhow::anyhow!("Fragment shader compile error: {}", log));
            }
            gl.attach_shader(program, frag_shader);

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                let log = gl.get_program_info_log(program);
                return Err(anyhow::anyhow!("Program link error: {}", log));
            }

            gl.delete_shader(vert_shader);
            gl.delete_shader(frag_shader);

            program
        };

        tracing::info!("Shader '{}' created (id={})", label, id);

        Ok(Box::new(GlShader { id, program }))
    }

    fn create_pipeline(
        &self,
        shader: &dyn RhiShader,
        _blend: BlendMode,
        _depth_test: bool,
        _depth_write: bool,
        _cull: CullMode,
    ) -> Box<dyn RhiPipeline> {
        let id = next_id();
        let gl_shader = shader
            .as_any()
            .downcast_ref::<GlShader>()
            .expect("Expected GlShader");

        Box::new(GlPipeline {
            id,
            program: gl_shader.program,
            depth_test: false,
            depth_write: false,
            cull_enabled: false,
            cull_face: glow::BACK,
        })
    }

    fn create_framebuffer(&self, attachments: &[&dyn RhiTexture]) -> Box<dyn RhiFramebuffer> {
        let gl = &self.gl;
        let id = next_id();
        let gl_fbo = unsafe { gl.create_framebuffer().unwrap() };

        let mut attachment_ids = Vec::new();
        for tex in attachments {
            let gl_tex = tex
                .as_any()
                .downcast_ref::<GlTexture>()
                .expect("Expected GlTexture");
            attachment_ids.push(gl_tex.gl_texture);
        }

        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(gl_fbo));
            for (i, tex_id) in attachment_ids.iter().enumerate() {
                let attachment = if i == 0 {
                    glow::COLOR_ATTACHMENT0
                } else {
                    glow::COLOR_ATTACHMENT0 + i as u32
                };
                gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    attachment,
                    glow::TEXTURE_2D,
                    Some(*tex_id),
                    0,
                );
            }
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }

        Box::new(GlFramebuffer {
            id,
            gl_fbo,
            width: 0,
            height: 0,
        })
    }

    fn upload_buffer(&self, buf: &dyn RhiBuffer, offset: usize, data: &[u8]) {
        let gl = &self.gl;
        let gl_buf = buf
            .as_any()
            .downcast_ref::<GlBuffer>()
            .expect("Expected GlBuffer");

        let target = match gl_buf.usage {
            BufferUsage::Vertex => glow::ARRAY_BUFFER,
            BufferUsage::Index => glow::ELEMENT_ARRAY_BUFFER,
            _ => glow::ARRAY_BUFFER,
        };

        unsafe {
            gl.bind_buffer(target, Some(gl_buf.gl_buffer));
            gl.buffer_sub_data_u8_slice(target, offset as i32, data);
            gl.bind_buffer(target, None);
        }
    }

    fn upload_texture(&self, tex: &dyn RhiTexture, data: &[u8]) {
        let gl = &self.gl;
        let gl_tex = tex
            .as_any()
            .downcast_ref::<GlTexture>()
            .expect("Expected GlTexture");

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(gl_tex.gl_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                gl_tex.width as i32,
                gl_tex.height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(data),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    fn generate_mipmaps(&self, tex: &dyn RhiTexture) {
        let gl = &self.gl;
        let gl_tex = tex
            .as_any()
            .downcast_ref::<GlTexture>()
            .expect("Expected GlTexture");

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(gl_tex.gl_texture));
            gl.generate_mipmap(glow::TEXTURE_2D);
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    fn begin_frame(&mut self) {}

    fn end_frame(&mut self) {}

    fn submit(&mut self, cmds: &CommandBuffer) {
        let gl = &self.gl;

        for cmd in &cmds.commands {
            match cmd {
                crate::rhi::command_buffer::RenderCommand::ClearColor { r, g, b, a } => {
                    unsafe {
                        gl.clear_color(*r, *g, *b, *a);
                    }
                }
                crate::rhi::command_buffer::RenderCommand::ClearDepth { depth } => {
                    unsafe {
                        gl.clear_depth_f32(*depth);
                    }
                }
                crate::rhi::command_buffer::RenderCommand::SetViewport { x, y, w, h } => {
                    unsafe {
                        gl.viewport(*x, *y, *w, *h);
                    }
                }
                crate::rhi::command_buffer::RenderCommand::Draw { vertices, instances } => {
                    unsafe {
                        if *instances > 1 {
                            gl.draw_arrays_instanced(glow::TRIANGLES, 0, *vertices as i32, *instances as i32);
                        } else {
                            gl.draw_arrays(glow::TRIANGLES, 0, *vertices as i32);
                        }
                    }
                }
                crate::rhi::command_buffer::RenderCommand::DrawIndexed { indices, instances } => {
                    unsafe {
                        if *instances > 1 {
                            gl.draw_elements_instanced(glow::TRIANGLES, *indices as i32, glow::UNSIGNED_SHORT, 0, *instances as i32);
                        } else {
                            gl.draw_elements(glow::TRIANGLES, *indices as i32, glow::UNSIGNED_SHORT, 0);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn backend_name(&self) -> &str {
        "OpenGL 3.3"
    }

    fn capabilities(&self) -> DeviceCaps {
        let gl = &self.gl;
        let max_texture_size = unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) as u32 };
        DeviceCaps {
            max_texture_size,
            max_uniform_buffer_size: 65536,
            supports_instancing: true,
        }
    }
}
