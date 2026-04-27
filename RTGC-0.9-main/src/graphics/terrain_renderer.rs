//! Terrain Rendering System - управление рендерингом чанков террейна
//! 
//! Интегрирует генерацию мешей, LOD, frustum culling и отправку команд рендеринга

use std::sync::Arc;
use std::collections::HashMap;

use nalgebra::Vector3;

use crate::graphics::rhi::{IDevice, ResourceHandle};
use crate::graphics::mesh::Mesh;
use crate::graphics::terrain_mesh_builder::TerrainMeshBuilder;
use crate::graphics::renderer::commands::RenderCommand;
use crate::world::chunk::{Chunk, ChunkData, ChunkId, CHUNK_SIZE};

/// Данные отрендеренного чанка
pub struct RenderedChunk {
    pub chunk_id: ChunkId,
    pub mesh: Mesh,
    pub material: ResourceHandle,
    pub lod_level: u32,
    pub is_visible: bool,
}

/// Система рендеринга террейна
pub struct TerrainRenderer {
    device: Arc<dyn IDevice>,
    /// Загруженные чанки с их мешами
    rendered_chunks: HashMap<ChunkId, RenderedChunk>,
    /// Материал для террейна
    terrain_material: Option<ResourceHandle>,
    /// Текущий LOD bias
    lod_bias: i32,
    /// Дистанции переключения LOD
    lod_distances: [f32; 4],
}

impl TerrainRenderer {
    pub fn new(device: Arc<dyn IDevice>) -> Self {
        Self {
            device,
            rendered_chunks: HashMap::new(),
            terrain_material: None,
            lod_bias: 0,
            lod_distances: [32.0, 64.0, 128.0, 256.0], // Дистанции для LOD 0, 1, 2, 3
        }
    }
    
    /// Инициализирует материал террейна
    pub fn initialize(&mut self) -> Result<(), String> {
        // Создаём материал террейна через MaterialManager
        self.terrain_material = Some(self.create_terrain_material()?);
        Ok(())
    }

    /// Creates terrain material
    fn create_terrain_material(&self) -> Result<ResourceHandle, String> {
        // Материал для террейна с текстурами и высотой
        Ok(ResourceHandle::default()) // В реальной реализации создать материал
    }
    
    /// Добавляет или обновляет чанк для рендеринга
    pub fn update_chunk(&mut self, chunk_id: ChunkId, chunk_data: &ChunkData) -> Result<(), String> {
        // Определяем LOD на основе расстояния до камеры (пока 0)
        let lod_level = 0;
        
        // Генерируем меш
        let mesh = TerrainMeshBuilder::build_mesh(
            self.device.as_ref(),
            chunk_data,
            lod_level
        ).map_err(|e| format!("Failed to build terrain mesh: {:?}", e))?;
        
        // Создаём или обновляем RenderedChunk
        let rendered_chunk = RenderedChunk {
            chunk_id,
            mesh,
            material: self.terrain_material.unwrap_or_default(),
            lod_level,
            is_visible: true,
        };
        
        self.rendered_chunks.insert(chunk_id, rendered_chunk);
        
        Ok(())
    }
    
    /// Удаляет чанк из рендеринга
    pub fn remove_chunk(&mut self, chunk_id: ChunkId) {
        self.rendered_chunks.remove(&chunk_id);
    }
    
    /// Вычисляет подходящий LOD для чанка на основе расстояния до камеры
    pub fn calculate_lod(&self, chunk: &Chunk, camera_pos: Vector3<f32>) -> u32 {
        let distance = chunk.distance_to(camera_pos);
        
        // Учитываем lod_bias
        let effective_distances = self.lod_distances.iter()
            .map(|&d| d * (1.0 + self.lod_bias as f32 * 0.25))
            .collect::<Vec<_>>();
        
        for (lod, &dist) in effective_distances.iter().enumerate() {
            if distance < dist {
                return lod as u32;
            }
        }
        
        // Максимальный LOD (наименьшая детализация)
        3
    }
    
    /// Собирает команды рендеринга для видимых чанков
    pub fn collect_render_commands(&self, _camera_pos: Vector3<f32>, frustum_planes: &[[f32; 4]; 6]) -> Vec<RenderCommand> {
        let mut commands = Vec::with_capacity(self.rendered_chunks.len());
        
        for rendered_chunk in self.rendered_chunks.values() {
            if !rendered_chunk.is_visible {
                continue;
            }
            
            // Frustum culling
            let chunk_world_pos = rendered_chunk.chunk_id.world_position();
            let center = Vector3::new(
                chunk_world_pos.x + CHUNK_SIZE as f32 / 2.0,
                0.0,
                chunk_world_pos.z + CHUNK_SIZE as f32 / 2.0,
            );
            let bounding_radius = CHUNK_SIZE as f32 * 0.75;
            
            if !Self::sphere_in_frustum(&center, bounding_radius, frustum_planes) {
                continue;
            }
            
            // Добавляем команду рендеринга
            commands.push(RenderCommand::TerrainChunk {
                chunk_id: (rendered_chunk.chunk_id.x, rendered_chunk.chunk_id.z),
                mesh: rendered_chunk.mesh.vertex_buffer,
                material: rendered_chunk.material,
                transform: nalgebra::Matrix4::identity(),
                lod: rendered_chunk.lod_level,
            });
        }
        
        commands
    }
    
    /// Проверяет пересечение сферы с фрустумом
    fn sphere_in_frustum(center: &Vector3<f32>, radius: f32, planes: &[[f32; 4]; 6]) -> bool {
        for plane in planes.iter() {
            let distance = plane[0] * center.x + plane[1] * center.y + plane[2] * center.z + plane[3];
            if distance < -radius {
                return false;
            }
        }
        true
    }
    
    /// Устанавливает LOD bias
    pub fn set_lod_bias(&mut self, bias: i32) {
        self.lod_bias = bias.clamp(-3, 3);
    }
    
    /// Получает количество отрендеренных чанков
    pub fn rendered_chunk_count(&self) -> usize {
        self.rendered_chunks.len()
    }
    
    /// Очищает все чанки
    pub fn clear(&mut self) {
        self.rendered_chunks.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sphere_frustum_test() {
        // Простые плоскости для теста (куб от -1 до 1)
        let planes = [
            [1.0, 0.0, 0.0, 1.0],   // Left
            [-1.0, 0.0, 0.0, 1.0],  // Right
            [0.0, 1.0, 0.0, 1.0],   // Bottom
            [0.0, -1.0, 0.0, 1.0],  // Top
            [0.0, 0.0, 1.0, 1.0],   // Near
            [0.0, 0.0, -1.0, 1.0],  // Far
        ];
        
        // Точка внутри должна быть видима
        assert!(TerrainRenderer::sphere_in_frustum(&Vector3::new(0.0, 0.0, 0.0), 0.5, &planes));
        
        // Точка снаружи не видима
        assert!(!TerrainRenderer::sphere_in_frustum(&Vector3::new(5.0, 0.0, 0.0), 0.5, &planes));
    }
}
