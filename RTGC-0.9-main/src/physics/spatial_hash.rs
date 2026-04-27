use nalgebra::Vector3;
use std::collections::HashMap;

/// Spatial hash structure for efficient broad-phase collision detection
/// Uses a 3D grid with configurable cell size and supports AABB-based insertion
#[derive(Clone)]
pub struct SpatialHash {
    cell_size: f32,
    inv_cell_size: f32, // Pre-computed inverse for faster division
    hash_map: HashMap<(i32, i32, i32), Vec<usize>>,
}

impl SpatialHash {
    /// Creates a new spatial hash with the specified cell size
    /// Cell size should be chosen based on the average object size in the scene
    pub fn new(cell_size: f32) -> Self {
        assert!(cell_size > 0.0, "Cell size must be positive");
        Self {
            cell_size,
            inv_cell_size: 1.0 / cell_size,
            hash_map: HashMap::with_capacity(1024), // Pre-allocate to reduce reallocations
        }
    }

    /// Hashes 3D cell coordinates to a normalized space to handle large worlds
    #[inline]
    fn hash_coords(&self, coords: (i32, i32, i32)) -> (i32, i32, i32) {
        // Use a prime-based hashing strategy for better distribution
        let hash_coord = |x: i32| -> i32 {
            // XOR-based folding for negative numbers
            if x >= 0 {
                x % 1000003 // Prime number for better distribution
            } else {
                ((x % 1000003) + 1000003) % 1000003
            }
        };

        (
            hash_coord(coords.0),
            hash_coord(coords.1),
            hash_coord(coords.2),
        )
    }

    /// Converts world position to cell coordinates
    #[inline]
    fn world_to_cell(&self, pos: &Vector3<f32>) -> (i32, i32, i32) {
        (
            (pos.x * self.inv_cell_size).floor() as i32,
            (pos.y * self.inv_cell_size).floor() as i32,
            (pos.z * self.inv_cell_size).floor() as i32,
        )
    }

    /// Clears all entries from the spatial hash
    /// Call this every frame before re-inserting dynamic objects
    pub fn clear(&mut self) {
        self.hash_map.clear();
    }

    /// Inserts an object at the given position into the spatial hash
    /// For AABB objects, use insert_aabb instead
    pub fn insert(&mut self, body_index: usize, position: &Vector3<f32>) {
        // Validate position to prevent NaN from breaking spatial hashing
        if !position.x.is_finite() || !position.y.is_finite() || !position.z.is_finite() {
            tracing::warn!(target: "physics", "NaN position in SpatialHash::insert, skipping");
            return;
        }
        
        let cell_coords = self.world_to_cell(position);
        let hashed_coords = self.hash_coords(cell_coords);

        self.hash_map
            .entry(hashed_coords)
            .or_insert_with(Vec::new)
            .push(body_index);
    }

    /// Inserts an AABB-defined object into all overlapping cells
    /// This ensures objects spanning multiple cells are found in all relevant cells
    pub fn insert_aabb(&mut self, body_index: usize, min: &Vector3<f32>, max: &Vector3<f32>) {
        // Validate AABB bounds
        if !min.x.is_finite() || !min.y.is_finite() || !min.z.is_finite() {
            tracing::warn!(target: "physics", "NaN in AABB min, skipping insert_aabb");
            return;
        }
        if !max.x.is_finite() || !max.y.is_finite() || !max.z.is_finite() {
            tracing::warn!(target: "physics", "NaN in AABB max, skipping insert_aabb");
            return;
        }
        
        let min_cell = self.world_to_cell(min);
        let max_cell = self.world_to_cell(max);

        // Iterate through all cells the AABB overlaps
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    let hashed_coords = self.hash_coords((x, y, z));
                    self.hash_map
                        .entry(hashed_coords)
                        .or_insert_with(Vec::new)
                        .push(body_index);
                }
            }
        }
    }

    /// Gets all potential collision candidates for a point query
    /// Returns indices of objects in the same and neighboring cells
    pub fn get_potential_collisions(&self, position: &Vector3<f32>) -> Vec<usize> {
        // Validate position
        if !position.x.is_finite() || !position.y.is_finite() || !position.z.is_finite() {
            tracing::warn!(target: "physics", "NaN position in get_potential_collisions, returning empty");
            return Vec::new();
        }
        
        let mut candidates = Vec::new();
        let cell_coords = self.world_to_cell(position);

        // Check current cell and neighboring cells (3x3x3 grid)
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let neighbor_coords =
                        (cell_coords.0 + dx, cell_coords.1 + dy, cell_coords.2 + dz);

                    let hashed_coords = self.hash_coords(neighbor_coords);

                    if let Some(bodies) = self.hash_map.get(&hashed_coords) {
                        candidates.extend_from_slice(bodies);
                    }
                }
            }
        }

        candidates
    }

    /// Gets all potential collision candidates for an AABB query
    /// More accurate than point query for larger objects
    pub fn get_potential_collisions_aabb(
        &self,
        min: &Vector3<f32>,
        max: &Vector3<f32>,
    ) -> Vec<usize> {
        let mut candidates = Vec::new();
        let min_cell = self.world_to_cell(min);
        let max_cell = self.world_to_cell(max);

        // Track seen indices to avoid duplicates
        let mut seen = std::collections::HashSet::new();

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    let hashed_coords = self.hash_coords((x, y, z));

                    if let Some(bodies) = self.hash_map.get(&hashed_coords) {
                        for &body_index in bodies {
                            if seen.insert(body_index) {
                                candidates.push(body_index);
                            }
                        }
                    }
                }
            }
        }

        candidates
    }

    /// Returns the current cell size
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Returns the approximate memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        let map_overhead =
            self.hash_map.capacity() * std::mem::size_of::<((i32, i32, i32), Vec<usize>)>();
        let vec_overhead = self
            .hash_map
            .values()
            .map(|v| v.capacity() * std::mem::size_of::<usize>())
            .sum::<usize>();
        map_overhead + vec_overhead
    }

    /// Returns the total number of entries across all cells
    pub fn total_entries(&self) -> usize {
        self.hash_map.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_hash_insert_and_query() {
        let mut hash = SpatialHash::new(10.0);

        // Insert some objects
        hash.insert(0, &Vector3::new(5.0, 5.0, 5.0));
        hash.insert(1, &Vector3::new(15.0, 5.0, 5.0));
        hash.insert(2, &Vector3::new(100.0, 100.0, 100.0));

        // Query near first object
        let candidates = hash.get_potential_collisions(&Vector3::new(5.0, 5.0, 5.0));
        assert!(candidates.contains(&0));

        // Query near second object (should also find first due to adjacent cells)
        let candidates = hash.get_potential_collisions(&Vector3::new(15.0, 5.0, 5.0));
        assert!(candidates.contains(&1));

        // Query far away (should only find object 2)
        let candidates = hash.get_potential_collisions(&Vector3::new(100.0, 100.0, 100.0));
        assert!(candidates.contains(&2));
        assert!(!candidates.contains(&0));
        assert!(!candidates.contains(&1));
    }

    #[test]
    fn test_aabb_insert() {
        let mut hash = SpatialHash::new(10.0);

        // Insert a large AABB spanning multiple cells
        let min = Vector3::new(-5.0, -5.0, -5.0);
        let max = Vector3::new(15.0, 15.0, 15.0);
        hash.insert_aabb(0, &min, &max);

        // Query at different points within the AABB should all find the object
        assert!(hash
            .get_potential_collisions(&Vector3::new(0.0, 0.0, 0.0))
            .contains(&0));
        assert!(hash
            .get_potential_collisions(&Vector3::new(10.0, 10.0, 10.0))
            .contains(&0));
        assert!(hash
            .get_potential_collisions(&Vector3::new(-5.0, -5.0, -5.0))
            .contains(&0));
    }

    #[test]
    fn test_negative_coordinates() {
        let mut hash = SpatialHash::new(10.0);

        // Test with negative coordinates
        hash.insert(0, &Vector3::new(-50.0, -50.0, -50.0));
        hash.insert(1, &Vector3::new(50.0, 50.0, 50.0));

        let candidates_neg = hash.get_potential_collisions(&Vector3::new(-50.0, -50.0, -50.0));
        assert!(candidates_neg.contains(&0));
        assert!(!candidates_neg.contains(&1));

        let candidates_pos = hash.get_potential_collisions(&Vector3::new(50.0, 50.0, 50.0));
        assert!(candidates_pos.contains(&1));
        assert!(!candidates_pos.contains(&0));
    }
}
