use crate::graphics::rhi::traits::RhiBuffer;
use crate::graphics::rhi::types::BufferUsage;
use glow::NativeBuffer;
use std::any::Any;

pub struct GlBuffer {
    pub id: u64,
    pub gl_buffer: NativeBuffer,
    pub size: usize,
    pub usage: BufferUsage,
}

impl RhiBuffer for GlBuffer {
    fn id(&self) -> u64 { self.id }
    fn as_any(&self) -> &dyn Any { self }
}
