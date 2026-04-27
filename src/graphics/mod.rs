//! RTGC-1.0 Graphics Module
//! Rendering pipeline with RHI abstraction

pub mod render_pipeline;
pub mod rhi;

pub use render_pipeline::RenderPipeline;
pub use rhi::gl_rhi_backend::GlRhiBackend;
