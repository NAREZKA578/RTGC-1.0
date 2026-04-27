use crate::game::scene::{Scene, SceneType};
use crate::graphics::renderer::Renderer;
use std::any::Any;

pub struct PauseScene;

impl PauseScene {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PauseScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for PauseScene {
    fn scene_type(&self) -> SceneType {
        SceneType::Pause
    }

    fn on_enter(&mut self) {
        tracing::info!("Paused");
    }

    fn on_exit(&mut self) {
        tracing::info!("Resumed");
    }

    fn render(&mut self, _renderer: &mut Renderer) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Pause"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}