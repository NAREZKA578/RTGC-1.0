use crate::graphics::ui_renderer::batch::{Color, DrawBatch};

pub struct Label {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub size: f32,
    pub color: Color,
}

impl Label {
    pub fn new(x: f32, y: f32, text: &str, size: f32, color: Color) -> Self {
        Self {
            x,
            y,
            text: text.to_string(),
            size,
            color,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
}
