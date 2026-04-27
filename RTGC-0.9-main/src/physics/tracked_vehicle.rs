//! Tracked Vehicle Physics for RTGC-0.9
//!
//! Реализация гусеничной техники:
//! - ГТ-СМ, ГАЗ-71, МТ-ЛБ, Т-150К
//! - Детальная физика гусениц с проскальзыванием
//! - Взаимодействие с мягким грунтом
//! - Поворот бортовыми фрикционами
//! - Поддержка лебёдки

use nalgebra::{Matrix3, UnitQuaternion, Vector3};
use std::f32::consts::PI;

use super::deformable_terrain::{DeformableTerrainComponent, DeformationType};
use super::physics_module::{Ray, RaycastHit, RigidBody};
use crate::world::SurfaceType;

/// Типы гусеничной техники
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackedVehicleType {
    /// ГТ-СМ — гусеничный тягач, лёгкий
    GtsM,
    /// ГАЗ-71 — армейский транспортер
    Gaz71,
    /// МТ-ЛБ — многоцелевой бронетранспортер
    MtLb,
    /// Т-150К — тяжёлый трактор
    T150k,
    /// ДТ-75 — сельскохозяйственный трактор
    Dt75,
    /// Витязь ДТ-30 — сочленённый вездеход
    VityazDt30,
}

impl TrackedVehicleType {
    /// Получить массу пустого транспортного средства (кг)
    pub fn empty_mass(&self) -> f32 {
        match self {
            TrackedVehicleType::GtsM => 4500.0,
            TrackedVehicleType::Gaz71 => 3200.0,
            TrackedVehicleType::MtLb => 7200.0,
            TrackedVehicleType::T150k => 6800.0,
            TrackedVehicleType::Dt75 => 4700.0,
            TrackedVehicleType::VityazDt30 => 12500.0,
        }
    }

    /// Максимальная полезная нагрузка (кг)
    pub fn max_payload(&self) -> f32 {
        match self {
            TrackedVehicleType::GtsM => 2000.0,
            TrackedVehicleType::Gaz71 => 1000.0,
            TrackedVehicleType::MtLb => 2000.0,
            TrackedVehicleType::T150k => 3000.0,
            TrackedVehicleType::Dt75 => 2500.0,
            TrackedVehicleType::VityazDt30 => 10000.0,
        }
    }

    /// Мощность двигателя (л.с.)
    pub fn engine_horsepower(&self) -> f32 {
        match self {
            TrackedVehicleType::GtsM => 240.0,
            TrackedVehicleType::Gaz71 => 115.0,
            TrackedVehicleType::MtLb => 240.0,
            TrackedVehicleType::T150k => 180.0,
            TrackedVehicleType::Dt75 => 75.0,
            TrackedVehicleType::VityazDt30 => 710.0,
        }
    }

    /// Ширина гусеницы (м)
    pub fn track_width(&self) -> f32 {
        match self {
            TrackedVehicleType::GtsM => 0.50,
            TrackedVehicleType::Gaz71 => 0.36,
            TrackedVehicleType::MtLb => 0.35,
            TrackedVehicleType::T150k => 0.58,
            TrackedVehicleType::Dt75 => 0.39,
            TrackedVehicleType::VityazDt30 => 0.80,
        }
    }

    /// Длина опорной поверхности гусеницы (м)
    pub fn track_length(&self) -> f32 {
        match self {
            TrackedVehicleType::GtsM => 2.8,
            TrackedVehicleType::Gaz71 => 2.5,
            TrackedVehicleType::MtLb => 2.9,
            TrackedVehicleType::T150k => 2.6,
            TrackedVehicleType::Dt75 => 2.4,
            TrackedVehicleType::VityazDt30 => 4.2,
        }
    }

    /// Количество опорных катков на сторону
    pub fn road_wheel_count(&self) -> u32 {
        match self {
            TrackedVehicleType::GtsM => 5,
            TrackedVehicleType::Gaz71 => 4,
            TrackedVehicleType::MtLb => 6,
            TrackedVehicleType::T150k => 5,
            TrackedVehicleType::Dt75 => 6,
            TrackedVehicleType::VityazDt30 => 7,
        }
    }

    /// Удельное давление на грунт (кгс/см²)
    pub fn ground_pressure(&self) -> f32 {
        // P = масса / (2 * ширина * длина)
        let mass = self.empty_mass() + self.max_payload();
        let area = 2.0 * self.track_width() * self.track_length(); // м²
        let pressure_kg_m2 = mass / area;
        pressure_kg_m2 / 10000.0 * 1000.0 // кгс/см²
    }

    /// Максимальная скорость по шоссе (км/ч)
    pub fn max_speed_kmh(&self) -> f32 {
        match self {
            TrackedVehicleType::GtsM => 50.0,
            TrackedVehicleType::Gaz71 => 60.0,
            TrackedVehicleType::MtLb => 60.0,
            TrackedVehicleType::T150k => 43.0,
            TrackedVehicleType::Dt75 => 11.0,
            TrackedVehicleType::VityazDt30 => 50.0,
        }
    }

    /// Запас хода по шоссе (км)
    pub fn range_km(&self) -> f32 {
        match self {
            TrackedVehicleType::GtsM => 500.0,
            TrackedVehicleType::Gaz71 => 600.0,
            TrackedVehicleType::MtLb => 500.0,
            TrackedVehicleType::T150k => 625.0,
            TrackedVehicleType::Dt75 => 750.0,
            TrackedVehicleType::VityazDt30 => 500.0,
        }
    }
}

/// Управление гусеничным транспортным средством
#[derive(Debug, Clone, Default)]
pub struct TrackedControls {
    /// Газ (0.0 - 1.0)
    pub throttle: f32,
    /// Тормоз (0.0 - 1.0)
    pub brake: f32,
    /// Поворот левой гусеницы (-1.0 - 1.0, отрицательный = назад)
    pub left_track: f32,
    /// Поворот правой гусеницы (-1.0 - 1.0, отрицательный = назад)
    pub right_track: f32,
    /// Бортовой фрикцион левый (0.0 - 1.0)
    pub left_clutch: f32,
    /// Бортовой фрикцион правый (0.0 - 1.0)
    pub right_clutch: f32,
}

impl TrackedControls {
    /// Создать управление из ввода (клавиатура/геймпад)
    pub fn from_input(forward: f32, turn: f32, brake_pressed: bool) -> Self {
        let mut controls = Self::default();

        if forward > 0.0 {
            controls.left_track = forward;
            controls.right_track = forward;
        } else if forward < 0.0 {
            controls.left_track = forward;
            controls.right_track = forward;
        }

        // Поворот дифференциалом
        if turn != 0.0 {
            if forward >= 0.0 {
                // Поворот в движении: притормаживаем одну гусеницу
                if turn < 0.0 {
                    controls.left_track *= 0.5;
                } else {
                    controls.right_track *= 0.5;
                }
            } else {
                // Поворот на месте: гусеницы в разные стороны
                controls.left_track = -turn;
                controls.right_track = turn;
            }
        }

        controls.brake = if brake_pressed { 1.0 } else { 0.0 };

        controls
    }
}

/// Состояние одной гусеницы
#[derive(Debug, Clone)]
pub struct TrackState {
    /// Текущая скорость (м/с)
    pub velocity: f32,
    /// Проскальзывание (0.0 = нет, 1.0 = полное)
    pub slip: f32,
    /// Нагрузка (Н)
    pub load: f32,
    /// Температура (°C)
    pub temperature: f32,
}

impl Default for TrackState {
    fn default() -> Self {
        Self {
            velocity: 0.0,
            slip: 0.0,
            load: 0.0,
            temperature: 20.0,
        }
    }
}

/// Компонент подвески опорного катка
#[derive(Debug, Clone)]
pub struct RoadWheelSuspension {
    /// Позиция относительно центра масс (локальные координаты)
    pub local_position: Vector3<f32>,
    /// Ход подвески (м)
    pub travel: f32,
    /// Жёсткость пружины (Н/м)
    pub spring_stiffness: f32,
    /// Демпфирование (Н·с/м)
    pub damping: f32,
    /// Текущее сжатие (м)
    pub compression: f32,
    /// Скорость сжатия (м/с)
    pub compression_velocity: f32,
}

impl RoadWheelSuspension {
    pub fn new(x: f32, y: f32, z: f32, travel: f32) -> Self {
        Self {
            local_position: Vector3::new(x, y, z),
            travel,
            spring_stiffness: 80000.0, // Н/м
            damping: 5000.0,           // Н·с/м
            compression: 0.0,
            compression_velocity: 0.0,
        }
    }

    /// Обновить состояние подвески
    pub fn update(&mut self, terrain_height: f32, wheel_world_y: f32, dt: f32) -> f32 {
        // Целевая позиция колеса на поверхности
        let target_y = terrain_height + self.local_position.y;

        // Фактическая позиция
        let current_y = wheel_world_y;

        // Сжатие подвески
        let new_compression = (target_y - current_y).max(0.0).min(self.travel);

        // Скорость сжатия
        self.compression_velocity = (new_compression - self.compression) / dt;
        self.compression = new_compression;

        // Сила подвески
        let spring_force = self.spring_stiffness * self.compression;
        let damping_force = self.damping * self.compression_velocity;

        spring_force + damping_force
    }
}

/// Гусеничное транспортное средство
#[derive(Clone, Debug)]
pub struct TrackedVehicle {
    /// Тип транспортного средства
    pub vehicle_type: TrackedVehicleType,
    /// Масса с грузом (кг)
    pub mass: f32,
    /// Позиция (м)
    pub position: Vector3<f32>,
    /// Ориентация
    pub orientation: UnitQuaternion<f32>,
    /// Линейная скорость (м/с)
    pub linear_velocity: Vector3<f32>,
    /// Угловая скорость (рад/с)
    pub angular_velocity: Vector3<f32>,
    /// Управление
    pub controls: TrackedControls,
    /// Состояние левой гусеницы
    pub left_track: TrackState,
    /// Состояние правой гусеницы
    pub right_track: TrackState,
    /// Подвеска опорных катков
    pub suspensions: Vec<RoadWheelSuspension>,
    /// Топливо (кг)
    pub fuel: f32,
    /// Расход топлива (кг/ч)
    pub fuel_consumption: f32,
    /// Температура двигателя (°C)
    pub engine_temperature: f32,
    /// Работает ли двигатель
    pub engine_running: bool,
    /// ID тела шасси в PhysicsWorld
    pub chassis_body_id: Option<usize>,
}

impl TrackedVehicle {
    /// Создать новое гусеничное транспортное средство
    pub fn new(vehicle_type: TrackedVehicleType, position: Vector3<f32>) -> Self {
        let mut suspensions = Vec::new();
        let track_length = vehicle_type.track_length();
        let wheel_count = vehicle_type.road_wheel_count() as usize;

        // Равномерно распределяем катки вдоль гусеницы
        let spacing = track_length / (wheel_count + 1) as f32;
        for i in 0..wheel_count {
            let x = -track_length / 2.0 + (i + 1) as f32 * spacing;
            suspensions.push(RoadWheelSuspension::new(x, -0.3, 0.0, 0.2));
        }

        let mass = vehicle_type.empty_mass();

        Self {
            vehicle_type,
            mass,
            position,
            orientation: UnitQuaternion::identity(),
            linear_velocity: Vector3::zeros(),
            angular_velocity: Vector3::zeros(),
            controls: TrackedControls::default(),
            left_track: TrackState::default(),
            right_track: TrackState::default(),
            suspensions,
            fuel: vehicle_type.range_km() * 0.5, // Половина бака
            fuel_consumption: 0.0,
            engine_temperature: 20.0,
            engine_running: false,
            chassis_body_id: None,
        }
    }

    /// Установить ID тела шасси
    pub fn set_chassis_body_id(&mut self, id: usize) {
        self.chassis_body_id = Some(id);
    }

    /// Set throttle input
    pub fn set_throttle(&mut self, throttle: f32) {
        self.controls.throttle = throttle.clamp(0.0, 1.0);
    }

    /// Set brake input
    pub fn set_brake(&mut self, brake: f32) {
        self.controls.brake = brake.clamp(0.0, 1.0);
    }

    /// Set turning (left/right track differential)
    pub fn set_turn(&mut self, turn: f32) {
        if turn < 0.0 {
            self.controls.left_track = turn;
            self.controls.right_track = 0.0;
        } else if turn > 0.0 {
            self.controls.left_track = 0.0;
            self.controls.right_track = turn;
        } else {
            self.controls.left_track = self.controls.throttle;
            self.controls.right_track = self.controls.throttle;
        }
    }

    /// Получить ID тела шасси
    pub fn chassis_body_id(&self) -> Option<usize> {
        self.chassis_body_id
    }

    /// Запустить двигатель
    pub fn start_engine(&mut self) -> bool {
        if !self.engine_running && self.fuel > 1.0 {
            self.engine_running = true;
            self.engine_temperature = 60.0;
            true
        } else {
            false
        }
    }

    /// Остановить двигатель
    pub fn stop_engine(&mut self) {
        self.engine_running = false;
    }

    /// Обновить физику с учётом типа поверхности
    pub fn update_with_surface(
        &mut self,
        dt: f32,
        terrain_getter: &dyn Fn(f32, f32) -> f32,
        surface_getter: &dyn Fn(f32, f32) -> SurfaceType,
        deformable_terrain: Option<&mut DeformableTerrainComponent>,
    ) {
        if !self.engine_running {
            // Затухание без двигателя
            self.linear_velocity *= 0.98;
            self.angular_velocity *= 0.95;
            return;
        }

        // Потребление топлива
        let throttle_avg = (self.controls.left_track.abs() + self.controls.right_track.abs()) / 2.0;
        let base_consumption = self.vehicle_type.engine_horsepower() * 0.0002; // кг/с при полном газе
        self.fuel_consumption = base_consumption * throttle_avg;
        self.fuel = (self.fuel - self.fuel_consumption * dt).max(0.0);

        if self.fuel < 1.0 {
            self.stop_engine();
            return;
        }

        // Температура двигателя
        let target_temp = 80.0 + throttle_avg * 40.0;
        self.engine_temperature += (target_temp - self.engine_temperature) * dt * 0.1;

        // Получаем высоту terrain под каждым катком
        let mut suspension_forces = Vec::new();
        let mut total_normal_force = 0.0;

        for suspension in &mut self.suspensions {
            // Мировая позиция катка
            let world_pos = self.position + self.orientation * suspension.local_position;

            // Высота terrain
            let terrain_height = terrain_getter(world_pos.x, world_pos.z);

            // Сила подвески
            let force = suspension.update(terrain_height, world_pos.y, dt);
            suspension_forces.push(force);
            total_normal_force += force;
        }

        // Силы от гусениц
        let max_force = self.vehicle_type.engine_horsepower() * 735.5
            / (self.vehicle_type.max_speed_kmh() / 3.6); // Н

        let left_force = self.controls.left_track * max_force;
        let right_force = self.controls.right_track * max_force;

        // Применяем бортовые фрикции
        let left_effective = left_force * (1.0 - self.controls.left_clutch);
        let right_effective = right_force * (1.0 - self.controls.right_clutch);

        // Торможение
        let brake_force = self.controls.brake * 20000.0;

        // Направление движения (локальная ось X)
        let forward = self.orientation * Vector3::x();
        let right = self.orientation * Vector3::z();

        // Средняя сила тяги
        let avg_force = (left_effective + right_effective) / 2.0;

        // Сила тяги с учётом проскальзывания
        let slip_factor_left = 1.0 - self.left_track.slip;
        let slip_factor_right = 1.0 - self.right_track.slip;

        let traction_force = forward
            * (left_effective * slip_factor_left + right_effective * slip_factor_right)
            / 2.0;

        // Момент поворота от разницы сил гусениц
        let track_width = self.vehicle_type.track_width() * 2.0;
        let turning_torque = (right_effective - left_effective) * track_width / 2.0;

        // Сопротивление качению с учётом типа поверхности
        let surface_pos = (self.position.x, self.position.z);
        let surface_type = surface_getter(surface_pos.0, surface_pos.1);
        let rolling_resistance_coeff = surface_type.rolling_resistance();
        let rolling_resistance = self.mass * 9.81 * rolling_resistance_coeff;
        let rolling_force = if self.linear_velocity.norm() > 0.001 {
            -self.linear_velocity.normalize() * rolling_resistance
        } else {
            Vector3::zeros()
        };

        // Тормозная сила
        let brake_vector = if self.linear_velocity.norm() > 0.001 {
            -self.linear_velocity.normalize() * brake_force
        } else {
            Vector3::zeros()
        };

        // Суммарная сила
        let total_force = traction_force + rolling_force + brake_vector;

        // Ускорение
        let acceleration = total_force / self.mass;

        // Обновляем линейную скорость
        self.linear_velocity += acceleration * dt;

        // Угловое ускорение от поворота
        let moment_of_inertia = self.mass * track_width.powi(2) / 12.0;
        let angular_acceleration = turning_torque / moment_of_inertia;

        // Угловая скорость вокруг Y
        self.angular_velocity.y += angular_acceleration * dt;
        self.angular_velocity.y *= 0.95; // Затухание

        // Применяем скорость
        self.position += self.linear_velocity * dt;

        // Поворот
        let rotation =
            UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.angular_velocity.y * dt);
        self.orientation = rotation * self.orientation;

        // Обновляем проскальзывание гусениц
        self.update_track_slip(dt, terrain_getter);

        // Деформация грунта
        if let Some(terrain) = deformable_terrain {
            self.deform_terrain(terrain, total_normal_force);
        }
    }

    /// Обновить проскальзывание гусениц
    fn update_track_slip(&mut self, dt: f32, terrain_getter: &dyn Fn(f32, f32) -> f32) {
        // Теоретическая скорость от вращения гусениц
        let left_theoretical = self.controls.left_track * self.vehicle_type.max_speed_kmh() / 3.6;
        let right_theoretical = self.controls.right_track * self.vehicle_type.max_speed_kmh() / 3.6;

        // Фактическая скорость (проекция на направление гусениц)
        let forward = self.orientation * Vector3::x();
        let actual_speed = self.linear_velocity.dot(&forward);

        // Вычисляем проскальзывание
        if left_theoretical.abs() > 0.1 {
            let slip_left = (left_theoretical - actual_speed) / left_theoretical;
            self.left_track.slip += (slip_left.abs() - self.left_track.slip) * dt * 2.0;
            self.left_track.slip = self.left_track.slip.clamp(0.0, 1.0);
        }

        if right_theoretical.abs() > 0.1 {
            let slip_right = (right_theoretical - actual_speed) / right_theoretical;
            self.right_track.slip += (slip_right.abs() - self.right_track.slip) * dt * 2.0;
            self.right_track.slip = self.right_track.slip.clamp(0.0, 1.0);
        }

        // Охлаждение гусениц
        self.left_track.temperature += (20.0 - self.left_track.temperature) * dt * 0.05;
        self.right_track.temperature += (20.0 - self.right_track.temperature) * dt * 0.05;

        // Нагрев от трения
        let friction_heat = (self.left_track.slip + self.right_track.slip) * 50.0 * dt;
        self.left_track.temperature += friction_heat;
        self.right_track.temperature += friction_heat;
    }

    /// Деформация грунта под гусеницами
    fn deform_terrain(&self, terrain: &mut DeformableTerrainComponent, normal_force: f32) {
        let pressure =
            normal_force / (self.vehicle_type.track_width() * self.vehicle_type.track_length());

        // Глубина колеи зависит от давления и типа грунта
        let depth_factor = pressure * 0.0001; // Упрощённая модель

        // Применяем деформацию для каждой точки подвески
        for suspension in &self.suspensions {
            let world_pos = self.position + self.orientation * suspension.local_position;
            terrain.apply_deformation(world_pos, DeformationType::Press(depth_factor));
        }
    }

    /// Получить состояние для рендеринга
    pub fn get_state(&self) -> TrackedVehicleState {
        TrackedVehicleState {
            position: self.position,
            orientation: self.orientation,
            linear_velocity: self.linear_velocity,
            angular_velocity: self.angular_velocity,
            left_track_slip: self.left_track.slip,
            right_track_slip: self.right_track.slip,
            fuel_remaining: self.fuel,
            engine_running: self.engine_running,
            engine_temperature: self.engine_temperature,
        }
    }

    /// Проверить, может ли проехать по поверхности
    pub fn can_traverse(&self, surface_type: &str) -> bool {
        // Гусеницы могут ехать почти везде
        matches!(
            surface_type,
            "dirt" | "mud" | "sand" | "snow" | "grass" | "gravel" | "asphalt_bad" | "asphalt_good"
        )
    }

    /// Обновление физики с интеграцией в PhysicsWorld
    pub fn physics_update(
        &mut self,
        dt: f32,
        physics_world: &mut crate::physics::PhysicsWorld,
        terrain_getter: &dyn Fn(f32, f32) -> f32,
        surface_getter: &dyn Fn(f32, f32) -> crate::world::SurfaceType,
        deformable_terrain: Option<&mut crate::physics::DeformableTerrainComponent>,
    ) {
        // Validate dt to prevent NaN/Inf propagation
        if !dt.is_finite() || dt <= 0.0 {
            tracing::warn!(target: "physics", "Invalid dt in tracked vehicle physics: {}, skipping update", dt);
            return;
        }

        // Validate current state before update
        if !self.validate_state() {
            tracing::warn!(target: "physics", "Invalid state detected in tracked vehicle, resetting to safe state");
            self.reset_to_safe_state();
        }

        // Вызываем основной метод update с terrain и surface
        self.update_with_surface(dt, terrain_getter, surface_getter, deformable_terrain);

        // Validate state after update
        if !self.validate_state() {
            tracing::warn!(target: "physics", "State became invalid after update in tracked vehicle, resetting");
            self.reset_to_safe_state();
        }

        // Синхронизируем состояние тела шасси с PhysicsWorld если есть
        if let Some(chassis_id) = self.chassis_body_id {
            if let Some(body) = physics_world.get_body_mut(chassis_id) {
                // Синхронизируем позицию и ориентацию (используем orientation для единообразия)
                body.position = self.position;
                body.rotation = self.orientation;
                body.velocity = self.linear_velocity;
                body.angular_velocity = self.angular_velocity;
            }
        }
    }

    /// Validates that all physical quantities are finite (not NaN or Inf)
    pub fn validate_state(&self) -> bool {
        self.position.x.is_finite()
            && self.position.y.is_finite()
            && self.position.z.is_finite()
            && self.linear_velocity.x.is_finite()
            && self.linear_velocity.y.is_finite()
            && self.linear_velocity.z.is_finite()
            && self.angular_velocity.x.is_finite()
            && self.angular_velocity.y.is_finite()
            && self.angular_velocity.z.is_finite()
            && self.orientation.coords.w.is_finite()
            && self.orientation.coords.x.is_finite()
            && self.orientation.coords.y.is_finite()
            && self.orientation.coords.z.is_finite()
            && self.left_track.slip.is_finite()
            && self.right_track.slip.is_finite()
            && self.fuel.is_finite()
            && self.engine_temperature.is_finite()
    }

    /// Resets the vehicle to a safe state when invalid values are detected
    pub fn reset_to_safe_state(&mut self) {
        self.linear_velocity = Vector3::zeros();
        self.angular_velocity = Vector3::zeros();
        self.left_track.slip = 0.0;
        self.right_track.slip = 0.0;
        // Keep position and orientation, but ensure they are finite
        if !self.position.x.is_finite()
            || !self.position.y.is_finite()
            || !self.position.z.is_finite()
        {
            self.position = Vector3::zeros();
        }
        if !self.orientation.coords.w.is_finite()
            || !self.orientation.coords.x.is_finite()
            || !self.orientation.coords.y.is_finite()
            || !self.orientation.coords.z.is_finite()
        {
            self.orientation = UnitQuaternion::identity();
        }
        // Reset engine parameters to safe values
        if !self.engine_temperature.is_finite() {
            self.engine_temperature = 60.0;
        }
        if !self.fuel.is_finite() {
            self.fuel = 200.0; // Default fuel amount
        }
    }
}

/// Состояние гусеничного транспортного средства для рендеринга
#[derive(Debug, Clone)]
pub struct TrackedVehicleState {
    pub position: Vector3<f32>,
    pub orientation: UnitQuaternion<f32>,
    pub linear_velocity: Vector3<f32>,
    pub angular_velocity: Vector3<f32>,
    pub left_track_slip: f32,
    pub right_track_slip: f32,
    pub fuel_remaining: f32,
    pub engine_running: bool,
    pub engine_temperature: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gtsm_creation() {
        let vehicle = TrackedVehicle::new(TrackedVehicleType::GtsM, Vector3::new(0.0, 10.0, 0.0));

        assert_eq!(vehicle.vehicle_type, TrackedVehicleType::GtsM);
        assert_eq!(vehicle.mass, 4500.0);
        assert!(vehicle.fuel > 0.0);
        assert!(!vehicle.engine_running);
    }

    #[test]
    fn test_engine_start() {
        let mut vehicle = TrackedVehicle::new(TrackedVehicleType::Gaz71, Vector3::zeros());

        assert!(vehicle.start_engine());
        assert!(vehicle.engine_running);
        assert_eq!(vehicle.engine_temperature, 60.0);
    }

    #[test]
    fn test_ground_pressure() {
        // МТ-ЛБ должен иметь низкое удельное давление
        let mt_lb = TrackedVehicleType::MtLb;
        let pressure = mt_lb.ground_pressure();

        // Должно быть около 0.2-0.5 кгс/см²
        assert!(pressure > 0.1 && pressure < 1.0);
    }

    #[test]
    fn test_controls_from_input() {
        let controls = TrackedControls::from_input(0.8, -0.3, false);

        assert!(controls.left_track > 0.0);
        assert!(controls.right_track > 0.0);
        assert_eq!(controls.brake, 0.0);
    }
}
