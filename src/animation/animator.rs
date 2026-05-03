use crate::animation::tween::Tween;

pub struct Animator {
    pub tweens: Vec<(String, Tween)>,
}

impl Animator {
    pub fn new() -> Self {
        Self {
            tweens: Vec::new(),
        }
    }

    pub fn add(&mut self, name: String, tween: Tween) {
        self.tweens.push((name, tween));
    }

    pub fn update(&mut self, dt: f32) {
        for (_, tween) in &mut self.tweens {
            tween.update(dt);
        }
    }

    pub fn get(&self, name: &str) -> Option<f32> {
        self.tweens
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, t)| {
                if t.done {
                    t.end
                } else {
                    t.start + (t.end - t.start) * (t.elapsed / t.duration)
                }
            })
    }

    pub fn is_done(&self, name: &str) -> bool {
        self.tweens
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, t)| t.done)
            .unwrap_or(true)
    }
}
