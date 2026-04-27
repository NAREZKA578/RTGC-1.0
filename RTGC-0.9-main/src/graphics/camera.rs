//! Camera - единая камера для рендеринга
//! 
//! Предоставляет матрицы view, proj и view_proj

use nalgebra::{Matrix4, Vector3};

/// Камера для 3D рендеринга
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub target: Vector3<f32>,
    pub up: Vector3<f32>,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 2.0, 5.0),
            target: Vector3::zeros(),
            up: Vector3::y(),
            fov: 60.0_f32.to_radians(),
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl Camera {
    /// Создаёт новую камеру
    pub fn new(
        position: Vector3<f32>,
        target: Vector3<f32>,
        up: Vector3<f32>,
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            position,
            target,
            up,
            fov,
            aspect,
            near,
            far,
        }
    }
    
    /// Установить позицию камеры
    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
    }
    
    /// Установить цель (куда смотрит камера)
    pub fn set_target(&mut self, target: Vector3<f32>) {
        self.target = target;
    }
    
    /// Установить вектор "вверх"
    pub fn set_up(&mut self, up: Vector3<f32>) {
        self.up = up;
    }
    
    /// Установить перспективу
    pub fn set_perspective(&mut self, fov: f32, aspect: f32, near: f32, far: f32) {
        self.fov = fov;
        self.aspect = aspect;
        self.near = near;
        self.far = far;
    }
    
    /// Create a camera looking at target
    pub fn look_at(
        position: Vector3<f32>,
        target: Vector3<f32>,
        up: Vector3<f32>,
        fov_degrees: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            position,
            target,
            up,
            fov: fov_degrees.to_radians(),
            aspect,
            near,
            far,
        }
    }

    /// Матрица вида (view matrix)
    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(&self.position.into(), &self.target.into(), &self.up)
    }

    /// Матрица проекции (projection matrix)
    pub fn proj_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_perspective(self.aspect, self.fov, self.near, self.far)
    }

    /// Комбинированная матрица view * proj
    pub fn view_proj_matrix(&self) -> Matrix4<f32> {
        self.proj_matrix() * self.view_matrix()
    }

    /// Обновляет aspect ratio при изменении размера окна
    pub fn update_aspect(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    /// Двигает камеру вперёд/назад
    pub fn move_forward(&mut self, distance: f32) {
        let dir = (self.target - self.position).normalize();
        self.position += dir * distance;
        self.target += dir * distance;
    }

    /// Двигает камеру влево/вправо
    pub fn move_right(&mut self, distance: f32) {
        let dir = (self.target - self.position).normalize();
        let right = dir.cross(&self.up).normalize();
        self.position += right * distance;
        self.target += right * distance;
    }

    /// Вращает камеру вокруг цели
    pub fn rotate_around_target(&mut self, yaw: f32, pitch: f32) {
        use nalgebra::{Unit, Quaternion};
        
        let offset = self.position - self.target;
        
        // Применяем вращение через quaternion
        let q_yaw = Unit::new_normalize(Quaternion::new(0.0, 0.0, yaw.sin(), yaw.cos()));
        let q_pitch = Unit::new_normalize(Quaternion::new(pitch.sin(), 0.0, 0.0, pitch.cos()));
        
        let rotated = q_pitch * q_yaw * offset;
        self.position = self.target + rotated;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_default() {
        let camera = Camera::default();
        assert_eq!(camera.fov, 60.0_f32.to_radians());
        assert_eq!(camera.aspect, 16.0 / 9.0);
    }

    #[test]
    fn test_update_aspect() {
        let mut camera = Camera::default();
        camera.update_aspect(1920, 1080);
        assert!((camera.aspect - 16.0 / 9.0).abs() < 0.001);
        
        camera.update_aspect(800, 600);
        assert!((camera.aspect - 4.0 / 3.0).abs() < 0.001);
    }
}
