//! Time utilities for RTGC-0.9
//! Provides delta time calculation, fixed timestep, and FPS counter

use std::time::{Duration, Instant};

/// Time manager for handling delta time and fixed timestep updates
pub struct TimeManager {
    last_frame_time: Instant,
    accumulator: Duration,
    fixed_timestep: Duration,
    frame_count: u32,
    fps_update_timer: Instant,
    fps: f32,
}

impl TimeManager {
    /// Creates a new TimeManager with specified fixed timestep
    pub fn new(fixed_timestep_secs: f32) -> Self {
        let now = Instant::now();
        Self {
            last_frame_time: now,
            accumulator: Duration::ZERO,
            fixed_timestep: Duration::from_secs_f32(fixed_timestep_secs),
            frame_count: 0,
            fps_update_timer: now,
            fps: 0.0,
        }
    }

    /// Default constructor with 60Hz fixed timestep (1/60 = 0.01667s)
    pub fn default() -> Self {
        Self::new(1.0 / 60.0)
    }

    /// Updates the timer and returns delta time since last frame
    pub fn update(&mut self) -> Duration {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        
        self.accumulator += delta;
        self.frame_count += 1;
        
        // Update FPS every second
        if self.fps_update_timer.elapsed() >= Duration::from_secs(1) {
            self.fps = self.frame_count as f32 / self.fps_update_timer.elapsed().as_secs_f32();
            self.frame_count = 0;
            self.fps_update_timer = now;
        }
        
        delta
    }

    /// Returns whether enough time has accumulated for a fixed step update
    pub fn should_fixed_step(&self) -> bool {
        self.accumulator >= self.fixed_timestep
    }

    /// Consumes one fixed timestep from the accumulator
    pub fn consume_fixed_step(&mut self) {
        if self.accumulator >= self.fixed_timestep {
            self.accumulator -= self.fixed_timestep;
        }
    }

    /// Returns the current delta time in seconds
    pub fn delta_time(&self) -> f32 {
        self.last_frame_time.elapsed().as_secs_f32()
    }

    /// Returns the fixed timestep duration
    pub fn fixed_timestep(&self) -> Duration {
        self.fixed_timestep
    }

    /// Returns the current FPS
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// Returns the interpolation factor for rendering between physics steps
    pub fn interpolation_alpha(&self) -> f32 {
        let total = self.fixed_timestep.as_secs_f32();
        let remaining = self.accumulator.as_secs_f32();
        (remaining / total).clamp(0.0, 1.0)
    }

    /// Reset the timer (useful after loading screens)
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.last_frame_time = now;
        self.accumulator = Duration::ZERO;
        self.frame_count = 0;
        self.fps_update_timer = now;
    }
}

impl Default for TimeManager {
    fn default() -> Self {
        Self::new(1.0 / 60.0)
    }
}

/// Simple FPS counter
pub struct FpsCounter {
    frame_count: u32,
    timer: Instant,
    fps: f32,
    update_interval: Duration,
}

impl FpsCounter {
    pub fn new(update_interval_secs: f32) -> Self {
        Self {
            frame_count: 0,
            timer: Instant::now(),
            fps: 0.0,
            update_interval: Duration::from_secs_f32(update_interval_secs),
        }
    }

    pub fn default() -> Self {
        Self::new(1.0)
    }

    /// Call once per frame, returns true if FPS was updated
    pub fn tick(&mut self) -> bool {
        self.frame_count += 1;
        
        if self.timer.elapsed() >= self.update_interval {
            self.fps = self.frame_count as f32 / self.timer.elapsed().as_secs_f32();
            self.frame_count = 0;
            self.timer = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn frame_time_ms(&self) -> f32 {
        if self.fps > 0.0 {
            1000.0 / self.fps
        } else {
            0.0
        }
    }
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_manager_creation() {
        let tm = TimeManager::new(0.016);
        assert!(tm.fps() >= 0.0);
    }

    #[test]
    fn test_time_manager_update() {
        let mut tm = TimeManager::default();
        let delta = tm.update();
        assert!(delta.as_secs_f32() >= 0.0);
    }
}
