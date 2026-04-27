//! Save System - Game save/load functionality
//! Saves only in "safe" locations: beds, vehicle bunks, tents, owned properties

use crate::game::player::{CameraMode, Player, PlayerState};
use serde::{Deserialize, Serialize};
use serde_json;

/// Maximum number of save slots
pub const MAX_SAVE_SLOTS: usize = 10;

/// Save game metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    /// Slot number (0-9)
    pub slot: u8,
    /// Player name
    pub player_name: String,
    /// In-game time when saved
    pub game_time_hours: f32,
    /// Real-world timestamp
    pub timestamp: u64,
    /// Location name where saved
    pub location_name: String,
    /// Position in world (E0117 fix: use [f32; 3] instead of Vector3)
    pub position: [f32; 3],
    /// Money
    pub money_rub: f64,
    /// Playtime in hours
    pub playtime_hours: f32,
}

/// Complete save data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    /// Metadata
    pub metadata: SaveMetadata,
    /// Player data
    pub player: PlayerData,
    /// World state
    pub world_state: WorldStateData,
    /// Vehicle states (if any)
    pub vehicles: Vec<VehicleSaveData>,
}

/// Serialized player data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    /// Name
    pub name: String,
    /// Gender (true = male)
    pub is_male: bool,
    /// Height in meters
    pub height: f32,
    /// Skin color [r, g, b]
    pub skin_color: [f32; 3],
    /// Face variant
    pub face_variant: u8,
    /// Hair style
    pub hair_style: u8,
    /// Hair color [r, g, b]
    pub hair_color: [f32; 3],
    /// Skills
    pub skills: PlayerSkillsData,
    /// Money
    pub money: PlayerMoneyData,
    /// Inventory
    pub inventory: Vec<InventoryItemData>,
    /// Inventory weight
    pub inventory_weight: f32,
    /// Position (E0117 fix: use [f32; 3] instead of Vector3)
    pub position: [f32; 3],
    /// Rotation (quaternion as [x, y, z, w])
    pub rotation: [f32; 4],
    /// State (OnFoot / InVehicle)
    pub state: PlayerStateData,
    /// Camera mode
    pub camera_mode: CameraModeData,
    /// Stamina
    pub stamina: f32,
}

/// Serialized skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSkillsData {
    pub strength: f32,
    pub stamina: f32,
    pub mechanics: SkillData,
    pub electrics: SkillData,
    pub welding: SkillData,
    pub construction: SkillData,
    pub road_building: SkillData,
    pub driving: SkillData,
    pub tracked: SkillData,
    pub piloting: SkillData,
    pub flying: SkillData,
    pub crane: SkillData,
    pub geology: SkillData,
    pub drilling: SkillData,
    pub logging: SkillData,
    pub mining: SkillData,
    pub business: SkillData,
    pub logistics: SkillData,
    pub trading: SkillData,
    pub navigation: SkillData,
    pub medicine: SkillData,
    pub fitness: SkillData,
}

/// Serialized single skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillData {
    pub rank: u8,
    pub mastery: f32,
    pub total_hours: f32,
}

impl Default for SkillData {
    fn default() -> Self {
        Self {
            rank: 0,
            mastery: 0.0,
            total_hours: 0.0,
        }
    }
}

impl Default for PlayerSkillsData {
    fn default() -> Self {
        Self {
            strength: 1.0,
            stamina: 1.0,
            mechanics: SkillData::default(),
            electrics: SkillData::default(),
            welding: SkillData::default(),
            construction: SkillData::default(),
            road_building: SkillData::default(),
            driving: SkillData::default(),
            tracked: SkillData::default(),
            piloting: SkillData::default(),
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
}

impl PlayerSkillsData {
    /// Create from education specialty
    pub fn from_education(specialty_id: &str) -> Self {
        let mut skills = Self::default();

        match specialty_id {
            "mechanic" => {
                skills.mechanics.rank = 3;
                skills.mechanics.mastery = 0.5;
            }
            "electrician" => {
                skills.electrics.rank = 3;
                skills.electrics.mastery = 0.5;
            }
            "driver" => {
                skills.driving.rank = 3;
                skills.driving.mastery = 0.5;
            }
            "pilot" => {
                skills.piloting.rank = 3;
                skills.piloting.mastery = 0.5;
            }
            _ => {}
        }

        skills
    }
}

/// Serialized money
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMoneyData {
    pub rub: f64,
    pub cny: f64,
    pub usd: f64,
}

impl Default for PlayerMoneyData {
    fn default() -> Self {
        Self {
            rub: 0.0,
            cny: 0.0,
            usd: 0.0,
        }
    }
}

/// Serialized inventory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItemData {
    pub name: String,
    pub weight: f32,
    pub item_type: String,
    pub quantity: u32,
}

/// Serialized player state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerStateData {
    OnFoot,
    InVehicle {
        vehicle_index: usize,
        vehicle_id: u64,
        seat_index: usize,
    },
}

/// Serialized camera mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraModeData {
    FirstPerson,
    ThirdPerson { distance: f32, yaw: f32, pitch: f32 },
}

/// Serialized world state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldStateData {
    /// Game time in hours
    pub time_hours: f32,
    /// Day number
    pub day: u32,
    /// Weather type
    pub weather: String,
    /// Weather intensity
    pub weather_intensity: f32,
    /// Discovered locations
    pub discovered_locations: Vec<String>,
    /// Completed missions
    pub completed_missions: Vec<u64>,
    /// Active missions
    pub active_missions: Vec<MissionSaveData>,
    /// Reputation with factions
    pub reputation: std::collections::HashMap<String, i32>,
}

/// Serialized mission data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionSaveData {
    pub id: u64,
    pub name: String,
    pub progress: f32,
    pub is_completed: bool,
}

/// Serialized vehicle data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleSaveData {
    /// Vehicle ID/name
    pub name: String,
    /// Position (E0117 fix: use [f32; 3] instead of Vector3)
    pub position: [f32; 3],
    /// Rotation
    pub rotation: [f32; 4],
    /// Velocity (E0117 fix: use [f32; 3] instead of Vector3)
    pub velocity: [f32; 3],
    /// Angular velocity (E0117 fix: use [f32; 3] instead of Vector3)
    pub angular_velocity: [f32; 3],
    /// Engine integrity
    pub engine_integrity: f32,
    /// Transmission integrity
    pub transmission_integrity: f32,
    /// Drivetrain integrity
    pub drivetrain_integrity: f32,
    /// Suspension integrity
    pub suspension_integrity: f32,
    /// Brake integrity
    pub brake_integrity: f32,
    /// Tire integrity (4 wheels)
    pub tire_integrity: [f32; 4],
    /// Fuel level (0.0 - 1.0)
    pub fuel_level: f32,
    /// Mileage in km
    pub mileage_km: f32,
    /// Color index
    pub color_index: usize,
    /// Is engine running
    pub engine_running: bool,
    /// Current gear
    pub current_gear: i8,
}

/// Save location types (where saving is allowed)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaveLocationType {
    /// Bed in house/apartment/hotel
    Bed,
    /// Bunk in truck/vehicle
    VehicleBunk,
    /// Large tent/camp
    Tent,
    /// Owned property (house/apartment)
    OwnedProperty,
}

impl std::fmt::Display for SaveLocationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveLocationType::Bed => write!(f, "Кровать"),
            SaveLocationType::VehicleBunk => write!(f, "Спальник в машине"),
            SaveLocationType::Tent => write!(f, "Палатка"),
            SaveLocationType::OwnedProperty => write!(f, "Собственная недвижимость"),
        }
    }
}

/// Save system manager
#[derive(Clone)]
pub struct SaveSystem {
    /// Save directory path
    pub save_directory: String,
    /// Available saves (metadata only)
    pub saves: Vec<Option<SaveMetadata>>,
    /// Current save slot
    pub current_slot: Option<u8>,
}

impl SaveSystem {
    /// Create new save system
    pub fn new(save_directory: &str) -> Self {
        Self {
            save_directory: save_directory.to_string(),
            saves: vec![None; MAX_SAVE_SLOTS],
            current_slot: None,
        }
    }

    /// Check if can save at location
    pub fn can_save_at(&self, location_type: SaveLocationType) -> bool {
        // All safe locations allow saving
        matches!(
            location_type,
            SaveLocationType::Bed
                | SaveLocationType::VehicleBunk
                | SaveLocationType::Tent
                | SaveLocationType::OwnedProperty
        )
    }

    /// Get save file path for slot
    pub fn get_save_path(&self, slot: u8) -> String {
        format!("{}/save_{}.bin", self.save_directory, slot)
    }

    /// Get metadata file path for slot
    pub fn get_metadata_path(&self, slot: u8) -> String {
        format!("{}/save_{}.meta", self.save_directory, slot)
    }

    /// Save game to slot
    pub fn save_game(&mut self, slot: u8, save_data: &SaveData) -> Result<(), String> {
        if slot >= MAX_SAVE_SLOTS as u8 {
            return Err(format!("Invalid save slot: {}", slot));
        }

        // Create save directory if not exists
        std::fs::create_dir_all(&self.save_directory)
            .map_err(|e| format!("Failed to create save directory: {}", e))?;

        // Serialize save data
        let save_bytes = bincode::serialize(save_data)
            .map_err(|e| format!("Failed to serialize save data: {}", e))?;

        // Write save file
        std::fs::write(self.get_save_path(slot), save_bytes)
            .map_err(|e| format!("Failed to write save file: {}", e))?;

        // Write metadata file (JSON for human readability)
        let meta_json = serde_json::to_string_pretty(&save_data.metadata)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
        std::fs::write(self.get_metadata_path(slot), meta_json)
            .map_err(|e| format!("Failed to write metadata file: {}", e))?;

        // Update metadata
        self.saves[slot as usize] = Some(save_data.metadata.clone());
        self.current_slot = Some(slot);

        Ok(())
    }

    /// Load game from slot
    pub fn load_game(&mut self, slot: u8) -> Result<SaveData, String> {
        if slot >= MAX_SAVE_SLOTS as u8 {
            return Err(format!("Invalid save slot: {}", slot));
        }

        // Read save file
        let save_bytes = std::fs::read(self.get_save_path(slot))
            .map_err(|e| format!("Failed to read save file: {}", e))?;

        // Deserialize save data
        let save_data: SaveData = bincode::deserialize(&save_bytes)
            .map_err(|e| format!("Failed to deserialize save data: {}", e))?;

        // Update metadata
        self.saves[slot as usize] = Some(save_data.metadata.clone());
        self.current_slot = Some(slot);

        Ok(save_data)
    }

    /// Delete save from slot
    pub fn delete_save(&mut self, slot: u8) -> Result<(), String> {
        if slot >= MAX_SAVE_SLOTS as u8 {
            return Err(format!("Invalid save slot: {}", slot));
        }

        // Remove files
        let _ = std::fs::remove_file(self.get_save_path(slot));
        let _ = std::fs::remove_file(self.get_metadata_path(slot));

        // Update metadata
        self.saves[slot as usize] = None;

        if self.current_slot == Some(slot) {
            self.current_slot = None;
        }

        Ok(())
    }

    /// Check if slot has save
    pub fn has_save(&self, slot: u8) -> bool {
        slot < MAX_SAVE_SLOTS as u8 && self.saves[slot as usize].is_some()
    }

    /// Get metadata for slot
    pub fn get_metadata(&self, slot: u8) -> Option<&SaveMetadata> {
        if slot < MAX_SAVE_SLOTS as u8 {
            self.saves[slot as usize].as_ref()
        } else {
            None
        }
    }

    /// List all saves
    pub fn list_saves(&self) -> Vec<(u8, &SaveMetadata)> {
        self.saves
            .iter()
            .enumerate()
            .filter_map(|(i, meta)| meta.as_ref().map(|m| (i as u8, m)))
            .collect()
    }

    /// Convert Player to PlayerData for saving
    pub fn player_to_save_data(
        player: &Player,
        position: [f32; 3],
        rotation: [f32; 4],
    ) -> PlayerData {
        PlayerData {
            name: player.name.clone(),
            is_male: player.is_male,
            height: player.height,
            skin_color: player.skin_color,
            face_variant: player.face_variant,
            hair_style: player.hair_style,
            hair_color: player.hair_color,
            skills: Self::skills_to_save_data(&player.skills),
            money: Self::money_to_save_data(&player.money),
            inventory: Self::inventory_to_save_data(&player.inventory),
            inventory_weight: player.inventory_weight,
            position,
            rotation,
            state: Self::state_to_save_data(&player.state),
            camera_mode: Self::camera_to_save_data(&player.camera_mode),
            stamina: player.stamina,
        }
    }

    /// Convert PlayerSkillsData to PlayerSkillsData (for saving)
    pub fn skills_to_save_data(skills: &PlayerSkillsData) -> PlayerSkillsData {
        skills.clone()
    }

    /// Convert PlayerWallet to PlayerMoneyData
    pub fn money_to_save_data(money: &PlayerMoneyData) -> PlayerMoneyData {
        PlayerMoneyData {
            rub: money.rub,
            cny: money.cny,
            usd: money.usd,
        }
    }

    /// Convert inventory to save data
    pub fn inventory_to_save_data(
        inventory: &[crate::game::player::InventoryItem],
    ) -> Vec<InventoryItemData> {
        inventory
            .iter()
            .map(|item| InventoryItemData {
                name: item.name.clone(),
                weight: item.count as f32 * 0.5, // approximate weight
                item_type: format!("{:?}", item.item_type),
                quantity: item.count,
            })
            .collect()
    }

    /// Convert PlayerState to PlayerStateData
    pub fn state_to_save_data(state: &PlayerState) -> PlayerStateData {
        match state {
            PlayerState::OnFoot => PlayerStateData::OnFoot,
            PlayerState::InVehicle {
                vehicle_index,
                seat_index,
            } => PlayerStateData::InVehicle {
                vehicle_index: *vehicle_index,
                vehicle_id: *vehicle_index as u64,
                seat_index: *seat_index as usize,
            },
            _ => PlayerStateData::OnFoot,
        }
    }

    /// Convert CameraMode to CameraModeData
    pub fn camera_to_save_data(camera: &CameraMode) -> CameraModeData {
        match camera {
            CameraMode::FirstPerson => CameraModeData::FirstPerson,
            CameraMode::ThirdPerson {
                distance,
                yaw,
                pitch,
            } => CameraModeData::ThirdPerson {
                distance: *distance,
                yaw: *yaw,
                pitch: *pitch,
            },
        }
    }
}

impl Default for SaveSystem {
    fn default() -> Self {
        Self::new("saves")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_location_types() {
        assert!(SaveSystem::new("").can_save_at(SaveLocationType::Bed));
        assert!(SaveSystem::new("").can_save_at(SaveLocationType::VehicleBunk));
        assert!(SaveSystem::new("").can_save_at(SaveLocationType::Tent));
        assert!(SaveSystem::new("").can_save_at(SaveLocationType::OwnedProperty));
    }

    #[test]
    fn test_save_paths() {
        let system = SaveSystem::new("test_saves");
        assert_eq!(system.get_save_path(0), "test_saves/save_0.bin");
        assert_eq!(system.get_metadata_path(0), "test_saves/save_0.meta");
    }
}
