//! Texture management module for RTGC engine
//! Handles loading, caching, and lifecycle of texture resources

use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;
use crate::graphics::rhi::types::{IDevice, TextureHandle, ResourceHandle, TextureFormat, TextureDimension};
use crate::graphics::resources::{ResourceManager, ResourceId};

/// Texture metadata
#[derive(Debug, Clone)]
pub struct TextureInfo {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: TextureFormat,
    pub dimension: TextureDimension,
    pub mip_levels: u32,
    pub array_size: u32,
}

impl TextureInfo {
    pub fn new(
        name: &str,
        width: u32,
        height: u32,
        depth: u32,
        format: TextureFormat,
        dimension: TextureDimension,
        mip_levels: u32,
        array_size: u32,
    ) -> Self {
        Self {
            name: name.to_string(),
            width,
            height,
            depth,
            format,
            dimension,
            mip_levels,
            array_size,
        }
    }

    /// Calculates approximate memory size in bytes
    pub fn memory_size(&self) -> usize {
        let pixel_bytes = match self.format {
            TextureFormat::R8Unorm => 1,
            TextureFormat::R8G8Unorm => 2,
            TextureFormat::R8G8B8A8Unorm => 4,
            TextureFormat::R16Float => 2,
            TextureFormat::R16G16Float => 4,
            TextureFormat::R16G16B16A16Float => 8,
            TextureFormat::R32Float => 4,
            TextureFormat::R32G32Float => 8,
            TextureFormat::R32G32B32A32Float => 16,
            TextureFormat::BC1Unorm => 0.5, // Compressed
            TextureFormat::BC3Unorm => 1.0, // Compressed
            _ => 4,
        };

        let base_size = (self.width * self.height * self.depth) as f32 * pixel_bytes;
        
        // Account for mipmaps (sum of geometric series)
        let mip_factor = if self.mip_levels > 1 {
            2.0 - 1.0 / (1 << (self.mip_levels - 1)) as f32
        } else {
            1.0
        };

        (base_size * mip_factor * self.array_size as f32) as usize
    }
}

/// Manages all texture resources
pub struct TextureManager {
    device: Arc<dyn IDevice>,
    resource_manager: ResourceManager,
    textures: HashMap<ResourceId, TextureHandle>,
    texture_info: HashMap<ResourceId, TextureInfo>,
    path_to_id: HashMap<String, ResourceId>,
    default_white: Option<ResourceId>,
    default_black: Option<ResourceId>,
    default_normal: Option<ResourceId>,
}

impl TextureManager {
    /// Creates a new TextureManager
    pub fn new(device: Arc<dyn IDevice>) -> Self {
        let resource_manager = ResourceManager::new(device.clone());
        Self {
            device,
            resource_manager,
            textures: HashMap::new(),
            texture_info: HashMap::new(),
            path_to_id: HashMap::new(),
            default_white: None,
            default_black: None,
            default_normal: None,
        }
    }

    /// Initializes default textures
    pub fn initialize(&mut self) -> Result<(), String> {
        // Create 1x1 white texture
        let white_data = vec![255u8; 4]; // RGBA
        self.default_white = Some(self.create_texture_from_data(
            "default_white",
            1,
            1,
            1,
            TextureFormat::R8G8B8A8Unorm,
            TextureDimension::Tex2D,
            &white_data,
        )?);

        // Create 1x1 black texture
        let black_data = vec![0u8; 4];
        self.default_black = Some(self.create_texture_from_data(
            "default_black",
            1,
            1,
            1,
            TextureFormat::R8G8B8A8Unorm,
            TextureDimension::Tex2D,
            &black_data,
        )?);

        // Create 1x1 normal map texture (RGB = 128, 128, 255, A = 255)
        let normal_data = vec![128u8, 128, 255, 255];
        self.default_normal = Some(self.create_texture_from_data(
            "default_normal",
            1,
            1,
            1,
            TextureFormat::R8G8B8A8Unorm,
            TextureDimension::Tex2D,
            &normal_data,
        )?);

        Ok(())
    }

    /// Gets the default white texture
    pub fn default_white(&self) -> Option<TextureHandle> {
        self.default_white.and_then(|id| {
            self.textures.get(&id).copied()
        })
    }

    /// Gets the default black texture
    pub fn default_black(&self) -> Option<TextureHandle> {
        self.default_black.and_then(|id| {
            self.textures.get(&id).copied()
        })
    }

    /// Gets the default normal texture
    pub fn default_normal(&self) -> Option<TextureHandle> {
        self.default_normal.and_then(|id| {
            self.textures.get(&id).copied()
        })
    }

    /// Loads a texture from file
    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<ResourceId, String> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        // Check if already loaded
        if let Some(&id) = self.path_to_id.get(&path_str) {
            self.resource_manager.touch_resource(id);
            return Ok(id);
        }

        // Load image data
        let img = image::open(path.as_ref())
            .map_err(|e| format!("Failed to load image {:?}: {}", path.as_ref(), e))?;
        
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        // Create texture from data
        let id = self.create_texture_from_data(
            &path_str,
            width,
            height,
            1,
            TextureFormat::R8G8B8A8Unorm,
            TextureDimension::Tex2D,
            rgba.as_raw(),
        )?;

        // Cache path
        self.path_to_id.insert(path_str, id);

        Ok(id)
    }

    /// Creates a texture from raw pixel data
    pub fn create_texture_from_data(
        &mut self,
        name: &str,
        width: u32,
        height: u32,
        depth: u32,
        format: TextureFormat,
        dimension: TextureDimension,
        data: &[u8],
    ) -> Result<ResourceId, String> {
        // Create texture via RHI
        let handle = self.device.create_texture_2d(
            width,
            height,
            format,
            Some(name),
        )?;

        // Upload data
        self.device.update_texture(
            handle.into(),
            0,
            0,
            0,
            width,
            height,
            depth,
            data,
        )?;

        // Create info
        let info = TextureInfo::new(
            name,
            width,
            height,
            depth,
            format,
            dimension,
            1, // mip levels
            1, // array size
        );

        // Register with resource manager
        let id = self.resource_manager.register_texture(
            handle,
            name,
            info.memory_size(),
        );

        // Store references
        self.textures.insert(id, handle);
        self.texture_info.insert(id, info);

        Ok(id)
    }

    /// Gets a texture handle by ID
    pub fn get_texture(&self, id: ResourceId) -> Option<TextureHandle> {
        self.textures.get(&id).copied()
    }

    /// Gets texture info by ID
    pub fn get_texture_info(&self, id: ResourceId) -> Option<&TextureInfo> {
        self.texture_info.get(&id)
    }

    /// Gets a resource handle wrapper
    pub fn get_resource_handle(&self, id: ResourceId) -> Option<ResourceHandle> {
        self.textures.get(&id).map(|h| (*h).into())
    }

    /// Unloads a texture by ID
    pub fn unload_texture(&mut self, id: ResourceId) -> bool {
        if let Some(info) = self.texture_info.remove(&id) {
            // Remove from path cache
            self.path_to_id.retain(|_, &mut v| v != id);
            
            // Remove from texture map
            self.textures.remove(&id);
            
            // Remove from resource manager
            self.resource_manager.remove_resource(id);
            
            true
        } else {
            false
        }
    }

    /// Unloads all unused textures (not used for more than duration)
    pub fn clear_unused(&mut self, duration: std::time::Duration) -> Vec<ResourceId> {
        let removed = self.resource_manager.clear_unused(duration);
        
        for &id in &removed {
            self.texture_info.remove(&id);
            self.textures.remove(&id);
        }
        
        // Clean up path cache
        self.path_to_id.retain(|_, id| !removed.contains(id));
        
        removed
    }

    /// Gets memory usage statistics
    pub fn get_stats(&self) -> TextureManagerStats {
        let rm_stats = self.resource_manager.get_stats();
        let total_memory = self.texture_info.values()
            .map(|info| info.memory_size())
            .sum::<usize>();

        TextureManagerStats {
            total_textures: self.textures.len(),
            total_memory_usage: total_memory,
            resource_manager_stats: rm_stats,
        }
    }

    /// Prints debug information
    pub fn print_debug_info(&self) {
        tracing::info!("=== TextureManager Stats ===");
        tracing::info!("Total textures: {}", self.textures.len());
        
        let total_memory = self.texture_info.values()
            .map(|info| info.memory_size())
            .sum::<usize>();
        
        tracing::info!("Memory usage: {:.2} MB", total_memory as f32 / (1024.0 * 1024.0));
        
        self.resource_manager.print_debug_info();
    }
}

/// Statistics about texture manager state
#[derive(Debug, Clone)]
pub struct TextureManagerStats {
    pub total_textures: usize,
    pub total_memory_usage: usize,
    pub resource_manager_stats: crate::graphics::resources::ResourceManagerStats,
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new(Arc::new(
            crate::graphics::rhi::gl::GLDevice::new()
                .unwrap_or_else(|_| panic!("Failed to create GL device for TextureManager"))
        ))
    }
}