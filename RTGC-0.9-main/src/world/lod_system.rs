//! Level of Detail (LOD) system for terrain and objects
//! 
//! Implements:
//! - Distance-based LOD switching
//! - Smooth LOD transitions (geomorphing)
//! - Hysteresis to prevent popping
//! - Per-object LOD thresholds

use std::collections::HashMap;
use nalgebra::Vector3;
use tracing::debug;

use super::chunk::{Chunk, ChunkId};

/// LOD configuration
#[derive(Debug, Clone)]
pub struct LodConfig {
    /// Number of LOD levels
    pub num_levels: u32,
    /// Base distance for LOD 0
    pub base_distance: f32,
    /// Distance multiplier per level
    pub distance_multiplier: f32,
    /// Hysteresis factor (0.0-1.0) to prevent popping
    pub hysteresis: f32,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            num_levels: 4,
            base_distance: 50.0,
            distance_multiplier: 2.0,
            hysteresis: 0.8, // Need to get 20% closer to increase detail
        }
    }
}

/// LOD state for a single chunk
#[derive(Debug)]
pub struct ChunkLodState {
    /// Current LOD level
    pub current_lod: u32,
    /// Previous LOD level (for transitions)
    pub previous_lod: u32,
    /// Transition progress (0.0-1.0)
    pub transition_progress: f32,
    /// Is currently transitioning?
    pub is_transitioning: bool,
    /// Distance to camera at last update
    pub last_distance: f32,
}

impl ChunkLodState {
    pub fn new() -> Self {
        Self {
            current_lod: 0,
            previous_lod: 0,
            transition_progress: 1.0,
            is_transitioning: false,
            last_distance: 0.0,
        }
    }
}

impl Default for ChunkLodState {
    fn default() -> Self {
        Self::new()
    }
}

/// LOD management system
pub struct LodSystem {
    /// Configuration
    config: LodConfig,
    /// LOD states for each chunk
    chunk_states: HashMap<ChunkId, ChunkLodState>,
    /// LOD distances cache
    lod_distances: Vec<f32>,
}

impl LodSystem {
    pub fn new() -> Self {
        let config = LodConfig::default();
        let lod_distances = Self::calculate_lod_distances(&config);
        
        Self {
            config,
            chunk_states: HashMap::new(),
            lod_distances,
        }
    }
    
    pub fn with_config(config: LodConfig) -> Self {
        let lod_distances = Self::calculate_lod_distances(&config);
        
        Self {
            config,
            chunk_states: HashMap::new(),
            lod_distances,
        }
    }
    
    /// Calculate LOD switch distances based on config
    fn calculate_lod_distances(config: &LodConfig) -> Vec<f32> {
        let mut distances = Vec::with_capacity(config.num_levels as usize);
        
        for i in 0..config.num_levels {
            let dist = config.base_distance * config.distance_multiplier.powi(i as i32);
            distances.push(dist);
        }
        
        distances
    }
    
    /// Update LOD for all chunks based on camera position
    pub fn update(&mut self, chunks: &HashMap<ChunkId, std::sync::Arc<Chunk>>, camera_pos: Vector3<f32>) {
        let dt = 0.016; // Assume ~60 FPS for transition timing

        // First pass: collect chunk IDs and distances
        let chunk_distances: Vec<(ChunkId, f32)> = chunks
            .iter()
            .map(|(chunk_id, chunk)| (*chunk_id, chunk.distance_to(camera_pos)))
            .collect();

        // Second pass: update LOD states
        for (chunk_id, distance) in chunk_distances {
            // Исправление: выносим determine_lod за пределы borrow entry()
            // Сначала получаем last_distance из существующего состояния (без mutable borrow)
            let last_distance = self.chunk_states.get(&chunk_id)
                .map(|s| s.last_distance)
                .unwrap_or(0.0);

            // Вычисляем target LOD до получения mutable borrow через entry()
            // Исправление: determine_lod вызывается когда self не заиммован
            let target_lod = self.determine_lod(distance, last_distance);

            // Get or create LOD state for this chunk
            let state = self.chunk_states.entry(chunk_id).or_insert_with(ChunkLodState::new);

            // Determine target LOD based on distance (уже вычислено выше)
            // target_lod уже готов

            // Check if LOD changed
            if target_lod != state.current_lod {
                if !state.is_transitioning {
                    // Start transition
                    state.previous_lod = state.current_lod;
                    state.current_lod = target_lod;
                    state.transition_progress = 0.0;
                    state.is_transitioning = true;
                    
                    debug!(
                        "Chunk {:?} LOD transition: {} -> {} at distance {:.1}",
                        chunk_id, state.previous_lod, state.current_lod, distance
                    );
                }
            }
            
            // Update transition progress
            if state.is_transitioning {
                state.transition_progress += dt / 0.5; // 0.5 second transition
                
                if state.transition_progress >= 1.0 {
                    state.transition_progress = 1.0;
                    state.is_transitioning = false;
                    state.previous_lod = state.current_lod;
                }
            }
            
            state.last_distance = distance;
        }
        
        // Clean up states for unloaded chunks
        self.chunk_states.retain(|chunk_id, _| chunks.contains_key(chunk_id));
    }
    
    /// Determine appropriate LOD level with hysteresis
    fn determine_lod(&self, distance: f32, last_distance: f32) -> u32 {
        // Apply hysteresis to prevent rapid LOD switching
        let hysteresis_factor = if distance > last_distance {
            // Moving away - use higher threshold to delay downgrade
            1.0 / self.config.hysteresis
        } else {
            // Moving closer - use lower threshold to upgrade sooner
            self.config.hysteresis
        };
        
        let adjusted_distance = distance * hysteresis_factor;
        
        // Find appropriate LOD level
        for (lod, &threshold) in self.lod_distances.iter().enumerate() {
            if adjusted_distance < threshold {
                return lod as u32;
            }
        }
        
        // Use highest LOD (lowest detail) for very far distances
        self.config.num_levels - 1
    }
    
    /// Get current LOD for a chunk
    pub fn get_lod(&self, chunk_id: ChunkId) -> u32 {
        self.chunk_states
            .get(&chunk_id)
            .map(|s| s.current_lod)
            .unwrap_or(0)
    }
    
    /// Get LOD state for a chunk
    pub fn get_lod_state(&self, chunk_id: ChunkId) -> Option<&ChunkLodState> {
        self.chunk_states.get(&chunk_id)
    }
    
    /// Get transition interpolation factor (0.0 = previous LOD, 1.0 = current LOD)
    pub fn get_transition_alpha(&self, chunk_id: ChunkId) -> f32 {
        self.chunk_states
            .get(&chunk_id)
            .map(|s| s.transition_progress)
            .unwrap_or(1.0)
    }
    
    /// Force immediate LOD update without transition
    pub fn set_lod(&mut self, chunk_id: ChunkId, lod: u32) {
        let state = self.chunk_states.entry(chunk_id).or_insert_with(ChunkLodState::new);
        state.current_lod = lod.min(self.config.num_levels - 1);
        state.previous_lod = state.current_lod;
        state.is_transitioning = false;
        state.transition_progress = 1.0;
    }
    
    /// Get recommended LOD distances for rendering
    pub fn get_lod_distances(&self) -> &[f32] {
        &self.lod_distances
    }
    
    /// Update LOD configuration
    pub fn set_config(&mut self, config: LodConfig) {
        self.config = config;
        self.lod_distances = Self::calculate_lod_distances(&self.config);
    }
    
    /// Get statistics about LOD system
    pub fn get_stats(&self) -> LodStats {
        let mut stats = LodStats::default();
        
        for state in self.chunk_states.values() {
            stats.total_chunks += 1;
            
            match state.current_lod {
                0 => stats.lod0_count += 1,
                1 => stats.lod1_count += 1,
                2 => stats.lod2_count += 1,
                _ => stats.high_lod_count += 1,
            }
            
            if state.is_transitioning {
                stats.transitioning_count += 1;
            }
        }
        
        stats
    }
}

impl Default for LodSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about LOD system state
#[derive(Debug, Default)]
pub struct LodStats {
    pub total_chunks: u32,
    pub lod0_count: u32,
    pub lod1_count: u32,
    pub lod2_count: u32,
    pub high_lod_count: u32,
    pub transitioning_count: u32,
}

impl LodStats {
    /// Get percentage of chunks at highest detail
    pub fn highest_detail_percentage(&self) -> f32 {
        if self.total_chunks == 0 {
            0.0
        } else {
            self.lod0_count as f32 / self.total_chunks as f32 * 100.0
        }
    }
}

/// LOD component for individual objects (props, buildings, etc.)
#[derive(Debug, Clone)]
pub struct ObjectLod {
    /// Current LOD level
    pub lod: u32,
    /// LOD switch distances
    pub distances: [f32; 4],
    /// Is object visible at current LOD?
    pub visible: bool,
}

impl ObjectLod {
    pub fn new(distances: [f32; 4]) -> Self {
        Self {
            lod: 0,
            distances,
            visible: true,
        }
    }
    
    /// Update LOD based on distance
    pub fn update(&mut self, distance: f32) {
        if distance < self.distances[0] {
            self.lod = 0;
        } else if distance < self.distances[1] {
            self.lod = 1;
        } else if distance < self.distances[2] {
            self.lod = 2;
        } else {
            self.lod = 3;
        }
        
        // Hide if beyond max distance
        self.visible = distance < self.distances[3];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lod_distances_calculation() {
        let config = LodConfig {
            num_levels: 4,
            base_distance: 50.0,
            distance_multiplier: 2.0,
            hysteresis: 0.8,
        };
        
        let distances = LodSystem::calculate_lod_distances(&config);
        
        assert_eq!(distances.len(), 4);
        assert_eq!(distances[0], 50.0);
        assert_eq!(distances[1], 100.0);
        assert_eq!(distances[2], 200.0);
        assert_eq!(distances[3], 400.0);
    }
    
    #[test]
    fn test_lod_determination() {
        let system = LodSystem::new();
        
        // Close distance should be LOD 0
        let lod = system.determine_lod(25.0, 25.0);
        assert_eq!(lod, 0);
        
        // Medium distance
        let lod = system.determine_lod(75.0, 75.0);
        assert_eq!(lod, 1);
        
        // Far distance
        let lod = system.determine_lod(300.0, 300.0);
        assert_eq!(lod, 2);
        
        // Very far
        let lod = system.determine_lod(500.0, 500.0);
        assert_eq!(lod, 3);
    }
    
    #[test]
    fn test_hysteresis() {
        let system = LodSystem::new();
        
        // At boundary distance
        let lod_moving_closer = system.determine_lod(50.0, 100.0); // Was far, now close
        let lod_moving_away = system.determine_lod(50.0, 25.0);   // Was close, now far
        
        // Should have different LODs due to hysteresis
        assert!(lod_moving_closer <= lod_moving_away);
    }
    
    #[test]
    fn test_object_lod() {
        let mut obj_lod = ObjectLod::new([20.0, 50.0, 100.0, 200.0]);
        
        obj_lod.update(10.0);
        assert_eq!(obj_lod.lod, 0);
        assert!(obj_lod.visible);
        
        obj_lod.update(35.0);
        assert_eq!(obj_lod.lod, 1);
        assert!(obj_lod.visible);
        
        obj_lod.update(150.0);
        assert_eq!(obj_lod.lod, 3);
        assert!(!obj_lod.visible); // Beyond max distance
    }
}
