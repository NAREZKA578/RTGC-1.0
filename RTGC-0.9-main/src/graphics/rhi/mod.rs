//! RHI (Render Hardware Interface) Module
//! Provides abstraction over different graphics APIs (Vulkan, DX12, DX11, OpenGL)

pub mod types;
pub mod device;
pub mod factory;
pub mod gl;
pub mod rhi_module;
pub mod resource_manager;

#[cfg(feature = "dx12")]
pub mod dx12;

#[cfg(feature = "dx11")]
pub mod dx11;

#[cfg(feature = "vulkan")]
pub mod vulkan;

pub use types::*;
pub use device::*;
pub use factory::*;
pub use rhi_module::*;
pub use resource_manager::*;
pub use gl::{GlDevice, GlCommandQueue, GlCommandList, GlSwapChainInternal, GlFence};

// Re-export types that are used via `crate::graphics::rhi::TypeName` path
pub use types::{
    ShaderStage, ShaderDescription, TextureDescription, TextureFormat, TextureType,
    PipelineStateObject, BufferDescription, BufferType, PrimitiveTopology, RasterizerState,
    DepthState, ColorBlendState, BlendOp, BlendMode, CompareFunc, CullMode, FrontFace,
    FillMode, StencilState, ResourceState, TextureUsage, BufferUsage, IndexType,
    StencilOp, StencilFaceState, DepthStencilState, TextureUsage as TexUsage,
};
pub use device::{LoadOp, StoreOp, RenderPassDescription, RenderAttachment, DepthStencilAttachment, IndexFormat};
