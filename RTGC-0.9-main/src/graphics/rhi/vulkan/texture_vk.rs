// Vulkan Backend - Texture Implementation
// Implements GPU texture resources for Vulkan

use crate::graphics::rhi::types::*;

#[cfg(feature = "vulkan")]
use ash::vk;

/// Vulkan Texture resource
pub struct VkTexture {
    #[cfg(feature = "vulkan")]
    image: vk::Image,
    
    #[cfg(feature = "vulkan")]
    allocation: Option<vk::DeviceMemory>,
    
    #[cfg(feature = "vulkan")]
    view: Option<vk::ImageView>,
    
    handle: ResourceHandle,
    description: TextureDescription,
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    mip_levels: u32,
}

unsafe impl Send for VkTexture {}
unsafe impl Sync for VkTexture {}

impl VkTexture {
    /// Create a new Vulkan texture
    #[cfg(feature = "vulkan")]
    pub fn new(
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        desc: &TextureDescription,
        handle: ResourceHandle,
    ) -> RhiResult<Self> {
        use ash::vk;
        
        let image_type = Self::to_vk_image_type(desc.dimension);
        let format = Self::to_vk_format(desc.format);
        let usage = Self::to_vk_usage(desc.usage);
        
        let (extent, array_layers) = match desc.dimension {
            TextureDimension::D1 => (vk::Extent3D { width: desc.width, height: 1, depth: 1 }, 1),
            TextureDimension::D2 => (vk::Extent3D { width: desc.width, height: desc.height, depth: 1 }, desc.depth_or_array_layers),
            TextureDimension::D3 => (vk::Extent3D { width: desc.width, height: desc.height, depth: desc.depth_or_array_layers }, 1),
            TextureDimension::Cube => (vk::Extent3D { width: desc.width, height: desc.height, depth: 1 }, desc.depth_or_array_layers * 6),
        };
        
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(image_type)
            .format(format)
            .extent(extent)
            .mip_levels(desc.mip_levels)
            .array_layers(array_layers)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        
        let image = unsafe {
            device.create_image(&image_info, None)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create image: {:?}", e)))?
        };
        
        // Allocate and bind memory for the image
        let mem_requirements = unsafe { device.get_image_memory_requirements(image) };
        
        // Find a suitable memory type
        let mem_type_index = Self::find_memory_type(
            physical_device,
            device,
            mem_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        
        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type_index);
        
        let allocation = unsafe {
            device.allocate_memory(&alloc_info, None)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to allocate memory: {:?}", e)))?
        };
        
        unsafe {
            device.bind_image_memory(image, allocation, 0)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to bind image memory: {:?}", e)))?;
        }
        
        // Create image view
        let view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(Self::to_vk_image_view_type(desc.dimension))
            .format(format)
            .components(vk::ComponentMapping::default())
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: Self::get_aspect_mask(desc.format),
                base_mip_level: 0,
                level_count: desc.mip_levels,
                base_array_layer: 0,
                layer_count: array_layers,
            });
        
        let view = unsafe {
            Some(device.create_image_view(&view_info, None)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create image view: {:?}", e)))?)
        };
        
        Ok(Self {
            image,
            allocation: Some(allocation),
            view,
            handle,
            description: desc.clone(),
            width: desc.width,
            height: desc.height,
            depth_or_array_layers: desc.depth_or_array_layers,
            mip_levels: desc.mip_levels,
        })
    }
    
    #[cfg(feature = "vulkan")]
    fn find_memory_type(
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> RhiResult<u32> {
        let mem_properties = unsafe { device.get_physical_device_memory_properties(physical_device) };
        
        for i in 0..mem_properties.memory_type_count {
            if (type_filter & (1 << i)) != 0
                && mem_properties.memory_types[i as usize].property_flags.contains(properties)
            {
                return Ok(i);
            }
        }
        
        Err(RhiError::ResourceCreationFailed("Failed to find suitable memory type".to_string()))
    }
    
    #[cfg(feature = "vulkan")]
    fn to_vk_image_view_type(dimension: TextureDimension) -> vk::ImageViewType {
        match dimension {
            TextureDimension::D1 => vk::ImageViewType::TYPE_1D,
            TextureDimension::D2 => vk::ImageViewType::TYPE_2D,
            TextureDimension::D3 => vk::ImageViewType::TYPE_3D,
            TextureDimension::Cube => vk::ImageViewType::CUBE,
        }
    }
    
    #[cfg(feature = "vulkan")]
    fn get_aspect_mask(format: TextureFormat) -> vk::ImageAspectFlags {
        match format {
            TextureFormat::D16Unorm | TextureFormat::D24UnormS8Uint | 
            TextureFormat::D32Float | TextureFormat::D32FloatS8UintX24 => {
                if matches!(format, TextureFormat::D24UnormS8Uint | TextureFormat::D32FloatS8UintX24) {
                    vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
                } else {
                    vk::ImageAspectFlags::DEPTH
                }
            }
            _ => vk::ImageAspectFlags::COLOR,
        }
    }
    
    #[cfg(feature = "vulkan")]
    fn to_vk_image_type(dimension: TextureDimension) -> vk::ImageType {
        match dimension {
            TextureDimension::D1 => vk::ImageType::TYPE_1D,
            TextureDimension::D2 | TextureDimension::Cube => vk::ImageType::TYPE_2D,
            TextureDimension::D3 => vk::ImageType::TYPE_3D,
        }
    }
    
    #[cfg(feature = "vulkan")]
    pub fn to_vk_format(format: TextureFormat) -> vk::Format {
        match format {
            TextureFormat::R8Unorm => vk::Format::R8_UNORM,
            TextureFormat::R8G8Unorm => vk::Format::R8G8_UNORM,
            TextureFormat::R8G8B8A8Unorm => vk::Format::R8G8B8A8_UNORM,
            TextureFormat::R8G8B8A8Srgb => vk::Format::R8G8B8A8_SRGB,
            TextureFormat::R16Float => vk::Format::R16_SFLOAT,
            TextureFormat::R16G16Float => vk::Format::R16G16_SFLOAT,
            TextureFormat::R16G16B16A16Float => vk::Format::R16G16B16A16_SFLOAT,
            TextureFormat::R32Float => vk::Format::R32_SFLOAT,
            TextureFormat::R32G32Float => vk::Format::R32G32_SFLOAT,
            TextureFormat::R32G32B32A32Float => vk::Format::R32G32B32A32_SFLOAT,
            TextureFormat::D16Unorm => vk::Format::D16_UNORM,
            TextureFormat::D24UnormS8Uint => vk::Format::D24_UNORM_S8_UINT,
            TextureFormat::D32Float => vk::Format::D32_SFLOAT,
            TextureFormat::D32FloatS8UintX24 => vk::Format::D32_SFLOAT_S8_UINT,
            TextureFormat::BC1RgbUnorm => vk::Format::BC1_RGB_UNORM_BLOCK,
            TextureFormat::BC1RgbaUnorm => vk::Format::BC1_RGBA_UNORM_BLOCK,
            TextureFormat::BC2Unorm => vk::Format::BC2_UNORM_BLOCK,
            TextureFormat::BC3Unorm => vk::Format::BC3_UNORM_BLOCK,
            TextureFormat::BC4Unorm => vk::Format::BC4_UNORM_BLOCK,
            TextureFormat::BC5Unorm => vk::Format::BC5_UNORM_BLOCK,
            TextureFormat::BC6HUfloat => vk::Format::BC6H_UFLOAT_BLOCK,
            TextureFormat::BC7Unorm => vk::Format::BC7_UNORM_BLOCK,
            _ => vk::Format::UNDEFINED,
        }
    }
    
    #[cfg(feature = "vulkan")]
    fn to_vk_usage(usage: TextureUsage) -> vk::ImageUsageFlags {
        let mut flags = vk::ImageUsageFlags::empty();
        
        if usage.contains(TextureUsage::SHADER_READ) {
            flags |= vk::ImageUsageFlags::SAMPLED;
        }
        if usage.contains(TextureUsage::SHADER_WRITE) {
            flags |= vk::ImageUsageFlags::STORAGE;
        }
        if usage.contains(TextureUsage::RENDER_TARGET) {
            flags |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
        }
        if usage.contains(TextureUsage::DEPTH_STENCIL) {
            flags |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
        }
        if usage.contains(TextureUsage::TRANSFER_SRC) {
            flags |= vk::ImageUsageFlags::TRANSFER_SRC;
        }
        if usage.contains(TextureUsage::TRANSFER_DST) {
            flags |= vk::ImageUsageFlags::TRANSFER_DST;
        }
        if usage.contains(TextureUsage::PRESENT) {
            flags |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
        }
        
        flags
    }
    
    #[cfg(not(feature = "vulkan"))]
    pub fn new(
        _device: &ash::Device,
        _physical_device: vk::PhysicalDevice,
        _desc: &TextureDescription,
        _handle: ResourceHandle,
    ) -> RhiResult<Self> {
        Err(RhiError::Unsupported("Vulkan feature not enabled".to_string()))
    }
    
    #[cfg(feature = "vulkan")]
    pub fn image(&self) -> vk::Image {
        self.image
    }
    
    pub fn handle(&self) -> ResourceHandle {
        self.handle
    }
    
    pub fn description(&self) -> &TextureDescription {
        &self.description
    }
    
    pub fn width(&self) -> u32 {
        self.width
    }
    
    pub fn height(&self) -> u32 {
        self.height
    }
}
