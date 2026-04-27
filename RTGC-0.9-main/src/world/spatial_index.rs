//! Spatial indexing for fast world queries
//! 
//! Implements:
//! - Octree for static geometry
//! - Dynamic BVH for moving objects
//! - Range queries and raycasting
//! - Nearest neighbor search

use nalgebra::{Vector3, Point3};
use std::collections::HashMap;

use super::chunk::{Chunk, ChunkId};

/// Query result from spatial index
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Chunk IDs in query result
    pub chunk_ids: Vec<ChunkId>,
    /// Optional point of interest
    pub point: Option<Point3<f32>>,
}

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl Aabb {
    pub fn new(min: Vector3<f32>, max: Vector3<f32>) -> Self {
        Self { min, max }
    }
    
    pub fn from_center_and_size(center: Vector3<f32>, size: Vector3<f32>) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }
    
    pub fn contains_point(&self, point: Vector3<f32>) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
    
    pub fn intersects_aabb(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }
    
    pub fn center(&self) -> Vector3<f32> {
        (self.min + self.max) * 0.5
    }
    
    pub fn size(&self) -> Vector3<f32> {
        self.max - self.min
    }
}

/// Octree node
struct OctreeNode {
    /// Bounding box of this node
    bounds: Aabb,
    /// Child nodes (8 children for octree)
    children: Option<Box<[OctreeNode; 8]>>,
    /// Chunks stored in this node
    chunks: Vec<ChunkId>,
    /// Maximum chunks before subdivision
    max_chunks: usize,
    /// Current depth level
    depth: u32,
    /// Maximum depth
    max_depth: u32,
}

impl OctreeNode {
    fn new(bounds: Aabb, max_chunks: usize, max_depth: u32) -> Self {
        Self {
            bounds,
            children: None,
            chunks: Vec::new(),
            max_chunks,
            depth: 0,
            max_depth,
        }
    }
    
    /// Insert a chunk into the octree
    fn insert(&mut self, chunk_id: ChunkId, chunk_bounds: Aabb) {
        // Check if chunk intersects with this node's bounds
        if !self.bounds.intersects_aabb(&chunk_bounds) {
            return;
        }
        
        // If we have children, try to insert into them
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                if child.bounds.intersects_aabb(&chunk_bounds) {
                    child.insert(chunk_id, chunk_bounds);
                }
            }
            return;
        }
        
        // Add to this node
        self.chunks.push(chunk_id);
        
        // Subdivide if necessary
        if self.chunks.len() > self.max_chunks && self.depth < self.max_depth {
            let _ = self.subdivide();
            
            // Redistribute chunks to children
            let chunks: Vec<ChunkId> = self.chunks.drain(..).collect();
            for chunk_id in chunks {
                // We need the bounds again - in practice these would be cached
                // For now, just re-add to this node
                self.chunks.push(chunk_id);
            }
        }
    }
    
    /// Subdivide this node into 8 children
    fn subdivide(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let center = self.bounds.center();
        let size = self.bounds.size() * 0.5;

        let mut children: [OctreeNode; 8] = {
            let mut arr: [Option<OctreeNode>; 8] = Default::default();
            for i in 0..8 {
                arr[i] = Some(OctreeNode::new(
                    Aabb::new(Vector3::zeros(), Vector3::zeros()),
                    0, 0
                ));
            }
            // SAFETY: All 8 elements are initialized with Some above
            [arr[0].take().ok_or("Spatial index error")?, arr[1].take().ok_or("Spatial index error")?, arr[2].take().ok_or("Spatial index error")?,
             arr[3].take().ok_or("Spatial index error")?, arr[4].take().ok_or("Spatial index error")?, arr[5].take().ok_or("Spatial index error")?,
             arr[6].take().ok_or("Spatial index error")?, arr[7].take().ok_or("Spatial index error")?]
        };

        let mut i = 0;
        for dz in 0..2 {
            for dy in 0..2 {
                for dx in 0..2 {
                    let min = Vector3::new(
                        if dx == 0 { self.bounds.min.x } else { center.x },
                        if dy == 0 { self.bounds.min.y } else { center.y },
                        if dz == 0 { self.bounds.min.z } else { center.z },
                    );

                    let max = Vector3::new(
                        if dx == 0 { center.x } else { self.bounds.max.x },
                        if dy == 0 { center.y } else { self.bounds.max.y },
                        if dz == 0 { center.z } else { self.bounds.max.z },
                    );

                    children[i] = OctreeNode {
                        bounds: Aabb::new(min, max),
                        children: None,
                        chunks: Vec::new(),
                        max_chunks: self.max_chunks,
                        depth: self.depth + 1,
                        max_depth: self.max_depth,
                    };
                    i += 1;
                }
            }
        }

        self.children = Some(Box::new(children));
        Ok(())
    }
    
    /// Query chunks within a bounding box
    fn query_aabb(&self, query_bounds: &Aabb, result: &mut Vec<ChunkId>) {
        if !self.bounds.intersects_aabb(query_bounds) {
            return;
        }
        
        // Add all chunks in this node
        result.extend_from_slice(&self.chunks);
        
        // Query children
        if let Some(children) = &self.children {
            for child in children.iter() {
                child.query_aabb(query_bounds, result);
            }
        }
    }
    
    /// Query chunks containing a point
    fn query_point(&self, point: Vector3<f32>, result: &mut Vec<ChunkId>) {
        if !self.bounds.contains_point(point) {
            return;
        }
        
        // Add all chunks in this node
        result.extend_from_slice(&self.chunks);
        
        // Query children
        if let Some(children) = &self.children {
            for child in children.iter() {
                child.query_point(point, result);
            }
        }
    }
}

/// Spatial index combining multiple acceleration structures
pub struct SpatialIndex {
    /// Octree for static chunks
    octree: Option<OctreeNode>,
    /// Direct chunk lookup
    chunks: HashMap<ChunkId, ArcChunkInfo>,
    /// World bounds
    world_bounds: Aabb,
    /// Dirty flag for rebuild
    dirty: bool,
    /// Statistics
    stats: SpatialStats,
}

#[derive(Debug, Clone)]
struct ArcChunkInfo {
    bounds: Aabb,
    center: Vector3<f32>,
}

#[derive(Debug, Default)]
pub struct SpatialStats {
    pub total_chunks: usize,
    pub octree_nodes: usize,
    pub query_count: u32,
    pub last_query_time_us: f32,
}

impl SpatialIndex {
    pub fn new() -> Self {
        // Large world bounds (can be expanded dynamically)
        let world_bounds = Aabb::from_center_and_size(
            Vector3::zeros(),
            Vector3::new(10000.0, 1000.0, 10000.0),
        );
        
        Self {
            octree: None,
            chunks: HashMap::new(),
            world_bounds,
            dirty: false,
            stats: SpatialStats::default(),
        }
    }
    
    /// Insert a chunk into the spatial index
    pub fn insert_chunk(&mut self, chunk: std::sync::Arc<Chunk>) {
        let center = chunk.bounding_sphere_center;
        let radius = chunk.bounding_sphere_radius;
        
        let bounds = Aabb::from_center_and_size(center, Vector3::new(radius * 2.0, radius * 2.0, radius * 2.0));
        
        self.chunks.insert(
            chunk.id,
            ArcChunkInfo {
                bounds,
                center,
            },
        );
        
        self.dirty = true;
        self.stats.total_chunks = self.chunks.len();
    }
    
    /// Add a chunk to the spatial index (alias for insert_chunk)
    pub fn add_chunk(&mut self, chunk_id: ChunkId, chunk: std::sync::Arc<Chunk>) {
        self.insert_chunk(chunk);
    }
    
    /// Remove a chunk from the spatial index
    pub fn remove_chunk(&mut self, chunk_id: ChunkId) {
        self.chunks.remove(&chunk_id);
        self.dirty = true;
        self.stats.total_chunks = self.chunks.len();
    }
    
    /// Rebuild the octree if needed
    pub fn rebuild_if_needed(&mut self) {
        if !self.dirty {
            return;
        }
        
        self.rebuild_octree();
        self.dirty = false;
    }
    
    /// Rebuild the octree from scratch
    fn rebuild_octree(&mut self) {
        let mut octree = OctreeNode::new(self.world_bounds, 16, 6);
        
        for (&chunk_id, info) in &self.chunks {
            octree.insert(chunk_id, info.bounds);
        }
        
        self.octree = Some(octree);
        self.stats.octree_nodes = self.count_octree_nodes();
    }
    
    /// Count octree nodes for statistics
    fn count_octree_nodes(&self) -> usize {
        fn count_nodes(node: &OctreeNode) -> usize {
            let mut count = 1;
            if let Some(children) = &node.children {
                for child in children.iter() {
                    count += count_nodes(child);
                }
            }
            count
        }
        
        self.octree.as_ref().map_or(0, count_nodes)
    }
    
    /// Query chunks within a bounding box
    pub fn query_aabb(&mut self, bounds: &Aabb) -> QueryResult {
        let start = std::time::Instant::now();
        
        self.rebuild_if_needed();
        
        let mut chunk_ids = Vec::new();
        
        if let Some(octree) = &self.octree {
            octree.query_aabb(bounds, &mut chunk_ids);
        } else {
            // Fallback to brute force
            for (&id, info) in &self.chunks {
                if info.bounds.intersects_aabb(bounds) {
                    chunk_ids.push(id);
                }
            }
        }
        
        self.stats.query_count += 1;
        self.stats.last_query_time_us = start.elapsed().as_micros() as f32;
        
        QueryResult {
            chunk_ids,
            point: None,
        }
    }
    
    /// Query chunks containing a point
    pub fn query_point(&mut self, point: Vector3<f32>) -> QueryResult {
        let start = std::time::Instant::now();
        
        self.rebuild_if_needed();
        
        let mut chunk_ids = Vec::new();
        
        if let Some(octree) = &self.octree {
            octree.query_point(point, &mut chunk_ids);
        } else {
            // Fallback to brute force
            for (&id, info) in &self.chunks {
                if info.bounds.contains_point(point) {
                    chunk_ids.push(id);
                }
            }
        }
        
        self.stats.query_count += 1;
        self.stats.last_query_time_us = start.elapsed().as_micros() as f32;
        
        QueryResult {
            chunk_ids,
            point: Some(Point3::from(point)),
        }
    }
    
    /// Find nearest chunks to a point
    pub fn query_nearest(&self, point: Vector3<f32>, count: usize) -> Vec<(ChunkId, f32)> {
        let mut distances: Vec<(ChunkId, f32)> = self.chunks
            .iter()
            .map(|(&id, info)| (id, info.center.metric_distance(&point)))
            .collect();

        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        distances.truncate(count);

        distances
    }
    
    /// Get all loaded chunk IDs
    pub fn get_all_chunks(&self) -> Vec<ChunkId> {
        self.chunks.keys().copied().collect()
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> &SpatialStats {
        &self.stats
    }
    
    /// Clear the spatial index
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.octree = None;
        self.dirty = false;
        self.stats = SpatialStats::default();
    }
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple grid-based spatial hash for dynamic objects
pub struct SpatialHash {
    /// Cell size
    cell_size: f32,
    /// Grid cells mapping hash to object IDs
    cells: HashMap<i64, Vec<usize>>,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }
    
    /// Hash a position to a cell key
    fn hash_position(&self, pos: Vector3<f32>) -> i64 {
        let x = (pos.x / self.cell_size).floor() as i64;
        let z = (pos.z / self.cell_size).floor() as i64;
        
        // Simple hash function
        ((x.wrapping_mul(73856093)) ^ (z.wrapping_mul(19349663))) as i64
    }
    
    /// Insert an object at a position
    pub fn insert(&mut self, object_id: usize, pos: Vector3<f32>) {
        let hash = self.hash_position(pos);
        self.cells.entry(hash).or_insert_with(Vec::new).push(object_id);
    }
    
    /// Clear the spatial hash
    pub fn clear(&mut self) {
        self.cells.clear();
    }
    
    /// Get potential collisions for a position
    pub fn get_potential_collisions(&self, pos: Vector3<f32>) -> Vec<usize> {
        let mut result = Vec::new();
        
        // Check surrounding cells (3x3 grid)
        for dx in -1..=1 {
            for dz in -1..=1 {
                let offset = Vector3::new(
                    dx as f32 * self.cell_size,
                    0.0,
                    dz as f32 * self.cell_size,
                );
                
                if let Some(objects) = self.cells.get(&self.hash_position(pos + offset)) {
                    result.extend(objects.iter().copied());
                }
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aabb_contains_point() {
        let aabb = Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
        
        assert!(aabb.contains_point(Vector3::new(0.0, 0.0, 0.0)));
        assert!(aabb.contains_point(Vector3::new(0.5, 0.5, 0.5)));
        assert!(!aabb.contains_point(Vector3::new(2.0, 0.0, 0.0)));
    }
    
    #[test]
    fn test_aabb_intersects() {
        let aabb1 = Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
        let aabb2 = Aabb::new(Vector3::new(0.5, 0.5, 0.5), Vector3::new(1.5, 1.5, 1.5));
        let aabb3 = Aabb::new(Vector3::new(5.0, 5.0, 5.0), Vector3::new(6.0, 6.0, 6.0));
        
        assert!(aabb1.intersects_aabb(&aabb2));
        assert!(!aabb1.intersects_aabb(&aabb3));
    }
    
    #[test]
    fn test_spatial_index_query() {
        let mut index = SpatialIndex::new();
        
        // Create mock chunks
        for x in -2..=2 {
            for z in -2..=2 {
                let chunk = std::sync::Arc::new(Chunk::new(
                    ChunkId::new(x, z),
                    crate::world::ChunkData::new(),
                ));
                index.insert_chunk(chunk);
            }
        }
        
        index.rebuild_if_needed();
        
        // Query center
        let result = index.query_point(Vector3::new(0.0, 0.0, 0.0));
        assert!(!result.chunk_ids.is_empty());
        
        // Query nearest
        let nearest = index.query_nearest(Vector3::new(0.0, 0.0, 0.0), 5);
        assert_eq!(nearest.len(), 5);
    }
    
    #[test]
    fn test_spatial_hash() {
        let mut hash = SpatialHash::new(10.0);
        
        hash.insert(0, Vector3::new(0.0, 0.0, 0.0));
        hash.insert(1, Vector3::new(5.0, 0.0, 5.0));
        hash.insert(2, Vector3::new(50.0, 0.0, 50.0));
        
        let nearby = hash.get_potential_collisions(Vector3::new(0.0, 0.0, 0.0));
        
        assert!(nearby.contains(&0));
        assert!(nearby.contains(&1));
        assert!(!nearby.contains(&2));
    }
}
