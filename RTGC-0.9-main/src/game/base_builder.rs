//! Base Building System for RTGC-0.8
//! Construction of bases, outposts, and structures in the open world
//!
//! Features:
//! - Grid-based placement system
//! - Multiple structure types (shelter, warehouse, workshop, fence, etc.)
//! - Resource requirements and costs
//! - Structure integrity and damage over time
//! - Upgrade system
//! - Ownership and permissions

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Type alias for backwards compatibility
type Vec3 = Vector3<f32>;

/// Maximum number of structures per base
pub const MAX_STRUCTURES_PER_BASE: usize = 64;

/// Maximum number of bases per player
pub const MAX_BASES_PER_PLAYER: usize = 5;

/// Grid cell size for placement (meters)
pub const GRID_CELL_SIZE: f32 = 1.0;

/// Minimum distance between bases (meters)
pub const MIN_BASE_DISTANCE: f32 = 500.0;

/// Base types available for construction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BaseType {
    /// Temporary camp with basic shelter
    Outpost,
    /// Permanent base with multiple buildings
    MainBase,
    /// Resource extraction point (mining, logging)
    ResourcePoint,
    /// Storage facility
    Warehouse,
    /// Vehicle maintenance point
    ServicePoint,
}

/// Structure types that can be built
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StructureType {
    // === Shelters ===
    /// Basic tent shelter (sleep, storage)
    TentShelter,
    /// Wooden cabin (better protection, sleep)
    WoodenCabin,
    /// Metal container shelter (durable, storage)
    ContainerShelter,
    /// Prefab house (comfortable, permanent)
    PrefabHouse,

    // === Storage ===
    /// Open storage platform
    StoragePlatform,
    /// Covered warehouse
    Warehouse,
    /// Refrigerated storage
    ColdStorage,
    /// Fuel tank
    FuelTank,

    // === Workshops ===
    /// Basic repair stand
    RepairStand,
    /// Full workshop with tools
    Workshop,
    /// Advanced garage with lift
    Garage,
    /// Welding station
    WeldingStation,

    // === Utilities ===
    /// Fence section (protection, boundary)
    Fence,
    /// Gate (vehicle access)
    Gate,
    /// Watchtower (security, visibility)
    Watchtower,
    /// Light pole (night work)
    LightPole,
    /// Power generator
    Generator,
    /// Water well/tank
    WaterSource,

    // === Special ===
    /// Helipad (for aviation operations)
    Helipad,
    /// Crane foundation
    CraneFoundation,
    /// Drill pad (for geology/mining)
    DrillPad,
    /// Logging equipment pad
    LoggingPad,
}

impl StructureType {
    /// Get dimensions of structure (width, length, height in meters)
    pub fn dimensions(&self) -> (f32, f32, f32) {
        match self {
            StructureType::TentShelter => (3.0, 4.0, 2.5),
            StructureType::WoodenCabin => (5.0, 6.0, 3.0),
            StructureType::ContainerShelter => (2.5, 6.0, 2.8),
            StructureType::PrefabHouse => (8.0, 10.0, 3.5),

            StructureType::StoragePlatform => (4.0, 4.0, 0.5),
            StructureType::Warehouse => (10.0, 20.0, 5.0),
            StructureType::ColdStorage => (6.0, 8.0, 4.0),
            StructureType::FuelTank => (3.0, 3.0, 4.0),

            StructureType::RepairStand => (4.0, 6.0, 3.0),
            StructureType::Workshop => (8.0, 12.0, 4.0),
            StructureType::Garage => (6.0, 10.0, 3.5),
            StructureType::WeldingStation => (3.0, 4.0, 3.0),

            StructureType::Fence => (0.2, 2.0, 2.0),
            StructureType::Gate => (4.0, 0.5, 2.5),
            StructureType::Watchtower => (2.0, 2.0, 8.0),
            StructureType::LightPole => (0.3, 0.3, 6.0),
            StructureType::Generator => (2.0, 3.0, 2.0),
            StructureType::WaterSource => (2.0, 2.0, 3.0),

            StructureType::Helipad => (20.0, 20.0, 0.5),
            StructureType::CraneFoundation => (4.0, 4.0, 1.0),
            StructureType::DrillPad => (8.0, 8.0, 0.5),
            StructureType::LoggingPad => (10.0, 15.0, 0.5),
        }
    }

    /// Get base cost in RUB for construction
    pub fn base_cost(&self) -> u32 {
        match self {
            StructureType::TentShelter => 15_000,
            StructureType::WoodenCabin => 85_000,
            StructureType::ContainerShelter => 120_000,
            StructureType::PrefabHouse => 450_000,

            StructureType::StoragePlatform => 25_000,
            StructureType::Warehouse => 380_000,
            StructureType::ColdStorage => 620_000,
            StructureType::FuelTank => 95_000,

            StructureType::RepairStand => 45_000,
            StructureType::Workshop => 280_000,
            StructureType::Garage => 195_000,
            StructureType::WeldingStation => 75_000,

            StructureType::Fence => 8_000,
            StructureType::Gate => 35_000,
            StructureType::Watchtower => 65_000,
            StructureType::LightPole => 12_000,
            StructureType::Generator => 85_000,
            StructureType::WaterSource => 45_000,

            StructureType::Helipad => 1_200_000,
            StructureType::CraneFoundation => 320_000,
            StructureType::DrillPad => 180_000,
            StructureType::LoggingPad => 95_000,
        }
    }

    /// Get construction time in minutes (real-time)
    /// NOTE: Set to 0 for instant construction as requested
    pub fn construction_time(&self) -> u32 {
        0 // Instant construction
    }

    /// Get required skills for construction (skill, minimum rank)
    pub fn required_skills(&self) -> Vec<(SkillType, u8)> {
        match self {
            StructureType::TentShelter => vec![],
            StructureType::WoodenCabin => vec![(SkillType::Construction, 2)],
            StructureType::ContainerShelter => {
                vec![(SkillType::Construction, 3), (SkillType::Welding, 2)]
            }
            StructureType::PrefabHouse => vec![(SkillType::Construction, 5)],

            StructureType::StoragePlatform => vec![(SkillType::Construction, 2)],
            StructureType::Warehouse => {
                vec![(SkillType::Construction, 4), (SkillType::Logistics, 3)]
            }
            StructureType::ColdStorage => {
                vec![(SkillType::Construction, 5), (SkillType::Electrics, 4)]
            }
            StructureType::FuelTank => vec![(SkillType::Construction, 3)],

            StructureType::RepairStand => vec![(SkillType::Mechanics, 3)],
            StructureType::Workshop => {
                vec![(SkillType::Mechanics, 5), (SkillType::Construction, 4)]
            }
            StructureType::Garage => vec![(SkillType::Construction, 4)],
            StructureType::WeldingStation => {
                vec![(SkillType::Welding, 4), (SkillType::Electrics, 3)]
            }

            StructureType::Fence => vec![],
            StructureType::Gate => vec![(SkillType::Construction, 2)],
            StructureType::Watchtower => vec![(SkillType::Construction, 3)],
            StructureType::LightPole => vec![(SkillType::Electrics, 2)],
            StructureType::Generator => vec![(SkillType::Electrics, 4), (SkillType::Mechanics, 3)],
            StructureType::WaterSource => vec![(SkillType::Construction, 3)],

            StructureType::Helipad => {
                vec![(SkillType::Construction, 6), (SkillType::Navigation, 4)]
            }
            StructureType::CraneFoundation => {
                vec![(SkillType::Construction, 5), (SkillType::Crane, 3)]
            }
            StructureType::DrillPad => vec![(SkillType::Drilling, 4), (SkillType::Geology, 3)],
            StructureType::LoggingPad => {
                vec![(SkillType::Logging, 4), (SkillType::Construction, 3)]
            }
        }
    }

    /// Check if this structure provides sleeping capability
    pub fn provides_sleep(&self) -> bool {
        matches!(
            self,
            StructureType::TentShelter
                | StructureType::WoodenCabin
                | StructureType::ContainerShelter
                | StructureType::PrefabHouse
        )
    }

    /// Check if this structure provides storage capability
    pub fn provides_storage(&self) -> bool {
        matches!(
            self,
            StructureType::TentShelter
                | StructureType::WoodenCabin
                | StructureType::ContainerShelter
                | StructureType::PrefabHouse
                | StructureType::StoragePlatform
                | StructureType::Warehouse
                | StructureType::ColdStorage
        )
    }

    /// Check if this structure provides vehicle repair capability
    pub fn provides_repair(&self) -> bool {
        matches!(
            self,
            StructureType::RepairStand | StructureType::Workshop | StructureType::Garage
        )
    }

    /// Get durability (max integrity) of structure
    pub fn max_integrity(&self) -> f32 {
        match self {
            StructureType::TentShelter => 100.0,
            StructureType::WoodenCabin => 500.0,
            StructureType::ContainerShelter => 800.0,
            StructureType::PrefabHouse => 1200.0,

            StructureType::StoragePlatform => 300.0,
            StructureType::Warehouse => 1000.0,
            StructureType::ColdStorage => 900.0,
            StructureType::FuelTank => 600.0,

            StructureType::RepairStand => 400.0,
            StructureType::Workshop => 900.0,
            StructureType::Garage => 700.0,
            StructureType::WeldingStation => 500.0,

            StructureType::Fence => 200.0,
            StructureType::Gate => 350.0,
            StructureType::Watchtower => 450.0,
            StructureType::LightPole => 250.0,
            StructureType::Generator => 400.0,
            StructureType::WaterSource => 350.0,

            StructureType::Helipad => 1500.0,
            StructureType::CraneFoundation => 1200.0,
            StructureType::DrillPad => 800.0,
            StructureType::LoggingPad => 600.0,
        }
    }
}

/// Skill types required for construction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillType {
    Construction,
    Welding,
    Electrics,
    Mechanics,
    Logistics,
    Navigation,
    Crane,
    Drilling,
    Geology,
    Logging,
}

/// Resource requirements for construction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Money cost (RUB)
    pub money_cost: u32,
    /// Materials needed
    pub materials: HashMap<String, u32>,
    /// Tools required (not consumed)
    pub tools_required: Vec<String>,
}

impl ResourceRequirements {
    pub fn new(money_cost: u32) -> Self {
        Self {
            money_cost,
            materials: HashMap::new(),
            tools_required: Vec::new(),
        }
    }

    pub fn add_material(&mut self, name: &str, amount: u32) {
        self.materials.insert(name.to_string(), amount);
    }

    pub fn add_tool(&mut self, name: &str) {
        if !self.tools_required.contains(&name.to_string()) {
            self.tools_required.push(name.to_string());
        }
    }
}

/// A single structure in a base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltStructure {
    /// Unique ID
    pub id: u64,
    /// Type of structure
    pub structure_type: StructureType,
    /// Position in world (grid-aligned)
    pub position: [f32; 3],
    /// Rotation in degrees (0, 90, 180, 270)
    pub rotation: u16,
    /// Current integrity (0.0 - max_integrity)
    pub integrity: f32,
    /// Construction progress (0.0 - 1.0)
    pub construction_progress: f32,
    /// Time when construction started
    pub construction_start: u64,
    /// Owner player ID
    pub owner_id: u64,
    /// Last maintenance time
    pub last_maintenance: u64,
    /// Upgrade level
    pub upgrade_level: u8,
}

impl BuiltStructure {
    pub fn new(
        id: u64,
        structure_type: StructureType,
        position: Vector3<f32>,
        owner_id: u64,
    ) -> Self {
        let max_integrity = structure_type.max_integrity();
        Self {
            id,
            structure_type,
            position: [position.x, position.y, position.z],
            rotation: 0,
            integrity: max_integrity,
            construction_progress: 1.0, // Instant build by default, can be modified
            construction_start: 0,
            owner_id,
            last_maintenance: 0,
            upgrade_level: 0,
        }
    }

    /// Check if structure is fully constructed
    pub fn is_complete(&self) -> bool {
        self.construction_progress >= 1.0
    }

    /// Check if structure is functional
    pub fn is_functional(&self) -> bool {
        self.is_complete() && self.integrity > 0.0
    }

    /// Get current functionality percentage
    pub fn functionality(&self) -> f32 {
        if !self.is_complete() {
            return self.construction_progress;
        }
        let max = self.structure_type.max_integrity();
        if max <= 0.0 {
            return 1.0;
        }
        (self.integrity / max).clamp(0.0, 1.0)
    }

    /// Apply damage to structure
    pub fn take_damage(&mut self, damage: f32) {
        self.integrity = (self.integrity - damage).max(0.0);
    }

    /// Repair structure
    pub fn repair(&mut self, amount: f32) {
        let max = self.structure_type.max_integrity();
        self.integrity = (self.integrity + amount).min(max);
    }

    /// Update construction progress (instant build - no update needed)
    pub fn update_construction(&mut self, _delta_seconds: f32) {
        // Instant construction: always complete
        self.construction_progress = 1.0;
    }
}

/// A base/outpost owned by a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerBase {
    /// Unique ID
    pub id: u64,
    /// Base name
    pub name: String,
    /// Base type
    pub base_type: BaseType,
    /// Center position
    pub center_position: [f32; 3],
    /// Owner player ID
    pub owner_id: u64,
    /// List of structures
    pub structures: Vec<BuiltStructure>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last activity timestamp
    pub last_activity: u64,
}

impl PlayerBase {
    pub fn new(
        id: u64,
        name: String,
        base_type: BaseType,
        center_position: Vector3<f32>,
        owner_id: u64,
    ) -> Self {
        Self {
            id,
            name,
            base_type,
            center_position: [center_position.x, center_position.y, center_position.z],
            owner_id,
            structures: Vec::with_capacity(MAX_STRUCTURES_PER_BASE),
            created_at: 0,
            last_activity: 0,
        }
    }

    /// Add structure to base
    pub fn add_structure(&mut self, structure: BuiltStructure) -> Result<(), &'static str> {
        if self.structures.len() >= MAX_STRUCTURES_PER_BASE {
            return Err("Maximum structures reached");
        }

        // Check for overlap with existing structures
        let (new_w, new_l, _) = structure.structure_type.dimensions();
        let new_rotation_rad = (structure.rotation as f32) * std::f32::consts::PI / 180.0;
        let new_cos = new_rotation_rad.cos();
        let new_sin = new_rotation_rad.sin();

        // Rotated dimensions
        let new_half_w = ((new_w * new_cos.abs()) + (new_l * new_sin.abs())) / 2.0;
        let new_half_l = ((new_w * new_sin.abs()) + (new_l * new_cos.abs())) / 2.0;

        for existing in &self.structures {
            if !existing.is_functional() {
                continue;
            }

            let (ex_w, ex_l, _) = existing.structure_type.dimensions();
            let ex_rotation_rad = (existing.rotation as f32) * std::f32::consts::PI / 180.0;
            let ex_cos = ex_rotation_rad.cos();
            let ex_sin = ex_rotation_rad.sin();

            let ex_half_w = ((ex_w * ex_cos.abs()) + (ex_l * ex_sin.abs())) / 2.0;
            let ex_half_l = ((ex_w * ex_sin.abs()) + (ex_l * ex_cos.abs())) / 2.0;

            // Simple AABB check (conservative)
            let dx = (structure.position[0] - existing.position[0]).abs();
            let dz = (structure.position[2] - existing.position[2]).abs();
            let min_dist_x = new_half_w + ex_half_w + 0.5; // 0.5m gap
            let min_dist_z = new_half_l + ex_half_l + 0.5;

            if dx < min_dist_x && dz < min_dist_z {
                return Err("Structure overlaps with existing structure");
            }
        }

        self.structures.push(structure);
        self.last_activity = self.get_current_timestamp();
        Ok(())
    }

    /// Remove structure from base
    pub fn remove_structure(&mut self, structure_id: u64) -> Option<BuiltStructure> {
        if let Some(pos) = self.structures.iter().position(|s| s.id == structure_id) {
            let structure = self.structures.remove(pos);
            self.last_activity = self.get_current_timestamp();
            Some(structure)
        } else {
            None
        }
    }

    /// Get structure by ID
    pub fn get_structure(&self, structure_id: u64) -> Option<&BuiltStructure> {
        self.structures.iter().find(|s| s.id == structure_id)
    }

    /// Get mutable structure by ID
    pub fn get_structure_mut(&mut self, structure_id: u64) -> Option<&mut BuiltStructure> {
        self.structures.iter_mut().find(|s| s.id == structure_id)
    }

    /// Count structures by type
    pub fn count_structures(&self, structure_type: StructureType) -> usize {
        self.structures
            .iter()
            .filter(|s| s.structure_type == structure_type && s.is_functional())
            .count()
    }

    /// Check if base has capability (sleep, storage, repair, etc.)
    pub fn has_capability(&self, capability: BaseCapability) -> bool {
        self.structures.iter().any(|s| {
            s.is_functional() && {
                match capability {
                    BaseCapability::Sleep => s.structure_type.provides_sleep(),
                    BaseCapability::Storage => s.structure_type.provides_storage(),
                    BaseCapability::Repair => s.structure_type.provides_repair(),
                }
            }
        })
    }

    /// Update all structures (construction, decay)
    pub fn update(&mut self, delta_seconds: f32) {
        for structure in &mut self.structures {
            structure.update_construction(delta_seconds);

            // Natural decay over time (very slow)
            if structure.is_complete() {
                let decay_rate = 0.0001; // Per second
                structure.take_damage(
                    decay_rate * delta_seconds * structure.structure_type.max_integrity(),
                );
            }
        }
    }

    fn get_current_timestamp(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Capabilities a base can provide
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseCapability {
    Sleep,
    Storage,
    Repair,
}

/// Base building system manager
pub struct BaseBuildingSystem {
    /// All player bases
    bases: HashMap<u64, PlayerBase>,
    /// Next base ID
    next_base_id: u64,
    /// Next structure ID
    next_structure_id: u64,
    /// Pending constructions (structure_id, completion_timestamp)
    pending_constructions: Vec<(u64, u64)>,
}

impl BaseBuildingSystem {
    pub fn new() -> Self {
        Self {
            bases: HashMap::new(),
            next_base_id: 1,
            next_structure_id: 1,
            pending_constructions: Vec::new(),
        }
    }

    /// Create a new base
    pub fn create_base(
        &mut self,
        name: String,
        base_type: BaseType,
        position: Vector3<f32>,
        owner_id: u64,
    ) -> Result<u64, &'static str> {
        // Check player doesn't exceed max bases
        let player_base_count = self
            .bases
            .values()
            .filter(|b| b.owner_id == owner_id)
            .count();

        if player_base_count >= MAX_BASES_PER_PLAYER {
            return Err("Maximum bases per player reached");
        }

        // Check minimum distance from other bases
        for existing in self.bases.values() {
            let center = Vector3::new(
                existing.center_position[0],
                existing.center_position[1],
                existing.center_position[2],
            );
            let dist = (position - center).norm();
            if dist < MIN_BASE_DISTANCE {
                return Err("Too close to another base");
            }
        }

        let base_id = self.next_base_id;
        self.next_base_id += 1;

        let base = PlayerBase::new(base_id, name, base_type, position, owner_id);
        self.bases.insert(base_id, base);

        Ok(base_id)
    }

    /// Delete a base
    pub fn delete_base(&mut self, base_id: u64) -> Option<PlayerBase> {
        self.pending_constructions
            .retain(|(bid, _)| *bid != base_id);
        self.bases.remove(&base_id)
    }

    /// Get base by ID
    pub fn get_base(&self, base_id: u64) -> Option<&PlayerBase> {
        self.bases.get(&base_id)
    }

    /// Get mutable base by ID
    pub fn get_base_mut(&mut self, base_id: u64) -> Option<&mut PlayerBase> {
        self.bases.get_mut(&base_id)
    }

    /// Get all bases for a player
    pub fn get_player_bases(&self, owner_id: u64) -> Vec<&PlayerBase> {
        self.bases
            .values()
            .filter(|b| b.owner_id == owner_id)
            .collect()
    }

    /// Построить здание мгновенно (основной метод)
    pub fn build_structure(
        &mut self,
        base_id: u64,
        structure_type: StructureType,
        position: Vector3<f32>,
        rotation: u16,
        owner_id: u64,
    ) -> Result<u64, &'static str> {
        let base = self.bases.get_mut(&base_id).ok_or("Base not found")?;

        if base.owner_id != owner_id {
            return Err("Not base owner");
        }

        let structure_id = self.next_structure_id;
        self.next_structure_id += 1;

        // Мгновенная постройка: прогресс = 1.0 сразу
        let mut structure = BuiltStructure::new(structure_id, structure_type, position, owner_id);
        structure.rotation = rotation;
        structure.construction_progress = 1.0;

        base.add_structure(structure)?;

        Ok(structure_id)
    }

    /// Устаревший метод start_construction (теперь вызывает мгновенную постройку)
    pub fn start_construction(
        &mut self,
        base_id: u64,
        structure_type: StructureType,
        position: Vector3<f32>,
        rotation: u16,
        owner_id: u64,
    ) -> Result<u64, &'static str> {
        // Вызываем мгновенную постройку
        self.build_structure(base_id, structure_type, position, rotation, owner_id)
    }

    /// Instant build (for debug/creative mode)
    pub fn instant_build(
        &mut self,
        base_id: u64,
        structure_type: StructureType,
        position: Vector3<f32>,
        rotation: u16,
        owner_id: u64,
    ) -> Result<u64, &'static str> {
        let base = self.bases.get_mut(&base_id).ok_or("Base not found")?;

        if base.owner_id != owner_id {
            return Err("Not base owner");
        }

        let structure_id = self.next_structure_id;
        self.next_structure_id += 1;

        let mut structure = BuiltStructure::new(structure_id, structure_type, position, owner_id);
        structure.rotation = rotation;
        structure.construction_progress = 1.0;

        base.add_structure(structure)?;

        Ok(structure_id)
    }

    /// Update pending constructions
    pub fn update(&mut self, delta_seconds: f32, current_time: u64) {
        // Complete finished constructions
        let completed: Vec<u64> = self
            .pending_constructions
            .iter()
            .filter(|(_, completion)| *completion <= current_time)
            .map(|(id, _)| *id)
            .collect();

        for structure_id in completed {
            self.pending_constructions
                .retain(|(id, _)| *id != structure_id);

            // Mark structure as complete
            for base in self.bases.values_mut() {
                if let Some(structure) = base.get_structure_mut(structure_id) {
                    structure.construction_progress = 1.0;
                }
            }
        }

        // Update all bases
        for base in self.bases.values_mut() {
            base.update(delta_seconds);
        }
    }

    /// Get resource requirements for a structure
    pub fn get_requirements(&self, structure_type: StructureType) -> ResourceRequirements {
        let mut req = ResourceRequirements::new(structure_type.base_cost());

        // Add material requirements based on type
        match structure_type {
            StructureType::TentShelter => {
                req.add_material("fabric", 50);
                req.add_material("metal_poles", 8);
            }
            StructureType::WoodenCabin => {
                req.add_material("logs", 120);
                req.add_material("nails", 500);
                req.add_tool("saw");
                req.add_tool("hammer");
            }
            StructureType::ContainerShelter => {
                req.add_material("shipping_container", 1);
                req.add_material("welding_rods", 20);
                req.add_tool("welder");
            }
            StructureType::Warehouse => {
                req.add_material("steel_beams", 80);
                req.add_material("metal_sheets", 200);
                req.add_material("concrete", 5000);
                req.add_tool("welder");
                req.add_tool("crane");
            }
            StructureType::Fence => {
                req.add_material("wood_planks", 20);
                req.add_material("nails", 100);
            }
            StructureType::Workshop => {
                req.add_material("steel_beams", 40);
                req.add_material("metal_sheets", 100);
                req.add_material("tools_set", 1);
                req.add_tool("welder");
            }
            _ => {
                // Generic requirements
                req.add_material("materials", structure_type.base_cost() / 1000);
            }
        }

        req
    }

    /// Check if player can afford construction
    pub fn can_afford(&self, player_money: u32, structure_type: StructureType) -> bool {
        player_money >= structure_type.base_cost()
    }

    /// Check if player has required skills
    pub fn has_required_skills<F>(&self, check_skill: F, structure_type: StructureType) -> bool
    where
        F: Fn(SkillType) -> u8,
    {
        for (skill, required_rank) in structure_type.required_skills() {
            let player_rank = check_skill(skill);
            if player_rank < required_rank {
                return false;
            }
        }
        true
    }

    /// Get nearest base to position
    pub fn get_nearest_base(
        &self,
        position: Vector3<f32>,
        max_distance: f32,
    ) -> Option<&PlayerBase> {
        self.bases
            .values()
            .filter(|b| {
                let center = Vector3::new(
                    b.center_position[0],
                    b.center_position[1],
                    b.center_position[2],
                );
                (center - position).norm() <= max_distance
            })
            .min_by(|a, b| {
                let center_a = Vector3::new(
                    a.center_position[0],
                    a.center_position[1],
                    a.center_position[2],
                );
                let center_b = Vector3::new(
                    b.center_position[0],
                    b.center_position[1],
                    b.center_position[2],
                );
                let dist_a = (center_a - position).norm();
                let dist_b = (center_b - position).norm();
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Get save location status at position
    pub fn is_safe_location(&self, position: Vector3<f32>, max_distance: f32) -> bool {
        self.get_nearest_base(position, max_distance)
            .map_or(false, |base| base.has_capability(BaseCapability::Sleep))
    }
}

impl Default for BaseBuildingSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_base() {
        let mut system = BaseBuildingSystem::new();
        let base_id = system
            .create_base(
                "Test Base".to_string(),
                BaseType::Outpost,
                Vec3::new(100.0, 0.0, 100.0),
                1,
            )
            .unwrap();

        assert_eq!(base_id, 1);
        assert!(system.get_base(1).is_some());
    }

    #[test]
    fn test_max_bases() {
        let mut system = BaseBuildingSystem::new();

        for i in 0..MAX_BASES_PER_PLAYER {
            let result = system.create_base(
                format!("Base {}", i),
                BaseType::Outpost,
                Vec3::new((i * 1000) as f32, 0.0, 0.0),
                1,
            );
            assert!(result.is_ok());
        }

        // Should fail - max reached
        let result = system.create_base(
            "Extra Base".to_string(),
            BaseType::Outpost,
            Vec3::new(9999.0, 0.0, 0.0),
            1,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_structure_dimensions() {
        let tent = StructureType::TentShelter;
        let (w, l, h) = tent.dimensions();
        assert!((w - 3.0).abs() < 0.01);
        assert!((l - 4.0).abs() < 0.01);
        assert!((h - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_structure_costs() {
        assert!(StructureType::PrefabHouse.base_cost() > StructureType::TentShelter.base_cost());
        assert!(StructureType::Helipad.base_cost() > StructureType::Warehouse.base_cost());
    }
}
