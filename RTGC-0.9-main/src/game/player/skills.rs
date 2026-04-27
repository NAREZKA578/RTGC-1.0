//! Player skills system

use crate::game::save::{PlayerSkillsData, SkillData};

/// Player skills data
#[derive(Debug, Clone)]
pub struct Skills {
    pub driving: f32,       // 0.0..100.0 - affects vehicle handling
    pub piloting: f32,      // 0.0..100.0 - affects helicopter control
    pub mechanics: f32,     // 0.0..100.0 - affects repair speed
    pub strength: f32,      // 0.0..100.0 - affects carrying capacity
    pub stamina_skill: f32, // 0.0..100.0 - affects stamina regeneration
}

impl Default for Skills {
    fn default() -> Self {
        Self {
            driving: 10.0,
            piloting: 0.0,
            mechanics: 5.0,
            strength: 10.0,
            stamina_skill: 10.0,
        }
    }
}

impl Skills {
    /// Create skills from save data
    pub fn from_save_data(data: &PlayerSkillsData) -> Self {
        Self {
            driving: data.driving.rank as f32 * 10.0 + data.driving.mastery,
            piloting: data.piloting.rank as f32 * 10.0 + data.piloting.mastery,
            mechanics: data.mechanics.rank as f32 * 10.0 + data.mechanics.mastery,
            strength: data.strength,
            stamina_skill: data.stamina,
        }
    }

    /// Convert to save data
    pub fn to_save_data(&self) -> PlayerSkillsData {
        let rank = (self.driving / 10.0) as u8;
        let mastery = self.driving % 10.0;
        let piloting_rank = (self.piloting / 10.0) as u8;
        let piloting_mastery = self.piloting % 10.0;
        let mechanics_rank = (self.mechanics / 10.0) as u8;
        let mechanics_mastery = self.mechanics % 10.0;

        PlayerSkillsData {
            strength: self.strength,
            stamina: self.stamina_skill,
            mechanics: SkillData {
                rank: mechanics_rank,
                mastery: mechanics_mastery,
                total_hours: 0.0,
            },
            electrics: SkillData::default(),
            welding: SkillData::default(),
            construction: SkillData::default(),
            road_building: SkillData::default(),
            driving: SkillData {
                rank,
                mastery,
                total_hours: 0.0,
            },
            tracked: SkillData::default(),
            piloting: SkillData {
                rank: piloting_rank,
                mastery: piloting_mastery,
                total_hours: 0.0,
            },
            flying: SkillData::default(),
            crane: SkillData::default(),
            geology: SkillData::default(),
            drilling: SkillData::default(),
            logging: SkillData::default(),
            mining: SkillData::default(),
            business: SkillData::default(),
            logistics: SkillData::default(),
            trading: SkillData::default(),
            navigation: SkillData::default(),
            medicine: SkillData::default(),
            fitness: SkillData::default(),
        }
    }

    /// Get driving bonus (affects vehicle grip, fuel efficiency)
    pub fn get_driving_bonus(&self) -> f32 {
        self.driving / 100.0 * 0.2 // Up to 20% bonus
    }

    /// Get piloting bonus (affects helicopter stability)
    pub fn get_piloting_bonus(&self) -> f32 {
        self.piloting / 100.0 * 0.3 // Up to 30% bonus
    }

    /// Get mechanics bonus (affects repair speed)
    pub fn get_mechanics_bonus(&self) -> f32 {
        self.mechanics / 100.0 * 0.5 // Up to 50% faster repairs
    }

    /// Add experience to a skill
    pub fn add_experience(&mut self, skill: SkillType, amount: f32) {
        let current = match skill {
            SkillType::Driving => &mut self.driving,
            SkillType::Piloting => &mut self.piloting,
            SkillType::Mechanics => &mut self.mechanics,
            SkillType::Strength => &mut self.strength,
            SkillType::Stamina => &mut self.stamina_skill,
        };

        *current = (*current + amount).min(100.0);
    }

    /// Check if skill can be increased
    pub fn can_increase(&self, skill: SkillType) -> bool {
        let current = match skill {
            SkillType::Driving => self.driving,
            SkillType::Piloting => self.piloting,
            SkillType::Mechanics => self.mechanics,
            SkillType::Strength => self.strength,
            SkillType::Stamina => self.stamina_skill,
        };
        current < 100.0
    }
}

/// Skill types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkillType {
    Driving,
    Piloting,
    Mechanics,
    Strength,
    Stamina,
}
