//! Resource management module for RTGC engine
//! Handles loading, caching, and lifecycle of graphics resources

use std::collections::HashMap;
use std::sync::Arc;
use crate::graphics::rhi::types::{ResourceHandle, IDevice, BufferHandle, TextureHandle, PipelineStateHandle};

/// Unique identifier for resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u64);

impl ResourceId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Types of resources that can be managed
#[derive(Debug, Clone)]
pub enum ResourceType {
    Buffer(BufferHandle),
    Texture(TextureHandle),
    PipelineState(PipelineStateHandle),
}

/// Metadata for a managed resource
#[derive(Debug)]
pub struct ResourceMetadata {
    pub name: String,
    pub resource_type: String,
    pub size_bytes: usize,
    pub created_at: std::time::Instant,
    pub last_used: std::time::Instant,
}

impl ResourceMetadata {
    pub fn new(name: &str, resource_type: &str, size_bytes: usize) -> Self {
        let now = std::time::Instant::now();
        Self {
            name: name.to_string(),
            resource_type: resource_type.to_string(),
            size_bytes,
            created_at: now,
            last_used: now,
        }
    }
}

/// Manages all graphics resources with automatic cleanup
pub struct ResourceManager {
    device: Arc<dyn IDevice>,
    resources: HashMap<ResourceId, ResourceType>,
    metadata: HashMap<ResourceId, ResourceMetadata>,
    next_id: u64,
    total_memory_usage: usize,
    max_memory_usage: usize,
}

impl ResourceManager {
    /// Creates a new ResourceManager
    pub fn new(device: Arc<dyn IDevice>) -> Self {
        Self {
            device,
            resources: HashMap::new(),
            metadata: HashMap::new(),
            next_id: 0,
            total_memory_usage: 0,
            max_memory_usage: 256 * 1024 * 1024, // 256 MB default limit
        }
    }

    /// Sets maximum memory usage limit
    pub fn set_memory_limit(&mut self, limit_bytes: usize) {
        self.max_memory_usage = limit_bytes;
    }

    /// Allocates a new resource ID
    fn allocate_id(&mut self) -> ResourceId {
        let id = ResourceId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Registers a buffer resource
    pub fn register_buffer(&mut self, handle: BufferHandle, name: &str, size_bytes: usize) -> ResourceId {
        let id = self.allocate_id();
        self.resources.insert(id, ResourceType::Buffer(handle));
        self.metadata.insert(id, ResourceMetadata::new(name, "buffer", size_bytes));
        self.total_memory_usage += size_bytes;
        id
    }

    /// Registers a texture resource
    pub fn register_texture(&mut self, handle: TextureHandle, name: &str, size_bytes: usize) -> ResourceId {
        let id = self.allocate_id();
        self.resources.insert(id, ResourceType::Texture(handle));
        self.metadata.insert(id, ResourceMetadata::new(name, "texture", size_bytes));
        self.total_memory_usage += size_bytes;
        id
    }

    /// Registers a pipeline state resource
    pub fn register_pipeline(&mut self, handle: PipelineStateHandle, name: &str) -> ResourceId {
        let id = self.allocate_id();
        self.resources.insert(id, ResourceType::PipelineState(handle));
        self.metadata.insert(id, ResourceMetadata::new(name, "pipeline", 0));
        id
    }

    /// Gets a resource by ID
    pub fn get_resource(&self, id: ResourceId) -> Option<&ResourceType> {
        self.resources.get(&id)
    }

    /// Gets metadata for a resource
    pub fn get_metadata(&self, id: ResourceId) -> Option<&ResourceMetadata> {
        self.metadata.get(&id)
    }

    /// Updates last used time for a resource
    pub fn touch_resource(&mut self, id: ResourceId) {
        if let Some(meta) = self.metadata.get_mut(&id) {
            meta.last_used = std::time::Instant::now();
        }
    }

    /// Removes a resource by ID
    pub fn remove_resource(&mut self, id: ResourceId) -> Option<ResourceType> {
        if let Some(resource) = self.resources.remove(&id) {
            if let Some(meta) = self.metadata.remove(&id) {
                self.total_memory_usage = self.total_memory_usage.saturating_sub(meta.size_bytes);
            }
            Some(resource)
        } else {
            None
        }
    }

    /// Clears unused resources (not used for more than duration)
    pub fn clear_unused(&mut self, duration: std::time::Duration) -> Vec<ResourceId> {
        let now = std::time::Instant::now();
        let mut removed = Vec::new();

        let to_remove: Vec<ResourceId> = self.metadata
            .iter()
            .filter(|(_, meta)| now.duration_since(meta.last_used) > duration)
            .map(|(&id, _)| id)
            .collect();

        for id in to_remove {
            if self.remove_resource(id).is_some() {
                removed.push(id);
            }
        }

        removed
    }

    /// Forces cleanup until memory usage is below threshold
    pub fn trim_memory(&mut self, target_usage: usize) -> Vec<ResourceId> {
        let mut removed = Vec::new();

        while self.total_memory_usage > target_usage {
            // Find least recently used resource
            let lru = self.metadata
                .iter()
                .min_by_key(|(_, meta)| meta.last_used)
                .map(|(&id, _)| id);

            if let Some(id) = lru {
                if self.remove_resource(id).is_some() {
                    removed.push(id);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        removed
    }

    /// Gets current memory usage
    pub fn memory_usage(&self) -> usize {
        self.total_memory_usage
    }

    /// Gets memory limit
    pub fn memory_limit(&self) -> usize {
        self.max_memory_usage
    }

    /// Gets number of active resources
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    /// Gets statistics about resource usage
    pub fn get_stats(&self) -> ResourceManagerStats {
        ResourceManagerStats {
            total_resources: self.resources.len(),
            total_memory_usage: self.total_memory_usage,
            max_memory_usage: self.max_memory_usage,
            memory_utilization: if self.max_memory_usage > 0 {
                self.total_memory_usage as f32 / self.max_memory_usage as f32
            } else {
                0.0
            },
        }
    }

    /// Prints debug information about resources
    pub fn print_debug_info(&self) {
        tracing::info!("=== ResourceManager Stats ===");
        tracing::info!("Total resources: {}", self.resources.len());
        tracing::info!("Memory usage: {:.2} MB / {:.2} MB",
            self.total_memory_usage as f32 / (1024.0 * 1024.0),
            self.max_memory_usage as f32 / (1024.0 * 1024.0));
        
        let mut buffers = 0;
        let mut textures = 0;
        let mut pipelines = 0;
        
        for resource in self.resources.values() {
            match resource {
                ResourceType::Buffer(_) => buffers += 1,
                ResourceType::Texture(_) => textures += 1,
                ResourceType::PipelineState(_) => pipelines += 1,
            }
        }
        
        tracing::info!("Buffers: {}, Textures: {}, Pipelines: {}", buffers, textures, pipelines);
    }
}

/// Statistics about resource manager state
#[derive(Debug, Clone)]
pub struct ResourceManagerStats {
    pub total_resources: usize,
    pub total_memory_usage: usize,
    pub max_memory_usage: usize,
    pub memory_utilization: f32,
}

impl Default for ResourceManager {
    fn default() -> Self {
        // Create a dummy device-aware manager - in real use, device should be provided
        Self {
            device: Arc::new(crate::graphics::rhi::gl::GLDevice::new().unwrap_or_else(|_| {
                panic!("Failed to create GL device for ResourceManager")
            })),
            resources: HashMap::new(),
            metadata: HashMap::new(),
            next_id: 0,
            total_memory_usage: 0,
            max_memory_usage: 256 * 1024 * 1024,
        }
    }
}