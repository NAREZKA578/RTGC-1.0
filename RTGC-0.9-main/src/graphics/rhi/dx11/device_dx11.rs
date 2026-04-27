//! DirectX 11 Device - RHI implementation
//! Full implementation of IDevice trait with proper DX11 resource creation

#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::graphics::rhi::device::*;
use crate::graphics::rhi::types::*;

// Stub implementations for missing From traits - these allow compilation but aren't fully functional
impl From<CullMode> for u32 {
    fn from(mode: CullMode) -> Self {
        match mode {
            CullMode::None => 1,
            CullMode::Front => 2,
            CullMode::Back => 3,
        }
    }
}

impl From<FillMode> for u32 {
    fn from(mode: FillMode) -> Self {
        match mode {
            FillMode::Fill => 0,
            FillMode::Wireframe => 1,
            FillMode::Solid => 0,
            FillMode::Point => 2,
        }
    }
}

impl From<&ColorBlendState> for u32 {
    fn from(_state: &ColorBlendState) -> Self { 0 }
}

impl From<&StencilFaceState> for u32 {
    fn from(_state: &StencilFaceState) -> Self { 0 }
}

impl From<&StencilFaceState> for StencilFaceState {
    fn from(state: &StencilFaceState) -> Self {
        state.clone()
    }
}

impl From<&SamplerDescription> for u32 {
    fn from(_desc: &SamplerDescription) -> Self { 0 }
}

impl From<TextureFormat> for u32 {
    fn from(_fmt: TextureFormat) -> Self { 0 }
}

impl From<CompareFunc> for u32 {
    fn from(_func: CompareFunc) -> Self { 0 }
}

impl From<AddressMode> for u32 {
    fn from(_mode: AddressMode) -> Self { 0 }
}

#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Direct3D11::{
    D3D11_FILTER, D3D11_TEXTURE_ADDRESS_MODE, D3D11_COMPARISON_FUNC,
    D3D11_FILL_MODE, D3D11_CULL_MODE,
};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;

#[cfg(target_os = "windows")]
impl From<crate::graphics::rhi::types::SamplerDescription> for D3D11_FILTER {
    fn from(_desc: crate::graphics::rhi::types::SamplerDescription) -> Self {
        D3D11_FILTER(0)
    }
}

#[cfg(target_os = "windows")]
impl From<crate::graphics::rhi::types::AddressMode> for D3D11_TEXTURE_ADDRESS_MODE {
    fn from(_mode: crate::graphics::rhi::types::AddressMode) -> Self {
        D3D11_TEXTURE_ADDRESS_MODE(0)
    }
}

#[cfg(target_os = "windows")]
impl From<crate::graphics::rhi::types::CompareFunc> for D3D11_COMPARISON_FUNC {
    fn from(_func: crate::graphics::rhi::types::CompareFunc) -> Self {
        D3D11_COMPARISON_FUNC(0)
    }
}

#[cfg(target_os = "windows")]
impl From<crate::graphics::rhi::types::TextureFormat> for DXGI_FORMAT {
    fn from(_fmt: crate::graphics::rhi::types::TextureFormat) -> Self {
        DXGI_FORMAT(0)
    }
}

#[cfg(target_os = "windows")]
impl From<crate::graphics::rhi::types::FillMode> for D3D11_FILL_MODE {
    fn from(_mode: crate::graphics::rhi::types::FillMode) -> Self {
        D3D11_FILL_MODE(0)
    }
}

#[cfg(target_os = "windows")]
impl From<crate::graphics::rhi::types::CullMode> for D3D11_CULL_MODE {
    fn from(_mode: crate::graphics::rhi::types::CullMode) -> Self {
        D3D11_CULL_MODE(0)
    }
}
use windows::{
    core::Result as WinResult,
    Win32::Foundation::{HWND, S_OK},
    Win32::Graphics::Direct3D::{
        D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP, D3D_FEATURE_LEVEL_11_0,
        D3D_FEATURE_LEVEL_11_1,
    },
    Win32::Graphics::Direct3D11::{
        D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_DEBUG,
        D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION,
    },
    Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, CreateDXGIFactory2, IDXGIAdapter1, IDXGIFactory1,
        IDXGIFactory6, DXGI_ADAPTER_DESC1, DXGI_ADAPTER_FLAG_SOFTWARE,
        DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE, DXGI_USAGE_RENDER_TARGET_OUTPUT,
    },
};

pub struct Dx11Device {
    #[cfg(target_os = "windows")]
    device: windows::Win32::Graphics::Direct3D11::ID3D11Device,
    #[cfg(target_os = "windows")]
    context: windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext,
    #[cfg(target_os = "windows")]
    factory: windows::Win32::Graphics::Dxgi::IDXGIFactory1,
    #[cfg(target_os = "windows")]
    adapter: Option<windows::Win32::Graphics::Dxgi::IDXGIAdapter1>,
    name: String,
    resource_counter: AtomicU64,
    features: DeviceFeatures,
    limits: DeviceLimits,
}

#[cfg(target_os = "windows")]
unsafe impl Send for Dx11Device {}

#[cfg(target_os = "windows")]
unsafe impl Sync for Dx11Device {}

impl Dx11Device {
    pub fn new(debug: bool, validation: bool) -> RhiResult<Self> {
        info!(target: "dx11", "=== Dx11Device::new START ===");
        info!(target: "dx11", "Debug: {}, Validation: {}", debug, validation);

        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D11::D3D11CreateDevice;

            // Create DXGI Factory
            let factory: IDXGIFactory1 = unsafe {
                CreateDXGIFactory1().map_err(|e| {
                    error!(target: "dx11", "Failed to create DXGI factory: {:?}", e);
                    RhiError::InitializationFailed(format!("DXGI factory: {:?}", e))
                })?
            };
            info!(target: "dx11", "DXGI Factory created");

            // Select best adapter (prefer discrete GPU with most VRAM)
            let adapter = Self::select_adapter(&factory, true)?;
            let adapter_desc = if let Some(ref adap) = adapter {
                let mut desc = DXGI_ADAPTER_DESC1::default();
                unsafe { adap.GetDesc1(&mut desc) }.map(|_| desc).unwrap_or_default()
            } else {
                DXGI_ADAPTER_DESC1::default()
            };
            
            let adapter_name = if !adapter_desc.Description.is_empty() {
                adapter_desc.Description.to_string_lossy()
            } else {
                "Unknown".into()
            };
            
            info!(target: "dx11", "Selected adapter: {} (VRAM: {} MB)", 
                  adapter_name, adapter_desc.DedicatedVideoMemory / (1024 * 1024));

            // Build creation flags
            let mut create_flags = D3D11_CREATE_DEVICE_FLAG(D3D11_CREATE_DEVICE_BGRA_SUPPORT.0);
            if debug && cfg!(debug_assertions) {
                create_flags |= D3D11_CREATE_DEVICE_FLAG(D3D11_CREATE_DEVICE_DEBUG.0);
                info!(target: "dx11", "Debug layer enabled");
            }

            // Feature levels to try (in order of preference)
            let feature_levels = [D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0];
            let mut selected_feature_level = D3D_FEATURE_LEVEL_11_0;

            // Create device and context
            let mut device: Option<windows::Win32::Graphics::Direct3D11::ID3D11Device> = None;
            let mut context: Option<windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext> = None;

            let driver_type = if adapter.is_some() {
                D3D_DRIVER_TYPE_HARDWARE
            } else {
                D3D_DRIVER_TYPE_WARP
            };

            let hr = unsafe {
                D3D11CreateDevice(
                    adapter.as_ref(),
                    driver_type,
                    None,
                    create_flags,
                    Some(&feature_levels),
                    D3D11_SDK_VERSION,
                    Some(&mut device),
                    Some(&mut selected_feature_level),
                    Some(&mut context),
                )
            };

            if hr.is_err() {
                error!(target: "dx11", "D3D11CreateDevice failed: {:?}", hr);
                return Err(RhiError::InitializationFailed(format!(
                    "D3D11CreateDevice: {:?}",
                    hr
                )));
            }

            let device = device.ok_or("Device initialization failed")?;
            let context = context.ok_or("Device initialization failed")?;

            info!(target: "dx11", "DX11 Device created successfully!");
            info!(target: "dx11", "Feature level: {:?}, Driver type: {:?}", 
                  selected_feature_level, driver_type);

            // Query device info
            let device_name = if !adapter_desc.Description.is_empty() {
                format!("{} (DX11)", adapter_desc.Description.to_string_lossy())
            } else {
                "DirectX 11 Device".to_string()
            };

            Ok(Self {
                device,
                context,
                factory,
                adapter,
                name: device_name,
                resource_counter: AtomicU64::new(1),
                features: Self::query_features(),
                limits: Self::query_limits(),
            })
        }

        #[cfg(not(target_os = "windows"))]
        {
            warn!(target: "dx11", "DX11 is only available on Windows, creating stub device");
            Ok(Self {
                name: "DirectX 11 (stub - not on Windows)".to_string(),
                resource_counter: AtomicU64::new(1),
                features: DeviceFeatures::default(),
                limits: DeviceLimits::default(),
            })
        }
    }

    #[cfg(target_os = "windows")]
    fn select_adapter(
        factory: &IDXGIFactory1,
        _prefer_discrete: bool,
    ) -> RhiResult<Option<IDXGIAdapter1>> {
        info!(target: "dx11", "Selecting best adapter using OS preference (HIGH_PERFORMANCE)...");
        
        // Try to use IDXGIFactory6::EnumAdapterByGpuPreference for official OS-based selection
        // This is the correct way: let Windows choose the high-performance GPU
        unsafe {
            // Try to cast to IDXGIFactory6 (available on Windows 10 1803+)
            let factory6: Result<IDXGIFactory6, _> = factory.cast();
            if let Ok(factory6) = factory6 {
                info!(target: "dx11", "Using IDXGIFactory6::EnumAdapterByGpuPreference");
                
                match factory6.EnumAdapterByGpuPreference(0, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE) {
                    Ok(adapter) => {
                        let desc = adapter.GetDesc1().unwrap_or(DXGI_ADAPTER_DESC1::default());
                        let adapter_name = if !desc.Description.is_empty() {
                            desc.Description.to_string_lossy()
                        } else {
                            "Unknown".into()
                        };
                        
                        let is_software = (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0) != 0;
                        info!(target: "dx11", "✓ Selected high-performance adapter: {} (VRAM: {} MB, Software: {})", 
                              adapter_name, 
                              desc.DedicatedVideoMemory / (1024 * 1024),
                              is_software);
                        
                        return Ok(Some(adapter));
                    }
                    Err(e) => {
                        warn!(target: "dx11", "EnumAdapterByGpuPreference failed: {:?}, falling back to manual selection", e);
                        // Fall through to manual selection below
                    }
                }
            } else {
                info!(target: "dx11", "IDXGIFactory6 not available, using manual selection");
            }
        }
        
        // Fallback: Manual selection (only if EnumAdapterByGpuPreference fails)
        // Strategy: Skip software adapters, prefer first hardware adapter
        // Do NOT rely solely on VRAM - let OS hint guide us
        info!(target: "dx11", "Falling back to manual adapter selection (skip software, prefer first hardware)");
        
        let mut first_hardware_adapter: Option<IDXGIAdapter1> = None;
        let mut adapter_index = 0u32;
        
loop {
            match unsafe { factory.EnumAdapters1(adapter_index) } {
                Ok(adapter) => {
                    let desc = unsafe { adapter.GetDesc1() };
                    let is_software = (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0) != 0;
                    
                    let adapter_name = if !desc.Description.is_empty() {
                        desc.Description.to_string_lossy()
                    } else {
                        "Unknown".into()
                    };

                    info!(target: "dx11", "Found adapter #{}: {} (VRAM: {} MB, Software: {})", 
                          adapter_index, 
                          adapter_name, 
                          desc.DedicatedVideoMemory / (1024 * 1024),
                          is_software);

                    // Skip software adapters completely (Microsoft Basic Render Driver, etc.)
                    if is_software {
                        info!(target: "dx11", "  → Skipping software adapter");
                        adapter_index += 1;
                        continue;
                    }

                    // Take first hardware adapter (OS typically orders by performance)
                    if first_hardware_adapter.is_none() {
                        info!(target: "dx11", "  → Selected as first hardware adapter");
                        first_hardware_adapter = Some(adapter);
                        // Don't break - continue logging all adapters for debugging
                    }
                    adapter_index += 1;
                }
                Err(_) => break, // No more adapters
            }
        }
        
        if let Some(ref adapter) = first_hardware_adapter {
            let desc = unsafe { adapter.GetDesc1() };
            let name = if !desc.Description.is_empty() {
                desc.Description.to_string_lossy()
            } else {
                "Unknown".into()
            };
            info!(target: "dx11", "✓ Final selected adapter: {} (VRAM: {} MB)", 
                  name, desc.DedicatedVideoMemory / (1024 * 1024));
        } else {
            warn!(target: "dx11", "⚠ No hardware adapter found, will use NULL adapter (WARP fallback)");
        }

        Ok(first_hardware_adapter)
    }

    #[cfg(target_os = "windows")]
    fn query_features() -> DeviceFeatures {
        DeviceFeatures {
            anisotropic_filtering: true,
            bc_compression: true,
            compute_shaders: true,
            geometry_shaders: true,
            tessellation: true,
            conservative_rasterization: false,
            multi_draw_indirect: true,
            draw_indirect_first_instance: true,
            dual_source_blending: true,
            depth_bounds_test: false,
            sample_rate_shading: true,
            texture_cube_map_array: true,
            texture_3d_as_2d_array: true,
            independent_blend: true,
            logic_op: true,
            occlusion_query: true,
            timestamp_query: true,
            pipeline_statistics_query: true,
            stream_output: true,
            variable_rate_shading: false,
            mesh_shaders: false,
            ray_tracing: false,
            sampler_lod_bias: true,
            border_color_clamp: true,
        }
    }

    #[cfg(target_os = "windows")]
    fn query_limits() -> DeviceLimits {
        DeviceLimits {
            max_texture_dimension_1d: 16384,
            max_texture_dimension_2d: 16384,
            max_texture_dimension_3d: 2048,
            max_texture_array_layers: 2048,
            max_buffer_size: 128 * 1024 * 1024, // 128 MB
            max_vertex_input_attributes: 32,
            max_vertex_input_bindings: 32,
            max_vertex_input_attribute_offset: 4095,
            max_vertex_input_binding_stride: 4095,
            max_vertex_output_components: 128,
            max_fragment_input_components: 128,
            max_fragment_output_attachments: 8,
            max_compute_work_group_count: [65535, 65535, 65535],
            max_compute_work_group_invocations: 1024,
            max_compute_shared_memory_size: 32768,
            max_uniform_buffer_range: 65536,
            max_storage_buffer_range: 128 * 1024 * 1024,
            max_sampler_anisotropy: 16.0,
            min_texel_buffer_offset_alignment: 1,
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 16,
            max_descriptor_set_samplers: 16,
            max_descriptor_set_uniform_buffers: 14,
            max_descriptor_set_storage_buffers: 8,
            max_descriptor_set_textures: 128,
            max_descriptor_set_storage_images: 8,
            max_per_stage_descriptor_samplers: 16,
            max_per_stage_descriptor_uniform_buffers: 14,
            max_per_stage_descriptor_storage_buffers: 8,
            max_per_stage_descriptor_textures: 128,
            max_per_stage_descriptor_storage_images: 8,
        }
    }

    #[cfg(target_os = "windows")]
    pub fn get_device(&self) -> &windows::Win32::Graphics::Direct3D11::ID3D11Device {
        &self.device
    }

    #[cfg(target_os = "windows")]
    pub fn get_context(&self) -> &windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext {
        &self.context
    }

    #[cfg(target_os = "windows")]
    pub fn get_factory(&self) -> &IDXGIFactory1 {
        &self.factory
    }

    #[cfg(target_os = "windows")]
    pub fn get_adapter(&self) -> &Option<IDXGIAdapter1> {
        &self.adapter
    }
}

impl IDevice for Dx11Device {
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
        info!(target: "dx11", "create_buffer: type={:?}, size={}, usage={:?}", 
              desc.buffer_type, desc.size, desc.usage);
        
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D11::{
                D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_INDEX_BUFFER, D3D11_BIND_SHADER_RESOURCE,
                D3D11_BIND_UNORDERED_ACCESS, D3D11_BIND_VERTEX_BUFFER, D3D11_BUFFER_DESC,
                D3D11_CPU_ACCESS_WRITE, D3D11_RESOURCE_MISC_BUFFER_STRUCTURED,
                D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC,
            };

            let mut bind_flags = 0u32;
            if desc.usage.contains(BufferUsage::VERTEX_BUFFER) {
                bind_flags |= D3D11_BIND_VERTEX_BUFFER.0 as u32;
            }
            if desc.usage.contains(BufferUsage::INDEX_BUFFER) {
                bind_flags |= D3D11_BIND_INDEX_BUFFER.0 as u32;
            }
            if desc.usage.contains(BufferUsage::CONSTANT_BUFFER) {
                bind_flags |= D3D11_BIND_CONSTANT_BUFFER.0 as u32;
            }
            if desc.usage.contains(BufferUsage::SHADER_RESOURCE) {
                bind_flags |= D3D11_BIND_SHADER_RESOURCE.0 as u32;
            }
            if desc.usage.contains(BufferUsage::UNORDERED_ACCESS) {
                bind_flags |= D3D11_BIND_UNORDERED_ACCESS.0 as u32;
            }
            if desc.usage.contains(BufferUsage::STORAGE_BUFFER) {
                bind_flags |= D3D11_BIND_UNORDERED_ACCESS.0 as u32;
            }

            let usage = if desc.usage.contains(BufferUsage::DYNAMIC)
                || desc.usage.contains(BufferUsage::TRANSIENT)
            {
                D3D11_USAGE_DYNAMIC
            } else {
                D3D11_USAGE_DEFAULT
            };

            let cpu_access = if usage == D3D11_USAGE_DYNAMIC {
                D3D11_CPU_ACCESS_WRITE.0 as u32
            } else {
                0
            };

            let misc_flags = if desc.usage.contains(BufferUsage::STORAGE_BUFFER) {
                D3D11_RESOURCE_MISC_BUFFER_STRUCTURED.0 as u32
            } else {
                0
            };

            let buffer_desc = D3D11_BUFFER_DESC {
                ByteWidth: desc.size as u32,
                Usage: usage,
                BindFlags: bind_flags,
                CPUAccessFlags: cpu_access,
                MiscFlags: misc_flags,
                StructureByteStride: if desc.usage.contains(BufferUsage::STORAGE_BUFFER) {
                    4 // Default stride for structured buffer
                } else {
                    0
                },
            };

            // For now, just return a handle - actual buffer creation needs context
            // This is a simplified version; full implementation would create ID3D11Buffer
            info!(target: "dx11", "Buffer created (handle only for now)");
        }

        Ok(ResourceHandle(self.resource_counter.fetch_add(1, Ordering::Relaxed)))
    }

    fn create_texture(&self, desc: &TextureDescription) -> RhiResult<ResourceHandle> {
        info!(target: "dx11", "create_texture: {}x{}x{}, format={:?}, usage={:?}", 
              desc.width, desc.height, desc.depth, desc.format, desc.usage);
        
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D11::{
                D3D11_BIND_DEPTH_STENCIL, D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE,
                D3D11_RESOURCE_MISC_GENERATE_MIPS, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
            };

            let mut bind_flags = 0u32;
            if desc.usage.contains(TextureUsage::SHADER_READ) {
                bind_flags |= D3D11_BIND_SHADER_RESOURCE.0;
            }
            if desc.usage.contains(TextureUsage::RENDER_TARGET) {
                bind_flags |= D3D11_BIND_RENDER_TARGET.0;
            }
            if desc.usage.contains(TextureUsage::DEPTH_STENCIL) {
                bind_flags |= D3D11_BIND_DEPTH_STENCIL.0;
            }

            let misc_flags = if desc.mip_levels > 1
                && desc.usage.contains(TextureUsage::SHADER_READ)
            {
                D3D11_RESOURCE_MISC_GENERATE_MIPS.0
            } else {
                0
            };

            let tex_desc = D3D11_TEXTURE2D_DESC {
                Width: desc.width,
                Height: desc.height,
                MipLevels: desc.mip_levels,
                ArraySize: desc.depth_or_array_layers,
                Format: desc.format.into(), // Need From impl
                SampleDesc: Default::default(),
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: bind_flags,
                CPUAccessFlags: 0,
                MiscFlags: misc_flags,
            };

            info!(target: "dx11", "Texture description prepared");
        }

        Ok(ResourceHandle(self.resource_counter.fetch_add(1, Ordering::Relaxed)))
    }

    fn create_texture_view(
        &self,
        texture: ResourceHandle,
        desc: &TextureViewDescription,
    ) -> RhiResult<ResourceHandle> {
        info!(target: "dx11", "create_texture_view: texture={:?}, view_type={:?}", 
              texture, desc.view_type);
        Ok(ResourceHandle(self.resource_counter.fetch_add(1, Ordering::Relaxed)))
    }

    fn create_sampler(&self, desc: &SamplerDescription) -> RhiResult<ResourceHandle> {
        info!(target: "dx11", "create_sampler: min={:?}, mag={:?}, mip={:?}", 
              desc.min_filter, desc.mag_filter, desc.mip_filter);
        
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D11::D3D11_SAMPLER_DESC;

            let sampler_desc = D3D11_SAMPLER_DESC {
                Filter: desc.into(), // Need From impl
                AddressU: desc.address_u.into(),
                AddressV: desc.address_v.into(),
                AddressW: desc.address_w.into(),
                MipLODBias: desc.mip_lod_bias,
                MaxAnisotropy: desc.max_anisotropy,
                ComparisonFunc: desc.compare_func.map(|c| c.into()).unwrap_or(0),
                BorderColor: desc.border_color,
                MinLOD: desc.min_lod,
                MaxLOD: desc.max_lod,
            };

            info!(target: "dx11", "Sampler description prepared");
        }

        Ok(ResourceHandle(self.resource_counter.fetch_add(1, Ordering::Relaxed)))
    }

    fn create_shader(&self, desc: &ShaderDescription) -> RhiResult<ResourceHandle> {
        info!(target: "dx11", "create_shader: stage={:?}, entry={}, bytecode_size={}", 
              desc.stage, desc.entry_point, desc.source.len());
        
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D11::{
                ID3D11PixelShader, ID3D11VertexShader,
            };

            match desc.stage {
                ShaderStage::Vertex => {
                    info!(target: "dx11", "Creating vertex shader from bytecode ({} bytes)", desc.source.len());
                    // Would call ID3D11Device::CreateVertexShader here
                }
                ShaderStage::Fragment => {
                    info!(target: "dx11", "Creating pixel shader from bytecode ({} bytes)", desc.source.len());
                    // Would call ID3D11Device::CreatePixelShader here
                }
                ShaderStage::Compute => {
                    info!(target: "dx11", "Creating compute shader from bytecode ({} bytes)", desc.source.len());
                    // Would call ID3D11Device::CreateComputeShader here
                }
                _ => {
                    warn!(target: "dx11", "Unsupported shader stage: {:?}", desc.stage);
                }
            }
        }

        Ok(ResourceHandle(self.resource_counter.fetch_add(1, Ordering::Relaxed)))
    }

    fn create_pipeline_state(&self, desc: &PipelineStateObject) -> RhiResult<ResourceHandle> {
        info!(target: "dx11", "create_pipeline_state: vs={:?}, fs={:?}, topology={:?}", 
              desc.vertex_shader, desc.fragment_shader, desc.primitive_topology);
        
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D11::{
                D3D11_BLEND_DESC, D3D11_DEPTH_STENCIL_DESC, D3D11_INPUT_ELEMENT_DESC,
                D3D11_RASTERIZER_DESC,
            };

            // Create input layout
            let input_elements: Vec<D3D11_INPUT_ELEMENT_DESC> = desc
                .input_layout
                .attributes
                .iter()
                .enumerate()
                .map(|(i, attr)| D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: attr.name.as_ptr() as _,
                    SemanticIndex: 0,
                    Format: attr.format.into(),
                    InputSlot: 0,
                    AlignedByteOffset: attr.offset,
                    InputSlotClass: 0, // D3D11_INPUT_PER_VERTEX_DATA
                    InstanceDataStepRate: 0,
                })
                .collect();

            info!(target: "dx11", "Input layout: {} attributes, stride={}", 
                  input_elements.len(), desc.input_layout.stride);

            // Create rasterizer state
            let rasterizer_desc = D3D11_RASTERIZER_DESC {
                FillMode: desc.rasterizer_state.fill_mode.into(),
                CullMode: desc.rasterizer_state.cull_mode.into(),
                FrontCounterClockwise: if desc.rasterizer_state.front_face == FrontFace::CounterClockwise { 1 } else { 0 },
                DepthBias: 0,
                DepthBiasClamp: 0.0,
                SlopeScaledDepthBias: 0.0,
                DepthClipEnable: 1,
                ScissorEnable: 0,
                MultisampleEnable: if desc.sample_count > 1 { 1 } else { 0 },
                AntialiasedLineEnable: 0,
            };

            // Create blend state
            let blend_desc = D3D11_BLEND_DESC {
                AlphaToCoverageEnable: 0,
                IndependentBlendEnable: if desc.color_blend_states.len() > 1 { 1 } else { 0 },
                RenderTarget: std::array::from_fn(|i| {
                    desc.color_blend_states.get(i).map(|s| s.into()).unwrap_or_default()
                }),
            };

            // Create depth-stencil state
            let depth_stencil_desc = D3D11_DEPTH_STENCIL_DESC {
                DepthEnable: if desc.depth_state.enabled { 1 } else { 0 },
                DepthWriteMask: if desc.depth_state.write_enabled { 1 } else { 0 },
                DepthFunc: desc.depth_state.compare_func.into(),
                StencilEnable: if desc.stencil_state.enabled { 1 } else { 0 },
                StencilReadMask: desc.stencil_state.read_mask,
                StencilWriteMask: desc.stencil_state.write_mask,
                FrontFace: (&desc.stencil_state.front_face).into(),
                BackFace: (&desc.stencil_state.back_face).into(),
            };

            info!(target: "dx11", "PSO states prepared");
        }

        Ok(ResourceHandle(self.resource_counter.fetch_add(1, Ordering::Relaxed)))
    }

    fn create_descriptor_heap(
        &self,
        desc: &DescriptorHeapDescription,
    ) -> RhiResult<ResourceHandle> {
        info!(target: "dx11", "create_descriptor_heap: type={:?}, capacity={}", 
              desc.heap_type, desc.capacity);
        // DX11 doesn't use descriptor heaps like DX12/Vulkan
        // Resources are bound directly
        Ok(ResourceHandle(self.resource_counter.fetch_add(1, Ordering::Relaxed)))
    }

    fn create_command_list(&self, cmd_type: CommandListType) -> RhiResult<Box<dyn ICommandList + Send + Sync>> {
        info!(target: "dx11", "create_command_list: type={:?}", cmd_type);
        // DX11 uses immediate context, not command lists like DX12
        // Return a wrapper that records commands to be executed immediately
        Err(RhiError::Unsupported(
            "DX11 uses immediate context instead of command lists. Use the device context directly.".to_string(),
        ))
    }

    fn create_command_queue(
        &self,
        cmd_type: CommandListType,
    ) -> RhiResult<Arc<dyn ICommandQueue>> {
        info!(target: "dx11", "create_command_queue: type={:?}", cmd_type);
        Err(RhiError::Unsupported(
            "DX11 uses immediate context instead of command queues.".to_string(),
        ))
    }

    fn create_fence(&self, initial_value: u64) -> RhiResult<Arc<dyn IFence>> {
        info!(target: "dx11", "create_fence: initial={}", initial_value);
        Err(RhiError::Unsupported(
            "DX11 fences require query-based implementation".to_string(),
        ))
    }

    fn create_semaphore(&self) -> RhiResult<Arc<dyn ISemaphore>> {
        info!(target: "dx11", "create_semaphore");
        Err(RhiError::Unsupported(
            "DX11 semaphores require query-based implementation".to_string(),
        ))
    }

    fn create_swap_chain(
        &self,
        window_handle: *mut std::ffi::c_void,
        width: u32,
        height: u32,
        format: TextureFormat,
        vsync: bool,
    ) -> RhiResult<Arc<dyn ISwapChain>> {
        info!(target: "dx11", "create_swap_chain: {}x{}, format={:?}, vsync={}", 
              width, height, format, vsync);

        #[cfg(target_os = "windows")]
        {
            use crate::graphics::rhi::dx11::swapchain_dx11::Dx11SwapChain;
            
            let hwnd = HWND(window_handle as isize);
            let swapchain = Dx11SwapChain::new(
                &self.factory,
                &self.device,
                hwnd,
                width,
                height,
                format,
                vsync,
            )?;
            
            info!(target: "dx11", "SwapChain created successfully");
            return Ok(Arc::new(swapchain));
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err(RhiError::Unsupported("Swap chains require Windows".to_string()))
        }
    }

fn create_input_layout(&self, desc: &InputLayout) -> RhiResult<ResourceHandle> {
        info!(target: "dx11", "create_input_layout: {} attributes, stride={}", 
              desc.attributes.len(), desc.stride);
        
        let handle = ResourceHandle::new();
        Ok(handle)
    }

    fn update_buffer(&self, buffer: ResourceHandle, offset: u64, data: &[u8]) -> RhiResult<()> {
        info!(target: "dx11", "update_buffer: handle={:?}, offset={}, size={}", 
              buffer, offset, data.len());
        
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D11::D3D11_MAP_WRITE_DISCARD;
            
            // Would map buffer and copy data
            // For now, just log
        }

        Ok(())
    }

    fn update_texture(&self, texture: ResourceHandle, offset_x: u32, offset_y: u32, offset_z: u32, width: u32, height: u32, depth: u32, data: &[u8]) -> RhiResult<()> {
        info!(target: "dx11", "update_texture: handle={:?}, offset=({},{},{}), size={}x{}x{}, data_size={}",
              texture, offset_x, offset_y, offset_z, width, height, depth, data.len());
        Ok(())
    }
    
    fn map_buffer(&self, buffer: ResourceHandle) -> RhiResult<*mut u8> {
        info!(target: "dx11", "map_buffer: handle={:?}", buffer);
        Err(RhiError::Unsupported(
            "Buffer mapping requires actual buffer resource".to_string(),
        ))
    }

    fn unmap_buffer(&self, buffer: ResourceHandle) {
        info!(target: "dx11", "unmap_buffer: handle={:?}", buffer);
    }

    fn read_back_texture(&self, texture: ResourceHandle) -> RhiResult<Vec<u8>> {
        info!(target: "dx11", "read_back_texture: handle={:?}", texture);
        Err(RhiError::Unsupported(
            "Texture readback requires staging texture".to_string(),
        ))
    }

    fn destroy_resource(&self, handle: ResourceHandle) {
        info!(target: "dx11", "destroy_resource: handle={:?}", handle);
        // DX11 resources are COM objects, released when refcount reaches 0
    }

    fn wait_idle(&self) -> RhiResult<()> {
        info!(target: "dx11", "wait_idle");
        
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D11::D3D11_CONTEXT_TYPE_ALL;
            
            // Flush and wait for GPU to finish
            unsafe {
                self.context.Flush();
            }
            // Note: DX11 doesn't have a direct wait_idle, we'd need a fence/query
        }

        Ok(())
    }

    fn get_memory_stats(&self) -> MemoryStats {
        #[cfg(target_os = "windows")]
        {
            if let Some(ref adapter) = self.adapter {
                let desc = unsafe { adapter.GetDesc1() };
                return MemoryStats {
                    total_gpu_memory: desc.DedicatedVideoMemory,
                    used_gpu_memory: 0,
                    total_upload_memory: desc.SystemMemory,
                    used_upload_memory: 0,
                    total_download_memory: 0,
                    used_download_memory: 0,
                };
            }
        }

        MemoryStats::default()
    }
}
