#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BufferUsage {
    Vertex,
    Index,
    Uniform,
    Storage,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BufferAccess {
    Static,
    Dynamic,
    Stream,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureFormat {
    Rgba8,
    R8,
    Depth24,
    Depth24Stencil8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendMode {
    None,
    Alpha,
    Additive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CullMode {
    None,
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndexFormat {
    U16,
    U32,
}

#[derive(Debug, Clone, Copy)]
pub struct BufferDesc {
    pub size: usize,
    pub usage: BufferUsage,
    pub access: BufferAccess,
}

#[derive(Debug, Clone, Copy)]
pub struct TextureDesc {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub mips: bool,
    pub samples: u32,
}

pub struct DeviceCaps {
    pub max_texture_size: u32,
    pub max_uniform_buffer_size: u32,
    pub supports_instancing: bool,
}

#[derive(Debug, Clone)]
pub enum UniformValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat4([[f32; 4]; 4]),
    Int(i32),
}
