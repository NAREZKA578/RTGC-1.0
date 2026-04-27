//! Error handling module for RTGC engine
//! Provides centralized error types and utilities

use std::error::Error;
use thiserror::Error;
use tracing;

/// Main error type for the RTGC engine
#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Graphics error: {0}")]
    Graphics(#[from] GraphicsError),

    #[error("Physics error: {0}")]
    Physics(#[from] PhysicsError),

    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    #[error("Asset loading error: {0}")]
    AssetLoading(#[from] AssetError),

    #[error("Input error: {0}")]
    Input(#[from] InputError),

    #[error("World error: {0}")]
    World(#[from] WorldError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Thread error: {0}")]
    Thread(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<ConfigError> for EngineError {
    fn from(err: ConfigError) -> Self {
        EngineError::Config(err.to_string())
    }
}

/// Configuration-specific errors for better error handling
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),

    #[error("Invalid FPS value: {0}. Must be between 1 and 1000.")]
    InvalidFps(u32),

    #[error("Memory budget exceeded: {0} MB. Maximum allowed is 4096 MB.")]
    MemoryBudgetExceeded(u32),

    #[error("Path traversal attempt detected")]
    PathTraversal,

    #[error("Forbidden path: {0}")]
    ForbiddenPath(String),

    #[error("Invalid graphics backend: {0}. Valid backends are: vulkan, dx12, opengl")]
    InvalidBackend(String),

    #[error("File read error: {0}")]
    FileReadError(String),

    #[error("File write error: {0}")]
    FileWriteError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Graphics-related errors
#[derive(Error, Debug)]
pub enum GraphicsError {
    #[error("Failed to initialize graphics backend: {0}")]
    InitializationFailed(String),

    #[error("Shader compilation error: {0}")]
    ShaderCompilation(String),

    #[error("Texture loading error: {0}")]
    TextureLoading(String),

    #[error("Mesh loading error: {0}")]
    MeshLoading(String),

    #[error("Render pipeline error: {0}")]
    PipelineError(String),

    #[error("Swapchain error: {0}")]
    SwapchainError(String),

    #[error("Device lost")]
    DeviceLost,

    #[error("Out of memory")]
    OutOfMemory,
}

/// Physics-related errors
#[derive(Error, Debug)]
pub enum PhysicsError {
    #[error("Failed to initialize physics world: {0}")]
    InitializationFailed(String),

    #[error("Invalid collision shape: {0}")]
    InvalidShape(String),

    #[error("Rigid body not found: {0}")]
    BodyNotFound(String),

    #[error("Constraint error: {0}")]
    ConstraintError(String),

    #[error("Simulation step failed: {0}")]
    SimulationFailed(String),
}

/// Audio-related errors
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Failed to initialize audio system: {0}")]
    InitializationFailed(String),

    #[error("Failed to load audio file: {0}")]
    FileLoading(String),

    #[error("Audio device not available")]
    DeviceNotAvailable,

    #[error("Invalid audio format: {0}")]
    InvalidFormat(String),

    #[error("Audio buffer error: {0}")]
    BufferError(String),
}

/// Asset loading errors
#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Asset not found: {0}")]
    NotFound(String),

    #[error("Failed to parse asset file: {0}")]
    ParseError(String),

    #[error("Unsupported asset format: {0}")]
    UnsupportedFormat(String),

    #[error("Asset loading timeout")]
    Timeout,

    #[error("Asset stream error: {0}")]
    StreamError(String),
}

/// Input-related errors
#[derive(Error, Debug)]
pub enum InputError {
    #[error("Failed to initialize input system: {0}")]
    InitializationFailed(String),

    #[error("Gamepad not found: {0}")]
    GamepadNotFound(String),

    #[error("Invalid key mapping: {0}")]
    InvalidKeyMapping(String),
}

/// World-related errors
#[derive(Error, Debug)]
pub enum WorldError {
    #[error("Failed to initialize world: {0}")]
    InitializationFailed(String),

    #[error("Chunk loading error: {0}")]
    ChunkLoading(String),

    #[error("Entity limit reached")]
    EntityLimitReached,

    #[error("Save/Load error: {0}")]
    SaveLoadError(String),
}

/// Result type alias for engine operations
pub type Result<T> = std::result::Result<T, EngineError>;

/// Extension trait for better error handling
pub trait ResultExt<T> {
    fn context(self, msg: impl Into<String>) -> Result<T>;
    fn with_context(self, f: impl FnOnce() -> String) -> Result<T>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for std::result::Result<T, E> {
    fn context(self, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| EngineError::Unknown(format!("{}: {}", msg.into(), e)))
    }

    fn with_context(self, f: impl FnOnce() -> String) -> Result<T> {
        self.map_err(|e| EngineError::Unknown(format!("{}: {}", f(), e)))
    }
}

/// Error reporting utility
pub fn report_error(error: &EngineError) {
    tracing::error!("Engine Error: {}", error);

    // Log error chain
    let mut source = error.source();
    while let Some(err) = source {
        tracing::error!("  Caused by: {}", err);
        source = err.source();
    }
}

/// Panic hook for better error messages
pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = if let Some(location) = panic_info.location() {
            format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        } else {
            "unknown location".to_string()
        };

        tracing::error!("Panic at {}: {}", location, message);
        // eprintln! оставлен для случаев, когда tracing ещё не инициализирован (early panic)
        eprintln!("Panic at {}: {}", location, message);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = EngineError::Config("test config error".to_string());
        assert!(matches!(err, EngineError::Config(_)));
    }

    #[test]
    fn test_graphics_error() {
        let err = GraphicsError::InitializationFailed("GPU not found".to_string());
        let engine_err: EngineError = err.into();
        assert!(matches!(engine_err, EngineError::Graphics(_)));
    }

    #[test]
    fn test_result_ext() {
        let result: std::result::Result<(), std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::Other, "test error"));
        let engine_result = result.context("operation failed");
        assert!(engine_result.is_err());
    }
}
