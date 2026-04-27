//! Chunk data structure for world streaming

use nalgebra::Vector3;

/// Size of a chunk in meters (must be power of 2 for efficient LOD)
pub const CHUNK_SIZE: u32 = 64;

/// Resolution of heightmap within a chunk
pub const HEIGHTMAP_RESOLUTION: u32 = 65; // CHUNK_SIZE + 1 for vertex sharing

/// Unique identifier for a chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkId {
    pub x: i32,
    pub z: i32,
}

impl ChunkId {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
    
    /// Get world position of chunk origin
    pub fn world_position(&self) -> Vector3<f32> {
        Vector3::new(
            self.x as f32 * CHUNK_SIZE as f32,
            0.0,
            self.z as f32 * CHUNK_SIZE as f32,
        )
    }
}

/// Raw chunk data before GPU upload
#[derive(Debug, Clone)]
pub struct ChunkData {
    /// Height values in row-major order (z * resolution + x)
    pub heights: Vec<f32>,
    /// Splatmap for terrain texturing (R,G,B,A for up to 4 materials)
    pub splatmap: Vec<[f32; 4]>,
    /// Grass/vegetation density map
    pub vegetation_density: Vec<f32>,
    /// Water level
    pub water_level: f32,
    /// List of prop positions and types
    pub props: Vec<PropInstance>,
}

impl ChunkData {
    pub fn new() -> Self {
        let size = (HEIGHTMAP_RESOLUTION * HEIGHTMAP_RESOLUTION) as usize;
        Self {
            heights: vec![0.0; size],
            splatmap: vec![[0.0; 4]; size],
            vegetation_density: vec![0.0; size],
            water_level: 0.0,
            props: Vec::new(),
        }
    }
    
    /// Get height at local coordinates
    pub fn get_height(&self, x: f32, z: f32) -> f32 {
        let x = x.clamp(0.0, CHUNK_SIZE as f32);
        let z = z.clamp(0.0, CHUNK_SIZE as f32);
        
        let fx = x * (HEIGHTMAP_RESOLUTION - 1) as f32 / CHUNK_SIZE as f32;
        let fz = z * (HEIGHTMAP_RESOLUTION - 1) as f32 / CHUNK_SIZE as f32;
        
        let x0 = fx.floor() as usize;
        let z0 = fz.floor() as usize;
        let x1 = (x0 + 1).min(HEIGHTMAP_RESOLUTION as usize - 1);
        let z1 = (z0 + 1).min(HEIGHTMAP_RESOLUTION as usize - 1);
        
        let tx = fx - x0 as f32;
        let tz = fz - z0 as f32;
        
        // Bilinear interpolation
        let h00 = self.heights[z0 * HEIGHTMAP_RESOLUTION as usize + x0];
        let h10 = self.heights[z0 * HEIGHTMAP_RESOLUTION as usize + x1];
        let h01 = self.heights[z1 * HEIGHTMAP_RESOLUTION as usize + x0];
        let h11 = self.heights[z1 * HEIGHTMAP_RESOLUTION as usize + x1];
        
        let h0 = h00 * (1.0 - tx) + h10 * tx;
        let h1 = h01 * (1.0 - tx) + h11 * tx;
        
        h0 * (1.0 - tz) + h1 * tz
    }
    
    /// Get normal at local coordinates
    pub fn get_normal(&self, x: f32, z: f32) -> Vector3<f32> {
        let sample_dist = 1.0;
        crate::utils::compute_terrain_normal(
            |sx, sz| self.get_height(sx, sz),
            x, z, sample_dist
        )
    }
}

impl Default for ChunkData {
    fn default() -> Self {
        Self::new()
    }
}

/// Instance of a prop (tree, rock, building, etc.)
#[derive(Debug, Clone)]
pub struct PropInstance {
    /// Position relative to chunk origin
    pub position: Vector3<f32>,
    /// Rotation around Y axis
    pub rotation: f32,
    /// Scale
    pub scale: f32,
    /// Prop type ID (references asset database)
    pub prop_type: u32,
    /// LOD distance thresholds
    pub lod_distances: [f32; 4],
}

/// Fully loaded chunk ready for rendering
#[derive(Clone, Debug)]
pub struct Chunk {
    /// Chunk identifier
    pub id: ChunkId,
    /// Chunk data
    pub data: ChunkData,
    /// Vertex buffer handle (GPU resource)
    pub vertex_buffer: Option<u64>, // Placeholder for GPU handle
    /// Index buffer handle
    pub index_buffer: Option<u64>,
    /// Texture handles
    pub textures: Vec<u64>,
    /// Current LOD level
    pub lod_level: u32,
    /// Is this chunk currently being rendered?
    pub is_visible: bool,
    /// Bounding sphere for frustum culling
    pub bounding_sphere_center: Vector3<f32>,
    pub bounding_sphere_radius: f32,
    /// Number of vertices
    pub vertex_count: u32,
    /// Number of indices
    pub index_count: u32,
}

impl Chunk {
    pub fn new(id: ChunkId, data: ChunkData) -> Self {
        let world_pos = id.world_position();
        let center = Vector3::new(
            world_pos.x + CHUNK_SIZE as f32 / 2.0,
            0.0, // Will be updated after height calculation
            world_pos.z + CHUNK_SIZE as f32 / 2.0,
        );
        
        // Calculate average height for center
        let avg_height = data.get_height(CHUNK_SIZE as f32 / 2.0, CHUNK_SIZE as f32 / 2.0);
        let center = Vector3::new(center.x, avg_height, center.z);
        
        // Bounding radius is half the diagonal of the chunk plus max height variation
        let bounding_radius = (CHUNK_SIZE as f32 * 2.0f32.sqrt()) / 2.0 + 50.0; // Add margin for height
        
        Self {
            id,
            data,
            vertex_buffer: None,
            index_buffer: None,
            textures: Vec::new(),
            lod_level: 0,
            is_visible: false,
            bounding_sphere_center: center,
            bounding_sphere_radius: bounding_radius,
            vertex_count: 0,
            index_count: 0,
        }
    }
    
    /// Get height at local coordinates
    pub fn get_height(&self, x: f32, z: f32) -> f32 {
        self.data.get_height(x, z)
    }
    
    /// Get normal at local coordinates
    pub fn get_normal(&self, x: f32, z: f32) -> Vector3<f32> {
        self.data.get_normal(x, z)
    }
    
    /// Check if point is inside chunk bounds
    pub fn contains_point(&self, world_pos: Vector3<f32>) -> bool {
        let chunk_min = self.id.world_position();
        let chunk_max = Vector3::new(
            chunk_min.x + CHUNK_SIZE as f32,
            f32::MAX,
            chunk_min.z + CHUNK_SIZE as f32,
        );
        
        world_pos.x >= chunk_min.x && world_pos.x < chunk_max.x &&
        world_pos.z >= chunk_min.z && world_pos.z < chunk_max.z
    }
    
    /// Distance to camera for LOD calculation
    pub fn distance_to(&self, camera_pos: Vector3<f32>) -> f32 {
        self.bounding_sphere_center.metric_distance(&camera_pos)
    }
    
    /// Frustum culling test
    pub fn is_in_frustum(&self, frustum_planes: &[[f32; 4]; 6]) -> bool {
        // Simple sphere-frustum intersection test
        for plane in frustum_planes.iter() {
            let distance = plane[0] * self.bounding_sphere_center.x
                         + plane[1] * self.bounding_sphere_center.y
                         + plane[2] * self.bounding_sphere_center.z
                         + plane[3];
            
            if distance < -self.bounding_sphere_radius {
                return false;
            }
        }
        true
    }
}

/// Vertex format for terrain rendering
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TerrainVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
    pub texcoord: [f32; 2],
    pub splat_weights: [f32; 4],
}

unsafe impl bytemuck::NoUninit for TerrainVertex {}

impl TerrainVertex {
    pub fn zeroed() -> Self {
        Self {
            position: [0.0; 3],
            normal: [0.0; 3],
            tangent: [1.0; 3],
            bitangent: [0.0; 3],
            texcoord: [0.0; 2],
            splat_weights: [1.0, 0.0, 0.0, 0.0],
        }
    }
}

/// Generate mesh data for a chunk at specified LOD level
pub fn generate_chunk_mesh(data: &ChunkData, lod_level: u32) -> (Vec<TerrainVertex>, Vec<u32>) {
    let stride = 1 << lod_level; // 1, 2, 4, 8...
    let stride_us = stride as usize;
    let resolution = ((HEIGHTMAP_RESOLUTION - 1) / stride + 1) as usize;

    let mut vertices = Vec::with_capacity(resolution * resolution);
    let mut indices = Vec::new();

    // Generate vertices
    for z in 0..resolution {
        for x in 0..resolution {
            let gx = (x * stride_us) as f32;
            let gz = (z * stride_us) as f32;

            let height = data.get_height(gx, gz);
            let normal = data.get_normal(gx, gz);

            let tex_u = gx / CHUNK_SIZE as f32;
            let tex_v = gz / CHUNK_SIZE as f32;

            let splat = data.splatmap[z * stride_us * HEIGHTMAP_RESOLUTION as usize + x * stride_us];
            
            vertices.push(TerrainVertex {
                position: [gx, height, gz],
                normal: [normal.x, normal.y, normal.z],
                tangent: [1.0, 0.0, 0.0], // Will be calculated properly in real implementation
                bitangent: [0.0, 0.0, 1.0],
                texcoord: [tex_u, tex_v],
                splat_weights: splat,
            });
        }
    }
    
    // Generate indices (triangle strip or triangle list)
    for z in 0..(resolution - 1) {
        for x in 0..(resolution - 1) {
            let i0 = (z * resolution + x) as u32;
            let i1 = (z * resolution + x + 1) as u32;
            let i2 = ((z + 1) * resolution + x) as u32;
            let i3 = ((z + 1) * resolution + x + 1) as u32;
            
            // Two triangles per quad
            indices.extend_from_slice(&[i0, i2, i1]);
            indices.extend_from_slice(&[i1, i2, i3]);
        }
    }
    
    (vertices, indices)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chunk_id_world_position() {
        let id = ChunkId::new(0, 0);
        assert_eq!(id.world_position(), Vector3::new(0.0, 0.0, 0.0));
        
        let id = ChunkId::new(1, 2);
        assert_eq!(id.world_position(), Vector3::new(64.0, 0.0, 128.0));
        
        let id = ChunkId::new(-1, -1);
        assert_eq!(id.world_position(), Vector3::new(-64.0, 0.0, -64.0));
    }
    
    #[test]
    fn test_chunk_data_height_interpolation() {
        let mut data = ChunkData::new();
        
        // Set corner heights
        data.heights[0] = 0.0; // (0, 0)
        data.heights[HEIGHTMAP_RESOLUTION as usize - 1] = 10.0; // (max, 0)
        data.heights[(HEIGHTMAP_RESOLUTION as usize - 1) * HEIGHTMAP_RESOLUTION as usize] = 20.0; // (0, max)
        data.heights[HEIGHTMAP_RESOLUTION as usize * HEIGHTMAP_RESOLUTION as usize - 1] = 30.0; // (max, max)
        
        // Test interpolation at corners
        let h = data.get_height(0.0, 0.0);
        assert!((h - 0.0).abs() < 0.1);
        
        let h = data.get_height(CHUNK_SIZE as f32, 0.0);
        assert!((h - 10.0).abs() < 0.1);
    }
    
    #[test]
    fn test_chunk_contains_point() {
        let id = ChunkId::new(0, 0);
        let data = ChunkData::new();
        let chunk = Chunk::new(id, data);
        
        assert!(chunk.contains_point(Vector3::new(32.0, 0.0, 32.0)));
        assert!(chunk.contains_point(Vector3::new(1.0, 0.0, 1.0)));
        assert!(!chunk.contains_point(Vector3::new(64.0, 0.0, 32.0))); // Edge is exclusive
        assert!(!chunk.contains_point(Vector3::new(-1.0, 0.0, 32.0)));
    }
}
