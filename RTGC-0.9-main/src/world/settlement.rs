//! Settlement System for RTGC
//! Generates Russian-style settlements: villages, towns, and industrial cities

use nalgebra::Vector3;
use rand_chacha::ChaCha8Rng;
use rand::{Rng, RngCore, SeedableRng};
use rand::prelude::SliceRandom;

/// Types of settlements in the game world
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementType {
    /// Village: 5-30 houses, dirt roads, well, shed
    Derevnya,
    /// Township: 30-150 houses, asphalt in center, shop, gas station
    Posyolok,
    /// Small town: 150-500 houses, everything except factory
    MalyiGorod,
    /// Industrial city: factory/mine/warehouse - main delivery destination
    PromGorod,
}

impl SettlementType {
    /// Get typical population range for settlement type
    pub fn population_range(&self) -> (u32, u32) {
        match self {
            SettlementType::Derevnya => (5, 30),
            SettlementType::Posyolok => (30, 150),
            SettlementType::MalyiGorod => (150, 500),
            SettlementType::PromGorod => (500, 2000),
        }
    }

    /// Get typical radius for settlement type (in meters)
    pub fn radius(&self) -> f32 {
        match self {
            SettlementType::Derevnya => 50.0,
            SettlementType::Posyolok => 150.0,
            SettlementType::MalyiGorod => 400.0,
            SettlementType::PromGorod => 800.0,
        }
    }
}

/// Services available at a settlement
#[derive(Debug, Clone, Default)]
pub struct SettlementServices {
    pub has_fuel_station: bool,
    pub has_repair_shop: bool,
    pub has_cargo_depot: bool,      // Where cargo is picked up
    pub has_delivery_point: bool,   // Where cargo is delivered
    pub fuel_price: f32,            // Increases with remoteness
}

/// A building instance in a settlement
#[derive(Debug, Clone)]
pub struct BuildingInstance {
    pub building_type: BuildingType,
    pub position: Vector3<f32>,
    pub rotation: f32, // radians around Y axis
    pub scale: f32,    // uniform scale factor
}

/// Types of buildings that can be placed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingType {
    // Residential
    IzbaDerevo,       // Wooden hut, 1 floor
    DomKirpich,       // Brick house, 1-2 floors
    KhrushchyovkaPyat, // 5-floor Khrushchev-era apartment (cities)
    PanelnyyDom,      // 9-floor panel building (industrial city)

    // Industrial (cargo sources)
    Sklad,            // Warehouse - simple rectangle with gates
    Pilorama,         // Sawmill - near forest, cargo = lumber
    Zavodskoi,        // Factory workshop - in industrial city
    ShakhtaVhod,      // Mine entrance - in mountains, cargo = coal/ore

    // Infrastructure (services)
    AZS,              // Gas station - fuel trigger
    RemontBaza,       // Repair base - repair trigger
    Most,             // Bridge - static rigid body
    Kolodets,         // Well - decoration
    ZaborDerevo,      // Wooden fence - static bodies

    // Signs and small objects
    DorZnak,          // Road sign
    StolbElektr,      // Power line pole
    Konteyner,        // Abandoned container (can be pushed)
}

/// A settlement in the world
#[derive(Debug, Clone)]
pub struct Settlement {
    pub id: u64,
    pub name: String,
    pub settlement_type: SettlementType,
    pub center: Vector3<f32>,
    pub radius: f32,
    pub population: u32,
    pub buildings: Vec<BuildingInstance>,
    pub connection_roads: Vec<u64>, // IDs of roads connecting to this settlement
    pub services: SettlementServices,
}

impl Settlement {
    /// Generate a settlement deterministically from seed and grid position
    pub fn generate(seed: u64, grid_x: i32, grid_z: i32, center_x: f32, center_z: f32) -> Option<Settlement> {
        let mut rng = ChaCha8Rng::seed_from_u64(seed ^ Self::hash_grid(grid_x, grid_z));

        // Not every grid cell has a settlement
        const SETTLEMENT_DENSITY: f32 = 0.15; // 15% chance per cell
        if rng.r#gen::<f32>() > SETTLEMENT_DENSITY {
            return None;
        }

        // Determine settlement type based on probability
        let s_type = match rng.gen_range(0..100) {
            0..=60 => SettlementType::Derevnya,
            61..=85 => SettlementType::Posyolok,
            86..=97 => SettlementType::MalyiGorod,
            _ => SettlementType::PromGorod,
        };

        // Generate population
        let (pop_min, pop_max) = s_type.population_range();
        let population = rng.gen_range(pop_min..=pop_max);

        // Generate name from Russian dictionary
        let name = crate::world::russian_names::generate_name(rng.next_u64());

        // Generate services based on settlement type
        let services = Self::generate_services(&mut rng, s_type);

        Some(Settlement {
            id: seed ^ Self::hash_grid(grid_x, grid_z),
            name,
            settlement_type: s_type,
            center: Vector3::new(center_x, 0.0, center_z),
            radius: s_type.radius(),
            population,
            buildings: Vec::new(), // Will be populated by place_buildings_in_settlement
            connection_roads: Vec::new(),
            services,
        })
    }

    fn hash_grid(x: i32, z: i32) -> u64 {
        // Simple hash combining x and z coordinates
        let mut h: u64 = 0xdeadbeef;
        h = h.wrapping_mul(31).wrapping_add(x as u64);
        h = h.wrapping_mul(31).wrapping_add(z as u64);
        h
    }

    fn generate_services(rng: &mut ChaCha8Rng, s_type: SettlementType) -> SettlementServices {
        let mut services = SettlementServices::default();

        match s_type {
            SettlementType::Derevnya => {
                // Villages rarely have services
                services.has_fuel_station = false;
                services.has_repair_shop = false;
                services.has_cargo_depot = rng.gen_bool(0.1);
                services.fuel_price = 1.5; // Expensive if available
            }
            SettlementType::Posyolok => {
                // Townships usually have gas station
                services.has_fuel_station = rng.gen_bool(0.6);
                services.has_repair_shop = rng.gen_bool(0.3);
                services.has_cargo_depot = rng.gen_bool(0.4);
                services.fuel_price = 1.2;
            }
            SettlementType::MalyiGorod => {
                // Small towns have most services
                services.has_fuel_station = true;
                services.has_repair_shop = rng.gen_bool(0.7);
                services.has_cargo_depot = true;
                services.has_delivery_point = true;
                services.fuel_price = 1.0;
            }
            SettlementType::PromGorod => {
                // Industrial cities have everything
                services.has_fuel_station = true;
                services.has_repair_shop = true;
                services.has_cargo_depot = true;
                services.has_delivery_point = true;
                services.fuel_price = 0.9; // Cheaper due to industry
            }
        }

        services
    }

    /// Check if a point is inside this settlement's bounds
    pub fn contains_point(&self, x: f32, z: f32) -> bool {
        let dx = x - self.center[0];
        let dz = z - self.center[2];
        let dist_sq = dx * dx + dz * dz;
        dist_sq <= self.radius * self.radius
    }

    /// Get distance from settlement center to a point
    pub fn distance_to(&self, x: f32, z: f32) -> f32 {
        let dx = x - self.center[0];
        let dz = z - self.center[2];
        (dx * dx + dz * dz).sqrt()
    }

    /// Check if settlement has cargo source (warehouse, sawmill, factory, mine)
    pub fn has_cargo_source(&self) -> bool {
        self.services.has_cargo_depot || 
        self.buildings.iter().any(|b| matches!(
            b.building_type,
            BuildingType::Sklad | BuildingType::Pilorama | BuildingType::Zavodskoi | BuildingType::ShakhtaVhod
        ))
    }

    /// Check if settlement has delivery point
    pub fn has_delivery_point(&self) -> bool {
        self.services.has_delivery_point
    }
}

/// Place buildings within a settlement based on type and terrain
pub fn place_buildings_in_settlement(
    settlement: &Settlement,
    seed: u64,
    terrain_heights: &dyn Fn(f32, f32) -> f32,
) -> Vec<BuildingInstance> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ settlement.id);
    let mut buildings = Vec::new();

    // Main street runs along the primary road direction
    let street_angle = rng.gen_range(0.0..std::f32::consts::PI);
    let street_direction = Vector3::new(street_angle.cos(), 0.0, street_angle.sin());

    // Number of buildings based on settlement type
    let num_buildings = (settlement.population as f32 * rng.gen_range(0.8..1.2)) as usize;

    for i in 0..num_buildings {
        // Position along street
        let t = (i as f32 / num_buildings.max(1) as f32) * 2.0 - 1.0; // -1 to 1
        let along_street = street_direction * (t * settlement.radius * 0.8);

        // Offset perpendicular to street
        let perp_offset = rng.gen_range(-15.0..15.0);
        let perp_direction = Vector3::new(-street_direction.z, 0.0, street_direction.x);
        let position = settlement.center + along_street + perp_direction * perp_offset;

        // Get terrain height at this position
        let height = terrain_heights(position.x, position.z);
        let final_pos = Vector3::new(position.x, height, position.z);

        // Choose building type based on settlement type and position
        let building_type = choose_building_type(&mut rng, settlement.settlement_type, i);

        // Random rotation slightly off street alignment
        let rotation = street_angle + rng.gen_range(-0.2..0.2);

        // Scale based on building type
        let scale = get_building_scale(building_type, &mut rng);

        buildings.push(BuildingInstance {
            building_type,
            position: final_pos,
            rotation,
            scale,
        });
    }

    // Add special buildings based on services
    if settlement.services.has_fuel_station {
        // Place gas station at edge of settlement
        let azs_angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
        let azs_dist = settlement.radius * 0.7;
        let azs_x = settlement.center[0] + azs_angle.cos() * azs_dist;
        let azs_z = settlement.center[2] + azs_angle.sin() * azs_dist;
        let azs_pos = Vector3::new(
            azs_x,
            terrain_heights(azs_x, azs_z),
            azs_z,
        );

        buildings.push(BuildingInstance {
            building_type: BuildingType::AZS,
            position: azs_pos,
            rotation: azs_angle,
            scale: 1.0,
        });
    }

    buildings
}

fn choose_building_type(
    rng: &mut ChaCha8Rng,
    s_type: SettlementType,
    index: usize,
) -> BuildingType {
    match s_type {
        SettlementType::Derevnya => {
            if rng.gen_bool(0.8) {
                BuildingType::IzbaDerevo
            } else {
                *[BuildingType::ZaborDerevo, BuildingType::Kolodets].choose(rng).unwrap_or(&BuildingType::ZaborDerevo)
            }
        }
        SettlementType::Posyolok => {
            match rng.gen_range(0..100) {
                0..=60 => BuildingType::IzbaDerevo,
                61..=80 => BuildingType::DomKirpich,
                81..=90 => BuildingType::AZS,
                _ => BuildingType::Sklad,
            }
        }
        SettlementType::MalyiGorod => {
            match rng.gen_range(0..100) {
                0..=40 => BuildingType::DomKirpich,
                41..=70 => BuildingType::KhrushchyovkaPyat,
                71..=85 => BuildingType::Sklad,
                86..=95 => BuildingType::AZS,
                _ => BuildingType::RemontBaza,
            }
        }
        SettlementType::PromGorod => {
            match rng.gen_range(0..100) {
                0..=30 => BuildingType::KhrushchyovkaPyat,
                31..=50 => BuildingType::PanelnyyDom,
                51..=70 => BuildingType::Zavodskoi,
                71..=85 => BuildingType::Sklad,
                86..=95 => BuildingType::AZS,
                _ => BuildingType::RemontBaza,
            }
        }
    }
}

fn get_building_scale(_building_type: BuildingType, rng: &mut ChaCha8Rng) -> f32 {
    rng.gen_range(0.8..1.2)
}
