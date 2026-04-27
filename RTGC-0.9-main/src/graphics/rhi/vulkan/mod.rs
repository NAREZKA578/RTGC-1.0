//! Vulkan Backend Module
//! Complete Vulkan RHI implementation

pub mod device_vk;
pub mod buffer_vk;
pub mod texture_vk;
pub mod swapchain_vk;
pub mod command_vk;
pub mod pipeline_vk;
pub mod shader_vk;
pub mod descriptor_vk;
pub mod fence_vk;

pub use device_vk::create_vulkan_device;
pub use device_vk::VkDevice;
pub use buffer_vk::VkBuffer;
pub use texture_vk::VkTexture;
pub use swapchain_vk::VkSwapChain;
pub use command_vk::VkCommandList;
pub use pipeline_vk::VkPipelineState;
pub use shader_vk::VkShader;
pub use descriptor_vk::VkDescriptorSet;
pub use fence_vk::VkFence;
