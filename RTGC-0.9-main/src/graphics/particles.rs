//! Particles - заглушка системы частиц
//! 
//! Будет переписана позже для использования с новым рендерером на RHI

/// Простая система частиц (заглушка)
#[derive(Clone)]
pub struct ParticleSystem {
    max_particles: usize,
}

impl ParticleSystem {
    pub fn new(max_particles: usize) -> Self {
        Self { max_particles }
    }

    pub fn update(&mut self, _dt: f32) {
        // Заглушка
    }

    pub fn emit(&mut self, _position: [f32; 3], _velocity: [f32; 3], _lifetime: f32) {
        // Заглушка
    }

    pub fn clear(&mut self) {
        // Заглушка
    }
}
