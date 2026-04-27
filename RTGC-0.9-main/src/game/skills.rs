//! Skills System - Player skill progression with ranks 1-12
//! Implements 20+ skills with XP gain, mastery, and gameplay effects

/// Skill structure with rank, mastery, and total hours
#[derive(Debug, Clone)]
pub struct Skill {
    /// Rank from 1 to 12
    pub rank: u8,
    /// Mastery within current rank (0.0 - 1.0)
    pub mastery: f32,
    /// Total hours spent on this skill
    pub total_hours: f32,
}

impl Skill {
    /// Create a new skill with initial values
    pub fn new(rank: u8, mastery: f32, total_hours: f32) -> Self {
        Self {
            rank: rank.clamp(1, 12),
            mastery: mastery.clamp(0.0, 1.0),
            total_hours,
        }
    }

    /// Create a skill at rank 1 with zero progress
    pub fn default() -> Self {
        Self::new(1, 0.0, 0.0)
    }

    /// Gain XP based on hours spent and difficulty
    /// difficulty: 0.5 (easy) to 2.0 (hard)
    pub fn gain_xp(&mut self, hours: f32, difficulty: f32) {
        if hours <= 0.0 {
            return;
        }

        let effective_hours = hours * difficulty.clamp(0.5, 2.0);
        self.total_hours += effective_hours;

        // Hours needed per rank increase (exponential growth)
        // Rank 1->2: 10 hours, Rank 11->12: 10000 hours
        let hours_per_rank = 10.0 * 2.0_f32.powf((self.rank - 1) as f32);

        let mastery_gain = effective_hours / hours_per_rank;
        self.mastery += mastery_gain;

        // Level up if mastery >= 1.0
        while self.mastery >= 1.0 && self.rank < 12 {
            self.mastery -= 1.0;
            self.rank += 1;
        }

        // Cap at rank 12
        if self.rank >= 12 {
            self.rank = 12;
            self.mastery = self.mastery.min(1.0);
        }
    }

    /// Check if skill can be trained via self-study (max rank 4)
    pub fn can_self_study(&self) -> bool {
        self.rank < 4
    }

    /// Get the multiplier for this skill rank (0.3x to 25x)
    pub fn get_multiplier(&self) -> f32 {
        match self.rank {
            1 => 0.3,
            2 => 0.5,
            3 => 0.7,
            4 => 1.0,
            5 => 1.3,
            6 => 1.6,
            7 => 2.0,
            8 => 3.0,
            9 => 5.0,
            10 => 8.0,
            11 => 12.0,
            12 => 25.0,
            _ => 1.0,
        }
    }
}

/// All player skills (20+ skills across categories)
pub struct PlayerSkills {
    // Technical skills
    pub mechanics: Skill,     // Engine repair, maintenance
    pub electrics: Skill,     // Electrical systems, wiring
    pub welding: Skill,       // Metal welding, fabrication
    pub construction: Skill,  // Building construction
    pub road_building: Skill, // Road construction and repair

    // Vehicle operation
    pub driving: Skill,  // Cars, trucks
    pub tracked: Skill,  // Tracked vehicles (tanks, bulldozers)
    pub piloting: Skill, // Helicopters
    pub flying: Skill,   // Airplanes
    pub crane: Skill,    // Crane operation

    // Resource extraction
    pub geology: Skill,  // Resource detection, analysis
    pub drilling: Skill, // Oil/gas drilling
    pub logging: Skill,  // Tree harvesting, lumber
    pub mining: Skill,   // Mining operations

    // Business & logistics
    pub business: Skill,   // Company management, contracts
    pub logistics: Skill,  // Transport efficiency, route planning
    pub trading: Skill,    // Market trading, negotiation
    pub navigation: Skill, // Map reading, route finding

    // Personal skills
    pub medicine: Skill, // First aid, healing
    pub fitness: Skill,  // Stamina, running speed
}

impl PlayerSkills {
    /// Create a new player with all skills at rank 1
    pub fn new() -> Self {
        Self {
            mechanics: Skill::default(),
            electrics: Skill::default(),
            welding: Skill::default(),
            construction: Skill::default(),
            road_building: Skill::default(),
            driving: Skill::default(),
            tracked: Skill::default(),
            piloting: Skill::default(),
            flying: Skill::default(),
            crane: Skill::default(),
            geology: Skill::default(),
            drilling: Skill::default(),
            logging: Skill::default(),
            mining: Skill::default(),
            business: Skill::default(),
            logistics: Skill::default(),
            trading: Skill::default(),
            navigation: Skill::default(),
            medicine: Skill::default(),
            fitness: Skill::default(),
        }
    }

    /// Create skills based on education (university/college)
    pub fn from_education(specialty: &str) -> Self {
        let mut skills = Self::new();

        match specialty.to_lowercase().as_str() {
            "automotive_engineering" => {
                skills.mechanics = Skill::new(4, 0.3, 500.0);
                skills.driving = Skill::new(3, 0.5, 200.0);
                skills.electrics = Skill::new(3, 0.2, 300.0);
            }
            "civil_engineering" => {
                skills.construction = Skill::new(4, 0.4, 600.0);
                skills.road_building = Skill::new(3, 0.3, 400.0);
                skills.welding = Skill::new(2, 0.5, 150.0);
            }
            "aviation" => {
                skills.piloting = Skill::new(4, 0.2, 800.0);
                skills.navigation = Skill::new(3, 0.4, 300.0);
                skills.mechanics = Skill::new(2, 0.3, 200.0);
            }
            "geology" => {
                skills.geology = Skill::new(4, 0.5, 700.0);
                skills.drilling = Skill::new(3, 0.3, 400.0);
                skills.mining = Skill::new(2, 0.4, 250.0);
            }
            "business" => {
                skills.business = Skill::new(4, 0.3, 500.0);
                skills.logistics = Skill::new(3, 0.4, 350.0);
                skills.trading = Skill::new(3, 0.2, 200.0);
            }
            "medicine" => {
                skills.medicine = Skill::new(4, 0.5, 800.0);
                skills.fitness = Skill::new(2, 0.3, 150.0);
            }
            _ => {
                // Default - basic driving and fitness
                skills.driving = Skill::new(2, 0.5, 100.0);
                skills.fitness = Skill::new(2, 0.3, 100.0);
            }
        }

        skills
    }

    /// Check if player can repair engine (mechanics >= 2)
    pub fn can_repair_engine(&self) -> bool {
        self.mechanics.rank >= 2
    }

    /// Check if player can pilot helicopter (piloting >= 4)
    pub fn can_pilot_helicopter(&self) -> bool {
        self.piloting.rank >= 4
    }

    /// Check if player can open individual entrepreneur (business >= 3)
    pub fn can_open_ie(&self) -> bool {
        self.business.rank >= 3
    }

    /// Check if player can open LLC (business >= 5)
    pub fn can_open_llc(&self) -> bool {
        self.business.rank >= 5
    }

    /// Check if player can see resource type when surveying (geology >= 4)
    pub fn can_identify_resources(&self) -> bool {
        self.geology.rank >= 4
    }

    /// Get contract payment bonus based on logistics skill
    /// logistics >= 6 → +52% bonus
    pub fn get_logistics_bonus(&self) -> f32 {
        match self.logistics.rank {
            0..=3 => 0.0,
            4 => 0.15,
            5 => 0.30,
            6 => 0.52,
            7 => 0.70,
            8 => 0.90,
            9..=12 => 1.0,
            _ => 0.0,
        }
    }

    /// Get running speed bonus based on fitness
    pub fn get_fitness_speed_bonus(&self) -> f32 {
        match self.fitness.rank {
            0..=2 => 1.0,
            3 => 1.1,
            4 => 1.2,
            5 => 1.3,
            6 => 1.4,
            7 => 1.5,
            8 => 1.6,
            9..=12 => 1.75,
            _ => 1.0,
        }
    }

    /// Get stamina regeneration bonus based on fitness
    pub fn get_stamina_regen_bonus(&self) -> f32 {
        match self.fitness.rank {
            0..=2 => 1.0,
            3 => 1.15,
            4 => 1.3,
            5 => 1.45,
            6 => 1.6,
            7 => 1.75,
            8 => 1.9,
            9..=12 => 2.0,
            _ => 1.0,
        }
    }

    /// Check if player can operate crane (crane >= 3 for excavator, >= 4 for crane)
    pub fn can_operate_excavator(&self) -> bool {
        self.crane.rank >= 3
    }

    pub fn can_operate_crane(&self) -> bool {
        self.crane.rank >= 4
    }

    /// Check if player can fly airplane (flying >= 4)
    pub fn can_fly_airplane(&self) -> bool {
        self.flying.rank >= 4
    }

    /// Check if player can drive tracked vehicles (tracked >= 3)
    pub fn can_drive_tracked(&self) -> bool {
        self.tracked.rank >= 3
    }

    /// Get healing effectiveness based on medicine skill
    pub fn get_medicine_effectiveness(&self) -> f32 {
        match self.medicine.rank {
            0..=1 => 0.5,
            2 => 0.65,
            3 => 0.8,
            4 => 1.0,
            5 => 1.2,
            6 => 1.4,
            7..=12 => 1.5,
            _ => 0.5,
        }
    }

    /// Get trade price bonus based on trading skill
    pub fn get_trading_discount(&self) -> f32 {
        match self.trading.rank {
            0..=2 => 0.0,
            3 => 0.05,
            4 => 0.10,
            5 => 0.15,
            6 => 0.20,
            7..=12 => 0.25,
            _ => 0.0,
        }
    }

    /// Apply self-study limitation (max rank 4)
    pub fn self_study(&mut self, skill_type: SkillType, hours: f32, difficulty: f32) {
        let skill = self.get_skill_mut(skill_type);
        if skill.can_self_study() {
            skill.gain_xp(hours, difficulty);
        }
        // If rank >= 4, self-study has no effect
    }

    /// Get skill by type
    pub fn get_skill(&self, skill_type: SkillType) -> &Skill {
        match skill_type {
            SkillType::Mechanics => &self.mechanics,
            SkillType::Electrics => &self.electrics,
            SkillType::Welding => &self.welding,
            SkillType::Construction => &self.construction,
            SkillType::RoadBuilding => &self.road_building,
            SkillType::Driving => &self.driving,
            SkillType::Tracked => &self.tracked,
            SkillType::Piloting => &self.piloting,
            SkillType::Flying => &self.flying,
            SkillType::Crane => &self.crane,
            SkillType::Geology => &self.geology,
            SkillType::Drilling => &self.drilling,
            SkillType::Logging => &self.logging,
            SkillType::Mining => &self.mining,
            SkillType::Business => &self.business,
            SkillType::Logistics => &self.logistics,
            SkillType::Trading => &self.trading,
            SkillType::Navigation => &self.navigation,
            SkillType::Medicine => &self.medicine,
            SkillType::Fitness => &self.fitness,
        }
    }

    /// Get mutable skill by type
    pub fn get_skill_mut(&mut self, skill_type: SkillType) -> &mut Skill {
        match skill_type {
            SkillType::Mechanics => &mut self.mechanics,
            SkillType::Electrics => &mut self.electrics,
            SkillType::Welding => &mut self.welding,
            SkillType::Construction => &mut self.construction,
            SkillType::RoadBuilding => &mut self.road_building,
            SkillType::Driving => &mut self.driving,
            SkillType::Tracked => &mut self.tracked,
            SkillType::Piloting => &mut self.piloting,
            SkillType::Flying => &mut self.flying,
            SkillType::Crane => &mut self.crane,
            SkillType::Geology => &mut self.geology,
            SkillType::Drilling => &mut self.drilling,
            SkillType::Logging => &mut self.logging,
            SkillType::Mining => &mut self.mining,
            SkillType::Business => &mut self.business,
            SkillType::Logistics => &mut self.logistics,
            SkillType::Trading => &mut self.trading,
            SkillType::Navigation => &mut self.navigation,
            SkillType::Medicine => &mut self.medicine,
            SkillType::Fitness => &mut self.fitness,
        }
    }
}

/// Skill type enum for indexing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkillType {
    Mechanics,
    Electrics,
    Welding,
    Construction,
    RoadBuilding,
    Driving,
    Tracked,
    Piloting,
    Flying,
    Crane,
    Geology,
    Drilling,
    Logging,
    Mining,
    Business,
    Logistics,
    Trading,
    Navigation,
    Medicine,
    Fitness,
}

impl Default for PlayerSkills {
    fn default() -> Self {
        Self::new()
    }
}
