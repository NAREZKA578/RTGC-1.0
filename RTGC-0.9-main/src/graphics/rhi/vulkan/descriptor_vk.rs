// Vulkan Backend - Descriptor Set Implementation
// Implements descriptor set and heap management for Vulkan

use crate::graphics::rhi::types::*;
use std::sync::Arc;

#[cfg(feature = "vulkan")]
use ash::vk;

/// Vulkan descriptor set wrapper
pub struct VkDescriptorSet {
    #[cfg(feature = "vulkan")]
    pub descriptor_set: vk::DescriptorSet,

    #[cfg(feature = "vulkan")]
    pub layout: vk::DescriptorSetLayout,

    bindings: Vec<u32>,
}

unsafe impl Send for VkDescriptorSet {}
unsafe impl Sync for VkDescriptorSet {}

impl VkDescriptorSet {
    /// Create a new Vulkan descriptor set
    #[cfg(feature = "vulkan")]
    pub fn new(
        device: &ash::Device,
        descriptor_pool: vk::DescriptorPool,
        layout: vk::DescriptorSetLayout,
    ) -> RhiResult<Self> {
        use ash::vk;

        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&[layout]);

        let descriptor_sets = unsafe {
            device.allocate_descriptor_sets(&alloc_info).map_err(|e| {
                RhiError::ResourceCreationFailed(format!(
                    "Failed to allocate descriptor set: {:?}",
                    e
                ))
            })?
        };

        Ok(Self {
            descriptor_set: descriptor_sets[0],
            layout,
            bindings: Vec::new(),
        })
    }

    #[cfg(not(feature = "vulkan"))]
    pub fn new(_device: &ash::Device, _descriptor_pool: u64, _layout: u64) -> RhiResult<Self> {
        Err(RhiError::Unsupported(
            "Vulkan feature not enabled".to_string(),
        ))
    }
    #[cfg(feature = "vulkan")]
    pub fn update_uniform_buffer(
        &mut self,
        device: &ash::Device,
        binding: u32,
        buffer: vk::Buffer,
        offset: u64,
        range: u64,
    ) {
        use ash::vk;

        let descriptor_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffer)
            .offset(offset)
            .range(range);

        let write_desc = vk::WriteDescriptorSet::builder()
            .dst_set(self.descriptor_set)
            .dst_binding(binding)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&[descriptor_info.build()]);

        unsafe {
            device.update_descriptor_sets(&[write_desc.build()], &[]);
        }

        if !self.bindings.contains(&binding) {
            self.bindings.push(binding);
        }
    }

    /// Update descriptor set with a combined image sampler
    #[cfg(feature = "vulkan")]
    pub fn update_combined_image_sampler(
        &mut self,
        device: &ash::Device,
        binding: u32,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
    ) {
        use ash::vk;

        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(image_view)
            .sampler(sampler);

        let write_desc = vk::WriteDescriptorSet::builder()
            .dst_set(self.descriptor_set)
            .dst_binding(binding)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&[image_info.build()]);

        unsafe {
            device.update_descriptor_sets(&[write_desc.build()], &[]);
        }

        if !self.bindings.contains(&binding) {
            self.bindings.push(binding);
        }
    }

    /// Get the bound bindings
    pub fn get_bindings(&self) -> &[u32] {
        &self.bindings
    }
}

/// Vulkan descriptor pool wrapper
pub struct VkDescriptorPool {
    #[cfg(feature = "vulkan")]
    pub pool: vk::DescriptorPool,

    max_sets: u32,
    allocated_sets: Vec<vk::DescriptorSet>,
}

unsafe impl Send for VkDescriptorPool {}
unsafe impl Sync for VkDescriptorPool {}

impl VkDescriptorPool {
    /// Create a new Vulkan descriptor pool
    #[cfg(feature = "vulkan")]
    pub fn new(
        device: &ash::Device,
        max_sets: u32,
        pool_sizes: &[(vk::DescriptorType, u32)],
    ) -> RhiResult<Self> {
        use ash::vk;

        let pool_sizes_vk: Vec<vk::DescriptorPoolSize> = pool_sizes
            .iter()
            .map(|&(ty, count)| {
                vk::DescriptorPoolSize::builder()
                    .ty(ty)
                    .descriptor_count(count)
                    .build()
            })
            .collect();

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(max_sets)
            .pool_sizes(&pool_sizes_vk)
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);

        let pool = unsafe {
            device
                .create_descriptor_pool(&pool_info, None)
                .map_err(|e| {
                    RhiError::ResourceCreationFailed(format!(
                        "Failed to create descriptor pool: {:?}",
                        e
                    ))
                })?
        };

        Ok(Self { pool, max_sets })
    }

    #[cfg(not(feature = "vulkan"))]
    pub fn new(
        _device: &ash::Device,
        _max_sets: u32,
        _pool_sizes: &[(u32, u32)],
    ) -> RhiResult<Self> {
        Err(RhiError::Unsupported(
            "Vulkan feature not enabled".to_string(),
        ))
    }

    /// Free descriptor sets
    #[cfg(feature = "vulkan")]
    pub fn free_sets(&mut self, device: &ash::Device, sets: &[vk::DescriptorSet]) {
        unsafe {
            device.free_descriptor_sets(self.pool, sets);
        }
        self.allocated_sets.retain(|s| !sets.contains(s));
    }

    /// Track allocated descriptor set for cleanup
    #[cfg(feature = "vulkan")]
    pub fn track_allocated_set(&mut self, set: vk::DescriptorSet) {
        self.allocated_sets.push(set);
    }

    /// Free all tracked descriptor sets
    #[cfg(feature = "vulkan")]
    pub fn free_all_sets(&mut self, device: &ash::Device) {
        if !self.allocated_sets.is_empty() {
            unsafe {
                device.free_descriptor_sets(self.pool, &self.allocated_sets);
            }
            self.allocated_sets.clear();
        }
    }
}

impl Drop for VkDescriptorPool {
    #[cfg(feature = "vulkan")]
    fn drop(&mut self) {
        // Pool destruction handled by device
    }

    #[cfg(not(feature = "vulkan"))]
    fn drop(&mut self) {}
}

/// Helper to create standard descriptor set layout
#[cfg(feature = "vulkan")]
pub fn create_descriptor_set_layout(
    device: &ash::Device,
    bindings: &[(u32, vk::DescriptorType, vk::ShaderStageFlags)],
) -> RhiResult<vk::DescriptorSetLayout> {
    use ash::vk;

    let layout_bindings: Vec<vk::DescriptorSetLayoutBinding> = bindings
        .iter()
        .map(|&(binding, ty, stage_flags)| {
            vk::DescriptorSetLayoutBinding::builder()
                .binding(binding)
                .descriptor_type(ty)
                .descriptor_count(1)
                .stage_flags(stage_flags)
                .build()
        })
        .collect();

    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_bindings);

    unsafe {
        device
            .create_descriptor_set_layout(&layout_info, None)
            .map_err(|e| {
                RhiError::ResourceCreationFailed(format!(
                    "Failed to create descriptor set layout: {:?}",
                    e
                ))
            })
    }
}
