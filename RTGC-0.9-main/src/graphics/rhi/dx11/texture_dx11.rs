//! DirectX 11 Texture - Full Implementation
//! Supports Texture2D, SRV, RTV, DSV, Sampler with all common formats

use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{error, info, warn};

use crate::graphics::rhi::device::*;
use crate::graphics::rhi::types::*;
use crate::graphics::rhi::RhiResult;

// Stub constant for missing Windows API
#[allow(dead_code)]
const D3D11_SRV_DIMENSION_TEXTURE2D: u32 = 2;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Graphics::Direct3D11::{
        D3D11_TEXTURE2D_DESC, D3D11_SHADER_RESOURCE_VIEW_DESC,
        D3D11_RENDER_TARGET_VIEW_DESC, D3D11_DEPTH_STENCIL_VIEW_DESC,
        D3D11_SAMPLER_DESC,
        D3D11_USAGE_DEFAULT,
        D3D11_BIND_SHADER_RESOURCE, D3D11_BIND_RENDER_TARGET, D3D11_BIND_DEPTH_STENCIL,
        D3D11_RESOURCE_MISC_GENERATE_MIPS,
        ID3D11Texture2D, ID3D11ShaderResourceView, ID3D11RenderTargetView,
        ID3D11DepthStencilView, ID3D11SamplerState,
        D3D11_RTV_DIMENSION_TEXTURE2D,
        D3D11_DSV_DIMENSION_TEXTURE2D,
        D3D11_FILTER_MIN_MAG_MIP_LINEAR, D3D11_TEXTURE_ADDRESS_WRAP,
        D3D11_COMPARISON_NEVER, D3D11_FILTER_MIN_MAG_MIP_POINT,
        D3D11_FILTER_MIN_MAG_LINEAR_MIP_POINT, D3D11_FILTER_MIN_LINEAR_MAG_MIP_POINT,
        D3D11_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT, D3D11_FILTER_MIN_POINT_MAG_MIP_LINEAR,
        D3D11_FILTER_MIN_LINEAR_MAG_POINT_MIP_LINEAR, D3D11_FILTER_ANISOTROPIC,
        D3D11_TEXTURE_ADDRESS_CLAMP, D3D11_TEXTURE_ADDRESS_MIRROR,
        D3D11_TEXTURE_ADDRESS_BORDER, D3D11_TEXTURE_ADDRESS_MIRROR_ONCE,
        D3D11_CPU_ACCESS_READ, D3D11_CPU_ACCESS_WRITE,
    },
    Win32::Graphics::Dxgi::Common::{
        DXGI_FORMAT_UNKNOWN, DXGI_FORMAT_R8G8B8A8_UNORM,
        DXGI_FORMAT_R8G8B8A8_UNORM_SRGB, DXGI_FORMAT_R32_FLOAT,
        DXGI_FORMAT_R32G32_FLOAT, DXGI_FORMAT_R32G32B32A32_FLOAT,
        DXGI_FORMAT_D32_FLOAT, DXGI_FORMAT_D24_UNORM_S8_UINT,
        DXGI_FORMAT_D16_UNORM, DXGI_FORMAT_BC1_UNORM, DXGI_FORMAT_BC3_UNORM,
    },
};

/// DX11 Texture resource
pub struct Dx11Texture {
    #[cfg(target_os = "windows")]
    texture: Option<ID3D11Texture2D>,
    #[cfg(target_os = "windows")]
    srv: Option<ID3D11ShaderResourceView>,
    #[cfg(target_os = "windows")]
    rtv: Option<ID3D11RenderTargetView>,
    #[cfg(target_os = "windows")]
    dsv: Option<ID3D11DepthStencilView>,
    
    handle: ResourceHandle,
    desc: TextureDescription,
    width: u32,
    height: u32,
    mip_levels: u32,
    name: String,
}

#[cfg(target_os = "windows")]
unsafe impl Send for Dx11Texture {}

#[cfg(target_os = "windows")]
unsafe impl Sync for Dx11Texture {}

/// DX11 Sampler resource
pub struct Dx11Sampler {
    #[cfg(target_os = "windows")]
    sampler: Option<ID3D11SamplerState>,
    
    handle: ResourceHandle,
    desc: SamplerDescription,
    name: String,
}

#[cfg(target_os = "windows")]
unsafe impl Send for Dx11Sampler {}

#[cfg(target_os = "windows")]
unsafe impl Sync for Dx11Sampler {}

#[cfg(target_os = "windows")]
fn dxgi_format_from_rhi(format: TextureFormat) -> windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT {
    use windows::Win32::Graphics::Dxgi::Common::*;
    
    match format {
        TextureFormat::R8Unorm => DXGI_FORMAT_R8_UNORM,
        TextureFormat::R8G8Unorm | TextureFormat::Rg8Unorm => DXGI_FORMAT_R8G8_UNORM,
        TextureFormat::R8G8B8A8Unorm | TextureFormat::Rgba8Unorm => DXGI_FORMAT_R8G8B8A8_UNORM,
        TextureFormat::R8G8B8A8Srgb | TextureFormat::Rgba8UnormSrgb => DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
        TextureFormat::R32Float => DXGI_FORMAT_R32_FLOAT,
        TextureFormat::R32G32Float | TextureFormat::Rg32Float => DXGI_FORMAT_R32G32_FLOAT,
        TextureFormat::R32G32B32A32Float | TextureFormat::Rgba32Float => DXGI_FORMAT_R32G32B32A32_FLOAT,
        TextureFormat::R16Float => DXGI_FORMAT_R16_FLOAT,
        TextureFormat::R16G16Float | TextureFormat::Rg16Float => DXGI_FORMAT_R16G16_FLOAT,
        TextureFormat::R16G16B16A16Float | TextureFormat::Rgba16Float => DXGI_FORMAT_R16G16B16A16_FLOAT,
        
        // Depth formats
        TextureFormat::D32Float | TextureFormat::Depth32Float => DXGI_FORMAT_D32_FLOAT,
        TextureFormat::D24UnormS8Uint | TextureFormat::Depth24PlusStencil8 => DXGI_FORMAT_D24_UNORM_S8_UINT,
        TextureFormat::D16Unorm | TextureFormat::Depth16Unorm => DXGI_FORMAT_D16_UNORM,
        
        // BC compression formats
        TextureFormat::BC1RgbUnorm | TextureFormat::BC1RgbaUnorm => DXGI_FORMAT_BC1_UNORM,
        TextureFormat::BC3Unorm | TextureFormat::BC3RgbaUnorm => DXGI_FORMAT_BC3_UNORM,
        
        // BGR formats
        TextureFormat::Bgra8Unorm => DXGI_FORMAT_B8G8R8A8_UNORM,
        TextureFormat::Bgra8UnormSrgb => DXGI_FORMAT_B8G8R8A8_UNORM_SRGB,
        
        _ => {
            warn!(target: "dx11.texture", "Unknown texture format {:?}, defaulting to R8G8B8A8_UNORM", format);
            DXGI_FORMAT_R8G8B8A8_UNORM
        }
    }
}

#[cfg(target_os = "windows")]
fn dxgi_depth_format_from_rhi(format: TextureFormat) -> windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT {
    use windows::Win32::Graphics::Dxgi::Common::*;
    
    match format {
        TextureFormat::D32Float | TextureFormat::Depth32Float => DXGI_FORMAT_D32_FLOAT,
        TextureFormat::D24UnormS8Uint | TextureFormat::Depth24PlusStencil8 => DXGI_FORMAT_D24_UNORM_S8_UINT,
        TextureFormat::D16Unorm | TextureFormat::Depth16Unorm => DXGI_FORMAT_D16_UNORM,
        _ => DXGI_FORMAT_D24_UNORM_S8_UINT,
    }
}

impl Dx11Texture {
    /// Create a new texture from description
    #[cfg(target_os = "windows")]
    pub fn new(
        device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
        desc: &TextureDescription,
        initial_data: Option<&[u8]>,
    ) -> RhiResult<Self> {
        info!(target: "dx11.texture", "Creating texture: {}x{}x{}, format={:?}, usage={:?}", 
              desc.width, desc.height, desc.depth, desc.format, desc.usage);
        
        // Determine bind flags
        let mut bind_flags = 0u32;
        if desc.usage.contains(TextureUsage::SHADER_READ) {
            bind_flags |= D3D11_BIND_SHADER_RESOURCE.0 as u32;
        }
        if desc.usage.contains(TextureUsage::RENDER_TARGET) {
            bind_flags |= D3D11_BIND_RENDER_TARGET.0 as u32;
        }
        if desc.usage.contains(TextureUsage::DEPTH_STENCIL) {
            bind_flags |= D3D11_BIND_DEPTH_STENCIL.0 as u32;
        }
        
        // Determine misc flags
        let mut misc_flags = 0u32;
        if desc.mip_levels > 1 && desc.usage.contains(TextureUsage::SHADER_READ) {
            misc_flags |= D3D11_RESOURCE_MISC_GENERATE_MIPS.0 as u32;
        }
        
        let dxgi_format = if desc.format.is_depth_format() {
            dxgi_depth_format_from_rhi(desc.format)
        } else {
            dxgi_format_from_rhi(desc.format)
        };
        
        let texture_desc = D3D11_TEXTURE2D_DESC {
            Width: desc.width,
            Height: desc.height,
            MipLevels: desc.mip_levels,
            ArraySize: desc.depth_or_array_layers,
            Format: dxgi_format,
            SampleDesc: windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: bind_flags,
            CPUAccessFlags: 0,
            MiscFlags: misc_flags,
        };
        
        info!(target: "dx11.texture", "Texture desc: {}x{}, MipLevels={}, Format={:?}, BindFlags=0x{:x}",
              texture_desc.Width, texture_desc.Height, texture_desc.MipLevels, texture_desc.Format, texture_desc.BindFlags.0);
        
        // Create texture
        let texture = unsafe {
            if let Some(data) = initial_data {
                use windows::Win32::Graphics::Direct3D11::D3D11_SUBRESOURCE_DATA;
                
                // Calculate row pitch and initial data
                let row_pitch = desc.width as usize * get_format_bytes_per_pixel(desc.format) as usize;
                let subresource_data = D3D11_SUBRESOURCE_DATA {
                    pSysMem: data.as_ptr() as *const _,
                    SysMemPitch: row_pitch as u32,
                    SysMemSlicePitch: 0,
                };
                
                device.CreateTexture2D(&texture_desc, Some(&subresource_data))
            } else {
                device.CreateTexture2D(&texture_desc, None)
            }
        };
        
        let texture = match texture {
            Ok(t) => {
                info!(target: "dx11.texture", "ID3D11Texture2D created successfully");
                Some(t)
            }
            Err(e) => {
                error!(target: "dx11.texture", "Failed to create texture: {:?}", e);
                return Err(RhiError::InitializationFailed(format!(
                    "CreateTexture2D: {:?}",
                    e
                )));
            }
        };
        
        // Create views based on usage
        let mut srv = None;
        let mut rtv = None;
        let mut dsv = None;
        
        if desc.usage.contains(TextureUsage::SHADER_READ) {
            srv = Self::create_srv(device, texture.as_ref().ok_or("Texture access failed")?, desc.format)?;
        }
        if desc.usage.contains(TextureUsage::RENDER_TARGET) {
            rtv = Self::create_rtv(device, texture.as_ref().ok_or("Texture access failed")?, desc.format)?;
        }
        if desc.usage.contains(TextureUsage::DEPTH_STENCIL) {
            dsv = Self::create_dsv(device, texture.as_ref().ok_or("Texture access failed")?, desc.format)?;
        }
        
        let name = format!("DX11Texture({}x{}, {:?})", desc.width, desc.height, desc.format);
        
        Ok(Self {
            texture,
            srv,
            rtv,
            dsv,
            handle: ResourceHandle(0),
            desc: desc.clone(),
            width: desc.width,
            height: desc.height,
            mip_levels: desc.mip_levels,
            name,
        })
    }
    
    #[cfg(target_os = "windows")]
    fn create_srv(
        device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
        texture: &ID3D11Texture2D,
        format: TextureFormat,
    ) -> RhiResult<Option<ID3D11ShaderResourceView>> {
        info!(target: "dx11.texture", "Creating ShaderResourceView");
        
        let srv_desc = D3D11_SHADER_RESOURCE_VIEW_DESC {
            Format: dxgi_format_from_rhi(format),
            ViewDimension: D3D11_SRV_DIMENSION_TEXTURE2D,
            Anonymous: windows::Win32::Graphics::Direct3D11::D3D11_SHADER_RESOURCE_VIEW_DESC_0 {
                Texture2D: windows::Win32::Graphics::Direct3D11::D3D11_TEX2D_SRV {
                    MostDetailedMip: 0,
                    MipLevels: 0, // Use all levels
                },
            },
        };
        
        match unsafe { device.CreateShaderResourceView(Some(texture), Some(&srv_desc)) } {
            Ok(srv) => {
                info!(target: "dx11.texture", "ID3D11ShaderResourceView created");
                Ok(Some(srv))
            }
            Err(e) => {
                error!(target: "dx11.texture", "Failed to create SRV: {:?}", e);
                Ok(None)
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    fn create_rtv(
        device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
        texture: &ID3D11Texture2D,
        format: TextureFormat,
    ) -> RhiResult<Option<ID3D11RenderTargetView>> {
        info!(target: "dx11.texture", "Creating RenderTargetView");
        
        let rtv_desc = D3D11_RENDER_TARGET_VIEW_DESC {
            Format: dxgi_format_from_rhi(format),
            ViewDimension: D3D11_RTV_DIMENSION_TEXTURE2D,
            Anonymous: windows::Win32::Graphics::Direct3D11::D3D11_RENDER_TARGET_VIEW_DESC_0 {
                Texture2D: windows::Win32::Graphics::Direct3D11::D3D11_TEX2D_RTV {
                    MipSlice: 0,
                },
            },
        };
        
        match unsafe { device.CreateRenderTargetView(Some(texture), Some(&rtv_desc)) } {
            Ok(rtv) => {
                info!(target: "dx11.texture", "ID3D11RenderTargetView created");
                Ok(Some(rtv))
            }
            Err(e) => {
                error!(target: "dx11.texture", "Failed to create RTV: {:?}", e);
                Ok(None)
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    fn create_dsv(
        device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
        texture: &ID3D11Texture2D,
        format: TextureFormat,
    ) -> RhiResult<Option<ID3D11DepthStencilView>> {
        info!(target: "dx11.texture", "Creating DepthStencilView");
        
        let dsv_desc = D3D11_DEPTH_STENCIL_VIEW_DESC {
            Format: dxgi_depth_format_from_rhi(format),
            ViewDimension: D3D11_DSV_DIMENSION_TEXTURE2D,
            Flags: windows::Win32::Graphics::Direct3D11::D3D11_DSV_FLAG(0),
            Anonymous: windows::Win32::Graphics::Direct3D11::D3D11_DEPTH_STENCIL_VIEW_DESC_0 {
                Texture2D: windows::Win32::Graphics::Direct3D11::D3D11_TEX2D_DSV {
                    MipSlice: 0,
                },
            },
        };
        
        match unsafe { device.CreateDepthStencilView(Some(texture), Some(&dsv_desc)) } {
            Ok(dsv) => {
                info!(target: "dx11.texture", "ID3D11DepthStencilView created");
                Ok(Some(dsv))
            }
            Err(e) => {
                error!(target: "dx11.texture", "Failed to create DSV: {:?}", e);
                Ok(None)
            }
        }
    }
    
    /// Get the underlying ID3D11Texture2D
    #[cfg(target_os = "windows")]
    pub fn get_texture(&self) -> &Option<ID3D11Texture2D> {
        &self.texture
    }
    
    /// Get the shader resource view
    #[cfg(target_os = "windows")]
    pub fn get_srv(&self) -> &Option<ID3D11ShaderResourceView> {
        &self.srv
    }
    
    /// Get the render target view
    #[cfg(target_os = "windows")]
    pub fn get_rtv(&self) -> &Option<ID3D11RenderTargetView> {
        &self.rtv
    }
    
    /// Get the depth stencil view
    #[cfg(target_os = "windows")]
    pub fn get_dsv(&self) -> &Option<ID3D11DepthStencilView> {
        &self.dsv
    }
    
    /// Get texture width
    pub fn get_width(&self) -> u32 {
        self.width
    }
    
    /// Get texture height
    pub fn get_height(&self) -> u32 {
        self.height
    }
    
    /// Get mip levels
    pub fn get_mip_levels(&self) -> u32 {
        self.mip_levels
    }
}

#[cfg(target_os = "windows")]
fn get_format_bytes_per_pixel(format: TextureFormat) -> u32 {
    match format {
        TextureFormat::R8Unorm => 1,
        TextureFormat::R8G8Unorm | TextureFormat::Rg8Unorm => 2,
        TextureFormat::R8G8B8A8Unorm | TextureFormat::Rgba8Unorm | TextureFormat::Bgra8Unorm => 4,
        TextureFormat::R32Float => 4,
        TextureFormat::R32G32Float | TextureFormat::Rg32Float => 8,
        TextureFormat::R32G32B32A32Float | TextureFormat::Rgba32Float => 16,
        TextureFormat::R16Float => 2,
        TextureFormat::R16G16Float | TextureFormat::Rg16Float => 4,
        TextureFormat::R16G16B16A16Float | TextureFormat::Rgba16Float => 8,
        TextureFormat::D32Float => 4,
        TextureFormat::D24UnormS8Uint => 4,
        TextureFormat::D16Unorm => 2,
        TextureFormat::BC1RgbUnorm | TextureFormat::BC1RgbaUnorm => 8, // Block size (4x4 pixels)
        TextureFormat::BC3Unorm | TextureFormat::BC3RgbaUnorm => 16,
        _ => 4,
    }
}

impl Dx11Sampler {
    /// Create a new sampler from description
    #[cfg(target_os = "windows")]
    pub fn new(
        device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
        desc: &SamplerDescription,
    ) -> RhiResult<Self> {
        info!(target: "dx11.sampler", "Creating sampler: min={:?}, mag={:?}, mip={:?}", 
              desc.min_filter, desc.mag_filter, desc.mip_filter);
        
        let filter = match (desc.min_filter, desc.mag_filter, desc.mip_filter, desc.max_anisotropy) {
            (FilterMode::Point, FilterMode::Point, FilterMode::Point, _) => D3D11_FILTER_MIN_MAG_MIP_POINT,
            (FilterMode::Bilinear, FilterMode::Bilinear, FilterMode::Point, _) => D3D11_FILTER_MIN_MAG_LINEAR_MIP_POINT,
            (FilterMode::Bilinear, FilterMode::Bilinear, FilterMode::Bilinear, _) => D3D11_FILTER_MIN_MAG_MIP_LINEAR,
            _ if desc.max_anisotropy > 1 => D3D11_FILTER_ANISOTROPIC,
            _ => D3D11_FILTER_MIN_MAG_MIP_LINEAR,
        };
        
        let address_mode = |mode: AddressMode| -> windows::Win32::Graphics::Direct3D11::D3D11_TEXTURE_ADDRESS_MODE {
            use windows::Win32::Graphics::Direct3D11::*;
            match mode {
                AddressMode::ClampToEdge => D3D11_TEXTURE_ADDRESS_CLAMP,
                AddressMode::Wrap => D3D11_TEXTURE_ADDRESS_WRAP,
                AddressMode::Mirror => D3D11_TEXTURE_ADDRESS_MIRROR,
                AddressMode::Border => D3D11_TEXTURE_ADDRESS_BORDER,
                AddressMode::MirrorOnce => D3D11_TEXTURE_ADDRESS_MIRROR_ONCE,
            }
        };
        
        let sampler_desc = D3D11_SAMPLER_DESC {
            Filter: filter,
            AddressU: address_mode(desc.address_u),
            AddressV: address_mode(desc.address_v),
            AddressW: address_mode(desc.address_w),
            MipLODBias: desc.mip_lod_bias,
            MaxAnisotropy: desc.max_anisotropy,
            ComparisonFunc: D3D11_COMPARISON_NEVER,
            BorderColor: desc.border_color,
            MinLOD: desc.min_lod,
            MaxLOD: desc.max_lod,
        };
        
        let sampler = unsafe { device.CreateSamplerState(&sampler_desc) };
        
        let sampler = match sampler {
            Ok(s) => {
                info!(target: "dx11.sampler", "ID3D11SamplerState created");
                Some(s)
            }
            Err(e) => {
                error!(target: "dx11.sampler", "Failed to create sampler: {:?}", e);
                return Err(RhiError::InitializationFailed(format!(
                    "CreateSamplerState: {:?}",
                    e
                )));
            }
        };
        
        let name = format!("DX11Sampler({:?})", desc.min_filter);
        
        Ok(Self {
            sampler,
            handle: ResourceHandle(0),
            desc: desc.clone(),
            name,
        })
    }
    
    /// Get the underlying ID3D11SamplerState
    #[cfg(target_os = "windows")]
    pub fn get_sampler(&self) -> &Option<ID3D11SamplerState> {
        &self.sampler
    }
    
    /// Bind sampler to pixel shader stage
    #[cfg(target_os = "windows")]
    pub fn bind_ps(
        &self,
        context: &windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext,
        slot: u32,
    ) {
        info!(target: "dx11.sampler", "Binding sampler to PS slot {}", slot);
        
        if let Some(ref sampler) = self.sampler {
            unsafe {
                context.PSSetSamplers(slot, &[Some(sampler)]);
            }
        }
    }
}

// Stub implementations for non-Windows platforms
#[cfg(not(target_os = "windows"))]
impl Dx11Texture {
    pub fn new(_device: &(), _desc: &TextureDescription, _initial_data: Option<&[u8]>) -> RhiResult<Self> {
        warn!(target: "dx11.texture", "DX11 texture creation requires Windows");
        Ok(Self {
            handle: ResourceHandle(0),
            desc: _desc.clone(),
            width: _desc.width,
            height: _desc.height,
            mip_levels: _desc.mip_levels,
            name: "DX11Texture(stub)".to_string(),
        })
    }
    
    pub fn get_width(&self) -> u32 {
        self.width
    }
    
    pub fn get_height(&self) -> u32 {
        self.height
    }
}

#[cfg(not(target_os = "windows"))]
impl Dx11Sampler {
    pub fn new(_device: &(), _desc: &SamplerDescription) -> RhiResult<Self> {
        warn!(target: "dx11.sampler", "DX11 sampler creation requires Windows");
        Ok(Self {
            handle: ResourceHandle(0),
            desc: _desc.clone(),
            name: "DX11Sampler(stub)".to_string(),
        })
    }
}
