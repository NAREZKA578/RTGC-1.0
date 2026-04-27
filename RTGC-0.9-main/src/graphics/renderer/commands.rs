//! Render Commands - определения команд рендеринга
//! 
//! Этот модуль содержит команды для SceneRenderer и UIRenderer

use crate::graphics::rhi::ResourceHandle;
use nalgebra::{Matrix4, Vector3};

/// Команды рендеринга сцены
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// Отрисовка меша с материалом
    Mesh {
        mesh: ResourceHandle,
        material: ResourceHandle,
        transform: Matrix4<f32>,
    },
    /// Отрисовка меша с инстансингом
    MeshInstanced {
        mesh: ResourceHandle,
        material: ResourceHandle,
        transforms: Vec<Matrix4<f32>>,
    },
    /// Отрисовка чанка террейна
    TerrainChunk {
        chunk_id: (i32, i32),
        mesh: ResourceHandle,
        material: ResourceHandle,
        transform: Matrix4<f32>,
        lod: u32,
    },
    /// Небо (скайбокс или процедурное)
    Skybox {
        texture: Option<ResourceHandle>,
        sun_direction: [f32; 3],
    },
    /// Солнце (визуальное представление)
    Sun {
        direction: [f32; 3],
        angular_radius: f32,
        color: [f32; 3],
    },
    /// Отрисовка линий (для отладки)
    LineList {
        vertices: Vec<[f32; 3]>,
        colors: Vec<[f32; 4]>,
    },
    /// Деформация меша (для повреждений транспорта)
    MeshDeform {
        mesh: ResourceHandle,
        deformations: Vec<(usize, Vector3<f32>)>,
    },
}

impl RenderCommand {
    /// Добавить смещение вершины (для деформаций)
    pub fn add_vertex_displacement(&mut self, vertex_idx: usize, offset: Vector3<f32>) {
        // Найдём или создадим вариант деформации
        match self {
            RenderCommand::MeshDeform { deformations, .. } => {
                deformations.push((vertex_idx, offset));
            }
            _ => {
                // Заменяем на деформацию
                *self = RenderCommand::MeshDeform {
                    mesh: ResourceHandle::default(),
                    deformations: vec![(vertex_idx, offset)],
                };
            }
        }
    }
}

/// Команды рендеринга UI
#[derive(Debug, Clone)]
pub enum UiCommand {
    /// Прямоугольник
    Rect {
        position: [f32; 2],
        size: [f32; 2],
        color: [f32; 4],
    },
    /// Прямоугольник с текстурой
    TexturedRect {
        position: [f32; 2],
        size: [f32; 2],
        texture: ResourceHandle,
        uv_rect: Option<[f32; 4]>, // [u0, v0, u1, v1]
        color: [f32; 4],
    },
    /// Текст
    Text {
        text: String,
        position: [f32; 2],
        font_size: f32,
        color: [f32; 4],
    },
    /// Спрайт
    Sprite {
        position: [f32; 2],
        size: [f32; 2],
        texture: ResourceHandle,
        color: [f32; 4],
        flip_x: bool,
        flip_y: bool,
    },
}

/// Конфигурация рендерера
#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub debug_mode: bool,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            vsync: true,
            debug_mode: false,
        }
    }
}
