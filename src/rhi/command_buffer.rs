use crate::rhi::types::{IndexFormat, UniformValue};

#[derive(Debug, Clone)]
pub enum RenderCommand {
    SetPipeline { pipeline_id: u64 },
    SetVertexBuffer { slot: u8, buffer_id: u64, offset: usize },
    SetIndexBuffer { buffer_id: u64, format: IndexFormat },
    SetUniform { name: String, value: UniformValue },
    SetTexture { slot: u8, texture_id: u64 },
    SetFramebuffer { fb_id: Option<u64> },
    SetViewport { x: i32, y: i32, w: i32, h: i32 },
    SetScissor { x: i32, y: i32, w: i32, h: i32 },
    ClearColor { r: f32, g: f32, b: f32, a: f32 },
    ClearDepth { depth: f32 },
    Draw { vertices: u32, instances: u32 },
    DrawIndexed { indices: u32, instances: u32 },
}

pub struct CommandBuffer {
    pub commands: Vec<RenderCommand>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self {
            commands: Vec::with_capacity(256),
        }
    }

    pub fn push(&mut self, cmd: RenderCommand) {
        self.commands.push(cmd);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}
