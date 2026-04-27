// Vulkan Backend - Device Implementation
// Implements IDevice trait for Vulkan
// Логирование: target = "vulkan"

use super::texture_vk::VkTexture;
use super::buffer_vk::VkBuffer;
use super::command_vk::{VkCommandList, VkCommandQueue};
use super::fence_vk::{VkFence, VkSemaphore};
use super::swapchain_vk::VkSwapChain;
use crate::graphics::rhi::{
    device::*,
    resource_manager::{BufferHandle, ManagedResource, PipelineHandle, ResourceManager, SamplerHandle, ShaderHandle, TextureHandle},
    types::*,
};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Helper macro to create C strings at compile time (replacement for cstr! macro)
macro_rules! cstr {
    ($s:literal) => {
        unsafe { CStr::from_bytes_with_nul_unchecked(concat!($s, "\0").as_bytes()) }
    };
}

/// Vulkan resource types for tracking
#[derive(Clone)]
enum VkResource {
    Buffer(BufferHandle),
    Texture(TextureHandle),
    Sampler(SamplerHandle),
    Pipeline(PipelineHandle),
    Shader(ShaderHandle),
}

/// Vulkan Device implementation
pub struct VkDevice {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub physical_device: ash::vk::PhysicalDevice,
    pub device: ash::Device,
    pub graphics_queue: ash::vk::Queue,
    pub compute_queue: ash::vk::Queue,
    pub transfer_queue: ash::vk::Queue,
    pub queue_family_index: u32,
    pub compute_queue_family_index: u32,
    pub transfer_queue_family_index: u32,
    pub features: DeviceFeatures,
    pub limits: DeviceLimits,
    pub name: String,
    resource_manager: Arc<Mutex<ResourceManager>>,
}

unsafe impl Send for VkDevice {}
unsafe impl Sync for VkDevice {}

impl VkDevice {
    /// Create a new Vulkan device with specified backend features
    pub fn new(enable_validation: bool) -> RhiResult<Self> {
        #[cfg(feature = "vulkan")]
        {
            use ash::vk;

            // Load Vulkan entry points
            let entry = unsafe { ash::Entry::load() }.map_err(|e| {
                RhiError::InitializationFailed(format!("Failed to load Vulkan library: {}", e))
            })?;

            // Create Vulkan instance with VK 1.3 support
            let app_info = vk::ApplicationInfo {
                p_application_name: cstr!("RTGC Engine").as_ptr(),
                application_version: vk::make_api_version(0, 1, 0, 0),
                p_engine_name: cstr!("RTGC").as_ptr(),
                engine_version: vk::make_api_version(0, 0, 7, 0),
                api_version: vk::API_VERSION_1_3,
                ..Default::default()
            };

            let mut enabled_layers: Vec<*const i8> = Vec::new();
            let mut enabled_extensions: Vec<*const i8> = vec![b"VK_KHR_surface\0".as_ptr() as *const i8];

            #[cfg(target_os = "windows")]
            enabled_extensions.push(b"VK_KHR_win32_surface\0".as_ptr() as *const i8);

            #[cfg(target_os = "linux")]
            enabled_extensions.push(b"VK_KHR_xlib_surface\0".as_ptr() as *const i8);

            #[cfg(target_os = "macos")]
            enabled_extensions.push(b"VK_MVK_macos_surface\0".as_ptr() as *const i8);

            if enable_validation {
                enabled_layers.push(cstr!("VK_LAYER_KHRONOS_validation").as_ptr());
                enabled_extensions.push(b"VK_EXT_debug_utils\0".as_ptr() as *const i8);
            }

            let create_info = vk::InstanceCreateInfo {
                p_application_info: &app_info,
                enabled_layer_count: enabled_layers.len() as u32,
                pp_enabled_layer_names: enabled_layers.as_ptr(),
                enabled_extension_count: enabled_extensions.len() as u32,
                pp_enabled_extension_names: enabled_extensions.as_ptr(),
                ..Default::default()
            };

            let instance = unsafe { entry.create_instance(&create_info, None) }.map_err(|e| {
                RhiError::InitializationFailed(format!("Failed to create Vulkan instance: {:?}", e))
            })?;

            // Find physical device with GPU occlusion query support
            let physical_devices =
                unsafe { instance.enumerate_physical_devices() }.map_err(|e| {
                    RhiError::InitializationFailed(format!(
                        "Failed to enumerate physical devices: {}",
                        e
                    ))
                })?;

            let physical_device = physical_devices
                .into_iter()
                .find(|&device| Self::is_suitable_device(&instance, device))
                .ok_or_else(|| {
                    RhiError::InitializationFailed("No suitable Vulkan device found".to_string())
                })?;

            // Get queue family indices for graphics, compute, and transfer
            let (graphics_queue_family, compute_queue_family, transfer_queue_family) =
                Self::find_queue_families(&instance, physical_device);

            // Create logical device with multiple queue types
            let priorities = [1.0f32];
            let mut queue_create_infos = Vec::new();

            queue_create_infos.push(vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: graphics_queue_family,
                queue_priorities: &priorities,
            });

            if compute_queue_family != graphics_queue_family {
                queue_create_infos.push(vk::DeviceQueueCreateInfo {
                    s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                    p_next: std::ptr::null(),
                    flags: vk::DeviceQueueCreateFlags::empty(),
                    queue_family_index: compute_queue_family,
                    queue_priorities: &priorities,
                });
            }

            if transfer_queue_family != graphics_queue_family
                && transfer_queue_family != compute_queue_family
            {
                queue_create_infos.push(vk::DeviceQueueCreateInfo {
                    s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                    p_next: std::ptr::null(),
                    flags: vk::DeviceQueueCreateFlags::empty(),
                    queue_family_index: transfer_queue_family,
                    queue_priorities: &priorities,
                });
            }

            // Enable Vulkan 1.3 features for advanced rendering
            let mut vulkan_13_features = vk::PhysicalDeviceVulkan13Features {
                s_type: vk::StructureType::PHYSICAL_DEVICE_VULKAN_1_3_FEATURES,
                p_next: std::ptr::null_mut(),
                synchronization2: true,
                dynamic_rendering: true,
                maintenance4: true,
                ..Default::default()
            };

            let enabled_features = vk::PhysicalDeviceFeatures2 {
                s_type: vk::StructureType::PHYSICAL_DEVICE_FEATURES_2,
                p_next: &mut vulkan_13_features as *mut _ as *mut std::ffi::c_void,
                features: vk::PhysicalDeviceFeatures {
                    fill_mode_non_solid: true,
                    multi_draw_indirect: true,
                    draw_indirect_first_instance: true,
                    depth_bounds: true,
                    occlusion_query_precise: true,
                    pipeline_statistics_query: true,
                    sample_rate_shading: true,
                    dual_src_blend: true,
                    independent_blend: true,
                    geometry_shader: true,
                    tessellation_shader: true,
                    shader_storage_image_extended_formats: true,
                    ..Default::default()
                },
            };

            let mut enabled_extensions = vec![
                "VK_KHR_swapchain".as_ptr() as *const i8,
                "VK_EXT_extended_dynamic_state".as_ptr() as *const i8,
            ];

            enabled_extensions
                .push("VK_KHR_get_physical_device_properties2".as_ptr() as *const i8);

            let device_info = vk::DeviceCreateInfo {
                s_type: vk::StructureType::DEVICE_CREATE_INFO,
                p_next: &mut enabled_features as *mut _ as *mut std::ffi::c_void,
                flags: vk::DeviceCreateFlags::empty(),
                queue_create_info_count: queue_create_infos.len() as u32,
                p_queue_create_infos: queue_create_infos.as_ptr(),
                enabled_extension_count: enabled_extensions.len() as u32,
                pp_enabled_extension_names: enabled_extensions.as_ptr(),
                p_enabled_features: std::ptr::null(),
            };

            let device = unsafe { instance.create_device(physical_device, &device_info, None) }
                .map_err(|e| {
                    RhiError::InitializationFailed(format!(
                        "Failed to create logical device: {}",
                        e
                    ))
                })?;

            // Get queues
            let graphics_queue = unsafe { device.get_device_queue(graphics_queue_family, 0) };
            let compute_queue = unsafe { device.get_device_queue(compute_queue_family, 0) };
            let transfer_queue = unsafe { device.get_device_queue(transfer_queue_family, 0) };

            // Query device properties
            let device_properties =
                unsafe { instance.get_physical_device_properties(physical_device) };
            let name = String::from_utf8_lossy(&device_properties.device_name[..])
                .trim_end_matches('\0')
                .to_string();

            Ok(Self {
                entry,
                instance,
                physical_device,
                device,
                graphics_queue,
                compute_queue,
                transfer_queue,
                queue_family_index: graphics_queue_family,
                compute_queue_family_index: compute_queue_family,
                transfer_queue_family_index: transfer_queue_family,
                features: Self::query_features(),
                limits: Self::query_limits(&instance, physical_device),
                name,
                resource_manager: Arc::new(Mutex::new(ResourceManager::new())),
            })
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    #[cfg(feature = "vulkan")]
    fn find_queue_families(
        instance: &ash::Instance,
        physical_device: ash::vk::PhysicalDevice,
    ) -> (u32, u32, u32) {
        use ash::vk;

        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut graphics_family = None;
        let mut compute_family = None;
        let mut transfer_family = None;

        for (i, family) in queue_families.iter().enumerate() {
            let flags = family.queue_flags;
            let index = i as u32;

            // Graphics queue
            if flags.contains(vk::QueueFlags::GRAPHICS) && graphics_family.is_none() {
                graphics_family = Some(index);
            }

            // Compute queue (prefer dedicated)
            if flags.contains(vk::QueueFlags::COMPUTE)
                && !flags.contains(vk::QueueFlags::GRAPHICS)
                && compute_family.is_none()
            {
                compute_family = Some(index);
            }

            // Transfer queue (prefer dedicated)
            if flags.contains(vk::QueueFlags::TRANSFER)
                && !flags.contains(vk::QueueFlags::GRAPHICS)
                && !flags.contains(vk::QueueFlags::COMPUTE)
                && transfer_family.is_none()
            {
                transfer_family = Some(index);
            }
        }

        // Fallback to graphics queue if dedicated queues not found
        let graphics = graphics_family.unwrap_or(0);
        let compute = compute_family.unwrap_or(graphics);
        let transfer = transfer_family.unwrap_or(graphics);

        (graphics, compute, transfer)
    }

    #[cfg(feature = "vulkan")]
    fn is_suitable_device(instance: &ash::Instance, device: ash::vk::PhysicalDevice) -> bool {
        use ash::vk;

        let props = unsafe { instance.get_physical_device_properties(device) };
        let features = unsafe { instance.get_physical_device_features(device) };

        // Check if discrete GPU or integrated GPU
        let is_discrete = props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU;
        let is_integrated = props.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU;

        // Must support geometry shaders and have minimum required features including occlusion query
        let has_required_features = features.geometry_shader == vk::TRUE
            && features.multi_draw_indirect == vk::TRUE
            && features.fill_mode_non_solid == vk::TRUE
            && features.occlusion_query_precise == vk::TRUE;

        (is_discrete || is_integrated) && has_required_features
    }

    #[cfg(not(feature = "vulkan"))]
    fn is_suitable_device(_instance: &ash::Instance, _device: ash::vk::PhysicalDevice) -> bool {
        false
    }

    fn query_features() -> DeviceFeatures {
        DeviceFeatures {
            anisotropic_filtering: true,
            bc_compression: true,
            compute_shaders: true,
            geometry_shaders: true,
            tessellation: true,
            conservative_rasterization: false, // Optional in Vulkan
            multi_draw_indirect: true,
            draw_indirect_first_instance: true,
            dual_source_blending: true,
            depth_bounds_test: true,
            sample_rate_shading: true,
            texture_cube_map_array: true,
            texture_3d_as_2d_array: true,
            independent_blend: true,
            logic_op: true,
            occlusion_query: true,
            timestamp_query: true,
            pipeline_statistics_query: true,
            stream_output: false,         // Not in Vulkan
            variable_rate_shading: false, // Optional
            mesh_shaders: false,          // Optional (Vulkan 1.3+)
            ray_tracing: false,           // Optional extension
            sampler_lod_bias: true,
            border_color_clamp: true,
        }
    }

    #[cfg(feature = "vulkan")]
    fn query_limits(
        instance: &ash::Instance,
        physical_device: ash::vk::PhysicalDevice,
    ) -> DeviceLimits {
        use ash::vk;

        let props = unsafe { instance.get_physical_device_properties(physical_device) };
        let limits = props.limits;

        DeviceLimits {
            max_texture_dimension_1d: limits.max_image_dimension1_d,
            max_texture_dimension_2d: limits.max_image_dimension2_d,
            max_texture_dimension_3d: limits.max_image_dimension3_d,
            max_texture_array_layers: limits.max_image_array_layers,
            max_buffer_size: limits.max_storage_buffer_range as u64,
            max_vertex_input_attributes: limits.max_vertex_input_attributes,
            max_vertex_input_bindings: limits.max_vertex_input_bindings,
            max_vertex_input_attribute_offset: limits.max_vertex_input_attribute_offset,
            max_vertex_input_binding_stride: limits.max_vertex_input_binding_stride,
            max_vertex_output_components: limits.max_vertex_output_components,
            max_fragment_input_components: limits.max_fragment_input_components,
            max_fragment_output_attachments: limits.max_fragment_output_attachments,
            max_compute_work_group_count: limits.max_compute_work_group_count,
            max_compute_work_group_invocations: limits.max_compute_work_group_invocations,
            max_compute_shared_memory_size: limits.max_compute_shared_memory_size,
            max_uniform_buffer_range: limits.max_uniform_buffer_range,
            max_storage_buffer_range: limits.max_storage_buffer_range,
            max_sampler_anisotropy: limits.max_sampler_anisotropy as f32,
            min_texel_buffer_offset_alignment: limits.min_texel_buffer_offset_alignment,
            min_uniform_buffer_offset_alignment: limits.min_uniform_buffer_offset_alignment,
            min_storage_buffer_offset_alignment: limits.min_storage_buffer_offset_alignment,
            max_descriptor_set_samplers: limits.max_descriptor_set_samplers,
            max_descriptor_set_uniform_buffers: limits.max_descriptor_set_uniform_buffers,
            max_descriptor_set_storage_buffers: limits.max_descriptor_set_storage_buffers,
            max_descriptor_set_textures: limits.max_descriptor_set_sampled_images,
            max_descriptor_set_storage_images: limits.max_descriptor_set_storage_images,
            max_per_stage_descriptor_samplers: limits.max_per_stage_descriptor_samplers,
            max_per_stage_descriptor_uniform_buffers: limits
                .max_per_stage_descriptor_uniform_buffers,
            max_per_stage_descriptor_storage_buffers: limits
                .max_per_stage_descriptor_storage_buffers,
            max_per_stage_descriptor_textures: limits.max_per_stage_descriptor_sampled_images,
            max_per_stage_descriptor_storage_images: limits.max_per_stage_descriptor_storage_images,
        }
    }

    #[cfg(not(feature = "vulkan"))]
    fn query_limits(
        _instance: &ash::Instance,
        _physical_device: ash::vk::PhysicalDevice,
    ) -> DeviceLimits {
        DeviceLimits::default()
    }

    #[cfg(feature = "vulkan")]
    fn to_vk_address_mode(address: AddressMode) -> ash::vk::SamplerAddressMode {
        use ash::vk;
        match address {
            AddressMode::Wrap => vk::SamplerAddressMode::REPEAT,
            AddressMode::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            AddressMode::Border => vk::SamplerAddressMode::CLAMP_TO_BORDER,
            AddressMode::Mirror => vk::SamplerAddressMode::MIRRORED_REPEAT,
            AddressMode::MirrorOnce => vk::SamplerAddressMode::MIRROR_CLAMP_TO_EDGE,
        }
    }
}

impl IDevice for VkDevice {
    fn get_device_name(&self) -> &str {
        &self.name
    }

    fn get_features(&self) -> DeviceFeatures {
        self.features.clone()
    }

    fn get_limits(&self) -> DeviceLimits {
        self.limits.clone()
    }

    fn create_buffer(&self, desc: &BufferDescription) -> RhiResult<ResourceHandle> {
        #[cfg(feature = "vulkan")]
        {
            use ash::vk;

            let buffer = VkBuffer::new(&self.device, self.physical_device, desc)?;
            let handle = self.resource_manager.lock().register_buffer(BufferHandle {
                handle: ResourceHandle(0),
                size: desc.size,
                buffer_type: desc.buffer_type,
                state: ResourceState::Common,
                dx12_resource: None,
                vulkan_buffer: Some(buffer.buffer().as_raw() as u64),
                vulkan_allocation: None,
                gl_buffer: None,
            });

            Ok(handle)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn create_texture(&self, desc: &TextureDescription) -> RhiResult<ResourceHandle> {
        #[cfg(feature = "vulkan")]
        {
            let texture = VkTexture::new(&self.device, self.physical_device, desc)?;
            let handle = self.resource_manager.lock().register_texture(TextureHandle {
                handle: ResourceHandle(0),
                desc: desc.clone(),
                dx12_resource: None,
                dx12_srv_handle: None,
                dx12_rtv_handle: None,
                dx12_dsv_handle: None,
                vulkan_image: Some(texture.image().as_raw() as u64),
                vulkan_view: texture.view().map(|v| v.as_raw() as u64),
                gl_texture: None,
                gl_target: None,
                gl_framebuffer: None,
            });

            Ok(handle)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn create_texture_view(
        &self,
        texture_handle: ResourceHandle,
        desc: &TextureViewDescription,
    ) -> RhiResult<ResourceHandle> {
        #[cfg(feature = "vulkan")]
        {
            use ash::vk;

            // Look up texture from resource manager
            let texture_data = self
                .resource_manager
                .lock()
                .get_texture(texture_handle)
                .ok_or_else(|| RhiError::ResourceCreationFailed("Texture not found".to_string()))?;

            // Get the Vulkan image from the stored handle
            let image_raw = texture_data.vulkan_image.ok_or_else(|| {
                RhiError::ResourceCreationFailed("Texture has no Vulkan image".to_string())
            })?;
            let image = unsafe { vk::Image::from_raw(image_raw) };

            // Create image view based on description
            let format = VkTexture::to_vk_format(desc.format);
            let view_type = match desc.dimension {
                TextureDimension::D1 => vk::ImageViewType::TYPE_1D,
                TextureDimension::D2 => vk::ImageViewType::TYPE_2D,
                TextureDimension::D3 => vk::ImageViewType::TYPE_3D,
                TextureDimension::Cube => vk::ImageViewType::CUBE,
            };

            let aspect_mask = match desc.format {
                TextureFormat::D16Unorm | TextureFormat::D32Float => vk::ImageAspectFlags::DEPTH,
                TextureFormat::D24UnormS8Uint | TextureFormat::D32FloatS8UintX24 => {
                    vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
                }
                _ => vk::ImageAspectFlags::COLOR,
            };

            let view_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(view_type)
                .format(format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask,
                    base_mip_level: desc.base_mip_level,
                    level_count: desc.mip_level_count,
                    base_array_layer: desc.base_array_layer,
                    layer_count: desc.array_layer_count,
                });

            let view = unsafe {
                self.device
                    .create_image_view(&view_info, None)
                    .map_err(|e| {
                        RhiError::ResourceCreationFailed(format!(
                            "Failed to create image view: {:?}",
                            e
                        ))
                    })?
            };

            // Create handle for the view
            let handle = ResourceHandle::new();

            // Store in resource manager (we can reuse texture handle structure or create a separate one)
            let mut texture_handle_data = texture_data.clone();
            texture_handle_data.handle = handle;
            texture_handle_data.vulkan_view = Some(view.as_raw() as u64);

            Ok(handle)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn create_sampler(&self, desc: &SamplerDescription) -> RhiResult<ResourceHandle> {
        #[cfg(feature = "vulkan")]
        {
            use ash::vk;

            let handle = ResourceHandle::new();

            let mag_filter = match desc.mag_filter {
                FilterMode::Point => vk::Filter::NEAREST,
                FilterMode::Linear => vk::Filter::LINEAR,
                FilterMode::Anisotropic => vk::Filter::LINEAR,
            };

            let min_filter = match desc.min_filter {
                FilterMode::Point => vk::Filter::NEAREST,
                FilterMode::Linear => vk::Filter::LINEAR,
                FilterMode::Anisotropic => vk::Filter::LINEAR,
            };

            let mipmap_mode = match desc.mip_filter {
                FilterMode::Point => vk::SamplerMipmapMode::NEAREST,
                _ => vk::SamplerMipmapMode::LINEAR,
            };

            let address_mode_u = Self::to_vk_address_mode(desc.address_u);
            let address_mode_v = Self::to_vk_address_mode(desc.address_v);
            let address_mode_w = Self::to_vk_address_mode(desc.address_w);

            let sampler_info = vk::SamplerCreateInfo::builder()
                .mag_filter(mag_filter)
                .min_filter(min_filter)
                .mipmap_mode(mipmap_mode)
                .address_mode_u(address_mode_u)
                .address_mode_v(address_mode_v)
                .address_mode_w(address_mode_w)
                .mip_lod_bias(desc.mip_lod_bias)
                .anisotropy_enable(desc.anisotropic_filtering)
                .max_anisotropy(desc.max_anisotropy)
                .compare_enable(false)
                .min_lod(desc.min_lod)
                .max_lod(desc.max_lod)
                .border_color(vk::BorderColor::INT_OPAQUE_BLACK);

            let sampler = unsafe {
                self.device
                    .create_sampler(&sampler_info, None)
                    .map_err(|e| {
                        RhiError::ResourceCreationFailed(format!(
                            "Failed to create sampler: {:?}",
                            e
                        ))
                    })?
            };

            Ok(handle)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn create_shader(&self, desc: &ShaderDescription) -> RhiResult<ResourceHandle> {
        #[cfg(feature = "vulkan")]
        {
            use ash::vk;

            let handle = ResourceHandle::new();

            let shader_info = vk::ShaderModuleCreateInfo::builder().code(desc.code);

            let shader_module = unsafe {
                self.device
                    .create_shader_module(&shader_info, None)
                    .map_err(|e| {
                        RhiError::ResourceCreationFailed(format!(
                            "Failed to create shader module: {:?}",
                            e
                        ))
                    })?
            };

            Ok(handle)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn create_pipeline_state(&self, desc: &PipelineStateObject) -> RhiResult<ResourceHandle> {
        #[cfg(feature = "vulkan")]
        {
            use ash::vk;

            let handle = ResourceHandle::new();

            // Note: Pipeline creation requires render pass which is not available in IDevice trait
            // This method should be called through a higher-level interface that provides render pass
            // For now, we return an error indicating that direct pipeline creation is not supported
            return Err(RhiError::Unsupported(
                "Pipeline creation requires render pass. Use Renderer or GraphicsContext to create pipelines.".to_string()
            ));
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn create_input_layout(&self, desc: &InputLayout) -> RhiResult<ResourceHandle> {
        #[cfg(feature = "vulkan")]
        {
            let handle = ResourceHandle::new();
            Ok(handle)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn create_descriptor_heap(
        &self,
        desc: &DescriptorHeapDescription,
    ) -> RhiResult<ResourceHandle> {
        #[cfg(feature = "vulkan")]
        {
            use ash::vk;

            let handle = ResourceHandle::new();

            // In Vulkan, we create descriptor pools and sets instead of heaps
            let mut pool_sizes = Vec::new();

            if desc.num_samplers > 0 {
                pool_sizes.push(vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::SAMPLER,
                    descriptor_count: desc.num_samplers,
                });
            }

            if desc.num_uniform_buffers > 0 {
                pool_sizes.push(vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: desc.num_uniform_buffers,
                });
            }

            if desc.num_storage_buffers > 0 {
                pool_sizes.push(vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: desc.num_storage_buffers,
                });
            }

            if desc.num_textures > 0 {
                pool_sizes.push(vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::SAMPLED_IMAGE,
                    descriptor_count: desc.num_textures,
                });
            }

            if desc.num_storage_textures > 0 {
                pool_sizes.push(vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_IMAGE,
                    descriptor_count: desc.num_storage_textures,
                });
            }

            let total_descriptors = desc.num_samplers
                + desc.num_uniform_buffers
                + desc.num_storage_buffers
                + desc.num_textures
                + desc.num_storage_textures;

            let pool_info = vk::DescriptorPoolCreateInfo::builder()
                .pool_sizes(&pool_sizes)
                .max_sets(total_descriptors)
                .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);

            let descriptor_pool = unsafe {
                self.device
                    .create_descriptor_pool(&pool_info, None)
                    .map_err(|e| {
                        RhiError::ResourceCreationFailed(format!(
                            "Failed to create descriptor pool: {:?}",
                            e
                        ))
                    })?
            };

            Ok(handle)
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn create_command_list(&self, cmd_type: CommandListType) -> RhiResult<Box<dyn ICommandList + Send + Sync>> {
        #[cfg(feature = "vulkan")]
        {
            let cmd_list = VkCommandList::new(&self.device, self.queue_family_index, cmd_type)?;
            Ok(Box::new(cmd_list))
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn create_command_queue(&self, cmd_type: CommandListType) -> RhiResult<Arc<dyn ICommandQueue>> {
        #[cfg(feature = "vulkan")]
        {
            let queue = unsafe { self.device.get_device_queue(self.queue_family_index, 0) };
            let cmd_queue = VkCommandQueue::new(queue, cmd_type);
            Ok(Arc::new(cmd_queue))
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn create_fence(&self, initial_value: u64) -> RhiResult<Arc<dyn IFence>> {
        #[cfg(feature = "vulkan")]
        {
            use ash::vk;

            let fence_info = vk::FenceCreateInfo::builder().flags(if initial_value > 0 {
                vk::FenceCreateFlags::SIGNALED
            } else {
                vk::FenceCreateFlags::empty()
            });

            let fence = unsafe {
                self.device.create_fence(&fence_info, None).map_err(|e| {
                    RhiError::ResourceCreationFailed(format!("Failed to create fence: {:?}", e))
                })?
            };

            let vk_fence = VkFence::new(fence);
            Ok(Arc::new(vk_fence))
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn create_semaphore(&self) -> RhiResult<Arc<dyn ISemaphore>> {
        #[cfg(feature = "vulkan")]
        {
            use ash::vk;

            let semaphore_info = vk::SemaphoreCreateInfo::builder();

            let semaphore = unsafe {
                self.device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| {
                        RhiError::ResourceCreationFailed(format!(
                            "Failed to create semaphore: {:?}",
                            e
                        ))
                    })?
            };

            let vk_semaphore = VkSemaphore::new(semaphore);
            Ok(Arc::new(vk_semaphore))
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn create_swap_chain(
        &self,
        _window_handle: *mut std::ffi::c_void,
        width: u32,
        height: u32,
        format: TextureFormat,
        vsync: bool,
    ) -> RhiResult<Arc<dyn ISwapChain>> {
        #[cfg(feature = "vulkan")]
        {
            // Create surface based on platform
            #[cfg(target_os = "windows")]
            let surface = {
                use ash::vk;
                // Stub - actual Win32Surface creation requires platform-specific handling
                // In ash 0.38+, use the new surface creation API
                tracing::warn!("Win32Surface not available - using stub surface");
                vk::SurfaceKHR::null()
            };

            #[cfg(target_os = "linux")]
            let surface = {
                tracing::warn!("Linux surface not available - using stub surface");
                ash::vk::SurfaceKHR::null()
            };

            let swapchain = VkSwapChain::new(
                &self.instance,
                &self.device,
                self.physical_device,
                surface,
                width,
                height,
                format,
                vsync,
            )?;

            Ok(Arc::new(swapchain))
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn update_buffer(&self, buffer: ResourceHandle, offset: u64, data: &[u8]) -> RhiResult<()> {
        #[cfg(feature = "vulkan")]
        {
            use ash::vk;

            // Получаем буфер из ResourceManager
            let mut resource_manager = self
                .resource_manager
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let buffer_data = resource_manager
                .get_buffer(buffer)
                .ok_or_else(|| RhiError::ResourceNotFound("Buffer not found".to_string()))?;

            let vulkan_buffer = buffer_data.vulkan_buffer.ok_or_else(|| {
                RhiError::ResourceNotFound("Vulkan buffer handle is null".to_string())
            })?;

            // Для обновления буфера используем staging buffer + copy commands
            // Это требует доступа к command queue, что выходит за рамки IDevice
            // Поэтому возвращаем ошибку с пояснением
            return Err(RhiError::Unsupported(
                "Buffer update requires command queue access. Use command list to update buffers via staging buffer.".to_string()
            ));
        }

#[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn update_texture(&self, texture: ResourceHandle, offset_x: u32, offset_y: u32, offset_z: u32, width: u32, height: u32, depth: u32, data: &[u8]) -> RhiResult<()> {
        #[cfg(feature = "vulkan")]
        {
            info!(target: "vulkan", "update_texture: handle={:?}, offset=({},{},{}), size={}x{}x{}, data_size={}",
                  texture, offset_x, offset_y, offset_z, width, height, depth, data.len());
            Ok(())
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }
    
    fn map_buffer(&self, buffer: ResourceHandle) -> RhiResult<*mut u8> {
        #[cfg(feature = "vulkan")]
        {
            // Получаем буфер из ResourceManager
            let mut resource_manager = self
                .resource_manager
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let buffer_data = resource_manager
                .get_buffer(buffer)
                .ok_or_else(|| RhiError::ResourceNotFound("Buffer not found".to_string()))?;

            let vulkan_buffer = buffer_data.vulkan_buffer.ok_or_else(|| {
                RhiError::ResourceNotFound("Vulkan buffer handle is null".to_string())
            })?;

            // Маппинг буфера требует HOST_VISIBLE памяти и vkMapMemory
            // Это должно делаться через command list или специальный интерфейс
            return Err(RhiError::Unsupported(
                "Buffer mapping requires HOST_VISIBLE memory and vkMapMemory. Use command list for buffer updates.".to_string()
            ));
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn unmap_buffer(&self, buffer: ResourceHandle) {
        #[cfg(feature = "vulkan")]
        unsafe {
            use ash::vk;

            let mut resource_manager = self
                .resource_manager
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(buffer_data) = resource_manager.get_buffer_mut(buffer) {
                if let Some(mapped_ptr) = buffer_data.mapped_ptr.take() {
                    if let Some(vulkan_buffer) = buffer_data.vulkan_buffer {
                        // Unmap the memory
                        self.device.unmap_memory(vulkan_buffer.memory);
                        buffer_data.is_mapped = false;
                    }
                }
            }
        }

        #[cfg(not(feature = "vulkan"))]
        {
            // No-op when Vulkan is not enabled
        }
    }

    fn read_back_texture(&self, texture: ResourceHandle) -> RhiResult<Vec<u8>> {
        #[cfg(feature = "vulkan")]
        {
            // Получаем текстуру из ResourceManager
            let mut resource_manager = self
                .resource_manager
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let texture_data = resource_manager
                .get_texture(texture)
                .ok_or_else(|| RhiError::ResourceNotFound("Texture not found".to_string()))?;

            let vulkan_image = texture_data.vulkan_image.ok_or_else(|| {
                RhiError::ResourceNotFound("Vulkan image handle is null".to_string())
            })?;

            // Readback текстуры требует:
            // 1. Создания staging buffer с HOST_VISIBLE памятью
            // 2. Команды копирования из GPU-only текстуры в staging buffer
            // 3. Синхронизации через fence/semaphore
            // 4. Маппинга staging buffer и чтения данных
            // Это всё должно делаться через command list
            return Err(RhiError::Unsupported(
                "Texture readback requires staging buffer and command list. Use command list for texture readback operations.".to_string()
            ));
        }

        #[cfg(not(feature = "vulkan"))]
        {
            Err(RhiError::Unsupported(
                "Vulkan feature not enabled".to_string(),
            ))
        }
    }

    fn destroy_resource(&self, handle: ResourceHandle) {
        #[cfg(feature = "vulkan")]
        unsafe {
            use ash::vk;

            let mut resource_manager = self
                .resource_manager
                .lock()
                .unwrap_or_else(|e| e.into_inner());

            // Remove from ResourceManager - this will drop the resource data
            // The actual Vulkan destruction happens in Drop impl of VulkanBuffer/VulkanImage
            if resource_manager.remove_buffer(handle).is_some() {
                // Buffer was removed and will be destroyed when dropped
            } else if resource_manager.remove_texture(handle).is_some() {
                // Texture was removed and will be destroyed when dropped
            } else if resource_manager.remove_pipeline(handle).is_some() {
                // Pipeline was removed and will be destroyed when dropped
            }
        }

        #[cfg(not(feature = "vulkan"))]
        {
            // No-op when Vulkan is not enabled
        }
    }

    fn wait_idle(&self) -> RhiResult<()> {
        #[cfg(feature = "vulkan")]
        unsafe {
            self.device
                .device_wait_idle()
                .map_err(|e| RhiError::DeviceLost)?;
        }
        Ok(())
    }

    fn get_memory_stats(&self) -> MemoryStats {
        #[cfg(feature = "vulkan")]
        unsafe {
            use ash::vk;

            // Try to get memory stats from physical device properties
            // This is a basic implementation - full implementation would use VK_EXT_memory_budget

            let mut memory_stats = MemoryStats::default();

            // Get memory properties
            let mem_properties = self.physical_device.get_memory_properties();

            // Count total and used memory from resource manager
            let resource_manager = self
                .resource_manager
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let allocated_bytes = resource_manager.get_allocated_bytes();

            memory_stats.total_gpu_memory = mem_properties.memory_heaps[0].size as u64;
            memory_stats.used_gpu_memory = allocated_bytes;
            memory_stats.total_cpu_memory = 0; // CPU memory not tracked separately
            memory_stats.used_cpu_memory = 0;

            return memory_stats;
        }

        #[cfg(not(feature = "vulkan"))]
        {
            MemoryStats::default()
        }
    }
}

/// Factory function to create Vulkan device
pub fn create_vulkan_device(enable_validation: bool) -> RhiResult<Box<dyn IDevice>> {
    let device = VkDevice::new(enable_validation)?;
    Ok(Box::new(device))
}
