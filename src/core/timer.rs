use std::time::Instant;

pub struct FrameTimer {
    last_frame: Instant,
    pub dt: f64,
    pub fps: f64,
    frame_count: u32,
    fps_timer: Instant,
}

impl FrameTimer {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            last_frame: now,
            dt: 0.0,
            fps: 0.0,
            frame_count: 0,
            fps_timer: now,
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.dt = now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;
        self.frame_count += 1;

        if now.duration_since(self.fps_timer).as_secs_f64() >= 1.0 {
            self.fps = self.frame_count as f64;
            self.frame_count = 0;
            self.fps_timer = now;
        }
    }
}
