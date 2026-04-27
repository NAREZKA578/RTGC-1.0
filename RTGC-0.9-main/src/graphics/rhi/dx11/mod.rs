//! DirectX 11 Backend - Stubs (DX11 реально не работает без переписывания RHI)

pub mod buffer_dx11;
pub mod context_dx11;
pub mod device_dx11;
pub mod pipeline_dx11;
pub mod shader_dx11;
pub mod swapchain_dx11;
pub mod texture_dx11;

pub use device_dx11::Dx11Device;

// Stub constants for missing Windows API
#[allow(dead_code)]
pub const D3D11_RESOURCE_MISC_NONE: u32 = 0;
#[allow(dead_code)]
pub const D3D11_PRIMITIVE_TOPOLOGY: u32 = 0;
#[allow(dead_code)]
pub const D3D11_SRV_DIMENSION_TEXTURE2D: u32 = 2;
