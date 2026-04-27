//! Open World Scene

use super::super::scene::{Scene, SceneType};
use std::any::Any;
use std::time::{SystemTime, UNIX_EPOCH};

/// Simplex noise implementation for terrain generation
mod noise {
    pub fn simplex_2d(x: f32, z: f32) -> f32 {
        // Simplified value noise for terrain height
        let x = x * 0.01;
        let z = z * 0.01;
        
        // Combine multiple octaves for more natural terrain
        let mut height = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_height = 0.0;
        
        for _ in 0..4 {
            height += amplitude * pseudo_noise(x * frequency, z * frequency);
            max_height += amplitude;
            amplitude *= 0.5;
            frequency *= 2.0;
        }
        
        height / max_height
    }
    
    fn pseudo_noise(x: f32, z: f32) -> f32 {
        // Simple pseudo-random noise based on coordinates
        let n = (x.sin() * 127.1 + z.cos() * 311.7).sin() * 43758.5453;
        n.fract() * 2.0 - 1.0
    }
}

pub struct OpenWorldScene {
    name: String,
    world_loaded: bool,
    seed: u64, // Seed for terrain generation
    terrain_data: Vec<f32>, // Cached terrain heights
    terrain_resolution: u32,
    terrain_size: f32,
}

impl OpenWorldScene {
    pub fn new() -> Self {
        // Используем текущее время в миллисекундах как seed для уникальности миров
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u32;
        
        Self {
            name: "Open World".to_string(),
            world_loaded: false,
            seed: seed as u64,
            terrain_data: Vec::new(),
            terrain_resolution: 256,
            terrain_size: 10000.0, // 10km x 10km world
        }
    }

    pub fn load_world(&mut self) {
        self.generate_terrain();
        self.world_loaded = true;
        tracing::info!("Open world loaded with seed {}", self.seed);
    }

    /// Get terrain height at given coordinates using procedural generation
    pub fn get_height(&self, x: f32, z: f32) -> f32 {
        if !self.terrain_data.is_empty() {
            // Use cached terrain data if available
            let grid_x = ((x + self.terrain_size / 2.0) / self.terrain_size * self.terrain_resolution as f32) as i32;
            let grid_z = ((z + self.terrain_size / 2.0) / self.terrain_size * self.terrain_resolution as f32) as i32;
            
            if grid_x >= 0 && grid_x < self.terrain_resolution as i32 && 
               grid_z >= 0 && grid_z < self.terrain_resolution as i32 {
                let index = (grid_z * self.terrain_resolution as i32 + grid_x) as usize;
                return self.terrain_data.get(index).copied().unwrap_or(0.0);
            }
        }
        
        // Fallback to procedural generation
        let scaled_x = x + self.seed as f32 * 0.001;
        let scaled_z = z + self.seed as f32 * 0.001;
        noise::simplex_2d(scaled_x, scaled_z) * 150.0 // Height range: -150 to 150 meters
    }

    /// Generate terrain data using procedural noise
    pub fn generate_terrain(&mut self) {
        tracing::info!("Generating terrain with seed {}", self.seed);
        
        self.terrain_data.clear();
        self.terrain_data.reserve((self.terrain_resolution * self.terrain_resolution) as usize);
        
        let step = self.terrain_size / self.terrain_resolution as f32;
        let offset = self.terrain_size / 2.0;
        
        for z in 0..self.terrain_resolution {
            for x in 0..self.terrain_resolution {
                let world_x = (x as f32 * step) - offset;
                let world_z = (z as f32 * step) - offset;
                
                let scaled_x = world_x + self.seed as f32 * 0.001;
                let scaled_z = world_z + self.seed as f32 * 0.001;
                
                let height = noise::simplex_2d(scaled_x, scaled_z) * 150.0;
                self.terrain_data.push(height);
            }
        }
        
        tracing::debug!("Terrain generated: {}x{} resolution, {} data points", 
            self.terrain_resolution, self.terrain_resolution, self.terrain_data.len());
    }

    /// Get the world seed
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Set the world seed
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
        // Regenerate terrain when seed changes
        self.generate_terrain();
    }
    
    /// Get terrain size in meters
    pub fn terrain_size(&self) -> f32 {
        self.terrain_size
    }
    
    /// Get terrain resolution
    pub fn terrain_resolution(&self) -> u32 {
        self.terrain_resolution
    }
}

impl Default for OpenWorldScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for OpenWorldScene {
    fn scene_type(&self) -> SceneType {
        SceneType::OpenWorld
    }

    fn on_enter(&mut self) {
        tracing::info!("Entering Open World");
        self.load_world();
    }

    fn on_exit(&mut self) {
        tracing::info!("Exiting Open World");
    }

    fn update(&mut self, _delta_time: f32) {
        // Update world entities, physics, etc.
    }

    fn render(
        &mut self,
        _renderer: &mut crate::graphics::renderer::Renderer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Render the open world - handled by engine renderer
        // This scene uses the full 3D renderer, not UI
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
