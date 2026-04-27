// DirectX 12 Backend - Descriptor Heap Implementation
// Implements descriptor heaps and tables for DX12

use crate::graphics::rhi::types::*;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Direct3D12::*,
};

/// DX12 Descriptor Heap
pub struct Dx12DescriptorHeap {
    #[cfg(target_os = "windows")]
    heap: ID3D12DescriptorHeap,
    
    handle: ResourceHandle,
    description: DescriptorHeapDescription,
    capacity: u32,
    descriptor_size: u32,
    
    #[cfg(target_os = "windows")]
    cpu_handle_start: D3D12_CPU_DESCRIPTOR_HANDLE,
    
    #[cfg(target_os = "windows")]
    gpu_handle_start: Option<D3D12_GPU_DESCRIPTOR_HANDLE>,
}

unsafe impl Send for Dx12DescriptorHeap {}
unsafe impl Sync for Dx12DescriptorHeap {}

impl Dx12DescriptorHeap {
    /// Create a new DX12 descriptor heap
    #[cfg(target_os = "windows")]
    pub fn new(
        device: &ID3D12Device,
        desc: &DescriptorHeapDescription,
        handle: ResourceHandle,
    ) -> RhiResult<Self> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        let heap_type = Self::to_dx12_heap_type(desc.heap_type);
        
        let flags = if desc.flags.contains(DescriptorHeapFlags::SHADER_VISIBLE) {
            D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE
        } else {
            D3D12_DESCRIPTOR_HEAP_FLAG_NONE
        };
        
        let heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: heap_type,
            NumDescriptors: desc.capacity,
            Flags: flags,
            NodeMask: 0,
        };
        
        let heap: ID3D12DescriptorHeap = unsafe {
            device.CreateDescriptorHeap(&heap_desc)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create descriptor heap: {:?}", e)))?
        };
        
        let descriptor_size = unsafe {
            device.GetDescriptorHandleIncrementSize(heap_type)
        };
        
        let cpu_handle_start = unsafe { heap.GetCPUDescriptorHandleForHeapStart() };
        
        let gpu_handle_start = if desc.flags.contains(DescriptorHeapFlags::SHADER_VISIBLE) {
            Some(unsafe { heap.GetGPUDescriptorHandleForHeapStart() })
        } else {
            None
        };
        
        Ok(Self {
            heap,
            handle,
            description: desc.clone(),
            capacity: desc.capacity,
            descriptor_size,
            cpu_handle_start,
            gpu_handle_start,
        })
    }
    
    #[cfg(target_os = "windows")]
    fn to_dx12_heap_type(heap_type: DescriptorHeapType) -> D3D12_DESCRIPTOR_HEAP_TYPE {
        match heap_type {
            DescriptorHeapType::ConstantBufferView => D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            DescriptorHeapType::ShaderResourceView => D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            DescriptorHeapType::UnorderedAccessView => D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            DescriptorHeapType::Sampler => D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER,
            DescriptorHeapType::RenderTargetView => D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            DescriptorHeapType::DepthStencilView => D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    pub fn new(
        _device: &ID3D12Device,
        _desc: &DescriptorHeapDescription,
        _handle: ResourceHandle,
    ) -> RhiResult<Self> {
        Err(RhiError::Unsupported("DirectX 12 is only available on Windows".to_string()))
    }
    
    /// Get CPU descriptor handle at specified index
    #[cfg(target_os = "windows")]
    pub fn get_cpu_handle(&self, index: u32) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        unsafe {
            D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: self.cpu_handle_start.ptr + (index as usize * self.descriptor_size as usize),
            }
        }
    }
    
    /// Get GPU descriptor handle at specified index
    #[cfg(target_os = "windows")]
    pub fn get_gpu_handle(&self, index: u32) -> Option<D3D12_GPU_DESCRIPTOR_HANDLE> {
        self.gpu_handle_start.map(|start| unsafe {
            D3D12_GPU_DESCRIPTOR_HANDLE {
                ptr: start.ptr + (index as usize * self.descriptor_size as usize),
            }
        })
    }
    
    #[cfg(target_os = "windows")]
    pub fn heap(&self) -> &ID3D12DescriptorHeap {
        &self.heap
    }
    
    pub fn handle(&self) -> ResourceHandle {
        self.handle
    }
    
    pub fn description(&self) -> &DescriptorHeapDescription {
        &self.description
    }
    
    pub fn capacity(&self) -> u32 {
        self.capacity
    }
    
    pub fn descriptor_size(&self) -> u32 {
        self.descriptor_size
    }
}

/// DX12 Descriptor Table - manages allocations within a heap
pub struct Dx12DescriptorTable {
    heap_handle: ResourceHandle,
    start_index: u32,
    count: u32,
}

unsafe impl Send for Dx12DescriptorTable {}
unsafe impl Sync for Dx12DescriptorTable {}

impl Dx12DescriptorTable {
    pub fn new(heap_handle: ResourceHandle, start_index: u32, count: u32) -> Self {
        Self {
            heap_handle,
            start_index,
            count,
        }
    }
    
    pub fn heap_handle(&self) -> ResourceHandle {
        self.heap_handle
    }
    
    pub fn start_index(&self) -> u32 {
        self.start_index
    }
    
    pub fn count(&self) -> u32 {
        self.count
    }
}

/// DX12 Descriptor Allocator - allocates descriptors from heaps
pub struct Dx12DescriptorAllocator {
    heap_type: DescriptorHeapType,
    heaps: Vec<Dx12DescriptorHeap>,
    current_heap_index: usize,
    current_free_index: u32,
    descriptors_per_heap: u32,
}

unsafe impl Send for Dx12DescriptorAllocator {}
unsafe impl Sync for Dx12DescriptorAllocator {}

impl Dx12DescriptorAllocator {
    #[cfg(target_os = "windows")]
    pub fn new(device: &ID3D12Device, heap_type: DescriptorHeapType, descriptors_per_heap: u32) -> RhiResult<Self> {
        let desc = DescriptorHeapDescription {
            heap_type,
            capacity: descriptors_per_heap,
            flags: if heap_type == DescriptorHeapType::Sampler || heap_type == DescriptorHeapType::ConstantBufferView {
                DescriptorHeapFlags::SHADER_VISIBLE
            } else {
                DescriptorHeapFlags::empty()
            },
        };
        
        static HANDLE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let handle = ResourceHandle(HANDLE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        
        let heap = Dx12DescriptorHeap::new(device, &desc, handle)?;
        
        Ok(Self {
            heap_type,
            heaps: vec![heap],
            current_heap_index: 0,
            current_free_index: 0,
            descriptors_per_heap,
        })
    }
    
    /// Allocate a descriptor from the heap
    #[cfg(target_os = "windows")]
    pub fn allocate(&mut self, device: &ID3D12Device, count: u32) -> RhiResult<Dx12DescriptorTable> {
        // Check if current heap has enough space
        if self.current_free_index + count > self.descriptors_per_heap {
            // Create new heap
            let desc = DescriptorHeapDescription {
                heap_type: self.heap_type,
                capacity: self.descriptors_per_heap,
                flags: if self.heap_type == DescriptorHeapType::Sampler || self.heap_type == DescriptorHeapType::ConstantBufferView {
                    DescriptorHeapFlags::SHADER_VISIBLE
                } else {
                    DescriptorHeapFlags::empty()
                },
            };
            
            static HANDLE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let handle = ResourceHandle(HANDLE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
            
            let heap = Dx12DescriptorHeap::new(device, &desc, handle)?;
            self.heaps.push(heap);
            self.current_heap_index = self.heaps.len() - 1;
            self.current_free_index = 0;
        }
        
        let table = Dx12DescriptorTable::new(
            self.heaps[self.current_heap_index].handle(),
            self.current_free_index,
            count,
        );
        
        self.current_free_index += count;
        
        Ok(table)
    }
    
    pub fn reset(&mut self) {
        self.current_heap_index = 0;
        self.current_free_index = 0;
    }
}
