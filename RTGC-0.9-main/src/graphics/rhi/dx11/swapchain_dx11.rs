//! DirectX 11 SwapChain - Full Implementation
//! Implements ISwapChain trait with IDXGISwapChain1

use std::sync::Arc;
use tracing::{error, info, warn};

use crate::graphics::rhi::device::ISwapChain;
use crate::graphics::rhi::types::{RhiError, RhiResult, ResourceHandle, TextureFormat};

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::{HWND, RECT},
    Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11RenderTargetView, ID3D11Texture2D},
    Win32::Graphics::Dxgi::{
        IDXGIFactory1, IDXGISwapChain1, DXGI_PRESENT_ALLOW_TEARING, DXGI_SCALING_STRETCH,
        DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING, DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT,
        DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_SWAP_CHAIN_DESC1, DXGI_USAGE,
    },
    Win32::Graphics::Dxgi::Common::{
        DXGI_FORMAT, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM,
        DXGI_SAMPLE_DESC,
    },
};

pub struct Dx11SwapChain {
    #[cfg(target_os = "windows")]
    swap_chain: IDXGISwapChain1,
    #[cfg(target_os = "windows")]
    render_target_view: Option<ID3D11RenderTargetView>,
    #[cfg(target_os = "windows")]
    back_buffer: Option<ID3D11Texture2D>,
    width: u32,
    height: u32,
    vsync: bool,
    format: TextureFormat,
    handle: ResourceHandle,
}

#[cfg(target_os = "windows")]
unsafe impl Send for Dx11SwapChain {}

#[cfg(target_os = "windows")]
unsafe impl Sync for Dx11SwapChain {}

impl Dx11SwapChain {
    pub fn new(
        factory: &IDXGIFactory1,
        device: &ID3D11Device,
        hwnd: HWND,
        width: u32,
        height: u32,
        format: TextureFormat,
        vsync: bool,
    ) -> RhiResult<Self> {
        info!(target: "dx11", "=== Dx11SwapChain::new START ===");
        info!(target: "dx11", "Creating swapchain: {}x{}, format={:?}, vsync={}", 
              width, height, format, vsync);

        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Dxgi::Common::{DXGI_MODE_DESC, DXGI_RATIONAL};
            use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory2};

            // Try to get IDXGIFactory2 for Flip model swap chain
            let factory2: IDXGIFactory2 = unsafe {
                CreateDXGIFactory1().map_err(|e| {
                    error!(target: "dx11", "Failed to create DXGI factory: {:?}", e);
                    RhiError::InitializationFailed(format!("DXGI factory: {:?}", e))
                })?
            };

            // Determine swap chain flags
            let mut swap_flags = DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT;
            
            // Check if tearing is allowed (for variable refresh rate)
            let allow_tearing = !vsync;
            if allow_tearing {
                swap_flags |= DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING;
            }

            // Convert texture format to DXGI format
            let dxgi_format = Self::texture_format_to_dxgi(format);
            info!(target: "dx11", "Using DXGI format: {:?}", dxgi_format);

            // Buffer description
            let buffer_desc = DXGI_MODE_DESC {
                Width: width,
                Height: height,
                RefreshRate: DXGI_RATIONAL { Numerator: 60, Denominator: 1 },
                Format: dxgi_format,
                ScanlineOrdering: 0, // DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED
                Scaling: 0,          // DXGI_MODE_SCALING_UNSPECIFIED
            };

            // Sample description (no MSAA for now)
            let sample_desc = DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            };

            // Create swap chain description
            let swap_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: width,
                Height: height,
                Format: dxgi_format,
                Stereo: false.into(),
                SampleDesc: sample_desc,
                BufferUsage: DXGI_USAGE(0x00000001u32), // DXGI_USAGE_RENDER_TARGET_OUTPUT
                BufferCount: 2, // Double buffering
                Scaling: DXGI_SCALING_STRETCH,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                AlphaMode: 0, // DXGI_ALPHA_MODE_UNSPECIFIED
                Flags: swap_flags.0,
            };

            // Create swap chain
            let swap_chain: IDXGISwapChain1 = unsafe {
                factory2.CreateSwapChainForHwnd(
                    device,
                    hwnd,
                    &swap_desc,
                    None, // fullscreen desc
                    None, // restrict to output
                ).map_err(|e| {
                    error!(target: "dx11", "Failed to create swap chain: {:?}", e);
                    RhiError::ResourceCreationFailed(format!("SwapChain: {:?}", e))
                })?
            };

            info!(target: "dx11", "SwapChain created successfully");

            // Disable Alt+Enter fullscreen handling by the swap chain
            unsafe {
                factory2.MakeWindowAssociation(hwnd, windows::Win32::Graphics::Dxgi::DXGI_MWA_NO_ALT_ENTER);
            }

            // Create render target view from back buffer
            let back_buffer: ID3D11Texture2D = unsafe {
                swap_chain.GetBuffer(0).map_err(|e| {
                    error!(target: "dx11", "Failed to get back buffer: {:?}", e);
                    RhiError::ResourceCreationFailed(format!("Back buffer: {:?}", e))
                })?
            };

            let render_target_view: ID3D11RenderTargetView = unsafe {
                device.CreateRenderTargetView(&back_buffer, None).map_err(|e| {
                    error!(target: "dx11", "Failed to create RTV: {:?}", e);
                    RhiError::ResourceCreationFailed(format!("RTV: {:?}", e))
                })?
            };

            info!(target: "dx11", "Render target view created");

            Ok(Self {
                swap_chain,
                render_target_view: Some(render_target_view),
                back_buffer: Some(back_buffer),
                width,
                height,
                vsync,
                format,
                handle: ResourceHandle(1),
            })
        }

        #[cfg(not(target_os = "windows"))]
        {
            warn!(target: "dx11", "SwapChain created as stub (not on Windows)");
            Ok(Self {
                width,
                height,
                vsync,
                format,
                handle: ResourceHandle(1),
            })
        }
    }

    #[cfg(target_os = "windows")]
    fn texture_format_to_dxgi(format: TextureFormat) -> DXGI_FORMAT {
        match format {
            TextureFormat::R8G8B8A8Unorm => DXGI_FORMAT_R8G8B8A8_UNORM,
            TextureFormat::Bgra8Unorm => DXGI_FORMAT_B8G8R8A8_UNORM,
            TextureFormat::Bgra8UnormSrgb | TextureFormat::R8G8B8A8UnormSrgb => {
                warn!(target: "dx11", "Srgb format mapping - using non-SRGB variant");
                DXGI_FORMAT_B8G8R8A8_UNORM
            }
            _ => {
                warn!(target: "dx11", "Unknown texture format {:?}, defaulting to B8G8R8A8_UNORM", format);
                DXGI_FORMAT_B8G8R8A8_UNORM
            }
        }
    }

    #[cfg(target_os = "windows")]
    pub fn get_swap_chain(&self) -> &IDXGISwapChain1 {
        &self.swap_chain
    }

    #[cfg(target_os = "windows")]
    pub fn get_render_target_view(&self) -> Option<&ID3D11RenderTargetView> {
        self.render_target_view.as_ref()
    }

    pub fn set_vsync(&mut self, vsync: bool) {
        info!(target: "dx11", "Setting vsync: {}", vsync);
        self.vsync = vsync;
    }

    pub fn get_vsync(&self) -> bool {
        self.vsync
    }
}

impl ISwapChain for Dx11SwapChain {
    fn get_current_back_buffer_index(&self) -> u32 {
        #[cfg(target_os = "windows")]
        {
            unsafe { self.swap_chain.GetCurrentBackBufferIndex() }
        }
        #[cfg(not(target_os = "windows"))]
        {
            0
        }
    }

    fn get_back_buffer(&self) -> ResourceHandle {
        self.handle
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn resize(&mut self, width: u32, height: u32) -> RhiResult<()> {
        info!(target: "dx11", "Resize swapchain: {}x{}", width, height);

        #[cfg(target_os = "windows")]
        {
            // Release old views before resizing
            self.render_target_view = None;
            self.back_buffer = None;

            // Resize buffers
            unsafe {
                self.swap_chain.ResizeBuffers(
                    2, // Keep double buffering
                    width,
                    height,
                    Self::texture_format_to_dxgi(self.format),
                    if !self.vsync { DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING.0 } else { 0 },
                ).map_err(|e| {
                    error!(target: "dx11", "ResizeBuffers failed: {:?}", e);
                    RhiError::ResourceCreationFailed(format!("Resize: {:?}", e))
                })?;
            }

            // Recreate back buffer and RTV
            let device = unsafe {
                self.swap_chain.GetDevice::<ID3D11Device>().map_err(|e| {
                    error!(target: "dx11", "GetDevice failed: {:?}", e);
                    RhiError::InvalidResourceHandle("No device".to_string())
                })?
            };

            let back_buffer: ID3D11Texture2D = unsafe {
                self.swap_chain.GetBuffer(0).map_err(|e| {
                    error!(target: "dx11", "GetBuffer failed: {:?}", e);
                    RhiError::ResourceCreationFailed(format!("Back buffer: {:?}", e))
                })?
            };

            let render_target_view: ID3D11RenderTargetView = unsafe {
                device.CreateRenderTargetView(&back_buffer, None).map_err(|e| {
                    error!(target: "dx11", "CreateRenderTargetView failed: {:?}", e);
                    RhiError::ResourceCreationFailed(format!("RTV: {:?}", e))
                })?
            };

            self.back_buffer = Some(back_buffer);
            self.render_target_view = Some(render_target_view);
        }

        self.width = width;
        self.height = height;

        info!(target: "dx11", "Swapchain resized successfully");
        Ok(())
    }

    fn present(&self) -> RhiResult<()> {
        #[cfg(target_os = "windows")]
        {
            let sync_interval = if self.vsync { 1 } else { 0 };
            let present_flags = if !self.vsync { DXGI_PRESENT_ALLOW_TEARING } else { Default::default() };

            unsafe {
                self.swap_chain.Present(sync_interval, present_flags.0).map_err(|e| {
                    error!(target: "dx11", "Present failed: {:?}", e);
                    RhiError::InitializationFailed(format!("Present: {:?}", e))
                })?;
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            warn!(target: "dx11", "Present called on stub swapchain");
        }

        Ok(())
    }
}

impl Drop for Dx11SwapChain {
    fn drop(&mut self) {
        info!(target: "dx11", "Dropping swapchain");
        #[cfg(target_os = "windows")]
        {
            // COM objects are released automatically when refcount reaches 0
            self.render_target_view = None;
            self.back_buffer = None;
        }
    }
}
