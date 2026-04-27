// RHI Resource Manager
// Manages GPU resources (buffers, textures, samplers, pipelines) with handles
// Provides thread-safe access and automatic cleanup

use crate::graphics::rhi::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::fmt;

/// Resource ID for internal tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u64);

impl ResourceId {
    pub const INVALID: Self = ResourceId(0);
    
    pub fn new(id: u64) -> Self {
        ResourceId(id)
    }
}

/// Internal resource wrapper with type information
#[derive(Clone)]
pub enum ManagedResource {
    Buffer(BufferHandle),
    Texture(TextureHandle),
    Sampler(SamplerHandle),
    Pipeline(PipelineHandle),
    Shader(ShaderHandle),
    Swapchain(SwapchainHandle),
}

/// Buffer handle with metadata
#[derive(Clone)]
pub struct BufferHandle {
    pub handle: ResourceHandle,
    pub size: u64,
    pub buffer_type: BufferType,
    pub state: ResourceState,
    #[cfg(target_os = "windows")]
    pub dx12_resource: Option<windows::Win32::Graphics::Direct3D12::ID3D12Resource>,
    #[cfg(not(target_os = "windows"))]
    pub dx12_resource: Option<()>,
    pub vulkan_buffer: Option<u64>, // Placeholder for ash::Buffer
    pub vulkan_allocation: Option<u64>, // For gpu-alloc or similar
    // OpenGL resources
    pub gl_buffer: Option<u32>, // glow::NativeBuffer as u32
}

impl fmt::Debug for BufferHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BufferHandle")
            .field("handle", &self.handle)
            .field("size", &self.size)
            .field("buffer_type", &self.buffer_type)
            .field("state", &self.state)
            .finish()
    }
}

/// Texture handle with metadata
#[derive(Clone)]
pub struct TextureHandle {
    pub handle: ResourceHandle,
    pub desc: TextureDescription,
    #[cfg(target_os = "windows")]
    pub dx12_resource: Option<windows::Win32::Graphics::Direct3D12::ID3D12Resource>,
    #[cfg(not(target_os = "windows"))]
    pub dx12_resource: Option<()>,
    pub dx12_srv_handle: Option<u64>, // D3D12_CPU_DESCRIPTOR_HANDLE as u64
    pub dx12_rtv_handle: Option<u64>,
    pub dx12_dsv_handle: Option<u64>,
    pub vulkan_image: Option<u64>, // Placeholder for ash::Image
    pub vulkan_view: Option<u64>,  // ImageView
    // OpenGL resources
    pub gl_texture: Option<u32>, // glow::NativeTexture as u32
    pub gl_target: Option<u32>,  // GL texture target
    pub gl_framebuffer: Option<u32>, // For render targets
}

impl fmt::Debug for TextureHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextureHandle")
            .field("handle", &self.handle)
            .field("desc", &self.desc)
            .finish()
    }
}

/// Sampler handle
#[derive(Clone, Debug)]
pub struct SamplerHandle {
    pub handle: ResourceHandle,
    pub desc: SamplerDescription,
    #[cfg(target_os = "windows")]
    pub dx12_handle: Option<u64>, // Descriptor handle
    #[cfg(not(target_os = "windows"))]
    pub dx12_handle: Option<u64>,
    pub vulkan_sampler: Option<u64>,
    // OpenGL resources
    pub gl_sampler: Option<u32>, // glow::NativeSampler as u32
}

/// Pipeline handle
#[derive(Clone, Debug)]
pub struct PipelineHandle {
    pub handle: ResourceHandle,
    pub desc: PipelineStateObject,
    #[cfg(target_os = "windows")]
    pub dx12_pso: Option<windows::Win32::Graphics::Direct3D12::ID3D12PipelineState>,
    #[cfg(not(target_os = "windows"))]
    pub dx12_pso: Option<()>,
    pub vulkan_pipeline: Option<u64>,
    pub vulkan_layout: Option<u64>,
    // OpenGL resources
    pub gl_program: Option<u32>, // OpenGL program ID
    pub gl_vao: Option<u32>,     // Vertex Array Object
}

/// Shader handle
#[derive(Clone, Debug)]
pub struct ShaderHandle {
    pub handle: ResourceHandle,
    pub stage: ShaderStage,
    pub entry_point: String,
    #[cfg(target_os = "windows")]
    pub dx12_bytecode: Vec<u8>, // DXIL or HLSL bytecode
    #[cfg(not(target_os = "windows"))]
    pub dx12_bytecode: Vec<u8>,
    pub spirv_bytecode: Vec<u8>,
    // OpenGL resources
    pub gl_shader: Option<u32>, // OpenGL shader ID
}

/// Swapchain handle
#[derive(Clone, Debug)]
pub struct SwapchainHandle {
    pub handle: ResourceHandle,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub back_buffer_count: u32,
    #[cfg(target_os = "windows")]
    pub dx12_swapchain: Option<windows::Win32::Graphics::Dxgi::IDXGISwapChain3>,
    #[cfg(not(target_os = "windows"))]
    pub dx12_swapchain: Option<()>,
    pub dx12_rtv_handles: Vec<u64>, // RTV descriptors for each back buffer
    pub vulkan_swapchain: Option<u64>,
    // OpenGL resources
    pub gl_framebuffer: Option<u32>, // Default framebuffer ID (usually 0)
    pub gl_color_texture: Option<u32>, // Color render target texture
    pub gl_depth_texture: Option<u32>, // Depth render target texture
}

/// Resource manager for RHI resources
pub struct ResourceManager {
    next_id: RwLock<u64>,
    buffers: RwLock<HashMap<ResourceHandle, BufferHandle>>,
    textures: RwLock<HashMap<ResourceHandle, TextureHandle>>,
    samplers: RwLock<HashMap<ResourceHandle, SamplerHandle>>,
    pipelines: RwLock<HashMap<ResourceHandle, PipelineHandle>>,
    shaders: RwLock<HashMap<ResourceHandle, ShaderHandle>>,
    swapchains: RwLock<HashMap<ResourceHandle, SwapchainHandle>>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self {
            next_id: RwLock::new(1),
            buffers: RwLock::new(HashMap::new()),
            textures: RwLock::new(HashMap::new()),
            samplers: RwLock::new(HashMap::new()),
            pipelines: RwLock::new(HashMap::new()),
            shaders: RwLock::new(HashMap::new()),
            swapchains: RwLock::new(HashMap::new()),
        }
    }
    
    /// Generate a new unique resource handle
    fn generate_handle(&self) -> ResourceHandle {
        let mut next_id = self.next_id.write();
        let handle = ResourceHandle(*next_id);
        *next_id += 1;
        handle
    }
    
    // ==================== BUFFER MANAGEMENT ====================
    
    /// Register a buffer resource
    pub fn register_buffer(&self, mut buffer: BufferHandle) -> ResourceHandle {
        let handle = self.generate_handle();
        buffer.handle = handle;
        
        let mut buffers = self.buffers.write();
        buffers.insert(handle, buffer);
        
        handle
    }
    
    /// Get a buffer by handle
    pub fn get_buffer(&self, handle: ResourceHandle) -> Option<BufferHandle> {
        let buffers = self.buffers.read();
        buffers.get(&handle).cloned()
    }
    
    /// Update buffer state
    pub fn update_buffer_state(&self, handle: ResourceHandle, state: ResourceState) {
        let mut buffers = self.buffers.write();
        if let Some(buffer) = buffers.get_mut(&handle) {
            buffer.state = state;
        }
    }
    
    /// Remove a buffer
    pub fn remove_buffer(&self, handle: ResourceHandle) -> Option<BufferHandle> {
        let mut buffers = self.buffers.write();
        buffers.remove(&handle)
    }
    
    // ==================== TEXTURE MANAGEMENT ====================
    
    /// Register a texture resource
    pub fn register_texture(&self, mut texture: TextureHandle) -> ResourceHandle {
        let handle = self.generate_handle();
        texture.handle = handle;
        
        let mut textures = self.textures.write();
        textures.insert(handle, texture);
        
        handle
    }
    
    /// Get a texture by handle
    pub fn get_texture(&self, handle: ResourceHandle) -> Option<TextureHandle> {
        let textures = self.textures.read();
        textures.get(&handle).cloned()
    }
    
    /// Set texture SRV handle (DX12)
    pub fn set_texture_srv(&self, handle: ResourceHandle, srv_handle: u64) {
        let mut textures = self.textures.write();
        if let Some(texture) = textures.get_mut(&handle) {
            texture.dx12_srv_handle = Some(srv_handle);
        }
    }
    
    /// Set texture RTV handle (DX12)
    pub fn set_texture_rtv(&self, handle: ResourceHandle, rtv_handle: u64) {
        let mut textures = self.textures.write();
        if let Some(texture) = textures.get_mut(&handle) {
            texture.dx12_rtv_handle = Some(rtv_handle);
        }
    }
    
    /// Set texture DSV handle (DX12)
    pub fn set_texture_dsv(&self, handle: ResourceHandle, dsv_handle: u64) {
        let mut textures = self.textures.write();
        if let Some(texture) = textures.get_mut(&handle) {
            texture.dx12_dsv_handle = Some(dsv_handle);
        }
    }
    
    /// Remove a texture
    pub fn remove_texture(&self, handle: ResourceHandle) -> Option<TextureHandle> {
        let mut textures = self.textures.write();
        textures.remove(&handle)
    }
    
    // ==================== SAMPLER MANAGEMENT ====================
    
    /// Register a sampler resource
    pub fn register_sampler(&self, mut sampler: SamplerHandle) -> ResourceHandle {
        let handle = self.generate_handle();
        sampler.handle = handle;
        
        let mut samplers = self.samplers.write();
        samplers.insert(handle, sampler);
        
        handle
    }
    
    /// Get a sampler by handle
    pub fn get_sampler(&self, handle: ResourceHandle) -> Option<SamplerHandle> {
        let samplers = self.samplers.read();
        samplers.get(&handle).cloned()
    }
    
    /// Remove a sampler
    pub fn remove_sampler(&self, handle: ResourceHandle) -> Option<SamplerHandle> {
        let mut samplers = self.samplers.write();
        samplers.remove(&handle)
    }
    
    // ==================== PIPELINE MANAGEMENT ====================
    
    /// Register a pipeline state object
    pub fn register_pipeline(&self, mut pipeline: PipelineHandle) -> ResourceHandle {
        let handle = self.generate_handle();
        pipeline.handle = handle;
        
        let mut pipelines = self.pipelines.write();
        pipelines.insert(handle, pipeline);
        
        handle
    }
    
    /// Get a pipeline by handle
    pub fn get_pipeline(&self, handle: ResourceHandle) -> Option<PipelineHandle> {
        let pipelines = self.pipelines.read();
        pipelines.get(&handle).cloned()
    }
    
    /// Remove a pipeline
    pub fn remove_pipeline(&self, handle: ResourceHandle) -> Option<PipelineHandle> {
        let mut pipelines = self.pipelines.write();
        pipelines.remove(&handle)
    }
    
    // ==================== SHADER MANAGEMENT ====================
    
    /// Register a shader resource
    pub fn register_shader(&self, mut shader: ShaderHandle) -> ResourceHandle {
        let handle = self.generate_handle();
        shader.handle = handle;
        
        let mut shaders = self.shaders.write();
        shaders.insert(handle, shader);
        
        handle
    }
    
    /// Get a shader by handle
    pub fn get_shader(&self, handle: ResourceHandle) -> Option<ShaderHandle> {
        let shaders = self.shaders.read();
        shaders.get(&handle).cloned()
    }
    
    /// Remove a shader
    pub fn remove_shader(&self, handle: ResourceHandle) -> Option<ShaderHandle> {
        let mut shaders = self.shaders.write();
        shaders.remove(&handle)
    }
    
    // ==================== SWAPCHAIN MANAGEMENT ====================
    
    /// Register a swapchain
    pub fn register_swapchain(&self, mut swapchain: SwapchainHandle) -> ResourceHandle {
        let handle = self.generate_handle();
        swapchain.handle = handle;
        
        let mut swapchains = self.swapchains.write();
        swapchains.insert(handle, swapchain);
        
        handle
    }
    
    /// Get a swapchain by handle
    pub fn get_swapchain(&self, handle: ResourceHandle) -> Option<SwapchainHandle> {
        let swapchains = self.swapchains.read();
        swapchains.get(&handle).cloned()
    }
    
    /// Set RTV handles for swapchain back buffers
    pub fn set_swapchain_rtvs(&self, handle: ResourceHandle, rtv_handles: Vec<u64>) {
        let mut swapchains = self.swapchains.write();
        if let Some(swapchain) = swapchains.get_mut(&handle) {
            swapchain.dx12_rtv_handles = rtv_handles;
        }
    }
    
    /// Remove a swapchain
    pub fn remove_swapchain(&self, handle: ResourceHandle) -> Option<SwapchainHandle> {
        let mut swapchains = self.swapchains.write();
        swapchains.remove(&handle)
    }
    
    // ==================== UTILITY METHODS ====================
    
    /// Get resource count by type
    pub fn get_buffer_count(&self) -> usize {
        self.buffers.read().len()
    }
    
    pub fn get_texture_count(&self) -> usize {
        self.textures.read().len()
    }
    
    pub fn get_pipeline_count(&self) -> usize {
        self.pipelines.read().len()
    }
    
    /// Clear all resources (for cleanup)
    pub fn clear(&self) {
        self.buffers.write().clear();
        self.textures.write().clear();
        self.samplers.write().clear();
        self.pipelines.write().clear();
        self.shaders.write().clear();
        self.swapchains.write().clear();
    }
}

// Thread-safe wrapper for sharing across command lists
pub type SharedResourceManager = Arc<ResourceManager>;
