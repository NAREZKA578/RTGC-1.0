//! Main Menu Scene

use super::super::main_menu::MainMenu;
use super::super::scene::{Scene, SceneType};
use std::any::Any;

pub struct MainMenuScene {
    name: String,
    main_menu: MainMenu,
}

impl MainMenuScene {
    pub fn new() -> Self {
        Self {
            name: "Main Menu".to_string(),
            main_menu: MainMenu::new(),
        }
    }

    pub fn get_main_menu(&self) -> &MainMenu {
        &self.main_menu
    }

    pub fn get_main_menu_mut(&mut self) -> &mut MainMenu {
        &mut self.main_menu
    }
}

impl Default for MainMenuScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for MainMenuScene {
    fn scene_type(&self) -> SceneType {
        SceneType::MainMenu
    }

    fn on_enter(&mut self) {
        tracing::info!("Entering Main Menu");
    }

    fn on_exit(&mut self) {
        tracing::info!("Exiting Main Menu");
    }

    fn update(&mut self, delta_time: f32) {
        // Обновляем главное меню
        self.main_menu.update(delta_time);
    }

fn render(
        &mut self,
        renderer: &mut crate::graphics::renderer::Renderer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // For now, just skip rendering - UI rendering needs more work to integrate with the scene system
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
