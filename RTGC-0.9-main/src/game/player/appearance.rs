//! Player appearance customization

/// Player appearance data
#[derive(Debug, Clone)]
pub struct Appearance {
    pub is_male: bool,
    pub height: f32, // 1.6..2.0
    pub skin_color: [f32; 3],
    pub face_variant: u8, // 0..7
    pub hair_style: u8,
    pub hair_color: [f32; 3],
    pub body_build: f32, // 0.0 (thin) .. 1.0 (muscular)
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            is_male: true,
            height: 1.75,
            skin_color: [0.8, 0.65, 0.5],
            face_variant: 0,
            hair_style: 0,
            hair_color: [0.3, 0.2, 0.1],
            body_build: 0.5,
        }
    }
}

impl Appearance {
    /// Create a random appearance
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Self {
            is_male: rng.gen_bool(0.5),
            height: rng.gen_range(1.6..2.0),
            skin_color: [
                rng.gen_range(0.4..0.9),
                rng.gen_range(0.3..0.7),
                rng.gen_range(0.2..0.5),
            ],
            face_variant: rng.gen_range(0..8),
            hair_style: rng.gen_range(0..10),
            hair_color: [
                rng.gen_range(0.0..0.5),
                rng.gen_range(0.0..0.4),
                rng.gen_range(0.0..0.3),
            ],
            body_build: rng.r#gen(),
        }
    }

    /// Get preset skin colors
    pub fn preset_skin_colors() -> Vec<[f32; 3]> {
        vec![
            [0.95, 0.8, 0.7],  // Very light
            [0.8, 0.65, 0.5],  // Light
            [0.7, 0.55, 0.4],  // Medium
            [0.55, 0.4, 0.3],  // Tan
            [0.4, 0.28, 0.2],  // Dark
            [0.25, 0.15, 0.1], // Very dark
        ]
    }

    /// Get preset hair colors
    pub fn preset_hair_colors() -> Vec<[f32; 3]> {
        vec![
            [0.95, 0.9, 0.85],  // Blonde
            [0.6, 0.4, 0.2],    // Brown
            [0.2, 0.15, 0.1],   // Dark brown
            [0.05, 0.03, 0.02], // Black
            [0.8, 0.3, 0.1],    // Red
            [0.9, 0.9, 0.9],    // Gray
            [0.1, 0.8, 0.3],    // Green (fun)
            [0.3, 0.2, 0.8],    // Blue (fun)
        ]
    }
}
