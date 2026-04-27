//! Render Manager - управление рендерингом и графическим контекстом
//!
//! Этот модуль предоставляет RenderManager, который координирует работу
//! Renderer и GraphicsContext.

use crate::graphics::GraphicsContext;
use crate::graphics::renderer::{Renderer, RendererConfig};
use crate::graphics::material::MaterialManager;
use crate::graphics::particles::ParticleSystem;
use crate::graphics::renderer::DebugRenderer;
use crate::ui::HudManager;
use nalgebra::Vector3;
use tracing::info;

/// Менеджер рендеринга
pub struct RenderManager {
    /// Графический контекст
    graphics_context: GraphicsContext,
    
    /// Основной рендерер
    renderer: Option<Renderer>,
    
    /// Менеджер материалов
    material_manager: MaterialManager,
    
    /// Система частиц
    particle_system: ParticleSystem,
    
    /// Отладочный рендерер
    debug_renderer: DebugRenderer,
    
    /// HUD менеджер
    hud_manager: HudManager,
}

impl RenderManager {
    /// Создаёт новый RenderManager
    pub fn new(
        graphics_context: GraphicsContext,
        material_manager: MaterialManager,
        particle_system: ParticleSystem,
        debug_renderer: DebugRenderer,
        hud_manager: HudManager,
    ) -> Self {
        Self {
            graphics_context,
            renderer: None,
            material_manager,
            particle_system,
            debug_renderer,
            hud_manager,
        }
    }
    
    /// Инициализирует рендерер
    pub fn initialize_renderer(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!(target: "render_manager", "Initializing renderer...");
        
        match &mut self.graphics_context {
            GraphicsContext::OpenGL(gl_ctx) => {
                // Создаём свопчейн
                gl_ctx.create_swapchain(false)?;
                
                // Получаем устройства из GL контекста
                let device = gl_ctx.device();
                let command_queue = gl_ctx.command_queue();
                let swapchain = gl_ctx.swapchain()
                    .ok_or("Swapchain not created")?;
                
                // Создаём конфигурацию рендерера
                let (width, height) = gl_ctx.size();
                let config = RendererConfig {
                    width,
                    height,
                    debug_mode: true,
                    vsync: false,
                };
                
                // Создаём рендерер
                let renderer = Renderer::new(
                    device,
                    command_queue,
                    swapchain,
                )?;
                
                self.renderer = Some(renderer);
                
                info!(target: "render_manager", "OpenGL renderer initialized successfully");
            }
        }
        
        Ok(())
    }
    
    /// Возвращает графический контекст (забирает из RenderManager)
    /// DEPRECATED: Контекст теперь остаётся внутри RenderManager
    pub fn take_context(&mut self) -> GraphicsContext {
        panic!("take_context() is deprecated - context stays in RenderManager");
    }
    
    /// Получить ссылку на рендерер
    pub fn renderer(&self) -> Option<&Renderer> {
        self.renderer.as_ref()
    }
    
    /// Получить mutable ссылку на рендерер
    pub fn renderer_mut(&mut self) -> Option<&mut Renderer> {
        self.renderer.as_mut()
    }
    
    /// Получить ссылку на графический контекст
    pub fn graphics_context(&self) -> &GraphicsContext {
        &self.graphics_context
    }
    
    /// Получить mutable ссылку на графический контекст
    pub fn graphics_context_mut(&mut self) -> &mut GraphicsContext {
        &mut self.graphics_context
    }
    
    /// Обработка изменения размера окна
    pub fn on_resize(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        info!(target: "render_manager", "Resizing to {}x{}", width, height);
        
        match &mut self.graphics_context {
            GraphicsContext::OpenGL(gl_ctx) => {
                gl_ctx.on_resize(width, height)?;
                
                // Обновляем размеры в рендерере
                if let Some(ref mut renderer) = self.renderer {
                    renderer.resize(width, height);
                }
            }
        }
        
        Ok(())
    }
    
    /// Рендер кадра
    pub fn render_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut renderer) = self.renderer {
            renderer.render_frame()?;
            
            // Презент через графический контекст
            match &mut self.graphics_context {
                GraphicsContext::OpenGL(gl_ctx) => {
                    gl_ctx.present()?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Начало кадра
    pub fn begin_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut renderer) = self.renderer {
            renderer.begin_frame()?;
        }
        Ok(())
    }
    
    /// Конец кадра
    pub fn end_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut renderer) = self.renderer {
            renderer.end_frame()?;
        }
        Ok(())
    }
    
    /// Рендеринг сцены
    pub fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut renderer) = self.renderer {
            // Пока рендерим пустую сцену - в будущем здесь будет логика рендеринга игрового мира
            // Для тестирования можно добавить отладочные объекты
        }
        Ok(())
    }
    
    /// Обновление позиции мыши для UI
    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        // В будущем здесь будет обновление позиции мыши в UI системе
    }
    
    /// Установка режима отладки
    pub fn set_debug_mode(&mut self, enabled: bool) {
        if let Some(ref mut renderer) = self.renderer {
            renderer.set_debug_mode(enabled);
        }
    }
    
    /// Получить материал менеджер
    pub fn material_manager(&self) -> &MaterialManager {
        &self.material_manager
    }
    
    /// Получить систему частиц
    pub fn particle_system(&self) -> &ParticleSystem {
        &self.particle_system
    }
    
    /// Получить отладочный рендерер
    pub fn debug_renderer(&self) -> &DebugRenderer {
        &self.debug_renderer
    }
    
    /// Получить HUD менеджер
    pub fn hud_manager(&self) -> &HudManager {
        &self.hud_manager
    }

    /// Установить трансформацию транспортного средства
    pub fn set_vehicle_transform(&mut self, _position: Vector3<f32>, _rotation: nalgebra::Quaternion<f32>) {}

    /// Обновить камеру на основе позиции транспортного средства
    pub fn update_camera_from_vehicle(&mut self, _position: Vector3<f32>, _rotation: nalgebra::Quaternion<f32>) {}

    /// Установить цвета неба
    pub fn set_sky_colors(&mut self, _sky_bottom: Vector3<f32>, _sky_top: Vector3<f32>) {}

    /// Установить направление солнца
    pub fn set_sun_direction(&mut self, _direction: Vector3<f32>) {}
}
