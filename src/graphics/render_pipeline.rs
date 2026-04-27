//! Render Pipeline - High-level rendering logic
//! 
//! This module handles:
//! - Frame rendering orchestration
//! - Camera management
//! - Scene graph traversal (future)
//! - Communication with RHI layer

use crate::graphics::rhi::gl_rhi_backend::GlRhiBackend;
use glow::HasContext;
use std::error::Error;
use tracing::{debug, error};
use winit::window::Window;

/// Main RenderPipeline struct
pub struct RenderPipeline {
    rhi_backend: GlRhiBackend,
    clear_color: [f32; 4],
}

impl RenderPipeline {
    /// Create a new RenderPipeline with the given window
    pub fn new(window: Box<dyn Window>) -> Result<Self, Box<dyn Error>> {
        debug!("Creating RenderPipeline");
        
        // Downcast to concrete type for glutin compatibility
        let window_ptr = window.as_any().downcast_ref::<winit::window::Window>()
            .ok_or("Failed to downcast window")?;
        
        // Initialize RHI backend (OpenGL via glutin/glow)
        let rhi_backend = GlRhiBackend::new(window_ptr)?;
        
        Ok(Self {
            rhi_backend,
            clear_color: [0.1, 0.1, 0.15, 1.0], // Dark blue-grey sky
        })
    }
    
    /// Render a frame
    pub fn render(&mut self, delta_time: f32) {
        let ctx = &self.rhi_backend.gl;
        
        // Clear screen
        unsafe {
            ctx.clear_color(
                self.clear_color[0],
                self.clear_color[1],
                self.clear_color[2],
                self.clear_color[3],
            );
            ctx.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
        
        // TODO: Add actual rendering passes here
        // - Terrain pass
        // - Vehicle pass
        // - UI pass
        
        debug!("Frame rendered in {}s", delta_time);
        
        // Swap buffers
        self.rhi_backend.swap_buffers();
    }
    
    /// Set clear color (background)
    pub fn set_clear_color(&mut self, color: [f32; 4]) {
        self.clear_color = color;
    }
    
    /// Get reference to RHI backend for advanced operations
    pub fn rhi_backend(&self) -> &GlRhiBackend {
        &self.rhi_backend
    }
}
