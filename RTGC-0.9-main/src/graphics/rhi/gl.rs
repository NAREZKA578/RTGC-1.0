//! OpenGL RHI Backend для RTGC-0.9 - Реализация на glow

use super::device::{
    IDevice, ICommandList, ICommandQueue, IFence, ISemaphore, ISwapChain, ResourceBarrier,
    TextureViewDescription, DescriptorHeapDescription, RenderPassDescription,
    DeviceFeatures, DeviceLimits, MemoryStats, IndexFormat, LoadOp, StoreOp, RenderAttachment, DepthStencilAttachment,
};
use super::types::{
    ResourceHandle, BufferDescription, BufferType, BufferUsage, TextureDescription,
    TextureFormat, TextureType, SamplerDescription, ShaderDescription,
    PipelineStateObject, CommandListType, Viewport, ScissorRect, PrimitiveTopology,
    RhiResult, RhiError, ShaderStage, ResourceState, ClearValue, InputLayout,
};
use glow::{Context, HasContext, NativeTexture, NativeBuffer, NativeSampler, NativeFramebuffer, NativeVertexArray, NativeShader, NativeProgram};
use std::cell::Cell;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use parking_lot::Mutex;

// ==================== ВНУТРЕННИЕ СТРУКТУРЫ ====================

/// Внутренние данные буфера
#[derive(Clone)]
pub struct GlBufferInternal {
    pub gl_id: NativeBuffer,
    pub size: u64,
    pub buffer_type: BufferType,
    pub usage: BufferUsage,
    pub state: ResourceState,
}

/// Внутренние данные текстуры
#[derive(Clone)]
pub struct GlTextureInternal {
    pub gl_id: NativeTexture,
    pub desc: TextureDescription,
    pub target: u32,
}

/// Внутренние данные сэмплера
#[derive(Clone)]
pub struct GlSamplerInternal {
    pub gl_id: NativeSampler,
    pub desc: SamplerDescription,
}

/// Внутренние данные шейдера
#[derive(Clone)]
pub struct GlShaderInternal {
    pub gl_id: NativeShader,
    pub stage: ShaderStage,
    pub source: String,
}

/// Внутренние данные PSO
#[derive(Clone)]
pub struct GlPipelineInternal {
    pub program: NativeProgram,
    pub vertex_array: NativeVertexArray,
    pub topology: PrimitiveTopology,
    pub desc: PipelineStateObject,
}

/// Framebuffer для render pass
pub struct GlFramebufferInternal {
    pub gl_id: NativeFramebuffer,
    pub color_attachments: Vec<ResourceHandle>,
    pub depth_stencil_attachment: Option<ResourceHandle>,
}

/// Заглушка для semaphore (в OpenGL эмулируется через fence)
pub struct GlSemaphoreInternal {
    _private: (),
}

// SAFETY: GlSemaphoreInternal wraps an OpenGL semaphore which is a GPU resource.
// It is safe to Send/Sync because the actual synchronization is handled by the GPU.
// CPU-side access must be synchronized via the command queue.
unsafe impl Send for GlSemaphoreInternal {}
unsafe impl Sync for GlSemaphoreInternal {}

impl super::device::ISemaphore for GlSemaphoreInternal {}

/// SwapChain для OpenGL
pub struct GlSwapChainInternal {
    pub context: Arc<Context>,
    pub width: Cell<u32>,
    pub height: Cell<u32>,
    pub format: TextureFormat,
    pub framebuffer: NativeFramebuffer,
    pub color_texture: NativeTexture,
    pub depth_texture: Option<NativeTexture>,
}

impl Clone for GlSwapChainInternal {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            width: Cell::new(self.width.get()),
            height: Cell::new(self.height.get()),
            format: self.format,
            framebuffer: self.framebuffer.clone(),
            color_texture: self.color_texture.clone(),
            depth_texture: self.depth_texture.clone(),
        }
    }
}

impl GlSwapChainInternal {
    pub fn owned_clone(&self) -> Self {
        self.clone()
    }
}

// SAFETY: GlSwapChainInternal contains OpenGL resources (framebuffer, textures) which
// are bound to the OpenGL context. Send/Sync is safe because these resources are
// managed by the GPU and accessed through thread-safe command queues. The context
// itself should remain on the main thread.
unsafe impl Send for GlSwapChainInternal {}
unsafe impl Sync for GlSwapChainInternal {}

// ==================== GL DEVICE ====================

/// OpenGL Device implementation
pub struct GlDevice {
    pub context: Arc<Context>,
    resource_counter: AtomicU64,
    device_name: String,
    features: DeviceFeatures,
    limits: DeviceLimits,
    buffers: Mutex<HashMap<ResourceHandle, GlBufferInternal>>,
    textures: Mutex<HashMap<ResourceHandle, GlTextureInternal>>,
    samplers: Mutex<HashMap<ResourceHandle, GlSamplerInternal>>,
    shaders: Mutex<HashMap<ResourceHandle, GlShaderInternal>>,
    pipelines: Mutex<HashMap<ResourceHandle, GlPipelineInternal>>,
}

// SAFETY: GlDevice manages OpenGL resources through interior mutability (Mutex).
// Send/Sync is safe because all mutable state is protected by Mutexes, and the
// underlying OpenGL context is designed to be used from a single thread (main thread).
// Cross-thread resource access is coordinated through the RHI command system.
unsafe impl Send for GlDevice {}
unsafe impl Sync for GlDevice {}

impl From<&GlDevice> for GlDevice {
    fn from(device: &GlDevice) -> Self {
        GlDevice {
            context: device.context.clone(),
            resource_counter: AtomicU64::new(device.resource_counter.load(Ordering::Relaxed)),
            device_name: device.device_name.clone(),
            features: device.features.clone(),
            limits: device.limits.clone(),
            buffers: Mutex::new(device.buffers.lock().clone()),
            textures: Mutex::new(device.textures.lock().clone()),
            samplers: Mutex::new(device.samplers.lock().clone()),
            shaders: Mutex::new(device.shaders.lock().clone()),
            pipelines: Mutex::new(device.pipelines.lock().clone()),
        }
    }
}

impl GlDevice {
    pub fn new(context: Arc<Context>) -> Self {
        let device_name = unsafe { context.get_parameter_string(glow::RENDERER) };
        let vendor = unsafe { context.get_parameter_string(glow::VENDOR) };
        let version = unsafe { context.get_parameter_string(glow::VERSION) };

        tracing::info!("OpenGL Device: {} ({}) - {}", device_name, vendor, version);

        let features = Self::query_features(&context);
        let limits = Self::query_limits(&context);

        Self {
            context,
            resource_counter: AtomicU64::new(1),
            device_name,
            features,
            limits,
            buffers: Mutex::new(HashMap::new()),
            textures: Mutex::new(HashMap::new()),
            samplers: Mutex::new(HashMap::new()),
            shaders: Mutex::new(HashMap::new()),
            pipelines: Mutex::new(HashMap::new()),
        }
    }
    
    /// Create a new OpenGL device with debug mode support
    pub fn create(debug_enabled: bool) -> Result<Self, crate::graphics::rhi::types::RhiError> {
        // Note: This method requires an active OpenGL context
        // In practice, this should be called after glutin/winit creates a window
        if debug_enabled {
            tracing::debug!("OpenGL device created (debug mode: {})", debug_enabled);
        }
        // The actual context will be provided by the graphics system
        // This is a placeholder for RHI factory integration
        Err(crate::graphics::rhi::types::RhiError::InitializationFailed(
            "OpenGL context must be provided by windowing system. Use GlDevice::new() with a valid Context.".to_string(),
        ))
    }

    fn generate_handle(&self) -> ResourceHandle {
        ResourceHandle(self.resource_counter.fetch_add(1, Ordering::Relaxed))
    }
    
    /// Создаёт mock-устройство для тестов
    #[cfg(test)]
    pub fn mock() -> Self {
        use std::sync::Arc;
        // Создаём минимальный контекст для тестов
        // В реальности это требует активного OpenGL контекста
        // Для тестов используем заглушку
        Self {
            context: Arc::new(Context::default()),
            resource_counter: AtomicU64::new(1),
            device_name: "Mock OpenGL Device".to_string(),
            features: DeviceFeatures::default(),
            limits: DeviceLimits::default(),
            buffers: Mutex::new(HashMap::new()),
            textures: Mutex::new(HashMap::new()),
            samplers: Mutex::new(HashMap::new()),
            shaders: Mutex::new(HashMap::new()),
            pipelines: Mutex::new(HashMap::new()),
        }
    }

    fn query_features(_ctx: &Context) -> DeviceFeatures {
        DeviceFeatures {
            anisotropic_filtering: true,
            bc_compression: false, // Зависит от драйвера
            compute_shaders: true, // Заглушка для OpenGL 4.3+
            geometry_shaders: true, // Заглушка для OpenGL 3.2+
            tessellation: false, // Заглушка - зависит от версии
            conservative_rasterization: false,
            multi_draw_indirect: true, // Заглушка для OpenGL 4.0+
            draw_indirect_first_instance: false,
            dual_source_blending: true,
            depth_bounds_test: false,
            sample_rate_shading: true, // Заглушка для OpenGL 4.0+
            texture_cube_map_array: false, // Заглушка для OpenGL 4.0+
            texture_3d_as_2d_array: true,
            independent_blend: true, // Заглушка для OpenGL 4.0+
            logic_op: true,
            occlusion_query: true,
            timestamp_query: true, // Заглушка для OpenGL 3.3+
            pipeline_statistics_query: false,
            stream_output: false,
            variable_rate_shading: false,
            mesh_shaders: false,
            ray_tracing: false,
            sampler_lod_bias: true,
            border_color_clamp: false,
        }
    }

    fn query_limits(ctx: &Context) -> DeviceLimits {
        let max_texture_size = unsafe { ctx.get_parameter_i32(glow::MAX_TEXTURE_SIZE) } as u32;
        let max_3d_texture_size = unsafe { ctx.get_parameter_i32(glow::MAX_3D_TEXTURE_SIZE) } as u32;
        let max_array_layers = unsafe { ctx.get_parameter_i32(glow::MAX_ARRAY_TEXTURE_LAYERS) } as u32;
        let max_vertex_attribs = unsafe { ctx.get_parameter_i32(glow::MAX_VERTEX_ATTRIBS) } as u32;
        let _max_uniforms = unsafe { ctx.get_parameter_i32(glow::MAX_VERTEX_UNIFORM_COMPONENTS) } as u32;
        
        DeviceLimits {
            max_texture_dimension_1d: max_texture_size,
            max_texture_dimension_2d: max_texture_size,
            max_texture_dimension_3d: max_3d_texture_size,
            max_texture_array_layers: max_array_layers,
            max_buffer_size: 256 * 1024 * 1024, // 256 MB
            max_vertex_input_attributes: max_vertex_attribs.min(16),
            max_vertex_input_bindings: max_vertex_attribs.min(16),
            max_vertex_input_attribute_offset: 2047,
            max_vertex_input_binding_stride: 2048,
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
            min_storage_buffer_offset_alignment: 1,
            max_descriptor_set_samplers: 128,
            max_descriptor_set_uniform_buffers: 84,
            max_descriptor_set_storage_buffers: 96,
            max_descriptor_set_textures: 128,
            max_descriptor_set_storage_images: 64,
            max_per_stage_descriptor_samplers: 32,
            max_per_stage_descriptor_uniform_buffers: 14,
            max_per_stage_descriptor_storage_buffers: 16,
            max_per_stage_descriptor_textures: 48,
            max_per_stage_descriptor_storage_images: 16,
        }
    }
    
    fn get_gl_texture_target(desc: &TextureDescription) -> u32 {
        match desc.texture_type {
            TextureType::Texture1D => glow::TEXTURE_1D,
            TextureType::Texture2D => glow::TEXTURE_2D,
            TextureType::Texture3D => glow::TEXTURE_3D,
            TextureType::TextureCube => glow::TEXTURE_CUBE_MAP,
            TextureType::Texture1DArray => glow::TEXTURE_1D_ARRAY,
            TextureType::Texture2DArray => glow::TEXTURE_2D_ARRAY,
            TextureType::TextureCubeArray => glow::TEXTURE_CUBE_MAP_ARRAY,
        }
    }
    
    fn get_gl_format(format: TextureFormat) -> (u32, u32, u32) {
        match format {
            TextureFormat::R8Unorm => (glow::RED, glow::UNSIGNED_BYTE, glow::R8),
            TextureFormat::R8Uint => (glow::RED_INTEGER, glow::UNSIGNED_BYTE, glow::R8UI),
            TextureFormat::R16Float => (glow::RED, glow::HALF_FLOAT, glow::R16F),
            TextureFormat::R32Float => (glow::RED, glow::FLOAT, glow::R32F),
            TextureFormat::Rg8Unorm => (glow::RG, glow::UNSIGNED_BYTE, glow::RG8),
            TextureFormat::Rg16Float => (glow::RG, glow::HALF_FLOAT, glow::RG16F),
            TextureFormat::Rg32Float => (glow::RG, glow::FLOAT, glow::RG32F),
            TextureFormat::Rgba8Unorm => (glow::RGBA, glow::UNSIGNED_BYTE, glow::RGBA8),
            TextureFormat::Rgba8Uint => (glow::RGBA_INTEGER, glow::UNSIGNED_BYTE, glow::RGBA8UI),
            TextureFormat::Rgba8Snorm => (glow::RGBA, glow::BYTE, glow::RGBA8_SNORM),
            TextureFormat::Rgba16Float => (glow::RGBA, glow::HALF_FLOAT, glow::RGBA16F),
            TextureFormat::Rgba32Float => (glow::RGBA, glow::FLOAT, glow::RGBA32F),
            TextureFormat::Bgra8Unorm => (glow::BGRA, glow::UNSIGNED_BYTE, glow::RGBA8), // Emulated
            TextureFormat::Depth16Unorm => (glow::DEPTH_COMPONENT, glow::UNSIGNED_SHORT, glow::DEPTH_COMPONENT16),
            TextureFormat::Depth24Plus => (glow::DEPTH_COMPONENT, glow::UNSIGNED_INT, glow::DEPTH_COMPONENT24),
            TextureFormat::Depth32Float => (glow::DEPTH_COMPONENT, glow::FLOAT, glow::DEPTH_COMPONENT32F),
            TextureFormat::Stencil8 => (glow::STENCIL_INDEX, glow::UNSIGNED_BYTE, glow::STENCIL_INDEX8),
            TextureFormat::Depth24PlusStencil8 => (glow::DEPTH_STENCIL, glow::UNSIGNED_INT_24_8, glow::DEPTH24_STENCIL8),
            TextureFormat::Depth32FloatStencil8 => (glow::DEPTH_STENCIL, glow::FLOAT_32_UNSIGNED_INT_24_8_REV, glow::DEPTH32F_STENCIL8),
            TextureFormat::BC1RgbaUnorm => (glow::COMPRESSED_RGBA_S3TC_DXT1_EXT, glow::NONE, glow::NONE),
            TextureFormat::BC3RgbaUnorm => (glow::COMPRESSED_RGBA_S3TC_DXT3_EXT, glow::NONE, glow::NONE),
            TextureFormat::BC7RgbaUnorm => (glow::COMPRESSED_RGBA_BPTC_UNORM, glow::NONE, glow::NONE),
            // Default fallback for unsupported formats
            _ => (glow::RGBA, glow::UNSIGNED_BYTE, glow::RGBA8),
        }
    }
    
    fn compile_shader(&self, stage: ShaderStage, source: &str) -> RhiResult<NativeShader> {
        let gl_type = match stage {
            ShaderStage::Vertex => glow::VERTEX_SHADER,
            ShaderStage::Fragment => glow::FRAGMENT_SHADER,
            ShaderStage::Compute => glow::COMPUTE_SHADER,
            ShaderStage::Geometry => glow::GEOMETRY_SHADER,
            ShaderStage::TessellationControl => glow::TESS_CONTROL_SHADER,
            ShaderStage::TessellationEvaluation => glow::TESS_EVALUATION_SHADER,
        };

        let shader = unsafe { self.context.create_shader(gl_type) }
            .map_err(|_| RhiError::InitializationFailed("Failed to create shader".to_string()))?;

        unsafe {
            self.context.shader_source(shader, source);
            self.context.compile_shader(shader);

            if !self.context.get_shader_compile_status(shader) {
                let error_log = self.context.get_shader_info_log(shader);
                self.context.delete_shader(shader);
                return Err(RhiError::CompilationFailed(error_log));
            }
        }

        Ok(shader)
    }
}

impl IDevice for GlDevice {
    fn get_device_name(&self) -> &str { &self.device_name }
    
    fn get_features(&self) -> DeviceFeatures { self.features.clone() }
    
    fn get_limits(&self) -> DeviceLimits { self.limits.clone() }
    
    fn create_buffer(&self, desc: &BufferDescription) -> RhiResult<ResourceHandle> {
        let gl_id = unsafe { self.context.create_buffer() }
            .map_err(|_| RhiError::InitializationFailed("Failed to create buffer".to_string()))?;
        
        let gl_usage = match desc.usage {
            BufferUsage::IMMUTABLE => glow::STATIC_DRAW,
            BufferUsage::DYNAMIC => glow::DYNAMIC_DRAW,
            BufferUsage::TRANSIENT => glow::STREAM_DRAW,
            BufferUsage::UPLOAD => glow::STREAM_DRAW,
            BufferUsage::READBACK => glow::STREAM_READ,
            _ => glow::DYNAMIC_DRAW,
        };
        
        unsafe {
            self.context.bind_buffer(glow::ARRAY_BUFFER, Some(gl_id));
            self.context.buffer_data_size(glow::ARRAY_BUFFER, desc.size as i32, gl_usage);
            self.context.bind_buffer(glow::ARRAY_BUFFER, None);
        }
        
        let handle = self.generate_handle();
        let buffer = GlBufferInternal {
            gl_id,
            size: desc.size,
            buffer_type: desc.buffer_type,
            usage: desc.usage,
            state: ResourceState::Common,
        };
        
        self.buffers.lock().insert(handle, buffer);
        Ok(handle)
    }
    
    fn create_texture(&self, desc: &TextureDescription) -> RhiResult<ResourceHandle> {
        let gl_id = unsafe { self.context.create_texture() }
            .map_err(|_| RhiError::InitializationFailed("Failed to create texture".to_string()))?;
        
        let target = Self::get_gl_texture_target(desc);
        let (internal_format, _format, _ty) = Self::get_gl_format(desc.format);
        
        unsafe {
            self.context.bind_texture(target, Some(gl_id));
            
            // Настройка параметров текстуры
            self.context.tex_parameter_i32(target, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            self.context.tex_parameter_i32(target, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            self.context.tex_parameter_i32(target, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            self.context.tex_parameter_i32(target, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            
            // Выделение памяти
            match desc.texture_type {
                TextureType::Texture1D => {
                    self.context.tex_storage_1d(target, desc.mip_levels as i32, internal_format, desc.width as i32);
                }
                TextureType::Texture2D => {
                    self.context.tex_storage_2d(target, desc.mip_levels as i32, internal_format, desc.width as i32, desc.height as i32);
                }
                TextureType::Texture3D => {
                    self.context.tex_storage_3d(target, desc.mip_levels as i32, internal_format, desc.width as i32, desc.height as i32, desc.depth as i32);
                }
                TextureType::TextureCube => {
                    for face in 0..6 {
                        self.context.tex_storage_2d(glow::TEXTURE_CUBE_MAP_POSITIVE_X + face, desc.mip_levels as i32, internal_format, desc.width as i32, desc.height as i32);
                    }
                }
                _ => {
                    self.context.tex_storage_2d(target, desc.mip_levels as i32, internal_format, desc.width as i32, desc.height as i32);
                }
            }
            
            self.context.bind_texture(target, None);
        }
        
        let handle = self.generate_handle();
        let texture = GlTextureInternal {
            gl_id,
            desc: desc.clone(),
            target,
        };
        
        self.textures.lock().insert(handle, texture);
        Ok(handle)
    }
    
    fn create_texture_view(&self, texture: ResourceHandle, _desc: &TextureViewDescription) -> RhiResult<ResourceHandle> {
        // В OpenGL texture view не требуется - используем саму текстуру
        Ok(texture)
    }
    
    fn create_sampler(&self, desc: &SamplerDescription) -> RhiResult<ResourceHandle> {
        let gl_id = unsafe { self.context.create_sampler() }
            .map_err(|_| RhiError::InitializationFailed("Failed to create sampler".to_string()))?;
        
        unsafe {
            self.context.sampler_parameter_i32(gl_id, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            self.context.sampler_parameter_i32(gl_id, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            self.context.sampler_parameter_i32(gl_id, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            self.context.sampler_parameter_i32(gl_id, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
        }
        
        let handle = self.generate_handle();
        let sampler = GlSamplerInternal {
            gl_id,
            desc: desc.clone(),
        };
        
        self.samplers.lock().insert(handle, sampler);
        Ok(handle)
    }
    
    fn create_shader(&self, desc: &ShaderDescription) -> RhiResult<ResourceHandle> {
        let source = std::str::from_utf8(&desc.source)
            .map_err(|e| RhiError::CompilationFailed(format!("Invalid UTF-8 in shader source: {}", e)))?;
        
        let gl_shader = self.compile_shader(desc.stage, source)?;
        
        let handle = self.generate_handle();
        let shader = GlShaderInternal {
            gl_id: gl_shader,
            stage: desc.stage,
            source: source.to_string(),
        };
        
        self.shaders.lock().insert(handle, shader);
        Ok(handle)
    }
    
    fn create_pipeline_state(&self, desc: &PipelineStateObject) -> RhiResult<ResourceHandle> {
        let program = unsafe { self.context.create_program() }
            .map_err(|_| RhiError::InitializationFailed("Failed to create program".to_string()))?;

        // Прикрепляем шейдеры из PSO
        for &shader_handle in &desc.shaders() {
            if let Some(shader) = self.shaders.lock().get(&shader_handle) {
                unsafe {
                    self.context.attach_shader(program, shader.gl_id);
                }
            }
        }
        
        unsafe {
            self.context.link_program(program);
            
            if !self.context.get_program_link_status(program) {
                let error_log = self.context.get_program_info_log(program);
                self.context.delete_program(program);
                return Err(RhiError::CompilationFailed(error_log));
            }
        }
        
        // Создаем VAO
        let vao = unsafe { self.context.create_vertex_array() }
            .map_err(|_| RhiError::InitializationFailed("Failed to create VAO".to_string()))?;
        
        let handle = self.generate_handle();
        let pipeline = GlPipelineInternal {
            program,
            vertex_array: vao,
            topology: desc.primitive_topology,
            desc: desc.clone(),
        };
        
        self.pipelines.lock().insert(handle, pipeline);
        Ok(handle)
    }
    
    fn create_input_layout(&self, desc: &InputLayout) -> RhiResult<ResourceHandle> {
        let handle = self.generate_handle();
        Ok(handle)
    }
    
    fn create_descriptor_heap(&self, _desc: &DescriptorHeapDescription) -> RhiResult<ResourceHandle> {
        // В OpenGL нет descriptor heaps - возвращаем фиктивный handle
        Ok(ResourceHandle(0))
    }
    
 fn create_command_list(&self, cmd_type: CommandListType) -> RhiResult<Box<dyn ICommandList + Send + Sync>> {
        let device = GlDevice::from(self);
        Ok(Box::new(GlCommandList::new(self.context.clone(), Arc::new(device), cmd_type)))
    }
    
    fn create_command_queue(&self, cmd_type: CommandListType) -> RhiResult<Arc<dyn ICommandQueue>> {
        Ok(Arc::new(GlCommandQueue::new(self.context.clone(), cmd_type)))
    }
    
    fn create_fence(&self, initial_value: u64) -> RhiResult<Arc<dyn IFence>> {
        Ok(Arc::new(GlFence::new(initial_value)))
    }
    
    fn create_semaphore(&self) -> RhiResult<Arc<dyn ISemaphore>> {
        Ok(Arc::new(GlSemaphoreInternal { _private: () }))
    }
    
    fn create_swap_chain(&self, _window_handle: *mut std::ffi::c_void, width: u32, height: u32, format: TextureFormat, _vsync: bool) -> RhiResult<Arc<dyn ISwapChain>> {
        let framebuffer = unsafe { self.context.create_framebuffer() }
            .map_err(|_| RhiError::InitializationFailed("Failed to create framebuffer".to_string()))?;

        let color_texture = unsafe { self.context.create_texture() }
            .map_err(|_| RhiError::InitializationFailed("Failed to create color texture".to_string()))?;
        
        unsafe {
            self.context.bind_texture(glow::TEXTURE_2D, Some(color_texture));
            self.context.tex_storage_2d(glow::TEXTURE_2D, 1, glow::RGBA8, width as i32, height as i32);
            self.context.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            self.context.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            self.context.bind_texture(glow::TEXTURE_2D, None);
        }
        
        let swapchain = GlSwapChainInternal {
            context: self.context.clone(),
            width: Cell::new(width),
            height: Cell::new(height),
            format,
            framebuffer,
            color_texture,
            depth_texture: None,
        };

        Ok(Arc::new(swapchain))
    }
    
    fn update_buffer(&self, buffer: ResourceHandle, offset: u64, data: &[u8]) -> RhiResult<()> {
        let buffers = self.buffers.lock();
        if let Some(buf) = buffers.get(&buffer) {
            unsafe {
                self.context.bind_buffer(glow::ARRAY_BUFFER, Some(buf.gl_id));
                self.context.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, offset as i32, data);
                self.context.bind_buffer(glow::ARRAY_BUFFER, None);
            }
        }
        Ok(())
    }
    
    fn update_texture(&self, _texture: ResourceHandle, _offset_x: u32, _offset_y: u32, _offset_z: u32, _width: u32, _height: u32, _depth: u32, _data: &[u8]) -> RhiResult<()> {
        // OpenGL texture updates use different approach - texture streaming or buffer mapping
        // For now, this is a stub
        Ok(())
    }
    
    fn map_buffer(&self, _buffer: ResourceHandle) -> RhiResult<*mut u8> {
        // В OpenGL нет прямого маппинга как в DX12/Vulkan
        // Используем буферизацию через update_buffer
        Err(RhiError::Unsupported("Buffer mapping not supported in OpenGL backend".to_string()))
    }
    
    fn unmap_buffer(&self, _buffer: ResourceHandle) {
        // В OpenGL нет явного unmap - данные отправляются сразу через glBufferSubData
        // Этот метод существует для совместимости с интерфейсом IDevice
        tracing::trace!("OpenGL unmap_buffer: no-op (data already uploaded via update_buffer)");
    }
    
    fn read_back_texture(&self, texture: ResourceHandle) -> RhiResult<Vec<u8>> {
        let textures = self.textures.lock();
        if let Some(tex) = textures.get(&texture) {
            let height = if tex.desc.height > 0 { tex.desc.height } else { 1 };
            let mut data = vec![0u8; (tex.desc.width * height * 4) as usize];
            unsafe {
                self.context.bind_texture(tex.target, Some(tex.gl_id));
                // Используем get_tex_image вместо get_tex_image_u8_slice
                self.context.get_tex_image(
                    tex.target,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelPackData::Slice(&mut data),
                );
                self.context.bind_texture(tex.target, None);
            }
            return Ok(data);
        }
        Err(RhiError::InvalidResourceHandle("Texture not found".to_string()))
    }
    
    fn destroy_resource(&self, handle: ResourceHandle) {
        // Пытаемся удалить из всех коллекций
        if let Some(buf) = self.buffers.lock().remove(&handle) {
            unsafe { self.context.delete_buffer(buf.gl_id); }
        }
        if let Some(tex) = self.textures.lock().remove(&handle) {
            unsafe { self.context.delete_texture(tex.gl_id); }
        }
        if let Some(samp) = self.samplers.lock().remove(&handle) {
            unsafe { self.context.delete_sampler(samp.gl_id); }
        }
        if let Some(shader) = self.shaders.lock().remove(&handle) {
            unsafe { self.context.delete_shader(shader.gl_id); }
        }
        if let Some(pipeline) = self.pipelines.lock().remove(&handle) {
            unsafe {
                self.context.delete_vertex_array(pipeline.vertex_array);
                self.context.delete_program(pipeline.program);
            }
        }
    }
    
    fn wait_idle(&self) -> RhiResult<()> {
        unsafe { self.context.flush(); }
        Ok(())
    }
    
    fn get_memory_stats(&self) -> MemoryStats {
        // В OpenGL сложно получить точную статистику памяти
        MemoryStats {
            total_gpu_memory: u64::MAX,
            used_gpu_memory: 0,
            total_upload_memory: u64::MAX,
            used_upload_memory: 0,
            total_download_memory: u64::MAX,
            used_download_memory: 0,
        }
    }
}

// ==================== GL COMMAND LIST ====================

/// OpenGL Command List - записывает команды для выполнения
pub struct GlCommandList {
    context: Arc<Context>,
    device: Arc<GlDevice>,
    cmd_type: CommandListType,
    is_recording: bool,
    current_program: Option<u32>,
    current_vao: Option<NativeVertexArray>,
    current_framebuffer: Option<NativeFramebuffer>,
}

unsafe impl Send for GlCommandList {}
unsafe impl Sync for GlCommandList {}

impl GlCommandList {
    pub fn new(context: Arc<Context>, device: Arc<GlDevice>, cmd_type: CommandListType) -> Self {
        Self {
            context,
            device,
            cmd_type,
            is_recording: false,
            current_program: None,
            current_vao: None,
            current_framebuffer: None,
        }
    }
}

impl ICommandList for GlCommandList {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn reset(&mut self) -> RhiResult<()> {
        self.is_recording = true;
        self.current_program = None;
        self.current_vao = None;
        self.current_framebuffer = None;
        Ok(())
    }
    
    fn close(&mut self) -> RhiResult<()> {
        self.is_recording = false;
        Ok(())
    }
    
    fn begin_render_pass(&mut self, desc: &RenderPassDescription) {
        let framebuffer = unsafe { self.context.create_framebuffer() }
            .unwrap_or_else(|_| {
                tracing::error!(target: "rhi", "Failed to create framebuffer, using default");
                // SAFETY: NonZeroU32::new(1) is guaranteed to succeed as 1 != 0
                glow::NativeFramebuffer(NonZeroU32::new(1).expect("1 is non-zero"))
            });

        unsafe {
            self.context.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));

            for (i, _attachment) in desc.color_attachments.iter().enumerate() {
                let draw_buffer = glow::COLOR_ATTACHMENT0 + i as u32;
                self.context.draw_buffers(&[draw_buffer]);
            }

            self.context.viewport(0, 0, desc.width as i32, desc.height as i32);

            for (i, attachment) in desc.color_attachments.iter().enumerate() {
                if attachment.load_op == LoadOp::Clear {
                    if let Some(clear_value) = attachment.clear_value {
                        let color = match clear_value {
                            ClearValue::Color(c) => c,
                            _ => [0.0, 0.0, 0.0, 1.0],
                        };
                        self.context.clear_buffer_f32_slice(glow::COLOR, i as u32, &color);
                    }
                }
            }
            
            if let Some(ds_attachment) = &desc.depth_stencil_attachment {
                let mut clear_bits = 0;
                if ds_attachment.depth_load_op == LoadOp::Clear {
                    clear_bits |= glow::DEPTH_BUFFER_BIT;
                    if let Some(depth) = ds_attachment.depth_clear_value {
                        self.context.clear_depth_f32(depth);
                    }
                }
                if ds_attachment.stencil_load_op == LoadOp::Clear {
                    clear_bits |= glow::STENCIL_BUFFER_BIT;
                    if let Some(stencil) = ds_attachment.stencil_clear_value {
                        self.context.clear_stencil(stencil as i32);
                    }
                }
                if clear_bits != 0 {
                    self.context.clear(clear_bits);
                }
            }
        }
        
        self.current_framebuffer = Some(framebuffer);
    }
    
    fn end_render_pass(&mut self) {
        unsafe {
            self.context.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
        self.current_framebuffer = None;
    }
    
    fn set_pipeline_state(&mut self, pso: ResourceHandle) {
        self.current_program = Some(pso.0 as u32);
    }
    
    fn set_primitive_topology(&mut self, _topology: PrimitiveTopology) {
    }
    
    fn set_viewport(&mut self, viewport: &Viewport) {
        unsafe {
            self.context.viewport(
                viewport.x as i32,
                viewport.y as i32,
                viewport.width as i32,
                viewport.height as i32,
            );
        }
    }
    
    fn set_scissor_rect(&mut self, scissor: &ScissorRect) {
        unsafe {
            self.context.scissor(
                scissor.x() as i32,
                scissor.y() as i32,
                scissor.width() as i32,
                scissor.height() as i32,
            );
        }
    }
    
    fn set_blend_constants(&mut self, constants: [f32; 4]) {
        unsafe {
            self.context.blend_color(constants[0], constants[1], constants[2], constants[3]);
        }
    }
    
    fn set_stencil_reference(&mut self, reference: u8) {
        unsafe {
            self.context.stencil_func(glow::ALWAYS, reference as i32, 0xFF);
        }
    }
    
    fn bind_vertex_buffers(&mut self, start_slot: u32, buffers: &[(ResourceHandle, u64)]) {
        for (i, (buffer_handle, offset)) in buffers.iter().enumerate() {
            unsafe {
                let gl_buffer = NativeBuffer(NonZeroU32::new(buffer_handle.0 as u32).unwrap_or_else(|| {
                    tracing::warn!("Invalid buffer handle {}", buffer_handle.0);
                    // SAFETY: 1 is guaranteed to be non-zero
                    NonZeroU32::new(1).expect("1 is non-zero")
                }));
                self.context.bind_buffer(glow::ARRAY_BUFFER, Some(gl_buffer));

                self.context.vertex_attrib_pointer_f32(
                    (start_slot + i as u32) as u32,
                    4,
                    glow::FLOAT,
                    false,
                    0,
                    *offset as i32,
                );
                self.context.enable_vertex_attrib_array((start_slot + i as u32) as u32);
            }
        }
    }
    
    fn bind_index_buffer(&mut self, buffer: ResourceHandle, _offset: u64, _format: IndexFormat) {
        unsafe {
            let gl_buffer = NativeBuffer(NonZeroU32::new(buffer.0 as u32).unwrap_or_else(|| {
                tracing::warn!("Invalid index buffer handle {}", buffer.0);
                // SAFETY: 1 is guaranteed to be non-zero
                NonZeroU32::new(1).expect("1 is non-zero")
            }));
            self.context.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(gl_buffer));
        }
    }
    
    fn bind_constant_buffer(&mut self, _stage: ShaderStage, slot: u32, buffer: ResourceHandle) {
        unsafe {
            let gl_buffer = NativeBuffer(NonZeroU32::new(buffer.0 as u32).unwrap_or_else(|| {
                tracing::warn!("Invalid constant buffer handle {}", buffer.0);
                // SAFETY: 1 is guaranteed to be non-zero
                NonZeroU32::new(1).expect("1 is non-zero")
            }));
            self.context.bind_buffer_range(
                glow::UNIFORM_BUFFER,
                slot,
                Some(gl_buffer),
                0,
                256,
            );
        }
    }
    
    fn bind_shader_resource(&mut self, _stage: ShaderStage, slot: u32, _view: ResourceHandle) {
        unsafe {
            self.context.active_texture(glow::TEXTURE0 + slot);
        }
    }
    
    fn bind_sampler(&mut self, _stage: ShaderStage, slot: u32, sampler: ResourceHandle) {
        unsafe {
            let gl_sampler = NativeSampler(NonZeroU32::new(sampler.0 as u32).unwrap_or_else(|| {
                tracing::warn!("Invalid sampler handle {}", sampler.0);
                // SAFETY: 1 is guaranteed to be non-zero
                NonZeroU32::new(1).expect("1 is non-zero")
            }));
            self.context.bind_sampler(slot, Some(gl_sampler));
        }
    }
    
    fn draw(&mut self, vertex_count: u32, instance_count: u32, start_vertex: u32, _start_instance: u32) {
        if instance_count > 1 {
            unsafe {
                self.context.draw_arrays_instanced(
                    glow::TRIANGLES,
                    start_vertex as i32,
                    vertex_count as i32,
                    instance_count as i32,
                );
            }
        } else {
            unsafe {
                self.context.draw_arrays(glow::TRIANGLES, start_vertex as i32, vertex_count as i32);
            }
        }
    }
    
    fn draw_indexed(&mut self, index_count: u32, instance_count: u32, start_index: u32, _base_vertex: i32, _start_instance: u32) {
        if instance_count > 1 {
            unsafe {
                self.context.draw_elements_instanced(
                    glow::TRIANGLES,
                    index_count as i32,
                    glow::UNSIGNED_INT,
                    (start_index * 4) as i32,
                    instance_count as i32,
                );
            }
        } else {
            unsafe {
                self.context.draw_elements(
                    glow::TRIANGLES,
                    index_count as i32,
                    glow::UNSIGNED_INT,
                    (start_index * 4) as i32,
                );
            }
        }
    }
    
    fn draw_indirect(&mut self, buffer: ResourceHandle, _offset: u64, draw_count: u32) {
        unsafe {
            let gl_buffer = NativeBuffer(NonZeroU32::new(buffer.0 as u32).unwrap_or_else(|| {
                tracing::warn!("Invalid indirect buffer handle {}", buffer.0);
                // SAFETY: 1 is guaranteed to be non-zero
                NonZeroU32::new(1).expect("1 is non-zero")
            }));
            self.context.bind_buffer(glow::DRAW_INDIRECT_BUFFER, Some(gl_buffer));
            for i in 0..draw_count {
                self.context.draw_arrays_instanced_base_instance(
                    glow::TRIANGLES,
                    0,
                    0,
                    0,
                    i,
                );
            }
            self.context.bind_buffer(glow::DRAW_INDIRECT_BUFFER, None);
        }
    }
    
    fn draw_indexed_indirect(&mut self, buffer: ResourceHandle, _offset: u64, draw_count: u32) {
        unsafe {
            let gl_buffer = NativeBuffer(NonZeroU32::new(buffer.0 as u32).unwrap_or_else(|| {
                tracing::warn!("Invalid indexed indirect buffer handle {}", buffer.0);
                // SAFETY: 1 is guaranteed to be non-zero
                NonZeroU32::new(1).expect("1 is non-zero")
            }));
            self.context.bind_buffer(glow::DRAW_INDIRECT_BUFFER, Some(gl_buffer));
            for i in 0..draw_count {
                self.context.draw_elements_instanced_base_vertex_base_instance(
                    glow::TRIANGLES,
                    0,
                    glow::UNSIGNED_INT,
                    0,
                    0,
                    0,
                    i,
                );
            }
            self.context.bind_buffer(glow::DRAW_INDIRECT_BUFFER, None);
        }
    }
    
    fn dispatch(&mut self, group_count_x: u32, group_count_y: u32, group_count_z: u32) {
        unsafe {
            self.context.dispatch_compute(group_count_x, group_count_y, group_count_z);
        }
    }
    
    fn dispatch_indirect(&mut self, buffer: ResourceHandle, offset: u64) {
        unsafe {
            let gl_buffer = NativeBuffer(NonZeroU32::new(buffer.0 as u32).unwrap_or_else(|| {
                tracing::warn!("Invalid dispatch indirect buffer handle {}", buffer.0);
                // SAFETY: 1 is guaranteed to be non-zero
                NonZeroU32::new(1).expect("1 is non-zero")
            }));
            self.context.bind_buffer(glow::DISPATCH_INDIRECT_BUFFER, Some(gl_buffer));
            self.context.dispatch_compute_indirect(offset as i32);
            self.context.bind_buffer(glow::DISPATCH_INDIRECT_BUFFER, None);
        }
    }
    
    fn resource_barrier(&mut self, barriers: &[ResourceBarrier]) {
        if !barriers.is_empty() {
            unsafe {
                self.context.memory_barrier(glow::ALL_BARRIER_BITS);
            }
        }
    }
    
    fn clear_render_target(&mut self, _view: ResourceHandle, color: [f32; 4]) {
        unsafe {
            self.context.clear_buffer_f32_slice(glow::COLOR, 0, &color);
        }
    }
    
    fn clear_depth_stencil(&mut self, _view: ResourceHandle, clear_depth: Option<f32>, clear_stencil: Option<u8>) {
        let mut bits = 0;
        if let Some(depth) = clear_depth {
            bits |= glow::DEPTH_BUFFER_BIT;
            unsafe { self.context.clear_depth_f32(depth); }
        }
        if let Some(stencil) = clear_stencil {
            bits |= glow::STENCIL_BUFFER_BIT;
            unsafe { self.context.clear_stencil(stencil as i32); }
        }
        if bits != 0 {
            unsafe { self.context.clear(bits); }
        }
    }
    
    fn insert_debug_marker(&mut self, name: &str) {
        unsafe {
            if self.context.supports_debug() {
                self.context.debug_message_insert(
                    glow::DEBUG_SOURCE_APPLICATION,
                    glow::DEBUG_TYPE_MARKER,
                    0,
                    glow::DEBUG_SEVERITY_NOTIFICATION,
                    name,
                );
            }
        }
    }
    
    fn begin_debug_group(&mut self, name: &str) {
        unsafe {
            if self.context.supports_debug() {
                self.context.push_debug_group(glow::DEBUG_SOURCE_APPLICATION, 0, name);
            }
        }
    }
    
    fn end_debug_group(&mut self) {
        unsafe {
            if self.context.supports_debug() {
                self.context.pop_debug_group();
            }
        }
    }
}

// ==================== GL COMMAND QUEUE ====================

/// OpenGL Command Queue - отправляет команды на GPU
pub struct GlCommandQueue {
    context: Arc<Context>,
    cmd_type: CommandListType,
}

unsafe impl Send for GlCommandQueue {}
unsafe impl Sync for GlCommandQueue {}

impl GlCommandQueue {
    pub fn new(context: Arc<Context>, cmd_type: CommandListType) -> Self {
        Self { context, cmd_type }
    }
}

impl ICommandQueue for GlCommandQueue {
    fn submit(&self, _command_lists: &[&dyn ICommandList], _wait_semaphores: &[Arc<dyn ISemaphore>], _signal_semaphores: &[Arc<dyn ISemaphore>]) -> RhiResult<()> {
        // В OpenGL команды выполняются сразу при записи
        // Здесь просто flush
        unsafe { self.context.flush(); }
        Ok(())
    }
    
    fn present(&self, _swap_chain: &dyn ISwapChain) -> RhiResult<()> {
        // В OpenGL с презентацией работает windowing система (glutin/winit)
        Ok(())
    }
    
    fn signal(&self, fence: &dyn IFence, value: u64) -> RhiResult<()> {
        fence.set_value(value);
        Ok(())
    }
    
    fn wait(&self, fence: &dyn IFence, value: u64, _timeout_ms: u32) -> RhiResult<bool> {
        Ok(fence.get_value() >= value)
    }
}

// ==================== GL FENCE ====================

/// OpenGL Fence для синхронизации CPU-GPU
pub struct GlFence {
    value: AtomicU64,
}

unsafe impl Send for GlFence {}
unsafe impl Sync for GlFence {}

impl GlFence {
    pub fn new(initial_value: u64) -> Self {
        Self {
            value: AtomicU64::new(initial_value),
        }
    }
    
    pub fn set_value(&self, value: u64) {
        self.value.store(value, Ordering::SeqCst);
    }
}

impl IFence for GlFence {
    fn get_value(&self) -> u64 {
        self.value.load(Ordering::SeqCst)
    }

    fn set_value(&self, value: u64) {
        self.value.store(value, Ordering::SeqCst);
    }

    fn set_event_on_completion(&self, _value: u64) -> RhiResult<Arc<dyn std::any::Any + Send + Sync>> {
        // В OpenGL нет нативных event'ов как в DX12
        Ok(Arc::new(()))
    }
}

// ==================== GL SWAPCHAIN ====================

impl ISwapChain for GlSwapChainInternal {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_current_back_buffer_index(&self) -> usize {
        0 // Double buffering в OpenGL обрабатывается автоматически
    }
    
    fn get_back_buffer(&self) -> ResourceHandle {
        // Возвращаем handle для цветовой текстуры swapchain
        // В полной реализации нужно создать отдельный ResourceHandle для texture view
        ResourceHandle(1) 
    }
    
    fn get_back_buffer_texture(&self) -> ResourceHandle {
        // Возвращаем handle для самой текстуры (не view)
        // Это нужно для создания framebuffer в render pass
        ResourceHandle(2)
    }

    fn width(&self) -> u32 {
        self.width.get()
    }

    fn height(&self) -> u32 {
        self.height.get()
    }
    
    fn resize(&self, width: u32, height: u32) -> RhiResult<()> {
        self.width.set(width);
        self.height.set(height);
        
        unsafe {
            self.context.bind_texture(glow::TEXTURE_2D, Some(self.color_texture));
            self.context.tex_storage_2d(glow::TEXTURE_2D, 1, glow::RGBA8, width as i32, height as i32);
            self.context.bind_texture(glow::TEXTURE_2D, None);
        }
        
        Ok(())
    }
    
    fn present(&self) -> RhiResult<()> {
        // В OpenGL презентация происходит через swap_buffers в winit/glutin
        // Этот метод вызывается из GlCommandQueue::present или напрямую из GlContext
        // Здесь просто flush для гарантии выполнения команд
        unsafe { self.context.flush(); }
        
        // Примечание: фактический swap_buffers должен вызываться из GlContext::present()
        // где есть доступ к surface. Этот метод только гарантирует, что все команды
        // отправлены GPU.
        Ok(())
    }

    fn present_with_sync(&self, _semaphore: Option<&dyn ISemaphore>) -> RhiResult<()> {
        self.present()
    }
}

// Any support is provided by the dyn Any trait on ISwapChain
