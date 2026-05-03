use crate::renderer::ui_renderer::batch::{Color, DrawBatch, Rect};
use glow::HasContext;

pub struct RectRenderer {
    gl: glow::Context,
    shader: Option<glow::NativeProgram>,
}

impl RectRenderer {
    pub fn new(gl: glow::Context) -> Self {
        Self { gl, shader: None }
    }

    pub fn render(&self, _batch: &mut DrawBatch, _screen_width: f32, _screen_height: f32) {
        // TODO: Implement rect rendering with shader
    }
}
