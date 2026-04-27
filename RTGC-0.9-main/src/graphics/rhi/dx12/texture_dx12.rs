// DirectX 12 Backend - Texture Implementation
// Implements GPU texture resources for DX12

use crate::graphics::rhi::types::*;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Direct3D12::*,
    Win32::Graphics::Dxgi::Common::*,
};

/// DX12 Texture resource
pub struct Dx12Texture {
    #[cfg(target_os = "windows")]
    resource: ID3D12Resource,
    
    #[cfg(target_os = "windows")]
    srv_handle: Option<u64>, // Descriptor handle for SRV
    
    handle: ResourceHandle,
    description: TextureDescription,
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    mip_levels: u32,
}

unsafe impl Send for Dx12Texture {}
unsafe impl Sync for Dx12Texture {}

impl Dx12Texture {
    /// Create a new DX12 texture
    #[cfg(target_os = "windows")]
    pub fn new(
        device: &ID3D12Device,
        desc: &TextureDescription,
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
        
        if desc.usage.contains(TextureUsage::RENDER_TARGET) {
            resource_flags |= D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET;
        }
        
        if desc.usage.contains(TextureUsage::DEPTH_STENCIL) {
            resource_flags |= D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL;
        }
        
        if desc.usage.contains(TextureUsage::UNORDERED_ACCESS) {
            resource_flags |= D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
        }
        
        let dxgi_format = Self::to_dxgi_format(desc.format);
        
        let (dimension, width, height, depth_or_array_size) = match desc.dimension {
            TextureDimension::D1 => (D3D12_RESOURCE_DIMENSION_TEXTURE1D, desc.width, 1, 1),
            TextureDimension::D2 => (D3D12_RESOURCE_DIMENSION_TEXTURE2D, desc.width, desc.height, desc.depth_or_array_layers),
            TextureDimension::D3 => (D3D12_RESOURCE_DIMENSION_TEXTURE3D, desc.width, desc.height, desc.depth_or_array_layers),
            TextureDimension::Cube => (D3D12_RESOURCE_DIMENSION_TEXTURE2D, desc.width, desc.height, desc.depth_or_array_layers * 6),
        };
        
        let texture_desc = D3D12_RESOURCE_DESC {
            Dimension: dimension,
            Alignment: 0,
            Width: width as u64,
            Height: height,
            DepthOrArraySize: depth_or_array_size,
            MipLevels: desc.mip_levels,
            Format: dxgi_format,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
            Flags: resource_flags,
        };
        
        let initial_state = Self::get_initial_state(desc.initial_state);
        
        let resource: ID3D12Resource = unsafe {
            device.CreateCommittedResource(
                &heap_properties,
                D3D12_HEAP_FLAG_NONE,
                &texture_desc,
                initial_state,
                None,
            )
            .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create texture: {:?}", e)))?
        };
        
        Ok(Self {
            resource,
            srv_handle: None,
            handle,
            description: desc.clone(),
            width: desc.width,
            height: desc.height,
            depth_or_array_layers: desc.depth_or_array_layers,
            mip_levels: desc.mip_levels,
        })
    }
    
    /// Create a texture from raw data
    #[cfg(target_os = "windows")]
    pub fn new_from_data(
        device: &ID3D12Device,
        command_queue: &ID3D12CommandQueue,
        desc: &TextureDescription,
        data: &[u8],
        handle: ResourceHandle,
    ) -> RhiResult<Self> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        let mut texture = Self::new(device, desc, handle)?;
        
        // Create upload buffer
        let upload_size = data.len() as u64;
        let upload_buffer = super::buffer_dx12::Dx12Buffer::new_upload(device, upload_size, ResourceHandle(handle.0 + 1))?;
        
        // Map and copy data
        unsafe {
            let ptr = upload_buffer.map()?;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
            upload_buffer.unmap();
        }
        
        // Create command list for copy operation
        let command_allocator: ID3D12CommandAllocator = device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)
            .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create command allocator: {:?}", e)))?;
        
        let command_list: ID3D12GraphicsCommandList = device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &command_allocator, None)
            .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create command list: {:?}", e)))?;
        
        // Transition texture to COPY_DEST
        // SAFETY: D3D12_RESOURCE_TRANSITION_BARRIER is a POD (plain old data) struct
        // with no padding or special alignment requirements. Transmuting to the
        // anonymous union member is safe and is the standard pattern for DirectX 12 FFI.
        let barrier = D3D12_RESOURCE_BARRIER {
            Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
            Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
            Anonymous: D3D12_RESOURCE_BARRIER_0 {
                Transition: std::mem::transmute(D3D12_RESOURCE_TRANSITION_BARRIER {
                    pResource: Some(texture.resource.clone()),
                    Subresource: 0,
                    StateBefore: D3D12_RESOURCE_STATE_COMMON,
                    StateAfter: D3D12_RESOURCE_STATE_COPY_DEST,
                }),
            },
        };
        
        unsafe {
            command_list.ResourceBarrier(&[barrier]);
            
            // Copy from upload buffer to texture
            command_list.CopyBufferRegion(
                &texture.resource,
                0,
                &upload_buffer.resource(),
                0,
                upload_size,
            );
            
            // Transition back to appropriate state
            // SAFETY: Same as above - transmuting POD struct to union member is safe.
            let barrier_back = D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: std::mem::transmute(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: Some(texture.resource.clone()),
                        Subresource: 0,
                        StateBefore: D3D12_RESOURCE_STATE_COPY_DEST,
                        StateAfter: D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
                    }),
                },
            };
            
            command_list.ResourceBarrier(&[barrier_back]);
            command_list.Close()?;
            
            // Execute and wait for completion
            command_queue.ExecuteCommandLists(&[command_list.cast().map_err(|_| RhiError::InitializationFailed("Failed to cast to ID3D12CommandList"))?]);
        }
        
        Ok(texture)
    }
    
    #[cfg(target_os = "windows")]
    fn to_dxgi_format(format: TextureFormat) -> DXGI_FORMAT {
        match format {
            TextureFormat::R8Unorm => DXGI_FORMAT_R8_UNORM,
            TextureFormat::R8G8Unorm => DXGI_FORMAT_R8G8_UNORM,
            TextureFormat::R8G8B8A8Unorm => DXGI_FORMAT_R8G8B8A8_UNORM,
            TextureFormat::R8G8B8A8Srgb => DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
            TextureFormat::R16Float => DXGI_FORMAT_R16_FLOAT,
            TextureFormat::R16G16Float => DXGI_FORMAT_R16G16_FLOAT,
            TextureFormat::R16G16B16A16Float => DXGI_FORMAT_R16G16B16A16_FLOAT,
            TextureFormat::R32Float => DXGI_FORMAT_R32_FLOAT,
            TextureFormat::R32G32Float => DXGI_FORMAT_R32G32_FLOAT,
            TextureFormat::R32G32B32A32Float => DXGI_FORMAT_R32G32B32A32_FLOAT,
            TextureFormat::D16Unorm => DXGI_FORMAT_D16_UNORM,
            TextureFormat::D24UnormS8Uint => DXGI_FORMAT_D24_UNORM_S8_UINT,
            TextureFormat::D32Float => DXGI_FORMAT_D32_FLOAT,
            TextureFormat::D32FloatS8UintX24 => DXGI_FORMAT_D32_FLOAT_S8X24_UINT,
            TextureFormat::BC1RgbUnorm => DXGI_FORMAT_BC1_RGB_UNORM,
            TextureFormat::BC1RgbaUnorm => DXGI_FORMAT_BC1_UNORM,
            TextureFormat::BC2Unorm => DXGI_FORMAT_BC2_UNORM,
            TextureFormat::BC3Unorm => DXGI_FORMAT_BC3_UNORM,
            TextureFormat::BC4Unorm => DXGI_FORMAT_BC4_UNORM,
            TextureFormat::BC5Unorm => DXGI_FORMAT_BC5_UNORM,
            TextureFormat::BC6HUfloat => DXGI_FORMAT_BC6H_UF16,
            TextureFormat::BC7Unorm => DXGI_FORMAT_BC7_UNORM,
            _ => DXGI_FORMAT_UNKNOWN,
        }
    }
    
    #[cfg(target_os = "windows")]
    fn get_initial_state(state: ResourceState) -> D3D12_RESOURCE_STATES {
        match state {
            ResourceState::Undefined => D3D12_RESOURCE_STATE_COMMON,
            ResourceState::Common => D3D12_RESOURCE_STATE_COMMON,
            ResourceState::ShaderResource => D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE 
                | D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            ResourceState::RenderTarget => D3D12_RESOURCE_STATE_RENDER_TARGET,
            ResourceState::DepthWrite => D3D12_RESOURCE_STATE_DEPTH_WRITE,
            ResourceState::DepthRead => D3D12_RESOURCE_STATE_DEPTH_READ,
            ResourceState::UnorderedAccess => D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            ResourceState::TransferSource => D3D12_RESOURCE_STATE_COPY_SOURCE,
            ResourceState::TransferDestination => D3D12_RESOURCE_STATE_COPY_DEST,
            ResourceState::Present => D3D12_RESOURCE_STATE_PRESENT,
            _ => D3D12_RESOURCE_STATE_COMMON,
        }
    }
    
    #[cfg(target_os = "windows")]
    pub fn resource(&self) -> &ID3D12Resource {
        &self.resource
    }
    
    pub fn handle(&self) -> ResourceHandle {
        self.handle
    }
    
    pub fn description(&self) -> &TextureDescription {
        &self.description
    }
    
    pub fn width(&self) -> u32 {
        self.width
    }
    
    pub fn height(&self) -> u32 {
        self.height
    }
    
    pub fn depth_or_array_layers(&self) -> u32 {
        self.depth_or_array_layers
    }
    
    pub fn mip_levels(&self) -> u32 {
        self.mip_levels
    }
}

/// DX12 Texture View (SRV/RTV/DSV/UAV)
pub struct Dx12TextureView {
    handle: ResourceHandle,
    texture_handle: ResourceHandle,
    view_type: TextureViewType,
    
    #[cfg(target_os = "windows")]
    descriptor_handle: u64, // CPU descriptor handle
}

unsafe impl Send for Dx12TextureView {}
unsafe impl Sync for Dx12TextureView {}

impl Dx12TextureView {
    #[cfg(target_os = "windows")]
    pub fn new(
        device: &ID3D12Device,
        texture: &Dx12Texture,
        desc: &TextureViewDescription,
        handle: ResourceHandle,
    ) -> RhiResult<Self> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        let view_type = desc.view_type;
        
        // Create descriptor based on view type
        let descriptor_handle = match view_type {
            TextureViewType::ShaderResource => {
                // Create SRV descriptor
                let srv_desc = D3D12_SHADER_RESOURCE_VIEW_DESC {
                    Format: Dx12Texture::to_dxgi_format(desc.format),
                    ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
                    Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
                    Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                        Texture2D: D3D12_TEX2D_SRV {
                            MostDetailedMip: 0,
                            MipLevels: texture.mip_levels(),
                            PlaneSlice: 0,
                            ResourceMinLODClamp: 0.0,
                        },
                    },
                };
                
                // Get descriptor heap and create SRV
                device.CreateShaderResourceView(
                    &texture.resource,
                    Some(&srv_desc),
                    std::mem::zeroed(), // Will be set by descriptor heap
                );
                
                0 // Placeholder - actual handle from descriptor heap
            }
            TextureViewType::RenderTarget => {
                // Create RTV descriptor
                let rtv_desc = D3D12_RENDER_TARGET_VIEW_DESC {
                    Format: Dx12Texture::to_dxgi_format(desc.format),
                    ViewDimension: D3D12_RTV_DIMENSION_TEXTURE2D,
                    Anonymous: D3D12_RENDER_TARGET_VIEW_DESC_0 {
                        Texture2D: D3D12_TEX2D_RTV {
                            MipSlice: 0,
                            PlaneSlice: 0,
                        },
                    },
                };
                
                device.CreateRenderTargetView(
                    &texture.resource,
                    Some(&rtv_desc),
                    std::mem::zeroed(), // Will be set by descriptor heap
                );
                
                0 // Placeholder - actual handle from descriptor heap
            }
            TextureViewType::DepthStencil => {
                // Create DSV descriptor
                let dsv_desc = D3D12_DEPTH_STENCIL_VIEW_DESC {
                    Format: Dx12Texture::to_dxgi_format(desc.format),
                    ViewDimension: D3D12_DSV_DIMENSION_TEXTURE2D,
                    Flags: D3D12_DSV_FLAG_NONE,
                    Anonymous: D3D12_DEPTH_STENCIL_VIEW_DESC_0 {
                        Texture2D: D3D12_TEX2D_DSV {
                            MipSlice: 0,
                        },
                    },
                };
                
                device.CreateDepthStencilView(
                    &texture.resource,
                    Some(&dsv_desc),
                    std::mem::zeroed(), // Will be set by descriptor heap
                );
                
                0 // Placeholder - actual handle from descriptor heap
            }
            TextureViewType::UnorderedAccess => {
                // Create UAV descriptor
                let uav_desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
                    Format: Dx12Texture::to_dxgi_format(desc.format),
                    ViewDimension: D3D12_UAV_DIMENSION_TEXTURE2D,
                    Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                        Texture2D: D3D12_TEX2D_UAV {
                            MipSlice: 0,
                            PlaneSlice: 0,
                        },
                    },
                };
                
                device.CreateUnorderedAccessView(
                    &texture.resource,
                    None, // No counter resource
                    Some(&uav_desc),
                    std::mem::zeroed(), // Will be set by descriptor heap
                );
                
                0 // Placeholder - actual handle from descriptor heap
            }
        };
        
        Ok(Self {
            handle,
            texture_handle: texture.handle(),
            view_type,
            descriptor_handle,
        })
    }
    
    pub fn handle(&self) -> ResourceHandle {
        self.handle
    }
    
    pub fn texture_handle(&self) -> ResourceHandle {
        self.texture_handle
    }
    
    pub fn view_type(&self) -> TextureViewType {
        self.view_type
    }
}
