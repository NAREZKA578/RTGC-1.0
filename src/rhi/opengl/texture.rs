use crate::rhi::traits::RhiTexture;
use glow::NativeTexture;
use std::any::Any;

pub struct GlTexture {
    pub id: u64,
    pub gl_texture: NativeTexture,
    pub width: u32,
    pub height: u32,
}

impl RhiTexture for GlTexture {
    fn id(&self) -> u64 {
        self.id
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
