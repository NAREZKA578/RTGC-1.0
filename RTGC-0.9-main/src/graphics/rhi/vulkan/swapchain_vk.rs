// Vulkan Backend - SwapChain Implementation
// Implements ISwapChain trait for Vulkan

use crate::graphics::rhi::{types::*, device::{ISwapChain, ISemaphore}};

#[cfg(feature = "vulkan")]
use ash::vk;

/// Vulkan SwapChain implementation
pub struct VkSwapChain {
    #[cfg(feature = "vulkan")]
    surface: vk::SurfaceKHR,

    #[cfg(feature = "vulkan")]
    swapchain: vk::SwapchainKHR,

    #[cfg(feature = "vulkan")]
    images: Vec<vk::Image>,

    #[cfg(feature = "vulkan")]
    image_views: Vec<vk::ImageView>,

    width: u32,
    height: u32,
    format: TextureFormat,
    vsync: bool,
}

unsafe impl Send for VkSwapChain {}
unsafe impl Sync for VkSwapChain {}

impl VkSwapChain {
    /// Create a new Vulkan swapchain
    #[cfg(feature = "vulkan")]
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        width: u32,
        height: u32,
        format: TextureFormat,
        vsync: bool,
    ) -> RhiResult<Self> {
        use ash::vk;

        // Get surface capabilities
        let capabilities = unsafe {
            instance
                .get_physical_device_surface_capabilities(physical_device, surface)
                .map_err(|e| {
                    RhiError::ResourceCreationFailed(format!(
                        "Failed to get surface capabilities: {:?}",
                        e
                    ))
                })?
        };

        // Get surface formats
        let formats = unsafe {
            instance
                .get_physical_device_surface_formats(physical_device, surface)
                .map_err(|e| {
                    RhiError::ResourceCreationFailed(format!(
                        "Failed to get surface formats: {:?}",
                        e
                    ))
                })?
        };

        // Choose swapchain format
        let surface_format = if formats.len() == 1 && formats[0].format == vk::Format::UNDEFINED {
            vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_SRGB,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            }
        } else {
            formats
                .iter()
                .find(|f| {
                    f.format == vk::Format::B8G8R8A8_SRGB
                        && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                })
                .copied()
                .unwrap_or(formats[0])
        };

        // Choose present mode
        let present_modes = unsafe {
            instance
                .get_physical_device_surface_present_modes(physical_device, surface)
                .map_err(|e| {
                    RhiError::ResourceCreationFailed(format!(
                        "Failed to get present modes: {:?}",
                        e
                    ))
                })?
        };

        let present_mode = if vsync {
            present_modes
                .iter()
                .find(|&&mode| mode == vk::PresentModeKHR::FIFO)
                .copied()
                .unwrap_or(vk::PresentModeKHR::FIFO)
        } else {
            present_modes
                .iter()
                .find(|&&mode| mode == vk::PresentModeKHR::MAILBOX)
                .copied()
                .unwrap_or(vk::PresentModeKHR::FIFO)
        };

        // Determine extent
        let extent = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        };

        // Determine image count
        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
            image_count = capabilities.max_image_count;
        }

        // Create swapchain
        let create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: std::ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface,
            min_image_count: image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
            pre_transform: capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: true,
            old_swapchain: vk::SwapchainKHR::null(),
        };

        let swapchain = unsafe {
            device.create_swapchain(&create_info, None).map_err(|e| {
                RhiError::ResourceCreationFailed(format!("Failed to create swapchain: {:?}", e))
            })?
        };

        // Get swapchain images
        let images = unsafe {
            device.get_swapchain_images(swapchain).map_err(|e| {
                RhiError::ResourceCreationFailed(format!("Failed to get swapchain images: {:?}", e))
            })?
        };

        // Create image views
        let mut image_views = Vec::new();
        for &image in &images {
            let view_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::ImageViewCreateFlags::empty(),
                image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: surface_format.format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
            };

            let view = unsafe {
                device.create_image_view(&view_info, None).map_err(|e| {
                    RhiError::ResourceCreationFailed(format!(
                        "Failed to create image view: {:?}",
                        e
                    ))
                })?
            };

            image_views.push(view);
        }

        Ok(Self {
            surface,
            swapchain,
            images,
            image_views,
            width: extent.width,
            height: extent.height,
            format,
            vsync,
        })
    }

    #[cfg(not(feature = "vulkan"))]
    pub fn new(
        _instance: &ash::Instance,
        _device: &ash::Device,
        _physical_device: vk::PhysicalDevice,
        _surface: vk::SurfaceKHR,
        width: u32,
        height: u32,
        format: TextureFormat,
        vsync: bool,
    ) -> RhiResult<Self> {
        Err(RhiError::Unsupported(
            "Vulkan feature not enabled".to_string(),
        ))
    }
}

impl VkSwapChain {
    /// Destroy swapchain resources
    #[cfg(feature = "vulkan")]
    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            for view in self.image_views.drain(..) {
                device.destroy_image_view(view, None);
            }
        }
        self.images.clear();
    }

    /// Recreate swapchain with new dimensions
    #[cfg(feature = "vulkan")]
    pub fn recreate(
        &mut self,
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        width: u32,
        height: u32,
    ) -> RhiResult<()> {
        self.destroy(device);

        // Get surface capabilities
        let capabilities = unsafe {
            instance
                .get_physical_device_surface_capabilities(physical_device, surface)
                .map_err(|e| {
                    RhiError::ResourceCreationFailed(format!(
                        "Failed to get surface capabilities: {:?}",
                        e
                    ))
                })?
        };

        let formats = unsafe {
            instance
                .get_physical_device_surface_formats(physical_device, surface)
                .map_err(|e| {
                    RhiError::ResourceCreationFailed(format!(
                        "Failed to get surface formats: {:?}",
                        e
                    ))
                })?
        };

        let surface_format = if formats.len() == 1 && formats[0].format == vk::Format::UNDEFINED {
            vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_SRGB,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            }
        } else {
            formats
                .iter()
                .find(|f| {
                    f.format == vk::Format::B8G8R8A8_SRGB
                        && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                })
                .copied()
                .unwrap_or(formats[0])
        };

        let present_modes = unsafe {
            instance
                .get_physical_device_surface_present_modes(physical_device, surface)
                .map_err(|e| {
                    RhiError::ResourceCreationFailed(format!(
                        "Failed to get present modes: {:?}",
                        e
                    ))
                })?
        };

        let present_mode = if self.vsync {
            present_modes
                .iter()
                .find(|&&mode| mode == vk::PresentModeKHR::FIFO)
                .copied()
                .unwrap_or(vk::PresentModeKHR::FIFO)
        } else {
            present_modes
                .iter()
                .find(|&&mode| mode == vk::PresentModeKHR::MAILBOX)
                .copied()
                .unwrap_or(vk::PresentModeKHR::FIFO)
        };

        let extent = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        };

        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
            image_count = capabilities.max_image_count;
        }

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain = unsafe {
            device.create_swapchain(&create_info, None).map_err(|e| {
                RhiError::ResourceCreationFailed(format!("Failed to create swapchain: {:?}", e))
            })?
        };

        let images = unsafe {
            device.get_swapchain_images(swapchain).map_err(|e| {
                RhiError::ResourceCreationFailed(format!("Failed to get swapchain images: {:?}", e))
            })?
        };

        let mut image_views = Vec::new();
        for &image in &images {
            let view_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(surface_format.format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            let view = unsafe {
                device.create_image_view(&view_info, None).map_err(|e| {
                    RhiError::ResourceCreationFailed(format!(
                        "Failed to create image view: {:?}",
                        e
                    ))
                })?
            };

            image_views.push(view);
        }

        self.swapchain = swapchain;
        self.images = images;
        self.image_views = image_views;
        self.width = extent.width;
        self.height = extent.height;

        Ok(())
    }
}

impl Drop for VkSwapChain {
    #[cfg(feature = "vulkan")]
    fn drop(&mut self) {
        // Note: Device must be valid - in production, use a proper destruction queue
        // This is a stub - actual cleanup requires device reference
    }

    #[cfg(not(feature = "vulkan"))]
    fn drop(&mut self) {}
}

impl ISwapChain for VkSwapChain {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn get_current_back_buffer_index(&self) -> usize {
        0
    }

    fn get_back_buffer(&self) -> ResourceHandle {
        ResourceHandle::default()
    }

    fn get_back_buffer_texture(&self) -> ResourceHandle {
        ResourceHandle::default()
    }

    fn present(&self) -> RhiResult<()> {
        #[cfg(feature = "vulkan")]
        {
            Ok(())
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn resize(&self, width: u32, height: u32) -> RhiResult<()> {
        #[cfg(feature = "vulkan")]
        {
            let _ = (width, height);
            Ok(())
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn present_with_sync(&self, _semaphore: Option<&dyn ISemaphore>) -> RhiResult<()> {
        self.present()
    }
}
