//! Prop Spawner for RTGC-0.9
//! Расстановка объектов на террейне (деревья, здания, камни и т.д.)

use nalgebra::Vector3;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use crate::world::chunk::{ChunkId, CHUNK_SIZE};

/// Тип пропса (объекта)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropType {
    Tree,
    Rock,
    Building,
    Bush,
    Fence,
    Lamp,
    Custom(u32),
}

/// Информация о пропсе
#[derive(Debug, Clone)]
pub struct Prop {
    pub prop_type: PropType,
    pub position: Vector3<f32>,
    pub rotation: f32, // Угол вокруг оси Y в радианах
    pub scale: Vector3<f32>,
    pub lod_distance: f32,
}

/// Конфигурация спавна пропсов
#[derive(Debug, Clone)]
pub struct PropSpawnConfig {
    pub density: f32, // Количество пропсов на квадратный метр
    pub min_slope: f32, // Минимальный уклон для спавна
    pub max_slope: f32, // Максимальный уклон для спавна
    pub min_height: f32, // Минимальная высота
    pub max_height: f32, // Максимальная высота
    pub exclude_water: bool, // Исключить воду
    pub prop_types: Vec<(PropType, f32)>, // Тип и вес вероятности
}

impl Default for PropSpawnConfig {
    fn default() -> Self {
        Self {
            density: 0.001, // 1 пропс на 1000 м²
            min_slope: 0.0,
            max_slope: 0.5, // ~27 градусов
            min_height: -10.0,
            max_height: 1000.0,
            exclude_water: true,
            prop_types: vec![
                (PropType::Tree, 0.5),
                (PropType::Rock, 0.3),
                (PropType::Bush, 0.15),
                (PropType::Fence, 0.05),
            ],
        }
    }
}

/// Менеджер спавна пропсов
pub struct PropSpawner {
    config: PropSpawnConfig,
    seed: u64,
    spawned_props: Vec<Prop>,
    chunk_props: std::collections::HashMap<ChunkId, Vec<Prop>>,
}

impl PropSpawner {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            config: PropSpawnConfig::default(),
            spawned_props: Vec::new(),
            chunk_props: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_config(seed: u64, config: PropSpawnConfig) -> Self {
        Self {
            seed,
            config,
            spawned_props: Vec::new(),
            chunk_props: std::collections::HashMap::new(),
        }
    }
    
    /// Спавн пропсов в чанке
    pub fn spawn_chunk(&mut self, chunk_id: ChunkId, get_height: impl Fn(f32, f32) -> f32) -> Vec<Prop> {
        // Проверяем, уже ли заспавнен этот чанк
        if let Some(props) = self.chunk_props.get(&chunk_id) {
            return props.clone();
        }
        
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed.wrapping_add(chunk_id.x as u64 * 1000 + chunk_id.z as u64));
        
        let mut props = Vec::new();
        let chunk_area = (CHUNK_SIZE * CHUNK_SIZE) as f32;
        let num_props = (chunk_area * self.config.density) as usize;
        
        for _ in 0..num_props {
            // Случайная позиция в чанке
            let local_x = rng.gen_range(0.0..CHUNK_SIZE as f32);
            let local_z = rng.gen_range(0.0..CHUNK_SIZE as f32);
            
            let world_x = chunk_id.x as f32 * CHUNK_SIZE as f32 + local_x;
            let world_z = chunk_id.z as f32 * CHUNK_SIZE as f32 + local_z;
            
            let height = get_height(world_x, world_z);
            
            // Проверка высоты
            if height < self.config.min_height || height > self.config.max_height {
                continue;
            }
            
            // Проверка уклона
            if !self.is_valid_slope(world_x, world_z, &get_height) {
                continue;
            }
            
            // Выбор типа пропса
            let prop_type = self.select_prop_type(&mut rng);
            
            // Создание пропса
            let prop = Prop {
                prop_type,
                position: Vector3::new(world_x, height, world_z),
                rotation: rng.gen_range(0.0..std::f32::consts::TAU),
                scale: self.get_prop_scale(prop_type, &mut rng),
                lod_distance: self.get_lod_distance(prop_type),
            };
            
            props.push(prop);
        }
        
        self.chunk_props.insert(chunk_id, props.clone());
        self.spawned_props.extend(props.clone());
        
        props
    }
    
    /// Проверка уклона местности
    fn is_valid_slope(&self, x: f32, z: f32, get_height: &impl Fn(f32, f32) -> f32) -> bool {
        let sample_dist = 1.0;
        let h_center = get_height(x, z);
        let h_right = get_height(x + sample_dist, z);
        let h_left = get_height(x - sample_dist, z);
        let h_front = get_height(x, z + sample_dist);
        let h_back = get_height(x, z - sample_dist);
        
        let dx = (h_right - h_left) / (2.0 * sample_dist);
        let dz = (h_front - h_back) / (2.0 * sample_dist);
        
        let slope = (dx * dx + dz * dz).sqrt();
        
        slope >= self.config.min_slope && slope <= self.config.max_slope
    }
    
    /// Выбор типа пропса на основе весов
    fn select_prop_type(&self, rng: &mut ChaCha8Rng) -> PropType {
        let total_weight: f32 = self.config.prop_types.iter().map(|(_, w)| w).sum();
        let mut roll = rng.gen_range(0.0..total_weight);
        
        for (prop_type, weight) in &self.config.prop_types {
            if roll < *weight {
                return *prop_type;
            }
            roll -= weight;
        }
        
        PropType::Tree // Fallback
    }
    
    /// Получение масштаба для типа пропса
    fn get_prop_scale(&self, prop_type: PropType, rng: &mut ChaCha8Rng) -> Vector3<f32> {
        let base_scale = match prop_type {
            PropType::Tree => rng.gen_range(0.8..1.5),
            PropType::Rock => rng.gen_range(0.5..2.0),
            PropType::Building => rng.gen_range(0.8..1.2),
            PropType::Bush => rng.gen_range(0.5..1.0),
            PropType::Fence => 1.0,
            PropType::Lamp => 1.0,
            PropType::Custom(_) => 1.0,
        };
        
        // Небольшая вариация масштаба
        let variation = rng.gen_range(0.9..1.1);
        let scale = base_scale * variation;
        
        Vector3::new(scale, scale, scale)
    }
    
    /// Получение дистанции LOD для типа пропса
    fn get_lod_distance(&self, prop_type: PropType) -> f32 {
        match prop_type {
            PropType::Tree => 100.0,
            PropType::Rock => 80.0,
            PropType::Building => 150.0,
            PropType::Bush => 50.0,
            PropType::Fence => 60.0,
            PropType::Lamp => 40.0,
            PropType::Custom(_) => 100.0,
        }
    }
    
    /// Получение пропсов в радиусе от позиции
    pub fn get_props_in_radius(&self, position: Vector3<f32>, radius: f32) -> Vec<&Prop> {
        let radius_sq = radius * radius;
        self.spawned_props
            .iter()
            .filter(|prop| {
                let dx = prop.position.x - position.x;
                let dz = prop.position.z - position.z;
                dx * dx + dz * dz <= radius_sq
            })
            .collect()
    }
    
    /// Очистка пропсов из чанка (при выгрузке)
    pub fn clear_chunk(&mut self, chunk_id: ChunkId) {
        if let Some(props) = self.chunk_props.remove(&chunk_id) {
            // Удаляем из общего списка
            self.spawned_props.retain(|p| {
                let prop_chunk_x = (p.position.x / CHUNK_SIZE as f32).floor() as i32;
                let prop_chunk_z = (p.position.z / CHUNK_SIZE as f32).floor() as i32;
                prop_chunk_x != chunk_id.x || prop_chunk_z != chunk_id.z
            });
        }
    }
    
    /// Получить количество заспавненных пропсов
    pub fn prop_count(&self) -> usize {
        self.spawned_props.len()
    }
    
    /// Получить конфигурацию
    pub fn config(&self) -> &PropSpawnConfig {
        &self.config
    }
    
    /// Установить новую конфигурацию
    pub fn set_config(&mut self, config: PropSpawnConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prop_spawner_creation() {
        let spawner = PropSpawner::new(12345);
        assert_eq!(spawner.prop_count(), 0);
    }
    
    #[test]
    fn test_prop_spawn() {
        let mut spawner = PropSpawner::new(12345);
        let chunk_id = ChunkId::new(0, 0);
        
        // Простая функция высоты (плоская поверхность)
        let get_height = |_x: f32, _z: f32| 0.0;
        
        let props = spawner.spawn_chunk(chunk_id, get_height);
        
        // Должны быть созданы пропсы
        assert!(!props.is_empty() || spawner.config.density == 0.0);
    }
    
    #[test]
    fn test_prop_type_selection() {
        let mut spawner = PropSpawner::new(12345);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        
        // Все типы должны выбираться с некоторой вероятностью
        let mut types_selected = std::collections::HashSet::new();
        for _ in 0..100 {
            let prop_type = spawner.select_prop_type(&mut rng);
            types_selected.insert(prop_type);
        }
        
        // Должно быть выбрано хотя бы несколько разных типов
        assert!(types_selected.len() >= 2);
    }
}
