//! Placeholder for deformable terrain (rut formation, mud)
use nalgebra::Vector3;

/// Interface for deformable terrain operations
/// This trait defines the contract for terrain deformation functionality
pub trait DeformableTerrainInterface {
    /// Apply a deformation to the terrain at a specific position
    fn apply_deformation(
        &mut self,
        position: Vector3<f32>,
        deformation_type: DeformationType,
    ) -> bool;

    /// Get the height at a specific world position
    fn get_height_at(&self, world_pos: Vector3<f32>) -> f32;

    /// Reset terrain to original state
    fn reset_terrain(&mut self);

    /// Apply erosion effects over time
    fn apply_erosion(&mut self, time_delta: f32);
}

/// Component for deformable terrain that allows modification of heightmaps
/// This implements the "деформируемость ландшафта" (terrain deformation) functionality
/// mentioned in the README as part of the "Системы динамического окружения" section
#[derive(Debug, Clone)]
pub struct DeformableTerrainComponent {
    /// Reference to the terrain rigid body
    pub terrain_body_index: usize,

    /// Original heightmap data
    pub original_heightmap: Vec<Vec<f32>>,

    /// Current deformed heightmap
    pub current_heightmap: Vec<Vec<f32>>,

    /// Resolution of the heightmap (number of points in each direction)
    pub resolution: (usize, usize),

    /// Physical properties affecting deformation
    pub deformation_properties: DeformationProperties,

    /// List of deformation events that have occurred
    pub deformation_history: Vec<DeformationEvent>,
}

impl DeformableTerrainComponent {
    /// Validate that all terrain state is finite and safe
    pub fn validate_state(&self) -> bool {
        // Check resolution is valid
        if self.resolution.0 == 0 || self.resolution.1 == 0 {
            return false;
        }

        // Check heightmap dimensions match resolution
        if self.current_heightmap.len() != self.resolution.1 {
            return false;
        }

        // Check all height values are finite
        for row in &self.current_heightmap {
            for &height in row {
                if !height.is_finite() {
                    return false;
                }
            }
        }

        // Check deformation properties
        if !self.deformation_properties.max_dig_depth.is_finite()
            || !self.deformation_properties.max_build_height.is_finite()
        {
            return false;
        }

        true
    }

    /// Reset terrain to a safe state
    pub fn reset_to_safe_state(&mut self) {
        tracing::warn!(target: "physics", "Resetting deformable terrain to safe state");
        self.current_heightmap = self.original_heightmap.clone();
        self.deformation_history.clear();
    }
}

impl DeformableTerrainInterface for DeformableTerrainComponent {
    fn apply_deformation(
        &mut self,
        position: Vector3<f32>,
        deformation_type: DeformationType,
    ) -> bool {
        self.apply_deformation(position, deformation_type)
    }

    fn get_height_at(&self, world_pos: Vector3<f32>) -> f32 {
        self.get_height_at(world_pos)
    }

    fn reset_terrain(&mut self) {
        self.reset_terrain()
    }

    fn apply_erosion(&mut self, time_delta: f32) {
        self.apply_erosion(time_delta)
    }
}

impl DeformableTerrainComponent {
    pub fn new(terrain_body_index: usize, width: usize, depth: usize) -> Self {
        let mut heightmap = vec![vec![0.0; width]; depth];

        // Initialize with some base terrain variation
        for i in 0..depth {
            for j in 0..width {
                heightmap[i][j] = (i as f32 * 0.1).sin() * (j as f32 * 0.1).cos() * 0.5;
            }
        }

        Self {
            terrain_body_index,
            original_heightmap: heightmap.clone(),
            current_heightmap: heightmap,
            resolution: (width, depth),
            deformation_properties: DeformationProperties::default(),
            deformation_history: Vec::new(),
        }
    }

    /// Apply a deformation to the terrain at a specific position
    pub fn apply_deformation(
        &mut self,
        position: Vector3<f32>,
        deformation_type: DeformationType,
    ) -> bool {
        // Validate position to prevent NaN/Inf propagation
        if !position.x.is_finite() || !position.y.is_finite() || !position.z.is_finite() {
            tracing::warn!(target: "physics", "Invalid position in terrain deformation: {:?}, skipping", position);
            return false;
        }

        let (grid_x, grid_z) = self.world_to_grid(position);

        if grid_x < 0
            || grid_z < 0
            || grid_x >= self.resolution.0 as i32
            || grid_z >= self.resolution.1 as i32
        {
            return false; // Position out of bounds
        }

        let grid_x = grid_x as usize;
        let grid_z = grid_z as usize;

        // Apply the deformation based on type
        match deformation_type {
            DeformationType::Dig(depth) => {
                if depth.is_finite() && depth > 0.0 {
                    self.current_heightmap[grid_z][grid_x] -= depth;
                }
            }
            DeformationType::Build(height) => {
                if height.is_finite() && height > 0.0 {
                    self.current_heightmap[grid_z][grid_x] += height;
                }
            }
            DeformationType::Press(force) => {
                if force.is_finite() && force > 0.0 {
                    // Apply pressure that creates a depression with surrounding uplift
                    self.apply_pressure_deformation(grid_x, grid_z, force);
                }
            }
            DeformationType::Smooth(factor) => {
                if factor.is_finite() && factor > 0.0 && factor <= 1.0 {
                    self.smooth_around_point(grid_x, grid_z, factor);
                }
            }
        }

        // Record the deformation event
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::ZERO)
            .as_secs_f32();

        self.deformation_history.push(DeformationEvent {
            position,
            deformation_type: deformation_type.clone(),
            timestamp,
        });

        true
    }

    /// Apply a pressure-based deformation that affects a circular area
    fn apply_pressure_deformation(&mut self, center_x: usize, center_z: usize, force: f32) {
        let radius = (force * self.deformation_properties.pressure_radius_multiplier) as i32;
        let max_displacement = force * self.deformation_properties.pressure_depth_multiplier;

        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let dist_sq = dx * dx + dz * dz;
                if dist_sq <= radius * radius {
                    let dist = (dist_sq as f32).sqrt();
                    let normalized_dist = dist / radius as f32;

                    // Use a smooth falloff function (quadratic for pressure)
                    let displacement = max_displacement * (1.0 - normalized_dist * normalized_dist);

                    let x = (center_x as i32 + dx) as usize;
                    let z = (center_z as i32 + dz) as usize;

                    if x < self.resolution.0 && z < self.resolution.1 {
                        self.current_heightmap[z][x] -= displacement;

                        // Apply some rebound effect at the edges
                        if normalized_dist > 0.7 {
                            let rebound_effect = displacement * 0.2;
                            self.current_heightmap[z][x] += rebound_effect;
                        }
                    }
                }
            }
        }
    }

    /// Smooth terrain around a point using averaging
    fn smooth_around_point(&mut self, center_x: usize, center_z: usize, factor: f32) {
        const SMOOTH_RADIUS: usize = 2;

        for dz in 0..=SMOOTH_RADIUS * 2 {
            for dx in 0..=SMOOTH_RADIUS * 2 {
                let x = center_x + dx - SMOOTH_RADIUS;
                let z = center_z + dz - SMOOTH_RADIUS;

                if x < self.resolution.0 && z < self.resolution.1 {
                    // Calculate average of neighbors
                    let mut sum = 0.0;
                    let mut count = 0;

                    for nz in z.saturating_sub(1)..=(z + 1).min(self.resolution.1 - 1) {
                        for nx in x.saturating_sub(1)..=(x + 1).min(self.resolution.0 - 1) {
                            sum += self.current_heightmap[nz][nx];
                            count += 1;
                        }
                    }

                    if count > 0 {
                        let avg = sum / count as f32;
                        self.current_heightmap[z][x] =
                            self.current_heightmap[z][x] * (1.0 - factor) + avg * factor;
                    }
                }
            }
        }
    }

    /// Convert world coordinates to grid coordinates
    fn world_to_grid(&self, world_pos: Vector3<f32>) -> (i32, i32) {
        // This is a simplified conversion - in a real implementation,
        // you'd need to consider the actual terrain dimensions
        let cell_size = 1.0; // Assume 1 unit per grid cell for simplicity
        let grid_x = (world_pos.x / cell_size) as i32;
        let grid_z = (world_pos.z / cell_size) as i32;

        (grid_x, grid_z)
    }

    /// Get the height at a specific world position
    pub fn get_height_at(&self, world_pos: Vector3<f32>) -> f32 {
        let (grid_x, grid_z) = self.world_to_grid(world_pos);

        if grid_x < 0
            || grid_z < 0
            || grid_x >= self.resolution.0 as i32
            || grid_z >= self.resolution.1 as i32
        {
            return 0.0; // Out of bounds
        }

        let grid_x = grid_x as usize;
        let grid_z = grid_z as usize;

        self.current_heightmap[grid_z][grid_x]
    }

    /// Reset terrain to original state
    pub fn reset_terrain(&mut self) {
        self.current_heightmap = self.original_heightmap.clone();
        self.deformation_history.clear();
    }

    /// Apply erosion effects over time to simulate natural processes
    pub fn apply_erosion(&mut self, time_delta: f32) {
        if self.deformation_properties.enable_erosion {
            // Simple diffusion-based erosion
            let erosion_amount = time_delta * self.deformation_properties.erosion_rate;

            // Create a temporary copy of the heightmap for calculations
            let mut new_heightmap = self.current_heightmap.clone();

            for z in 1..(self.resolution.1 - 1) {
                for x in 1..(self.resolution.0 - 1) {
                    // Calculate height differences with neighbors
                    let current_height = self.current_heightmap[z][x];
                    let mut neighbor_sum = 0.0;
                    let mut valid_neighbors = 0;

                    // Check 4-connected neighbors
                    for &(dx, dz) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                        let nx = x as i32 + dx;
                        let nz = z as i32 + dz;

                        if nx >= 0
                            && nx < self.resolution.0 as i32
                            && nz >= 0
                            && nz < self.resolution.1 as i32
                        {
                            neighbor_sum += self.current_heightmap[nz as usize][nx as usize];
                            valid_neighbors += 1;
                        }
                    }

                    if valid_neighbors > 0 {
                        let avg_neighbor_height = neighbor_sum / valid_neighbors as f32;
                        let height_diff = current_height - avg_neighbor_height;

                        // Only erode if there's a significant slope
                        if height_diff.abs() > self.deformation_properties.erosion_threshold {
                            new_heightmap[z][x] = current_height - height_diff * erosion_amount;
                        }
                    }
                }
            }

            self.current_heightmap = new_heightmap;
        }
    }

    /// Get reference to current heightmap for rendering
    pub fn get_heightmap(&self) -> &Vec<Vec<f32>> {
        &self.current_heightmap
    }

    /// Get resolution of the heightmap
    pub fn get_resolution(&self) -> (usize, usize) {
        self.resolution
    }

    /// Get list of modified regions since last query (for incremental renderer updates)
    pub fn get_modified_regions(&self) -> Vec<(usize, usize)> {
        // Returns grid coordinates that have been modified
        // In a full implementation, this would track changes since last call
        // For now, return all points that differ from original
        let mut modified = Vec::new();
        for z in 0..self.resolution.1 {
            for x in 0..self.resolution.0 {
                if (self.current_heightmap[z][x] - self.original_heightmap[z][x]).abs() > 0.001 {
                    modified.push((x, z));
                }
            }
        }
        modified
    }

    /// Clear modification history after renderer has updated
    pub fn clear_modification_history(&mut self) {
        // In a full implementation, this would reset the dirty tracking
        // For now, we just keep the history for debugging
        tracing::debug!("Cleared terrain modification history");
    }
}

#[derive(Debug, Clone)]
pub enum DeformationType {
    /// Dig down by specified depth
    Dig(f32),
    /// Build up by specified height
    Build(f32),
    /// Apply pressure that depresses terrain (like vehicle tracks)
    Press(f32),
    /// Smooth terrain around the point
    Smooth(f32),
}

#[derive(Debug, Clone)]
pub struct DeformationProperties {
    /// How much terrain is displaced per unit of pressure force
    pub pressure_depth_multiplier: f32,

    /// How far the pressure effect spreads
    pub pressure_radius_multiplier: f32,

    /// Rate of erosion over time
    pub erosion_rate: f32,

    /// Threshold for slope before erosion applies
    pub erosion_threshold: f32,

    /// Whether to enable automatic erosion
    pub enable_erosion: bool,

    /// Maximum depth that can be dug at one location
    pub max_dig_depth: f32,

    /// Maximum height that can be built at one location
    pub max_build_height: f32,
}

impl Default for DeformationProperties {
    fn default() -> Self {
        Self {
            pressure_depth_multiplier: 0.1,
            pressure_radius_multiplier: 3.0,
            erosion_rate: 0.01,
            erosion_threshold: 0.5,
            enable_erosion: true,
            max_dig_depth: 5.0,
            max_build_height: 5.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeformationEvent {
    /// World position where deformation occurred
    pub position: Vector3<f32>,

    /// Type and magnitude of deformation
    pub deformation_type: DeformationType,

    /// Time when deformation occurred (seconds since epoch)
    pub timestamp: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_deformation_creation() {
        let mut terrain = DeformableTerrainComponent::new(0, 10, 10);

        assert_eq!(terrain.resolution.0, 10);
        assert_eq!(terrain.resolution.1, 10);
        assert!(terrain.deformation_history.is_empty());
    }

    #[test]
    fn test_apply_dig_deformation() {
        let mut terrain = DeformableTerrainComponent::new(0, 10, 10);

        let position = Vector3::new(5.0, 0.0, 5.0);
        let success = terrain.apply_deformation(position, DeformationType::Dig(1.0));

        assert!(success);
        assert_eq!(terrain.deformation_history.len(), 1);
    }

    #[test]
    fn test_get_height_after_deformation() {
        let mut terrain = DeformableTerrainComponent::new(0, 10, 10);

        let position = Vector3::new(5.0, 0.0, 5.0);
        terrain.apply_deformation(position, DeformationType::Build(2.0));

        let height = terrain.get_height_at(position);
        // Height should be positive due to building
        assert!(height > 0.0);
    }

    #[test]
    fn test_out_of_bounds_access() {
        let terrain = DeformableTerrainComponent::new(0, 10, 10);

        let out_of_bounds_pos = Vector3::new(-10.0, 0.0, -10.0);
        let height = terrain.get_height_at(out_of_bounds_pos);

        assert_eq!(height, 0.0);
    }
}
