use crate::game::scene::{Scene, SceneType};
use crate::graphics::renderer::Renderer;
use std::any::Any;

pub struct LoadingScene {
    pub progress: f32,
    pub loading_complete: bool,
}

impl LoadingScene {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            loading_complete: false,
        }
    }
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.min(1.0).max(0.0);
        if self.progress >= 1.0 {
            self.loading_complete = true;
        }
    }
}

impl Default for LoadingScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for LoadingScene {
    fn scene_type(&self) -> SceneType {
        SceneType::Loading
    }

    fn on_enter(&mut self) {
        tracing::info!("Entering Loading Screen");
        self.progress = 0.0;
        self.loading_complete = false;
    }

    fn on_exit(&mut self) {
        tracing::info!("Exiting Loading Screen");
    }

    fn update(&mut self, delta_time: f32) {
        if !self.loading_complete && self.progress < 1.0 {
            let progress_increment = delta_time * 0.5;
            self.progress = (self.progress + progress_increment).min(1.0);
            
            if self.progress >= 1.0 {
                self.loading_complete = true;
                tracing::info!("Loading complete");
            }
        }
    }

    fn render(&mut self, _renderer: &mut Renderer) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Loading"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}