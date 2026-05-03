use crate::animation::tween::Tween;
use crate::core::app_state::AppState;
use crate::graphics::ui_renderer::batch::{Color, DrawBatch, Rect};

pub struct SplashScreen {
    logo_alpha: Tween,
    elapsed: f32,
    duration: f32,
}

impl SplashScreen {
    pub fn new() -> Self {
        Self {
            logo_alpha: Tween::new(0.0, 1.0, 1.5).with_easing(crate::animation::easing::ease_out_cubic),
            elapsed: 0.0,
            duration: 2.5,
        }
    }

    pub fn update(&mut self, dt: f32) -> Option<AppState> {
        self.elapsed += dt;
        self.logo_alpha.update(dt);

        if self.elapsed >= self.duration {
            Some(AppState::MainMenu)
        } else {
            None
        }
    }

    pub fn render(&self, batch: &mut DrawBatch, screen_w: f32, screen_h: f32) {
        batch.push_rect(
            Rect { x: 0.0, y: 0.0, w: screen_w, h: screen_h },
            Color::new(0.05, 0.05, 0.08, 1.0),
            0.0,
        );

        let alpha = (self.logo_alpha.elapsed / self.logo_alpha.duration).clamp(0.0, 1.0);
        let logo_color = Color::new(0.8, 0.85, 0.9, alpha);
        let logo_w = 300.0;
        let logo_h = 60.0;
        let logo_x = (screen_w - logo_w) / 2.0;
        let logo_y = (screen_h - logo_h) / 2.0 - 40.0;

        batch.push_rect(
            Rect { x: logo_x, y: logo_y, w: logo_w, h: logo_h },
            logo_color,
            4.0,
        );
    }
}
