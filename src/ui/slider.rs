use crate::graphics::ui_renderer::batch::{Color, DrawBatch, Rect};

pub struct Slider {
    pub rect: Rect,
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub dragging: bool,
    pub knob_size: f32,
}

impl Slider {
    pub fn new(x: f32, y: f32, w: f32, h: f32, min: f32, max: f32, value: f32) -> Self {
        Self {
            rect: Rect { x, y, w, h },
            min,
            max,
            value,
            dragging: false,
            knob_size: 16.0,
        }
    }

    pub fn update(&mut self, mouse_x: f32, mouse_y: f32, mouse_pressed: bool) {
        if mouse_pressed {
            let is_on_knob = self.knob_rect().contains(mouse_x, mouse_y);
            if is_on_knob {
                self.dragging = true;
            }
        } else {
            self.dragging = false;
        }

        if self.dragging {
            let t = ((mouse_x - self.rect.x) / self.rect.w).clamp(0.0, 1.0);
            self.value = self.min + t * (self.max - self.min);
        }
    }

    pub fn knob_rect(&self) -> Rect {
        let t = (self.value - self.min) / (self.max - self.min);
        let knob_x = self.rect.x + t * (self.rect.w - self.knob_size);
        Rect {
            x: knob_x,
            y: self.rect.y - (self.knob_size - self.rect.h) / 2.0,
            w: self.knob_size,
            h: self.knob_size,
        }
    }

    pub fn render(&self, batch: &mut DrawBatch) {
        batch.push_rect(
            Rect {
                x: self.rect.x,
                y: self.rect.y,
                w: self.rect.w,
                h: self.rect.h,
            },
            Color::new(0.3, 0.3, 0.35, 1.0),
            2.0,
        );

        let fill_w = ((self.value - self.min) / (self.max - self.min)) * self.rect.w;
        batch.push_rect(
            Rect {
                x: self.rect.x,
                y: self.rect.y,
                w: fill_w,
                h: self.rect.h,
            },
            Color::new(0.2, 0.6, 0.9, 1.0),
            2.0,
        );

        let knob = self.knob_rect();
        batch.push_rect(knob, Color::WHITE, 4.0);
    }
}
