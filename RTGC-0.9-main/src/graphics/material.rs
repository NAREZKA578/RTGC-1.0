//! Materials - заглушка менеджера материалов
//! 
//! Будет переписан позже для использования с новым рендерером на RHI

/// Качество текстур
#[derive(Debug, Clone, Copy)]
pub enum TextureQuality {
    Low,
    Medium,
    High,
}

/// Менеджер материалов (заглушка)
#[derive(Clone)]
pub struct MaterialManager {
    texture_quality: TextureQuality,
}

impl MaterialManager {
    pub fn new(texture_quality: TextureQuality) -> Self {
        Self { texture_quality }
    }

    pub fn texture_quality(&self) -> TextureQuality {
        self.texture_quality
    }

    pub fn set_texture_quality(&mut self, quality: TextureQuality) {
        self.texture_quality = quality;
    }
}
