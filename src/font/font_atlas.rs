use fontdue::{Font, FontSettings, Metrics};
use std::collections::BTreeMap;

pub struct GlyphData {
    pub bitmap: Vec<u8>,
    pub metrics: Metrics,
    pub width: u32,
    pub height: u32,
}

pub struct FontAtlas {
    font: Font,
    glyph_cache: BTreeMap<(char, u32), GlyphData>,
}

impl FontAtlas {
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        let font = Font::from_bytes(data, FontSettings::default())
            .map_err(|e| e.to_string())?;

        Ok(Self {
            font,
            glyph_cache: BTreeMap::new(),
        })
    }

    pub fn rasterize(&mut self, c: char, size: f32) -> &GlyphData {
        let size_key = size as u32;
        self.glyph_cache
            .entry((c, size_key))
            .or_insert_with(|| {
                let (metrics, bitmap) = self.font.rasterize(c, size);
                GlyphData {
                    bitmap,
                    metrics,
                    width: metrics.width as u32,
                    height: metrics.height as u32,
                }
            })
    }

    pub fn metrics(&self, c: char, size: f32) -> Metrics {
        self.font.metrics(c, size)
    }

    pub fn line_metrics(&self, text: &str, size: f32) -> (f32, f32) {
        let mut width = 0.0f32;
        let mut height: f32 = 0.0;
        for c in text.chars() {
            let m = self.font.metrics(c, size);
            width += m.advance_width;
            height = height.max(m.height as f32);
        }
        (width, height)
    }
}
