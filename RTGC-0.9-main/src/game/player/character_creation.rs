//! Character creation screen

use crate::game::player::{Appearance, Player};

/// Character creation state
#[derive(Debug, Clone)]
pub struct CharacterCreation {
    pub name_input: String,
    pub appearance: Appearance,
    pub selected_tab: CreationTab,

    // UI state
    pub skin_color_index: usize,
    pub hair_color_index: usize,
    pub face_variant: u8,
    pub hair_style: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CreationTab {
    Appearance,
    Skills,
    Confirm,
}

impl Default for CharacterCreation {
    fn default() -> Self {
        Self {
            name_input: String::from("Player"),
            appearance: Appearance::default(),
            selected_tab: CreationTab::Appearance,
            skin_color_index: 1,
            hair_color_index: 1,
            face_variant: 0,
            hair_style: 0,
        }
    }
}

impl CharacterCreation {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply current selection to appearance
    pub fn update_appearance(&mut self) {
        let skin_colors = Appearance::preset_skin_colors();
        let hair_colors = Appearance::preset_hair_colors();

        if self.skin_color_index < skin_colors.len() {
            self.appearance.skin_color = skin_colors[self.skin_color_index];
        }

        if self.hair_color_index < hair_colors.len() {
            self.appearance.hair_color = hair_colors[self.hair_color_index];
        }

        self.appearance.face_variant = self.face_variant;
        self.appearance.hair_style = self.hair_style;
    }

    /// Create player from character creation
    pub fn create_player(&self) -> Player {
        let mut player = Player::new(self.name_input.clone());
        player.is_male = self.appearance.is_male;
        player.height = self.appearance.height;
        player.skin_color = self.appearance.skin_color;
        player.face_variant = self.appearance.face_variant;
        player.hair_style = self.appearance.hair_style;
        player.hair_color = self.appearance.hair_color;
        player
    }

    /// Cycle to next skin color
    pub fn next_skin_color(&mut self) {
        let colors = Appearance::preset_skin_colors();
        self.skin_color_index = (self.skin_color_index + 1) % colors.len();
        self.update_appearance();
    }

    /// Cycle to previous skin color
    pub fn prev_skin_color(&mut self) {
        let colors = Appearance::preset_skin_colors();
        self.skin_color_index = if self.skin_color_index == 0 {
            colors.len() - 1
        } else {
            self.skin_color_index - 1
        };
        self.update_appearance();
    }

    /// Cycle to next hair color
    pub fn next_hair_color(&mut self) {
        let colors = Appearance::preset_hair_colors();
        self.hair_color_index = (self.hair_color_index + 1) % colors.len();
        self.update_appearance();
    }

    /// Cycle to previous hair color
    pub fn prev_hair_color(&mut self) {
        let colors = Appearance::preset_hair_colors();
        self.hair_color_index = if self.hair_color_index == 0 {
            colors.len() - 1
        } else {
            self.hair_color_index - 1
        };
        self.update_appearance();
    }

    /// Toggle gender
    pub fn toggle_gender(&mut self) {
        self.appearance.is_male = !self.appearance.is_male;
    }

    /// Increase height
    pub fn increase_height(&mut self) {
        self.appearance.height = (self.appearance.height + 0.05).min(2.0);
    }

    /// Decrease height
    pub fn decrease_height(&mut self) {
        self.appearance.height = (self.appearance.height - 0.05).max(1.6);
    }

    /// Next face variant
    pub fn next_face(&mut self) {
        self.face_variant = (self.face_variant + 1) % 8;
        self.appearance.face_variant = self.face_variant;
    }

    /// Previous face variant
    pub fn prev_face(&mut self) {
        self.face_variant = if self.face_variant == 0 {
            7
        } else {
            self.face_variant - 1
        };
        self.appearance.face_variant = self.face_variant;
    }

    /// Next hair style
    pub fn next_hair_style(&mut self) {
        self.hair_style = (self.hair_style + 1) % 10;
        self.appearance.hair_style = self.hair_style;
    }

    /// Previous hair style
    pub fn prev_hair_style(&mut self) {
        self.hair_style = if self.hair_style == 0 {
            9
        } else {
            self.hair_style - 1
        };
        self.appearance.hair_style = self.hair_style;
    }

    /// Randomize appearance
    pub fn randomize(&mut self) {
        self.appearance = Appearance::random();
        self.skin_color_index = 1;
        self.hair_color_index = 1;
        self.face_variant = self.appearance.face_variant;
        self.hair_style = self.appearance.hair_style;
    }
}
