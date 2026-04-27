//! Debug Renderer - отладочная визуализация

use crate::graphics::rhi::{
    IDevice, ResourceHandle, BufferDescription, BufferType, BufferUsage, 
    ResourceState, VertexFormat, VertexAttribute, InputLayout, 
    CommandListGuard, PrimitiveTopology,
};
use parking_lot::Mutex;
use std::sync::Arc;
use tracing;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DebugVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

unsafe impl bytemuck::NoUninit for DebugVertex {}

impl DebugVertex {
    pub fn new(position: [f32; 3], color: [f32; 4]) -> Self {
        Self { position, color }
    }
}

pub struct DebugRenderer {
    device: Option<Arc<dyn IDevice>>,
    pipeline: Option<ResourceHandle>,
    vertex_buffer: Option<ResourceHandle>,
    vertices: Vec<DebugVertex>,
    max_vertices: usize,
}

impl Clone for DebugRenderer {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(),
            pipeline: self.pipeline,
            vertex_buffer: self.vertex_buffer,
            vertices: Vec::new(),
            max_vertices: self.max_vertices,
        }
    }
}

impl DebugRenderer {
    pub fn new(max_vertices: usize) -> Self {
        Self {
            device: None,
            pipeline: None,
            vertex_buffer: None,
            vertices: Vec::new(),
            max_vertices,
        }
    }
    
    pub fn new_with_device(device: Arc<dyn IDevice>, max_vertices: usize) -> Self {
        Self {
            device: Some(device),
            pipeline: None,
            vertex_buffer: None,
            vertices: Vec::new(),
            max_vertices,
        }
    }
    
    pub fn create(&mut self) -> Result<(), String> {
        let device = self.device.as_ref().ok_or("No device")?;
        let attributes = vec![
            VertexAttribute { 
                name: "aPosition".to_string(), 
                format: VertexFormat::Float32x3, 
                offset: 0, 
                location: 0, 
                buffer_slot: 0, 
                semantic: String::new() 
            },
            VertexAttribute { 
                name: "aColor".to_string(), 
                format: VertexFormat::Float32x4, 
                offset: 12, 
                location: 1, 
                buffer_slot: 0, 
                semantic: String::new() 
            },
        ];
        
        let input_layout_desc = InputLayout { attributes, stride: 28 };
        
        device.create_input_layout(&input_layout_desc)
            .map_err(|e| format!("Failed to create input layout: {:?}", e))?;
        
        let vb_desc = BufferDescription {
            buffer_type: BufferType::Vertex,
            size: (self.max_vertices * std::mem::size_of::<DebugVertex>()) as u64,
            usage: BufferUsage::VERTEX_BUFFER,
            initial_state: ResourceState::VertexBuffer,
            initial_data: None,
        };
        
        self.vertex_buffer = Some(device.create_buffer(&vb_desc)
            .map_err(|e| format!("Failed to create vertex buffer: {:?}", e))?);
        
        Ok(())
    }
    
    pub fn line(&mut self, start: [f32; 3], end: [f32; 3], color: [f32; 4]) {
        if self.vertices.len() + 2 > self.max_vertices {
            tracing::warn!("DebugRenderer: vertex buffer full");
            return;
        }
        self.vertices.push(DebugVertex::new(start, color));
        self.vertices.push(DebugVertex::new(end, color));
    }
    
    pub fn add_lines_vec(&mut self, positions: Vec<[f32; 3]>, colors: Vec<[f32; 4]>) {
        if positions.is_empty() {
            return;
        }
        let count = positions.len();
        for i in 0..count {
            let start = positions[i];
            let end = if i + 1 < count { positions[i + 1] } else { positions[0] };
            let color = if i < colors.len() { colors[i] } else { [1.0, 1.0, 1.0, 1.0] };
            self.line(start, end, color);
        }
    }
    
    pub fn clear(&mut self) {
        self.vertices.clear();
    }
    
    pub fn render(&mut self, command_list: &CommandListGuard) -> Result<(), String> {
        if self.vertices.is_empty() {
            return Ok(());
        }
        
        if let (Some(buffer), Some(device)) = (self.vertex_buffer, self.device.as_ref()) {
            let vertex_data: &[u8] = bytemuck::cast_slice(&self.vertices);
            device.update_buffer(buffer, 0, vertex_data)
                .map_err(|e| format!("Failed to update debug buffer: {:?}", e))?;
        }
        
        if let Some(pipeline) = self.pipeline {
            let mut cmd = command_list.lock();
            cmd.set_pipeline_state(pipeline);
            
            if let Some(buffer) = self.vertex_buffer {
                cmd.bind_vertex_buffers(0, &[(buffer, 0)]);
            }
            
            let vertex_count = self.vertices.len() as u32;
            cmd.draw(vertex_count, 1, 0, 0);
        }
        
        self.vertices.clear();
        Ok(())
    }
}