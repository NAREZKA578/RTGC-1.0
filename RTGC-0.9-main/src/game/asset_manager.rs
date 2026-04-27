pub struct AssetManager;

impl AssetManager {
    pub fn new() -> Self {
        Self
    }
    
    pub fn get_texture(&self, name: &str) -> Option<crate::graphics::rhi::types::ResourceHandle> {
        None
    }
    
    pub fn get_model(&self, name: &str) -> Option<std::sync::Arc<dyn std::any::Any>> {
        None
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}