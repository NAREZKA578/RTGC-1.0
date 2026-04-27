// Render Hardware Interface (RHI) - Universal Abstraction Layer
// Provides unified interface for Vulkan, DirectX 12, and OpenGL backends
// Designed for multi-threaded command recording and PSO-based rendering

use std::fmt;
use std::any::Any;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub use super::device::*;

/// Resource handle for GPU resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceHandle(pub u64);

impl ResourceHandle {
    pub const INVALID: Self = ResourceHandle(u64::MAX);
    pub const DEFAULT: Self = ResourceHandle(0);
    
    pub fn is_valid(&self) -> bool {
        self.0 != u64::MAX
    }
}

impl Default for ResourceHandle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Vertex attribute format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexFormat {
    Float32x2,
    Float32x3,
    Float32x4,
    Float32x2x2, // mat2
    Float32x3x3, // mat3
    Float32x4x4, // mat4
    Uint8x4Norm,
    Uint16x2Norm,
    Uint16x4Norm,
}

impl VertexFormat {
    pub fn size_bytes(&self) -> usize {
        match self {
            VertexFormat::Float32x2 => 8,
            VertexFormat::Float32x3 => 12,
            VertexFormat::Float32x4 => 16,
            VertexFormat::Float32x2x2 => 16,
            VertexFormat::Float32x3x3 => 36,
            VertexFormat::Float32x4x4 => 64,
            VertexFormat::Uint8x4Norm => 4,
            VertexFormat::Uint16x2Norm => 4,
            VertexFormat::Uint16x4Norm => 8,
        }
    }
}

/// Vertex attribute description
#[derive(Debug, Clone, Hash)]
pub struct VertexAttribute {
    pub name: String,
    pub format: VertexFormat,
    pub offset: u32,
    pub location: u32,
    pub semantic: String,
    pub buffer_slot: u32,
}

/// Input layout for vertex shader
#[derive(Debug, Clone)]
pub struct InputLayout {
    pub attributes: Vec<VertexAttribute>,
    pub stride: u32,
}

impl Default for InputLayout {
    fn default() -> Self {
        Self {
            attributes: Vec::new(),
            stride: 0,
        }
    }
}

impl InputLayout {
    pub fn new(attributes: Vec<VertexAttribute>) -> Self {
        let stride = attributes.iter().map(|a| a.format.size_bytes() as u32).sum();
        Self { attributes, stride }
    }
    
    pub fn len(&self) -> usize {
        self.attributes.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }
    
    pub fn iter(&self) -> std::slice::Iter<'_, VertexAttribute> {
        self.attributes.iter()
    }
}

/// Resource state for barrier synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceState {
    Undefined,
    Common,
    VertexBuffer,
    IndexBuffer,
    ConstantBuffer,
    ShaderResource,
    UnorderedAccess,
    RenderTarget,
    DepthWrite,
    DepthRead,
    Present,
    TransferSource,
    TransferDestination,
    GenericRead,
    CopyDest,
    GenericWrite,
}

/// Buffer type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    Vertex,
    Index,
    Constant,
    Storage,
    Indirect,
    Uniform,
}

/// Buffer description
#[derive(Debug, Clone)]
pub struct BufferDescription {
    pub buffer_type: BufferType,
    pub size: u64,
    pub usage: BufferUsage,
    pub initial_state: ResourceState,
    pub initial_data: Option<Vec<u8>>,
}

impl Default for BufferDescription {
    fn default() -> Self {
        Self {
            buffer_type: BufferType::Vertex,
            size: 0,
            usage: BufferUsage::VERTEX_BUFFER,
            initial_state: ResourceState::Common,
            initial_data: None,
        }
    }
}

bitflags::bitflags! {
    /// Buffer usage flags
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct BufferUsage: u32 {
        const VERTEX_BUFFER = 1 << 0;
        const INDEX_BUFFER = 1 << 1;
        const CONSTANT_BUFFER = 1 << 2;
        const SHADER_RESOURCE = 1 << 3;
        const UNORDERED_ACCESS = 1 << 4;
        const TRANSFER_SRC = 1 << 5;
        const TRANSFER_DST = 1 << 6;
        const STORAGE_BUFFER = 1 << 7;
        const INDIRECT_BUFFER = 1 << 8;
        const IMMUTABLE = 1 << 9;
        const DYNAMIC = 1 << 10;
        const TRANSIENT = 1 << 11;
        const UPLOAD = 1 << 12;
        const READBACK = 1 << 13;
    }
}

bitflags::bitflags! {
    /// Texture usage flags
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct TextureUsage: u32 {
        const SHADER_READ = 1 << 0;
        const SHADER_WRITE = 1 << 1;
        const RENDER_TARGET = 1 << 2;
        const DEPTH_STENCIL = 1 << 3;
        const TRANSFER_SRC = 1 << 4;
        const TRANSFER_DST = 1 << 5;
        const STORAGE = 1 << 6;
        const PRESENT = 1 << 7;
    }
}

/// Sampler filter mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    Point,
    Bilinear,
    Trilinear,
    Anisotropic,
}

/// Sampler address mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressMode {
    ClampToEdge,
    Wrap,
    Mirror,
    Border,
    MirrorOnce,
}

/// Sampler description
#[derive(Debug, Clone)]
pub struct SamplerDescription {
    pub min_filter: FilterMode,
    pub mag_filter: FilterMode,
    pub mip_filter: FilterMode,
    pub address_u: AddressMode,
    pub address_v: AddressMode,
    pub address_w: AddressMode,
    pub mip_lod_bias: f32,
    pub max_anisotropy: u32,
    pub compare_func: Option<CompareFunc>,
    pub min_lod: f32,
    pub max_lod: f32,
    pub border_color: [f32; 4],
}

impl Default for SamplerDescription {
    fn default() -> Self {
        Self {
            min_filter: FilterMode::Bilinear,
            mag_filter: FilterMode::Bilinear,
            mip_filter: FilterMode::Bilinear,
            address_u: AddressMode::Wrap,
            address_v: AddressMode::Wrap,
            address_w: AddressMode::Wrap,
            mip_lod_bias: 0.0,
            max_anisotropy: 1,
            compare_func: None,
            min_lod: 0.0,
            max_lod: f32::MAX,
            border_color: [0.0; 4],
        }
    }
}

/// Command list type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandListType {
    Direct,     // Graphics + compute + transfer
    Compute,    // Compute only
    Copy,       // Transfer only
    Graphics,   // Graphics-only (alias for Direct)
}

/// Clear value for render targets / depth buffers
#[derive(Debug, Clone, Copy)]
pub enum ClearValue {
    Color([f32; 4]),
    Depth(f32),
    DepthStencil(f32, u8),
}

/// Viewport
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

impl Viewport {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width,
            height,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }
    
    pub fn full_screen(width: u32, height: u32) -> Self {
        Self::new(width as f32, height as f32)
    }
}

/// Scissor rect
#[derive(Debug, Clone, Copy)]
pub struct ScissorRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl ScissorRect {
    /// Get x position (alias for left)
    pub fn x(&self) -> i32 {
        self.left
    }
    
    /// Get y position (alias for top)
    pub fn y(&self) -> i32 {
        self.top
    }
    
    /// Get width (right - left)
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    
    /// Get height (bottom - top)
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
    
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self { left, top, right, bottom }
    }
    
    pub fn full_screen(width: u32, height: u32) -> Self {
        Self {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        }
    }
}

/// Draw indexed indirect structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrawIndexedIndirectArgs {
    pub index_count: u32,
    pub instance_count: u32,
    pub start_index: u32,
    pub base_vertex: i32,
    pub start_instance: u32,
}

/// Draw indirect structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrawIndirectArgs {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub start_vertex: u32,
    pub start_instance: u32,
}

/// Dispatch indirect structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DispatchIndirectArgs {
    pub group_count_x: u32,
    pub group_count_y: u32,
    pub group_count_z: u32,
}

/// Resource error types
#[derive(Debug, Clone)]
pub enum RhiError {
    InitializationFailed(String),
    OutOfMemory,
    DeviceLost,
    InvalidParameter(String),
    ShaderCompilationFailed(String),
    CompilationFailed(String),
    InvalidResourceHandle(String),
    ResourceCreationFailed(String),
    QueueFull,
    Timeout,
    Unsupported(String),
    OperationFailed(String),
    InvalidResource,
    OutOfBounds,
    LockError(String),
    Other(String),
}

impl fmt::Display for RhiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RhiError::InitializationFailed(msg) => write!(f, "RHI initialization failed: {}", msg),
            RhiError::OutOfMemory => write!(f, "Out of memory"),
            RhiError::DeviceLost => write!(f, "Device lost"),
            RhiError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            RhiError::ShaderCompilationFailed(msg) => write!(f, "Shader compilation failed: {}", msg),
            RhiError::CompilationFailed(msg) => write!(f, "Compilation failed: {}", msg),
            RhiError::InvalidResourceHandle(msg) => write!(f, "Invalid resource handle: {}", msg),
            RhiError::ResourceCreationFailed(msg) => write!(f, "Resource creation failed: {}", msg),
            RhiError::QueueFull => write!(f, "Command queue full"),
            RhiError::Timeout => write!(f, "Operation timeout"),
            RhiError::Unsupported(msg) => write!(f, "Unsupported: {}", msg),
            RhiError::OperationFailed(msg) => write!(f, "Operation failed: {}", msg),
            RhiError::InvalidResource => write!(f, "Invalid resource"),
            RhiError::OutOfBounds => write!(f, "Out of bounds"),
            RhiError::LockError(msg) => write!(f, "Lock error: {}", msg),
            RhiError::Other(msg) => write!(f, "Other: {}", msg),
        }
    }
}

impl std::error::Error for RhiError {}

impl From<String> for RhiError {
    fn from(msg: String) -> Self {
        RhiError::InitializationFailed(msg)
    }
}

impl From<&str> for RhiError {
    fn from(msg: &str) -> Self {
        RhiError::Other(msg.to_string())
    }
}

pub type RhiResult<T> = Result<T, RhiError>;

// Type aliases for backwards compatibility
pub type TextureDesc = TextureDescription;
pub type SamplerDesc = SamplerDescription;
pub type PipelineDesc = PipelineStateObject;
pub type BufferDesc = BufferDescription;

/// Shader stage enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
    Geometry,
    TessellationControl,
    TessellationEvaluation,
}

/// Shader description
#[derive(Debug, Clone)]
pub struct ShaderDescription {
    pub source: Vec<u8>,
    pub stage: ShaderStage,
    pub entry_point: String,
}

impl ShaderDescription {
    pub fn from_source(source: &str, stage: ShaderStage, entry_point: &str) -> Self {
        Self {
            source: source.as_bytes().to_vec(),
            stage,
            entry_point: entry_point.to_string(),
        }
    }
}

impl Default for ShaderDescription {
    fn default() -> Self {
        Self {
            source: Vec::new(),
            stage: ShaderStage::Vertex,
            entry_point: "main".to_string(),
        }
    }
}

/// Texture dimension
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureDimension {
    D1,
    D2,
    D3,
    Cube,
    D1Array,
    D2Array,
    CubeArray,
}

/// Texture type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureType {
    Texture1D,
    Texture2D,
    Texture3D,
    TextureCube,
    Texture1DArray,
    Texture2DArray,
    TextureCubeArray,
}

/// Texture format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    Unknown,
    R8Unorm,
    R8Uint,
    R16Float,
    R16Uint,
    R32Float,
    R32Uint,
    Rg8Unorm,
    Rg16Float,
    Rg32Float,
    Rgba8Unorm,
    Rgba8Uint,
    Rgba8Snorm,
    Rgba16Float,
    Rgba32Float,
    Bgra8Unorm,
    Depth16Unorm,
    Depth24Plus,
    Depth32Float,
    Stencil8,
    Depth24PlusStencil8,
    Depth32FloatStencil8,
    BC1RgbaUnorm,
    BC3RgbaUnorm,
    BC7RgbaUnorm,
}

/// Texture description
#[derive(Debug, Clone)]
pub struct TextureDescription {
    pub texture_type: TextureType,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mip_levels: u32,
    pub array_size: u32,
    pub usage: TextureUsage,
    pub initial_state: ResourceState,
    pub dimension: TextureDimension,
    pub depth_or_array_layers: u32,
}

impl Default for TextureDescription {
    fn default() -> Self {
        Self {
            texture_type: TextureType::Texture2D,
            format: TextureFormat::Rgba8Unorm,
            width: 1,
            height: 1,
            depth: 1,
            mip_levels: 1,
            array_size: 1,
            usage: TextureUsage::SHADER_READ,
            initial_state: ResourceState::Common,
            dimension: TextureDimension::D2,
            depth_or_array_layers: 1,
        }
    }
}

/// Compare function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompareFunc {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

/// Cull mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CullMode {
    None,
    Front,
    Back,
    FrontAndBack,
}

/// Fill mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FillMode {
    Solid,
    Point,
    Wireframe,
}

/// Front face orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrontFace {
    Clockwise,
    CounterClockwise,
}

/// Stencil operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    IncrClamp,
    DecrClamp,
    Invert,
    IncrWrap,
    DecrWrap,
}

/// Blend mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    Zero,
    One,
    SrcColor,
    InvSrcColor,
    SrcAlpha,
    InvSrcAlpha,
    DstAlpha,
    InvDstAlpha,
    DstColor,
    InvDstColor,
    SrcAlphaSaturate,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Zero
    }
}

/// Blend operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendOp {
    Add,
    Subtract,
    RevSubtract,
    Min,
    Max,
}

/// Stencil face state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StencilFaceState {
    pub stencil_fail_op: StencilOp,
    pub stencil_depth_fail_op: StencilOp,
    pub stencil_pass_op: StencilOp,
    pub stencil_func: CompareFunc,
}

impl Default for StencilFaceState {
    fn default() -> Self {
        Self {
            stencil_fail_op: StencilOp::Keep,
            stencil_depth_fail_op: StencilOp::Keep,
            stencil_pass_op: StencilOp::Keep,
            stencil_func: CompareFunc::Always,
        }
    }
}

/// Stencil state description
#[derive(Debug, Clone)]
pub struct StencilState {
    pub enable: bool,
    pub reference: u8,
    pub read_mask: u8,
    pub write_mask: u8,
    pub front: StencilOpDesc,
    pub back: StencilOpDesc,
}

impl Default for StencilState {
    fn default() -> Self {
        Self {
            enable: false,
            reference: 0,
            read_mask: 0xFF,
            write_mask: 0xFF,
            front: StencilOpDesc::default(),
            back: StencilOpDesc::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StencilOpDesc {
    pub fail_op: StencilOp,
    pub depth_fail_op: StencilOp,
    pub pass_op: StencilOp,
    pub func: CompareFunc,
}

impl Default for StencilOpDesc {
    fn default() -> Self {
        Self {
            fail_op: StencilOp::Keep,
            depth_fail_op: StencilOp::Keep,
            pass_op: StencilOp::Keep,
            func: CompareFunc::Always,
        }
    }
}

/// Rasterizer state description
#[derive(Debug, Clone, PartialEq)]
pub struct RasterizerState {
    pub fill_mode: FillMode,
    pub cull_mode: CullMode,
    pub front_face: FrontFace,
    pub front_counter_clockwise: bool,
    pub depth_bias: f32,
    pub depth_bias_clamp: f32,
    pub slope_scaled_depth_bias: f32,
    pub depth_clip_enable: bool,
    pub scissor_enable: bool,
    pub multisample_enable: bool,
    pub antialiased_line_enable: bool,
}

impl Default for RasterizerState {
    fn default() -> Self {
        Self {
            fill_mode: FillMode::Solid,
            cull_mode: CullMode::Back,
            front_face: FrontFace::CounterClockwise,
            front_counter_clockwise: true,
            depth_bias: 0.0,
            depth_bias_clamp: 0.0,
            slope_scaled_depth_bias: 0.0,
            depth_clip_enable: true,
            scissor_enable: false,
            multisample_enable: false,
            antialiased_line_enable: false,
        }
    }
}

impl Hash for RasterizerState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.fill_mode.hash(state);
        self.cull_mode.hash(state);
        self.front_face.hash(state);
        self.front_counter_clockwise.hash(state);
        self.depth_bias.to_bits().hash(state);
        self.depth_bias_clamp.to_bits().hash(state);
        self.slope_scaled_depth_bias.to_bits().hash(state);
        self.depth_clip_enable.hash(state);
        self.scissor_enable.hash(state);
        self.multisample_enable.hash(state);
        self.antialiased_line_enable.hash(state);
    }
}

/// Depth state description
#[derive(Debug, Clone, PartialEq)]
pub struct DepthState {
    pub enabled: bool,
    pub write_enabled: bool,
    pub compare_func: CompareFunc,
}

impl Default for DepthState {
    fn default() -> Self {
        Self {
            enabled: true,
            write_enabled: true,
            compare_func: CompareFunc::Less,
        }
    }
}

impl Hash for DepthState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.enabled.hash(state);
        self.write_enabled.hash(state);
        self.compare_func.hash(state);
    }
}

/// Color blend state description
#[derive(Debug, Clone, PartialEq)]
pub struct ColorBlendState {
    pub enabled: bool,
    pub logic_op_enable: bool,
    pub src_blend: BlendMode,
    pub dst_blend: BlendMode,
    pub blend_op: BlendOp,
    pub src_blend_alpha: BlendMode,
    pub dst_blend_alpha: BlendMode,
    pub blend_op_alpha: BlendOp,
    pub logic_op: BlendOp,
    pub render_target_write_mask: u8,
}

impl Hash for ColorBlendState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.enabled.hash(state);
        self.logic_op_enable.hash(state);
        self.src_blend.hash(state);
        self.dst_blend.hash(state);
        self.blend_op.hash(state);
        self.src_blend_alpha.hash(state);
        self.dst_blend_alpha.hash(state);
        self.blend_op_alpha.hash(state);
        self.logic_op.hash(state);
        self.render_target_write_mask.hash(state);
    }
}

impl Default for ColorBlendState {
    fn default() -> Self {
        Self {
            enabled: true,
            logic_op_enable: false,
            src_blend: BlendMode::SrcAlpha,
            dst_blend: BlendMode::InvSrcAlpha,
            blend_op: BlendOp::Add,
            src_blend_alpha: BlendMode::One,
            dst_blend_alpha: BlendMode::InvSrcAlpha,
            blend_op_alpha: BlendOp::Add,
            logic_op: BlendOp::Add,
            render_target_write_mask: 0xF,
        }
    }
}

/// Primitive topology for draw calls
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
    TriangleFan,
    LineListWithAdjacency,
    LineStripWithAdjacency,
    TriangleListWithAdjacency,
    TriangleStripWithAdjacency,
    PatchList(u8),
}

impl PrimitiveTopology {
    pub fn as_u32(&self) -> u32 {
        match self {
            PrimitiveTopology::PointList => 0,
            PrimitiveTopology::LineList => 1,
            PrimitiveTopology::LineStrip => 2,
            PrimitiveTopology::TriangleList => 3,
            PrimitiveTopology::TriangleStrip => 4,
            PrimitiveTopology::TriangleFan => 5,
            PrimitiveTopology::LineListWithAdjacency => 6,
            PrimitiveTopology::LineStripWithAdjacency => 7,
            PrimitiveTopology::TriangleListWithAdjacency => 8,
            PrimitiveTopology::TriangleStripWithAdjacency => 9,
            PrimitiveTopology::PatchList(_) => 10,
        }
    }
}

/// Pipeline state object description
#[derive(Debug, Clone)]
pub struct PipelineStateObject {
    pub vertex_shader: ResourceHandle,
    pub fragment_shader: ResourceHandle,
    pub compute_shader: Option<ResourceHandle>,
    pub geometry_shader: Option<ResourceHandle>,
    pub hull_shader: Option<ResourceHandle>,
    pub domain_shader: Option<ResourceHandle>,
    pub input_layout: InputLayout,
    pub primitive_topology: PrimitiveTopology,
    pub rasterizer_state: RasterizerState,
    pub depth_state: DepthState,
    pub blend_state: ColorBlendState,
    pub stencil_state: StencilState,
    pub color_blend_states: Vec<ColorBlendState>,
    pub num_render_targets: u32,
    pub render_target_formats: [TextureFormat; 8],
    pub depth_stencil_format: TextureFormat,
    pub sample_count: u32,
    pub sample_quality: u32,
}

impl PipelineStateObject {
    pub fn shaders(&self) -> Vec<ResourceHandle> {
        let mut shaders = vec![self.vertex_shader, self.fragment_shader];
        if let Some(cs) = self.compute_shader {
            shaders.push(cs);
        }
        if let Some(gs) = self.geometry_shader {
            shaders.push(gs);
        }
        if let Some(hs) = self.hull_shader {
            shaders.push(hs);
        }
        if let Some(ds) = self.domain_shader {
            shaders.push(ds);
        }
        shaders
    }
}

impl Default for PipelineStateObject {
    fn default() -> Self {
        Self {
            vertex_shader: ResourceHandle::INVALID,
            fragment_shader: ResourceHandle::INVALID,
            compute_shader: None,
            geometry_shader: None,
            hull_shader: None,
            domain_shader: None,
            input_layout: InputLayout::default(),
            primitive_topology: PrimitiveTopology::TriangleList,
            rasterizer_state: RasterizerState::default(),
            depth_state: DepthState::default(),
            blend_state: ColorBlendState::default(),
            stencil_state: StencilState::default(),
            color_blend_states: vec![ColorBlendState::default()],
            num_render_targets: 1,
            render_target_formats: [TextureFormat::Rgba8Unorm; 8],
            depth_stencil_format: TextureFormat::Depth32Float,
            sample_count: 1,
            sample_quality: 0,
        }
    }
}

/// 4-component color (RGBA)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color4f {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color4f {
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn black() -> Self {
        Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
    }

    pub fn white() -> Self {
        Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }
    }
}

impl Default for Color4f {
    fn default() -> Self {
        Self::WHITE
    }
}

/// 2D rectangle
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect2D {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect2D {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
}

/// Index format for indexed rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    Uint16,
    Uint32,
}

/// Blend state for render pipeline
pub type BlendState = ColorBlendState;

/// Depth stencil state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepthStencilState {
    pub depth_test_enabled: bool,
    pub depth_write_enabled: bool,
    pub depth_compare_func: CompareFunc,
    pub stencil_test_enabled: bool,
    pub stencil_front: StencilFaceState,
    pub stencil_back: StencilFaceState,
    pub stencil_read_mask: u8,
    pub stencil_write_mask: u8,
}

impl Default for DepthStencilState {
    fn default() -> Self {
        Self {
            depth_test_enabled: true,
            depth_write_enabled: true,
            depth_compare_func: CompareFunc::Less,
            stencil_test_enabled: false,
            stencil_front: StencilFaceState::default(),
            stencil_back: StencilFaceState::default(),
            stencil_read_mask: 0xFF,
            stencil_write_mask: 0xFF,
        }
    }
}

// Re-export IDevice and ICommandList as RhiDevice and RhiCommandList for backwards compatibility
pub use super::device::IDevice as RhiDevice;
pub use super::device::ICommandList as RhiCommandList;
pub use super::device::IndexFormat;

// Pipeline state types for DX11 pipeline_dx11.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Pipeline creation failed: {0}")]
    CreationFailed(String),
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

pub trait IPipelineState: Any + Send + Sync {
    fn bind(&self, context: &mut dyn Any) -> Result<(), PipelineError>;
    fn set_primitive_topology(&mut self, topology: PrimitiveTopology);
    fn set_blend_constants(&mut self, factors: [f32; 4]);
    fn set_stencil_reference(&mut self, reference: u32);
}

pub trait IShader: Any + Send + Sync {
    fn get_name(&self) -> &str;
    fn get_stage(&self) -> ShaderStage;
    fn get_bytecode(&self) -> &[u8];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum InputElementFormat {
    #[default]
    Float32x2,
    Float32x3,
    Float32x4,
    Float16x2,
    Float16x4,
    UInt8x4,
    Int8x4,
    UInt16x2,
    UInt16x4,
    Int16x2,
    Int16x4,
    UInt32,
    UInt32x2,
    UInt32x4,
    Int32,
    Int32x2,
    Int32x4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputElementSemantic {
    Position,
    Normal,
    Tangent,
    Binormal,
    Color(u32),
    TexCoord(u32),
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendFactor {
    #[default]
    Zero,
    One,
    SrcColor,
    InvSrcColor,
    SrcAlpha,
    InvSrcAlpha,
    DstAlpha,
    InvDstAlpha,
    DstColor,
    InvDstColor,
    SrcAlphaSat,
    BlendFactor,
    InvBlendFactor,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ColorWriteMask: u8 {
        const RED = 1 << 0;
        const GREEN = 1 << 1;
        const BLUE = 1 << 2;
        const ALPHA = 1 << 3;
        const ALL = Self::RED.bits() | Self::GREEN.bits() | Self::BLUE.bits() | Self::ALPHA.bits();
    }
}

/// RHI Buffer trait alias
pub trait RhiBuffer: Send + Sync {}

/// RHI Texture trait alias
pub trait RhiTexture: Send + Sync {}

/// RHI Sampler trait alias
pub trait RhiSampler: Send + Sync {}

/// RHI Pipeline trait alias
pub trait RhiPipeline: Send + Sync {}
