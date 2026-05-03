use std::fmt;

#[derive(Debug)]
pub enum RtgcError {
    Io(std::io::Error),
    Graphics(String),
    Audio(String),
    AssetNotFound(String),
    ConfigError(String),
    Physics(String),
    Network(String),
}

impl fmt::Display for RtgcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RtgcError::Io(e) => write!(f, "IO error: {}", e),
            RtgcError::Graphics(msg) => write!(f, "Graphics error: {}", msg),
            RtgcError::Audio(msg) => write!(f, "Audio error: {}", msg),
            RtgcError::AssetNotFound(path) => write!(f, "Asset not found: {}", path),
            RtgcError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            RtgcError::Physics(msg) => write!(f, "Physics error: {}", msg),
            RtgcError::Network(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for RtgcError {}

impl From<std::io::Error> for RtgcError {
    fn from(err: std::io::Error) -> Self {
        RtgcError::Io(err)
    }
}
