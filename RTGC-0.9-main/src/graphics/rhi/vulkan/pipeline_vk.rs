// Vulkan Backend - Pipeline State Object Implementation
// Implements PSO creation and management for Vulkan

use crate::graphics::rhi::types::*;
use ash::vk;

/// Vulkan Pipeline State Object
pub struct VkPipelineState {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    handle: ResourceHandle,
    description: PipelineStateObject,
}

unsafe impl Send for VkPipelineState {}
unsafe impl Sync for VkPipelineState {}

impl VkPipelineState {
    /// Create a new Vulkan PSO
    pub fn new(
        device: &ash::Device,
        desc: &PipelineStateObject,
        render_pass: vk::RenderPass,
        handle: ResourceHandle,
    ) -> RhiResult<Self> {
        // Convert input layout to Vulkan
        let input_elements = Self::build_input_layout(&desc.input_layout);
        
        // Convert blend states
        let color_blend_attachments = Self::build_blend_states(&desc.color_blend_states);
        
        // Convert depth/stencil state
        let depth_stencil_state = Self::build_depth_stencil_state(&desc.depth_state, &desc.stencil_state);
        
        // Convert rasterizer state
        let rasterization_state = Self::build_rasterizer_state(&desc.rasterizer_state);
        
        // Convert primitive topology
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(Self::to_vk_primitive_topology(desc.primitive_topology));
        
        // Vertex input state
        let binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: desc.input_layout.stride,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
        
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&input_elements);
        
        // Viewport state (dynamic)
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        
        // Dynamic states
        let dynamic_states = [
            vk::DynamicState::VIEWPORT,
            vk::DynamicState::SCISSOR,
            vk::DynamicState::LINE_WIDTH,
        ];
        
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&dynamic_states);
        
        // Color blend state
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);
        
        // Multisample state
        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::from_raw(desc.sample_count.max(1)));
        
        // Depth stencil state
        let depth_stencil = depth_stencil_state;
        
        // Graphics pipeline create info
        let stages = Self::build_shader_stages(device, desc);
        
        // Create pipeline layout (empty for now - can be extended with descriptor set layouts)
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::default();
        let pipeline_layout = unsafe {
            device.create_pipeline_layout(&pipeline_layout_create_info, None)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create pipeline layout: {:?}", e)))?
        };
        
        let create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);
        
        let pipelines = unsafe {
            device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[create_info],
                None,
            )
            .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create pipeline: {:?}", e)))?
        };
        
        let pipeline = pipelines[0];
        
        Ok(Self {
            pipeline,
            pipeline_layout,
            handle,
            description: desc.clone(),
        })
    }
    
    fn build_input_layout(layout: &InputLayout) -> Vec<vk::VertexInputAttributeDescription> {
        let mut attributes = Vec::with_capacity(layout.attributes.len());
        let mut offset = 0u32;
        
        for (i, attr) in layout.attributes.iter().enumerate() {
            attributes.push(vk::VertexInputAttributeDescription {
                location: i as u32,
                binding: 0,
                format: Self::to_vk_format(attr.format),
                offset,
            });
            
            offset += attr.format.size_bytes() as u32;
        }
        
        attributes
    }
    
    fn to_vk_format(format: VertexFormat) -> vk::Format {
        match format {
            VertexFormat::Float32x2 => vk::Format::R32G32_SFLOAT,
            VertexFormat::Float32x3 => vk::Format::R32G32B32_SFLOAT,
            VertexFormat::Float32x4 => vk::Format::R32G32B32A32_SFLOAT,
            VertexFormat::Float32x2x2 => vk::Format::R32G32B32A32_SFLOAT,
            VertexFormat::Float32x3x3 => vk::Format::R32G32B32A32_SFLOAT,
            VertexFormat::Float32x4x4 => vk::Format::R32G32B32A32_SFLOAT,
            VertexFormat::Uint8x4Norm => vk::Format::R8G8B8A8_UNORM,
            VertexFormat::Uint16x2Norm => vk::Format::R16G16_UNORM,
            VertexFormat::Uint16x4Norm => vk::Format::R16G16B16A16_UNORM,
        }
    }
    
    fn build_blend_states(blend_states: &[ColorBlendState]) -> Vec<vk::PipelineColorBlendAttachmentState> {
        blend_states
            .iter()
            .map(|state| {
                if state.enabled {
                    vk::PipelineColorBlendAttachmentState {
                        blend_enable: vk::TRUE,
                        src_color_blend_factor: Self::to_vk_blend_factor(state.src_color_blend),
                        dst_color_blend_factor: Self::to_vk_blend_factor(state.dst_color_blend),
                        color_blend_op: Self::to_vk_blend_op(state.color_blend_op),
                        src_alpha_blend_factor: Self::to_vk_blend_factor(state.src_alpha_blend),
                        dst_alpha_blend_factor: Self::to_vk_blend_factor(state.dst_alpha_blend),
                        alpha_blend_op: Self::to_vk_blend_op(state.alpha_blend_op),
                        color_write_mask: vk::ColorComponentFlags::from_raw(state.write_mask),
                    }
                } else {
                    vk::PipelineColorBlendAttachmentState::default()
                        .color_write_mask(vk::ColorComponentFlags::from_raw(state.write_mask))
                }
            })
            .collect()
    }
    
    fn to_vk_blend_factor(blend: BlendMode) -> vk::BlendFactor {
        match blend {
            BlendMode::Zero => vk::BlendFactor::ZERO,
            BlendMode::One => vk::BlendFactor::ONE,
            BlendMode::SrcColor => vk::BlendFactor::SRC_COLOR,
            BlendMode::OneMinusSrcColor => vk::BlendFactor::ONE_MINUS_SRC_COLOR,
            BlendMode::DstColor => vk::BlendFactor::DST_COLOR,
            BlendMode::OneMinusDstColor => vk::BlendFactor::ONE_MINUS_DST_COLOR,
            BlendMode::SrcAlpha => vk::BlendFactor::SRC_ALPHA,
            BlendMode::OneMinusSrcAlpha => vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            BlendMode::DstAlpha => vk::BlendFactor::DST_ALPHA,
            BlendMode::OneMinusDstAlpha => vk::BlendFactor::ONE_MINUS_DST_ALPHA,
        }
    }
    
    fn to_vk_blend_op(op: BlendOp) -> vk::BlendOp {
        match op {
            BlendOp::Add => vk::BlendOp::ADD,
            BlendOp::Subtract => vk::BlendOp::SUBTRACT,
            BlendOp::ReverseSubtract => vk::BlendOp::REVERSE_SUBTRACT,
            BlendOp::Min => vk::BlendOp::MIN,
            BlendOp::Max => vk::BlendOp::MAX,
        }
    }
    
    fn build_depth_stencil_state(depth: &DepthState, stencil: &StencilState) -> vk::PipelineDepthStencilStateCreateInfo<'static> {
        vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(if depth.enabled { vk::TRUE } else { vk::FALSE })
            .depth_write_enable(if depth.write_enabled { vk::TRUE } else { vk::FALSE })
            .depth_compare_op(Self::to_vk_compare_op(depth.compare_func))
            .depth_bounds_test_enable(vk::FALSE)
            .stencil_test_enable(if stencil.enabled { vk::TRUE } else { vk::FALSE })
            .front(Self::to_vk_stencil_op_state(&stencil.front_face))
            .back(Self::to_vk_stencil_op_state(&stencil.back_face))
    }
    
    fn to_vk_compare_op(func: CompareFunc) -> vk::CompareOp {
        match func {
            CompareFunc::Never => vk::CompareOp::NEVER,
            CompareFunc::Less => vk::CompareOp::LESS,
            CompareFunc::Equal => vk::CompareOp::EQUAL,
            CompareFunc::LessEqual => vk::CompareOp::LESS_OR_EQUAL,
            CompareFunc::Greater => vk::CompareOp::GREATER,
            CompareFunc::NotEqual => vk::CompareOp::NOT_EQUAL,
            CompareFunc::GreaterEqual => vk::CompareOp::GREATER_OR_EQUAL,
            CompareFunc::Always => vk::CompareOp::ALWAYS,
        }
    }
    
    fn to_vk_stencil_op_state(face: &StencilFaceState) -> vk::StencilOpState {
        vk::StencilOpState {
            fail_op: Self::to_vk_stencil_op(face.fail_op),
            pass_op: Self::to_vk_stencil_op(face.pass_op),
            depth_fail_op: Self::to_vk_stencil_op(face.depth_fail_op),
            compare_op: Self::to_vk_compare_op(face.compare_func),
            ..Default::default()
        }
    }
    
    fn to_vk_stencil_op(op: StencilOp) -> vk::StencilOp {
        match op {
            StencilOp::Keep => vk::StencilOp::KEEP,
            StencilOp::Zero => vk::StencilOp::ZERO,
            StencilOp::Replace => vk::StencilOp::REPLACE,
            StencilOp::IncrementClamp => vk::StencilOp::INCREMENT_AND_CLAMP,
            StencilOp::DecrementClamp => vk::StencilOp::DECREMENT_AND_CLAMP,
            StencilOp::Invert => vk::StencilOp::INVERT,
            StencilOp::IncrementWrap => vk::StencilOp::INCREMENT_AND_WRAP,
            StencilOp::DecrementWrap => vk::StencilOp::DECREMENT_AND_WRAP,
        }
    }
    
    fn build_rasterizer_state(rasterizer: &RasterizerState) -> vk::PipelineRasterizationStateCreateInfo {
        vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(vk::FALSE)
            .rasterizer_discard_enable(vk::FALSE)
            .polygon_mode(match rasterizer.fill_mode {
                crate::graphics::rhi::types::FillMode::Solid => vk::PolygonMode::FILL,
                crate::graphics::rhi::types::FillMode::Wireframe => vk::PolygonMode::LINE,
                crate::graphics::rhi::types::FillMode::Point => vk::PolygonMode::POINT,
            })
            .cull_mode(match rasterizer.cull_mode {
                CullMode::None => vk::CullModeFlags::NONE,
                CullMode::Front => vk::CullModeFlags::FRONT,
                CullMode::Back => vk::CullModeFlags::BACK,
            })
            .front_face(match rasterizer.front_face {
                FrontFace::CounterClockwise => vk::FrontFace::COUNTER_CLOCKWISE,
                FrontFace::Clockwise => vk::FrontFace::CLOCKWISE,
            })
            .line_width(1.0)
    }
    
    fn to_vk_primitive_topology(topology: PrimitiveTopology) -> vk::PrimitiveTopology {
        match topology {
            PrimitiveTopology::PointList => vk::PrimitiveTopology::POINT_LIST,
            PrimitiveTopology::LineList => vk::PrimitiveTopology::LINE_LIST,
            PrimitiveTopology::LineStrip => vk::PrimitiveTopology::LINE_STRIP,
            PrimitiveTopology::TriangleList => vk::PrimitiveTopology::TRIANGLE_LIST,
            PrimitiveTopology::TriangleStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
        }
    }
    
    fn build_shader_stages(device: &ash::Device, desc: &PipelineStateObject) -> Vec<vk::PipelineShaderStageCreateInfo<'static>> {
        let mut stages = Vec::new();
        
        // Vertex shader
        if let Some(stage_info) = Self::create_shader_stage(device, &desc.vertex_shader, vk::ShaderStageFlags::VERTEX) {
            stages.push(stage_info);
        }
        
        // Fragment shader
        if let Some(fs_handle) = desc.fragment_shader {
            if let Some(stage_info) = Self::create_shader_stage(device, &fs_handle, vk::ShaderStageFlags::FRAGMENT) {
                stages.push(stage_info);
            }
        }
        
        stages
    }
    
    fn create_shader_stage(
        device: &ash::Device,
        shader_handle: &ResourceHandle,
        stage: vk::ShaderStageFlags,
    ) -> Option<vk::PipelineShaderStageCreateInfo<'static>> {
        // Получаем SPIR-V байткод из ResourceManager
        // В реальном использовании здесь должен быть доступ к ResourceManager
        // Для демонстрации создаем заглушку с правильными полями
        
        // Note: В production коде здесь нужно получить ResourceManager из контекста устройства
        // и извлечь spirv_bytecode по shader_handle
        // Пример: let shader_data = resource_manager.get_shader(*shader_handle)?;
        //         let spirv_code = shader_data.spirv_bytecode;
        // Это требует рефакторинга передачи ResourceManager в pipeline creation
        
        // Для текущей реализации возвращаем None, так как нет доступа к ResourceManager
        None
    }
    
    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }
    
    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }
    
    pub fn handle(&self) -> ResourceHandle {
        self.handle
    }
    
    pub fn description(&self) -> &PipelineStateObject {
        &self.description
    }
}