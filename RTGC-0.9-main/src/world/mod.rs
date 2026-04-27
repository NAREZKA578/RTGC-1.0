//! World Module for RTGC-0.9
//! Provides terrain generation, chunk management, world streaming, and day/night cycle

pub mod world_module;
pub mod chunk;
pub mod chunk_manager;
pub mod prop_spawner;
pub mod terrain_generator;
pub mod lod_system;
pub mod spatial_index;
pub mod world_streaming;
pub mod day_night_cycle;
pub mod settlement;
pub mod russian_names;
pub mod road_network;
pub mod buildings;
pub mod novosibirsk_map;

pub use world_module::OpenWorld;
pub use chunk::{Chunk, ChunkData, ChunkId, TerrainVertex, generate_chunk_mesh, CHUNK_SIZE, HEIGHTMAP_RESOLUTION};
pub use chunk_manager::ChunkManager;
pub use prop_spawner::PropSpawner;
pub use terrain_generator::{TerrainGenerator, SurfaceType};
pub use lod_system::LodSystem;
pub use spatial_index::SpatialIndex;
pub use world_streaming::WorldStreamer;
pub use day_night_cycle::DayNightCycle;
pub use settlement::{Settlement, SettlementType, BuildingInstance, BuildingType, SettlementServices};
pub use russian_names::generate_name as generate_settlement_name;
pub use road_network::{RoadNetwork, RoadSegment, RoadType};
pub use buildings::{BuildingPlacer, BuildingBoxDesc};
pub use novosibirsk_map::{NovosibirskMap, create_novosibirsk_map, City, Highway, River, Landmark};
