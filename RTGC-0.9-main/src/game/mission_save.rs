//! Mission and Save System
//!
//! Implements:
//! - Mission definitions with objectives and rewards
//! - Mission progress tracking
//! - Save/Load game state with serialization
//! - Multiple save slots
//! - Auto-save functionality

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use tracing::{info, warn};

/// Mission objective types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionObjective {
    /// Reach a specific location
    ReachLocation { x: f32, y: f32, z: f32, radius: f32 },
    /// Collect items
    CollectItems { item_id: String, count: u32 },
    /// Eliminate targets
    EliminateTargets { target_type: String, count: u32 },
    /// Time trial
    TimeTrial { max_time_seconds: f32 },
    /// Survive for a duration
    Survive { duration_seconds: f32 },
    /// Complete a specific action
    CompleteAction { action_name: String },
}

/// Objective completion status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveStatus {
    pub objective: MissionObjective,
    pub current_progress: u32,
    pub required_progress: u32,
    pub completed: bool,
}

impl ObjectiveStatus {
    pub fn new(objective: MissionObjective) -> Self {
        let required = match &objective {
            MissionObjective::CollectItems { count, .. } => *count,
            MissionObjective::EliminateTargets { count, .. } => *count,
            MissionObjective::TimeTrial { max_time_seconds } => (*max_time_seconds * 100.0) as u32,
            MissionObjective::Survive { duration_seconds } => (*duration_seconds * 100.0) as u32,
            _ => 1,
        };

        Self {
            objective,
            current_progress: 0,
            required_progress: required,
            completed: false,
        }
    }

    pub fn update(&mut self, progress: u32) {
        self.current_progress = progress.min(self.required_progress);
        self.completed = self.current_progress >= self.required_progress;
    }

    pub fn is_complete(&self) -> bool {
        self.completed
    }

    pub fn get_completion_percentage(&self) -> f32 {
        if self.required_progress == 0 {
            return 100.0;
        }
        (self.current_progress as f32 / self.required_progress as f32) * 100.0
    }
}

/// Mission definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: String,
    pub name: String,
    pub description: String,
    pub difficulty: MissionDifficulty,
    pub objectives: Vec<ObjectiveStatus>,
    pub reward_xp: u32,
    pub reward_money: u32,
    pub reward_items: Vec<String>,
    pub prerequisites: Vec<String>, // IDs of missions that must be completed first
    pub is_main_story: bool,
    pub is_repeatable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionDifficulty {
    Easy,
    Normal,
    Hard,
    Expert,
}

impl Mission {
    pub fn new(id: String, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            difficulty: MissionDifficulty::Normal,
            objectives: Vec::new(),
            reward_xp: 100,
            reward_money: 500,
            reward_items: Vec::new(),
            prerequisites: Vec::new(),
            is_main_story: false,
            is_repeatable: false,
        }
    }

    pub fn add_objective(&mut self, objective: MissionObjective) {
        self.objectives.push(ObjectiveStatus::new(objective));
    }

    pub fn get_completion_percentage(&self) -> f32 {
        if self.objectives.is_empty() {
            return 0.0;
        }

        let total: f32 = self
            .objectives
            .iter()
            .map(|o| o.get_completion_percentage())
            .sum();
        total / self.objectives.len() as f32
    }

    pub fn is_complete(&self) -> bool {
        !self.objectives.is_empty() && self.objectives.iter().all(|o| o.is_complete())
    }

    pub fn update_objective(&mut self, objective_index: usize, progress: u32) {
        if let Some(obj) = self.objectives.get_mut(objective_index) {
            obj.update(progress);
        }
    }
}

/// Player progress data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProgress {
    pub player_id: String,
    pub level: u32,
    pub experience: u32,
    pub money: u32,
    pub inventory: HashMap<String, u32>,
    pub unlocked_vehicles: Vec<String>,
    pub unlocked_maps: Vec<String>,
    pub statistics: GameStatistics,
}

/// Mission progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionProgress {
    pub objectives: Vec<ObjectiveStatus>,
    pub current_objective_index: usize,
    pub is_complete: bool,
    pub world_state: WorldState,
}

/// Game statistics tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameStatistics {
    pub total_playtime_seconds: f64,
    pub distance_traveled_km: f64,
    pub missions_completed: u32,
    pub enemies_eliminated: u32,
    pub items_collected: u32,
    pub deaths: u32,
    pub saves_created: u32,
    pub achievements_unlocked: u32,
}

/// Save game data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveGame {
    pub save_slot: u32,
    pub save_name: String,
    pub timestamp: u64,
    pub playtime_seconds: f64,
    pub player: PlayerProgress,
    pub active_missions: Vec<Mission>,
    pub completed_missions: Vec<String>,
    pub world_state: WorldState,
    pub settings: GameSettings,
}

/// World state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub current_location: String,
    pub weather: String,
    pub time_of_day: f32, // 0.0 - 24.0
    pub vehicle_position: [f32; 3],
    pub vehicle_rotation: [f32; 4],
    pub vehicle_velocity: [f32; 3],
    pub vehicle_fuel: f32,
    pub vehicle_health: f32,
    pub npcs_alive: HashMap<String, bool>,
    pub destroyed_objects: Vec<String>,
    pub collected_items: Vec<String>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            current_location: String::new(),
            weather: String::from("Clear"),
            time_of_day: 12.0,
            vehicle_position: [0.0; 3],
            vehicle_rotation: [0.0, 0.0, 0.0, 1.0],
            vehicle_velocity: [0.0; 3],
            vehicle_fuel: 1.0,
            vehicle_health: 1.0,
            npcs_alive: HashMap::new(),
            destroyed_objects: Vec::new(),
            collected_items: Vec::new(),
        }
    }
}

/// Game settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub graphics_quality: GraphicsQuality,
    pub controls_inverted: bool,
    pub subtitles_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphicsQuality {
    Low,
    Medium,
    High,
    Ultra,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 1.0,
            graphics_quality: GraphicsQuality::High,
            controls_inverted: false,
            subtitles_enabled: true,
        }
    }
}

/// Mission and Save manager
pub struct MissionSaveManager {
    save_directory: PathBuf,
    max_save_slots: u32,
    autosave_interval_seconds: f64,
    last_autosave_time: f64,
}

impl MissionSaveManager {
    pub fn new(save_directory: PathBuf) -> Result<Self, String> {
        // Create save directory if it doesn't exist
        if !save_directory.exists() {
            fs::create_dir_all(&save_directory)
                .map_err(|e| format!("Cannot create save dir: {}", e))?;
        }

        Ok(Self {
            save_directory,
            max_save_slots: 10,
            autosave_interval_seconds: 300.0, // 5 minutes
            last_autosave_time: 0.0,
        })
    }

    /// Create a new save game
    pub fn create_save(
        &self,
        slot: u32,
        name: String,
        player: PlayerProgress,
    ) -> Result<SaveGame, String> {
        if slot >= self.max_save_slots {
            return Err(format!("Invalid save slot: {}", slot));
        }

        let save = SaveGame {
            save_slot: slot,
            save_name: name,
            timestamp: 0,
            playtime_seconds: player.statistics.total_playtime_seconds,
            player,
            active_missions: Vec::new(),
            completed_missions: Vec::new(),
            world_state: WorldState {
                current_location: "Starting Area".to_string(),
                weather: "Clear".to_string(),
                time_of_day: 12.0,
                vehicle_position: [0.0; 3],
                vehicle_rotation: [0.0, 0.0, 0.0, 1.0],
                vehicle_velocity: [0.0; 3],
                vehicle_fuel: 1.0,
                vehicle_health: 1.0,
                npcs_alive: HashMap::new(),
                destroyed_objects: Vec::new(),
                collected_items: Vec::new(),
            },
            settings: GameSettings::default(),
        };

        Ok(save)
    }

    /// Save game to file
    pub fn save_game(&self, save: &SaveGame) -> Result<(), String> {
        let filename = self
            .save_directory
            .join(format!("save_{}.json", save.save_slot));

        let file =
            File::create(&filename).map_err(|e| format!("Failed to create save file: {}", e))?;

        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, save)
            .map_err(|e| format!("Failed to serialize save data: {}", e))?;

        info!("Game saved to slot {}", save.save_slot);
        Ok(())
    }

    /// Load game from file
    pub fn load_game(&self, slot: u32) -> Result<SaveGame, String> {
        let filename = self.save_directory.join(format!("save_{}.json", slot));

        if !filename.exists() {
            return Err(format!("Save file not found for slot {}", slot));
        }

        let file = File::open(&filename).map_err(|e| format!("Failed to open save file: {}", e))?;

        let reader = BufReader::new(file);
        let save: SaveGame = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to deserialize save data: {}", e))?;

        info!("Game loaded from slot {}", slot);
        Ok(save)
    }

    /// Delete a save file
    pub fn delete_save(&self, slot: u32) -> Result<(), String> {
        let filename = self.save_directory.join(format!("save_{}.json", slot));

        if filename.exists() {
            fs::remove_file(&filename).map_err(|e| format!("Failed to delete save file: {}", e))?;
            info!("Save deleted from slot {}", slot);
        }

        Ok(())
    }

    /// List all available saves
    pub fn list_saves(&self) -> Vec<SaveGameInfo> {
        let mut saves = Vec::new();

        for slot in 0..self.max_save_slots {
            let filename = self.save_directory.join(format!("save_{}.json", slot));

            if filename.exists() {
                if let Ok(file) = File::open(&filename) {
                    let reader = BufReader::new(file);
                    if let Ok(save) = serde_json::from_reader::<_, SaveGame>(reader) {
                        saves.push(SaveGameInfo {
                            slot,
                            name: save.save_name,
                            timestamp: save.timestamp,
                            playtime: save.playtime_seconds,
                            player_level: save.player.level,
                        });
                    }
                }
            }
        }

        saves
    }

    /// Perform auto-save if enough time has passed
    pub fn try_autosave(&mut self, current_time: f64, save: &SaveGame) -> Result<bool, String> {
        if current_time - self.last_autosave_time >= self.autosave_interval_seconds {
            self.save_game(save)?;
            self.last_autosave_time = current_time;
            return Ok(true);
        }
        Ok(false)
    }

    /// Add mission to active missions
    pub fn add_mission(&mut self, save: &mut SaveGame, mission: Mission) {
        // Check prerequisites
        for prereq in &mission.prerequisites {
            if !save.completed_missions.contains(prereq) {
                warn!(
                    "Cannot add mission {}: prerequisite {} not completed",
                    mission.id, prereq
                );
                return;
            }
        }

        let mission_name = mission.name.clone();
        save.active_missions.push(mission);
        info!("Mission added: {}", mission_name);
    }

    /// Complete a mission
    pub fn complete_mission(&mut self, save: &mut SaveGame, mission_id: &str) -> Option<Mission> {
        if let Some(pos) = save.active_missions.iter().position(|m| m.id == mission_id) {
            let mission = save.active_missions.remove(pos);

            if mission.is_complete() {
                // Grant rewards
                save.player.experience += mission.reward_xp;
                save.player.money += mission.reward_money;

                // Add items to inventory
                for item in &mission.reward_items {
                    *save.player.inventory.entry(item.clone()).or_insert(0) += 1;
                }

                save.completed_missions.push(mission_id.to_string());
                save.player.statistics.missions_completed += 1;

                info!(
                    "Mission completed: {} - XP: {}, Money: {}",
                    mission.name, mission.reward_xp, mission.reward_money
                );
                Some(mission)
            } else {
                warn!(
                    "Cannot complete mission {}: objectives not finished",
                    mission_id
                );
                None
            }
        } else {
            None
        }
    }

    /// Update mission objective progress
    pub fn update_mission_objective(
        &mut self,
        save: &mut SaveGame,
        mission_id: &str,
        objective_index: usize,
        progress: u32,
    ) {
        if let Some(mission) = save.active_missions.iter_mut().find(|m| m.id == mission_id) {
            mission.update_objective(objective_index, progress);

            if mission.is_complete() {
                info!("Mission {} objectives complete!", mission.name);
            }
        }
    }
}

/// Summary information for save listing
#[derive(Debug, Clone)]
pub struct SaveGameInfo {
    pub slot: u32,
    pub name: String,
    pub timestamp: u64,
    pub playtime: f64,
    pub player_level: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_mission_creation() {
        let mut mission = Mission::new(
            "test_1".to_string(),
            "Test Mission".to_string(),
            "A test mission".to_string(),
        );
        mission.add_objective(MissionObjective::CollectItems {
            item_id: "apples".to_string(),
            count: 10,
        });

        assert_eq!(mission.get_completion_percentage(), 0.0);
        assert!(!mission.is_complete());

        mission.update_objective(0, 5);
        assert!((mission.get_completion_percentage() - 50.0).abs() < 0.1);

        mission.update_objective(0, 10);
        assert!((mission.get_completion_percentage() - 100.0).abs() < 0.1);
        assert!(mission.is_complete());
    }

    #[test]
    fn test_save_load_cycle() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let manager = MissionSaveManager::new(temp_dir.path().to_path_buf())
            .expect("Failed to create MissionSaveManager");

        let player = PlayerProgress {
            player_id: "player1".to_string(),
            level: 5,
            experience: 1500,
            money: 2500,
            inventory: HashMap::new(),
            unlocked_vehicles: Vec::new(),
            unlocked_maps: Vec::new(),
            statistics: GameStatistics::default(),
        };

        let save = manager
            .create_save(0, "Test Save".to_string(), player.clone())
            .expect("Failed to create save");
        manager.save_game(&save).expect("Failed to save game");

        let loaded = manager.load_game(0).expect("Failed to load game");
        assert_eq!(loaded.player.level, 5);
        assert_eq!(loaded.player.experience, 1500);
    }
}
