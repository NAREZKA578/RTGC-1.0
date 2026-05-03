use crate::graphics::ui_renderer::batch::{Color, DrawBatch, Rect};

pub struct Panel {
    pub rect: Rect,
    pub bg_color: Color,
    pub corner_radius: f32,
    pub border_color: Option<Color>,
    pub border_width: f32,
}

impl Panel {
    pub fn new(x: f32, y: f32, w: f32, h: f32, bg_color: Color) -> Self {
        Self {
            rect: Rect { x, y, w, h },
            bg_color,
            corner_radius: 0.0,
            border_color: None,
            border_width: 1.0,
        }
    }

    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn with_border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = width;
        self
    }

    pub fn render(&self, batch: &mut DrawBatch) {
        batch.push_rect(self.rect.clone(), self.bg_color, self.corner_radius);
    }
}
