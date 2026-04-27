//! Graphics Module for RTGC-0.9
//! Provides rendering, camera, shaders, meshes, textures, and RHI abstraction

pub mod camera;
pub mod material;
pub mod particles;
pub mod renderer;
pub mod lighting;
pub mod rhi;
pub mod gl_context;
pub mod shader;
pub mod mesh;
pub mod font;
pub mod resources;
pub mod texture;
pub mod render_command;
pub mod render_queue;
pub mod terrain_mesh_builder;
pub mod terrain_renderer;
pub mod sky_renderer;

pub use camera::Camera;
pub use material::{MaterialManager, TextureQuality};
pub use particles::ParticleSystem;
pub use renderer::{Renderer, RenderCommand, RendererConfig, SceneRenderer, SceneRendererStats};
pub use render_command::RenderCommand as RCmd;
pub use renderer::commands::UiCommand;
pub use render_queue::RenderQueue;
pub use gl_context::GlContext;
pub use shader::{load_shader_from_file, load_vertex_shader, load_fragment_shader};
pub use mesh::{Mesh, SimpleVertex};
pub use font::{FontAtlas, FontManager, GlyphData};
pub use terrain_mesh_builder::TerrainMeshBuilder;
pub use terrain_renderer::TerrainRenderer;
pub use sky_renderer::SkyRenderer;

/// Универсальный графический контекст
pub enum GraphicsContext {
    OpenGL(GlContext),
}

impl GraphicsContext {
    /// Создать OpenGL контекст
    pub fn new_opengl(ctx: GlContext) -> Self {
        Self::OpenGL(ctx)
    }
    
    /// Получить GL контекст если это OpenGL
    pub fn as_gl(&self) -> Option<&GlContext> {
        match self {
            Self::OpenGL(ctx) => Some(ctx),
        }
    }
    
    /// Получить GL контекст если это OpenGL (mutable)
    pub fn as_gl_mut(&mut self) -> Option<&mut GlContext> {
        match self {
            Self::OpenGL(ctx) => Some(ctx),
        }
    }

    /// Get window width
    pub fn width(&self) -> u32 {
        match self {
            Self::OpenGL(ctx) => ctx.size().0,
        }
    }

    /// Get window height
    pub fn height(&self) -> u32 {
        match self {
            Self::OpenGL(ctx) => ctx.size().1,
        }
    }
}
