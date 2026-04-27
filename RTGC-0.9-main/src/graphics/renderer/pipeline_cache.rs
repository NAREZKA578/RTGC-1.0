//! Pipeline Cache - кэш PSO для избежания повторного создания

use crate::graphics::rhi::{ResourceHandle, PipelineStateObject};
use std::collections::HashMap;

/// Ключ для кэширования пайплайнов
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineKey {
    pub vertex_shader: u64,
    pub fragment_shader: Option<u64>,
    pub input_layout_hash: u64,
    pub blend_state_hash: u64,
    pub depth_state_hash: u64,
    pub rasterizer_state_hash: u64,
    pub primitive_topology: u32,
}

impl PipelineKey {
    pub fn from_pso(pso: &PipelineStateObject) -> Self {
        Self {
            vertex_shader: pso.vertex_shader.0,
            fragment_shader: Some(pso.fragment_shader.0),
            input_layout_hash: 0,
            blend_state_hash: 0,
            depth_state_hash: 0,
            rasterizer_state_hash: 0,
            primitive_topology: 0,
        }
    }
}

/// Кэш Pipeline State Objects
pub struct PipelineCache {
    cache: HashMap<PipelineKey, ResourceHandle>,
    hit_count: usize,
    miss_count: usize,
}

impl PipelineCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            hit_count: 0,
            miss_count: 0,
        }
    }
    
    /// Получает пайплайн из кэша или создаёт новый
    pub fn get_or_insert<F>(&mut self, key: PipelineKey, create_fn: F) -> ResourceHandle
    where
        F: FnOnce() -> ResourceHandle,
    {
        match self.cache.get(&key) {
            Some(handle) => {
                self.hit_count += 1;
                *handle
            }
            None => {
                self.miss_count += 1;
                let pipeline = create_fn();
                self.cache.insert(key, pipeline);
                pipeline
            }
        }
    }
    
    /// Проверяет наличие пайплайна в кэше
    pub fn contains(&self, key: &PipelineKey) -> bool {
        self.cache.contains_key(key)
    }
    
    /// Статистика кэша
    pub fn stats(&self) -> PipelineCacheStats {
        PipelineCacheStats {
            hit_count: self.hit_count,
            miss_count: self.miss_count,
            cached_pipelines: self.cache.len(),
        }
    }
    
    /// Очищает кэш
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hit_count = 0;
        self.miss_count = 0;
    }
}

impl Default for PipelineCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Статистика кэша пайплайнов
#[derive(Debug, Clone, Default)]
pub struct PipelineCacheStats {
    pub hit_count: usize,
    pub miss_count: usize,
    pub cached_pipelines: usize,
}

impl PipelineCacheStats {
    pub fn hit_rate(&self) -> f32 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f32 / total as f32
        }
    }
}
