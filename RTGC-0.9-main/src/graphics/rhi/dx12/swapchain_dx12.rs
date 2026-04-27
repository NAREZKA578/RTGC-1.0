// DirectX 12 Backend - Swap Chain Implementation
// Implements swap chain for presenting to window

use crate::graphics::rhi::types::*;
use std::sync::Arc;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Direct3D12::*,
    Win32::Graphics::Dxgi::*,
    Win32::Graphics::Dxgi::Common::*,
};

/// DX12 Swap Chain
pub struct Dx12SwapChain {
    #[cfg(target_os = "windows")]
    swap_chain: IDXGISwapChain3,
    
    #[cfg(target_os = "windows")]
    back_buffers: Vec<ID3D12Resource>,
    
    #[cfg(target_os = "windows")]
    rtv_handles: Vec<u64>, // RTV descriptor handles
    
    #[cfg(target_os = "windows")]
    device: Option<ID3D12Device>, // Stored device reference for resize operations
    
    width: u32,
    height: u32,
    format: TextureFormat,
    vsync: bool,
    current_index: u32,
}

unsafe impl Send for Dx12SwapChain {}
unsafe impl Sync for Dx12SwapChain {}

impl Dx12SwapChain {
    /// Create a new DX12 swap chain
    #[cfg(target_os = "windows")]
    pub fn new(
        device: &ID3D12Device,
        command_queue: &ID3D12CommandQueue,
        window_handle: *mut std::ffi::c_void,
        width: u32,
        height: u32,
        format: TextureFormat,
        vsync: bool,
    ) -> RhiResult<Self> {
        use windows::Win32::Graphics::Dxgi::*;
        
        // Get DXGI device
        let dxgi_device: IDXGIDevice = unsafe {
            device.cast::<IDXGIDevice>()
                .map_err(|e| RhiError::InitializationFailed(format!("Failed to get DXGI device: {:?}", e)))?
        };
        
        // Get adapter
        let adapter: IDXGIAdapter4 = unsafe {
            dxgi_device.GetAdapter()
                .map_err(|e| RhiError::InitializationFailed(format!("Failed to get adapter: {:?}", e)))?
        };
        
        // Get factory
        let factory: IDXGIFactory4 = unsafe {
            adapter.GetParent()
                .map_err(|e| RhiError::InitializationFailed(format!("Failed to get factory: {:?}", e)))?
        };
        
        // Create swap chain description
        let mut desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            Format: Self::to_dxgi_format(format),
            Stereo: false.into(),
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT | DXGI_USAGE_SHADER_INPUT,
            BufferCount: 2, // Double buffering
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            AlphaMode: DXGI_ALPHA_MODE_UNSPECIFIED,
            Flags: if vsync { 0 } else { DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT },
        };
        
        // Create swap chain
        let swap_chain: IDXGISwapChain1 = unsafe {
            factory.CreateSwapChainForHwnd(
                command_queue,
                HWND(window_handle as *mut _),
                &desc,
                None,
                None,
            )
            .map_err(|e| RhiError::InitializationFailed(format!("Failed to create swap chain: {:?}", e)))?
        };
        
        // Cast to IDXGISwapChain3
        let swap_chain: IDXGISwapChain3 = unsafe {
            swap_chain.cast()
                .map_err(|e| RhiError::InitializationFailed(format!("Failed to cast swap chain: {:?}", e)))?
        };
        
        // Disable Alt+Enter
        unsafe {
            factory.MakeWindowAssociation(HWND(window_handle as *mut _), DXGI_MWA_NO_ALT_ENTER);
        }
        
        // Get back buffers
        let buffer_count = 2;
        let mut back_buffers = Vec::with_capacity(buffer_count);
        
        for i in 0..buffer_count {
            let buffer: ID3D12Resource = unsafe {
                swap_chain.GetBuffer(i)
                    .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to get back buffer: {:?}", e)))?
            };
            back_buffers.push(buffer);
        }
        
        // Create RTV descriptor heap for back buffers
        let rtv_heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: back_buffer_count,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            NodeMask: 0,
        };
        
        let rtv_heap: ID3D12DescriptorHeap = unsafe {
            device.CreateDescriptorHeap(&rtv_heap_desc)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create RTV heap: {:?}", e)))?
        };
        
        // Create RTV descriptors for each back buffer
        let rtv_handle_size = unsafe {
            device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV)
        };
        let rtv_heap_start = unsafe { rtv_heap.GetCPUDescriptorHandleForHeapStart() };
        
        let mut rtv_handles = Vec::with_capacity(back_buffer_count as usize);
        
        for (i, buffer) in back_buffers.iter().enumerate() {
            let rtv_handle = D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: rtv_heap_start.ptr + (rtv_handle_size as usize) * i,
            };
            
            unsafe {
                device.CreateRenderTargetView(buffer, None, rtv_handle);
            }
            
            rtv_handles.push(rtv_handle.ptr as u64);
        }
        
        Ok(Self {
            swap_chain,
            back_buffers,
            rtv_handles,
            device: Some(device.clone()), // Store device reference for resize operations
            width,
            height,
            format,
            vsync,
            current_index: 0,
        })
    }
    
    #[cfg(target_os = "windows")]
    fn to_dxgi_format(format: TextureFormat) -> DXGI_FORMAT {
        match format {
            TextureFormat::R8G8B8A8Unorm | TextureFormat::R8G8B8A8Srgb => DXGI_FORMAT_R8G8B8A8_UNORM,
            TextureFormat::R16G16B16A16Float => DXGI_FORMAT_R16G16B16A16_FLOAT,
            TextureFormat::R32G32B32A32Float => DXGI_FORMAT_R32G32B32A32_FLOAT,
            _ => DXGI_FORMAT_R8G8B8A8_UNORM,
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    pub fn new(
        _device: &ID3D12Device,
        _command_queue: &ID3D12CommandQueue,
        _window_handle: *mut std::ffi::c_void,
        _width: u32,
        _height: u32,
        _format: TextureFormat,
        _vsync: bool,
    ) -> RhiResult<Self> {
        Err(RhiError::Unsupported("DirectX 12 is only available on Windows".to_string()))
    }
    
    /// Get the current back buffer index
    pub fn get_current_back_buffer_index(&self) -> u32 {
        self.current_index
    }
    
    /// Get the back buffer texture at specified index
    #[cfg(target_os = "windows")]
    pub fn get_back_buffer(&self, index: u32) -> Option<&ID3D12Resource> {
        self.back_buffers.get(index as usize)
    }
    
    /// Resize the swap chain
    #[cfg(target_os = "windows")]
    pub fn resize(&mut self, width: u32, height: u32) -> RhiResult<()> {
        self.width = width;
        self.height = height;
        
        unsafe {
            self.swap_chain.ResizeBuffers(
                2,
                width,
                height,
                DXGI_FORMAT_UNKNOWN,
                if self.vsync { 0 } else { DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT },
            )
            .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to resize swap chain: {:?}", e)))?;
        }
        
        // Recreate back buffers
        self.back_buffers.clear();
        
        for i in 0..2 {
            let buffer: ID3D12Resource = unsafe {
                self.swap_chain.GetBuffer(i)
                    .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to get back buffer: {:?}", e)))?
            };
            self.back_buffers.push(buffer);
        }
        
        // Recreate RTV descriptors
        self.rtv_handles.clear();
        
        // Use stored device reference to recreate RTV descriptors
        #[cfg(target_os = "windows")]
        {
            if let Some(ref device) = self.device {
                let rtv_heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
                    Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                    NumDescriptors: 2,
                    Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
                    NodeMask: 0,
                };
                
                let rtv_heap: ID3D12DescriptorHeap = unsafe {
                    device.CreateDescriptorHeap(&rtv_heap_desc)
                        .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create RTV heap: {:?}", e)))?
                };
                
                let rtv_handle_size = unsafe {
                    device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV)
                };
                let rtv_heap_start = unsafe { rtv_heap.GetCPUDescriptorHandleForHeapStart() };
                
                for (i, buffer) in self.back_buffers.iter().enumerate() {
                    let rtv_handle = D3D12_CPU_DESCRIPTOR_HANDLE {
                        ptr: rtv_heap_start.ptr + (rtv_handle_size as usize) * i,
                    };
                    
                    unsafe {
                        device.CreateRenderTargetView(buffer, None, rtv_handle);
                    }
                    
                    self.rtv_handles.push(rtv_handle.ptr as u64);
                }
            } else {
                return Err(RhiError::ResourceCreationFailed("Device reference not available for RTV recreation".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Present the current frame
    #[cfg(target_os = "windows")]
    pub fn present(&self) -> RhiResult<()> {
        unsafe {
            let sync_interval = if self.vsync { 1 } else { 0 };
            self.swap_chain.Present(sync_interval, 0)
                .map_err(|e| RhiError::DeviceLost)?;
        }
        
        Ok(())
    }
    
    pub fn width(&self) -> u32 {
        self.width
    }
    
    pub fn height(&self) -> u32 {
        self.height
    }
    
    pub fn format(&self) -> TextureFormat {
        self.format
    }
    
    pub fn vsync(&self) -> bool {
        self.vsync
    }
}

impl super::device::ISwapChain for Dx12SwapChain {
    fn get_current_back_buffer_index(&self) -> u32 {
        self.current_index
    }
    
    fn get_back_buffer(&self) -> ResourceHandle {
        // Return handle to current back buffer
        ResourceHandle(self.current_index as u64)
    }
    
    fn resize(&mut self, width: u32, height: u32) -> RhiResult<()> {
        self.resize(width, height)
    }
    
    fn present(&self) -> RhiResult<()> {
        self.present()
    }
}
