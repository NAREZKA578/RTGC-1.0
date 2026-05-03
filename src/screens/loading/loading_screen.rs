use crate::core::app_state::{AppState, CharacterData};
use crate::graphics::ui_renderer::batch::{Color, DrawBatch, Rect};
use crate::ui::progress_bar::ProgressBar;

pub struct LoadingScreen {
    _character_data: CharacterData,
    progress_bar: ProgressBar,
    current_stage: u8,
    total_stages: u8,
    stage_progress: f32,
}

impl LoadingScreen {
    pub fn new(_character_data: CharacterData) -> Self {
        Self {
            _character_data,
            progress_bar: ProgressBar::new(0.0, 0.0, 400.0, 16.0),
            current_stage: 1,
            total_stages: 11,
            stage_progress: 0.0,
        }
    }

    pub fn stage_message(&self) -> &'static str {
        match self.current_stage {
            1 => "Загрузка шрифтов",
            2 => "Загрузка текстур",
            3 => "Инициализация рендерера",
            4 => "Генерация дорожной сети",
            5 => "Загрузка ландшафта",
            6 => "Инициализация физики",
            7 => "Загрузка звуков",
            8 => "Создание персонажа",
            9 => "Спавн техники",
            10 => "Подготовка мира",
            11 => "Финальная настройка",
            _ => "Загрузка...",
        }
    }

    pub fn update(&mut self, dt: f32) -> Option<AppState> {
        self.stage_progress += dt * 2.5;

        if self.stage_progress >= 1.0 {
            self.current_stage += 1;
            self.stage_progress = 0.0;

            if self.current_stage > self.total_stages {
                return Some(AppState::Playing);
            }
        }

        let total_progress =
            ((self.current_stage as f32 - 1.0 + self.stage_progress) / self.total_stages as f32)
                .clamp(0.0, 1.0);
        self.progress_bar.set_progress(total_progress);

        None
    }

    pub fn render(&self, batch: &mut DrawBatch, screen_w: f32, screen_h: f32) {
        let bar_w = 400.0;
        let bar_h = 16.0;
        let bar_x = (screen_w - bar_w) / 2.0;
        let bar_y = screen_h / 2.0 + 30.0;

        let mut pb = ProgressBar::new(bar_x, bar_y, bar_w, bar_h);
        pb.progress = self.progress_bar.progress;
        pb.render(batch);
    }
}
