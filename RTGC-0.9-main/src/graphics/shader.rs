use crate::graphics::rhi::{IDevice, ShaderDescription, ShaderStage, RhiError, ResourceHandle};
use std::fs;

/// Загружает шейдер из файла по указанному пути
pub fn load_shader_from_file(
    device: &dyn IDevice,
    path: &str,
    stage: ShaderStage,
    entry_point: &str,
) -> Result<ResourceHandle, RhiError> {
    let source = fs::read_to_string(path)
        .map_err(|e| RhiError::InitializationFailed(format!("Failed to read shader file {}: {}", path, e)))?;
    
    let desc = ShaderDescription {
        stage,
        source: source.into_bytes(),
        entry_point: entry_point.to_string(),
    };
    
    device.create_shader(&desc)
}

/// Удобная функция для загрузки вершинного шейдера
pub fn load_vertex_shader(device: &dyn IDevice, path: &str) -> Result<ResourceHandle, RhiError> {
    load_shader_from_file(device, path, ShaderStage::Vertex, "main")
}

/// Удобная функция для загрузки фрагментного шейдера
pub fn load_fragment_shader(device: &dyn IDevice, path: &str) -> Result<ResourceHandle, RhiError> {
    load_shader_from_file(device, path, ShaderStage::Fragment, "main")
}
