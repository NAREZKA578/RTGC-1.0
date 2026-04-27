use crate::game::scene::{Scene, SceneType};
use crate::graphics::renderer::Renderer;
use std::any::Any;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gender {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CreationStep {
    Welcome,
    Gender,
    Appearance,
    Background,
    StartLocation,
    Summary,
}

pub struct EducationOption {
    pub name: String,
    pub starting_capital_rub: f32,
}

pub struct StartLocation {
    pub name: String,
    pub position: [f32; 3],
    pub description: String,
}

pub struct CharacterCreationData {
    pub gender: Gender,
    pub skin_tone: u32,
    pub hair_color: u32,
    pub player_name: String,
}

impl Default for CharacterCreationData {
    fn default() -> Self {
        Self {
            gender: Gender::Male,
            skin_tone: 0,
            hair_color: 0,
            player_name: "Player".to_string(),
        }
    }
}

pub struct CharacterCreationManager {
    pub data: CharacterCreationData,
    pub current_step: CreationStep,
}

impl CharacterCreationManager {
    pub fn new() -> Self {
        Self {
            data: CharacterCreationData::default(),
            current_step: CreationStep::Welcome,
        }
    }
}

impl Default for CharacterCreationManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CharacterCreationScene {
    pub manager: CharacterCreationManager,
    pub page: u32,
}

impl CharacterCreationScene {
    pub fn new() -> Self {
        Self {
            manager: CharacterCreationManager::new(),
            page: 0,
        }
    }
}

impl Default for CharacterCreationScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for CharacterCreationScene {
    fn scene_type(&self) -> SceneType {
        SceneType::CharacterCreation
    }

    fn on_enter(&mut self) {
        tracing::info!("Character Creation");
    }

    fn render(&mut self, _renderer: &mut Renderer) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn name(&self) -> &str {
        "CharacterCreation"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct CharacterCreation;

impl CharacterCreation {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CharacterCreation {
    fn default() -> Self {
        Self::new()
    }
}