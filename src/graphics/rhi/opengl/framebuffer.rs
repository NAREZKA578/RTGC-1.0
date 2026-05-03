use crate::graphics::rhi::traits::RhiFramebuffer;
use glow::NativeFramebuffer;

pub struct GlFramebuffer {
    pub id: u64,
    pub gl_fbo: NativeFramebuffer,
    pub width: u32,
    pub height: u32,
}

impl RhiFramebuffer for GlFramebuffer {
    fn bind(&self) {}
    fn unbind(&self) {}
}
