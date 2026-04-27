//! Graphics API configuration

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsApi {
    /// OpenGL (default, most compatible)
    OpenGL,
    /// DirectX 11 (Windows only, better performance)
    DirectX11,
}

impl Default for GraphicsApi {
    fn default() -> Self {
        // Default to DX11 on Windows, OpenGL elsewhere
        #[cfg(target_os = "windows")]
        {
            Self::DirectX11
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self::OpenGL
        }
    }
}

impl std::fmt::Display for GraphicsApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphicsApi::OpenGL => write!(f, "OpenGL"),
            GraphicsApi::DirectX11 => write!(f, "DirectX 11"),
        }
    }
}
