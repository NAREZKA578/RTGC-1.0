//! Render Queue for batching and sorting render commands
//! 
//! Этот модуль реализует сортировку команд для оптимизации рендеринга:
//! - Сортировка по материалу/PSO для минимизации переключений
//! - Front-to-back для opaque объектов (overdraw optimization)
//! - Back-to-front для transparent объектов

use crate::graphics::render_command::RenderCommand;
use crate::graphics::renderer::commands::UiCommand;
use nalgebra::Vector3;

/// Ключ сортировки для команды рендеринга
#[derive(Debug, Clone)]
struct RenderSortKey {
    /// ID материала для группировки по PSO
    material_id: u64,
    /// Дистанция до камеры (квадрат)
    distance_sq: f32,
    /// Тип рендеринга (opaque/transparent)
    is_transparent: bool,
    /// Индекс команды в оригинальном буфере
    command_index: usize,
}

/// Queue for render commands with proper sorting and batching
pub struct RenderQueue {
    pub commands: Vec<RenderCommand>,
    pub ui_commands: Vec<UiCommand>,
    /// Флаг необходимости пересортировки
    needs_sort: bool,
    /// Позиция камеры для сортировки по дистанции
    camera_position: Option<Vector3<f32>>,
}

impl RenderQueue {
    pub fn new() -> Self {
        Self {
            commands: Vec::with_capacity(256),
            ui_commands: Vec::with_capacity(128),
            needs_sort: false,
            camera_position: None,
        }
    }

    /// Устанавливает позицию камеры для сортировки
    pub fn set_camera_position(&mut self, pos: Vector3<f32>) {
        self.camera_position = Some(pos);
        self.needs_sort = true;
    }

    /// Добавляет команду в очередь
    pub fn add_command(&mut self, command: RenderCommand) {
        self.commands.push(command);
        self.needs_sort = true;
    }

    pub fn add_ui_command(&mut self, command: UiCommand) {
        self.ui_commands.push(command);
    }

    /// Очищает очередь
    pub fn clear(&mut self) {
        self.commands.clear();
        self.ui_commands.clear();
        self.needs_sort = false;
    }

    /// Сортирует команды для оптимального рендеринга
    /// Вызывается автоматически перед получением команд
    pub fn sort_commands(&mut self) {
        if !self.needs_sort || self.commands.is_empty() {
            return;
        }

        let camera_pos = self.camera_position.unwrap_or(Vector3::new(0.0, 0.0, 0.0));

        // Создаём ключи сортировки для каждой команды
        let mut sort_keys: Vec<RenderSortKey> = self.commands
            .iter()
            .enumerate()
            .map(|(index, cmd)| {
                let (material_id, distance_sq, is_transparent) = match cmd {
                    RenderCommand::Mesh { material, transform, .. } => {
                        let mesh_pos = Vector3::new(transform.m14, transform.m24, transform.m34);
                        let dist_sq = mesh_pos.distance_squared(&camera_pos);
                        (material.0, dist_sq, false)
                    }
                    RenderCommand::MeshInstanced { material, transforms, .. } => {
                        // Используем центр всех трансформаций
                        let center = transforms.iter()
                            .map(|t| Vector3::new(t.m14, t.m24, t.m34))
                            .sum::<Vector3<_>>() / transforms.len() as f32;
                        let dist_sq = center.distance_squared(&camera_pos);
                        (material.0, dist_sq, false)
                    }
                    RenderCommand::TerrainChunk { material, transform, .. } => {
                        let chunk_pos = Vector3::new(transform.m14, transform.m24, transform.m34);
                        let dist_sq = chunk_pos.distance_squared(&camera_pos);
                        (material.0, dist_sq, false)
                    }
                    RenderCommand::Skybox { .. } => {
                        // Скайбокс всегда рендерится первым
                        (u64::MAX, f32::MAX, false)
                    }
                    RenderCommand::Sun { .. } => {
                        // Солнце после скайбокса
                        (u64::MAX - 1, f32::MAX, false)
                    }
                    RenderCommand::LineList { .. } => {
                        // Линии прозрачные (для отладки)
                        (0, 0.0, true)
                    }
                    RenderCommand::MeshDeform { .. } => {
                        // Деформации обрабатываются отдельно
                        (0, 0.0, false)
                    }
                };

                RenderSortKey {
                    material_id,
                    distance_sq,
                    is_transparent,
                    command_index: index,
                }
            })
            .collect();

        // Сортируем ключи:
        // 1. Сначала opaque, потом transparent
        // 2. Opaque: по материалу, затем front-to-back (меньшая дистанция primero)
        // 3. Transparent: по материалу, затем back-to-front (большая дистанция primero)
        sort_keys.sort_by(|a, b| {
            // Сначала разделяем opaque и transparent
            match (a.is_transparent, b.is_transparent) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => {
                    // Внутри одной группы сортируем по материалу
                    match a.material_id.cmp(&b.material_id) {
                        std::cmp::Ordering::Equal => {
                            // Если материал одинаковый, сортируем по дистанции
                            if a.is_transparent {
                                // Transparent: back-to-front (дальше -> ближе)
                                b.distance_sq.partial_cmp(&a.distance_sq).unwrap_or(std::cmp::Ordering::Equal)
                            } else {
                                // Opaque: front-to-back (ближе -> дальше)
                                a.distance_sq.partial_cmp(&b.distance_sq).unwrap_or(std::cmp::Ordering::Equal)
                            }
                        }
                        other => other,
                    }
                }
            }
        });

        // Перестраиваем массив команд согласно отсортированным ключам
        let sorted_commands: Vec<RenderCommand> = sort_keys
            .iter()
            .map(|key| self.commands[key.command_index].clone())
            .collect();

        self.commands = sorted_commands;
        self.needs_sort = false;
    }

    /// Получает отсортированные команды
    pub fn get_commands(&mut self) -> &[RenderCommand] {
        self.sort_commands();
        &self.commands
    }

    pub fn get_ui_commands(&self) -> &[UiCommand] {
        &self.ui_commands
    }

    /// Получает количество команд
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for RenderQueue {
    fn default() -> Self {
        Self::new()
    }
}