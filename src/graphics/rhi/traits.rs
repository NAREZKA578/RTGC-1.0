use crate::graphics::rhi::command_buffer::CommandBuffer;
use crate::graphics::rhi::types::*;
use anyhow::Result;
use std::any::Any;

pub trait RhiDevice: Send + Sync {
    fn create_buffer(&self, desc: &BufferDesc) -> Box<dyn RhiBuffer>;
    fn create_texture(&self, desc: &TextureDesc) -> Box<dyn RhiTexture>;
    fn create_shader(&self, vert_src: &str, frag_src: &str, label: &str) -> Result<Box<dyn RhiShader>>;
    fn create_pipeline(&self, shader: &dyn RhiShader, blend: BlendMode, depth_test: bool, depth_write: bool, cull: CullMode) -> Box<dyn RhiPipeline>;
    fn create_framebuffer(&self, attachments: &[&dyn RhiTexture]) -> Box<dyn RhiFramebuffer>;

    fn upload_buffer(&self, buf: &dyn RhiBuffer, offset: usize, data: &[u8]);
    fn upload_texture(&self, tex: &dyn RhiTexture, data: &[u8]);
    fn generate_mipmaps(&self, tex: &dyn RhiTexture);

    fn begin_frame(&mut self);
    fn end_frame(&mut self);
    fn submit(&mut self, cmds: &CommandBuffer);

    fn backend_name(&self) -> &str;
    fn capabilities(&self) -> DeviceCaps;
}

pub trait RhiBuffer: Send + Sync {
    fn id(&self) -> u64;
    fn as_any(&self) -> &dyn Any;
}

pub trait RhiTexture: Send + Sync {
    fn id(&self) -> u64;
    fn size(&self) -> (u32, u32);
    fn as_any(&self) -> &dyn Any;
}

pub trait RhiShader: Send + Sync {
    fn id(&self) -> u64;
    fn as_any(&self) -> &dyn Any;
}

pub trait RhiPipeline: Send + Sync {
    fn id(&self) -> u64;
}

pub trait RhiFramebuffer: Send + Sync {
    fn bind(&self);
    fn unbind(&self);
}
