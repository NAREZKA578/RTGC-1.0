use crate::physics::{PhysicsWorld, Ray, SpringConstraint};
use nalgebra::{Point3, Vector3};

/// Лебёдка — устройство для подтягивания объектов тросом
pub struct Winch {
    pub vehicle_body_index: usize,
    pub target_point: Option<Vector3<f32>>, // Точка куда выстрелили
    pub constraint_id: Option<usize>,       // Индекс constraint в physics world
    pub max_length: f32,                    // Максимальная длина троса
    pub current_length: f32,                // Текущая длина
    pub retract_speed: f32,                 // Скорость подмотки (м/с)
    pub is_active: bool,
    pub is_retracting: bool,
}

impl Winch {
    pub fn new(vehicle_body_index: usize) -> Self {
        Self {
            vehicle_body_index,
            target_point: None,
            constraint_id: None,
            max_length: 50.0, // 50 метров трос
            current_length: 0.0,
            retract_speed: 2.0, // 2 м/с подмотка
            is_active: false,
            is_retracting: false,
        }
    }

    /// Выстрелить тросом в направлении - использует physics.raycast()
    pub fn shoot(
        &mut self,
        origin: Vector3<f32>,
        direction: Vector3<f32>,
        physics_world: &PhysicsWorld,
    ) -> bool {
        // Perform raycast to find hit point
        let ray = Ray::new(Point3::from(origin), direction);

        if let Some(hit) = physics_world.raycast(&ray) {
            // Hit something - attach cable
            self.target_point = Some(nalgebra::Vector3::new(
                hit.point.x,
                hit.point.y,
                hit.point.z,
            ));
            self.current_length = (hit.point - origin).coords.norm();
            self.is_active = true;
            return true;
        }

        // No hit - extend to max length
        self.target_point = Some(origin + direction.normalize() * self.max_length);
        self.current_length = self.max_length;
        self.is_active = true;
        false
    }

    /// Начать подмотку
    pub fn start_retract(&mut self) {
        if self.is_active {
            self.is_retracting = true;
        }
    }

    /// Остановить подмотку
    pub fn stop_retract(&mut self) {
        self.is_retracting = false;
    }

    /// Обновить лебёдку (физика подмотки)
    pub fn update(
        &mut self,
        dt: f32,
        _physics_world: &mut PhysicsWorld,
        _constraints: &mut Vec<SpringConstraint>,
    ) {
        if !self.is_active {
            return;
        }

        if self.is_retracting && self.current_length > 0.5 {
            // Подматываем трос
            let delta = self.retract_speed * dt;
            self.current_length -= delta;

            if self.current_length <= 0.5 {
                // Полностью смотано
                self.current_length = 0.5;
                self.is_retracting = false;
            }
        }

        // В реальной реализации здесь нужно обновлять constraint:
        // if let Some(cid) = self.constraint_id {
        //     constraints[cid].rest_length = self.current_length;
        // }
    }

    /// Обновить лебёдку (physics_update alias для совместимости)
    pub fn physics_update(
        &mut self,
        dt: f32,
        _cargo: Option<usize>,
        physics_world: &mut PhysicsWorld,
    ) {
        let mut constraints = std::mem::take(&mut physics_world.spring_constraints);
        self.update(dt, physics_world, &mut constraints);
        physics_world.spring_constraints = constraints;
    }

    /// Отцепить трос
    pub fn release(&mut self, _constraints: &mut Vec<SpringConstraint>) {
        self.is_active = false;
        self.is_retracting = false;
        self.target_point = None;
        self.constraint_id = None;
        self.current_length = 0.0;
    }

    /// Получить натяжение троса (для UI)
    pub fn get_tension(&self) -> f32 {
        if !self.is_active {
            return 0.0;
        }

        // Чем короче трос относительно максимума, тем больше натяжение
        let ratio = self.current_length / self.max_length;
        1.0 - ratio
    }

    /// Статус троса для отображения
    pub fn get_status(&self) -> &'static str {
        if !self.is_active {
            "READY"
        } else if self.is_retracting {
            "RETRACTING"
        } else if self.current_length < 1.0 {
            "TIGHT"
        } else {
            "LOOSE"
        }
    }

    /// Игра-6: Get winch active status for HUD
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Игра-6: Get current winch cable length for HUD
    pub fn current_length(&self) -> f32 {
        self.current_length
    }
}
