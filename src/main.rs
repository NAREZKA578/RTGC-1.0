//входная точка игры!

use rtgc::EngineHub;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();
    
    tracing::info!("RTGC-1.0 Starting...");
    
    // Create event loop
    let event_loop = EventLoop::new()?;
    
    // Create and run EngineHub
    let hub = EngineHub::new();
    hub.run(event_loop)?;
    
    Ok(())
}
