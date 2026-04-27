//! Главный модуль Renderer - координирует рендеринг сцены, UI и отладки
//! 
//! Использует RHI для абстракции над графическим бэкендом

use parking_lot::Mutex;
use crate::graphics::rhi::{IDevice, ICommandQueue, ISwapChain, ICommandList, ResourceHandle, CommandListType, CommandListGuard, make_command_list_guard};
use crate::graphics::renderer::{
    SceneRenderer, UIRenderer, DebugRenderer, PipelineCache,
    MainRenderPass, ShadowRenderPass, PostProcessRenderPass,
    RenderCommand, UiCommand,
};
use crate::graphics::camera::Camera;
use crate::graphics::terrain_renderer::TerrainRenderer;
use crate::graphics::sky_renderer::SkyRenderer;
use nalgebra::Vector3;
use std::sync::Arc;

/// Основной Renderer
pub struct Renderer {
    pub device: Arc<dyn IDevice>,
    pub command_queue: Arc<dyn ICommandQueue>,
    pub swap_chain: Arc<dyn ISwapChain>,
    
    // Под-рендереры
    pub scene_renderer: SceneRenderer,
    pub ui_renderer: UIRenderer,
    pub debug_renderer: DebugRenderer,
    pub terrain_renderer: TerrainRenderer,
    pub sky_renderer: SkyRenderer,
    
    // Кэш пайплайнов
    pipeline_cache: PipelineCache,
    
    // Render passes
    main_pass: Option<MainRenderPass>,
    shadow_pass: Option<ShadowRenderPass>,
    post_process_pass: Option<PostProcessRenderPass>,
    
    // Камера
    camera: Camera,
    
    // Размеры экрана
    width: u32,
    height: u32,
    
    // Состояние
    debug_mode: bool,
    vsync: bool,
}

impl Renderer {
    /// Создаёт новый рендерер
    pub fn new(device: Arc<dyn IDevice>, command_queue: Arc<dyn ICommandQueue>, swap_chain: Arc<dyn ISwapChain>) -> Result<Self, String> {
        let width = swap_chain.width();
        let height = swap_chain.height();
        
        // Initialize renderers
        let scene_renderer = SceneRenderer::new(device.clone());
        let ui_renderer = UIRenderer::new(device.clone());
        let debug_renderer = DebugRenderer::new_with_device(device.clone(), 10000);
        let terrain_renderer = TerrainRenderer::new(device.clone());
        let sky_renderer = SkyRenderer::new(device.clone());
        
        // Initialize pipeline cache
        let pipeline_cache = PipelineCache::new();
        
        // Initialize render passes - create passes after getting attachments from swapchain
        let main_pass = None;
        let shadow_pass = None;
        let post_process_pass = None;
        
        let mut renderer = Self {
            device,
            command_queue,
            swap_chain,
            scene_renderer,
            ui_renderer,
            debug_renderer,
            terrain_renderer,
            sky_renderer,
            pipeline_cache,
            main_pass,
            shadow_pass,
            post_process_pass,
            camera: Camera::new(
                Vector3::new(0.0, 5.0, -10.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                45.0,
                width as f32 / height as f32,
                0.1,
                1000.0,
            ),
            width,
            height,
            debug_mode: false,
            vsync: true,
        };
        
        // Initialize camera
        renderer.camera.set_position(Vector3::new(0.0, 5.0, -10.0));
        renderer.camera.set_target(Vector3::new(0.0, 0.0, 0.0));
        renderer.camera.set_up(Vector3::new(0.0, 1.0, 0.0));
        renderer.camera.set_perspective(45.0, width as f32 / height as f32, 0.1, 1000.0);
        
        Ok(renderer)
    }
    
    /// Get screen width
    pub fn get_width(&self) -> u32 {
        self.width
    }
    
    /// Get screen height
    pub fn get_height(&self) -> u32 {
        self.height
    }
    
    /// Render a single frame
    pub fn render(&mut self) -> Result<(), String> {
        self.render_frame()
    }
    
    /// Создаёт все render passes
    fn create_render_passes(&mut self) -> Result<(), String> {
        // Для начала создадим простой тестовый pass с очисткой экрана
        // В полной реализации здесь будут созданы framebuffer'ы для color/depth
        
        // Получаем backbuffer texture из swapchain для основного прохода
        let backbuffer_texture = self.swap_chain.get_back_buffer_texture();
        
        // Создаём depth texture (пока заглушка - в полной реализации создать через device.create_texture)
        let depth_texture = ResourceHandle::default();
        
        self.main_pass = Some(MainRenderPass::new(
            backbuffer_texture,
            depth_texture,
            self.width,
            self.height,
        ));
        
        Ok(())
    }
    
    /// Рисует текст (заглушка через UIRenderer)
    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        if !text.is_empty() {
            self.ui_renderer.add_text_simple(text, [x, y], size, color);
        }
    }
    
    /// Рисует прямоугольник (заглушка через UIRenderer)
    pub fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        self.ui_renderer.add_rect([x, y], [width, height], color, None);
    }
    
    /// Рисует рамку прямоугольника
    pub fn draw_rect_border(&mut self, x: f32, y: f32, width: f32, height: f32, border_width: f32, color: [f32; 4]) {
        let bw = border_width;
        // Top border
        self.ui_renderer.add_rect([x, y], [width, bw], color, None);
        // Bottom border  
        self.ui_renderer.add_rect([x, y + height - bw], [width, bw], color, None);
        // Left border
        self.ui_renderer.add_rect([x, y], [bw, height], color, None);
        // Right border
        self.ui_renderer.add_rect([x + width - bw, y], [bw, height], color, None);
    }
    
    /// Рисует линию
    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: [f32; 4]) {
        let vertices = vec![[x1, y1, 0.0], [x2, y2, 0.0]];
        let colors = vec![color, color];
        self.debug_renderer.add_lines_vec(vertices, colors);
    }

    /// Рисует линию (старый API с thickness)
    pub fn draw_line_old(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, _thickness: f32, color: [f32; 4]) {
        self.draw_line(x1, y1, x2, y2, color);
    }
    
    /// Рисует треугольник
    pub fn draw_triangle(&mut self, x: f32, y: f32, size: f32, color: [f32; 4]) {
        let h = size * 0.866;
        let vertices = vec![
            [x, y, 0.0],
            [x + size, y, 0.0],
            [x + size * 0.5, y + h, 0.0]
        ];
        let colors = vec![color, color, color];
        self.debug_renderer.add_lines_vec(vertices, colors);
    }

    /// Рисует треугольник (старый API с координатами вершин)
    pub fn draw_triangle_old(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32, color: [f32; 4]) {
        let vertices = vec![[x1, y1, 0.0], [x2, y2, 0.0], [x3, y3, 0.0]];
        let colors = vec![color, color, color];
        self.debug_renderer.add_lines_vec(vertices, colors);
    }
    
    /// Рисует круг
    pub fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: [f32; 4], segments: u32) {
        let mut vertices = Vec::new();
        let mut colors = Vec::new();
        let segs = segments.max(3) as usize;
        for i in 0..=segs {
            let angle = (i as f32 / segs as f32) * std::f32::consts::TAU;
            let vx = x + radius * angle.cos();
            let vy = y + radius * angle.sin();
            vertices.push([vx, vy, 0.0]);
            colors.push(color);
        }
        self.debug_renderer.add_lines_vec(vertices, colors);
    }
    
    /// Начинает кадр
    pub fn begin_frame(&mut self) -> Result<(), String> {
        // Очищаем накопленные команды в под-рендерерах
        self.debug_renderer.clear();
        self.ui_renderer.clear();
        self.scene_renderer.clear_commands();
        
        Ok(())
    }
    
    /// Обновляет террейн (добавляет/удаляет чанки)
    pub fn update_terrain(&mut self, chunk_id: crate::world::chunk::ChunkId, chunk_data: &crate::world::chunk::ChunkData) -> Result<(), String> {
        self.terrain_renderer.update_chunk(chunk_id, chunk_data)
    }
    
    /// Удаляет чанк из рендеринга
    pub fn remove_terrain_chunk(&mut self, chunk_id: crate::world::chunk::ChunkId) {
        self.terrain_renderer.remove_chunk(chunk_id);
    }
    
    /// Устанавливает время суток для неба
    pub fn set_time_of_day(&mut self, time: f32) {
        self.sky_renderer.set_time_of_day(time);
    }
    
    /// Обновляет освещение из DayNightCycle
    pub fn update_lighting_from_cycle(&mut self, cycle: &crate::world::day_night_cycle::DayNightCycle) {
        // Получаем направление солнца
        let sun_dir = cycle.get_sun_direction();
        
        // Получаем цвета неба
        let sky_top = cycle.get_sky_color_top();
        let sky_horizon = cycle.get_sky_color_horizon();
        
        // Получаем интенсивность
        let intensity = cycle.get_ambient_intensity();
        
        // Вычисляем цвет солнца на основе времени суток
        let sun_color = if cycle.is_daytime() {
            [1.0, 0.95, 0.8]
        } else {
            [0.1, 0.1, 0.15]
        };
        
        // Вычисляем ambient цвет
        let ambient = [
            sky_horizon.x * intensity * 0.3,
            sky_horizon.y * intensity * 0.3,
            sky_horizon.z * intensity * 0.3,
        ];
        
        // Передаём в SkyRenderer
        self.sky_renderer.set_sun_direction(sun_dir);
        
        // Передаём в SceneRenderer
        self.scene_renderer.set_sun_direction(sun_dir);
        self.scene_renderer.set_sun_params(sun_color, ambient);
    }
    
    /// Получает направление солнца
    pub fn sun_direction(&self) -> nalgebra::Vector3<f32> {
        self.sky_renderer.sun_direction()
    }
    
    /// Рендер кадра (основной метод)
    pub fn render_frame(&mut self) -> Result<(), String> {
        use parking_lot::Mutex;
        
use crate::graphics::rhi::make_command_list_guard;

        // Создаём command list для текущего кадра
        let cmd_list = self.device.create_command_list(crate::graphics::rhi::CommandListType::Direct)
            .map_err(|e| format!("Failed to create command list: {:?}", e))?;
        
        let cmd_guard = make_command_list_guard(cmd_list);
        
        if let Some(ref main_pass) = self.main_pass {
            let mut cmd = cmd_guard.lock();
            cmd.begin_render_pass(&main_pass.description());
            cmd.end_render_pass();
        }
        
        // Завершаем command list
        {
            let mut cmd = cmd_guard.lock();
            cmd.close();
        }
        
        // Отправляем на выполнение
        let guard = cmd_guard.lock();
        let raw_ref: &dyn ICommandList = &**guard;
        self.command_queue.submit(&[raw_ref], &[], &[])
            .map_err(|e| format!("Failed to submit command list: {:?}", e))?;
        
        // Present
        self.end_frame()
    }
    
    /// Рендер кадра с поддержкой состояний движка (меню, загрузка, игра)
    pub fn render_frame_with_state(
        &mut self,
        game_state: &crate::engine::state::EngineState,
        main_menu: &crate::game::MainMenu,
    ) -> Result<(), String> {
use crate::graphics::rhi::make_command_list_guard;

// Создаём command list для текущего кадра
        let cmd_list = self.device.create_command_list(crate::graphics::rhi::CommandListType::Direct)
            .map_err(|e| format!("Failed to create command list: {:?}", e))?;
        
        let cmd_guard = make_command_list_guard(cmd_list);
        
        // Начинаем render pass с очисткой экрана
        if let Some(ref main_pass) = self.main_pass {
            {
                let mut cmd = cmd_guard.lock();
                cmd.begin_render_pass(&main_pass.description());
            }
            
            match game_state {
                crate::engine::state::EngineState::MainMenu { .. } => {
                    // Рендеринг главного меню через UI команды
                    let window_size = [self.width as f32, self.height as f32];
                    let mut ui_commands = Vec::new();
                    main_menu.render(&mut ui_commands, window_size);
                    self.render_ui(&ui_commands, &cmd_guard)?;
                }
                crate::engine::state::EngineState::Loading { progress, resource_type } => {
                    // Рендеринг экрана загрузки
                    let message = format!("Loading {:?}...", resource_type);
                    self.ui_renderer.render_loading_screen(*progress, &message, &cmd_guard)?;
                }
                crate::engine::state::EngineState::Playing { .. } |
                crate::engine::state::EngineState::Paused { .. } => {
                    // Рендеринг 3D сцены
                    self.render_3d_scene(&cmd_guard)?;
                    
                    // Если пауза, добавляем полупрозрачный оверлей
                    if matches!(game_state, crate::engine::state::EngineState::Paused { .. }) {
                        self.ui_renderer.render(&[UiCommand::Rect {
                            position: [0.0, 0.0],
                            size: [self.width as f32, self.height as f32],
                            color: [0.0, 0.0, 0.0, 0.5],
                        }], &cmd_guard)?;
                    }
                }
                _ => {
                    // Другие состояния (ошибка, инициализация) - просто очищаем экран
                }
            }
            
            {
                let mut cmd = cmd_guard.lock();
                cmd.end_render_pass();
            }
        }
        
        // Завершаем command list и отправляем на выполнение
        {
            let mut cmd = cmd_guard.lock();
            cmd.close();
        }
        let guard = cmd_guard.lock();
        let raw_ref: &dyn ICommandList = &**guard;
        self.command_queue.submit(&[raw_ref], &[], &[])
            .map_err(|e| format!("Failed to submit command list: {:?}", e))?;
        
        // Present
        self.end_frame()
    }
    
    /// Рендерит 3D сцену (террейн, небо, объекты)
    fn render_3d_scene(&mut self, cmd_list: &CommandListGuard) -> Result<(), String> {
        use crate::graphics::renderer::scene::SceneRenderer;
        
        // Вычисляем плоскости фрустума
        let view_proj = self.camera.view_proj_matrix();
        let frustum_planes = SceneRenderer::compute_frustum_planes(&view_proj);
        
        // Собираем команды от SkyRenderer
        let sky_commands = self.sky_renderer.collect_render_commands(self.camera.position);
        for cmd in sky_commands {
            self.scene_renderer.add_command(cmd);
        }
        
        // Собираем команды от TerrainRenderer
        let terrain_commands = self.terrain_renderer.collect_render_commands(
            self.camera.position,
            &frustum_planes
        );
        for cmd in terrain_commands {
            self.scene_renderer.add_command(cmd);
        }
        
        // Собираем команды от других объектов (пропсы, здания, транспорт)
        // Здесь должна быть логика добавления команд от world objects
        
        // Рендерим все команды через SceneRenderer
        let all_commands = std::mem::take(&mut self.scene_renderer.command_buffer);
        self.scene_renderer.render(&self.camera, &all_commands, cmd_list)?;
        
        Ok(())
    }
    
    /// Рендерит сцену
    pub fn render_scene(&mut self, commands: &[RenderCommand], cmd_list: &CommandListGuard) -> Result<(), String> {
        self.scene_renderer.render(&self.camera, commands, cmd_list)?;
        Ok(())
    }
    
    /// Рендерит UI
    pub fn render_ui(&mut self, commands: &[UiCommand], cmd_list: &CommandListGuard) -> Result<(), String> {
        self.ui_renderer.render(commands, cmd_list)?;
        Ok(())
    }
    
    /// Рендерит отладочную информацию
    pub fn render_debug(&mut self) -> Result<(), String> {
        // Debug rendering будет вызван внутри render_scene или отдельно
        Ok(())
    }
    
    /// Завершает кадр
    pub fn end_frame(&mut self) -> Result<(), String> {
        // Present через swap chain
        self.swap_chain.present()
            .map_err(|e| format!("Failed to present: {:?}", e))?;
        
        Ok(())
    }
    
    /// Создаёт command list для записи GPU команд
    pub fn create_command_list(&mut self, cmd_type: CommandListType) -> Result<CommandListGuard, String> {
        self.device.create_command_list(cmd_type)
            .map(|cmd| make_command_list_guard(cmd))
            .map_err(|e| format!("Failed to create command list: {:?}", e))
    }
    
    /// Обрабатывает изменение размера окна
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        
        // Обновляем камеру
        self.camera.update_aspect(self.width, self.height);
        
        // Обновляем UI орто-матрицу
        self.ui_renderer.update_ortho_matrix(self.width, self.height);
        
        // Пересоздаём swap chain и render passes при необходимости
        self.swap_chain.resize(self.width, self.height)
            .unwrap_or_else(|e| tracing::warn!("Failed to resize swap chain: {:?}", e));
    }
    
    /// Устанавливает камеру
    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }
    
    /// Получает ссылку на камеру
    pub fn camera(&self) -> &Camera {
        &self.camera
    }
    
    /// Получает мутабельную ссылку на камеру
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
    
    /// Устанавливает режим отладки
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
    }
    
    /// Проверяет режим отладки
    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode
    }
    
    /// Получает ширину экрана
    pub fn width(&self) -> u32 {
        self.width
    }
    
    /// Получает высоту экрана
    pub fn height(&self) -> u32 {
        self.height
    }
    
    /// Получает статистику pipeline cache
    pub fn pipeline_cache_stats(&self) -> crate::graphics::renderer::PipelineCacheStats {
        self.pipeline_cache.stats()
    }
    
    /// Получает статистику SceneRenderer
    pub fn scene_renderer_stats(&self) -> crate::graphics::renderer::SceneRendererStats {
        self.scene_renderer.get_stats()
    }
    
    /// Получает количество отрендеренных чанков
    pub fn rendered_chunk_count(&self) -> usize {
        self.terrain_renderer.rendered_chunk_count()
    }
}
