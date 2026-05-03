use crate::animation::easing;

pub type EasingFn = fn(f32) -> f32;

pub struct Tween {
    pub start: f32,
    pub end: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub easing_fn: EasingFn,
    pub done: bool,
}

impl Tween {
    pub fn new(start: f32, end: f32, duration: f32) -> Self {
        Self {
            start,
            end,
            duration,
            elapsed: 0.0,
            easing_fn: easing::linear,
            done: false,
        }
    }

    pub fn with_easing(mut self, easing_fn: EasingFn) -> Self {
        self.easing_fn = easing_fn;
        self
    }

    pub fn update(&mut self, dt: f32) -> f32 {
        self.elapsed = (self.elapsed + dt).min(self.duration);
        self.done = self.elapsed >= self.duration;
        let t = self.elapsed / self.duration;
        let eased_t = (self.easing_fn)(t);
        self.start + (self.end - self.start) * eased_t
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.done = false;
    }
}
