//! Chunk Manager for RTGC-0.9
//! Менеджер жизненного цикла чанков

use crate::world::chunk::Chunk;
use crate::world::generate_chunk_mesh;
use std::collections::{HashMap, HashSet};

/// Координаты чанка
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoords {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoords {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn from_world_pos(world_x: f32, world_z: f32, chunk_size: f32) -> Self {
        Self {
            x: (world_x / chunk_size).floor() as i32,
            z: (world_z / chunk_size).floor() as i32,
        }
    }
}

/// Менеджер чанков
pub struct ChunkManager {
    chunks: HashMap<ChunkCoords, Chunk>,
    chunk_size: f32,
    render_distance: i32,
    pending_loads: Vec<ChunkCoords>,
    pending_unloads: Vec<ChunkCoords>,
}

impl ChunkManager {
    pub fn new(chunk_size: f32, render_distance: i32) -> Self {
        Self {
            chunks: HashMap::new(),
            chunk_size,
            render_distance,
            pending_loads: Vec::new(),
            pending_unloads: Vec::new(),
        }
    }

    /// Обновление позиции камеры и управление загрузкой/выгрузкой чанков
    pub fn update_camera_position(&mut self, camera_x: f32, camera_z: f32) {
        let center = ChunkCoords::from_world_pos(camera_x, camera_z, self.chunk_size);

        // Определяем какие чанки должны быть загружены
        let mut required_chunks = HashSet::new();
        for dx in -self.render_distance..=self.render_distance {
            for dz in -self.render_distance..=self.render_distance {
                required_chunks.insert(ChunkCoords {
                    x: center.x + dx,
                    z: center.z + dz,
                });
            }
        }

        // Находим чанки для загрузки
        for coords in &required_chunks {
            if !self.chunks.contains_key(coords) && !self.pending_loads.contains(coords) {
                self.pending_loads.push(*coords);
            }
        }

        // Находим чанки для выгрузки
        for coords in self.chunks.keys() {
            if !required_chunks.contains(coords) && !self.pending_unloads.contains(coords) {
                self.pending_unloads.push(*coords);
            }
        }
    }

    /// Асинхронная загрузка чанка
    pub fn load_chunk(&mut self, coords: ChunkCoords) -> Option<Chunk> {
        // Создаём пустые данные чанка
        let chunk_data = crate::world::chunk::ChunkData::new();

        // Генерируем меш чанка
        let (vertices, indices) = generate_chunk_mesh(&chunk_data, 0);

        // Создаём ID чанка
        let chunk_id = crate::world::chunk::ChunkId::new(coords.x, coords.z);

        // Создаём чанк
        let chunk = Chunk::new(chunk_id, chunk_data);

        self.chunks.insert(coords, chunk);
        self.chunks.get(&coords).cloned()
    }

    /// Выгрузка чанка
    pub fn unload_chunk(&mut self, coords: ChunkCoords) -> Option<Chunk> {
        self.chunks.remove(&coords)
    }

    /// Получение чанка по координатам
    pub fn get_chunk(&self, coords: ChunkCoords) -> Option<&Chunk> {
        self.chunks.get(&coords)
    }

    /// Получение чанка по мировым координатам
    pub fn get_chunk_at_world_pos(&self, world_x: f32, world_z: f32) -> Option<&Chunk> {
        let coords = ChunkCoords::from_world_pos(world_x, world_z, self.chunk_size);
        self.get_chunk(coords)
    }

    /// Обработка очереди загрузки
    pub fn process_load_queue(&mut self, max_per_frame: usize) -> usize {
        let mut loaded = 0;
        while loaded < max_per_frame && !self.pending_loads.is_empty() {
            if let Some(coords) = self.pending_loads.pop() {
                if !self.chunks.contains_key(&coords) {
                    self.load_chunk(coords);
                    loaded += 1;
                }
            }
        }
        loaded
    }

    /// Обработка очереди выгрузки
    pub fn process_unload_queue(&mut self, max_per_frame: usize) -> usize {
        let mut unloaded = 0;
        while unloaded < max_per_frame && !self.pending_unloads.is_empty() {
            if let Some(coords) = self.pending_unloads.pop() {
                self.unload_chunk(coords);
                unloaded += 1;
            }
        }
        unloaded
    }

    /// Получить количество загруженных чанков
    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Получить размер чанка
    pub fn chunk_size(&self) -> f32 {
        self.chunk_size
    }

    /// Получить дистанцию рендеринга
    pub fn render_distance(&self) -> i32 {
        self.render_distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_coords_from_world_pos() {
        let coords = ChunkCoords::from_world_pos(50.0, 75.0, 100.0);
        assert_eq!(coords.x, 0);
        assert_eq!(coords.z, 0);

        let coords = ChunkCoords::from_world_pos(150.0, 250.0, 100.0);
        assert_eq!(coords.x, 1);
        assert_eq!(coords.z, 2);
    }

    #[test]
    fn test_chunk_manager_creation() {
        let manager = ChunkManager::new(100.0, 5);
        assert_eq!(manager.chunk_size(), 100.0);
        assert_eq!(manager.render_distance(), 5);
        assert_eq!(manager.loaded_chunk_count(), 0);
    }
}
