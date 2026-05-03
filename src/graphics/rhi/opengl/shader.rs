use crate::graphics::rhi::traits::RhiShader;
use glow::NativeProgram;
use std::any::Any;

pub struct GlShader {
    pub id: u64,
    pub program: NativeProgram,
}

impl RhiShader for GlShader {
    fn id(&self) -> u64 { self.id }
    fn as_any(&self) -> &dyn Any { self }
}
