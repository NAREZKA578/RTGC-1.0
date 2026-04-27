//! Render Passes - общие render passes (main, shadow, post)

use crate::graphics::rhi::{ResourceHandle, RenderPassDescription, RenderAttachment, LoadOp, StoreOp, ClearValue};

/// Trait для render pass
pub trait RenderPass {
    fn description(&self) -> RenderPassDescription;
}

/// Основной render pass для рендеринга сцены
pub struct MainRenderPass {
    pub color_attachment: ResourceHandle,
    pub depth_attachment: ResourceHandle,
    pub clear_color: [f32; 4],
    pub clear_depth: f32,
    pub width: u32,
    pub height: u32,
}

/// Alias для совместимости
pub type MainPass = MainRenderPass;

impl MainRenderPass {
    pub fn new(color_attachment: ResourceHandle, depth_attachment: ResourceHandle, width: u32, height: u32) -> Self {
        Self {
            color_attachment,
            depth_attachment,
            clear_color: [0.1, 0.1, 0.15, 1.0],
            clear_depth: 1.0,
            width,
            height,
        }
    }
    
    pub fn description(&self) -> RenderPassDescription {
        RenderPassDescription {
            color_attachments: vec![RenderAttachment {
                view: self.color_attachment,
                load_op: LoadOp::Clear,
                store_op: StoreOp::Store,
                clear_value: Some(ClearValue::Color(self.clear_color)),
            }],
            depth_stencil_attachment: Some(crate::graphics::rhi::DepthStencilAttachment {
                view: self.depth_attachment,
                depth_load_op: LoadOp::Clear,
                depth_store_op: StoreOp::Store,
                stencil_load_op: LoadOp::Discard,
                stencil_store_op: StoreOp::Discard,
                depth_clear_value: Some(self.clear_depth),
                stencil_clear_value: None,
            }),
            width: self.width,
            height: self.height,
        }
    }
}

impl RenderPass for MainRenderPass {
    fn description(&self) -> RenderPassDescription {
        self.description()
    }
}

/// Render pass для теней (shadow map)
pub struct ShadowRenderPass {
    pub depth_attachment: ResourceHandle,
    pub width: u32,
    pub height: u32,
}

/// Alias для совместимости
pub type ShadowPass = ShadowRenderPass;

impl ShadowRenderPass {
    pub fn new(depth_attachment: ResourceHandle, width: u32, height: u32) -> Self {
        Self { depth_attachment, width, height }
    }
    
    pub fn description(&self) -> RenderPassDescription {
        RenderPassDescription {
            color_attachments: vec![],
            depth_stencil_attachment: Some(crate::graphics::rhi::DepthStencilAttachment {
                view: self.depth_attachment,
                depth_load_op: LoadOp::Clear,
                depth_store_op: StoreOp::Store,
                stencil_load_op: LoadOp::Discard,
                stencil_store_op: StoreOp::Discard,
                depth_clear_value: Some(1.0),
                stencil_clear_value: None,
            }),
            width: self.width,
            height: self.height,
        }
    }
}

impl RenderPass for ShadowRenderPass {
    fn description(&self) -> RenderPassDescription {
        self.description()
    }
}

/// Post-processing render pass
pub struct PostProcessRenderPass {
    pub input_attachment: ResourceHandle,
    pub output_attachment: ResourceHandle,
    pub width: u32,
    pub height: u32,
}

/// Alias для совместимости
pub type PostProcessPass = PostProcessRenderPass;

impl PostProcessRenderPass {
    pub fn new(input_attachment: ResourceHandle, output_attachment: ResourceHandle, width: u32, height: u32) -> Self {
        Self {
            input_attachment,
            output_attachment,
            width,
            height,
        }
    }
    
    pub fn description(&self) -> RenderPassDescription {
        RenderPassDescription {
            color_attachments: vec![RenderAttachment {
                view: self.output_attachment,
                load_op: LoadOp::Clear,
                store_op: StoreOp::Store,
                clear_value: Some(ClearValue::Color([0.0, 0.0, 0.0, 1.0])),
            }],
            depth_stencil_attachment: None,
            width: self.width,
            height: self.height,
        }
    }
}

impl RenderPass for PostProcessRenderPass {
    fn description(&self) -> RenderPassDescription {
        self.description()
    }
}
