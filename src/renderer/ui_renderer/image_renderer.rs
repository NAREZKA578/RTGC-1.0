use crate::renderer::ui_renderer::batch::{Color, DrawBatch, Rect};

pub struct ImageRenderer;

impl ImageRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        _batch: &mut DrawBatch,
        _rect: Rect,
        _texture_id: u32,
        _tint: Color,
    ) {
        // TODO: Implement image rendering
    }
}
