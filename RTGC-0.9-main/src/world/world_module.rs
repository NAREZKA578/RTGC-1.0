//! Step 4: Procedural World Generation and Streaming
//!
//! This module implements:
//! - Deterministic procedural terrain generation using Perlin/Simplex noise
//! - Chunk-based world streaming with async loading
//! - LOD (Level of Detail) system for geometry and textures
//! - Frustum and occlusion culling
//! - Spatial indexing with Octree for fast queries

use nalgebra::{Vector3, Point3};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, debug};

pub use super::chunk::{Chunk, ChunkId, ChunkData, CHUNK_SIZE};
pub use super::chunk_manager::{ChunkManager, ChunkCoords};
pub use super::terrain_generator::{TerrainGenerator, NoiseConfig};
pub use super::world_streaming::{WorldStreamer, StreamingConfig};
pub use super::lod_system::LodSystem;
pub use super::spatial_index::{SpatialIndex, QueryResult};
pub use super::day_night_cycle::{DayNightCycle, CelestialBody, MoonPhase, TimeOfDay};
pub use super::prop_spawner::PropSpawner;

/// Main world manager that coordinates all world subsystems
pub struct OpenWorld {
    /// Currently loaded chunks
    loaded_chunks: HashMap<ChunkId, Arc<Chunk>>,
    /// Terrain generator for procedural generation
    pub generator: TerrainGenerator,
    /// World streaming system for async loading
    streamer: WorldStreamer,
    /// LOD management system
    lod_system: LodSystem,
    /// Spatial index for fast queries
    spatial_index: SpatialIndex,
    /// Player position for determining visible chunks
    player_position: Vector3<f32>,
    /// Load radius in chunks
    load_radius: u32,
    /// Unload radius (must be >= load_radius)
    unload_radius: u32,
    /// World seed
    pub seed: u64,
}

impl OpenWorld {
    /// Create a new open world with default configuration
    pub fn new(seed: u64) -> Self {
        let noise_config = NoiseConfig {
            seed,
            base_frequency: 0.01,
            octaves: 6,
            persistence: 0.5,
            lacunarity: 2.0,
            height_scale: 100.0,
        };
        
        let generator = TerrainGenerator::new(noise_config);

        let streaming_config = StreamingConfig {
            load_radius: 5,
            unload_radius: 7,
            max_concurrent_loads: 4,
            chunk_update_interval_ms: 100,
        };

        let load_radius = streaming_config.load_radius;
        let unload_radius = streaming_config.unload_radius;
        let streamer = WorldStreamer::new(streaming_config);
        let lod_system = LodSystem::new();
        let spatial_index = SpatialIndex::new();

        Self {
            loaded_chunks: HashMap::new(),
            generator,
            streamer,
            lod_system,
            spatial_index,
            player_position: Vector3::zeros(),
            load_radius,
            unload_radius,
            seed,
        }
    }

    /// Get terrain height at given coordinates
    pub fn get_height(&self, x: f32, z: f32) -> f32 {
        self.get_height_at(Vector3::new(x, 0.0, z))
    }

    /// Generate terrain data
    pub fn generate_terrain(&mut self) {
        // заглушка
    }
    
    /// Update the world based on player position
    pub fn update(&mut self, player_pos: Vector3<f32>, dt: f32) {
        self.player_position = player_pos;
        
        // Determine which chunks should be loaded
        let player_chunk = self.world_to_chunk(player_pos);
        let chunks_to_load = self.get_chunks_in_radius(player_chunk, self.load_radius);
        let chunks_to_unload = self.get_chunks_to_unload(player_chunk, self.unload_radius);
        
        // Request loading of new chunks
        for chunk_id in chunks_to_load {
            if !self.loaded_chunks.contains_key(&chunk_id) {
                self.request_chunk_load(chunk_id);
            }
        }
        
        // Process completed loads
        self.process_loaded_chunks();
        
        // Unload distant chunks
        for chunk_id in chunks_to_unload {
            self.unload_chunk(chunk_id);
        }
        
        // Update LOD levels based on distance to camera
        self.lod_system.update(&self.loaded_chunks, player_pos);
        
        // Update spatial index
        self.update_spatial_index();
    }
    
    /// Convert world coordinates to chunk coordinates
    fn world_to_chunk(&self, world_pos: Vector3<f32>) -> ChunkId {
        let x = (world_pos.x / CHUNK_SIZE as f32).floor() as i32;
        let z = (world_pos.z / CHUNK_SIZE as f32).floor() as i32;
        ChunkId::new(x, z)
    }
    
    /// Get all chunk IDs within a radius of a center chunk
    fn get_chunks_in_radius(&self, center: ChunkId, radius: u32) -> Vec<ChunkId> {
        let mut chunks = Vec::new();
        let r = radius as i32;
        
        for dx in -r..=r {
            for dz in -r..=r {
                if dx * dx + dz * dz <= r * r {
                    chunks.push(ChunkId::new(center.x + dx, center.z + dz));
                }
            }
        }
        
        chunks
    }
    
    /// Get chunks that should be unloaded
    fn get_chunks_to_unload(&self, center: ChunkId, unload_radius: u32) -> Vec<ChunkId> {
        let mut to_unload = Vec::new();
        
        for &chunk_id in self.loaded_chunks.keys() {
            let dx = (chunk_id.x - center.x).abs();
            let dz = (chunk_id.z - center.z).abs();
            
            if dx > unload_radius as i32 || dz > unload_radius as i32 {
                to_unload.push(chunk_id);
            }
        }
        
        to_unload
    }
    
    /// Request asynchronous loading of a chunk
    fn request_chunk_load(&mut self, chunk_id: ChunkId) {
        debug!("Requesting load of chunk {:?}", chunk_id);
        
        // Generate chunk data
        let chunk_data = self.generator.generate_chunk(chunk_id);
        
        // Create chunk
        let chunk = Chunk::new(chunk_id, chunk_data);
        
        // Queue for loading
        self.streamer.queue_chunk_load(chunk);
    }
    
    /// Process chunks that have finished loading
    fn process_loaded_chunks(&mut self) {
        // Poll for completed chunk loads from the streaming system
        while let Some(loaded) = self.streamer.poll_loaded_chunk() {
            debug!("Chunk {:?} loaded successfully in {:.2}ms", loaded.chunk.id, loaded.load_time_ms);
            
            // Add to loaded chunks
            let chunk = Arc::new(loaded.chunk);
            self.loaded_chunks.insert(chunk.id, chunk.clone());
            
            // Add to spatial index
            self.spatial_index.add_chunk(chunk.id, chunk.clone());
            
            info!("Chunk {:?} added to loaded_chunks", chunk.id);
        }
    }
    
    /// Unload a chunk
    fn unload_chunk(&mut self, chunk_id: ChunkId) {
        debug!("Unloading chunk {:?}", chunk_id);
        
        if self.loaded_chunks.remove(&chunk_id).is_some() {
            self.spatial_index.remove_chunk(chunk_id);
            info!("Chunk {:?} unloaded", chunk_id);
        }
    }
    
    /// Update the spatial index with current chunks
    fn update_spatial_index(&mut self) {
        // Rebuild or update spatial index as needed
        self.spatial_index.rebuild_if_needed();
    }
    
    /// Get the height at a world position
    pub fn get_height_at(&self, world_pos: Vector3<f32>) -> f32 {
        let chunk_id = self.world_to_chunk(world_pos);
        
        if let Some(chunk) = self.loaded_chunks.get(&chunk_id) {
            let local_x = ((world_pos.x % CHUNK_SIZE as f32) + CHUNK_SIZE as f32) % CHUNK_SIZE as f32;
            let local_z = ((world_pos.z % CHUNK_SIZE as f32) + CHUNK_SIZE as f32) % CHUNK_SIZE as f32;
            chunk.get_height(local_x, local_z)
        } else {
            // Fall back to generator for unloaded chunks
            self.generator.get_height(world_pos.x, world_pos.z)
        }
    }
    
    /// Get surface type at world coordinates (x, z)
    pub fn get_surface_type_at(&self, x: f32, z: f32) -> crate::world::SurfaceType {
        let height = self.get_height(x, z);
        self.generator.get_surface_type(x, z, height)
    }
    
    /// Raycast against the terrain
    pub fn raycast_terrain(&self, origin: Point3<f32>, direction: Vector3<f32>, max_distance: f32) -> Option<(Point3<f32>, Vector3<f32>)> {
        // Simple raycast implementation - can be optimized with spatial index
        let step = 0.5; // Step size in meters
        let mut current = origin;
        
        for _ in 0..((max_distance / step) as usize) {
            let height = self.get_height_at(Vector3::new(current.x, 0.0, current.z));
            
            if current.y <= height {
                // Hit terrain - calculate normal
                let sample_dist = 1.0;
                let h_left = self.get_height_at(Vector3::new(current.x - sample_dist, 0.0, current.z));
                let h_right = self.get_height_at(Vector3::new(current.x + sample_dist, 0.0, current.z));
                let h_back = self.get_height_at(Vector3::new(current.x, 0.0, current.z - sample_dist));
                let h_front = self.get_height_at(Vector3::new(current.x, 0.0, current.z + sample_dist));
                
                let normal = Vector3::new(h_left - h_right, 2.0 * sample_dist, h_back - h_front);
                let normal = normal.normalize();
                
                return Some((current, normal));
            }
            
            current += direction * step;
        }
        
        None
    }
    
    /// Get all loaded chunks
    pub fn get_loaded_chunks(&self) -> &HashMap<ChunkId, Arc<Chunk>> {
        &self.loaded_chunks
    }
    
    /// Get the terrain generator (immutable)
    pub fn get_generator(&self) -> &TerrainGenerator {
        &self.generator
    }
    
    /// Get the terrain generator (mutable)
    pub fn generator_mut(&mut self) -> &mut TerrainGenerator {
        &mut self.generator
    }
    
    /// Get the LOD system
    pub fn get_lod_system(&self) -> &LodSystem {
        &self.lod_system
    }
    
    /// Get the spatial index
    pub fn get_spatial_index(&self) -> &SpatialIndex {
        &self.spatial_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_world_creation() {
        let world = OpenWorld::new(12345);
        assert_eq!(world.loaded_chunks.len(), 0);
    }
    
    #[test]
    fn test_chunk_coordinate_conversion() {
        let world = OpenWorld::new(12345);
        
        // Test at origin
        let chunk = world.world_to_chunk(Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(chunk, ChunkId::new(0, 0));
        
        // Test positive coordinates
        let chunk = world.world_to_chunk(Vector3::new(CHUNK_SIZE as f32 * 2.5, 0.0, CHUNK_SIZE as f32 * 1.5));
        assert_eq!(chunk, ChunkId::new(2, 1));
        
        // Test negative coordinates
        let chunk = world.world_to_chunk(Vector3::new(-CHUNK_SIZE as f32 * 0.5, 0.0, -CHUNK_SIZE as f32 * 0.5));
        assert_eq!(chunk, ChunkId::new(-1, -1));
    }
    
    #[test]
    fn test_chunks_in_radius() {
        let world = OpenWorld::new(12345);
        let center = ChunkId::new(0, 0);
        
        let chunks = world.get_chunks_in_radius(center, 1);
        
        // Should include center and 4 adjacent chunks (not diagonals at radius 1)
        assert!(chunks.len() >= 5);
        assert!(chunks.contains(&ChunkId::new(0, 0)));
        assert!(chunks.contains(&ChunkId::new(1, 0)));
        assert!(chunks.contains(&ChunkId::new(-1, 0)));
        assert!(chunks.contains(&ChunkId::new(0, 1)));
        assert!(chunks.contains(&ChunkId::new(0, -1)));
    }
    
    #[test]
    fn test_height_sampling() {
        let mut world = OpenWorld::new(12345);
        
        // Update world to load some chunks
        world.update(Vector3::new(0.0, 0.0, 0.0), 0.016);
        
        // Height should be deterministic
        let h1 = world.get_height_at(Vector3::new(0.0, 0.0, 0.0));
        let h2 = world.get_height_at(Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(h1, h2);
    }
}
