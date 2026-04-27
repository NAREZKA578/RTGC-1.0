//! DirectX 12 Backend Module
//! Complete DX12 RHI implementation for Windows

pub mod device_dx12;
pub mod buffer_dx12;
pub mod texture_dx12;
pub mod swapchain_dx12;
pub mod command_dx12;
pub mod pipeline_dx12;
pub mod shader_dx12;
pub mod descriptor_heap_dx12;

pub use device_dx12::create_dx12_device;
// pub use buffer_dx12::BufferDx12;
// pub use texture_dx12::TextureDx12;
// pub use swapchain_dx12::SwapchainDx12;
// pub use command_dx12::CommandListDx12;
// pub use pipeline_dx12::PipelineDx12;
// pub use shader_dx12::ShaderDx12;
// pub use descriptor_heap_dx12::{DescriptorHeapDx12, DescriptorAllocationDx12};
