// RHI Factory - Backend selection and device creation
// Provides unified interface for creating GPU devices across platforms

use crate::graphics::rhi::device::IDevice;
use crate::graphics::rhi::types::RhiResult;
use std::sync::Arc;
use tracing;

#[cfg(feature = "dx12")]
use crate::graphics::rhi::dx12::device_dx12::Dx12Device;

#[cfg(feature = "dx11")]
use crate::graphics::rhi::dx11::device_dx11::Dx11Device;

#[cfg(feature = "vulkan")]
use crate::graphics::rhi::vulkan::device_vk::VkDevice;

#[cfg(feature = "gl")]
use crate::graphics::rhi::gl::GlDevice;

/// Graphics API backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhiBackend {
    /// Auto-select best available backend
    Auto,
    /// DirectX 11 (Windows only, for older hardware)
    Dx11,
    /// DirectX 12 (Windows only)
    Dx12,
    /// Vulkan (Cross-platform)
    Vulkan,
    /// OpenGL 4.5+ (Cross-platform, default fallback)
    OpenGL,
}

impl RhiBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            RhiBackend::Auto => "Auto",
            RhiBackend::Dx11 => "DirectX 11",
            RhiBackend::Dx12 => "DirectX 12",
            RhiBackend::Vulkan => "Vulkan",
            RhiBackend::OpenGL => "OpenGL",
        }
    }
}

/// RHI configuration
#[derive(Debug, Clone)]
pub struct RhiConfig {
    pub backend: RhiBackend,
    pub debug_enabled: bool,
    pub validation_enabled: bool,
    pub preferred_adapter_index: Option<usize>,
    pub prefer_discrete_gpu: bool,
}

impl Default for RhiConfig {
    fn default() -> Self {
        Self {
            backend: RhiBackend::Auto,
            debug_enabled: cfg!(debug_assertions), // Enable debug layer in debug builds
            validation_enabled: cfg!(debug_assertions),
            preferred_adapter_index: None,
            prefer_discrete_gpu: true,
        }
    }
}

/// RHI Factory - creates and manages GPU devices
pub struct RhiFactory;

impl RhiFactory {
    /// Create a new RHI device with the specified configuration
    pub fn create_device(config: &RhiConfig) -> RhiResult<Arc<dyn IDevice>> {
        let selected_backend = Self::select_backend(config.backend)?;

        tracing::info!(target: "rhi", "=== RHI Factory: Creating device ===");
        tracing::info!(target: "rhi", "Requested backend: {:?}", config.backend);
        tracing::info!(target: "rhi", "Selected backend: {:?}", selected_backend);
        tracing::info!(target: "rhi", "Debug: {}, Validation: {}, PreferDiscreteGPU: {}", 
            config.debug_enabled, config.validation_enabled, config.prefer_discrete_gpu);

        match selected_backend {
            #[cfg(feature = "dx11")]
            RhiBackend::Dx11 => {
                #[cfg(target_os = "windows")]
                {
                    tracing::info!(target: "rhi", ">>> Using DX11 backend via RHI");
                    use crate::graphics::rhi::dx11::device_dx11::Dx11Device;
                    let device = Dx11Device::new(config.debug_enabled, config.validation_enabled)?;
                    tracing::info!(target: "rhi", "<<< DX11 device created: {}", device.get_device_name());
                    Ok(Arc::new(device))
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Err(crate::graphics::rhi::types::RhiError::Unsupported(
                        "DirectX 11 is only available on Windows".to_string(),
                    ))
                }
            }

            #[cfg(feature = "dx12")]
            RhiBackend::Dx12 => {
                #[cfg(target_os = "windows")]
                {
                    tracing::info!(target: "rhi", ">>> Using DX12 backend");
                    use crate::graphics::rhi::dx12::device_dx12::Dx12Device;
                    let device = Dx12Device::new(config.debug_enabled, config.validation_enabled)?;
                    tracing::info!(target: "rhi", "<<< DX12 device created: {}", device.get_device_name());
                    Ok(Arc::new(device))
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Err(crate::graphics::rhi::types::RhiError::Unsupported(
                        "DirectX 12 is only available on Windows".to_string(),
                    ))
                }
            }

            #[cfg(feature = "vulkan")]
            RhiBackend::Vulkan => {
                let device = VkDevice::new(config.validation_enabled)?;
                Ok(Arc::new(device))
            }

            // OpenGL fallback - always available via GlContext
            RhiBackend::OpenGL => {
                // OpenGL device requires an active GL context from glutin/winit
                // The GlContext creates the GlDevice internally when the window is created
                // Return a descriptive error to guide users to the correct API
                Err(crate::graphics::rhi::types::RhiError::InitializationFailed(
                    "OpenGL backend requires window creation via GlContext. Call GlContext::new() to create a window, then use gl_context.rhi_device() to access the device.".to_string(),
                ))
            }

            _ => Err(crate::graphics::rhi::types::RhiError::Unsupported(
                "No suitable RHI backend available".to_string(),
            )),
        }
    }

    /// Select the best available backend based on configuration and platform
    fn select_backend(requested: RhiBackend) -> RhiResult<RhiBackend> {
        match requested {
            RhiBackend::Auto => Self::detect_best_backend(),
            RhiBackend::Dx11 => {
                #[cfg(all(feature = "dx11", target_os = "windows"))]
                {
                    Ok(RhiBackend::Dx11)
                }
                #[cfg(not(all(feature = "dx11", target_os = "windows")))]
                {
                    Err(crate::graphics::rhi::types::RhiError::Unsupported(
                        "DirectX 11 is not available on this platform".to_string(),
                    ))
                }
            }
            RhiBackend::Dx12 => {
                #[cfg(all(feature = "dx12", target_os = "windows"))]
                {
                    Ok(RhiBackend::Dx12)
                }
                #[cfg(not(all(feature = "dx12", target_os = "windows")))]
                {
                    Err(crate::graphics::rhi::types::RhiError::Unsupported(
                        "DirectX 12 is not available on this platform".to_string(),
                    ))
                }
            }
            RhiBackend::Vulkan => {
                #[cfg(feature = "vulkan")]
                {
                    Ok(RhiBackend::Vulkan)
                }
                #[cfg(not(feature = "vulkan"))]
                {
                    Err(crate::graphics::rhi::types::RhiError::Unsupported(
                        "Vulkan support is not compiled in".to_string(),
                    ))
                }
            }
            RhiBackend::OpenGL => {
                // OpenGL is always available as fallback
                Ok(RhiBackend::OpenGL)
            }
        }
    }

    /// Detect the best available backend for the current platform
    fn detect_best_backend() -> RhiResult<RhiBackend> {
        // Priority order: DX12 > DX11 > Vulkan > OpenGL
        // Use best available Windows graphics API

        #[cfg(all(feature = "dx12", target_os = "windows"))]
        {
            if Self::is_dx12_available() {
                tracing::info!("DirectX 12 backend detected as available");
                return Ok(RhiBackend::Dx12);
            }
        }

        #[cfg(all(feature = "dx11", target_os = "windows"))]
        {
            if Self::is_dx11_available() {
                tracing::info!("DirectX 11 backend detected as available");
                return Ok(RhiBackend::Dx11);
            }
        }

        #[cfg(feature = "vulkan")]
        {
            if Self::is_vulkan_available() {
                tracing::info!("Vulkan backend detected as available");
                return Ok(RhiBackend::Vulkan);
            }
        }

        // Fallback to OpenGL (always available)
        tracing::info!("Falling back to OpenGL backend");
        Ok(RhiBackend::OpenGL)
    }

    /// Check if DX11 is available on the system
    #[cfg(all(feature = "dx11", target_os = "windows"))]
    fn is_dx11_available() -> bool {
        true
    }

    /// Check if Vulkan is available on the system
    #[cfg(feature = "vulkan")]
    fn is_vulkan_available() -> bool {
        use ash::vk;
        use ash::Entry;

        let entry = match unsafe { Entry::load() } {
            Ok(e) => e,
            Err(_) => return false,
        };

        match entry.enumerate_instance_extension_properties(None) {
            Ok(extensions) => extensions.iter().any(|ext| {
                let name = unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) };
                name.to_str().unwrap_or("").contains("VK_KHR_surface")
            }),
            Err(_) => false,
        }
    }

    /// Check if DX12 is available on the system
    #[cfg(all(feature = "dx12", target_os = "windows"))]
    fn is_dx12_available() -> bool {
        // DX12 is available on Windows 10+ with compatible hardware
        // For now, assume it's available if we're on Windows
        true
    }

    /// Get list of available backends for this platform
    pub fn get_available_backends() -> Vec<RhiBackend> {
        let mut backends = Vec::new();

        #[cfg(all(feature = "dx12", target_os = "windows"))]
        {
            backends.push(RhiBackend::Dx12);
        }

        #[cfg(all(feature = "dx11", target_os = "windows"))]
        {
            backends.push(RhiBackend::Dx11);
        }

        #[cfg(feature = "vulkan")]
        {
            backends.push(RhiBackend::Vulkan);
        }

        // OpenGL is always available
        backends.push(RhiBackend::OpenGL);

        backends
    }
}

/// Helper function to create a default RHI device
pub fn create_default_device() -> RhiResult<Arc<dyn IDevice>> {
    RhiFactory::create_device(&RhiConfig::default())
}
