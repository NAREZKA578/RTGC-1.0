//! RTGC-1.0 Core Engine Library
//! 
//! Modular architecture with clear separation:
//! - `core`: Central orchestration (engine_hub.rs)
//! - `graphics`: Rendering with RHI abstraction (render_pipeline.rs + rhi/)
//! - `physics`: Physics simulation in dedicated thread (physics_thread.rs)

pub mod core;
pub mod graphics;
pub mod physics;

pub use core::engine_hub::EngineHub;
