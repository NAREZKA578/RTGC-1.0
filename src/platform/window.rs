use anyhow::{Context, Result};
use glow::HasContext;
use glutin::config::{ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::DisplayBuilder;
use std::num::NonZeroU32;
use winit::raw_window_handle::HasWindowHandle;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowAttributes};

pub struct GlSurface {
    pub surface: Surface<WindowSurface>,
    pub context: PossiblyCurrentContext,
    pub display: glutin::display::Display,
}

pub fn create_window(
    event_loop: &EventLoop<()>,
    width: u32,
    height: u32,
    title: &str,
) -> Result<(Window, GlSurface, glow::Context)> {
    let window_attributes = WindowAttributes::default()
        .with_title(title)
        .with_inner_size(PhysicalSize::new(width, height))
        .with_visible(false)
        .with_resizable(true);

    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8);

    let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

    let (window, gl_config) = display_builder
        .build(event_loop, template, |mut configs| {
            configs
                .reduce(|accum, config| {
                    if config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .map_err(|e| anyhow::anyhow!("Failed to create OpenGL config: {}", e))?;

    let window = window.ok_or_else(|| anyhow::anyhow!("Failed to create window"))?;

    let raw_window_handle = window
        .window_handle()
        .context("Failed to get window handle")?
        .as_raw();

    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
        .build(Some(raw_window_handle));

    let gl_display = gl_config.display();
    let not_current_context = unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .context("Failed to create OpenGL context")?
    };

    let surface_attrs = glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new()
        .build(
            raw_window_handle,
            std::num::NonZeroU32::new(width).unwrap(),
            std::num::NonZeroU32::new(height).unwrap(),
        );

    let surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &surface_attrs)
            .context("Failed to create surface")?
    };

    let context = not_current_context
        .make_current(&surface)
        .context("Failed to make context current")?;

    // Store display for swap_buffers
    let display = gl_config.display();

    window.set_visible(true);

    let gl_context = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = std::ffi::CString::new(s).unwrap();
            gl_config
                .display()
                .get_proc_address(s.as_c_str())
                .cast()
        })
    };

    unsafe {
        gl_context.viewport(0, 0, width as i32, height as i32);
    }

    Ok((window, GlSurface { surface, context, display }, gl_context))
}
