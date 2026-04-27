// Vulkan Backend - Shader Module Implementation
// Implements shader loading and management for Vulkan

use crate::graphics::rhi::types::*;
use std::sync::Arc;

#[cfg(feature = "vulkan")]
use ash::vk;

/// Vulkan shader module wrapper
pub struct VkShader {
    #[cfg(feature = "vulkan")]
    pub shader_module: vk::ShaderModule,
    
    stage: ShaderStage,
    entry_point: String,
}

unsafe impl Send for VkShader {}
unsafe impl Sync for VkShader {}

impl VkShader {
    /// Create a new Vulkan shader module from SPIR-V bytecode
    #[cfg(feature = "vulkan")]
    pub fn new(device: &ash::Device, spirv_data: &[u8], stage: ShaderStage, entry_point: &str) -> RhiResult<Self> {
        use ash::vk;
        
        if spirv_data.len() % 4 != 0 {
            return Err(RhiError::ShaderCompilationFailed("SPIR-V data must be aligned to 4 bytes".to_string()));
        }
        
        let shader_info = vk::ShaderModuleCreateInfo::builder()
            .code(unsafe { std::slice::from_raw_parts(spirv_data.as_ptr() as *const u32, spirv_data.len() / 4) });
        
        let shader_module = unsafe {
            device.create_shader_module(&shader_info, None)
                .map_err(|e| RhiError::ShaderCompilationFailed(format!("Failed to create shader module: {:?}", e)))?
        };
        
        Ok(Self {
            shader_module,
            stage,
            entry_point: entry_point.to_string(),
        })
    }
    
    #[cfg(not(feature = "vulkan"))]
    pub fn new(_device: &ash::Device, _spirv_data: &[u8], _stage: ShaderStage, _entry_point: &str) -> RhiResult<Self> {
        Err(RhiError::Unsupported("Vulkan feature not enabled".to_string()))
    }
    
    /// Get the shader stage
    pub fn get_stage(&self) -> ShaderStage {
        self.stage
    }
    
    /// Get the entry point name
    pub fn get_entry_point(&self) -> &str {
        &self.entry_point
    }
}

impl Drop for VkShader {
    #[cfg(feature = "vulkan")]
    fn drop(&mut self) {
        // Shader module destruction handled by device
    }
    
    #[cfg(not(feature = "vulkan"))]
    fn drop(&mut self) {}
}

/// Helper function to load SPIR-V shader from file
pub fn load_spirv_file(path: &str) -> RhiResult<Vec<u8>> {
    use std::fs;
    
    fs::read(path)
        .map_err(|e| RhiError::ShaderCompilationFailed(format!("Failed to read shader file '{}': {}", path, e)))
}

/// Helper function to create shader description from SPIR-V file
pub fn create_shader_desc_from_file(path: &str, stage: ShaderStage, entry_point: &str) -> RhiResult<ShaderDescription> {
    let source = load_spirv_file(path)?;
    
    Ok(ShaderDescription {
        stage,
        source,
        entry_point: entry_point.to_string(),
    })
}
