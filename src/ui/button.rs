use crate::graphics::ui_renderer::batch::{Color, DrawBatch, Rect};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Normal,
    Hover,
    Pressed,
}

pub struct Button {
    pub rect: Rect,
    pub text: String,
    pub state: ButtonState,
    pub enabled: bool,
    pub normal_color: Color,
    pub hover_color: Color,
    pub pressed_color: Color,
    pub text_color: Color,
    pub corner_radius: f32,
}

impl Button {
    pub fn new(x: f32, y: f32, w: f32, h: f32, text: &str) -> Self {
        Self {
            rect: Rect { x, y, w, h },
            text: text.to_string(),
            state: ButtonState::Normal,
            enabled: true,
            normal_color: Color::new(0.2, 0.2, 0.25, 0.9),
            hover_color: Color::new(0.3, 0.3, 0.4, 0.95),
            pressed_color: Color::new(0.15, 0.15, 0.2, 1.0),
            text_color: Color::WHITE,
            corner_radius: 4.0,
        }
    }

    pub fn update(&mut self, mouse_x: f32, mouse_y: f32, mouse_pressed: bool) -> bool {
        if !self.enabled {
            self.state = ButtonState::Normal;
            return false;
        }

        let is_over = mouse_x >= self.rect.x
            && mouse_x <= self.rect.x + self.rect.w
            && mouse_y >= self.rect.y
            && mouse_y <= self.rect.y + self.rect.h;

        if mouse_pressed && is_over {
            self.state = ButtonState::Pressed;
            true
        } else if is_over {
            self.state = ButtonState::Hover;
            false
        } else {
            self.state = ButtonState::Normal;
            false
        }
    }

    pub fn render(&self, batch: &mut DrawBatch) {
        let bg_color = match self.state {
            ButtonState::Normal => self.normal_color,
            ButtonState::Hover => self.hover_color,
            ButtonState::Pressed => self.pressed_color,
        };

        batch.push_rect(self.rect.clone(), bg_color, self.corner_radius);
    }
}
