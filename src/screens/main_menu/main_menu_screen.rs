use crate::animation::tween::Tween;
use crate::core::app_state::AppState;
use crate::graphics::ui_renderer::batch::{Color, DrawBatch, Rect};
use crate::ui::button::Button;

pub enum MenuResult {
    NewGame,
    LoadGame,
    Settings,
    Exit,
}

pub struct MainMenuScreen {
    logo_alpha: Tween,
    buttons_alpha: [Tween; 4],
    elapsed: f32,
    btn_w: f32,
    btn_h: f32,
    pub result: Option<MenuResult>,
}

impl MainMenuScreen {
    pub fn new() -> Self {
        Self {
            logo_alpha: Tween::new(0.0, 1.0, 0.8).with_easing(crate::animation::easing::ease_out_cubic),
            buttons_alpha: [
                Tween::new(0.0, 1.0, 0.5).with_easing(crate::animation::easing::ease_out_cubic),
                Tween::new(0.0, 1.0, 0.5).with_easing(crate::animation::easing::ease_out_cubic),
                Tween::new(0.0, 1.0, 0.5).with_easing(crate::animation::easing::ease_out_cubic),
                Tween::new(0.0, 1.0, 0.5).with_easing(crate::animation::easing::ease_out_cubic),
            ],
            elapsed: 0.0,
            btn_w: 300.0,
            btn_h: 50.0,
            result: None,
        }
    }

    fn compute_layout(&self, sw: f32, sh: f32) -> Vec<Rect> {
        let start_y = sh / 2.0 + 30.0;
        let spacing = self.btn_h + 16.0;
        let center_x = (sw - self.btn_w) / 2.0;
        let labels = ["НОВАЯ ИГРА", "ЗАГРУЗИТЬ ИГРУ", "НАСТРОЙКИ", "ВЫХОД"];
        labels
            .iter()
            .enumerate()
            .map(|(i, _)| Rect {
                x: center_x,
                y: start_y + (i as f32) * spacing,
                w: self.btn_w,
                h: self.btn_h,
            })
            .collect()
    }

    pub fn update(
        &mut self,
        mouse_x: f32,
        mouse_y: f32,
        mouse_just_pressed: bool,
        dt: f32,
        screen_w: f32,
        screen_h: f32,
    ) -> Option<AppState> {
        self.elapsed += dt;
        self.logo_alpha.update(dt);
        for i in 0..4 {
            self.buttons_alpha[i].update(dt);
        }

        let rects = self.compute_layout(screen_w, screen_h);

        for (i, rect) in rects.iter().enumerate() {
            let is_over = mouse_x >= rect.x
                && mouse_x <= rect.x + rect.w
                && mouse_y >= rect.y
                && mouse_y <= rect.y + rect.h;

            if mouse_just_pressed && is_over {
                return match i {
                    0 => Some(AppState::CharacterCreation),
                    1 => Some(AppState::SaveSelect),
                    2 => Some(AppState::Settings { return_to: Box::new(AppState::MainMenu) }),
                    3 => std::process::exit(0),
                    _ => None,
                };
            }
        }

        None
    }

    pub fn render(&self, batch: &mut DrawBatch, screen_w: f32, screen_h: f32) {
        batch.push_rect(
            Rect { x: 0.0, y: 0.0, w: screen_w, h: screen_h },
            Color::new(0.08, 0.08, 0.12, 1.0),
            0.0,
        );

        let logo_alpha = (self.logo_alpha.elapsed / self.logo_alpha.duration).clamp(0.0, 1.0);
        let logo_color = Color::new(0.85, 0.9, 0.95, logo_alpha);

        let logo_w = 320.0;
        let logo_h = 70.0;
        let logo_x = (screen_w - logo_w) / 2.0;
        let logo_y = screen_h / 2.0 - 180.0;

        batch.push_rect(
            Rect { x: logo_x, y: logo_y, w: logo_w, h: logo_h },
            logo_color,
            6.0,
        );

        let rects = self.compute_layout(screen_w, screen_h);
        for rect in &rects {
            batch.push_rect(rect.clone(), Color::new(0.2, 0.2, 0.25, 0.9), 4.0);
        }
    }

    pub fn get_button_rects(&self, screen_w: f32, screen_h: f32) -> Vec<Rect> {
        self.compute_layout(screen_w, screen_h)
    }
}
