use crate::graphics::rhi::{IDevice, BufferDesc, BufferType, BufferUsage, ResourceHandle, RhiError};
use crate::world::chunk::TerrainVertex;
use bytemuck;

/// Простая вершина для тестового рендеринга
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct SimpleVertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
}

unsafe impl bytemuck::NoUninit for SimpleVertex {}

/// Меш с вершинным и опционально индексным буфером
pub struct Mesh {
    pub vertex_buffer: ResourceHandle,
    pub index_buffer: Option<ResourceHandle>,
    pub vertex_count: u32,
    pub index_count: u32,
}

impl Mesh {
    /// Создаёт тестовый треугольник
    pub fn new_triangle(device: &dyn IDevice) -> Result<Self, RhiError> {
        let vertices = [
            SimpleVertex { pos: [-0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0] }, // красный
            SimpleVertex { pos: [ 0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] }, // зелёный
            SimpleVertex { pos: [ 0.0,  0.5, 0.0], color: [0.0, 0.0, 1.0] }, // синий
        ];

        let vb_desc = BufferDesc {
            size: (std::mem::size_of::<SimpleVertex>() * vertices.len()) as u64,
            usage: BufferUsage::VERTEX_BUFFER,
            buffer_type: BufferType::Vertex,
            ..Default::default()
        };
        
        let vb = device.create_buffer(&vb_desc)?;
        device.update_buffer(vb, 0, bytemuck::cast_slice(&vertices))?;

        Ok(Self {
            vertex_buffer: vb,
            index_buffer: None,
            vertex_count: vertices.len() as u32,
            index_count: 0,
        })
    }

    /// Создаёт квадрат с UV-координатами (для текстур)
    pub fn new_quad(device: &dyn IDevice) -> Result<Self, RhiError> {
        #[repr(C)]
        #[derive(Copy, Clone, Debug)]
        struct TexturedVertex {
            pos: [f32; 3],
            uv: [f32; 2],
        }

        unsafe impl bytemuck::NoUninit for TexturedVertex {}

        let vertices = [
            TexturedVertex { pos: [-0.5, -0.5, 0.0], uv: [0.0, 1.0] },
            TexturedVertex { pos: [ 0.5, -0.5, 0.0], uv: [1.0, 1.0] },
            TexturedVertex { pos: [ 0.5,  0.5, 0.0], uv: [1.0, 0.0] },
            TexturedVertex { pos: [-0.5,  0.5, 0.0], uv: [0.0, 0.0] },
        ];

        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

        let vb_desc = BufferDesc {
            size: (std::mem::size_of::<TexturedVertex>() * vertices.len()) as u64,
            usage: BufferUsage::VERTEX_BUFFER,
            buffer_type: BufferType::Vertex,
            ..Default::default()
        };
        
        let vb = device.create_buffer(&vb_desc)?;
        device.update_buffer(vb, 0, bytemuck::cast_slice(&vertices))?;

        let ib_desc = BufferDesc {
            size: (std::mem::size_of::<u32>() * indices.len()) as u64,
            usage: BufferUsage::INDEX_BUFFER,
            buffer_type: BufferType::Index,
            ..Default::default()
        };
        
        let ib = device.create_buffer(&ib_desc)?;
        device.update_buffer(ib, 0, bytemuck::cast_slice(&indices))?;

        Ok(Self {
            vertex_buffer: vb,
            index_buffer: Some(ib),
            vertex_count: vertices.len() as u32,
            index_count: indices.len() as u32,
        })
    }

    /// Создаёт меш террейна из ChunkData
    pub fn new_terrain(device: &dyn IDevice, vertices: &[TerrainVertex], indices: &[u32]) -> Result<Self, RhiError> {
        if vertices.is_empty() || indices.is_empty() {
            return Err(RhiError::Other("Empty terrain mesh".to_string()));
        }

        let vb_desc = BufferDesc {
            size: (std::mem::size_of::<TerrainVertex>() * vertices.len()) as u64,
            usage: BufferUsage::VERTEX_BUFFER,
            buffer_type: BufferType::Vertex,
            ..Default::default()
        };
        
        let vb = device.create_buffer(&vb_desc)?;
        device.update_buffer(vb, 0, bytemuck::cast_slice(vertices))?;

        let ib_desc = BufferDesc {
            size: (std::mem::size_of::<u32>() * indices.len()) as u64,
            usage: BufferUsage::INDEX_BUFFER,
            buffer_type: BufferType::Index,
            ..Default::default()
        };
        
        let ib = device.create_buffer(&ib_desc)?;
        device.update_buffer(ib, 0, bytemuck::cast_slice(indices))?;

        Ok(Self {
            vertex_buffer: vb,
            index_buffer: Some(ib),
            vertex_count: vertices.len() as u32,
            index_count: indices.len() as u32,
        })
    }

    /// Создаёт процедурное небо (полноэкранный квад)
    pub fn new_skybox_quad(device: &dyn IDevice) -> Result<Self, RhiError> {
        // Квад на весь экран в NDC координатах (-1..1)
        #[repr(C)]
        #[derive(Copy, Clone, Debug)]
        struct SkyboxVertex {
            pos: [f32; 3],
        }

        unsafe impl bytemuck::NoUninit for SkyboxVertex {}

        let vertices = [
            SkyboxVertex { pos: [-1.0, -1.0, 1.0] },
            SkyboxVertex { pos: [ 1.0, -1.0, 1.0] },
            SkyboxVertex { pos: [ 1.0,  1.0, 1.0] },
            SkyboxVertex { pos: [-1.0,  1.0, 1.0] },
        ];

        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

        let vb_desc = BufferDesc {
            size: (std::mem::size_of::<SkyboxVertex>() * vertices.len()) as u64,
            usage: BufferUsage::VERTEX_BUFFER,
            buffer_type: BufferType::Vertex,
            ..Default::default()
        };
        
        let vb = device.create_buffer(&vb_desc)?;
        device.update_buffer(vb, 0, bytemuck::cast_slice(&vertices))?;

        let ib_desc = BufferDesc {
            size: (std::mem::size_of::<u32>() * indices.len()) as u64,
            usage: BufferUsage::INDEX_BUFFER,
            buffer_type: BufferType::Index,
            ..Default::default()
        };
        
        let ib = device.create_buffer(&ib_desc)?;
        device.update_buffer(ib, 0, bytemuck::cast_slice(&indices))?;

        Ok(Self {
            vertex_buffer: vb,
            index_buffer: Some(ib),
            vertex_count: vertices.len() as u32,
            index_count: indices.len() as u32,
        })
    }
}
