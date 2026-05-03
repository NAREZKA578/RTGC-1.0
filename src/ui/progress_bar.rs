use crate::graphics::ui_renderer::batch::{Color, DrawBatch, Rect};

pub struct ProgressBar {
    pub rect: Rect,
    pub progress: f32,
    pub bg_color: Color,
    pub fill_color: Color,
    pub corner_radius: f32,
}

impl ProgressBar {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            rect: Rect { x, y, w, h },
            progress: 0.0,
            bg_color: Color::new(0.15, 0.15, 0.2, 1.0),
            fill_color: Color::new(0.2, 0.7, 0.9, 1.0),
            corner_radius: 3.0,
        }
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    pub fn render(&self, batch: &mut DrawBatch) {
        batch.push_rect(self.rect.clone(), self.bg_color, self.corner_radius);

        let fill_w = self.rect.w * self.progress;
        if fill_w > 0.0 {
            batch.push_rect(
                Rect {
                    x: self.rect.x,
                    y: self.rect.y,
                    w: fill_w,
                    h: self.rect.h,
                },
                self.fill_color,
                self.corner_radius,
            );
        }
    }
}
