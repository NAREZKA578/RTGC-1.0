//! Mission generator - creates delivery missions from settlement infrastructure
//!
//! Implements:
//! - Automatic mission generation from POI (Points of Interest)
//! - Cargo types based on settlement specialization
//! - Reward calculation based on distance and difficulty

use nalgebra::Vector3;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::world::RoadNetwork;
use crate::world::{BuildingType, Settlement, SettlementType};

/// Cargo types available for transport
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CargoType {
    /// Lumber from sawmill - medium weight, fragile
    Lumber,
    /// Coal from mine - heavy, can spill
    Coal,
    /// Fuel from depot - hazardous, careful driving
    Fuel,
    /// Metal from factory - very heavy, durable
    Metal,
    /// Food from farm - fragile, time limit
    Food,
    /// Machinery - very fragile, high value
    Machinery,
    /// General cargo - balanced properties
    General,
}

impl CargoType {
    /// Get base weight in kg for this cargo type
    pub fn base_weight(&self) -> f32 {
        match self {
            CargoType::Lumber => 2000.0,
            CargoType::Coal => 5000.0,
            CargoType::Fuel => 4000.0,
            CargoType::Metal => 8000.0,
            CargoType::Food => 1500.0,
            CargoType::Machinery => 3000.0,
            CargoType::General => 2500.0,
        }
    }

    /// Fragility: how easily damaged (0.0 = indestructible, 1.0 = very fragile)
    pub fn fragility(&self) -> f32 {
        match self {
            CargoType::Lumber => 0.3,
            CargoType::Coal => 0.1,
            CargoType::Fuel => 0.5,
            CargoType::Metal => 0.05,
            CargoType::Food => 0.7,
            CargoType::Machinery => 0.9,
            CargoType::General => 0.2,
        }
    }

    /// Base reward per km
    pub fn reward_per_km(&self) -> f32 {
        match self {
            CargoType::Lumber => 5.0,
            CargoType::Coal => 8.0,
            CargoType::Fuel => 12.0,
            CargoType::Metal => 15.0,
            CargoType::Food => 10.0,
            CargoType::Machinery => 25.0,
            CargoType::General => 7.0,
        }
    }
}

/// A delivery mission
#[derive(Debug, Clone)]
pub struct Mission {
    pub id: String,
    pub pickup_settlement_id: u64,
    pub delivery_settlement_id: u64,
    pub pickup_position: Vector3<f32>,
    pub delivery_position: Vector3<f32>,
    pub cargo_type: CargoType,
    pub cargo_weight: f32,
    pub distance_km: f32,
    pub base_reward: i32,
    pub time_limit: Option<f32>, // seconds, None = no limit
    pub description: String,
}

impl Mission {
    /// Calculate damage penalty based on cargo fragility and impact severity
    pub fn calculate_damage_penalty(&self, total_impact: f32) -> f32 {
        let cargo_fragility = self.cargo_type.fragility();
        // Impact threshold before damage starts
        let threshold = 5.0;

        if total_impact < threshold {
            return 0.0;
        }

        // Penalty scales with fragility and impact above threshold
        let damage = (total_impact - threshold) * cargo_fragility * 0.1;
        damage.min(0.75) // Max 75% penalty
    }

    /// Calculate time penalty if mission has time limit
    pub fn calculate_time_penalty(&self, time_taken: f32) -> f32 {
        if let Some(limit) = self.time_limit {
            if time_taken > limit {
                let overtime = time_taken - limit;
                let penalty = (overtime / limit) * 0.5; // 50% penalty at 2x time
                return penalty.min(0.5); // Max 50% penalty
            }
        }
        0.0
    }

    /// Calculate final reward after penalties
    pub fn calculate_final_reward(&self, total_impact: f32, time_taken: f32) -> i32 {
        let damage_penalty = self.calculate_damage_penalty(total_impact);
        let time_penalty = self.calculate_time_penalty(time_taken);

        let total_penalty = (damage_penalty + time_penalty).min(0.9);
        let final_reward = self.base_reward as f32 * (1.0 - total_penalty);

        final_reward.max(0.0) as i32
    }
}

/// Mission generator using settlement infrastructure
pub struct MissionGenerator {
    settlements: Vec<Settlement>,
    road_network: RoadNetwork,
    seed: u64,
    mission_counter: u32,
}

impl MissionGenerator {
    pub fn new(settlements: Vec<Settlement>, road_network: RoadNetwork, seed: u64) -> Self {
        Self {
            settlements,
            road_network,
            seed,
            mission_counter: 0,
        }
    }

    /// Generate a mission suitable for player's current position
    pub fn generate_mission(&mut self, player_pos: Vector3<f32>) -> Option<Mission> {
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed + self.mission_counter as u64);
        self.mission_counter += 1;

        // Find nearest settlement with cargo depot to player
        let pickup_settlement = self.find_nearest_cargo_source(player_pos)?;

        // Find suitable delivery destination
        let delivery_settlement = self.find_suitable_destination(&pickup_settlement, &mut rng)?;

        // Determine cargo type based on pickup settlement's specialization
        let cargo_type = self.determine_cargo_type(&pickup_settlement, &mut rng);

        // Calculate distance
        let dx = pickup_settlement.center[0] - delivery_settlement.center[0];
        let dz = pickup_settlement.center[2] - delivery_settlement.center[2];
        let distance_km = (dx * dx + dz * dz).sqrt() / 1000.0; // meters to km

        // Calculate base reward
        let mut base_reward = (cargo_type.reward_per_km() * distance_km.max(1.0)) as i32;
        base_reward = base_reward.max(50); // Minimum reward

        // Get pickup and delivery positions (at depot buildings)
        let pickup_pos = self.get_depot_position(&pickup_settlement, true);
        let delivery_pos = self.get_depot_position(&delivery_settlement, false);

        // Time limit for fragile cargo
        let time_limit = if cargo_type == CargoType::Food || cargo_type == CargoType::Machinery {
            Some(distance_km * 120.0) // ~50 km/h average + buffer
        } else {
            None
        };

        // Generate description
        let description = format!(
            "Доставить {} из {} в {}",
            self.cargo_name(cargo_type),
            pickup_settlement.name,
            delivery_settlement.name,
        );

        Some(Mission {
            id: format!("mission_{}_{}", pickup_settlement.id, self.mission_counter),
            pickup_settlement_id: pickup_settlement.id,
            delivery_settlement_id: delivery_settlement.id,
            pickup_position: pickup_pos,
            delivery_position: delivery_pos,
            cargo_type,
            cargo_weight: cargo_type.base_weight() * rng.gen_range(0.8..1.2),
            distance_km,
            base_reward,
            time_limit,
            description,
        })
    }

    /// Find nearest settlement that can be a cargo source
    fn find_nearest_cargo_source(&self, player_pos: Vector3<f32>) -> Option<&Settlement> {
        let mut best: Option<(&Settlement, f32)> = None;

        for settlement in &self.settlements {
            // Check if settlement has cargo depot
            if !settlement.has_cargo_source() {
                continue;
            }

            let dist = ((player_pos.x - settlement.center[0]).powi(2)
                + (player_pos.z - settlement.center[2]).powi(2))
            .sqrt();

            if best.is_none() || dist < best.map_or(f32::MAX, |(_, d)| d) {
                best = Some((settlement, dist));
            }
        }

        best.map(|(s, _)| s)
    }

    /// Find suitable delivery destination
    fn find_suitable_destination(
        &self,
        pickup: &Settlement,
        rng: &mut ChaCha8Rng,
    ) -> Option<&Settlement> {
        let candidates: Vec<_> = self
            .settlements
            .iter()
            .filter(|s| s.id != pickup.id && s.has_delivery_point())
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Weight by distance (prefer medium distances)
        let mut weights = Vec::new();
        for candidate in &candidates {
            let dx = pickup.center[0] - candidate.center[0];
            let dz = pickup.center[2] - candidate.center[2];
            let dist = (dx * dx + dz * dz).sqrt();

            // Prefer 5-50km distances
            let dist_km = dist / 1000.0;
            let weight = if dist_km >= 5.0 && dist_km <= 50.0 {
                1.0
            } else if dist_km < 5.0 {
                0.3
            } else {
                0.5
            };

            weights.push(weight);
        }

        // Weighted random selection
        let total: f32 = weights.iter().sum();
        let mut roll = rng.r#gen::<f32>() * total;

        for (i, candidate) in candidates.iter().enumerate() {
            roll -= weights[i];
            if roll <= 0.0 {
                return Some(*candidate);
            }
        }

        candidates.last().copied()
    }

    /// Determine cargo type based on settlement's industry
    fn determine_cargo_type(&self, settlement: &Settlement, rng: &mut ChaCha8Rng) -> CargoType {
        // Check settlement's primary industry
        match settlement.settlement_type {
            SettlementType::PromGorod => {
                // Industrial city: metal, machinery, fuel
                match rng.gen_range(0..100) {
                    0..=40 => CargoType::Metal,
                    41..=70 => CargoType::Machinery,
                    71..=90 => CargoType::Fuel,
                    _ => CargoType::General,
                }
            }
            SettlementType::MalyiGorod => {
                // Small town: general cargo, food, lumber
                match rng.gen_range(0..100) {
                    0..=30 => CargoType::General,
                    31..=60 => CargoType::Food,
                    61..=85 => CargoType::Lumber,
                    _ => CargoType::Metal,
                }
            }
            SettlementType::Posyolok => {
                // Town: lumber, food, coal
                match rng.gen_range(0..100) {
                    0..=40 => CargoType::Lumber,
                    41..=70 => CargoType::Food,
                    71..=90 => CargoType::Coal,
                    _ => CargoType::General,
                }
            }
            SettlementType::Derevnya => {
                // Village: food, lumber
                match rng.gen_range(0..100) {
                    0..=60 => CargoType::Food,
                    61..=90 => CargoType::Lumber,
                    _ => CargoType::General,
                }
            }
        }
    }

    /// Get position of cargo depot in settlement
    fn get_depot_position(&self, settlement: &Settlement, is_pickup: bool) -> Vector3<f32> {
        // Find warehouse/sklad building or use center
        for building in &settlement.buildings {
            match building.building_type {
                BuildingType::Sklad | BuildingType::Pilorama | BuildingType::Zavodskoi => {
                    if is_pickup {
                        return building.position;
                    }
                }
                BuildingType::AZS => {
                    if !is_pickup && settlement.services.has_fuel_station {
                        return building.position;
                    }
                }
                _ => {}
            }
        }

        // Fallback to settlement center with offset
        let offset = if is_pickup { 10.0 } else { -10.0 };
        Vector3::new(
            settlement.center[0] + offset,
            settlement.center[1],
            settlement.center[2],
        )
    }

    /// Get human-readable cargo name in Russian
    fn cargo_name(&self, cargo: CargoType) -> &'static str {
        match cargo {
            CargoType::Lumber => "доски",
            CargoType::Coal => "уголь",
            CargoType::Fuel => "топливо",
            CargoType::Metal => "металл",
            CargoType::Food => "продукты",
            CargoType::Machinery => "технику",
            CargoType::General => "груз",
        }
    }

    /// Get all available settlements
    pub fn settlements(&self) -> &[Settlement] {
        &self.settlements
    }

    /// Get road network
    pub fn road_network(&self) -> &RoadNetwork {
        &self.road_network
    }
}
