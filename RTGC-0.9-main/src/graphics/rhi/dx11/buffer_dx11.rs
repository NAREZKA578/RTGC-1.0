//! DirectX 11 Buffer - Full Implementation
//! Supports Vertex, Index, Constant, Structured, Dynamic buffers with map/unmap/update

use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{error, info, warn};

use crate::graphics::rhi::device::*;
use crate::graphics::rhi::types::*;
use crate::graphics::rhi::RhiResult;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Graphics::Direct3D11::{
        D3D11_BUFFER_DESC, D3D11_MAPPED_SUBRESOURCE,
        D3D11_MAP_WRITE_DISCARD, D3D11_MAP_READ, D3D11_MAP_WRITE,
        D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC, D3D11_USAGE_IMMUTABLE,
        D3D11_CPU_ACCESS_WRITE, D3D11_CPU_ACCESS_READ,
        D3D11_BIND_VERTEX_BUFFER, D3D11_BIND_INDEX_BUFFER, D3D11_BIND_CONSTANT_BUFFER,
        D3D11_BIND_SHADER_RESOURCE, D3D11_BIND_UNORDERED_ACCESS,
        D3D11_RESOURCE_MISC_BUFFER_STRUCTURED,
        ID3D11Buffer,
    },
};

/// DX11 Buffer resource
pub struct Dx11Buffer {
    #[cfg(target_os = "windows")]
    buffer: Option<ID3D11Buffer>,
    
    handle: ResourceHandle,
    desc: BufferDescription,
    size: u64,
    name: String,
    resource_counter: AtomicU64,
}

#[cfg(target_os = "windows")]
unsafe impl Send for Dx11Buffer {}

#[cfg(target_os = "windows")]
unsafe impl Sync for Dx11Buffer {}

impl Dx11Buffer {
    /// Create a new buffer from description
    #[cfg(target_os = "windows")]
    pub fn new(
        device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
        desc: &BufferDescription,
        initial_data: Option<&[u8]>,
    ) -> RhiResult<Self> {
        info!(target: "dx11.buffer", "Creating buffer: type={:?}, size={}, usage={:?}", 
              desc.buffer_type, desc.size, desc.usage);
        
        // Determine bind flags based on buffer type and usage
        let mut bind_flags = 0u32;
        match desc.buffer_type {
            BufferType::Vertex => bind_flags |= D3D11_BIND_VERTEX_BUFFER.0 as u32,
            BufferType::Index => bind_flags |= D3D11_BIND_INDEX_BUFFER.0 as u32,
            BufferType::Constant => bind_flags |= D3D11_BIND_CONSTANT_BUFFER.0 as u32,
            BufferType::Storage => {
                bind_flags |= D3D11_BIND_UNORDERED_ACCESS.0 as u32;
                if desc.usage.contains(BufferUsage::SHADER_RESOURCE) {
                    bind_flags |= D3D11_BIND_SHADER_RESOURCE.0 as u32;
                }
            }
            BufferType::Indirect => {}
            BufferType::Uniform => bind_flags |= D3D11_BIND_CONSTANT_BUFFER.0 as u32,
        }
        
        // Add additional bind flags from usage
        if desc.usage.contains(BufferUsage::SHADER_RESOURCE) && desc.buffer_type != BufferType::Storage {
            bind_flags |= D3D11_BIND_SHADER_RESOURCE.0 as u32;
        }
        if desc.usage.contains(BufferUsage::UNORDERED_ACCESS) && desc.buffer_type != BufferType::Storage {
            bind_flags |= D3D11_BIND_UNORDERED_ACCESS.0 as u32;
        }
        
        // Determine usage and CPU access flags
        let (usage, cpu_access) = if desc.usage.contains(BufferUsage::DYNAMIC) 
            || desc.usage.contains(BufferUsage::TRANSIENT)
            || desc.usage.contains(BufferUsage::UPLOAD) {
            (D3D11_USAGE_DYNAMIC, D3D11_CPU_ACCESS_WRITE.0 as u32)
        } else if desc.usage.contains(BufferUsage::READBACK) {
            (D3D11_USAGE_DYNAMIC, (D3D11_CPU_ACCESS_READ.0 | D3D11_CPU_ACCESS_WRITE.0) as u32)
        } else if desc.usage.contains(BufferUsage::IMMUTABLE) {
            (D3D11_USAGE_IMMUTABLE, 0)
        } else {
            (D3D11_USAGE_DEFAULT, 0)
        };
        
        // Determine misc flags
        let mut misc_flags: u32 = 0;
        if desc.usage.contains(BufferUsage::STORAGE_BUFFER) || desc.buffer_type == BufferType::Storage {
            misc_flags |= D3D11_RESOURCE_MISC_BUFFER_STRUCTURED.0 as u32;
        }
        
        let buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: desc.size as u32,
            Usage: usage,
            BindFlags: D3D11_BIND_FLAG(bind_flags.0),
            CPUAccessFlags: D3D11_CPU_ACCESS_FLAG(cpu_access.0),
            MiscFlags: D3D11_RESOURCE_MISC_FLAG(misc_flags.0),
            StructureByteStride: if misc_flags.0 & D3D11_RESOURCE_MISC_BUFFER_STRUCTURED.0 != 0 {
                4
            } else {
                0
            },
        };
        
        info!(target: "dx11.buffer", "Buffer desc: ByteWidth={}, Usage={:?}, BindFlags=0x{:x}, CPUAccess=0x{:x}, MiscFlags=0x{:x}",
              buffer_desc.ByteWidth, buffer_desc.Usage, buffer_desc.BindFlags.0, buffer_desc.CPUAccessFlags.0, buffer_desc.MiscFlags.0);
        
        // Create buffer
        let buffer = unsafe {
            if let Some(data) = initial_data {
                use windows::Win32::Graphics::Direct3D11::D3D11_SUBRESOURCE_DATA;
                
                let subresource_data = D3D11_SUBRESOURCE_DATA {
                    pSysMem: data.as_ptr() as *const _,
                    SysMemPitch: 0,
                    SysMemSlicePitch: 0,
                };
                
                device.CreateBuffer(&buffer_desc, Some(&subresource_data))
            } else {
                device.CreateBuffer(&buffer_desc, None)
            }
        };
        
        let buffer = match buffer {
            Ok(b) => {
                info!(target: "dx11.buffer", "ID3D11Buffer created successfully");
                Some(b)
            }
            Err(e) => {
                error!(target: "dx11.buffer", "Failed to create buffer: {:?}", e);
                return Err(RhiError::InitializationFailed(format!(
                    "CreateBuffer: {:?}",
                    e
                )));
            }
        };
        
        let name = format!("DX11Buffer({:?}, {} bytes)", desc.buffer_type, desc.size);
        
        Ok(Self {
            buffer,
            handle: ResourceHandle(AtomicU64::new(1).load(Ordering::Relaxed)),
            desc: desc.clone(),
            size: desc.size,
            name,
            resource_counter: AtomicU64::new(1),
        })
    }
    
    /// Create vertex buffer from typed data
    #[cfg(target_os = "windows")]
    pub fn create_vertex_buffer<T: bytemuck::Pod>(
        device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
        data: &[T],
    ) -> RhiResult<Self> {
        info!(target: "dx11.buffer", "Creating vertex buffer: {} vertices, {} bytes each", 
              data.len(), std::mem::size_of::<T>());
        
        let desc = BufferDescription {
            buffer_type: BufferType::Vertex,
            size: (data.len() * std::mem::size_of::<T>()) as u64,
            usage: BufferUsage::VERTEX_BUFFER,
            initial_state: ResourceState::VertexBuffer,
            initial_data: None,
        };
        
        let byte_data = bytemuck::cast_slice::<T, u8>(data);
        Self::new(device, &desc, Some(byte_data))
    }
    
    /// Create index buffer from u32 indices
    #[cfg(target_os = "windows")]
    pub fn create_index_buffer(
        device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
        data: &[u32],
    ) -> RhiResult<Self> {
        info!(target: "dx11.buffer", "Creating index buffer: {} indices", data.len());
        
        let desc = BufferDescription {
            buffer_type: BufferType::Index,
            size: (data.len() * 4) as u64,
            usage: BufferUsage::INDEX_BUFFER,
            initial_state: ResourceState::IndexBuffer,
            initial_data: None,
        };
        
        let byte_data = bytemuck::cast_slice::<u32, u8>(data);
        Self::new(device, &desc, Some(byte_data))
    }
    
    /// Create index buffer from u16 indices
    #[cfg(target_os = "windows")]
    pub fn create_index_buffer_u16(
        device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
        data: &[u16],
    ) -> RhiResult<Self> {
        info!(target: "dx11.buffer", "Creating index buffer (u16): {} indices", data.len());
        
        let desc = BufferDescription {
            buffer_type: BufferType::Index,
            size: (data.len() * 2) as u64,
            usage: BufferUsage::INDEX_BUFFER,
            initial_state: ResourceState::IndexBuffer,
            initial_data: None,
        };
        
        let byte_data = bytemuck::cast_slice::<u16, u8>(data);
        Self::new(device, &desc, Some(byte_data))
    }
    
    /// Create constant buffer
    #[cfg(target_os = "windows")]
    pub fn create_constant_buffer<T: bytemuck::Pod>(
        device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
        initial_data: Option<&T>,
    ) -> RhiResult<Self> {
        let size = std::mem::size_of::<T>() as u64;
        // Constant buffers must be aligned to 16 bytes in DX11
        let aligned_size = ((size + 15) / 16) * 16;
        
        info!(target: "dx11.buffer", "Creating constant buffer: {} bytes (aligned: {})", size, aligned_size);
        
        let desc = BufferDescription {
            buffer_type: BufferType::Constant,
            size: aligned_size,
            usage: BufferUsage::CONSTANT_BUFFER | BufferUsage::DYNAMIC,
            initial_state: ResourceState::ConstantBuffer,
            initial_data: None,
        };
        
        let byte_data = initial_data.map(|d| bytemuck::bytes_of(d));
        Self::new(device, &desc, byte_data)
    }
    
    /// Create structured buffer (for compute shaders / UAV)
    #[cfg(target_os = "windows")]
    pub fn create_structured_buffer<T: bytemuck::Pod>(
        device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
        data: Option<&[T]>,
    ) -> RhiResult<Self> {
        let element_count = data.map(|d| d.len()).unwrap_or(0);
        let element_size = std::mem::size_of::<T>() as u64;
        let total_size = if element_count > 0 {
            (element_count as u64) * element_size
        } else {
            1024 // Default size if no data provided
        };
        
        info!(target: "dx11.buffer", "Creating structured buffer: {} elements, {} bytes each", 
              element_count, element_size);
        
        let desc = BufferDescription {
            buffer_type: BufferType::Storage,
            size: total_size,
            usage: BufferUsage::STORAGE_BUFFER | BufferUsage::SHADER_RESOURCE,
            initial_state: ResourceState::UnorderedAccess,
            initial_data: None,
        };
        
        let byte_data = data.map(|d| bytemuck::cast_slice::<T, u8>(d));
        Self::new(device, &desc, byte_data)
    }
    
    /// Get the underlying ID3D11Buffer
    #[cfg(target_os = "windows")]
    pub fn get_buffer(&self) -> &Option<ID3D11Buffer> {
        &self.buffer
    }
    
    /// Get buffer size
    pub fn get_size(&self) -> u64 {
        self.size
    }
    
    /// Get buffer description
    pub fn get_description(&self) -> &BufferDescription {
        &self.desc
    }
    
    /// Map buffer for CPU write access
    #[cfg(target_os = "windows")]
    pub fn map(&self, context: &windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext) -> RhiResult<*mut u8> {
        info!(target: "dx11.buffer", "Mapping buffer: {:?}", self.handle);
        
        if self.buffer.is_none() {
            return Err(RhiError::InvalidResource("Buffer not created".to_string()));
        }
        
        unsafe {
            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            let hr = context.Map(
                self.buffer.as_ref().ok_or("Buffer access failed")?,
                0,
                D3D11_MAP_WRITE_DISCARD,
                0,
                Some(&mut mapped),
            );
            
            if hr.is_err() {
                error!(target: "dx11.buffer", "Failed to map buffer: {:?}", hr);
                return Err(RhiError::OperationFailed(format!("Map: {:?}", hr)));
            }
            
            Ok(mapped.pData as *mut u8)
        }
    }
    
    /// Unmap buffer after CPU write
    #[cfg(target_os = "windows")]
    pub fn unmap(&self, context: &windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext) {
        info!(target: "dx11.buffer", "Unmapping buffer: {:?}", self.handle);
        
        if let Some(ref buffer) = self.buffer {
            unsafe {
                context.Unmap(buffer, 0);
            }
        }
    }
    
    /// Update buffer data (for dynamic/immutable buffers)
    #[cfg(target_os = "windows")]
    pub fn update(
        &self,
        context: &windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext,
        offset: u64,
        data: &[u8],
    ) -> RhiResult<()> {
        info!(target: "dx11.buffer", "Updating buffer: offset={}, size={}", offset, data.len());
        
        if offset + data.len() as u64 > self.size {
            return Err(RhiError::OutOfBounds(format!(
                "Buffer update out of bounds: offset={} + size={} > total={}",
                offset, data.len(), self.size
            )));
        }
        
        // For dynamic buffers, use map/unmap
        if self.desc.usage.contains(BufferUsage::DYNAMIC) {
            let ptr = self.map(context)?;
            unsafe {
                std::ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    ptr.add(offset as usize),
                    data.len(),
                );
            }
            self.unmap(context);
            Ok(())
        } else {
            // For default/immutable buffers, we'd need to use UpdateSubresource
            // This is a simplified version
            warn!(target: "dx11.buffer", "Update on non-dynamic buffer may not work correctly");
            Err(RhiError::Unsupported(
                "Update requires dynamic buffer".to_string()
            ))
        }
    }
    
    /// Bind as vertex buffer
    #[cfg(target_os = "windows")]
    pub fn bind_vertex(
        &self,
        context: &windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext,
        slot: u32,
        offset: u64,
    ) {
        info!(target: "dx11.buffer", "Binding as vertex buffer: slot={}, offset={}", slot, offset);
        
        if let Some(ref buffer) = self.buffer {
            unsafe {
                let stride = self.get_stride();
                context.IASetVertexBuffers(slot, &[Some(buffer)], &[stride], &[offset as u32]);
            }
        }
    }
    
    /// Bind as index buffer
    #[cfg(target_os = "windows")]
    pub fn bind_index(
        &self,
        context: &windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext,
        format: IndexFormat,
        offset: u64,
    ) {
        info!(target: "dx11.buffer", "Binding as index buffer: format={:?}", format);
        
        if let Some(ref buffer) = self.buffer {
            use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R16_UINT, DXGI_FORMAT_R32_UINT};
            
            let dxgi_format = match format {
                IndexFormat::Uint16 => DXGI_FORMAT_R16_UINT,
                IndexFormat::Uint32 => DXGI_FORMAT_R32_UINT,
            };
            
            unsafe {
                context.IASetIndexBuffer(Some(buffer), dxgi_format, offset as u32);
            }
        }
    }
    
    /// Bind as constant buffer
    #[cfg(target_os = "windows")]
    pub fn bind_constant(
        &self,
        context: &windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext,
        slot: u32,
        stage: ShaderStage,
    ) {
        info!(target: "dx11.buffer", "Binding as constant buffer: slot={}, stage={:?}", slot, stage);
        
        if let Some(ref buffer) = self.buffer {
            unsafe {
                match stage {
                    ShaderStage::Vertex => context.VSSetConstantBuffers(slot, &[Some(buffer)]),
                    ShaderStage::Fragment => context.PSSetConstantBuffers(slot, &[Some(buffer)]),
                    ShaderStage::Compute => context.CSSetConstantBuffers(slot, &[Some(buffer)]),
                    ShaderStage::Geometry => context.GSSetConstantBuffers(slot, &[Some(buffer)]),
                    _ => warn!(target: "dx11.buffer", "Constant buffer binding not supported for stage {:?}", stage),
                }
            }
        }
    }
    
    fn get_stride(&self) -> u32 {
        match self.desc.buffer_type {
            BufferType::Vertex => {
                // Would need to know vertex format - this is a simplification
                32
            }
            BufferType::Index => 4,
            _ => 0,
        }
    }
}

// Stub implementation for non-Windows platforms
#[cfg(not(target_os = "windows"))]
impl Dx11Buffer {
    pub fn new(_device: &(), _desc: &BufferDescription, _initial_data: Option<&[u8]>) -> RhiResult<Self> {
        warn!(target: "dx11.buffer", "DX11 buffer creation requires Windows");
        Ok(Self {
            handle: ResourceHandle(0),
            desc: _desc.clone(),
            size: _desc.size,
            name: "DX11Buffer(stub)".to_string(),
            resource_counter: AtomicU64::new(1),
        })
    }
    
    pub fn get_size(&self) -> u64 {
        self.size
    }
    
    pub fn get_description(&self) -> &BufferDescription {
        &self.desc
    }
}
