//! Render commands for GPU execution
//! Defines the command buffer structures that the renderer uses to draw scenes

use crate::graphics::rhi::types::*;

/// Render command for drawing 3D objects
#[derive(Debug, Clone)]
pub struct RenderCommand {
    pub pipeline_state: ResourceHandle,
    pub vertex_buffers: Vec<(ResourceHandle, u64)>, // (buffer, offset)
    pub index_buffer: Option<(ResourceHandle, u64, IndexFormat)>,
    pub instance_count: u32,
    pub vertex_count: u32,
    pub start_vertex: u32,
    pub start_instance: u32,
    pub depth_bias: f32,
    pub slope_scaled_depth_bias: f32,
}

impl RenderCommand {
    pub fn draw(
        pipeline_state: ResourceHandle,
        vertex_buffers: Vec<(ResourceHandle, u64)>,
        vertex_count: u32,
        instance_count: u32,
    ) -> Self {
        Self {
            pipeline_state,
            vertex_buffers,
            index_buffer: None,
            instance_count,
            vertex_count,
            start_vertex: 0,
            start_instance: 0,
            depth_bias: 0.0,
            slope_scaled_depth_bias: 0.0,
        }
    }

    pub fn draw_indexed(
        pipeline_state: ResourceHandle,
        vertex_buffers: Vec<(ResourceHandle, u64)>,
        index_buffer: (ResourceHandle, u64, IndexFormat),
        _index_count: u32,
        instance_count: u32,
    ) -> Self {
        Self {
            pipeline_state,
            vertex_buffers,
            index_buffer: Some(index_buffer),
            instance_count,
            vertex_count: 0, // Not used for indexed drawing
            start_vertex: 0,
            start_instance: 0,
            depth_bias: 0.0,
            slope_scaled_depth_bias: 0.0,
        }
    }
}

/// Depth values for UI layering
pub const UI_DEPTH_HUD: f32 = 0.9;
pub const UI_DEPTH_NOTIFICATIONS: f32 = 0.95;
pub const UI_DEPTH_PROMPT: f32 = 0.99;