use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use crate::graphics::rhi::ResourceHandle;
use std::collections::HashMap;
use std::sync::Arc;

/// Данные одного глифа в атласе
#[derive(Debug, Clone)]
pub struct GlyphData {
    pub uv_rect: [f32; 4], // u, v, width, height (в координатах текстуры 0..1)
    pub advance: f32,
    pub offset: [f32; 2], // смещение относительно базовой линии
}

/// Шрифт с загруженным атласом глифов
pub struct FontAtlas {
    font: FontRef<'static>,
    /// Кэш загруженных глифов (по Unicode скаляру)
    glyphs: HashMap<char, GlyphData>,
    /// Размер шрифта в пикселях (высота)
    pub pixel_height: f32,
    /// Текстура атласа (заполняется при инициализации в RHI)
    pub texture: Option<ResourceHandle>,
    /// Данные пикселей атласа (RGBA)
    atlas_data: Vec<u8>,
    /// Размеры атласа
    pub atlas_width: u32,
    pub atlas_height: u32,
    /// Владелец данных шрифта (для поддержания времени жизни)
    _font_data: Vec<u8>,
}

impl FontAtlas {
    /// Загрузить шрифт из файла TTF/OTF
    pub fn load_from_file(path: &str, pixel_height: f32) -> Result<Self, String> {
        let font_bytes = std::fs::read(path)
            .map_err(|e| format!("Failed to read font file {}: {}", path, e))?;
        
        // Создаём FontRef из вектора байтов
        // Используем Box::leak для получения 'static времени жизни
        let font_data: &'static [u8] = Box::leak(font_bytes.into_boxed_slice());
        let font = FontRef::try_from_slice(font_data)
            .map_err(|e| format!("Failed to parse font: {}", e))?;
        
        let scale = PxScale::from(pixel_height);
        let scaled_font = font.as_scaled(scale);
        
        // Предварительный расчет размеров атласа
        // Для простоты создадим фиксированный атлас 512x512 для ASCII + Cyrillic
        // В продакшене лучше делать динамический упаковщик
        let atlas_width = 512;
        let atlas_height = 512;
        
        let mut glyphs = HashMap::new();
        let mut atlas_data = vec![0u8; (atlas_width * atlas_height * 4) as usize];
        
        // Упаковка глифов (простая сетка для начала)
        // Для ASCII (32-126) и Basic Cyrillic (1024-1103)
        let mut x = 0u32;
        let mut y = 0u32;
        let mut row_max_height = 0u32;
        
        // Символы для генерации: пробел, ASCII печатные, кириллица
        let chars: Vec<char> = (32..127)
            .chain(1024..1104)
            .filter_map(std::char::from_u32)
            .collect();
        
        // Create a question mark glyph for fallback
        let qmark_id = font.glyph_id('?');
        
        for ch in chars {
            let glyph_id = font.glyph_id(ch);
            let glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(0.0, 0.0));
            
            let outlined = font.outline_glyph(glyph);
            
            let bounds = match outlined {
                Some(ref o) => o.px_bounds(),
                None => {
                    // Fallback to question mark
                    let qmark_glyph = qmark_id.with_scale_and_position(scale, ab_glyph::point(0.0, 0.0));
                    match font.outline_glyph(qmark_glyph) {
                        Some(q) => q.px_bounds(),
                        None => continue,
                    }
                }
            };
            
            let width = bounds.width() as u32;
            let height = bounds.height() as u32;
            
            if width == 0 || height == 0 {
                // Пустой глиф (например, пробел)
                glyphs.insert(ch, GlyphData {
                    uv_rect: [0.0, 0.0, 0.0, 0.0],
                    advance: scaled_font.h_advance(glyph_id) / pixel_height,
                    offset: [0.0, 0.0],
                });
                continue;
            }
            
            // Перенос строки если не влезает
            if x + width > atlas_width {
                x = 0;
                y += row_max_height;
                row_max_height = 0;
            }
            
            if y + height > atlas_height {
                return Err("Font atlas overflow! Increase atlas size.".to_string());
            }
            
            row_max_height = row_max_height.max(height);
            
            // Растеризация глифа в атлас
            if let Some(outlined_glyph) = outlined {
                outlined_glyph.draw(|gx, gy, v| {
                    let px_x = (bounds.min.x as u32 + gx) as usize;
                    let px_y = (bounds.min.y as u32 + gy) as usize;
                    
                    if px_x < atlas_width as usize && px_y < atlas_height as usize {
                        let idx = (px_y * atlas_width as usize + px_x) * 4;
                        // Alpha channel only (white glyph)
                        atlas_data[idx] = 255;
                        atlas_data[idx + 1] = 255;
                        atlas_data[idx + 2] = 255;
                        atlas_data[idx + 3] = (v * 255.0) as u8;
                    }
                });
            }
            
            // Сохранение данных глифа
            let u = x as f32 / atlas_width as f32;
            let v = y as f32 / atlas_height as f32;
            let w = width as f32 / atlas_width as f32;
            let h = height as f32 / atlas_height as f32;
            
            glyphs.insert(ch, GlyphData {
                uv_rect: [u, v, w, h],
                advance: scaled_font.h_advance(glyph_id) / pixel_height,
                offset: [
                    bounds.min.x as f32 / pixel_height,
                    bounds.min.y as f32 / pixel_height,
                ],
            });
            
            x += width;
        }
        
        Ok(Self {
            font,
            glyphs,
            pixel_height,
            texture: None,
            atlas_data,
            atlas_width: atlas_width as u32,
            atlas_height: atlas_height as u32,
            _font_data: font_data.to_vec(),
        })
    }
    
    /// Получить данные глифа
    pub fn get_glyph(&self, ch: char) -> Option<&GlyphData> {
        self.glyphs.get(&ch)
    }
    
    /// Получить размеры текста (ширина, высота)
    pub fn measure_text(&self, text: &str) -> (f32, f32) {
        let scale = PxScale::from(self.pixel_height);
        let scaled_font = self.font.as_scaled(scale);
        
        let mut width = 0.0f32;
        for ch in text.chars() {
            let gid = self.font.glyph_id(ch);
            width += scaled_font.h_advance(gid);
        }
        
        let height = self.pixel_height;
        (width, height)
    }
    
    /// Получить сырые данные атласа для загрузки в текстуру
    pub fn get_atlas_data(&self) -> &[u8] {
        &self.atlas_data
    }
    
    pub fn get_atlas_dimensions(&self) -> (u32, u32) {
        (self.atlas_width, self.atlas_height)
    }
}

/// Менеджер шрифтов
pub struct FontManager {
    fonts: HashMap<String, Arc<FontAtlas>>,
}

impl FontManager {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }
    
    pub fn load_font(&mut self, name: &str, path: &str, size: f32) -> Result<(), String> {
        let atlas = FontAtlas::load_from_file(path, size)?;
        self.fonts.insert(name.to_string(), Arc::new(atlas));
        Ok(())
    }
    
    pub fn get_font(&self, name: &str) -> Option<Arc<FontAtlas>> {
        self.fonts.get(name).cloned()
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}