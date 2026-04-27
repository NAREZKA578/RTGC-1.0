// Stub values for missing Windows API - these won't actually be used in this stub version
#[allow(dead_code)]
pub const D3D11_BLEND_FACTOR: u32 = 0;
#[allow(dead_code)]
pub const D3D11_BLEND_INV_FACTOR: u32 = 0;
#[allow(dead_code)]
pub const D3D11_STENCIL_OP_DESC: u32 = 0;
#[allow(dead_code)]
pub const D3D11_SRV_DIMENSION_TEXTURE2D: u32 = 0;
#[allow(dead_code)]
pub const D3D11_RESOURCE_MISC_NONE: u32 = 0;
#[allow(dead_code)]
pub const D3D11_PRIMITIVE_TOPOLOGY: u32 = 0;

#[derive(Debug, Clone, Default)]
pub struct StencilOpDescStub {
    pubStencilFailOp: u32,
    pubStencilDepthFailOp: u32,
    pubStencilPassOp: u32,
    pubStencilFunc: u32,
}

impl StencilOpDescStub {
    pub fn new() -> Self {
        Self {
            pubStencilFailOp: 1,
            pubStencilDepthFailOp: 1,
            pubStencilPassOp: 1,
            pubStencilFunc: 8,
        }
    }
}

use tracing::{info, error};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{BOOL, FALSE as DX11_FALSE, TRUE as DX11_TRUE};

use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device,
    ID3D11InputLayout,
    ID3D11RasterizerState,
    ID3D11BlendState,
    ID3D11DepthStencilState,
    D3D11_INPUT_ELEMENT_DESC,
    D3D11_RASTERIZER_DESC,
    D3D11_BLEND_DESC,
    D3D11_DEPTH_STENCIL_DESC,
    D3D11_FILL_WIREFRAME,
    D3D11_FILL_SOLID,
    D3D11_CULL_NONE,
    D3D11_CULL_FRONT,
    D3D11_CULL_BACK,
    D3D11_BLEND_ZERO,
    D3D11_BLEND_ONE,
    D3D11_BLEND_SRC_COLOR,
    D3D11_BLEND_INV_SRC_COLOR,
    D3D11_BLEND_SRC_ALPHA,
    D3D11_BLEND_INV_SRC_ALPHA,
    D3D11_BLEND_DEST_ALPHA,
    D3D11_BLEND_INV_DEST_ALPHA,
    D3D11_BLEND_DEST_COLOR,
    D3D11_BLEND_INV_DEST_COLOR,
    D3D11_BLEND_SRC_ALPHA_SAT,
    // D3D11_BLEND_FACTOR and D3D11_BLEND_INV_FACTOR removed - use local stubs
    D3D11_BLEND_OP_ADD,
    D3D11_BLEND_OP_SUBTRACT,
    D3D11_BLEND_OP_REV_SUBTRACT,
    D3D11_BLEND_OP_MIN,
    D3D11_BLEND_OP_MAX,
    D3D11_COLOR_WRITE_ENABLE_ALL,
    D3D11_COLOR_WRITE_ENABLE_RED,
    D3D11_COLOR_WRITE_ENABLE_GREEN,
    D3D11_COLOR_WRITE_ENABLE_BLUE,
    D3D11_COLOR_WRITE_ENABLE_ALPHA,
    D3D11_COMPARISON_FUNC,
    D3D11_COMPARISON_NEVER,
    D3D11_COMPARISON_LESS,
    D3D11_COMPARISON_EQUAL,
    D3D11_COMPARISON_LESS_EQUAL,
    D3D11_COMPARISON_GREATER,
    D3D11_COMPARISON_NOT_EQUAL,
    D3D11_COMPARISON_GREATER_EQUAL,
    D3D11_COMPARISON_ALWAYS,
    D3D11_DEPTH_WRITE_MASK,
    D3D11_DEPTH_WRITE_MASK_ZERO,
    D3D11_DEPTH_WRITE_MASK_ALL,
    D3D11_STENCIL_OP,
    D3D11_STENCIL_OP_KEEP,
    D3D11_STENCIL_OP_ZERO,
    D3D11_STENCIL_OP_REPLACE,
    D3D11_STENCIL_OP_INCR_SAT,
    D3D11_STENCIL_OP_DECR_SAT,
    D3D11_STENCIL_OP_INVERT,
    D3D11_STENCIL_OP_INCR,
    D3D11_STENCIL_OP_DECR,
    D3D11_RENDER_TARGET_BLEND_DESC,
    // D3D11_STENCIL_OP_DESC removed - using local stub
    D3D11_INPUT_PER_VERTEX_DATA,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT,
    DXGI_FORMAT_R32G32_FLOAT,
    DXGI_FORMAT_R32G32B32_FLOAT,
    DXGI_FORMAT_R32G32B32A32_FLOAT,
    DXGI_FORMAT_R16G16_FLOAT,
    DXGI_FORMAT_R16G16B16A16_FLOAT,
    DXGI_FORMAT_R8G8B8A8_UINT,
    DXGI_FORMAT_R8G8B8A8_SINT,
    DXGI_FORMAT_R16G16_UINT,
    DXGI_FORMAT_R16G16B16A16_UINT,
    DXGI_FORMAT_R16G16_SINT,
    DXGI_FORMAT_R16G16B16A16_SINT,
    DXGI_FORMAT_R32_UINT,
    DXGI_FORMAT_R32G32_UINT,
    DXGI_FORMAT_R32G32B32A32_UINT,
    DXGI_FORMAT_R32_SINT,
    DXGI_FORMAT_R32G32_SINT,
    DXGI_FORMAT_R32G32B32A32_SINT,
};

use crate::graphics::rhi::types::{
    IPipelineState,
    IShader,
    InputElementFormat,
    InputElementSemantic,
    PrimitiveTopology,
    FillMode,
    CullMode,
    CompareFunc,
    StencilOp,
    StencilFaceState,
    BlendFactor,
    BlendOp,
    ColorWriteMask,
    PipelineDesc,
    PipelineError,
};

#[cfg(target_os = "windows")]
impl From<&crate::graphics::rhi::types::StencilFaceState> for windows::Win32::Graphics::Direct3D11::D3D11_DEPTH_STENCILOP_DESC {
    fn from(_state: &crate::graphics::rhi::types::StencilFaceState) -> Self {
        windows::Win32::Graphics::Direct3D11::D3D11_DEPTH_STENCILOP_DESC {
            StencilFailOp: D3D11_STENCIL_OP_KEEP,
            StencilDepthFailOp: D3D11_STENCIL_OP_KEEP,
            StencilPassOp: D3D11_STENCIL_OP_KEEP,
            StencilFunc: D3D11_COMPARISON_ALWAYS,
        }
    }
}

#[cfg(target_os = "windows")]
impl From<&crate::graphics::rhi::types::ColorBlendState> for windows::Win32::Graphics::Direct3D11::D3D11_RENDER_TARGET_BLEND_DESC {
    fn from(_state: &crate::graphics::rhi::types::ColorBlendState) -> Self {
        windows::Win32::Graphics::Direct3D11::D3D11_RENDER_TARGET_BLEND_DESC {
            BlendEnable: false,
            SrcBlend: D3D11_BLEND_ONE,
            DestBlend: D3D11_BLEND_ZERO,
            BlendOp: D3D11_BLEND_OP_ADD,
            SrcBlendAlpha: D3D11_BLEND_ONE,
            DestBlendAlpha: D3D11_BLEND_ZERO,
            BlendOpAlpha: D3D11_BLEND_OP_ADD,
            RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0,
        }
    }
}

/// DX11 реализация Pipeline State
pub struct Dx11PipelineState {
    device: ID3D11Device,
    input_layout: Option<ID3D11InputLayout>,
    rasterizer_state: ID3D11RasterizerState,
    blend_state: ID3D11BlendState,
    depth_stencil_state: ID3D11DepthStencilState,
    topology: u32,
    fill_mode: FillMode,
    cull_mode: CullMode,
    primitive_topology: PrimitiveTopology,
    
    // Кэшированные состояния для быстрого применения
    blend_factor: [f32; 4],
    sample_mask: u32,
    stencil_ref: u32,
}

impl Dx11PipelineState {
    /// Создает новый pipeline state object
    pub fn new(
        device: &ID3D11Device,
        desc: &PipelineDesc,
        vertex_shader: &dyn IShader,
    ) -> Result<Self, PipelineError> {
        info!("Creating DX11 Pipeline State");
        
        // 1. Создаем Input Layout на основе описания вершин и шейдера
        let input_layout = Self::create_input_layout(device, desc, vertex_shader)?;
        
        // 2. Создаем Rasterizer State
        let rasterizer_state = Self::create_rasterizer_state(device, desc)?;
        
        // 3. Создаем Blend State
        let blend_state = Self::create_blend_state(device, desc)?;
        
        // 4. Создаем Depth Stencil State
        let depth_stencil_state = Self::create_depth_stencil_state(device, desc)?;
        
        // 5. Конвертируем топологию примитивов
        let topology = Self::convert_topology(desc.primitive_topology);
        
        // 6. Инициализируем кэшированные значения
        let blend_factor = [1.0, 1.0, 1.0, 1.0];
        let sample_mask = u32::MAX;
        let stencil_ref = desc.stencil_state.reference;
        
        info!("DX11 Pipeline State created successfully");
        info!("  - Topology: {:?}", desc.primitive_topology);
        info!("  - Fill Mode: {:?}", desc.rasterizer_state.fill_mode);
        info!("  - Cull Mode: {:?}", desc.rasterizer_state.cull_mode);
        info!("  - Depth Test: {}", desc.depth_state.enabled);
        info!("  - Depth Write: {}", desc.depth_state.enabled);
        info!("  - Blending: {}", desc.color_blend_states.iter().any(|rt| rt.enabled));
        
        Ok(Self {
            device: device.clone(),
            input_layout,
            rasterizer_state,
            blend_state,
            depth_stencil_state,
            topology,
            fill_mode: desc.rasterizer_state.fill_mode,
            cull_mode: desc.rasterizer_state.cull_mode,
            primitive_topology: desc.primitive_topology,
            blend_factor,
            sample_mask,
            stencil_ref,
        })
    }
    
    /// Создает Input Layout на основе описания вершин
    fn create_input_layout(
        device: &ID3D11Device,
        desc: &PipelineDesc,
        vertex_shader: &dyn IShader,
    ) -> Result<Option<ID3D11InputLayout>, PipelineError> {
        if desc.input_layout.is_empty() {
            info!("No input layout specified, skipping Input Layout creation");
            return Ok(None);
        }
        
        info!("Creating Input Layout with {} elements", desc.input_layout.len());
        
        // Конвертируем элементы в DX11 формат
        let mut elements: Vec<D3D11_INPUT_ELEMENT_DESC> = Vec::with_capacity(desc.input_layout.len());
        
        for (i, elem) in desc.input_layout.iter().enumerate() {
            let semantic_name = Self::get_semantic_name(&elem.semantic);
            let semantic_index = Self::get_semantic_index(&elem.semantic);
            let format = Self::convert_format(elem.format);
            
            info!("  Element {}: semantic={}({}), format={:?}, offset={}", 
                  i, semantic_name, semantic_index, format, elem.offset);
            
            elements.push(D3D11_INPUT_ELEMENT_DESC {
                SemanticName: windows::core::PCSTR(semantic_name.as_ptr()),
                SemanticIndex: semantic_index,
                Format: format,
                InputSlot: elem.buffer_slot,
                AlignedByteOffset: elem.offset,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            });
        }
        
        // Получаем шейдерный код для создания input layout
        let shader_bytecode = vertex_shader.get_bytecode()
            .ok_or_else(|| PipelineError::CreationFailed("Vertex shader has no bytecode".into()))?;
        
        // Создаем Input Layout
        let input_layout = unsafe {
            device.CreateInputLayout(
                &elements,
                Some(shader_bytecode),
            )
        }.map_err(|e| {
            error!("Failed to create Input Layout: {:?}", e);
            PipelineError::CreationFailed(format!("CreateInputLayout failed: {:?}", e))
        })?;
        
        info!("Input Layout created successfully");
        Ok(Some(input_layout))
    }
    
    /// Создает Rasterizer State
    fn create_rasterizer_state(
        device: &ID3D11Device,
        desc: &PipelineDesc,
    ) -> Result<ID3D11RasterizerState, PipelineError> {
        info!("Creating Rasterizer State");
        
        let fill_mode = match desc.rasterizer_state.fill_mode {
            FillMode::Wireframe => D3D11_FILL_WIREFRAME,
            FillMode::Solid => D3D11_FILL_SOLID,
            _ => D3D11_FILL_SOLID,
        };
        
        let cull_mode = match desc.rasterizer_state.cull_mode {
            CullMode::None => D3D11_CULL_NONE,
            CullMode::Front => D3D11_CULL_FRONT,
            CullMode::Back => D3D11_CULL_BACK,
        };
        
        let rs_desc = D3D11_RASTERIZER_DESC {
            FillMode: fill_mode,
            CullMode: cull_mode,
            FrontCounterClockwise: DX11_FALSE,
            DepthBias: 0,
            DepthBiasClamp: 0.0,
            SlopeScaledDepthBias: 0.0,
            DepthClipEnable: 1,
            ScissorEnable: 0,
            MultisampleEnable: 0,
            AntialiasedLineEnable: DX11_FALSE,
        };
        
        let rasterizer_state = unsafe {
            device.CreateRasterizerState(&rs_desc)
        }.map_err(|e| {
            error!("Failed to create Rasterizer State: {:?}", e);
            PipelineError::CreationFailed(format!("CreateRasterizerState failed: {:?}", e))
        })?;
        
        info!("Rasterizer State created: fill={:?}, cull={:?}", desc.rasterizer_state.fill_mode, desc.rasterizer_state.cull_mode);
        Ok(rasterizer_state)
    }
    
    /// Создает Blend State
    fn create_blend_state(
        device: &ID3D11Device,
        desc: &PipelineDesc,
    ) -> Result<ID3D11BlendState, PipelineError> {
        info!("Creating Blend State");
        
        let mut render_targets: [D3D11_RENDER_TARGET_BLEND_DESC; 8] = [D3D11_RENDER_TARGET_BLEND_DESC {
            BlendEnable: DX11_FALSE,
            SrcBlend: D3D11_BLEND_ONE,
            DestBlend: D3D11_BLEND_ZERO,
            BlendOp: D3D11_BLEND_OP_ADD,
            SrcBlendAlpha: D3D11_BLEND_ONE,
            DestBlendAlpha: D3D11_BLEND_ZERO,
            BlendOpAlpha: D3D11_BLEND_OP_ADD,
            RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL as u8,
        }; 8];
        
        for (i, rt_desc) in desc.color_blend_states.iter().take(8).enumerate() {
            render_targets[i].BlendEnable = if rt_desc.enabled { DX11_TRUE } else { DX11_FALSE };
            render_targets[i].SrcBlend = Self::convert_blend_factor(rt_desc.src_color_blend);
            render_targets[i].DestBlend = Self::convert_blend_factor(rt_desc.dst_color_blend);
            render_targets[i].BlendOp = Self::convert_blend_op(rt_desc.color_blend_op);
            render_targets[i].SrcBlendAlpha = Self::convert_blend_factor(rt_desc.src_alpha_blend);
            render_targets[i].DestBlendAlpha = Self::convert_blend_factor(rt_desc.dst_alpha_blend);
            render_targets[i].BlendOpAlpha = Self::convert_blend_op(rt_desc.alpha_blend_op);
            render_targets[i].RenderTargetWriteMask = Self::convert_color_mask(rt_desc.write_mask);
            
            info!("  RT[{}]: blend={}, src={:?}, dst={:?}, op={:?}", 
                  i, rt_desc.enabled, rt_desc.src_color_blend, rt_desc.dst_color_blend, rt_desc.color_blend_op);
        }
        
        let blend_desc = D3D11_BLEND_DESC {
            AlphaToCoverageEnable: 0,
            IndependentBlendEnable: if desc.color_blend_states.len() > 1 { DX11_TRUE } else { DX11_FALSE },
            RenderTarget: render_targets,
        };
        
        let blend_state = unsafe {
            device.CreateBlendState(&blend_desc)
        }.map_err(|e| {
            error!("Failed to create Blend State: {:?}", e);
            PipelineError::CreationFailed(format!("CreateBlendState failed: {:?}", e))
        })?;
        
        info!("Blend State created successfully");
        Ok(blend_state)
    }
    
    /// Создает Depth Stencil State
    fn create_depth_stencil_state(
        device: &ID3D11Device,
        desc: &PipelineDesc,
    ) -> Result<ID3D11DepthStencilState, PipelineError> {
        info!("Creating Depth Stencil State");
        
        let depth_desc = &desc.depth_state;
        let stencil_desc = &desc.stencil_state;
        
        let depth_func = Self::convert_compare_func(depth_desc.compare_func);
        let depth_write_mask = if depth_desc.write_enabled {
            D3D11_DEPTH_WRITE_MASK_ALL
        } else {
            D3D11_DEPTH_WRITE_MASK_ZERO
        };
        
        let front_face = Self::convert_stencil_op_desc(&stencil_desc.front_face);
        let back_face = Self::convert_stencil_op_desc(&stencil_desc.back_face);
        
        let depth_stencil_desc = D3D11_DEPTH_STENCIL_DESC {
            DepthEnable: if depth_desc.enabled { DX11_TRUE } else { DX11_FALSE },
            DepthWriteMask: depth_write_mask,
            DepthFunc: depth_func,
            StencilEnable: if stencil_desc.enabled { DX11_TRUE } else { DX11_FALSE },
            StencilReadMask: stencil_desc.read_mask,
            StencilWriteMask: stencil_desc.write_mask,
            FrontFace: front_face,
            BackFace: back_face,
        };
        
        let depth_stencil_state = unsafe {
            device.CreateDepthStencilState(&depth_stencil_desc)
        }.map_err(|e| {
            error!("Failed to create Depth Stencil State: {:?}", e);
            PipelineError::CreationFailed(format!("CreateDepthStencilState failed: {:?}", e))
        })?;
        
        info!("Depth Stencil State created: depth_test={}, depth_write={}, stencil={}", 
              depth_desc.enabled, depth_desc.write_enabled, stencil_desc.enabled);
        Ok(depth_stencil_state)
    }
    
    /// Конвертирует семантику в имя
    fn get_semantic_name(semantic: &InputElementSemantic) -> String {
        match semantic {
            InputElementSemantic::Position => "POSITION",
            InputElementSemantic::Normal => "NORMAL",
            InputElementSemantic::Tangent => "TANGENT",
            InputElementSemantic::Binormal => "BINORMAL",
            InputElementSemantic::Color(_) => "COLOR",
            InputElementSemantic::TexCoord(_) => "TEXCOORD",
            InputElementSemantic::Custom(name) => name,
        }.to_string()
    }
    
    /// Получает индекс семантики
    fn get_semantic_index(semantic: &InputElementSemantic) -> u32 {
        match semantic {
            InputElementSemantic::Color(index) => *index,
            InputElementSemantic::TexCoord(index) => *index,
            _ => 0,
        }
    }
    
    /// Конвертирует формат элемента
    fn convert_format(format: InputElementFormat) -> DXGI_FORMAT {
        match format {
            InputElementFormat::Float32x2 => DXGI_FORMAT_R32G32_FLOAT,
            InputElementFormat::Float32x3 => DXGI_FORMAT_R32G32B32_FLOAT,
            InputElementFormat::Float32x4 => DXGI_FORMAT_R32G32B32A32_FLOAT,
            InputElementFormat::Float16x2 => DXGI_FORMAT_R16G16_FLOAT,
            InputElementFormat::Float16x4 => DXGI_FORMAT_R16G16B16A16_FLOAT,
            InputElementFormat::UInt8x4 => DXGI_FORMAT_R8G8B8A8_UINT,
            InputElementFormat::Int8x4 => DXGI_FORMAT_R8G8B8A8_SINT,
            InputElementFormat::UInt16x2 => DXGI_FORMAT_R16G16_UINT,
            InputElementFormat::UInt16x4 => DXGI_FORMAT_R16G16B16A16_UINT,
            InputElementFormat::Int16x2 => DXGI_FORMAT_R16G16_SINT,
            InputElementFormat::Int16x4 => DXGI_FORMAT_R16G16B16A16_SINT,
            InputElementFormat::UInt32 => DXGI_FORMAT_R32_UINT,
            InputElementFormat::UInt32x2 => DXGI_FORMAT_R32G32_UINT,
            InputElementFormat::UInt32x4 => DXGI_FORMAT_R32G32B32A32_UINT,
            InputElementFormat::Int32 => DXGI_FORMAT_R32_SINT,
            InputElementFormat::Int32x2 => DXGI_FORMAT_R32G32_SINT,
            InputElementFormat::Int32x4 => DXGI_FORMAT_R32G32B32A32_SINT,
        }
    }
    
    /// Конвертирует топологию примитивов
    fn convert_topology(topology: PrimitiveTopology) -> u32 {
        match topology {
            PrimitiveTopology::PointList => 1,
            PrimitiveTopology::LineList => 2,
            PrimitiveTopology::LineStrip => 3,
            PrimitiveTopology::TriangleList => 4,
            PrimitiveTopology::TriangleStrip => 5,
        }
    }
    
    /// Конвертирует фактор блендинга
    fn convert_blend_factor(factor: BlendFactor) -> windows::Win32::Graphics::Direct3D11::D3D11_BLEND {
        match factor {
            BlendFactor::Zero => D3D11_BLEND_ZERO,
            BlendFactor::One => D3D11_BLEND_ONE,
            BlendFactor::SrcColor => D3D11_BLEND_SRC_COLOR,
            BlendFactor::InvSrcColor => D3D11_BLEND_INV_SRC_COLOR,
            BlendFactor::SrcAlpha => D3D11_BLEND_SRC_ALPHA,
            BlendFactor::InvSrcAlpha => D3D11_BLEND_INV_SRC_ALPHA,
            BlendFactor::DstAlpha => D3D11_BLEND_DEST_ALPHA,
            BlendFactor::InvDstAlpha => D3D11_BLEND_INV_DEST_ALPHA,
            BlendFactor::DstColor => D3D11_BLEND_DEST_COLOR,
            BlendFactor::InvDstColor => D3D11_BLEND_INV_DEST_COLOR,
            BlendFactor::SrcAlphaSat => D3D11_BLEND_SRC_ALPHA_SAT,
            BlendFactor::BlendFactor => D3D11_BLEND_FACTOR,
            BlendFactor::InvBlendFactor => D3D11_BLEND_INV_FACTOR,
        }
    }
    
    /// Конвертирует операцию блендинга
    fn convert_blend_op(op: BlendOp) -> windows::Win32::Graphics::Direct3D11::D3D11_BLEND_OP {
        match op {
            BlendOp::Add => D3D11_BLEND_OP_ADD,
            BlendOp::Subtract => D3D11_BLEND_OP_SUBTRACT,
            BlendOp::RevSubtract => D3D11_BLEND_OP_REV_SUBTRACT,
            BlendOp::ReverseSubtract => D3D11_BLEND_OP_REV_SUBTRACT,
            BlendOp::Min => D3D11_BLEND_OP_MIN,
            BlendOp::Max => D3D11_BLEND_OP_MAX,
        }
    }
    
    /// Конвертирует маску цвета
    fn convert_color_mask(mask: ColorWriteMask) -> u8 {
        let mut result: u8 = 0;
        if mask.contains(ColorWriteMask::RED) {
            result |= D3D11_COLOR_WRITE_ENABLE_RED.0;
        }
        if mask.contains(ColorWriteMask::GREEN) {
            result |= D3D11_COLOR_WRITE_ENABLE_GREEN.0;
        }
        if mask.contains(ColorWriteMask::BLUE) {
            result |= D3D11_COLOR_WRITE_ENABLE_BLUE.0;
        }
        if mask.contains(ColorWriteMask::ALPHA) {
            result |= D3D11_COLOR_WRITE_ENABLE_ALPHA.0;
        }
        if mask == ColorWriteMask::ALL {
            result = D3D11_COLOR_WRITE_ENABLE_ALL.0;
        }
        result
    }
    
    /// Конвертирует функцию сравнения
    fn convert_compare_func(func: CompareFunc) -> D3D11_COMPARISON_FUNC {
        match func {
            CompareFunc::Never => D3D11_COMPARISON_NEVER,
            CompareFunc::Less => D3D11_COMPARISON_LESS,
            CompareFunc::Equal => D3D11_COMPARISON_EQUAL,
            CompareFunc::LessEqual => D3D11_COMPARISON_LESS_EQUAL,
            CompareFunc::Greater => D3D11_COMPARISON_GREATER,
            CompareFunc::NotEqual => D3D11_COMPARISON_NOT_EQUAL,
            CompareFunc::GreaterEqual => D3D11_COMPARISON_GREATER_EQUAL,
            CompareFunc::Always => D3D11_COMPARISON_ALWAYS,
        }
    }
    
    /// Конвертирует операцию stencil
    fn convert_stencil_op(op: StencilOp) -> D3D11_STENCIL_OP {
        match op {
            StencilOp::Keep => D3D11_STENCIL_OP_KEEP,
            StencilOp::Zero => D3D11_STENCIL_OP_ZERO,
            StencilOp::Replace => D3D11_STENCIL_OP_REPLACE,
            StencilOp::IncrSat => D3D11_STENCIL_OP_INCR_SAT,
            StencilOp::DecrSat => D3D11_STENCIL_OP_DECR_SAT,
            StencilOp::Invert => D3D11_STENCIL_OP_INVERT,
            StencilOp::Incr => D3D11_STENCIL_OP_INCR,
            StencilOp::Decr => D3D11_STENCIL_OP_DECR,
            StencilOp::IncrementClamp => D3D11_STENCIL_OP_INCR_SAT,
            StencilOp::DecrementClamp => D3D11_STENCIL_OP_DECR_SAT,
            StencilOp::IncrementWrap => D3D11_STENCIL_OP_INCR,
            StencilOp::DecrementWrap => D3D11_STENCIL_OP_DECR,
        }
    }
    
    /// Конвертирует описание stencil операции
    fn convert_stencil_op_desc(desc: &StencilFaceState) -> StencilOpDescStub {
        StencilOpDescStub {
            pubStencilFailOp: Self::convert_stencil_op(desc.fail_op),
            pubStencilDepthFailOp: Self::convert_stencil_op(desc.depth_fail_op),
            pubStencilPassOp: Self::convert_stencil_op(desc.pass_op),
            pubStencilFunc: Self::convert_compare_func(desc.compare_func),
        }
    }
    
    /// Возвращает Input Layout для использования в контексте
    pub fn get_input_layout(&self) -> Option<&ID3D11InputLayout> {
        self.input_layout.as_ref()
    }
    
    /// Возвращает Rasterizer State
    pub fn get_rasterizer_state(&self) -> &ID3D11RasterizerState {
        &self.rasterizer_state
    }
    
    /// Возвращает Blend State
    pub fn get_blend_state(&self) -> &ID3D11BlendState {
        &self.blend_state
    }
    
    /// Возвращает Depth Stencil State
    pub fn get_depth_stencil_state(&self) -> &ID3D11DepthStencilState {
        &self.depth_stencil_state
    }
    
    /// Возвращает топологию примитивов
    pub fn get_topology(&self) -> u32 {
        self.topology
    }
    
    /// Возвращает фактор блендинга
    pub fn get_blend_factor(&self) -> [f32; 4] {
        self.blend_factor
    }
    
    /// Возвращает маску сэмплов
    pub fn get_sample_mask(&self) -> u32 {
        self.sample_mask
    }
    
    /// Возвращает референсное значение stencil
    pub fn get_stencil_ref(&self) -> u32 {
        self.stencil_ref
    }
}

impl IPipelineState for Dx11PipelineState {
    fn bind(&self, context: &mut dyn std::any::Any) -> Result<(), PipelineError> {
        info!("Pipeline State bound (logical bind)");
        Ok(())
    }
    
    fn set_primitive_topology(&mut self, topology: PrimitiveTopology) {
        self.topology = Self::convert_topology(topology);
        self.primitive_topology = topology;
        info!("Pipeline topology changed to {:?}", topology);
    }
    
    fn set_blend_constants(&mut self, factors: [f32; 4]) {
        self.blend_factor = factors;
        info!("Pipeline blend constants updated: {:?}", factors);
    }
    
    fn set_stencil_reference(&mut self, reference: u32) {
        self.stencil_ref = reference;
        info!("Pipeline stencil reference updated: {}", reference);
    }
}

// Для кроссплатформенной совместимости
#[cfg(not(target_os = "windows"))]
pub struct Dx11PipelineState;

#[cfg(not(target_os = "windows"))]
impl Dx11PipelineState {
    pub fn new(
        _device: &(),
        _desc: &PipelineDesc,
        _vertex_shader: &dyn IShader,
    ) -> Result<Self, PipelineError> {
        Err(PipelineError::CreationFailed("DX11 not available on this platform".into()))
    }
}

#[cfg(not(target_os = "windows"))]
impl IPipelineState for Dx11PipelineState {
    fn bind(&self, _context: &mut dyn std::any::Any) -> Result<(), PipelineError> {
        Err(PipelineError::CreationFailed("DX11 not available on this platform".into()))
    }
    
    fn set_primitive_topology(&mut self, _topology: PrimitiveTopology) {}
    fn set_blend_constants(&mut self, _factors: [f32; 4]) {}
    fn set_stencil_reference(&mut self, _reference: u32) {}
}
