use crate::font::font_atlas::FontAtlas;
use crate::renderer::ui_renderer::batch::{Color, DrawBatch};

pub struct TextRenderer;

impl TextRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render_text(
        &self,
        _batch: &mut DrawBatch,
        _text: &str,
        _x: f32,
        _y: f32,
        _size: f32,
        _color: Color,
        _font: &FontAtlas,
    ) {
        // TODO: Implement text rendering
    }
}
