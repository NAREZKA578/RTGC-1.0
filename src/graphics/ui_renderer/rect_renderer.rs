use crate::graphics::ui_renderer::batch::{Color, DrawBatch, Rect};

pub struct RectRenderer;

impl RectRenderer {
    pub fn new() -> Self { Self }

    pub fn render(&self, _batch: &mut DrawBatch, _screen_width: f32, _screen_height: f32) {}
}
