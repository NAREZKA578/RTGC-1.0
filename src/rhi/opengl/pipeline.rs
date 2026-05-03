use crate::rhi::traits::RhiPipeline;
use glow::NativeProgram;

pub struct GlPipeline {
    pub id: u64,
    pub program: NativeProgram,
    pub depth_test: bool,
    pub depth_write: bool,
    pub cull_enabled: bool,
    pub cull_face: u32,
}

impl RhiPipeline for GlPipeline {
    fn id(&self) -> u64 {
        self.id
    }
}
