//! Terrain Mesh Builder - утилиты для создания мешей террейна из ChunkData
//! 
//! Генерирует вершинные и индексные буферы для рендеринга ландшафта

use crate::world::chunk::{ChunkData, TerrainVertex, CHUNK_SIZE, HEIGHTMAP_RESOLUTION};
use crate::graphics::mesh::Mesh;
use crate::graphics::rhi::IDevice;

/// Строитель меша террейна
pub struct TerrainMeshBuilder;

impl TerrainMeshBuilder {
    /// Создаёт меш террейна из ChunkData с указанным LOD
    pub fn build_mesh(device: &dyn IDevice, chunk_data: &ChunkData, lod_level: u32) -> Result<Mesh, crate::graphics::rhi::RhiError> {
        let (vertices, indices) = Self::generate_mesh_data(chunk_data, lod_level);
        
        if vertices.is_empty() || indices.is_empty() {
            return Err(crate::graphics::rhi::RhiError::InvalidParameter(
                "Generated terrain mesh is empty".to_string()
            ));
        }
        
        Mesh::new_terrain(device, &vertices, &indices)
    }
    
    /// Генерирует данные меша (вершины и индексы) для чанка
    pub fn generate_mesh_data(chunk_data: &ChunkData, lod_level: u32) -> (Vec<TerrainVertex>, Vec<u32>) {
        let stride = 1 << lod_level; // 1, 2, 4, 8...
        let stride_us = stride as usize;
        
        // Количество вершин по каждой оси с учётом LOD
        let resolution = ((HEIGHTMAP_RESOLUTION - 1) / stride + 1) as usize;
        
        let mut vertices = Vec::with_capacity(resolution * resolution);
        let mut indices = Vec::new();
        
        // Генерация вершин
        for z in 0..resolution {
            for x in 0..resolution {
                let gx = (x * stride_us) as f32;
                let gz = (z * stride_us) as f32;
                
                // Получаем высоту из chunk data
                let height = chunk_data.get_height(gx, gz);
                
                // Получаем нормаль
                let normal = chunk_data.get_normal(gx, gz);
                
                // UV координаты (0..1 для всего чанка)
                let tex_u = gx / CHUNK_SIZE as f32;
                let tex_v = gz / CHUNK_SIZE as f32;
                
                // Splat weights из splatmap
                let splat_idx = z * stride_us * HEIGHTMAP_RESOLUTION as usize + x * stride_us;
                let splat = if splat_idx < chunk_data.splatmap.len() {
                    chunk_data.splatmap[splat_idx]
                } else {
                    [1.0, 0.0, 0.0, 0.0] // Default to dirt
                };
                
                // Вычисляем тангенс и битангенс
                let (tangent, bitangent) = Self::compute_tangent_bitangent(
                    chunk_data, x * stride_us, z * stride_us, stride_us
                );
                
                vertices.push(TerrainVertex {
                    position: [gx, height, gz],
                    normal: [normal.x, normal.y, normal.z],
                    tangent,
                    bitangent,
                    texcoord: [tex_u, tex_v],
                    splat_weights: splat,
                });
            }
        }
        
        // Генерация индексов (два треугольника на квад)
        for z in 0..(resolution - 1) {
            for x in 0..(resolution - 1) {
                let i0 = (z * resolution + x) as u32;
                let i1 = (z * resolution + x + 1) as u32;
                let i2 = ((z + 1) * resolution + x) as u32;
                let i3 = ((z + 1) * resolution + x + 1) as u32;
                
                // Два треугольника на квад
                indices.extend_from_slice(&[i0, i2, i1]);
                indices.extend_from_slice(&[i1, i2, i3]);
            }
        }
        
        (vertices, indices)
    }
    
    /// Вычисляет тангенс и битангенс для вершины
    fn compute_tangent_bitangent(
        chunk_data: &ChunkData,
        x: usize,
        z: usize,
        stride: usize
    ) -> ([f32; 3], [f32; 3]) {
        // Простая оценка через соседние вершины
        let h_center = chunk_data.get_height(x as f32, z as f32);
        
        // Соседи по X и Z
        let h_right = if x + stride < HEIGHTMAP_RESOLUTION as usize {
            chunk_data.get_height((x + stride) as f32, z as f32)
        } else {
            h_center
        };
        
        let h_forward = if z + stride < HEIGHTMAP_RESOLUTION as usize {
            chunk_data.get_height(x as f32, (z + stride) as f32)
        } else {
            h_center
        };
        
        // Векторы касательных плоскостей
        let dx = stride as f32;
        let dz = stride as f32;
        
        let tangent_x = dx;
        let tangent_y = h_right - h_center;
        let tangent_z = 0.0;
        
        let bitangent_x = 0.0;
        let bitangent_y = h_forward - h_center;
        let bitangent_z = dz;
        
        // Нормализуем
        let tangent_len = (tangent_x * tangent_x + tangent_y * tangent_y + tangent_z * tangent_z).sqrt();
        let bitangent_len = (bitangent_x * bitangent_x + bitangent_y * bitangent_y + bitangent_z * bitangent_z).sqrt();
        
        let tangent = if tangent_len > 0.0001 {
            [tangent_x / tangent_len, tangent_y / tangent_len, tangent_z / tangent_len]
        } else {
            [1.0, 0.0, 0.0]
        };
        
        let bitangent = if bitangent_len > 0.0001 {
            [bitangent_x / bitangent_len, bitangent_y / bitangent_len, bitangent_z / bitangent_len]
        } else {
            [0.0, 0.0, 1.0]
        };
        
        (tangent, bitangent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_mesh_data_basic() {
        let chunk_data = ChunkData::new();
        let (vertices, indices) = TerrainMeshBuilder::generate_mesh_data(&chunk_data, 0);
        
        // При LOD 0 должно быть HEIGHTMAP_RESOLUTION x HEIGHTMAP_RESOLUTION вершин
        assert_eq!(vertices.len(), HEIGHTMAP_RESOLUTION as usize * HEIGHTMAP_RESOLUTION as usize);
        
        // Индексов должно быть достаточно для покрытия всех квадов
        assert!(!indices.is_empty());
        assert!(indices.len() % 3 == 0);
    }
    
    #[test]
    fn test_lod_reduces_vertices() {
        let chunk_data = ChunkData::new();
        
        let (v0, _) = TerrainMeshBuilder::generate_mesh_data(&chunk_data, 0);
        let (v1, _) = TerrainMeshBuilder::generate_mesh_data(&chunk_data, 1);
        let (v2, _) = TerrainMeshBuilder::generate_mesh_data(&chunk_data, 2);
        
        // С увеличением LOD количество вершин должно уменьшаться
        assert!(v0.len() > v1.len());
        assert!(v1.len() > v2.len());
    }
}
