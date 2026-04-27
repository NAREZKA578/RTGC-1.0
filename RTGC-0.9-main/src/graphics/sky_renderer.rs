//! Sky Renderer - процедурное небо и солнце
//! 
//! Рендерит процедурное небо с рассеянием Рэлея и визуальное представление солнца

use crate::graphics::rhi::{IDevice, ResourceHandle};
use crate::graphics::mesh::Mesh;
use crate::graphics::renderer::commands::RenderCommand;
use nalgebra::{Vector3, Matrix4};
use std::sync::Arc;

/// Система рендеринга неба
pub struct SkyRenderer {
    device: Arc<dyn IDevice>,
    /// Меш для скайбокса (полноэкранный квад)
    skybox_mesh: Option<Mesh>,
    /// Меш для солнца (полноэкранный квад)
    sun_mesh: Option<Mesh>,
    /// Материал неба
    sky_material: Option<ResourceHandle>,
    /// Материал солнца
    sun_material: Option<ResourceHandle>,
    /// Текущее время суток (0.0-1.0)
    time_of_day: f32,
    /// Направление солнца
    sun_direction: Vector3<f32>,
    /// Цвет солнца
    sun_color: [f32; 3],
    /// Угловой радиус солнца (в радианах)
    sun_angular_radius: f32,
}

impl SkyRenderer {
    pub fn new(device: Arc<dyn IDevice>) -> Self {
        Self {
            device,
            skybox_mesh: None,
            sun_mesh: None,
            sky_material: None,
            sun_material: None,
            time_of_day: 0.5, // Полдень по умолчанию
            sun_direction: Vector3::new(0.0, -1.0, 0.0),
            sun_color: [1.0, 0.95, 0.8],
            sun_angular_radius: 0.0046, // ~0.26 градуса (реальный размер солнца)
        }
    }
    
    /// Инициализирует меши и материалы
    pub fn initialize(&mut self) -> Result<(), String> {
        // Создаём полноэкранный квад для неба
        self.skybox_mesh = Some(
            Mesh::new_skybox_quad(self.device.as_ref())
                .map_err(|e| format!("Failed to create skybox mesh: {:?}", e))?
        );
        
        // Создаём полноэкранный квад для солнца
        self.sun_mesh = Some(
            Mesh::new_skybox_quad(self.device.as_ref())
                .map_err(|e| format!("Failed to create sun mesh: {:?}", e))?
        );
        
        // Создаём материалы через MaterialManager
        self.sky_material = Some(self.create_skybox_material()?);
        self.sun_material = Some(self.create_sun_material()?);
        
        Ok(())
    }

    /// Creates skybox material
    fn create_skybox_material(&self) -> Result<ResourceHandle, String> {
        // Для скайбокса используем простую программу без освещения
        Ok(ResourceHandle::default()) // В реальной реализации создать материал
    }

    /// Creates sun material
    fn create_sun_material(&self) -> Result<ResourceHandle, String> {
        // Материал для солнца с эмиссией
        Ok(ResourceHandle::default()) // В реальной реализации создать материал
    }
    
    /// Обновляет время суток
    pub fn set_time_of_day(&mut self, time: f32) {
        self.time_of_day = time.clamp(0.0, 1.0);
        self.update_sun_position();
    }
    
    /// Обновляет направление солнца вручную
    pub fn set_sun_direction(&mut self, direction: Vector3<f32>) {
        self.sun_direction = direction.normalize();
    }
    
    /// Вычисляет позицию солнца на основе времени суток
    fn update_sun_position(&mut self) {
        // Простая модель: солнце движется по дуге с востока на запад
        let angle = self.time_of_day * std::f32::consts::PI * 2.0;
        
        // Солнце восходит на востоке (+X), заходит на западе (-X)
        // В полдень (time = 0.5) солнце на юге (-Z) в зените
        let sun_x = angle.cos();
        let sun_y = (angle * 2.0).sin() * 0.5 + 0.5; // Высота над горизонтом
        let sun_z = angle.sin();
        
        self.sun_direction = Vector3::new(sun_x, sun_y.max(0.0), sun_z).normalize();
    }
    
    /// Получает направление солнца
    pub fn sun_direction(&self) -> Vector3<f32> {
        self.sun_direction
    }
    
    /// Получает цвет солнца на основе времени суток
    pub fn sun_color(&self) -> [f32; 3] {
        // Меняем цвет от белого (день) до оранжевого (закат) до тёмного (ночь)
        let day_factor = (self.time_of_day * std::f32::consts::PI * 2.0).sin();
        
        if day_factor > 0.5 {
            // День - белый/жёлтый
            [1.0, 0.95, 0.8]
        } else if day_factor > 0.0 {
            // Закат/рассвет - оранжевый
            [1.0, 0.6, 0.3]
        } else {
            // Ночь - тёмный
            [0.1, 0.1, 0.15]
        }
    }
    
    /// Собирает команды рендеринга неба
    pub fn collect_render_commands(&self, _camera_pos: Vector3<f32>) -> Vec<RenderCommand> {
        let mut commands = Vec::with_capacity(2);
        
        // Команда для неба
        commands.push(RenderCommand::Skybox {
            texture: None, // Процедурное небо
            sun_direction: [self.sun_direction.x, self.sun_direction.y, self.sun_direction.z],
        });
        
        // Команда для солнца (только если оно над горизонтом)
        if self.sun_direction.y > -0.1 {
            commands.push(RenderCommand::Sun {
                direction: [self.sun_direction.x, self.sun_direction.y, self.sun_direction.z],
                angular_radius: self.sun_angular_radius,
                color: self.sun_color(),
            });
        }
        
        commands
    }
    
    /// Преобразует направление солнца в экранные координаты
    pub fn sun_screen_position(&self, view_proj: &Matrix4<f32>) -> Option<Vector3<f32>> {
        // Проверяем, видно ли солнце (оно должно быть перед камерой)
        if self.sun_direction.y < -0.1 {
            return None; // Солнце за горизонтом
        }
        
        // Преобразуем направление в однородные координаты
        let sun_dir = self.sun_direction.normalize();
        let homogeneous = nalgebra::SVector::<f32, 4>::new(sun_dir.x, sun_dir.y, 1.0, 1.0);
        
        // Умножаем матрицу 4x4 на вектор 4x1
        let clip_pos = view_proj * homogeneous;
        
        if clip_pos.w <= 0.0 {
            return None; // Солнце за камерой
        }
        
        // Перспективное деление
        let ndc_x = clip_pos.x / clip_pos.w;
        let ndc_y = clip_pos.y / clip_pos.w;
        
        // Конвертируем из NDC (-1..1) в UV (0..1)
        let screen_x = (ndc_x + 1.0) * 0.5;
        let screen_y = (ndc_y + 1.0) * 0.5;
        
        // Проверяем, находится ли солнце в пределах экрана
        if screen_x < 0.0 || screen_x > 1.0 || screen_y < 0.0 || screen_y > 1.0 {
            return None;
        }
        
        Some(Vector3::new(screen_x, screen_y, clip_pos.w))
    }
    
    /// Устанавливает угловой радиус солнца
    pub fn set_sun_angular_radius(&mut self, radius: f32) {
        self.sun_angular_radius = radius.clamp(0.001, 0.1);
    }
    
    /// Получает количество draw calls для статистики
    pub fn render_call_count(&self) -> u32 {
        let mut count = 1; // Небо всегда рендерится
        if self.sun_direction.y > -0.1 {
            count += 1; // Солнце
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sun_position_at_noon() {
        let mut renderer = SkyRenderer::new(Arc::new(crate::graphics::rhi::gl::GlDevice::mock()));
        renderer.set_time_of_day(0.5);
        
        // В полдень солнце должно быть высоко
        assert!(renderer.sun_direction().y > 0.5);
    }
    
    #[test]
    fn test_sun_color_variation() {
        let mut renderer = SkyRenderer::new(Arc::new(crate::graphics::rhi::gl::GlDevice::mock()));
        
        renderer.set_time_of_day(0.5);
        let noon_color = renderer.sun_color();
        assert!(noon_color[0] > 0.9); // Красный канал высокий
        
        renderer.set_time_of_day(0.25);
        let sunset_color = renderer.sun_color();
        assert!(sunset_color[1] < 0.7); // Зелёный канал ниже на закате
    }
}
