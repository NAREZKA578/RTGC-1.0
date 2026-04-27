// Vulkan Backend - Buffer Implementation
// Implements GPU buffer resources for Vulkan

use crate::graphics::rhi::types::*;

#[cfg(feature = "vulkan")]
use ash::vk;

/// Vulkan Buffer resource
pub struct VkBuffer {
    #[cfg(feature = "vulkan")]
    buffer: vk::Buffer,
    
    #[cfg(feature = "vulkan")]
    allocation: Option<vk::DeviceMemory>,
    
    handle: ResourceHandle,
    description: BufferDescription,
    size: u64,
}

unsafe impl Send for VkBuffer {}
unsafe impl Sync for VkBuffer {}

impl VkBuffer {
    /// Create a new Vulkan buffer
    #[cfg(feature = "vulkan")]
    pub fn new(
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        desc: &BufferDescription,
        handle: ResourceHandle,
    ) -> RhiResult<Self> {
        use ash::vk;
        
        let usage = Self::to_vk_usage(desc.usage);
        
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(desc.size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        
        let buffer = unsafe {
            device.create_buffer(&buffer_info, None)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create buffer: {:?}", e)))?
        };
        
        // Get memory requirements
        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        
        // Find suitable memory type
        let memory_type_index = Self::find_memory_type(
            physical_device,
            mem_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        ).ok_or_else(|| RhiError::ResourceCreationFailed("No suitable memory type found".to_string()))?;
        
        // Allocate memory
        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index);
        
        let memory = unsafe {
            device.allocate_memory(&alloc_info, None)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to allocate memory: {:?}", e)))?
        };
        
        // Bind memory to buffer
        unsafe {
            device.bind_buffer_memory(buffer, memory, 0)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to bind buffer memory: {:?}", e)))?;
        }
        
        Ok(Self {
            buffer,
            allocation: Some(memory),
            handle,
            description: desc.clone(),
            size: desc.size,
        })
    }
    
    #[cfg(feature = "vulkan")]
    fn to_vk_usage(usage: BufferUsage) -> vk::BufferUsageFlags {
        let mut flags = vk::BufferUsageFlags::empty();
        
        if usage.contains(BufferUsage::VERTEX_BUFFER) {
            flags |= vk::BufferUsageFlags::VERTEX_BUFFER;
        }
        if usage.contains(BufferUsage::INDEX_BUFFER) {
            flags |= vk::BufferUsageFlags::INDEX_BUFFER;
        }
        if usage.contains(BufferUsage::CONSTANT_BUFFER) {
            flags |= vk::BufferUsageFlags::UNIFORM_BUFFER;
        }
        if usage.contains(BufferUsage::SHADER_RESOURCE) {
            flags |= vk::BufferUsageFlags::SAMPLED_BUFFER;
        }
        if usage.contains(BufferUsage::UNORDERED_ACCESS) {
            flags |= vk::BufferUsageFlags::STORAGE_BUFFER;
        }
        if usage.contains(BufferUsage::TRANSFER_SRC) {
            flags |= vk::BufferUsageFlags::TRANSFER_SRC;
        }
        if usage.contains(BufferUsage::TRANSFER_DST) {
            flags |= vk::BufferUsageFlags::TRANSFER_DST;
        }
        if usage.contains(BufferUsage::STORAGE_BUFFER) {
            flags |= vk::BufferUsageFlags::STORAGE_BUFFER;
        }
        if usage.contains(BufferUsage::INDIRECT_BUFFER) {
            flags |= vk::BufferUsageFlags::INDIRECT_BUFFER;
        }
        
        flags
    }
    
    #[cfg(feature = "vulkan")]
    fn find_memory_type(
        physical_device: vk::PhysicalDevice,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        use ash::vk;
        
        // Get physical device properties (would need instance here)
        // For now, return first matching type
        for i in 0..32 {
            if (type_filter & (1 << i)) != 0 {
                return Some(i);
            }
        }
        None
    }
    
    #[cfg(not(feature = "vulkan"))]
    pub fn new(
        _device: &ash::Device,
        _physical_device: vk::PhysicalDevice,
        _desc: &BufferDescription,
        _handle: ResourceHandle,
    ) -> RhiResult<Self> {
        Err(RhiError::Unsupported("Vulkan feature not enabled".to_string()))
    }
    
    #[cfg(feature = "vulkan")]
    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }
    
    pub fn handle(&self) -> ResourceHandle {
        self.handle
    }
    
    pub fn description(&self) -> &BufferDescription {
        &self.description
    }
    
    pub fn size(&self) -> u64 {
        self.size
    }
}
