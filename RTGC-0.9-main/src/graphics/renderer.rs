//! Renderer module - main entry point for rendering subsystem
//! 
//! This module provides a modular renderer based on RHI with separation of:
//! - Scene rendering (3D)
//! - UI rendering
//! - Debug visualization
//! - Render passes

pub mod commands;
pub mod debug;
pub mod passes;
pub mod pipeline_cache;
pub mod scene;
pub mod ui;
pub mod renderer;

pub use commands::{RenderCommand, UiCommand, RendererConfig};
pub use debug::DebugRenderer;
pub use passes::{RenderPass, MainPass, ShadowPass, PostProcessPass, MainRenderPass, ShadowRenderPass, PostProcessRenderPass};
pub use pipeline_cache::{PipelineCache, PipelineKey, PipelineCacheStats};
pub use scene::{SceneRenderer, SceneRendererStats};
pub use ui::UIRenderer;
pub use renderer::Renderer;
pub use crate::graphics::mesh::Mesh;
