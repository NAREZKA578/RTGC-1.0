//! Building placement and management in settlements
//! 
//! Implements:
//! - Building types with parameters for rendering
//! - Procedural placement within settlements
//! - Flat color descriptors for alpha rendering

use nalgebra::Vector3;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::settlement::{Settlement, SettlementType, BuildingInstance, BuildingType};
use super::terrain_generator::TerrainGenerator;

/// Building box descriptor for rendering (flat colors, no textures)
#[derive(Debug, Clone)]
pub struct BuildingBoxDesc {
    pub size: Vector3<f32>,      // width, height, depth
    pub color: [f32; 3],         // wall color
    pub roof_color: [f32; 3],    // roof color
    pub has_roof: bool,
    pub roof_slope: f32,         // 0 = flat, >0 = sloped
    pub windows: bool,
    pub door_offset: f32,        // offset from center for door
}

impl BuildingBoxDesc {
    /// Get building descriptor by type
    pub fn from_building_type(building_type: BuildingType) -> Self {
        match building_type {
            // Residential buildings
            BuildingType::IzbaDerevo => Self {
                size: Vector3::new(6.0, 4.0, 8.0),
                color: [0.6, 0.45, 0.3],     // Wood brown
                roof_color: [0.4, 0.2, 0.1], // Dark wood roof
                has_roof: true,
                roof_slope: 0.4,
                windows: true,
                door_offset: -0.2,
            },
            BuildingType::DomKirpich => Self {
                size: Vector3::new(8.0, 6.0, 10.0),
                color: [0.7, 0.5, 0.4],      // Light brick
                roof_color: [0.3, 0.25, 0.2], // Dark roof
                has_roof: true,
                roof_slope: 0.3,
                windows: true,
                door_offset: 0.0,
            },
            BuildingType::KhrushchyovkaPyat => Self {
                size: Vector3::new(12.0, 15.0, 30.0), // 5 floors
                color: [0.65, 0.65, 0.6],      // Concrete gray
                roof_color: [0.3, 0.3, 0.3],   // Flat roof
                has_roof: false,
                roof_slope: 0.0,
                windows: true,
                door_offset: 0.0,
            },
            BuildingType::PanelnyyDom => Self {
                size: Vector3::new(15.0, 27.0, 40.0), // 9 floors
                color: [0.7, 0.7, 0.72],       // Panel gray
                roof_color: [0.25, 0.25, 0.25],
                has_roof: false,
                roof_slope: 0.0,
                windows: true,
                door_offset: 0.0,
            },

            // Industrial buildings
            BuildingType::Sklad => Self {
                size: Vector3::new(20.0, 8.0, 40.0),
                color: [0.5, 0.5, 0.5],        // Metal siding
                roof_color: [0.7, 0.3, 0.1],   // Rusted metal
                has_roof: true,
                roof_slope: 0.15,
                windows: false,
                door_offset: 0.0,
            },
            BuildingType::Pilorama => Self {
                size: Vector3::new(25.0, 10.0, 50.0),
                color: [0.55, 0.45, 0.35],     // Weathered wood/metal
                roof_color: [0.6, 0.35, 0.15],
                has_roof: true,
                roof_slope: 0.2,
                windows: false,
                door_offset: 0.0,
            },
            BuildingType::Zavodskoi => Self {
                size: Vector3::new(30.0, 15.0, 60.0),
                color: [0.55, 0.55, 0.6],      // Industrial gray
                roof_color: [0.4, 0.35, 0.3],
                has_roof: true,
                roof_slope: 0.1,
                windows: false,
                door_offset: 0.0,
            },
            BuildingType::ShakhtaVhod => Self {
                size: Vector3::new(15.0, 8.0, 20.0),
                color: [0.4, 0.35, 0.3],       // Concrete/steel
                roof_color: [0.35, 0.3, 0.25],
                has_roof: true,
                roof_slope: 0.0,
                windows: false,
                door_offset: 0.0,
            },

            // Infrastructure
            BuildingType::AZS => Self {
                size: Vector3::new(10.0, 4.0, 10.0), // Canopy
                color: [0.8, 0.2, 0.15],       // Red canopy
                roof_color: [0.8, 0.2, 0.15],
                has_roof: true,
                roof_slope: 0.05,
                windows: false,
                door_offset: 0.0,
            },
            BuildingType::RemontBaza => Self {
                size: Vector3::new(18.0, 6.0, 25.0),
                color: [0.5, 0.55, 0.5],       // Green-gray workshop
                roof_color: [0.4, 0.45, 0.4],
                has_roof: true,
                roof_slope: 0.15,
                windows: true,
                door_offset: 0.0,
            },
            BuildingType::Most => Self {
                size: Vector3::new(8.0, 0.5, 30.0), // Bridge deck
                color: [0.4, 0.4, 0.45],       // Concrete
                roof_color: [0.4, 0.4, 0.45],
                has_roof: false,
                roof_slope: 0.0,
                windows: false,
                door_offset: 0.0,
            },
            BuildingType::Kolodets => Self {
                size: Vector3::new(1.5, 1.2, 1.5),
                color: [0.55, 0.4, 0.25],      // Wood/stone
                roof_color: [0.4, 0.25, 0.15],
                has_roof: true,
                roof_slope: 0.3,
                windows: false,
                door_offset: 0.0,
            },
            BuildingType::ZaborDerevo => Self {
                size: Vector3::new(2.0, 1.8, 0.1), // Fence segment
                color: [0.5, 0.35, 0.2],       // Weathered wood
                roof_color: [0.4, 0.25, 0.15],
                has_roof: false,
                roof_slope: 0.0,
                windows: false,
                door_offset: 0.0,
            },

            // Signs and props
            BuildingType::DorZnak => Self {
                size: Vector3::new(0.8, 1.5, 0.05),
                color: [0.9, 0.9, 0.1],        // Yellow sign
                roof_color: [0.9, 0.9, 0.1],
                has_roof: false,
                roof_slope: 0.0,
                windows: false,
                door_offset: 0.0,
            },
            BuildingType::StolbElektr => Self {
                size: Vector3::new(0.2, 8.0, 0.2),
                color: [0.4, 0.35, 0.3],       // Concrete pole
                roof_color: [0.4, 0.35, 0.3],
                has_roof: false,
                roof_slope: 0.0,
                windows: false,
                door_offset: 0.0,
            },
            BuildingType::Konteyner => Self {
                size: Vector3::new(2.5, 2.5, 6.0),
                color: [0.6, 0.3, 0.2],        // Rusted container
                roof_color: [0.55, 0.25, 0.15],
                has_roof: false,
                roof_slope: 0.0,
                windows: false,
                door_offset: 0.0,
            },
        }
    }
}

/// Building placement generator for settlements
pub struct BuildingPlacer {
    seed: u64,
}

impl BuildingPlacer {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Place buildings within a settlement
    pub fn place_buildings_in_settlement(
        &self,
        settlement: &Settlement,
        terrain: &TerrainGenerator,
    ) -> Vec<BuildingInstance> {
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed ^ settlement.id);
        let mut buildings = Vec::new();

        // Get main road direction (from first connected road)
        let main_road_dir = if !settlement.connection_roads.is_empty() {
            // Simplified: use x-axis as main direction
            Vector3::new(1.0, 0.0, 0.0)
        } else {
            Vector3::new(1.0, 0.0, 0.0)
        };

        match settlement.settlement_type {
            SettlementType::Derevnya => {
                self.place_derevnya(settlement, &mut rng, terrain, &main_road_dir, &mut buildings);
            }
            SettlementType::Posyolok => {
                self.place_posyolok(settlement, &mut rng, terrain, &main_road_dir, &mut buildings);
            }
            SettlementType::MalyiGorod => {
                self.place_malyi_gorod(settlement, &mut rng, terrain, &main_road_dir, &mut buildings);
            }
            SettlementType::PromGorod => {
                self.place_prom_gorod(settlement, &mut rng, terrain, &main_road_dir, &mut buildings);
            }
        }

        buildings
    }

    /// Place buildings in a деревня (village): 5-30 houses, dirt roads, well
    fn place_derevnya(
        &self,
        settlement: &Settlement,
        rng: &mut ChaCha8Rng,
        terrain: &TerrainGenerator,
        main_road_dir: &Vector3<f32>,
        buildings: &mut Vec<BuildingInstance>,
    ) {
        let num_houses = rng.gen_range(5..=30);
        let center = settlement.center;

        // Place houses along main road
        for i in 0..num_houses {
            let offset_along = (i as f32 / num_houses as f32 - 0.5) * settlement.radius * 1.5;
            let offset_side = if rng.gen_bool(0.5) {
                rng.gen_range(8.0..15.0)
            } else {
                -rng.gen_range(8.0..15.0)
            };

            let pos = Vector3::new(
                center.x + offset_along * main_road_dir.x + offset_side * main_road_dir.z,
                0.0, // Will be set to terrain height
                center.z + offset_along * main_road_dir.z - offset_side * main_road_dir.x,
            );

            // Get terrain height
            let height = terrain.get_height(pos.x, pos.z);
            let pos = Vector3::new(pos.x, height, pos.z);

            buildings.push(BuildingInstance {
                building_type: BuildingType::IzbaDerevo,
                position: pos,
                rotation: rng.gen_range(0.0..std::f32::consts::TAU),
                scale: rng.gen_range(0.9..1.1),
            });

            // Add fence segments around house
            if rng.gen_bool(0.5) {
                self.add_fence_around(pos, rng, buildings);
            }
        }

        // Add well in center
        buildings.push(BuildingInstance {
            building_type: BuildingType::Kolodets,
            position: Vector3::new(center.x, terrain.get_height(center.x, center.z), center.z),
            rotation: 0.0,
            scale: 1.0,
        });
    }

    /// Place buildings in a посёлок (town): 30-150 houses, asphalt center, shop, gas station
    fn place_posyolok(
        &self,
        settlement: &Settlement,
        rng: &mut ChaCha8Rng,
        terrain: &TerrainGenerator,
        main_road_dir: &Vector3<f32>,
        buildings: &mut Vec<BuildingInstance>,
    ) {
        let num_houses = rng.gen_range(30..=150);
        let center = settlement.center;

        // Central street with houses
        for i in 0..num_houses {
            let angle = (i as f32 / num_houses as f32) * std::f32::consts::TAU;
            let radius = rng.gen_range(20.0..settlement.radius);
            
            let pos = Vector3::new(
                center.x + angle.cos() * radius,
                0.0,
                center.z + angle.sin() * radius,
            );

            let height = terrain.get_height(pos.x, pos.z);
            let pos = Vector3::new(pos.x, height, pos.z);

            // Mix of house types
            let building_type = match rng.gen_range(0..100) {
                0..=60 => BuildingType::IzbaDerevo,
                61..=90 => BuildingType::DomKirpich,
                _ => BuildingType::IzbaDerevo,
            };

            buildings.push(BuildingInstance {
                building_type,
                position: pos,
                rotation: angle + std::f32::consts::FRAC_PI_2,
                scale: rng.gen_range(0.9..1.15),
            });
        }

        // Add gas station on edge
        let azs_pos = Vector3::new(
            center.x + settlement.radius * 0.8,
            terrain.get_height(center.x + settlement.radius * 0.8, center.z),
            center.z,
        );
        buildings.push(BuildingInstance {
            building_type: BuildingType::AZS,
            position: azs_pos,
            rotation: 0.0,
            scale: 1.0,
        });

        // Add repair shop
        let repair_pos = Vector3::new(
            center.x - settlement.radius * 0.5,
            terrain.get_height(center.x - settlement.radius * 0.5, center.z),
            center.z + settlement.radius * 0.3,
        );
        buildings.push(BuildingInstance {
            building_type: BuildingType::RemontBaza,
            position: repair_pos,
            rotation: 0.0,
            scale: 1.0,
        });
    }

    /// Place buildings in a малый город (small town): 150-500 houses, everything except factory
    fn place_malyi_gorod(
        &self,
        settlement: &Settlement,
        rng: &mut ChaCha8Rng,
        terrain: &TerrainGenerator,
        main_road_dir: &Vector3<f32>,
        buildings: &mut Vec<BuildingInstance>,
    ) {
        let center = settlement.center;

        // Grid-based placement for urban area
        let grid_size = 20.0; // meters between buildings
        let grid_radius = (settlement.radius / grid_size) as i32;

        for gx in -grid_radius..=grid_radius {
            for gz in -grid_radius..=grid_radius {
                if rng.gen_bool(0.3) {
                    continue; // Skip some cells for variety
                }

                let pos = Vector3::new(
                    center.x + gx as f32 * grid_size,
                    0.0,
                    center.z + gz as f32 * grid_size,
                );

                // Skip if too far from center
                let dist = ((pos.x - center.x).powi(2) + (pos.z - center.z).powi(2)).sqrt();
                if dist > settlement.radius {
                    continue;
                }

                let height = terrain.get_height(pos.x, pos.z);
                let pos = Vector3::new(pos.x, height, pos.z);

                // Building type based on distance from center
                let building_type = if dist < settlement.radius * 0.3 {
                    // Center: larger buildings
                    match rng.gen_range(0..100) {
                        0..=40 => BuildingType::KhrushchyovkaPyat,
                        41..=70 => BuildingType::DomKirpich,
                        _ => BuildingType::PanelnyyDom,
                    }
                } else {
                    // Outer: smaller buildings
                    match rng.gen_range(0..100) {
                        0..=50 => BuildingType::DomKirpich,
                        51..=80 => BuildingType::IzbaDerevo,
                        _ => BuildingType::KhrushchyovkaPyat,
                    }
                };

                buildings.push(BuildingInstance {
                    building_type,
                    position: pos,
                    rotation: rng.gen_range(0.0..std::f32::consts::TAU),
                    scale: rng.gen_range(0.9..1.1),
                });
            }
        }

        // Add warehouse on edge
        let sklad_pos = Vector3::new(
            center.x + settlement.radius * 0.7,
            terrain.get_height(center.x + settlement.radius * 0.7, center.z),
            center.z,
        );
        buildings.push(BuildingInstance {
            building_type: BuildingType::Sklad,
            position: sklad_pos,
            rotation: 0.0,
            scale: 1.0,
        });
    }

    /// Place buildings in a промгород (industrial city): factory/mine as main feature
    fn place_prom_gorod(
        &self,
        settlement: &Settlement,
        rng: &mut ChaCha8Rng,
        terrain: &TerrainGenerator,
        main_road_dir: &Vector3<f32>,
        buildings: &mut Vec<BuildingInstance>,
    ) {
        let center = settlement.center;

        // Place industrial zone
        let industrial_pos = Vector3::new(
            center.x + settlement.radius * 0.5,
            terrain.get_height(center.x + settlement.radius * 0.5, center.z),
            center.z,
        );

        buildings.push(BuildingInstance {
            building_type: BuildingType::Zavodskoi,
            position: industrial_pos,
            rotation: 0.0,
            scale: 1.2,
        });

        // Add mine entrance if in mountains (simplified check)
        if rng.gen_bool(0.5) {
            let mine_pos = Vector3::new(
                center.x - settlement.radius * 0.6,
                terrain.get_height(center.x - settlement.radius * 0.6, center.z),
                center.z + settlement.radius * 0.4,
            );
            buildings.push(BuildingInstance {
                building_type: BuildingType::ShakhtaVhod,
                position: mine_pos,
                rotation: 0.0,
                scale: 1.0,
            });
        }

        // Add sawmill
        let pilorama_pos = Vector3::new(
            center.x,
            terrain.get_height(center.x, center.z + settlement.radius * 0.6),
            center.z + settlement.radius * 0.6,
        );
        buildings.push(BuildingInstance {
            building_type: BuildingType::Pilorama,
            position: pilorama_pos,
            rotation: std::f32::consts::FRAC_PI_2,
            scale: 1.0,
        });

        // Residential areas (similar to malyi_gorod but denser)
        let grid_size = 18.0;
        let grid_radius = (settlement.radius / grid_size) as i32;

        for gx in -grid_radius..=grid_radius {
            for gz in -grid_radius..=grid_radius {
                if rng.gen_bool(0.25) {
                    continue;
                }

                let pos = Vector3::new(
                    center.x + gx as f32 * grid_size,
                    0.0,
                    center.z + gz as f32 * grid_size,
                );

                let dist = ((pos.x - center.x).powi(2) + (pos.z - center.z).powi(2)).sqrt();
                if dist > settlement.radius || dist < 30.0 {
                    continue; // Skip industrial zone
                }

                let height = terrain.get_height(pos.x, pos.z);
                let pos = Vector3::new(pos.x, height, pos.z);

                let building_type = match rng.gen_range(0..100) {
                    0..=30 => BuildingType::PanelnyyDom,
                    31..=70 => BuildingType::KhrushchyovkaPyat,
                    _ => BuildingType::DomKirpich,
                };

                buildings.push(BuildingInstance {
                    building_type,
                    position: pos,
                    rotation: rng.gen_range(0.0..std::f32::consts::TAU),
                    scale: rng.gen_range(0.95..1.05),
                });
            }
        }
    }

    /// Add fence segments around a building
    fn add_fence_around(
        &self,
        pos: Vector3<f32>,
        rng: &mut ChaCha8Rng,
        buildings: &mut Vec<BuildingInstance>,
    ) {
        let fence_size = 8.0;
        let num_segments = 8;

        for i in 0..num_segments {
            let angle = (i as f32 / num_segments as f32) * std::f32::consts::TAU;
            let fx = pos.x + angle.cos() * fence_size / 2.0;
            let fz = pos.z + angle.sin() * fence_size / 2.0;
            let fy = pos.y;

            buildings.push(BuildingInstance {
                building_type: BuildingType::ZaborDerevo,
                position: Vector3::new(fx, fy, fz),
                rotation: angle + std::f32::consts::FRAC_PI_2,
                scale: 1.0,
            });
        }
    }
}
