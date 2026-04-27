//! OpenGL RHI Backend Implementation
//! 
//! Uses glutin for context creation and glow for OpenGL bindings.

use glow::HasContext;
use std::error::Error;
use winit::window::Window;

/// OpenGL RHI Backend
pub struct GlRhiBackend {
    pub gl: glow::Context,
    surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    context: glutin::context::PossiblyCurrentContext,
    display: glutin::display::Display,
}

impl GlRhiBackend {
    /// Create a new OpenGL RHI backend
    pub fn new(window: &Window) -> Result<Self, Box<dyn Error>> {
        use glutin::context::ContextApi;
        use glutin::prelude::*;
        
        // Create glutin config
        let template = glutin::config::ConfigTemplateBuilder::new()
            .compatible_with_native_window(window.as_raw().clone())
            .build();
        
        // Get display
        let display_builder = glutin_winit::DisplayBuilder::new().with_window_attributes(None);
        let (display, config_iterator) = display_builder.build(window, Some(template))?;
        let display = display.ok_or("Failed to create glutin display")?;
        
        // Find config
        let config = config_iterator.next().ok_or("No suitable config found")?;
        
        // Create context
        let context_attributes = glutin::context::ContextAttributesBuilder::new()
            .build(Some(window.as_raw().clone()));
        
        let fallback_context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(window.as_raw().clone()));
        
        let context = unsafe {
            display.create_context(&config, &context_attributes)
                .or_else(|_| display.create_context(&config, &fallback_context_attributes))?
        };
        
        // Create surface
        let attrs = window.build_surface_attributes(<_>::default());
        let surface = unsafe {
            display.create_window_surface(&config, &attrs)?
        };
        
        // Make context current
        let context = context.make_current(&surface)?;
        
        // Create glow context
        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                display.get_proc_address(std::ffi::CString::new(s).unwrap().as_c_str())
                    as *const _
            })
        };
        
        Ok(Self {
            gl,
            surface,
            context,
            display,
        })
    }
    
    /// Swap buffers (present frame)
    pub fn swap_buffers(&self) {
        if let Err(e) = self.surface.swap_buffers(&self.context) {
            tracing::warn!("Failed to swap buffers: {:?}", e);
        }
    }
    
    /// Resize the surface
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface.resize(
                &self.context,
                width,
                height,
            );
        }
    }
}
