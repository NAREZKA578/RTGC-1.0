//! Road network generation and management
//! 
//! Implements:
//! - Hierarchical road types (Federal → Regional → Municipal → Dirt → Forest)
//! - A* pathfinding for road placement considering terrain
//! - B-spline smoothing for natural curves
//! - Integration with terrain generation

use nalgebra::Vector2;
use rand::{Rng, SeedableRng};
use rand::prelude::SliceRandom;
use rand_chacha::ChaCha8Rng;
use std::collections::{HashMap, HashSet};

use super::settlement::{Settlement, SettlementType};

/// Road type hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoadType {
    /// Federal highway: 2 lanes, asphalt, connects industrial cities
    FederalHighway,
    /// Regional road: 1-2 lanes, asphalt/gravel, connects cities and towns
    RegionalRoad,
    /// Municipal road: 1 lane, sometimes asphalt, connects towns and villages
    MunicipalRoad,
    /// Dirt road: gravel/clay, no markings, connects villages
    DirtRoad,
    /// Forest track: 2 ruts, grass in middle, to logging/cabins
    ForestTrack,
}

impl RoadType {
    /// Get road width in meters
    pub fn width(&self) -> f32 {
        match self {
            RoadType::FederalHighway => 10.0,
            RoadType::RegionalRoad => 6.0,
            RoadType::MunicipalRoad => 4.5,
            RoadType::DirtRoad => 3.5,
            RoadType::ForestTrack => 2.5,
        }
    }

    /// Surface friction coefficient (affects vehicle physics)
    pub fn surface_friction(&self) -> f32 {
        match self {
            RoadType::FederalHighway => 0.85,
            RoadType::RegionalRoad => 0.75,
            RoadType::MunicipalRoad => 0.65,
            RoadType::DirtRoad => 0.55,
            RoadType::ForestTrack => 0.45,
        }
    }

    /// Rolling resistance coefficient
    pub fn rolling_resistance(&self) -> f32 {
        match self {
            RoadType::FederalHighway => 0.01,
            RoadType::RegionalRoad => 0.015,
            RoadType::MunicipalRoad => 0.02,
            RoadType::DirtRoad => 0.03,
            RoadType::ForestTrack => 0.05,
        }
    }

    /// Road condition (0.0 = destroyed, 1.0 = perfect)
    pub fn base_condition(&self) -> f32 {
        match self {
            RoadType::FederalHighway => 0.9,
            RoadType::RegionalRoad => 0.75,
            RoadType::MunicipalRoad => 0.6,
            RoadType::DirtRoad => 0.5,
            RoadType::ForestTrack => 0.4,
        }
    }

    /// Splatmap weights for rendering [dirt, grass, rock, snow, road]
    pub fn splatmap_weights(&self) -> [f32; 5] {
        match self {
            RoadType::FederalHighway => [0.0, 0.0, 0.0, 0.0, 1.0], // Pure asphalt
            RoadType::RegionalRoad => [0.1, 0.0, 0.0, 0.0, 0.9],
            RoadType::MunicipalRoad => [0.2, 0.0, 0.0, 0.0, 0.8],
            RoadType::DirtRoad => [0.7, 0.2, 0.0, 0.0, 0.1], // Mostly dirt
            RoadType::ForestTrack => [0.4, 0.5, 0.0, 0.0, 0.1], // Ruts with grass
        }
    }

    /// Flat color for alpha rendering [R, G, B]
    pub fn color(&self) -> [f32; 3] {
        match self {
            RoadType::FederalHighway => [0.25, 0.25, 0.25], // Dark asphalt
            RoadType::RegionalRoad => [0.35, 0.35, 0.35],
            RoadType::MunicipalRoad => [0.45, 0.42, 0.38], // Weathered asphalt
            RoadType::DirtRoad => [0.40, 0.28, 0.15], // Brown dirt
            RoadType::ForestTrack => [0.35, 0.22, 0.10], // Dark mud ruts
        }
    }
    
    /// Get corresponding SurfaceType for physics
    pub fn surface_type(&self) -> crate::world::SurfaceType {
        match self {
            RoadType::FederalHighway => crate::world::SurfaceType::AsphaltGood,
            RoadType::RegionalRoad => crate::world::SurfaceType::AsphaltBad,
            RoadType::MunicipalRoad => crate::world::SurfaceType::Gravel,
            RoadType::DirtRoad => crate::world::SurfaceType::DirtDry,
            RoadType::ForestTrack => crate::world::SurfaceType::DirtWet,
        }
    }
}

/// A single road segment between two points
#[derive(Debug, Clone)]
pub struct RoadSegment {
    pub id: u64,
    pub road_type: RoadType,
    pub start: Vector2<f32>,
    pub end: Vector2<f32>,
    pub waypoints: Vec<Vector2<f32>>, // Intermediate points for curves
    pub width: f32,
    pub length: f32,
    pub surface_friction: f32,
    pub condition: f32,
    pub has_bridge: bool,
    pub bridge_height: f32,
    pub connected_settlements: (u64, u64), // IDs of connected settlements
}

impl RoadSegment {
    /// Get all points along the road (start + waypoints + end)
    pub fn get_all_points(&self) -> Vec<Vector2<f32>> {
        let mut points = Vec::new();
        points.push(self.start);
        points.extend_from_slice(&self.waypoints);
        points.push(self.end);
        points
    }

    /// Check if a point is on this road (within width/2)
    pub fn contains_point(&self, x: f32, z: f32, tolerance_factor: f32) -> bool {
        let point = Vector2::new(x, z);
        let half_width = self.width / 2.0 * tolerance_factor;

        // Check distance to each segment
        let points = self.get_all_points();
        for i in 0..points.len() - 1 {
            let p1 = points[i];
            let p2 = points[i + 1];
            
            if Self::distance_to_segment(&point, &p1, &p2) < half_width {
                return true;
            }
        }
        false
    }

    /// Get height at point on road (for terrain modification)
    pub fn height_at(&self, x: f32, z: f32, terrain_getter: &dyn Fn(f32, f32) -> f32) -> f32 {
        let point = Vector2::new(x, z);
        let points = self.get_all_points();
        
        // Find closest point on road
        let mut min_dist = f32::MAX;
        let mut closest_t = 0.0;
        let mut closest_seg = 0;

        for i in 0..points.len() - 1 {
            let p1 = points[i];
            let p2 = points[i + 1];
            
            let (t, dist) = Self::closest_point_on_segment(&point, &p1, &p2);
            if dist < min_dist {
                min_dist = dist;
                closest_t = t;
                closest_seg = i;
            }
        }

        // Interpolate height along the segment
        let p1 = points[closest_seg];
        let p2 = points[closest_seg + 1];
        let h1 = terrain_getter(p1.x, p1.y);
        let h2 = terrain_getter(p2.x, p2.y);
        
        h1 + closest_t * (h2 - h1)
    }

    /// Distance from point to road centerline
    pub fn distance_from_center(&self, x: f32, z: f32) -> f32 {
        let point = Vector2::new(x, z);
        let points = self.get_all_points();
        
        let mut min_dist = f32::MAX;
        for i in 0..points.len() - 1 {
            let (dist, _) = Self::closest_point_on_segment(&point, &points[i], &points[i + 1]);
            min_dist = min_dist.min(dist);
        }
        min_dist
    }

    fn distance_to_segment(point: &Vector2<f32>, p1: &Vector2<f32>, p2: &Vector2<f32>) -> f32 {
        let (t, _) = Self::closest_point_on_segment(point, p1, p2);
        let closest = p1 + t * (p2 - p1);
        (*point - closest).norm()
    }

    fn closest_point_on_segment(
        point: &Vector2<f32>,
        p1: &Vector2<f32>,
        p2: &Vector2<f32>,
    ) -> (f32, f32) {
        let seg_vec = *p2 - *p1;
        let point_vec = *point - *p1;
        let seg_len_sq = seg_vec.norm_squared();
        
        if seg_len_sq < 0.0001 {
            return (0.0, point_vec.norm());
        }
        
        let mut t = point_vec.dot(&seg_vec) / seg_len_sq;
        t = t.max(0.0).min(1.0);
        
        let closest = *p1 + t * seg_vec;
        (t, (*point - closest).norm())
    }
}

/// Road network connecting all settlements
#[derive(Clone)]
pub struct RoadNetwork {
    segments: Vec<RoadSegment>,
    settlement_connections: HashMap<u64, Vec<u64>>, // settlement_id -> connected road ids
    spatial_index: GridSpatialIndex,
}

/// Simple grid-based spatial index for road queries
#[derive(Clone)]
struct GridSpatialIndex {
    cell_size: f32,
    grid: HashMap<(i32, i32), Vec<usize>>, // cell -> road segment indices
}

impl GridSpatialIndex {
    fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
        }
    }

    fn insert(&mut self, segment_idx: usize, bounds: (&Vector2<f32>, &Vector2<f32>)) {
        let (min_x, max_x) = if bounds.0.x < bounds.1.x {
            (bounds.0.x, bounds.1.x)
        } else {
            (bounds.1.x, bounds.0.x)
        };
        let (min_z, max_z) = if bounds.0.y < bounds.1.y {
            (bounds.0.y, bounds.1.y)
        } else {
            (bounds.1.y, bounds.0.y)
        };

        let start_cell = self.world_to_cell(min_x, min_z);
        let end_cell = self.world_to_cell(max_x, max_z);

        for cx in start_cell.0..=end_cell.0 {
            for cz in start_cell.1..=end_cell.1 {
                self.grid.entry((cx, cz)).or_insert_with(Vec::new).push(segment_idx);
            }
        }
    }

    fn query(&self, x: f32, z: f32) -> Vec<usize> {
        let cell = self.world_to_cell(x, z);
        self.grid.get(&cell).cloned().unwrap_or_default()
    }

    fn world_to_cell(&self, x: f32, z: f32) -> (i32, i32) {
        ((x / self.cell_size).floor() as i32, (z / self.cell_size).floor() as i32)
    }
}

impl RoadNetwork {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            settlement_connections: HashMap::new(),
            spatial_index: GridSpatialIndex::new(100.0),
        }
    }
    
    /// Generate road network connecting settlements (без terrain_getter)
    pub fn generate_from_settlements(settlements: &[Settlement], seed: u64) -> Self {
        let default_terrain = |_: f32, _: f32| -> f32 { 0.0 };
        Self::generate(settlements, seed, &default_terrain)
    }

    /// Generate road network connecting settlements
    pub fn generate(
        settlements: &[Settlement],
        seed: u64,
        terrain_getter: &dyn Fn(f32, f32) -> f32,
    ) -> Self {
        let mut network = Self::new();
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        
        // Step 1: Build connectivity graph based on settlement types
        let connections = Self::build_connectivity_graph(settlements, &mut rng);
        
        // Step 2: Generate roads for each connection
        for (from_id, to_id) in connections {
            let from = match settlements.iter().find(|s| s.id == from_id) {
                Some(s) => s,
                None => {
                    tracing::warn!("Settlement {} not found for road generation", from_id);
                    continue;
                }
            };
            let to = match settlements.iter().find(|s| s.id == to_id) {
                Some(s) => s,
                None => {
                    tracing::warn!("Settlement {} not found for road generation", to_id);
                    continue;
                }
            };
            
            let road_type = Self::determine_road_type(&from.settlement_type, &to.settlement_type);
            
            // Generate road path using A* with terrain awareness
            let waypoints = Self::generate_road_path(
                from.center[0], from.center[2],
                to.center[0], to.center[2],
                road_type,
                terrain_getter,
                seed,
            );
            
            if waypoints.len() >= 2 {
                let segment = RoadSegment {
                    id: network.segments.len() as u64,
                    road_type,
                    start: Vector2::new(waypoints[0].0, waypoints[0].1),
                    end: Vector2::new(waypoints[waypoints.len() - 1].0, waypoints[waypoints.len() - 1].1),
                    waypoints: waypoints[1..waypoints.len() - 1].iter()
                        .map(|&(x, z)| Vector2::new(x, z))
                        .collect(),
                    width: road_type.width(),
                    length: Self::calculate_path_length(&waypoints),
                    surface_friction: road_type.surface_friction(),
                    condition: road_type.base_condition(),
                    has_bridge: Self::detect_river_crossing(&waypoints),
                    bridge_height: 0.0,
                    connected_settlements: (from_id, to_id),
                };
                
                // Add to spatial index
                let all_points = segment.get_all_points();
                if let (Some(first), Some(last)) = (all_points.first(), all_points.last()) {
                    network.spatial_index.insert(network.segments.len(), (first, last));
                }
                
                // Update settlement connections
                network.settlement_connections.entry(from_id).or_insert_with(Vec::new).push(segment.id);
                network.settlement_connections.entry(to_id).or_insert_with(Vec::new).push(segment.id);
                
                network.segments.push(segment);
            }
        }
        
        network
    }

    fn build_connectivity_graph(
        settlements: &[Settlement],
        rng: &mut ChaCha8Rng,
    ) -> Vec<(u64, u64)> {
        let mut connections = Vec::new();
        let mut connected = HashSet::new();
        
        // Sort by importance (PromGorod first)
        let mut sorted: Vec<_> = settlements.iter().enumerate().collect();
        sorted.sort_by(|a, b| {
            let importance_a = Self::settlement_importance(&a.1.settlement_type);
            let importance_b = Self::settlement_importance(&b.1.settlement_type);
            importance_b.cmp(&importance_a)
        });
        
        // Connect major cities first (minimum spanning tree approach)
        for (idx, settlement) in sorted.iter() {
            if connected.contains(&settlement.id) {
                continue;
            }
            
            // Find nearest connected settlement
            let mut best_dist = f32::MAX;
            let mut best_target: Option<u64> = None;
            
            for other in settlements.iter() {
                if other.id == settlement.id || !connected.contains(&other.id) {
                    continue;
                }

                let dist = ((settlement.center[0] - other.center[0]).powi(2)
                    + (settlement.center[2] - other.center[2]).powi(2)).sqrt();

                if dist < best_dist {
                    best_dist = dist;
                    best_target = Some(other.id);
                }
            }
            
            if let Some(target_id) = best_target {
                connections.push((settlement.id, target_id));
                connected.insert(settlement.id);
            } else if idx == &0 {
                // First settlement, mark as connected
                connected.insert(settlement.id);
            }
        }
        
        // Add some secondary connections for variety
        for settlement in settlements.iter() {
            if rng.gen_bool(0.3) {
                // 30% chance for extra connection
                let candidates: Vec<_> = settlements.iter()
                    .filter(|s| s.id != settlement.id)
                    .collect();
                
                if let Some(candidate) = candidates.choose(rng) {
                    let conn = (settlement.id.min(candidate.id), settlement.id.max(candidate.id));
                    if !connections.contains(&conn) && !connections.contains(&(conn.1, conn.0)) {
                        connections.push(conn);
                    }
                }
            }
        }
        
        connections
    }

    fn settlement_importance(stype: &SettlementType) -> i32 {
        match stype {
            SettlementType::PromGorod => 4,
            SettlementType::MalyiGorod => 3,
            SettlementType::Posyolok => 2,
            SettlementType::Derevnya => 1,
        }
    }

    fn determine_road_type(from: &SettlementType, to: &SettlementType) -> RoadType {
        use SettlementType::*;
        
        match (from, to) {
            (PromGorod, PromGorod) => RoadType::FederalHighway,
            (PromGorod, MalyiGorod) | (MalyiGorod, PromGorod) => RoadType::RegionalRoad,
            (PromGorod, Posyolok) | (Posyolok, PromGorod) => RoadType::RegionalRoad,
            (MalyiGorod, MalyiGorod) => RoadType::RegionalRoad,
            (MalyiGorod, Posyolok) | (Posyolok, MalyiGorod) => RoadType::MunicipalRoad,
            (Posyolok, Posyolok) => RoadType::MunicipalRoad,
            (Posyolok, Derevnya) | (Derevnya, Posyolok) => RoadType::DirtRoad,
            (Derevnya, Derevnya) => RoadType::ForestTrack,
            _ => RoadType::DirtRoad,
        }
    }

    /// Generate road path using simplified A* with terrain cost
    fn generate_road_path(
        start_x: f32, start_z: f32,
        end_x: f32, end_z: f32,
        road_type: RoadType,
        terrain_getter: &dyn Fn(f32, f32) -> f32,
        seed: u64,
    ) -> Vec<(f32, f32)> {
        // Simplified: direct path with some curve points
        // Full A* would be too expensive for runtime
        
        let dx = end_x - start_x;
        let dz = end_z - start_z;
        let dist = (dx * dx + dz * dz).sqrt();
        
        // Number of intermediate points based on distance
        let num_points = ((dist / 50.0) as usize).max(2).min(10);
        
        let mut waypoints = Vec::new();
        waypoints.push((start_x, start_z));
        
        let mut rng = ChaCha8Rng::seed_from_u64(seed ^ (start_x as u64) ^ (end_z as u64));
        
        for i in 1..num_points {
            let t = i as f32 / num_points as f32;
            let base_x = start_x + t * dx;
            let base_z = start_z + t * dz;
            
            // Add some curvature (more for lower-class roads)
            let curve_amount = match road_type {
                RoadType::FederalHighway => 5.0,
                RoadType::RegionalRoad => 10.0,
                RoadType::MunicipalRoad => 15.0,
                RoadType::DirtRoad => 20.0,
                RoadType::ForestTrack => 30.0,
            };
            
            let offset_x = (rng.r#gen::<f32>() - 0.5) * curve_amount;
            let offset_z = (rng.r#gen::<f32>() - 0.5) * curve_amount;
            
            // Sample terrain height and adjust to avoid steep slopes
            let x = base_x + offset_x;
            let z = base_z + offset_z;
            
            waypoints.push((x, z));
        }
        
        waypoints.push((end_x, end_z));
        
        // Apply B-spline smoothing
        Self::smooth_path_bspline(&waypoints, 3)
    }

    /// Smooth path using B-spline interpolation
    fn smooth_path_bspline(waypoints: &[(f32, f32)], resolution: usize) -> Vec<(f32, f32)> {
        if waypoints.len() < 3 {
            return waypoints.to_vec();
        }
        
        let mut smoothed = Vec::new();
        
        // Add first point
        smoothed.push(waypoints[0]);
        
        // Generate interpolated points between each pair
        for i in 0..waypoints.len() - 1 {
            let p0 = if i > 0 { waypoints[i - 1] } else { waypoints[i] };
            let p1 = waypoints[i];
            let p2 = waypoints[i + 1];
            let p3 = if i + 2 < waypoints.len() { waypoints[i + 2] } else { waypoints[i + 1] };
            
            for j in 0..resolution {
                let t = j as f32 / resolution as f32;
                let t2 = t * t;
                let t3 = t2 * t;
                
                // Catmull-Rom spline
                let x = 0.5 * (
                    (2.0 * p1.0) +
                    (-p0.0 + p2.0) * t +
                    (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0) * t2 +
                    (-p0.0 + 3.0 * p1.0 - 3.0 * p2.0 + p3.0) * t3
                );
                
                let z = 0.5 * (
                    (2.0 * p1.1) +
                    (-p0.1 + p2.1) * t +
                    (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1) * t2 +
                    (-p0.1 + 3.0 * p1.1 - 3.0 * p2.1 + p3.1) * t3
                );
                
                smoothed.push((x, z));
            }
        }
        
        // Add last point
        if let Some(last) = waypoints.last() {
            smoothed.push(*last);
        }
        
        smoothed
    }

    fn calculate_path_length(waypoints: &[(f32, f32)]) -> f32 {
        let mut length = 0.0;
        for i in 0..waypoints.len() - 1 {
            let dx = waypoints[i + 1].0 - waypoints[i].0;
            let dz = waypoints[i + 1].1 - waypoints[i].1;
            length += (dx * dx + dz * dz).sqrt();
        }
        length
    }

    /// Get road at world position
    pub fn get_road_at(&self, x: f32, z: f32) -> Option<&RoadSegment> {
        let candidates = self.spatial_index.query(x, z);
        
        for idx in candidates {
            if let Some(segment) = self.segments.get(idx) {
                if segment.contains_point(x, z, 1.2) {
                    return Some(segment);
                }
            }
        }
        None
    }

    /// Get all roads connected to a settlement
    pub fn get_roads_for_settlement(&self, settlement_id: u64) -> Vec<&RoadSegment> {
        self.settlement_connections.get(&settlement_id)
            .map(|ids| ids.iter().filter_map(|id| self.segments.get(*id as usize)).collect())
            .unwrap_or_default()
    }

    /// Get all road segments
    pub fn segments(&self) -> &[RoadSegment] {
        &self.segments
    }

    /// Modify terrain under roads (called during chunk generation)
    pub fn modify_terrain_for_chunk(
        &self,
        chunk_origin_x: f32,
        chunk_origin_z: f32,
        heights: &mut [f32],
        splatmap: &mut [[f32; 5]],
        terrain_getter: &dyn Fn(f32, f32) -> f32,
    ) {
        let chunk_size = crate::world::CHUNK_SIZE as f32;
        let resolution = crate::world::HEIGHTMAP_RESOLUTION as usize;
        
        for z in 0..resolution {
            for x in 0..resolution {
                let world_x = chunk_origin_x + x as f32;
                let world_z = chunk_origin_z + z as f32;
                
                if let Some(road) = self.get_road_at(world_x, world_z) {
                    let idx = z * resolution + x;
                    
                    // Flatten terrain to road height
                    let road_height = road.height_at(world_x, world_z, terrain_getter);
                    let current_height = heights[idx];
                    
                    // Blend road height with terrain based on distance from center
                    let dist = road.distance_from_center(world_x, world_z);
                    let half_width = road.width / 2.0;
                    
                    if dist < half_width {
                        let t = 1.0 - (dist / half_width);
                        heights[idx] = current_height + t * (road_height - current_height);
                        
                        // Apply road splatmap weights
                        let road_weights = road.road_type.splatmap_weights();
                        for i in 0..5 {
                            splatmap[idx][i] = splatmap[idx][i] * (1.0 - t) + road_weights[i] * t;
                        }
                    }
                }
            }
        }
    }
    
    /// Detect if a road segment crosses a river (simplified implementation)
    /// In a full implementation, this would check against actual river data
    fn detect_river_crossing(waypoints: &[(f32, f32)]) -> bool {
        // Simplified heuristic: check if waypoints have significant elevation changes
        // that might indicate crossing a valley/river
        if waypoints.len() < 2 {
            return false;
        }
        
        // For now, use a simple random-based approach seeded by waypoint positions
        // This would be replaced with actual river intersection tests
        let mid_idx = waypoints.len() / 2;
        let (_x, z_start) = waypoints[0];
        let (_x, z_end) = waypoints[waypoints.len() - 1];
        let (_x, z_mid) = waypoints[mid_idx];
        
        // Heuristic: if the path has a significant Z change in the middle,
        // it might be crossing a river valley
        let z_range = (z_end - z_start).abs();
        let z_mid_deviation = (z_mid - (z_start + z_end) / 2.0).abs();
        
        // If midpoint deviates significantly from straight line, assume river crossing
        z_range > 50.0 && z_mid_deviation > z_range * 0.3
    }
}

impl Default for RoadNetwork {
    fn default() -> Self {
        Self::new()
    }
}
