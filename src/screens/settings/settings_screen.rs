use crate::core::app_state::AppState;
use crate::graphics::ui_renderer::batch::{Color, DrawBatch, Rect};
use crate::ui::panel::Panel;

pub struct SettingsScreen {
    return_to: AppState,
}

impl SettingsScreen {
    pub fn new(return_to: AppState) -> Self {
        Self { return_to }
    }

    fn btn_back_rect(&self, sw: f32, sh: f32) -> Rect {
        let btn_w = 200.0;
        let btn_h = 48.0;
        Rect {
            x: (sw - btn_w) / 2.0,
            y: sh - 80.0,
            w: btn_w,
            h: btn_h,
        }
    }

    pub fn update(
        &mut self,
        mouse_x: f32,
        mouse_y: f32,
        mouse_just_pressed: bool,
    ) -> Option<AppState> {
        let rect = self.btn_back_rect(1280.0, 720.0);
        if mouse_just_pressed && rect.contains(mouse_x, mouse_y) {
            return Some(self.return_to.clone());
        }
        None
    }

    pub fn render(&self, batch: &mut DrawBatch, screen_w: f32, screen_h: f32) {
        batch.push_rect(
            Rect { x: 0.0, y: 0.0, w: screen_w, h: screen_h },
            Color::new(0.1, 0.1, 0.14, 1.0),
            0.0,
        );

        let panel_w = 500.0;
        let panel_h = 350.0;
        Panel::new(
            (screen_w - panel_w) / 2.0,
            (screen_h - panel_h) / 2.0 - 30.0,
            panel_w,
            panel_h,
            Color::new(0.15, 0.15, 0.2, 0.95),
        )
        .with_corner_radius(8.0)
        .render(batch);

        let back = self.btn_back_rect(screen_w, screen_h);
        batch.push_rect(back, Color::new(0.2, 0.2, 0.25, 0.9), 4.0);
    }
}
