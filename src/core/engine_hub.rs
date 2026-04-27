//! Engine Hub - The central hub where everything connects and navigates
//! 
//! This struct orchestrates:
//! - Graphics/Render subsystem
//! - Physics subsystem (running in separate thread)
//! - Input handling
//! - Game loop management

use crate::graphics::render_pipeline::RenderPipeline;
use crate::physics::physics_thread::PhysicsThread;
use crossbeam_channel::{bounded, Receiver, Sender};
use tracing::{debug, info, warn};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// Messages from Physics thread to EngineHub
#[derive(Debug, Clone)]
pub enum PhysicsMessage {
    StepComplete(f32), // Delta time processed
    Error(String),
}

/// Messages from EngineHub to Physics thread
#[derive(Debug, Clone)]
pub enum PhysicsCommand {
    Step(f32), // Delta time
    Shutdown,
}

/// Main Engine hub struct
pub struct EngineHub {
    window: Option<Box<dyn Window>>,
    render_pipeline: Option<RenderPipeline>,
    physics_thread: Option<PhysicsThread>,
    physics_tx: Sender<PhysicsCommand>,
    physics_rx: Receiver<PhysicsMessage>,
    is_running: bool,
    last_frame_time: std::time::Instant,
}

impl EngineHub {
    /// Create a new EngineHub instance
    pub fn new() -> Self {
        debug!("Initializing RTGC-1.0 EngineHub");
        
        let (physics_tx, core_physics_rx) = bounded::<PhysicsCommand>(16);
        let (core_physics_tx, physics_rx) = bounded::<PhysicsMessage>(16);
        
        let physics_thread = PhysicsThread::new(core_physics_tx, physics_rx);
        
        Self {
            window: None,
            render_pipeline: None,
            physics_thread: Some(physics_thread),
            physics_tx,
            physics_rx: core_physics_rx,
            is_running: false,
            last_frame_time: std::time::Instant::now(),
        }
    }
    
    /// Run the engine with an event loop
    pub fn run(mut self, event_loop: EventLoop<()>) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting RTGC-1.0 engine");
        self.is_running = true;
        
        let mut app = AppWrapper { hub: &mut self };
        event_loop.run_app(&mut app)?;
        
        info!("Engine shutdown complete");
        Ok(())
    }
    
    /// Called when window is created
    pub fn on_window_created(&mut self, window: Box<dyn Window>) {
        debug!("Window created, initializing render pipeline");
        
        // Initialize render pipeline with the window
        match RenderPipeline::new(window) {
            Ok(pipeline) => {
                self.render_pipeline = Some(pipeline);
            }
            Err(e) => {
                warn!("Failed to initialize render pipeline: {}", e);
            }
        }
    }
    
    /// Called each frame for rendering
    pub fn on_frame(&mut self) {
        let now = std::time::Instant::now();
        let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        
        // Send physics step command
        if let Some(ref _physics) = self.physics_thread {
            let _ = self.physics_tx.try_send(PhysicsCommand::Step(delta_time));
        }
        
        // Receive physics messages (non-blocking)
        while let Ok(msg) = self.physics_rx.try_recv() {
            match msg {
                PhysicsMessage::StepComplete(dt) => {
                    debug!("Physics step completed: {}s", dt);
                }
                PhysicsMessage::Error(e) => {
                    warn!("Physics error: {}", e);
                }
            }
        }
        
        // Render frame
        if let Some(ref mut pipeline) = self.render_pipeline {
            pipeline.render(delta_time);
        }
    }
    
    /// Graceful shutdown
    pub fn shutdown(&mut self) {
        info!("Shutting down EngineHub...");
        self.is_running = false;
        
        let _ = self.physics_tx.send(PhysicsCommand::Shutdown);
        
        if let Some(mut physics) = self.physics_thread.take() {
            physics.shutdown();
        }
        
        self.render_pipeline = None;
        self.window = None;
    }
}

/// Wrapper to implement ApplicationHandler for EngineHub
struct AppWrapper<'a> {
    hub: &'a mut EngineHub,
}

impl ApplicationHandler for AppWrapper<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.hub.window.is_none() {
            let window_attrs = Window::default_attributes()
                .with_title("RTGC-1.0 - Russian Open World Simulator")
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
            
            match event_loop.create_window(window_attrs) {
                Ok(window) => {
                    self.hub.on_window_created(Box::new(window));
                }
                Err(e) => {
                    warn!("Failed to create window: {}", e);
                    event_loop.exit();
                }
            }
        }
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested, shutting down");
                self.hub.shutdown();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.hub.on_frame();
                if self.hub.is_running {
                    if let Some(ref window) = self.hub.window {
                        window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}
