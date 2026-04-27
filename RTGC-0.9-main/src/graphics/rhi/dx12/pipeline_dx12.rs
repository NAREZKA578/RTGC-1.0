// DirectX 12 Backend - Pipeline State Object Implementation
// Implements PSO creation and management for DX12

use crate::graphics::rhi::types::*;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Direct3D12::*,
    Win32::Graphics::Dxgi::Common::*,
};

/// DX12 Pipeline State Object
pub struct Dx12PipelineState {
    #[cfg(target_os = "windows")]
    pso: ID3D12PipelineState,
    
    handle: ResourceHandle,
    description: PipelineStateObject,
}

unsafe impl Send for Dx12PipelineState {}
unsafe impl Sync for Dx12PipelineState {}

impl Dx12PipelineState {
    /// Create a new DX12 PSO
    #[cfg(target_os = "windows")]
    pub fn new(
        device: &ID3D12Device,
        desc: &PipelineStateObject,
        root_signature: &ID3D12RootSignature,
        handle: ResourceHandle,
    ) -> RhiResult<Self> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        // Convert input layout to DX12
        let input_elements = Self::build_input_layout(&desc.input_layout);
        
        // Convert blend states
        let blend_desc = Self::build_blend_state(&desc.color_blend_states);
        
        // Convert depth/stencil state
        let depth_stencil_desc = Self::build_depth_stencil_state(&desc.depth_state, &desc.stencil_state);
        
        // Convert rasterizer state
        let rasterizer_desc = Self::build_rasterizer_state(&desc.rasterizer_state);
        
        // Convert primitive topology
        let primitive_topology = match desc.primitive_topology {
            PrimitiveTopology::PointList => D3D_PRIMITIVE_TOPOLOGY_POINTLIST,
            PrimitiveTopology::LineList => D3D_PRIMITIVE_TOPOLOGY_LINELIST,
            PrimitiveTopology::LineStrip => D3D_PRIMITIVE_TOPOLOGY_LINESTRIP,
            PrimitiveTopology::TriangleList => D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            PrimitiveTopology::TriangleStrip => D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP,
        };
        
        // Get shader bytecode from shader handles
        let vs_bytecode = if let Some(vs_handle) = desc.vertex_shader {
            // In a real implementation, we would get the shader from the resource manager
            // For now, use empty bytecode
            D3D12_SHADER_BYTECODE::default()
        } else {
            D3D12_SHADER_BYTECODE::default()
        };
        
        let ps_bytecode = if let Some(ps_handle) = desc.fragment_shader {
            // In a real implementation, we would get the shader from the resource manager
            // For now, use empty bytecode
            D3D12_SHADER_BYTECODE::default()
        } else {
            D3D12_SHADER_BYTECODE::default()
        };
        
        // Set RTV formats from color blend states
        let mut rtv_formats = [DXGI_FORMAT_UNKNOWN; 8];
        for (i, _) in desc.color_blend_states.iter().enumerate().take(8) {
            rtv_formats[i] = DXGI_FORMAT_R8G8B8A8_UNORM; // Default format, should come from swapchain/texture
        }
        
        // Set DSV format if depth buffer exists
        let dsv_format = if desc.depth_state.enabled {
            DXGI_FORMAT_D32_FLOAT_S8X24_UINT // Default depth format
        } else {
            DXGI_FORMAT_UNKNOWN
        };
        
        let pso_desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
            pRootSignature: Some(root_signature.clone()),
            VS: vs_bytecode,
            PS: ps_bytecode,
            GS: D3D12_SHADER_BYTECODE::default(),
            DS: D3D12_SHADER_BYTECODE::default(),
            HS: D3D12_SHADER_BYTECODE::default(),
            StreamOutput: D3D12_STREAM_OUTPUT_DESC::default(),
            BlendState: blend_desc,
            SampleMask: u32::MAX,
            RasterizerState: rasterizer_desc,
            DepthStencilState: depth_stencil_desc,
            InputLayout: D3D12_INPUT_LAYOUT_DESC {
                pInputElementDescs: input_elements.as_ptr(),
                NumElements: input_elements.len() as u32,
            },
            IBStripCutValue: D3D12_INDEX_BUFFER_STRIP_CUT_VALUE_DISABLED,
            PrimitiveTopologyType: Self::get_topology_type(desc.primitive_topology),
            NumRenderTargets: desc.color_blend_states.len() as u32,
            RTVFormats: rtv_formats,
            DSVFormat: dsv_format,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: desc.sample_count,
                Quality: 0,
            },
            NodeMask: 0,
            CachedPSO: D3D12_CACHED_PIPELINE_STATE::default(),
            Flags: D3D12_PIPELINE_STATE_FLAG_NONE,
        };
        
        let pso: ID3D12PipelineState = unsafe {
            device.CreateGraphicsPipelineState(&pso_desc)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create PSO: {:?}", e)))?
        };
        
        Ok(Self {
            pso,
            handle,
            description: desc.clone(),
        })
    }
    
    #[cfg(target_os = "windows")]
    fn build_input_layout(layout: &InputLayout) -> Vec<D3D12_INPUT_ELEMENT_DESC> {
        let mut elements = Vec::with_capacity(layout.attributes.len());
        let mut offset = 0u32;
        
        for (i, attr) in layout.attributes.iter().enumerate() {
            let semantic_name = Self::get_semantic_name(&attr.name);
            let format = Self::to_dxgi_format(attr.format);
            
            elements.push(D3D12_INPUT_ELEMENT_DESC {
                SemanticName: semantic_name,
                SemanticIndex: i as u32,
                Format: format,
                InputSlot: 0,
                AlignedByteOffset: offset,
                InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            });
            
            offset += attr.format.size_bytes() as u32;
        }
        
        elements
    }
    
    #[cfg(target_os = "windows")]
    fn get_semantic_name(name: &str) -> *const i8 {
        match name.to_lowercase().as_str() {
            "position" | "pos" => c"POSITION".as_ptr(),
            "normal" | "norm" => c"NORMAL".as_ptr(),
            "tangent" | "tang" => c"TANGENT".as_ptr(),
            "binormal" | "binorm" => c"BINORMAL".as_ptr(),
            "texcoord" | "uv" | "tex" => c"TEXCOORD".as_ptr(),
            "color" | "col" => c"COLOR".as_ptr(),
            _ => c"TEXCOORD".as_ptr(),
        }
    }
    
    #[cfg(target_os = "windows")]
    fn to_dxgi_format(format: VertexFormat) -> DXGI_FORMAT {
        match format {
            VertexFormat::Float32x2 => DXGI_FORMAT_R32G32_FLOAT,
            VertexFormat::Float32x3 => DXGI_FORMAT_R32G32B32_FLOAT,
            VertexFormat::Float32x4 => DXGI_FORMAT_R32G32B32A32_FLOAT,
            VertexFormat::Float32x2x2 => DXGI_FORMAT_R32G32B32A32_FLOAT,
            VertexFormat::Float32x3x3 => DXGI_FORMAT_R32G32B32A32_FLOAT,
            VertexFormat::Float32x4x4 => DXGI_FORMAT_R32G32B32A32_FLOAT,
            VertexFormat::Uint8x4Norm => DXGI_FORMAT_R8G8B8A8_UNORM,
            VertexFormat::Uint16x2Norm => DXGI_FORMAT_R16G16_UNORM,
            VertexFormat::Uint16x4Norm => DXGI_FORMAT_R16G16B16A16_UNORM,
        }
    }
    
    #[cfg(target_os = "windows")]
    fn build_blend_state(blend_states: &[ColorBlendState]) -> D3D12_BLEND_DESC {
        let mut rt = [D3D12_RENDER_TARGET_BLEND_DESC::default(); 8];
        
        for (i, state) in blend_states.iter().take(8).enumerate() {
            rt[i] = D3D12_RENDER_TARGET_BLEND_DESC {
                BlendEnable: state.enabled.into(),
                LogicOpEnable: false.into(),
                SrcBlend: Self::to_dx12_blend(state.src_color_blend),
                DestBlend: Self::to_dx12_blend(state.dst_color_blend),
                BlendOp: Self::to_dx12_blend_op(state.color_blend_op),
                SrcBlendAlpha: Self::to_dx12_blend(state.src_alpha_blend),
                DestBlendAlpha: Self::to_dx12_blend(state.dst_alpha_blend),
                BlendOpAlpha: Self::to_dx12_blend_op(state.alpha_blend_op),
                RenderTargetWriteMask: state.write_mask,
            };
        }
        
        D3D12_BLEND_DESC {
            AlphaToCoverageEnable: false.into(),
            IndependentBlendEnable: (blend_states.len() > 1).into(),
            RenderTarget: rt,
        }
    }
    
    #[cfg(target_os = "windows")]
    fn to_dx12_blend(blend: BlendMode) -> D3D12_BLEND {
        match blend {
            BlendMode::Zero => D3D12_BLEND_ZERO,
            BlendMode::One => D3D12_BLEND_ONE,
            BlendMode::SrcColor => D3D12_BLEND_SRC_COLOR,
            BlendMode::OneMinusSrcColor => D3D12_BLEND_INV_SRC_COLOR,
            BlendMode::DstColor => D3D12_BLEND_DEST_COLOR,
            BlendMode::OneMinusDstColor => D3D12_BLEND_INV_DEST_COLOR,
            BlendMode::SrcAlpha => D3D12_BLEND_SRC_ALPHA,
            BlendMode::OneMinusSrcAlpha => D3D12_BLEND_INV_SRC_ALPHA,
            BlendMode::DstAlpha => D3D12_BLEND_DEST_ALPHA,
            BlendMode::OneMinusDstAlpha => D3D12_BLEND_INV_DEST_ALPHA,
        }
    }
    
    #[cfg(target_os = "windows")]
    fn to_dx12_blend_op(op: BlendOp) -> D3D12_BLEND_OP {
        match op {
            BlendOp::Add => D3D12_BLEND_OP_ADD,
            BlendOp::Subtract => D3D12_BLEND_OP_SUBTRACT,
            BlendOp::ReverseSubtract => D3D12_BLEND_OP_REV_SUBTRACT,
            BlendOp::Min => D3D12_BLEND_OP_MIN,
            BlendOp::Max => D3D12_BLEND_OP_MAX,
        }
    }
    
    #[cfg(target_os = "windows")]
    fn build_depth_stencil_state(depth: &DepthState, stencil: &StencilState) -> D3D12_DEPTH_STENCIL_DESC {
        D3D12_DEPTH_STENCIL_DESC {
            DepthEnable: depth.enabled.into(),
            DepthWriteMask: if depth.write_enabled { D3D12_DEPTH_WRITE_MASK_ALL } else { D3D12_DEPTH_WRITE_MASK_ZERO },
            DepthFunc: Self::to_dx12_compare_func(depth.compare_func),
            StencilEnable: stencil.enabled.into(),
            StencilReadMask: stencil.read_mask,
            StencilWriteMask: stencil.write_mask,
            FrontFace: Self::to_dx12_depth_stencil_op(&stencil.front_face),
            BackFace: Self::to_dx12_depth_stencil_op(&stencil.back_face),
        }
    }
    
    #[cfg(target_os = "windows")]
    fn to_dx12_compare_func(func: CompareFunc) -> D3D12_COMPARISON_FUNC {
        match func {
            CompareFunc::Never => D3D12_COMPARISON_FUNC_NEVER,
            CompareFunc::Less => D3D12_COMPARISON_FUNC_LESS,
            CompareFunc::Equal => D3D12_COMPARISON_FUNC_EQUAL,
            CompareFunc::LessEqual => D3D12_COMPARISON_FUNC_LESS_EQUAL,
            CompareFunc::Greater => D3D12_COMPARISON_FUNC_GREATER,
            CompareFunc::NotEqual => D3D12_COMPARISON_FUNC_NOT_EQUAL,
            CompareFunc::GreaterEqual => D3D12_COMPARISON_FUNC_GREATER_EQUAL,
            CompareFunc::Always => D3D12_COMPARISON_FUNC_ALWAYS,
        }
    }
    
    #[cfg(target_os = "windows")]
    fn to_dx12_depth_stencil_op(face: &StencilFaceState) -> D3D12_DEPTH_STENCILOP_DESC {
        D3D12_DEPTH_STENCILOP_DESC {
            StencilFailOp: Self::to_dx12_stencil_op(face.fail_op),
            StencilDepthFailOp: Self::to_dx12_stencil_op(face.depth_fail_op),
            StencilPassOp: Self::to_dx12_stencil_op(face.pass_op),
            StencilFunc: Self::to_dx12_compare_func(face.compare_func),
        }
    }
    
    #[cfg(target_os = "windows")]
    fn to_dx12_stencil_op(op: StencilOp) -> D3D12_STENCIL_OP {
        match op {
            StencilOp::Keep => D3D12_STENCIL_OP_KEEP,
            StencilOp::Zero => D3D12_STENCIL_OP_ZERO,
            StencilOp::Replace => D3D12_STENCIL_OP_REPLACE,
            StencilOp::IncrementClamp => D3D12_STENCIL_OP_INCR_SAT,
            StencilOp::DecrementClamp => D3D12_STENCIL_OP_DECR_SAT,
            StencilOp::Invert => D3D12_STENCIL_OP_INVERT,
            StencilOp::IncrementWrap => D3D12_STENCIL_OP_INCR,
            StencilOp::DecrementWrap => D3D12_STENCIL_OP_DECR,
        }
    }
    
    #[cfg(target_os = "windows")]
    fn build_rasterizer_state(rasterizer: &RasterizerState) -> D3D12_RASTERIZER_DESC {
        D3D12_RASTERIZER_DESC {
            FillMode: match rasterizer.fill_mode {
                FillMode::Solid => D3D12_FILL_MODE_SOLID,
                FillMode::Wireframe => D3D12_FILL_MODE_WIREFRAME,
                FillMode::Point => D3D12_FILL_MODE_POINT,
            },
            CullMode: match rasterizer.cull_mode {
                CullMode::None => D3D12_CULL_MODE_NONE,
                CullMode::Front => D3D12_CULL_MODE_FRONT,
                CullMode::Back => D3D12_CULL_MODE_BACK,
            },
            FrontCounterClockwise: match rasterizer.front_face {
                FrontFace::CounterClockwise => true.into(),
                FrontFace::Clockwise => false.into(),
            },
            DepthBias: 0,
            DepthBiasClamp: 0.0,
            SlopeScaledDepthBias: 0.0,
            DepthClipEnable: true.into(),
            MultisampleEnable: false.into(),
            AntialiasedLineEnable: false.into(),
            ConservativeRaster: D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
        }
    }
    
    #[cfg(target_os = "windows")]
    fn get_topology_type(topology: PrimitiveTopology) -> D3D12_PRIMITIVE_TOPOLOGY_TYPE {
        match topology {
            PrimitiveTopology::PointList => D3D12_PRIMITIVE_TOPOLOGY_TYPE_POINT,
            PrimitiveTopology::LineList | PrimitiveTopology::LineStrip => D3D12_PRIMITIVE_TOPOLOGY_TYPE_LINE,
            PrimitiveTopology::TriangleList | PrimitiveTopology::TriangleStrip => D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
        }
    }
    
    #[cfg(target_os = "windows")]
    pub fn pso(&self) -> &ID3D12PipelineState {
        &self.pso
    }
    
    pub fn handle(&self) -> ResourceHandle {
        self.handle
    }
    
    pub fn description(&self) -> &PipelineStateObject {
        &self.description
    }
}

/// DX12 Root Signature
pub struct Dx12RootSignature {
    #[cfg(target_os = "windows")]
    root_signature: ID3D12RootSignature,
}

unsafe impl Send for Dx12RootSignature {}
unsafe impl Sync for Dx12RootSignature {}

impl Dx12RootSignature {
    #[cfg(target_os = "windows")]
    pub fn new(device: &ID3D12Device) -> RhiResult<Self> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        // Build root signature with descriptor tables for CBVs, SRVs, and samplers
        let mut root_params = Vec::new();
        
        // Parameter 0: Constant Buffer View (b0)
        root_params.push(D3D12_ROOT_PARAMETER {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_CBV,
            Anonymous: D3D12_ROOT_PARAMETER_0 {
                Descriptor: D3D12_ROOT_DESCRIPTOR {
                    ShaderRegister: 0,
                    RegisterSpace: 0,
                },
            },
            ShaderVisibility: D3D12_SHADER_VISIBILITY_VERTEX,
        });
        
        // Parameter 1: Shader Resource View (t0)
        root_params.push(D3D12_ROOT_PARAMETER {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_SRV,
            Anonymous: D3D12_ROOT_PARAMETER_0 {
                Descriptor: D3D12_ROOT_DESCRIPTOR {
                    ShaderRegister: 0,
                    RegisterSpace: 0,
                },
            },
            ShaderVisibility: D3D12_SHADER_VISIBILITY_PIXEL,
        });
        
        // Parameter 2: Sampler (s0)
        root_params.push(D3D12_ROOT_PARAMETER {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_SAMPLER,
            Anonymous: D3D12_ROOT_PARAMETER_0 {
                Descriptor: D3D12_ROOT_DESCRIPTOR {
                    ShaderRegister: 0,
                    RegisterSpace: 0,
                },
            },
            ShaderVisibility: D3D12_SHADER_VISIBILITY_PIXEL,
        });
        
        let static_samplers = [];
        
        let desc = D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: root_params.len() as u32,
            pParameters: root_params.as_ptr(),
            NumStaticSamplers: 0,
            pStaticSamplers: static_samplers.as_ptr(),
            Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
        };
        
        let serialized = unsafe {
            D3D12SerializeRootSignature(&desc, D3D_ROOT_SIGNATURE_VERSION_1)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to serialize root signature: {:?}", e)))?
        };
        
        let root_signature: ID3D12RootSignature = unsafe {
            device.CreateRootSignature(0, &serialized)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create root signature: {:?}", e)))?
        };
        
        Ok(Self {
            root_signature,
        })
    }
    
    #[cfg(target_os = "windows")]
    pub fn root_signature(&self) -> &ID3D12RootSignature {
        &self.root_signature
    }
}
