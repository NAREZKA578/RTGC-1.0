//! Crane Arm Physics for RTGC-0.9
//!
//! Реализация крановой техники:
//! - Автомобильные краны (Ивановец, Галичанин)
//! - Буровые установки
//! - Экскаваторы с крановой стрелой
//! - Физика тросов и грузов
//! - Ограничители грузоподъёмности

use nalgebra::{Matrix3, Point3, UnitQuaternion, Vector3};
use std::f32::consts::PI;

use super::physics_module::{Ray, RaycastHit, RigidBody};

/// Типы крановых установок
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CraneType {
    /// Автомобильный кран (Ивановец КС-4571)
TruckCrane25t,
    /// Тяжёлый автокран (Галичанин КС-6571)
    TruckCrane50t,
    /// Гусеничный кран
    CrawlerCrane,
    /// Буровая установка
    DrillingRig,
    /// Экскаватор с крановой стрелой
    ExcavatorCrane,
}

impl CraneType {
    /// Максимальная грузоподъёмность (кг)
    pub fn max_load_capacity(&self) -> f32 {
        match self {
            CraneType::TruckCrane25t => 25_000.0,
            CraneType::TruckCrane50t => 50_000.0,
            CraneType::CrawlerCrane => 100_000.0,
            CraneType::DrillingRig => 5_000.0,
            CraneType::ExcavatorCrane => 3_000.0,
        }
    }

    /// Максимальная длина стрелы (м)
    pub fn max_boom_length(&self) -> f32 {
        match self {
            CraneType::TruckCrane25t => 31.0,
            CraneType::TruckCrane50t => 45.0,
            CraneType::CrawlerCrane => 60.0,
            CraneType::DrillingRig => 20.0,
            CraneType::ExcavatorCrane => 12.0,
        }
    }

    /// Минимальная длина стрелы (м)
    pub fn min_boom_length(&self) -> f32 {
        match self {
            CraneType::TruckCrane25t => 10.0,
            CraneType::TruckCrane50t => 12.0,
            CraneType::CrawlerCrane => 15.0,
            CraneType::DrillingRig => 8.0,
            CraneType::ExcavatorCrane => 5.0,
        }
    }

    /// Скорость подъёма груза (м/с)
    pub fn hoist_speed(&self) -> f32 {
        match self {
            CraneType::TruckCrane25t => 0.5,
            CraneType::TruckCrane50t => 0.4,
            CraneType::CrawlerCrane => 0.3,
            CraneType::DrillingRig => 0.2,
            CraneType::ExcavatorCrane => 0.6,
        }
    }

    /// Скорость поворота башни (рад/с)
    pub fn slew_speed(&self) -> f32 {
        match self {
            CraneType::TruckCrane25t => 0.03,
            CraneType::TruckCrane50t => 0.025,
            CraneType::CrawlerCrane => 0.02,
            CraneType::DrillingRig => 0.04,
            CraneType::ExcavatorCrane => 0.05,
        }
    }

    /// Масса крана без груза (кг)
    pub fn empty_mass(&self) -> f32 {
        match self {
            CraneType::TruckCrane25t => 28_000.0,
            CraneType::TruckCrane50t => 45_000.0,
            CraneType::CrawlerCrane => 80_000.0,
            CraneType::DrillingRig => 15_000.0,
            CraneType::ExcavatorCrane => 12_000.0,
        }
    }
}

/// Управление краном
#[derive(Debug, Clone, Default)]
pub struct CraneControls {
    /// Подъём/опускание стрелы (-1.0 = опустить, 1.0 = поднять)
    pub boom_elevation: f32,
    /// Выдвижение/втягивание секций стрелы (-1.0 = втянуть, 1.0 = выдвинуть)
    pub boom_extension: f32,
    /// Поворот башни (-1.0 = влево, 1.0 = вправо)
    pub slew: f32,
    /// Подъём/опускание груза (-1.0 = опустить, 1.0 = поднять)
    pub hoist: f32,
    /// Аутригеры выставлены
    pub outriggers_deployed: bool,
}

/// Состояние стрелы
#[derive(Debug, Clone)]
pub struct BoomState {
    /// Угол возвышения (радианы, 0 = горизонтально)
    pub elevation_angle: f32,
    /// Текущая длина (м)
    pub length: f32,
    /// Угол поворота башни (радианы)
    pub slew_angle: f32,
    /// Длина троса (м)
    pub cable_length: f32,
}

impl Default for BoomState {
    fn default() -> Self {
        Self {
            elevation_angle: PI / 6.0, // 30 градусов
            length: 10.0,
            slew_angle: 0.0,
            cable_length: 5.0,
        }
    }
}

/// Конфигурация крана
#[derive(Debug, Clone)]
pub struct CraneConfig {
    pub crane_type: CraneType,
    /// Позиция основания крана
    pub base_position: Vector3<f32>,
    /// Максимальный угол возвышения стрелы
    pub max_elevation_angle: f32,
    /// Минимальный угол возвышения стрелы
    pub min_elevation_angle: f32,
}

impl Default for CraneConfig {
    fn default() -> Self {
        Self {
            crane_type: CraneType::TruckCrane25t,
            base_position: Vector3::zeros(),
            max_elevation_angle: PI / 2.0 - 0.1, // Почти вертикально
            min_elevation_angle: 0.05,           // Почти горизонтально
        }
    }
}

/// Груз на крюке
#[derive(Debug, Clone)]
pub struct SuspendedLoad {
    /// Масса груза (кг)
    pub mass: f32,
    /// Позиция груза (мировая)
    pub position: Vector3<f32>,
    /// Линейная скорость
    pub velocity: Vector3<f32>,
    /// Привязан ли груз
    pub attached: bool,
}

/// Компонент крановой стрелы
pub struct CraneArm {
    /// Конфигурация
    pub config: CraneConfig,
    /// Масса крана
    pub mass: f32,
    /// Ориентация базы
    pub base_orientation: UnitQuaternion<f32>,
    /// Состояние стрелы
    pub boom: BoomState,
    /// Управление
    pub controls: CraneControls,
    /// Подвешенный груз
    pub load: Option<SuspendedLoad>,
    /// Перегрузка (true если превышена грузоподъёмность)
    pub overloaded: bool,
    /// Давление на аутригеры (для стабильности)
    pub outrigger_pressure: [f32; 4], // [перед-лев, перед-прав, зад-лев, зад-прав]
    /// Температура гидравлики
    pub hydraulic_temperature: f32,
}

impl CraneArm {
    /// Создать новый кран
    pub fn new(crane_type: CraneType, base_position: Vector3<f32>) -> Self {
        let config = CraneConfig {
            crane_type,
            base_position,
            max_elevation_angle: PI / 2.0 - 0.1,
            min_elevation_angle: 0.05,
        };

        Self {
            config,
            mass: crane_type.empty_mass(),
            base_orientation: UnitQuaternion::identity(),
            boom: BoomState {
                length: crane_type.min_boom_length(),
                ..Default::default()
            },
            controls: CraneControls::default(),
            load: None,
            overloaded: false,
            outrigger_pressure: [0.0; 4],
            hydraulic_temperature: 40.0,
        }
    }

    /// Validates that all physical quantities are finite (not NaN or Inf)
    pub fn validate_state(&self) -> bool {
        self.config.base_position.x.is_finite()
            && self.config.base_position.y.is_finite()
            && self.config.base_position.z.is_finite()
            && self.boom.slew_angle.is_finite()
            && self.boom.elevation_angle.is_finite()
            && self.boom.length.is_finite()
            && self.boom.cable_length.is_finite()
            && self.controls.slew.is_finite()
            && self.controls.boom_elevation.is_finite()
            && self.controls.boom_extension.is_finite()
            && self.controls.hoist.is_finite()
    }

    /// Resets the crane to a safe state when invalid values are detected
    pub fn reset_to_safe_state(&mut self) {
        self.boom.slew_angle = 0.0;
        self.boom.elevation_angle = self.config.min_elevation_angle;
        self.boom.cable_length = 1.0;
        self.controls.slew = 0.0;
        self.controls.boom_elevation = 0.0;
        self.controls.boom_extension = 0.0;
        self.controls.hoist = 0.0;
        if !self.config.base_position.x.is_finite()
            || !self.config.base_position.y.is_finite()
            || !self.config.base_position.z.is_finite()
        {
            self.config.base_position = Vector3::zeros();
        }
    }

    /// Прицепить груз
    pub fn attach_load(&mut self, mass: f32, position: Vector3<f32>) -> Result<(), String> {
        let capacity = self.config.crane_type.max_load_capacity();

        if mass > capacity {
            return Err(format!(
                "Перегрузка! Масса {} кг превышает максимальную {} кг",
                mass as u32, capacity as u32
            ));
        }

        self.load = Some(SuspendedLoad {
            mass,
            position,
            velocity: Vector3::zeros(),
            attached: true,
        });

        Ok(())
    }

    /// Отцепить груз
    pub fn detach_load(&mut self) {
        self.load = None;
    }

    /// Проверить стабильность крана
    pub fn check_stability(&self) -> bool {
        // Без выставленных аутригеров работа запрещена
        if !self.controls.outriggers_deployed {
            return false;
        }

        // Проверка опрокидывающего момента
        if let Some(ref load) = self.load {
            let boom_tip = self.get_boom_tip_position();
            let horizontal_distance = ((load.position.x - boom_tip.x).powi(2)
                + (load.position.z - boom_tip.z).powi(2))
            .sqrt();

            // Опрокидывающий момент
            let overturning_moment = load.mass * 9.81 * horizontal_distance;

            // Удерживающий момент (упрощённо)
            let holding_moment = self.mass * 9.81 * 2.0; // 2м - половина ширины базы

            if overturning_moment > holding_moment * 0.8 {
                // 80% запас
                return false;
            }
        }

        true
    }

    /// Получить позицию конца стрелы
    pub fn get_boom_tip_position(&self) -> Vector3<f32> {
        // Локальная позиция конца стрелы
        let local_x = self.boom.length * self.boom.elevation_angle.cos();
        let local_y = self.boom.length * self.boom.elevation_angle.sin();

        // Поворот вокруг оси Y (башня)
        let rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.boom.slew_angle);
        let rotated = rotation * Vector3::new(local_x, local_y, 0.0);

        self.config.base_position + rotated
    }

    /// Получить позицию крюка
    pub fn get_hook_position(&self) -> Vector3<f32> {
        let tip = self.get_boom_tip_position();
        Vector3::new(tip.x, tip.y - self.boom.cable_length, tip.z)
    }

    /// Обновить физику крана
    pub fn update(&mut self, dt: f32, terrain_getter: &dyn Fn(f32, f32) -> f32) {
        // Validate dt to prevent NaN/Inf propagation
        if !dt.is_finite() || dt <= 0.0 {
            tracing::warn!(target: "physics", "Invalid dt in crane physics: {}, skipping update", dt);
            return;
        }

        // Validate current state before update
        if !self.validate_state() {
            tracing::warn!(target: "physics", "Invalid state detected in crane, resetting to safe state");
            self.reset_to_safe_state();
        }

        let crane_type = self.config.crane_type;

        // Выставляем аутригеры только на ровной поверхности
        if self.controls.outriggers_deployed {
            self.update_outrigger_pressure(terrain_getter);
        }

        // Поворот башни
        if self.controls.slew != 0.0 && self.check_stability() {
            let slew_delta = self.controls.slew * crane_type.slew_speed() * dt;
            self.boom.slew_angle += slew_delta;
            // Нормализация угла
            while self.boom.slew_angle > PI * 2.0 {
                self.boom.slew_angle -= PI * 2.0;
            }
            while self.boom.slew_angle < 0.0 {
                self.boom.slew_angle += PI * 2.0;
            }
        }

        // Подъём/опускание стрелы
        if self.controls.boom_elevation != 0.0 && self.check_stability() {
            let elevation_delta = self.controls.boom_elevation * 0.02 * dt;
            self.boom.elevation_angle += elevation_delta;
            self.boom.elevation_angle = self.boom.elevation_angle.clamp(
                self.config.min_elevation_angle,
                self.config.max_elevation_angle,
            );
        }

        // Выдвижение стрелы
        if self.controls.boom_extension != 0.0 && self.check_stability() {
            let extension_speed = 0.3; // м/с
            let extension_delta = self.controls.boom_extension * extension_speed * dt;
            self.boom.length += extension_delta;
            self.boom.length = self
                .boom
                .length
                .clamp(crane_type.min_boom_length(), crane_type.max_boom_length());
        }

        // Подъём/опускание груза
        if self.controls.hoist != 0.0 && self.check_stability() {
            // Get hook position first to avoid borrow issues
            let hook_pos = self.get_hook_position();

            if let Some(ref mut load) = self.load {
                let hoist_speed = crane_type.hoist_speed();
                let hoist_delta = self.controls.hoist * hoist_speed * dt;

                self.boom.cable_length -= hoist_delta;
                self.boom.cable_length = self.boom.cable_length.clamp(1.0, self.boom.length * 0.95);

                // Обновляем позицию груза
                load.position = Vector3::new(hook_pos.x, hook_pos.y - 0.5, hook_pos.z);
            }
        }

        // Физика подвешенного груза (маятник)
        let load_physics_data = if let Some(ref load) = self.load {
            Some((load.position, load.mass, load.velocity))
        } else {
            None
        };

        if let Some((pos, mass, vel)) = load_physics_data {
            // Update physics and get new position
            let hook_pos = self.get_hook_position();
            let new_pos = pos + vel * dt; // Simple physics update

            if let Some(ref mut load) = self.load {
                load.position = new_pos;
                // Проверка перегрузки
                self.overloaded = mass > crane_type.max_load_capacity();
            }
        } else {
            self.overloaded = false;
        }

        // Нагрев гидравлики
        let activity = (self.controls.boom_elevation.abs()
            + self.controls.boom_extension.abs()
            + self.controls.slew.abs()
            + self.controls.hoist.abs())
            / 4.0;

        let target_temp = 40.0 + activity * 60.0;
        self.hydraulic_temperature += (target_temp - self.hydraulic_temperature) * dt * 0.05;
    }

    /// Физика подвешенного груза (простой маятник)
    fn update_load_physics(&mut self, load: &mut SuspendedLoad, dt: f32) {
        let hook_pos = self.get_hook_position();

        // Вектор от крюка к грузу
        let to_load = load.position - hook_pos;
        let distance = to_load.norm();

        if distance < 0.1 {
            return;
        }

        // Направление троса
        let cable_dir = to_load.normalize();

        // Сила тяжести
        let gravity = Vector3::new(0.0, -9.81, 0.0);

        // Сила натяжения троса (упрощённо)
        let tension = -cable_dir * load.mass * 9.81;

        // Демпфирование колебаний
        let damping = -load.velocity * 0.5;

        // Ускорение
        let acceleration = gravity + tension / load.mass + damping / load.mass;

        // Обновление скорости и позиции
        load.velocity += acceleration * dt;
        load.position += load.velocity * dt;

        // Ограничение длины троса
        if distance > self.boom.cable_length {
            // Возвращаем груз в пределах длины троса
            let correction = (distance - self.boom.cable_length) * cable_dir;
            load.position -= correction;

            // Гасим скорость вдоль троса
            let vel_along_cable = load.velocity.dot(&cable_dir);
            load.velocity -= cable_dir * vel_along_cable;
        }
    }

    /// Обновить давление на аутригеры
    fn update_outrigger_pressure(&mut self, terrain_getter: &dyn Fn(f32, f32) -> f32) {
        // Позиции аутригеров относительно базы
        let outrigger_offsets = [
            Vector3::new(2.5, 0.0, 2.0),   // перед-лев
            Vector3::new(2.5, 0.0, -2.0),  // перед-прав
            Vector3::new(-2.5, 0.0, 2.0),  // зад-лев
            Vector3::new(-2.5, 0.0, -2.0), // зад-прав
        ];

        let rotation = self.base_orientation;

        for (i, offset) in outrigger_offsets.iter().enumerate() {
            let world_pos = self.config.base_position + rotation * offset;
            let terrain_height = terrain_getter(world_pos.x, world_pos.z);

            // Простая модель: давление зависит от разницы высот
            let height_diff = (world_pos.y - terrain_height).abs();
            self.outrigger_pressure[i] = if height_diff < 0.3 {
                1.0 - height_diff / 0.3 // Полное давление если касается
            } else {
                0.0 // Не касается
            };
        }
    }

    /// Получить максимальный радиус работы
    pub fn get_max_reach(&self) -> f32 {
        self.config.crane_type.max_boom_length()
    }

    /// Получить состояние для рендеринга
    pub fn get_state(&self) -> CraneState {
        CraneState {
            base_position: self.config.base_position,
            base_orientation: self.base_orientation,
            boom_elevation: self.boom.elevation_angle,
            boom_length: self.boom.length,
            boom_slew: self.boom.slew_angle,
            cable_length: self.boom.cable_length,
            hook_position: self.get_hook_position(),
            load_position: self.load.as_ref().map(|l| l.position),
            load_mass: self.load.as_ref().map(|l| l.mass),
            overloaded: self.overloaded,
            outriggers_deployed: self.controls.outriggers_deployed,
            hydraulic_temperature: self.hydraulic_temperature,
        }
    }

    /// Может ли кран работать на данной поверхности
    pub fn can_operate_on(&self, surface_type: &str) -> bool {
        matches!(
            surface_type,
            "asphalt_good" | "asphalt_bad" | "concrete" | "gravel" | "dirt"
        )
    }
}

/// Состояние крана для рендеринга
#[derive(Debug, Clone)]
pub struct CraneState {
    pub base_position: Vector3<f32>,
    pub base_orientation: UnitQuaternion<f32>,
    pub boom_elevation: f32,
    pub boom_length: f32,
    pub boom_slew: f32,
    pub cable_length: f32,
    pub hook_position: Vector3<f32>,
    pub load_position: Option<Vector3<f32>>,
    pub load_mass: Option<f32>,
    pub overloaded: bool,
    pub outriggers_deployed: bool,
    pub hydraulic_temperature: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crane_creation() {
        let crane = CraneArm::new(CraneType::TruckCrane25t, Vector3::new(0.0, 0.0, 0.0));

        assert_eq!(crane.config.crane_type, CraneType::TruckCrane25t);
        assert_eq!(crane.mass, 28_000.0);
        assert!(!crane.overloaded);
    }

    #[test]
    fn test_attach_load() {
        let mut crane = CraneArm::new(CraneType::TruckCrane25t, Vector3::zeros());

        // Должен успешно прицепить груз 10 тонн
        assert!(crane
            .attach_load(10_000.0, Vector3::new(5.0, -2.0, 0.0))
            .is_ok());
        assert!(crane.load.is_some());

        // Не должен прицепить перегруз
        assert!(crane.attach_load(30_000.0, Vector3::zeros()).is_err());
    }

    #[test]
    fn test_boom_tip_position() {
        let crane = CraneArm::new(CraneType::TruckCrane25t, Vector3::new(0.0, 10.0, 0.0));

        let tip = crane.get_boom_tip_position();

        // Длина стрелы 10м под углом 30°
        let expected_x = 10.0 * (PI / 6.0).cos();
        let expected_y = 10.0 + 10.0 * (PI / 6.0).sin();

        assert!((tip.x - expected_x).abs() < 0.1);
        assert!((tip.y - expected_y).abs() < 0.1);
    }

    #[test]
    fn test_stability_check() {
        let mut crane = CraneArm::new(CraneType::TruckCrane25t, Vector3::zeros());

        // Без аутригеров - нестабилен
        assert!(!crane.check_stability());

        // С аутригерами но без груза - стабилен
        crane.controls.outriggers_deployed = true;
        assert!(crane.check_stability());

        // Прицепить груз
        crane
            .attach_load(5_000.0, Vector3::new(10.0, -5.0, 0.0))
            .expect("Failed to attach load in test");

        // Должен быть стабилен с нормальным грузом
        assert!(crane.check_stability());
    }

    #[test]
    fn test_crane_physics_handles_nan() {
        let mut crane = CraneArm::new(CraneType::TruckCrane25t, Vector3::zeros());

        // Test that NaN in state is detected
        assert!(crane.validate_state());

        // Simulate invalid dt
        crane.update(f32::NAN, &|_, _| 0.0);
        // Should not panic and should reset to safe state
        assert!(crane.validate_state());
    }

    /// Validates that all physical quantities are finite (not NaN or Inf)
    pub fn validate_state(&self) -> bool {
        self.position.x.is_finite()
            && self.position.y.is_finite()
            && self.position.z.is_finite()
            && self.boom.slew_angle.is_finite()
            && self.boom.elevation_angle.is_finite()
            && self.boom.length.is_finite()
            && self.boom.cable_length.is_finite()
            && self.controls.slew.is_finite()
            && self.controls.boom_elevation.is_finite()
            && self.controls.boom_extension.is_finite()
            && self.controls.hoist.is_finite()
            && self.stability_factor.is_finite()
    }

    /// Resets the crane to a safe state when invalid values are detected
    pub fn reset_to_safe_state(&mut self) {
        self.boom.slew_angle = 0.0;
        self.boom.elevation_angle = self.config.min_elevation_angle;
        self.boom.cable_length = 1.0;
        self.controls.slew = 0.0;
        self.controls.boom_elevation = 0.0;
        self.controls.boom_extension = 0.0;
        self.controls.hoist = 0.0;
        self.stability_factor = 1.0;
        // Keep position but ensure it's finite
        if !self.position.is_finite() {
            self.position = Vector3::zeros();
        }
        // Reset load if present
        if let Some(ref mut load) = self.load {
            load.velocity = Vector3::zeros();
            if !load.position.is_finite() {
                load.position = self.position + Vector3::new(0.0, -2.0, 0.0);
            }
        }
    }
}
