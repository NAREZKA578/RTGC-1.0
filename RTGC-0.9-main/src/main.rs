// RTGC-0.9 Main Entry Point
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Initialize logging
    rtgc::utils::logger::init_logger();

    eprintln!("========================================");
    eprintln!("RTGC Starting... v0.9.0");
    eprintln!("========================================");

    // Create and run the engine
    match rtgc::engine::core::Engine::new() {
        Ok(mut engine) => {
            if let Err(e) = engine.run() {
                eprintln!("Engine error: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to create engine: {}", e);
        }
    }
}