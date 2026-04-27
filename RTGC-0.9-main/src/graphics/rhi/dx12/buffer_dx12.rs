// DirectX 12 Backend - Buffer Implementation
// Implements GPU buffer resources for DX12

use crate::graphics::rhi::{
    types::*,
    device::IDevice,
};
use std::sync::Arc;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Direct3D12::*,
    Win32::System::Memory::*,
};

/// DX12 Buffer resource
pub struct Dx12Buffer {
    #[cfg(target_os = "windows")]
    resource: ID3D12Resource,
    
    handle: ResourceHandle,
    description: BufferDescription,
    mapped_ptr: *mut u8,
}

unsafe impl Send for Dx12Buffer {}
unsafe impl Sync for Dx12Buffer {}

impl Dx12Buffer {
    /// Create a new DX12 buffer
    #[cfg(target_os = "windows")]
    pub fn new(
        device: &ID3D12Device,
        desc: &BufferDescription,
        handle: ResourceHandle,
    ) -> RhiResult<Self> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        let heap_properties = D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_DEFAULT,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };
        
        let mut resource_flags = D3D12_RESOURCE_FLAG_NONE;
        
        if desc.usage.contains(BufferUsage::UNORDERED_ACCESS) {
            resource_flags |= D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
        }
        
        let buffer_desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Alignment: 0,
            Width: desc.size,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: DXGI_FORMAT_UNKNOWN,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: resource_flags,
        };
        
        let resource: ID3D12Resource = unsafe {
            device.CreateCommittedResource(
                &heap_properties,
                D3D12_HEAP_FLAG_NONE,
                &buffer_desc,
                Self::get_initial_state(desc.initial_state),
                None,
            )
            .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create buffer: {:?}", e)))?
        };
        
        Ok(Self {
            resource,
            handle,
            description: desc.clone(),
            mapped_ptr: std::ptr::null_mut(),
        })
    }
    
    /// Create an upload heap buffer for staging data
    #[cfg(target_os = "windows")]
    pub fn new_upload(
        device: &ID3D12Device,
        size: u64,
        handle: ResourceHandle,
    ) -> RhiResult<Self> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        let heap_properties = D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_UPLOAD,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };
        
        let buffer_desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Alignment: 0,
            Width: size,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: DXGI_FORMAT_UNKNOWN,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: D3D12_RESOURCE_FLAG_NONE,
        };
        
        let resource: ID3D12Resource = unsafe {
            device.CreateCommittedResource(
                &heap_properties,
                D3D12_HEAP_FLAG_NONE,
                &buffer_desc,
                D3D12_RESOURCE_STATE_GENERIC_READ,
                None,
            )
            .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create upload buffer: {:?}", e)))?
        };
        
        let desc = BufferDescription {
            buffer_type: BufferType::Constant,
            size,
            usage: BufferUsage::CONSTANT_BUFFER | BufferUsage::TRANSFER_DST,
            initial_state: ResourceState::GenericRead,
            initial_data: None,
        };
        
        Ok(Self {
            resource,
            handle,
            description: desc,
            mapped_ptr: std::ptr::null_mut(),
        })
    }
    
    #[cfg(target_os = "windows")]
    fn get_initial_state(state: ResourceState) -> D3D12_RESOURCE_STATES {
        match state {
            ResourceState::Undefined => D3D12_RESOURCE_STATE_COMMON,
            ResourceState::Common => D3D12_RESOURCE_STATE_COMMON,
            ResourceState::VertexBuffer => D3D12_RESOURCE_STATE_VERTEX_AND_CONSTANT_BUFFER,
            ResourceState::IndexBuffer => D3D12_RESOURCE_STATE_INDEX_BUFFER,
            ResourceState::ConstantBuffer => D3D12_RESOURCE_STATE_VERTEX_AND_CONSTANT_BUFFER,
            ResourceState::ShaderResource => D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE 
                | D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            ResourceState::UnorderedAccess => D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            ResourceState::RenderTarget => D3D12_RESOURCE_STATE_RENDER_TARGET,
            ResourceState::DepthWrite => D3D12_RESOURCE_STATE_DEPTH_WRITE,
            ResourceState::DepthRead => D3D12_RESOURCE_STATE_DEPTH_READ,
            ResourceState::Present => D3D12_RESOURCE_STATE_PRESENT,
            ResourceState::TransferSource => D3D12_RESOURCE_STATE_COPY_SOURCE,
            ResourceState::TransferDestination => D3D12_RESOURCE_STATE_COPY_DEST,
        }
    }
    
    #[cfg(target_os = "windows")]
    pub fn resource(&self) -> &ID3D12Resource {
        &self.resource
    }
    
    pub fn handle(&self) -> ResourceHandle {
        self.handle
    }
    
    pub fn description(&self) -> &BufferDescription {
        &self.description
    }
    
    /// Map the buffer for CPU write access
    #[cfg(target_os = "windows")]
    pub fn map(&mut self) -> RhiResult<*mut u8> {
        if !self.mapped_ptr.is_null() {
            return Ok(self.mapped_ptr);
        }
        
        unsafe {
            let ptr = self.resource.Map(0, None::<*const D3D12_RANGE>)
                .map_err(|e| RhiError::InvalidParameter(format!("Failed to map buffer: {:?}", e)))?;
            
            if ptr.is_null() {
                return Err(RhiError::InvalidParameter("Map returned null pointer".to_string()));
            }
            
            self.mapped_ptr = ptr as *mut u8;
            Ok(self.mapped_ptr)
        }
    }
    
    /// Unmap the buffer after CPU write
    #[cfg(target_os = "windows")]
    pub fn unmap(&mut self) {
        if !self.mapped_ptr.is_null() {
            unsafe {
                self.resource.Unmap(0, None::<*const D3D12_RANGE>);
            }
            self.mapped_ptr = std::ptr::null_mut();
        }
    }
}

/// Helper function to create DX12 buffer via IDevice
#[cfg(target_os = "windows")]
pub fn create_dx12_buffer(
    device: &ID3D12Device,
    desc: &BufferDescription,
) -> RhiResult<Dx12Buffer> {
    static HANDLE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let handle = ResourceHandle(HANDLE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
    
    Dx12Buffer::new(device, desc, handle)
}
