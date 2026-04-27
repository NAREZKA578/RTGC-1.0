//! Physics Thread - Dedicated thread for physics simulation
//! 
//! This module handles:
//! - Running physics simulation in a separate thread
//! - Communication with EngineHub via channels
//! - Fixed timestep physics updates

use crate::core::engine_hub::{PhysicsCommand, PhysicsMessage};
use crossbeam_channel::{Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tracing::{debug, error, info, warn};

/// Physics simulation state (future expansion)
pub struct PhysicsWorld {
    // Future: rigid bodies, constraints, vehicle simulations
    time_accumulator: f32,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            time_accumulator: 0.0,
        }
    }
    
    /// Step the physics simulation
    pub fn step(&mut self, delta_time: f32) {
        // Fixed timestep for stability (60 Hz = 0.01667s)
        const FIXED_TIMESTEP: f32 = 1.0 / 60.0;
        
        self.time_accumulator += delta_time;
        
        while self.time_accumulator >= FIXED_TIMESTEP {
            // TODO: Implement actual physics step here
            // - Integrate velocities
            // - Collision detection
            // - Constraint solving
            // - Vehicle physics
            
            self.time_accumulator -= FIXED_TIMESTEP;
        }
    }
}

/// Physics thread handle
pub struct PhysicsThread {
    handle: Option<JoinHandle<()>>,
    shutdown_flag: Arc<AtomicBool>,
}

impl PhysicsThread {
    /// Create and start the physics thread
    pub fn new(
        tx_to_core: Sender<PhysicsMessage>,
        rx_from_core: Receiver<PhysicsCommand>,
    ) -> Self {
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let shutdown_clone = Arc::clone(&shutdown_flag);
        
        let handle = thread::spawn(move || {
            Self::physics_loop(tx_to_core, rx_from_core, shutdown_clone);
        });
        
        Self {
            handle: Some(handle),
            shutdown_flag,
        }
    }
    
    /// Main physics loop running in dedicated thread
    fn physics_loop(
        tx_to_core: Sender<PhysicsMessage>,
        rx_from_core: Receiver<PhysicsCommand>,
        shutdown_flag: Arc<AtomicBool>,
    ) {
        info!("Physics thread started");
        let mut physics_world = PhysicsWorld::new();
        
        while !shutdown_flag.load(Ordering::Relaxed) {
            // Check for commands from EngineHub (non-blocking)
            while let Ok(cmd) = rx_from_core.try_recv() {
                match cmd {
                    PhysicsCommand::Step(delta_time) => {
                        physics_world.step(delta_time);
                        
                        // Notify EngineHub of completion
                        if let Err(e) = tx_to_core.send(PhysicsMessage::StepComplete(delta_time)) {
                            warn!("Failed to send physics message to core: {}", e);
                            break;
                        }
                    }
                    PhysicsCommand::Shutdown => {
                        info!("Physics thread received shutdown command");
                        shutdown_flag.store(true, Ordering::Relaxed);
                    }
                }
            }
            
            // Small sleep to prevent busy-waiting when idle
            thread::sleep(std::time::Duration::from_millis(1));
        }
        
        info!("Physics thread shutting down");
    }
    
    /// Graceful shutdown
    pub fn shutdown(&mut self) {
        self.shutdown_flag.store(true, Ordering::Relaxed);
        
        if let Some(handle) = self.handle.take() {
            if let Err(e) = handle.join() {
                error!("Failed to join physics thread: {:?}", e);
            }
        }
    }
}

impl Drop for PhysicsThread {
    fn drop(&mut self) {
        self.shutdown();
    }
}
