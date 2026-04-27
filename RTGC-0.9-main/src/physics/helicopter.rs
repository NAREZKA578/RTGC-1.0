//! Helicopter Physics Module for RTGC-0.9
//!
//! Универсальная реализация физики вертолётов с поддержкой различных конфигураций:
//! - Параметрическая настройка для любых типов вертолётов (от лёгких до тяжёлых)
//! - Главный ротор с Blade Element Momentum Theory (BEMT)
//! - Хвостовой ротор с компенсацией реактивного момента
//! - Турбовальный/поршневой двигатель с настраиваемыми характеристиками
//! - Аэродинамика фюзеляжа с учётом формы и размеров
//! - Система управления с настраиваемой чувствительностью
//! - Автопилот и системы стабилизации
//! - Оптимизированные вычисления для высокой производительности
//!
//! # Пример использования
//! ```
//! use nalgebra::Vector3;
//! use rtgc::physics::helicopter::{Helicopter, HelicopterConfig};
//!
//! // Создание конфигурации для лёгкого вертолёта
//! let config = HelicopterConfig::light_helicopter();
//! let mut heli = Helicopter::with_config(Vector3::new(0.0, 10.0, 0.0), config);
//!
//! // Обновление физики
//! heli.update(0.016); // dt = 16ms
//! ```

use nalgebra::{Matrix3, Point3, UnitQuaternion, Vector3};
use std::f32::consts::PI;
use std::sync::Arc;

/// Конфигурация вертолёта для быстрой настройки параметров
#[derive(Debug, Clone)]
pub struct HelicopterConfig {
    // Основные параметры
    pub mass_empty: f32,             // Масса пустого вертолёта (кг)
    pub max_payload: f32,            // Максимальная полезная нагрузка (кг)
    pub fuel_capacity: f32,          // Вместимость топливного бака (кг)
    pub main_rotor_radius: f32,      // Радиус главного ротора (м)
    pub main_rotor_blade_count: u32, // Количество лопастей главного ротора
    pub tail_rotor_radius: f32,      // Радиус хвостового ротора (м)
    pub tail_rotor_blade_count: u32, // Количество лопастей хвостового ротора
    pub tail_rotor_distance: f32,    // Расстояние от ЦМ до хвостового ротора (м)

    // Параметры двигателя
    pub engine_type: EngineType, // Тип двигателя
    pub max_engine_power: f32,   // Максимальная мощность (Вт)
    pub max_engine_torque: f32,  // Максимальный крутящий момент (Н*м)
    pub idle_rpm: f32,           // Обороты холостого хода
    pub max_rpm: f32,            // Максимальные обороты

    // Аэродинамика фюзеляжа
    pub fuselage_length: f32,    // Длина фюзеляжа (м)
    pub fuselage_width: f32,     // Ширина фюзеляжа (м)
    pub fuselage_height: f32,    // Высота фюзеляжа (м)
    pub drag_coefficient_x: f32, // Коэффициент сопротивления по X
    pub drag_coefficient_y: f32, // Коэффициент сопротивления по Y
    pub drag_coefficient_z: f32, // Коэффициент сопротивления по Z

    // Инерция
    pub inertia_xx: f32, // Момент инерции вокруг оси X (кг*м²)
    pub inertia_yy: f32, // Момент инерции вокруг оси Y (кг*м²)
    pub inertia_zz: f32, // Момент инерции вокруг оси Z (кг*м²)

    // Управление
    pub max_collective_pitch: f32, // Максимальный общий шаг (рад)
    pub max_cyclic_pitch: f32,     // Максимальный циклический шаг (рад)
    pub max_tail_pitch: f32,       // Максимальный шаг хвостового ротора (рад)
    pub control_smoothing: f32,    // Коэффициент сглаживания управления

    // Внешняя подвеска (для перевозки грузов)
    pub has_cargo_hook: bool,            // Наличие грузового крюка
    pub cargo_hook_offset: Vector3<f32>, // Позиция крюка относительно ЦМ (м)
    pub max_cargo_mass: f32,             // Максимальная масса груза на внешней подвеске (кг)
    pub cargo_cable_length: f32,         // Длина троса подвески (м)
}

/// Тип двигателя вертолёта
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineType {
    Turboshaft, // Турбовальный (для средних и тяжёлых вертолётов)
    Piston,     // Поршневой (для лёгких вертолётов)
    Electric,   // Электрический (перспективные модели)
}

impl HelicopterConfig {
    /// Конфигурация для лёгкого вертолёта (типа Robinson R22)
    pub fn light_helicopter() -> Self {
        Self {
            mass_empty: 620.0,
            max_payload: 340.0,
            fuel_capacity: 75.0,
            main_rotor_radius: 3.83,
            main_rotor_blade_count: 2,
            tail_rotor_radius: 0.58,
            tail_rotor_blade_count: 2,
            tail_rotor_distance: 4.4,
            engine_type: EngineType::Piston,
            max_engine_power: 97000.0, // 131 л.с.
            max_engine_torque: 350.0,
            idle_rpm: 500.0,
            max_rpm: 2500.0,
            fuselage_length: 6.4,
            fuselage_width: 1.2,
            fuselage_height: 2.0,
            drag_coefficient_x: 0.6,
            drag_coefficient_y: 1.0,
            drag_coefficient_z: 0.7,
            inertia_xx: 800.0,
            inertia_yy: 1200.0,
            inertia_zz: 1500.0,
            max_collective_pitch: 0.26, // ~15°
            max_cyclic_pitch: 0.17,     // ~10°
            max_tail_pitch: 0.35,       // ~20°
            control_smoothing: 5.0,
            // Внешняя подвеска (для перевозки грузов)
            has_cargo_hook: false, // R22 обычно не имеет внешнего крюка
            cargo_hook_offset: Vector3::new(0.0, -0.5, 0.0),
            max_cargo_mass: 0.0,
            cargo_cable_length: 0.0,
        }
    }

    /// Конфигурация для среднего вертолёта (типа Bell UH-1 Huey)
    pub fn medium_helicopter() -> Self {
        Self {
            mass_empty: 2370.0,
            max_payload: 1800.0,
            fuel_capacity: 400.0,
            main_rotor_radius: 7.32,
            main_rotor_blade_count: 2,
            tail_rotor_radius: 1.1,
            tail_rotor_blade_count: 2,
            tail_rotor_distance: 8.5,
            engine_type: EngineType::Turboshaft,
            max_engine_power: 1050000.0, // 1400 л.с.
            max_engine_torque: 4000.0,
            idle_rpm: 6000.0,
            max_rpm: 6600.0,
            fuselage_length: 12.7,
            fuselage_width: 2.0,
            fuselage_height: 2.7,
            drag_coefficient_x: 0.7,
            drag_coefficient_y: 1.2,
            drag_coefficient_z: 0.8,
            inertia_xx: 3500.0,
            inertia_yy: 5000.0,
            inertia_zz: 6500.0,
            max_collective_pitch: 0.30,
            max_cyclic_pitch: 0.20,
            max_tail_pitch: 0.40,
            control_smoothing: 4.0,
            // Внешняя подвеска
            has_cargo_hook: true, // UH-1 может перевозить грузы на внешней подвеске
            cargo_hook_offset: Vector3::new(0.0, -1.0, 0.5),
            max_cargo_mass: 1500.0,
            cargo_cable_length: 10.0,
        }
    }

    /// Конфигурация для тяжёлого вертолёта (типа Mi-8)
    pub fn heavy_helicopter() -> Self {
        Self {
            mass_empty: 7200.0,
            max_payload: 4000.0,
            fuel_capacity: 1800.0,
            main_rotor_radius: 9.55,
            main_rotor_blade_count: 5,
            tail_rotor_radius: 1.8,
            tail_rotor_blade_count: 3,
            tail_rotor_distance: 11.0,
            engine_type: EngineType::Turboshaft,
            max_engine_power: 2500000.0, // 2×1700 л.с.
            max_engine_torque: 12000.0,
            idle_rpm: 6500.0,
            max_rpm: 7200.0,
            fuselage_length: 18.4,
            fuselage_width: 2.5,
            fuselage_height: 3.8,
            drag_coefficient_x: 0.8,
            drag_coefficient_y: 1.4,
            drag_coefficient_z: 0.9,
            inertia_xx: 12000.0,
            inertia_yy: 18000.0,
            inertia_zz: 22000.0,
            max_collective_pitch: 0.35,
            max_cyclic_pitch: 0.22,
            max_tail_pitch: 0.45,
            control_smoothing: 3.0,
            // Внешняя подвеска (основное назначение Ми-8 - перевозка грузов)
            has_cargo_hook: true, // Ми-8 оснащён грузовым крюком БЛ-56
            cargo_hook_offset: Vector3::new(0.0, -1.2, 0.8),
            max_cargo_mass: 4000.0,   // До 4 тонн на внешней подвеске
            cargo_cable_length: 15.0, // Стандартная длина троса
        }
    }

    /// Пользовательская конфигурация
    pub fn custom(mass: f32, main_rotor_radius: f32, blade_count: u32, engine_power: f32) -> Self {
        // Автоматический расчёт параметров на основе основных характеристик
        let rotor_disk_area = PI * main_rotor_radius * main_rotor_radius;
        let _disk_loading = mass / rotor_disk_area;

        Self {
            mass_empty: mass * 0.6,
            max_payload: mass * 0.4,
            fuel_capacity: mass * 0.15,
            main_rotor_radius,
            main_rotor_blade_count: blade_count,
            tail_rotor_radius: main_rotor_radius * 0.15,
            tail_rotor_blade_count: if blade_count >= 4 { 3 } else { 2 },
            tail_rotor_distance: main_rotor_radius * 1.2,
            engine_type: if engine_power > 500000.0 {
                EngineType::Turboshaft
            } else {
                EngineType::Piston
            },
            max_engine_power: engine_power,
            max_engine_torque: engine_power / 400.0,
            idle_rpm: if engine_power > 500000.0 {
                6000.0
            } else {
                500.0
            },
            max_rpm: if engine_power > 500000.0 {
                6500.0
            } else {
                2500.0
            },
            fuselage_length: main_rotor_radius * 2.0,
            fuselage_width: main_rotor_radius * 0.3,
            fuselage_height: main_rotor_radius * 0.4,
            drag_coefficient_x: 0.7,
            drag_coefficient_y: 1.2,
            drag_coefficient_z: 0.8,
            inertia_xx: mass * main_rotor_radius * main_rotor_radius * 0.3,
            inertia_yy: mass * main_rotor_radius * main_rotor_radius * 0.5,
            inertia_zz: mass * main_rotor_radius * main_rotor_radius * 0.6,
            max_collective_pitch: 0.30,
            max_cyclic_pitch: 0.20,
            max_tail_pitch: 0.40,
            control_smoothing: 4.0,
            // Внешняя подвеска (по умолчанию включена для транспортных вертолётов)
            has_cargo_hook: true,
            cargo_hook_offset: Vector3::new(0.0, -1.0, 0.5),
            max_cargo_mass: mass * 0.4, // До 40% от массы
            cargo_cable_length: 10.0,
        }
    }

    /// Конфигурация для транспортного вертолёта с внешней подвеской
    pub fn cargo_helicopter(
        mass: f32,
        main_rotor_radius: f32,
        blade_count: u32,
        engine_power: f32,
        max_cargo_mass: f32,
        cable_length: f32,
    ) -> Self {
        let mut config = Self::custom(mass, main_rotor_radius, blade_count, engine_power);
        config.has_cargo_hook = true;
        config.max_cargo_mass = max_cargo_mass;
        config.cargo_cable_length = cable_length;
        config.cargo_hook_offset = Vector3::new(0.0, -1.2, 0.8);
        config
    }
}

/// Конфигурация главного ротора
#[derive(Debug, Clone)]
pub struct MainRotor {
    pub radius: f32,                    // Радиус ротора (м)
    pub blade_count: u32,               // Количество лопастей
    pub chord_length: f32,              // Хорда лопасти (м)
    pub rotational_speed: f32,          // Скорость вращения (рад/с)
    pub collective_pitch: f32,          // Общий шаг лопастей (рад)
    pub cyclic_pitch_longitudinal: f32, // Циклический шаг продольный (рад)
    pub cyclic_pitch_lateral: f32,      // Циклический шаг поперечный (рад)
    pub blade_twist: f32,               // Закрутка лопасти (рад/м)
    pub airfoil_lift_slope: f32,        // Наклон кривой подъёмной силы (1/рад)
    pub tip_loss_factor: f32,           // Фактор потерь на концах (0.0-1.0)
    pub coning_angle: f32,              // Угол конусности (рад)
    pub flapping_hinge_offset: f32,     // Смещение горизонтального шарнира (м)
    pub current_rpm: f32,               // Текущие обороты (об/мин)
    pub target_rpm: f32,                // Целевые обороты (об/мин)
    pub idle_rpm: f32,                  // Обороты холостого хода (об/мин)
    pub rotor_tilt_x: f32,              // Наклон ротора вперёд/назад (рад)
    pub rotor_tilt_y: f32,              // Наклон ротора влево/вправо (рад)
}

impl MainRotor {
    pub fn new(radius: f32, blade_count: u32) -> Self {
        Self {
            radius,
            blade_count,
            chord_length: 0.3,      // Типичная хорда для лёгкого вертолёта
            rotational_speed: 40.0, // ~380 RPM
            collective_pitch: 0.1,  // ~6 градусов
            cyclic_pitch_longitudinal: 0.0,
            cyclic_pitch_lateral: 0.0,
            blade_twist: -0.087, // -5 градусов закрутки (для равномерной подъёмной силы)
            airfoil_lift_slope: 5.7, // Типично для NACA 0012
            tip_loss_factor: 0.95,
            coning_angle: 0.05, // ~3 градуса
            flapping_hinge_offset: 0.1,
            current_rpm: 0.0,
            target_rpm: 400.0,
            idle_rpm: 200.0, // Idle RPM
            rotor_tilt_x: 0.0,
            rotor_tilt_y: 0.0,
        }
    }

    /// Расчёт подъёмной силы главного ротора using Blade Element Momentum Theory
    pub fn calculate_thrust(
        &self,
        air_density: f32,
        inflow_velocity: Vector3<f32>,
    ) -> Vector3<f32> {
        let omega = self.current_rpm * 2.0 * PI / 60.0; // Convert RPM to rad/s
        let tip_speed = omega * self.radius;

        if tip_speed < 1.0 {
            return Vector3::zeros();
        }

        // Разбиваем лопасть на элементы для интегрирования
        let num_elements = 20;
        let dr = self.radius / num_elements as f32;

        let mut total_thrust = 0.0;
        let mut _total_torque = 0.0;

        for i in 0..num_elements {
            let r = (i as f32 + 0.5) * dr; // Радиус элемента
            let local_collective = self.collective_pitch + self.blade_twist * r;

            // Локальная скорость потока
            let tangential_velocity = omega * r;
            let axial_velocity = -inflow_velocity.y; // Вертикальная составляющая

            // Resultant velocity at blade element
            let resultant_vel = (tangential_velocity * tangential_velocity
                + axial_velocity * axial_velocity)
                .sqrt();

            if resultant_vel < 1.0 {
                continue;
            }

            // Угол атаки
            let inflow_angle = (axial_velocity / tangential_velocity).atan();
            let angle_of_attack = local_collective - inflow_angle;

            // Коэффициент подъёмной силы (линейная аппроксимация)
            let cl = self.airfoil_lift_slope * angle_of_attack;

            // Подъёмная сила элемента лопасти
            let dynamic_pressure = 0.5 * air_density * resultant_vel * resultant_vel;
            let d_l = dynamic_pressure * cl * self.chord_length * dr;

            // Учёт циклического шага (азимутальное усреднение)
            let azimuthal_factor = 1.0
                + 0.3 * (self.cyclic_pitch_longitudinal.abs() + self.cyclic_pitch_lateral.abs());

            total_thrust += d_l * azimuthal_factor * self.blade_count as f32;

            // Индуктивное сопротивление
            let cd = 0.01 + cl * cl / (PI * self.blade_count as f32); // Упрощённая модель
            let d_d = dynamic_pressure * cd * self.chord_length * dr;
            _total_torque += d_d * r * self.blade_count as f32;
        }

        // Применяем фактор потерь на концах
        total_thrust *= self.tip_loss_factor;

        // Вектор тяги с учётом наклона ротора
        let thrust_body = Vector3::new(
            -total_thrust * self.rotor_tilt_x.sin(),
            -total_thrust * self.rotor_tilt_x.cos() * self.rotor_tilt_y.cos(),
            -total_thrust * self.rotor_tilt_y.sin(),
        );

        thrust_body
    }

    /// Расчёт крутящего момента от ротора
    pub fn calculate_torque(&self, air_density: f32, inflow_velocity: Vector3<f32>) -> f32 {
        let omega = self.current_rpm * 2.0 * PI / 60.0;
        let tip_speed = omega * self.radius;

        if tip_speed < 1.0 {
            return 0.0;
        }

        let num_elements = 20;
        let dr = self.radius / num_elements as f32;
        let mut _total_torque = 0.0;

        for i in 0..num_elements {
            let r = (i as f32 + 0.5) * dr;
            let local_collective = self.collective_pitch + self.blade_twist * r;

            let tangential_velocity = omega * r;
            let axial_velocity = -inflow_velocity.y;
            let resultant_vel = (tangential_velocity * tangential_velocity
                + axial_velocity * axial_velocity)
                .sqrt();

            if resultant_vel < 1.0 {
                continue;
            }

            let inflow_angle = (axial_velocity / tangential_velocity).atan();
            let angle_of_attack = local_collective - inflow_angle;
            let cl = self.airfoil_lift_slope * angle_of_attack;

            let dynamic_pressure = 0.5 * air_density * resultant_vel * resultant_vel;
            let cd = 0.01 + cl * cl / (PI * self.blade_count as f32);
            let d_d = dynamic_pressure * cd * self.chord_length * dr;

            _total_torque += d_d * r * self.blade_count as f32;
        }

        _total_torque * self.tip_loss_factor
    }

    /// Обновление скорости вращения ротора
    pub fn update_rotor_speed(
        &mut self,
        engine_torque: f32,
        rotor_torque: f32,
        moment_of_inertia: f32,
        dt: f32,
    ) {
        let omega = self.current_rpm * 2.0 * PI / 60.0;
        let angular_acceleration = (engine_torque - rotor_torque) / moment_of_inertia;

        let new_omega = omega + angular_acceleration * dt;
        self.current_rpm = new_omega * 60.0 / (2.0 * PI);

        // Ограничиваем RPM
        self.current_rpm = self.current_rpm.clamp(0.0, self.target_rpm * 1.1);
    }
}

/// Конфигурация хвостового ротора
#[derive(Debug, Clone)]
pub struct TailRotor {
    pub radius: f32,
    pub blade_count: u32,
    pub chord_length: f32,
    pub distance_from_cg: Vector3<f32>, // Позиция относительно центра масс
    pub rotational_speed: f32,
    pub pitch_angle: f32, // Управляемый шаг
    pub current_rpm: f32,
    pub target_rpm: f32,
    pub tilt_angle: f32, // Наклон для создания вертикальной компоненты
}

impl TailRotor {
    pub fn new(radius: f32, distance_from_cg: Vector3<f32>) -> Self {
        Self {
            radius,
            blade_count: 4,
            chord_length: 0.15,
            distance_from_cg,
            rotational_speed: 150.0,
            pitch_angle: 0.0,
            current_rpm: 0.0,
            target_rpm: 1200.0,
            tilt_angle: 0.0,
        }
    }

    /// Расчёт боковой тяги хвостового ротора
    pub fn calculate_thrust(&self, air_density: f32, _main_rotor_torque: f32) -> Vector3<f32> {
        let omega = self.current_rpm * 2.0 * PI / 60.0;
        let tip_speed = omega * self.radius;

        if tip_speed < 1.0 {
            return Vector3::zeros();
        }

        // Упрощённая модель: тяга пропорциональна квадрату скорости и углу шага
        let dynamic_pressure = 0.5 * air_density * tip_speed * tip_speed;
        let disk_area = PI * self.radius * self.radius;

        // Коэффициент тяги зависит от угла шага
        let thrust_coefficient = 0.1 * self.pitch_angle.sin();

        let thrust_magnitude =
            dynamic_pressure * disk_area * thrust_coefficient * self.blade_count as f32;

        // Направление тяги (перпендикулярно хвостовой балке)
        let thrust_direction = Vector3::new(1.0, 0.0, 0.0); // Вправо по оси X

        thrust_direction * thrust_magnitude
    }

    /// Расчёт крутящего момента от хвостового ротора
    pub fn calculate_torque(&self, thrust: Vector3<f32>) -> Vector3<f32> {
        self.distance_from_cg.cross(&thrust)
    }
}

/// Модель турбовального двигателя
#[derive(Debug, Clone)]
pub struct TurboshaftEngine {
    pub max_power: f32,           // Максимальная мощность (Вт)
    pub max_torque: f32,          // Максимальный крутящий момент (Н*м)
    pub idle_rpm: f32,            // Обороты холостого хода
    pub max_rpm: f32,             // Максимальные обороты
    pub fuel_flow_rate: f32,      // Расход топлива (кг/с)
    pub current_fuel: f32,        // Текущее топливо (кг)
    pub max_fuel: f32,            // Максимальное топливо (кг)
    pub throttle_position: f32,   // Положение дросселя (0.0-1.0)
    pub governor_enabled: bool,   // Автономный регулятор оборотов
    pub engine_temperature: f32,  // Температура двигателя (°C)
    pub ambient_temperature: f32, // Температура окружающей среды
    pub oil_pressure: f32,        // Давление масла (бар)
    pub is_running: bool,         // Двигатель запущен
}

impl TurboshaftEngine {
    pub fn new(max_power: f32, max_fuel: f32) -> Self {
        Self {
            max_power,
            max_torque: max_power / 400.0, // Приблизительно при 400 рад/с
            idle_rpm: 100.0,
            max_rpm: 450.0,
            fuel_flow_rate: 0.05,
            current_fuel: max_fuel,
            max_fuel,
            throttle_position: 0.0,
            governor_enabled: true,
            engine_temperature: 20.0,
            ambient_temperature: 20.0,
            oil_pressure: 0.0,
            is_running: false,
        }
    }

    /// Запуск двигателя
    pub fn start_engine(&mut self) -> bool {
        if self.current_fuel > 0.0 && !self.is_running {
            self.is_running = true;
            self.throttle_position = 0.1;
            self.oil_pressure = 3.0; // Нормальное давление
            true
        } else {
            false
        }
    }

    /// Остановка двигателя
    pub fn stop_engine(&mut self) {
        self.is_running = false;
        self.throttle_position = 0.0;
        self.oil_pressure = 0.0;
    }

    /// Расчёт доступного крутящего момента
    pub fn calculate_available_torque(&self, current_rpm: f32) -> f32 {
        if !self.is_running || self.current_fuel <= 0.0 {
            return 0.0;
        }

        // Governor пытается поддерживать целевые обороты
        let target_rpm = if self.governor_enabled {
            self.max_rpm * 0.95
        } else {
            self.max_rpm * self.throttle_position
        };

        // Пропорционально-интегральный регулятор
        let rpm_error = target_rpm - current_rpm;
        let governor_output = (rpm_error / self.max_rpm).clamp(-0.2, 1.0);

        let effective_throttle = if self.governor_enabled {
            governor_output.max(self.throttle_position)
        } else {
            self.throttle_position
        };

        // Доступный момент зависит от оборотов и положения дросселя
        let torque_curve =
            1.0 - ((current_rpm - self.max_rpm * 0.7).abs() / (self.max_rpm * 0.3)).max(0.0);

        self.max_torque * effective_throttle * torque_curve
    }

    /// Обновление состояния двигателя
    pub fn update(&mut self, dt: f32, load_torque: f32) {
        if !self.is_running {
            // Остывание двигателя
            self.engine_temperature +=
                (self.ambient_temperature - self.engine_temperature) * 0.01 * dt;
            return;
        }

        // Расход топлива
        let fuel_consumption = self.fuel_flow_rate * self.throttle_position * dt;
        self.current_fuel = (self.current_fuel - fuel_consumption).max(0.0);

        if self.current_fuel <= 0.0 {
            self.stop_engine();
            return;
        }

        // Нагрев двигателя под нагрузкой
        let load_factor = load_torque / self.max_torque;
        let target_temp =
            self.ambient_temperature + 150.0 * load_factor + 50.0 * self.throttle_position;
        self.engine_temperature += (target_temp - self.engine_temperature) * 0.05 * dt;
        self.engine_temperature = self
            .engine_temperature
            .clamp(self.ambient_temperature, 900.0);

        // Проверка перегрева
        if self.engine_temperature > 850.0 {
            // Автоматическая остановка при критическом перегреве
            self.stop_engine();
        }
    }
}

/// Система управления вертолётом
#[derive(Debug, Clone)]
pub struct HelicopterControls {
    pub collective: f32,              // Общий шаг (0.0-1.0)
    pub cyclic_longitudinal: f32,     // Циклик вперёд/назад (-1.0-1.0)
    pub cyclic_lateral: f32,          // Циклик влево/вправо (-1.0-1.0)
    pub tail_rotor_pedals: f32,       // Педали хвостового ротора (-1.0-1.0)
    pub throttle: f32,                // Дроссель двигателя (0.0-1.0)
    pub autopilot_enabled: bool,      // Автопилот включён
    pub stability_augmentation: bool, // Система стабилизации
}

impl HelicopterControls {
    pub fn new() -> Self {
        Self {
            collective: 0.0,
            cyclic_longitudinal: 0.0,
            cyclic_lateral: 0.0,
            tail_rotor_pedals: 0.0,
            throttle: 0.0,
            autopilot_enabled: false,
            stability_augmentation: true,
        }
    }

    /// Применение входных данных с фильтрацией и ограничениями
    pub fn apply_input(
        &mut self,
        collective_input: f32,
        longitudinal_input: f32,
        lateral_input: f32,
        pedal_input: f32,
        throttle_input: f32,
        dt: f32,
        smoothing_factor: f32,
    ) {
        // Плавное изменение управляющих сигналов
        self.collective = self.lerp_smooth(
            self.collective,
            collective_input.clamp(0.0, 1.0),
            smoothing_factor,
            dt,
        );
        self.cyclic_longitudinal = self.lerp_smooth(
            self.cyclic_longitudinal,
            longitudinal_input.clamp(-1.0, 1.0),
            smoothing_factor,
            dt,
        );
        self.cyclic_lateral = self.lerp_smooth(
            self.cyclic_lateral,
            lateral_input.clamp(-1.0, 1.0),
            smoothing_factor,
            dt,
        );
        self.tail_rotor_pedals = self.lerp_smooth(
            self.tail_rotor_pedals,
            pedal_input.clamp(-1.0, 1.0),
            smoothing_factor,
            dt,
        );
        self.throttle = self.lerp_smooth(
            self.throttle,
            throttle_input.clamp(0.0, 1.0),
            smoothing_factor,
            dt,
        );
    }

    fn lerp_smooth(&self, current: f32, target: f32, factor: f32, dt: f32) -> f32 {
        current + (target - current) * factor * dt
    }
}

/// Полная модель вертолёта
#[derive(Clone, Debug)]
pub struct Helicopter {
    // Основные параметры
    pub mass: f32,                      // Масса (кг)
    pub main_rotor_radius: f32,           // Радиус главного ротора
    pub inertia_tensor: Matrix3<f32>,   // Тензор инерции
    pub position: Vector3<f32>,         // Позиция в мире
    pub rotation: UnitQuaternion<f32>,  // Ориентация
    pub velocity: Vector3<f32>,         // Линейная скорость
    pub angular_velocity: Vector3<f32>, // Угловая скорость

    // Компоненты
    pub main_rotor: MainRotor,
    pub tail_rotor: TailRotor,
    pub engine: TurboshaftEngine,
    pub controls: HelicopterControls,

    // Аэродинамика фюзеляжа
    pub fuselage_drag_coefficient: Vector3<f32>, // Коэффициенты сопротивления по осям
    pub fuselage_reference_area: f32,            // Характерная площадь
    pub side_area: f32,                          // Площадь боковой проекции
    pub top_area: f32,                           // Площадь верхней проекции

    // Внешние условия
    pub air_density: f32,            // Плотность воздуха (кг/м³)
    pub wind_velocity: Vector3<f32>, // Скорость ветра
    pub gravity: f32,                // Ускорение свободного падения

    // Состояние
    pub is_on_ground: bool,                  // На земле ли вертолёт
    pub ground_contact_normal: Vector3<f32>, // Нормаль поверхности
    pub rotor_moment_of_inertia: f32,        // Момент инерции ротора

    // Внешняя подвеска (грузовой крюк)
    pub has_cargo_hook: bool,                 // Наличие грузового крюка
    pub cargo_hook_offset: Vector3<f32>,      // Позиция крюка относительно ЦМ
    pub cargo_hook_force: Vector3<f32>,       // Сила от подвешенного груза
    pub cargo_mass: f32,                      // Текущая масса груза на подвеске
    pub max_cargo_mass: f32,                  // Максимальная масса груза
    pub cargo_cable_length: f32,              // Длина троса
    pub cargo_position: Option<Vector3<f32>>, // Позиция груза (если есть)
    pub cargo_velocity: Vector3<f32>,         // Скорость груза
    pub is_cargo_attached: bool,              // Прицеплен ли груз

    // Интеграция с PhysicsWorld
    pub chassis_body_id: Option<usize>,
    pub body_index: Option<usize>,
}

impl Helicopter {
    pub fn new(position: Vector3<f32>) -> Self {
        // Параметры для лёгкого вертолёта типа Robinson R44
        let mass = 1100.0; // кг
        let main_rotor_radius = 5.0;
        let inertia_tensor = Matrix3::new(
            1500.0, 0.0, 0.0, // Ixx
            0.0, 2000.0, 0.0, // Iyy
            0.0, 0.0, 2500.0, // Izz
        );

        let main_rotor = MainRotor::new(main_rotor_radius, 2); // Радиус 5м, 2 лопасти
        let tail_rotor = TailRotor::new(0.8, Vector3::new(0.0, 0.0, -6.0)); // 6м от ЦМ

        Self {
            mass,
            main_rotor_radius,
            inertia_tensor,
            position,
            rotation: UnitQuaternion::identity(),
            velocity: Vector3::zeros(),
            angular_velocity: Vector3::zeros(),
            main_rotor,
            tail_rotor,
            engine: TurboshaftEngine::new(200000.0, 100.0), // 200 кВт, 100 кг топлива
            controls: HelicopterControls::new(),
            fuselage_drag_coefficient: Vector3::new(0.8, 1.2, 0.9),
            fuselage_reference_area: 2.5,
            side_area: 8.0,
            top_area: 15.0,
            air_density: 1.225,
            wind_velocity: Vector3::zeros(),
            gravity: 9.81,
            is_on_ground: true,
            ground_contact_normal: Vector3::y(),
            rotor_moment_of_inertia: 50.0,
            // Внешняя подвеска (по умолчанию отключена для лёгких вертолётов)
            has_cargo_hook: false,
            cargo_hook_offset: Vector3::new(0.0, -0.5, 0.0),
            cargo_hook_force: Vector3::zeros(),
            cargo_mass: 0.0,
            max_cargo_mass: 0.0,
            cargo_cable_length: 0.0,
            cargo_position: None,
            cargo_velocity: Vector3::zeros(),
            is_cargo_attached: false,
            chassis_body_id: None,
            body_index: None,
        }
    }

    /// Создание физического тела для вертолёта
    pub fn create_physics_body(&self) -> crate::physics::RigidBody {
        // Размеры коллайдера зависят от типа вертолёта
        let collider_size = match self.main_rotor_radius {
            r if r < 4.0 => Vector3::new(2.5, 1.0, 5.0),   // Лёгкий вертолёт
            r if r < 8.0 => Vector3::new(3.5, 1.5, 7.0),   // Средний вертолёт
            r if r < 12.0 => Vector3::new(4.5, 2.0, 9.0),  // Тяжёлый вертолёт
            _ => Vector3::new(6.0, 2.5, 12.0),             // Грузовой вертолёт
        };
        crate::physics::RigidBody::new_box(self.position, self.mass, collider_size)
    }

    /// Установить ID тела в физическом мире
    pub fn set_chassis_body_id(&mut self, id: usize) {
        self.chassis_body_id = Some(id);
    }

    /// Получить ID тела из физического мира
    pub fn get_chassis_body_id(&self) -> Option<usize> {
        self.chassis_body_id
    }

    /// Создание вертолёта с конфигурацией и внешней подвеской
    pub fn with_config(position: Vector3<f32>, config: HelicopterConfig) -> Self {
        let main_rotor = MainRotor::new(config.main_rotor_radius, config.main_rotor_blade_count);
        let tail_rotor = TailRotor::new(
            config.tail_rotor_radius,
            Vector3::new(0.0, 0.0, -config.tail_rotor_distance),
        );

        let inertia_tensor = Matrix3::new(
            config.inertia_xx,
            0.0,
            0.0,
            0.0,
            config.inertia_yy,
            0.0,
            0.0,
            0.0,
            config.inertia_zz,
        );

        let total_mass = config.mass_empty + config.fuel_capacity * 0.5; // Половина бака

        Self {
            mass: total_mass,
            main_rotor_radius: config.main_rotor_radius,
            inertia_tensor,
            position,
            rotation: UnitQuaternion::identity(),
            velocity: Vector3::zeros(),
            angular_velocity: Vector3::zeros(),
            main_rotor,
            tail_rotor,
            engine: match config.engine_type {
                EngineType::Turboshaft => {
                    TurboshaftEngine::new(config.max_engine_power, config.fuel_capacity)
                }
                EngineType::Piston => {
                    TurboshaftEngine::new(config.max_engine_power, config.fuel_capacity)
                }
                EngineType::Electric => TurboshaftEngine::new(config.max_engine_power, 0.0), // Электрический без топлива
            },
            controls: HelicopterControls::new(),
            fuselage_drag_coefficient: Vector3::new(
                config.drag_coefficient_x,
                config.drag_coefficient_y,
                config.drag_coefficient_z,
            ),
            fuselage_reference_area: config.fuselage_width * config.fuselage_height,
            side_area: config.fuselage_length * config.fuselage_height,
            top_area: config.fuselage_length * config.fuselage_width,
            air_density: 1.225,
            wind_velocity: Vector3::zeros(),
            gravity: 9.81,
            is_on_ground: true,
            ground_contact_normal: Vector3::y(),
            rotor_moment_of_inertia: config.mass_empty
                * config.main_rotor_radius
                * config.main_rotor_radius
                * 0.1,
            // Внешняя подвеска из конфигурации
            has_cargo_hook: config.has_cargo_hook,
            cargo_hook_offset: config.cargo_hook_offset,
            cargo_hook_force: Vector3::zeros(),
            cargo_mass: 0.0,
            max_cargo_mass: config.max_cargo_mass,
            cargo_cable_length: config.cargo_cable_length,
            cargo_position: None,
            cargo_velocity: Vector3::zeros(),
            is_cargo_attached: false,
            chassis_body_id: None,
            body_index: None,
        }
    }

    /// Основной шаг симуляции физики с защитой от NaN/Inf
    pub fn update(&mut self, dt: f32) {
        // Защита от некорректного dt
        if !dt.is_finite() || dt <= 0.0 {
            tracing::warn!(target: "physics", "Invalid dt in helicopter physics: {}, using default", dt);
            return;
        }

        // Проверка текущего состояния на NaN/Inf перед началом обновления
        if !self.validate_state() {
            tracing::warn!(target: "physics", "Invalid state detected in helicopter, resetting to safe state");
            self.reset_to_safe_state();
            return;
        }

        // 1. Обновляем состояние двигателя
        self.engine.update(
            dt,
            self.main_rotor
                .calculate_torque(self.air_density, self.velocity),
        );

        // 2. Применяем управление к компонентам
        self.apply_controls();

        // 3. Рассчитываем силы и моменты (включая груз на подвеске)
        let (forces, torques) = self.calculate_forces_and_torques();

        // 4. Обновляем физику груза на внешней подвеске
        if self.is_cargo_attached && self.has_cargo_hook {
            self.update_cargo_physics(dt);
        }

        // 5. Интегрируем уравнения движения (Symplectic Euler)
        self.integrate_motion(forces, torques, dt);

        // 6. Обновляем скорость вращения ротора
        let available_torque = self
            .engine
            .calculate_available_torque(self.main_rotor.current_rpm);
        let rotor_torque = self
            .main_rotor
            .calculate_torque(self.air_density, self.velocity);
        self.main_rotor.update_rotor_speed(
            available_torque,
            rotor_torque,
            self.rotor_moment_of_inertia,
            dt,
        );

        // 7. Обновляем хвостовой ротор
        self.tail_rotor.current_rpm = self.main_rotor.current_rpm * 3.0; // Передаточное отношение

        // Финальная проверка состояния после обновления
        if !self.validate_state() {
            tracing::error!(target: "physics", "State became invalid after helicopter update, resetting");
            self.reset_to_safe_state();
        }
    }

    /// Validate that all physics state values are finite (not NaN or Inf)
    pub fn validate_state(&self) -> bool {
        self.position.x.is_finite()
            && self.position.y.is_finite()
            && self.position.z.is_finite()
            && self.velocity.x.is_finite()
            && self.velocity.y.is_finite()
            && self.velocity.z.is_finite()
            && self.angular_velocity.x.is_finite()
            && self.angular_velocity.y.is_finite()
            && self.angular_velocity.z.is_finite()
            && self.rotation.i.is_finite()
            && self.rotation.j.is_finite()
            && self.rotation.k.is_finite()
            && self.rotation.w.is_finite()
            && self.main_rotor.current_rpm.is_finite()
            && self.tail_rotor.current_rpm.is_finite()
    }

    /// Reset helicopter to a safe state when invalid values are detected
    pub fn reset_to_safe_state(&mut self) {
        tracing::info!(target: "physics", "Resetting helicopter to safe state");
        self.velocity = nalgebra::Vector3::zeros();
        self.angular_velocity = nalgebra::Vector3::zeros();
        self.position.y = self.position.y.max(1.0); // Ensure we're above ground
        self.main_rotor.current_rpm = self.main_rotor.idle_rpm;
        self.tail_rotor.current_rpm = self.tail_rotor.target_rpm;
        self.controls = HelicopterControls::new();
    }

    /// Обновление физики с интеграцией в PhysicsWorld
    pub fn physics_update(&mut self, dt: f32, physics_world: &mut crate::physics::PhysicsWorld) {
        // Выполняем основной update вертолёта
        self.update(dt);

        // Синхронизируем состояние тела шасси с PhysicsWorld
        if let Some(chassis_id) = self.chassis_body_id {
            if let Some(body) = physics_world.get_body_mut(chassis_id) {
                // Синхронизируем позицию и ориентацию
                body.position = self.position;
                body.rotation = self.rotation;
                body.velocity = self.velocity;
                body.angular_velocity = self.angular_velocity;

                // Применяем внешние силы (ветер, подъемная сила и т.д.)
                // Силы уже применены в update(), здесь только синхронизация
            }
        }
    }

    /// Обновление физики груза на внешней подвеске
    fn update_cargo_physics(&mut self, dt: f32) {
        if !self.is_cargo_attached || self.cargo_mass <= 0.0 {
            return;
        }

        // Позиция точки подвески в мире
        let hook_world_pos =
            self.position + self.rotation.transform_vector(&self.cargo_hook_offset);

        if let Some(cargo_pos) = self.cargo_position {
            // Вектор от крюка к грузу
            let cable_vector = cargo_pos - hook_world_pos;
            let cable_length = cable_vector.norm();

            // Сила натяжения троса (пружинная модель с демпфированием)
            let spring_constant = 5000.0; // Жёсткость троса
            let damping_constant = 500.0; // Демпфирование

            // Растяжение троса
            let stretch = (cable_length - self.cargo_cable_length).max(0.0);
            let spring_force = spring_constant * stretch;

            // Направление силы натяжения
            let tension_direction = if cable_length > 0.001 {
                cable_vector.normalize()
            } else {
                Vector3::y()
            };

            // Скорость груза относительно точки подвески
            let hook_velocity =
                self.velocity + self.angular_velocity.cross(&self.cargo_hook_offset);
            let relative_velocity = self.cargo_velocity - hook_velocity;
            let damping_force = damping_constant * relative_velocity.dot(&tension_direction);

            // Общая сила натяжения
            let tension = (spring_force + damping_force).max(0.0);
            let tension_force = tension * tension_direction;

            // Сила тяжести на груз
            let gravity_force = Vector3::new(0.0, -self.cargo_mass * self.gravity, 0.0);

            // Ускорение груза
            let total_force = gravity_force + tension_force;
            let acceleration = total_force / self.cargo_mass;

            // Интегрирование скорости и позиции груза
            self.cargo_velocity += acceleration * dt;
            self.cargo_velocity *= 0.99; // Небольшое затухание

            // Ограничение длины троса
            let new_cargo_pos = cargo_pos + self.cargo_velocity * dt;
            let new_cable_vector = new_cargo_pos - hook_world_pos;
            let new_cable_length = new_cable_vector.norm();

            if new_cable_length > self.cargo_cable_length {
                // Корректировка позиции для соблюдения ограничения длины троса
                let corrected_pos =
                    hook_world_pos + new_cable_vector.normalize() * self.cargo_cable_length;
                self.cargo_position = Some(corrected_pos);

                // Корректировка скорости (проекция на перпендикуляр к тросу)
                let tangent = self.cargo_velocity
                    - tension_direction * self.cargo_velocity.dot(&tension_direction);
                self.cargo_velocity = tangent * 0.95;
            } else {
                self.cargo_position = Some(new_cargo_pos);
            }

            // Сила реакции на вертолёт от груза (третий закон Ньютона)
            self.cargo_hook_force = -tension_force;
        }
    }

    /// Прицепить груз к внешней подвеске
    pub fn attach_cargo(&mut self, cargo_mass: f32, initial_position: Vector3<f32>) -> bool {
        if !self.has_cargo_hook || cargo_mass > self.max_cargo_mass || cargo_mass <= 0.0 {
            return false;
        }

        self.cargo_mass = cargo_mass;
        self.cargo_position = Some(initial_position);
        self.cargo_velocity = Vector3::zeros();
        self.is_cargo_attached = true;
        self.mass += cargo_mass; // Учитываем массу груза

        true
    }

    /// Отцепить груз с внешней подвески
    pub fn detach_cargo(&mut self) -> Option<Vector3<f32>> {
        if !self.is_cargo_attached {
            return None;
        }

        let cargo_pos = self.cargo_position;
        self.cargo_mass = 0.0;
        self.cargo_position = None;
        self.cargo_velocity = Vector3::zeros();
        self.is_cargo_attached = false;
        self.mass -= self.cargo_mass;
        self.cargo_hook_force = Vector3::zeros();

        cargo_pos
    }

    /// Получить текущую позицию груза
    pub fn get_cargo_position(&self) -> Option<Vector3<f32>> {
        if self.is_cargo_attached {
            self.cargo_position
        } else {
            None
        }
    }

    /// Сила от груза на крюке (для расчёта сил)

    /// Применение управляющих воздействий
    fn apply_controls(&mut self) {
        // Преобразуем входные данные в физические параметры
        let max_collective_pitch = 0.26; // ~15 градусов
        let max_cyclic_pitch = 0.17; // ~10 градусов
        let max_tail_pitch = 0.35; // ~20 градусов

        self.main_rotor.collective_pitch = self.controls.collective * max_collective_pitch;
        self.main_rotor.cyclic_pitch_longitudinal =
            self.controls.cyclic_longitudinal * max_cyclic_pitch;
        self.main_rotor.cyclic_pitch_lateral = self.controls.cyclic_lateral * max_cyclic_pitch;
        self.main_rotor.rotor_tilt_x = self.controls.cyclic_longitudinal * 0.1; // Небольшой наклон ротора
        self.main_rotor.rotor_tilt_y = self.controls.cyclic_lateral * 0.1;

        self.tail_rotor.pitch_angle = self.controls.tail_rotor_pedals * max_tail_pitch;
        self.engine.throttle_position = self.controls.throttle;
    }

    /// Расчёт всех сил и моментов действующих на вертолёт
    fn calculate_forces_and_torques(&self) -> (Vector3<f32>, Vector3<f32>) {
        let mut total_force = Vector3::zeros();
        let mut total_torque = Vector3::zeros();

        // 1. Гравитация (с учётом массы груза)
        let gravity_force = Vector3::new(0.0, -self.mass * self.gravity, 0.0);
        total_force += gravity_force;

        // 2. Подъёмная сила главного ротора
        let body_velocity = self.rotation.inverse().transform_vector(&self.velocity);
        let main_rotor_thrust = self
            .main_rotor
            .calculate_thrust(self.air_density, body_velocity);
        let main_rotor_thrust_world = self.rotation.transform_vector(&main_rotor_thrust);
        total_force += main_rotor_thrust_world;

        // Крутящий момент от главного ротора (реактивный)
        let main_rotor_torque = self
            .main_rotor
            .calculate_torque(self.air_density, body_velocity);
        total_torque += Vector3::new(0.0, -main_rotor_torque, 0.0); // Против направления вращения

        // 3. Тяга хвостового ротора
        let tail_thrust = self
            .tail_rotor
            .calculate_thrust(self.air_density, main_rotor_torque);
        let tail_thrust_world = self.rotation.transform_vector(&tail_thrust);
        total_force += tail_thrust_world;

        // Момент от хвостового ротора
        let tail_torque = self.tail_rotor.calculate_torque(tail_thrust_world);
        total_torque += tail_torque;

        // 4. Аэродинамическое сопротивление фюзеляжа
        let relative_velocity = self.velocity - self.wind_velocity;
        let drag_force = self.calculate_fuselage_drag(relative_velocity);
        total_force += drag_force;

        // 5. Аэродинамический момент демпфирования
        let damping_torque = self.calculate_aerodynamic_damping();
        total_torque += damping_torque;

        // 6. Контакт с землёй (если на земле)
        if self.is_on_ground {
            let (ground_force, ground_torque) = self.calculate_ground_contact();
            total_force += ground_force;
            total_torque += ground_torque;
        }

        // 7. Сила от груза на внешней подвеске (реактивная сила на вертолёт)
        if self.cargo_hook_force.norm() > 0.001 {
            // Применяем силу от груза к точке подвески
            total_force += self.cargo_hook_force;

            // Момент от силы груза относительно ЦМ
            let hook_world_offset = self.rotation.transform_vector(&self.cargo_hook_offset);
            let cargo_torque = hook_world_offset.cross(&self.cargo_hook_force);
            total_torque += cargo_torque;
        }

        (total_force, total_torque)
    }

    /// Расчёт аэродинамического сопротивления фюзеляжа
    fn calculate_fuselage_drag(&self, relative_velocity: Vector3<f32>) -> Vector3<f32> {
        let speed_squared = relative_velocity.magnitude_squared();
        if speed_squared < 0.01 {
            return Vector3::zeros();
        }

        let _speed = speed_squared.sqrt();
        let direction = -relative_velocity.normalize();

        // Сопротивление по разным осям
        let drag_x = 0.5
            * self.air_density
            * speed_squared
            * self.fuselage_drag_coefficient.x
            * self.side_area;
        let drag_y = 0.5
            * self.air_density
            * speed_squared
            * self.fuselage_drag_coefficient.y
            * self.top_area;
        let drag_z = 0.5
            * self.air_density
            * speed_squared
            * self.fuselage_drag_coefficient.z
            * self.fuselage_reference_area;

        Vector3::new(
            direction.x * drag_x,
            direction.y * drag_y,
            direction.z * drag_z,
        )
    }

    /// Аэродинамическое демпфирование углового движения
    fn calculate_aerodynamic_damping(&self) -> Vector3<f32> {
        let damping_coeffs = Vector3::new(500.0, 800.0, 600.0); // Коэффициенты демпфирования
        -Vector3::new(
            self.angular_velocity.x * damping_coeffs.x,
            self.angular_velocity.y * damping_coeffs.y,
            self.angular_velocity.z * damping_coeffs.z,
        )
    }

    /// Расчёт сил реакции опоры
    fn calculate_ground_contact(&self) -> (Vector3<f32>, Vector3<f32>) {
        if !self.is_on_ground {
            return (Vector3::zeros(), Vector3::zeros());
        }

        // Простая модель: сила реакции компенсирует часть веса
        let compression = 0.1; // Условное сжатие шасси
        let spring_constant = 50000.0;
        let damping_constant = 5000.0;

        let normal_force = spring_constant * compression
            - damping_constant * self.velocity.dot(&self.ground_contact_normal);
        let ground_force = self.ground_contact_normal * normal_force.max(0.0);

        // Момент от силы реакции (приложенной к точкам шасси)
        let skid_positions = vec![
            Vector3::new(-1.5, -0.5, 2.0),
            Vector3::new(1.5, -0.5, 2.0),
            Vector3::new(-1.5, -0.5, -2.0),
            Vector3::new(1.5, -0.5, -2.0),
        ];

        let mut ground_torque = Vector3::zeros();
        for skid_pos in skid_positions {
            let world_skid_pos = self.rotation.transform_vector(&skid_pos);
            let force_vec = ground_force / 4.0;
            ground_torque += world_skid_pos.cross(&force_vec);
        }

        (ground_force, ground_torque)
    }

    /// Интегрирование уравнений движения
    fn integrate_motion(&mut self, forces: Vector3<f32>, torques: Vector3<f32>, dt: f32) {
        // Линейное движение
        let linear_acceleration = forces / self.mass;
        self.velocity += linear_acceleration * dt;
        self.position += self.velocity * dt;

        // Угловое движение
        let angular_acceleration = self
            .inertia_tensor
            .try_inverse()
            .unwrap_or(Matrix3::zeros())
            * torques;
        self.angular_velocity += angular_acceleration * dt;

        // Обновление ориентации через exponential map (стандарт для игровых движков)
        let angle = self.angular_velocity.magnitude() * dt;
        if angle > 1e-6 {
            let axis = nalgebra::Unit::new_normalize(self.angular_velocity);
            let delta_rotation = UnitQuaternion::from_axis_angle(&axis, angle);
            self.rotation = delta_rotation * self.rotation;
            // Ренормализация для предотвращения накопления ошибок
            self.rotation.renormalize();
        }

        // Проверка контакта с землёй
        self.is_on_ground = self.position.y < 0.5;
        if self.is_on_ground && self.position.y < 0.0 {
            self.position.y = 0.0;
            self.velocity.y = self.velocity.y.max(0.0);
        }
    }

    /// Получение текущего состояния для рендеринга
    pub fn get_state(&self) -> HelicopterState {
        HelicopterState {
            position: self.position,
            rotation: self.rotation,
            velocity: self.velocity,
            angular_velocity: self.angular_velocity,
            main_rotor_rpm: self.main_rotor.current_rpm,
            tail_rotor_rpm: self.tail_rotor.current_rpm,
            collective: self.controls.collective,
            engine_running: self.engine.is_running,
            fuel_level: self.engine.current_fuel / self.engine.max_fuel,
            altitude: self.position.y,
            airspeed: self.velocity.magnitude(),
        }
    }
}

/// Состояние вертолёта для передачи в другие системы
#[derive(Debug, Clone)]
pub struct HelicopterState {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub velocity: Vector3<f32>,
    pub angular_velocity: Vector3<f32>,
    pub main_rotor_rpm: f32,
    pub tail_rotor_rpm: f32,
    pub collective: f32,
    pub engine_running: bool,
    pub fuel_level: f32,
    pub altitude: f32,
    pub airspeed: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helicopter_creation() {
        let heli = Helicopter::new(Vector3::new(0.0, 10.0, 0.0));
        assert_eq!(heli.mass, 1100.0);
        assert!(heli.is_on_ground);
        assert!(!heli.engine.is_running);
    }

    #[test]
    fn test_engine_start() {
        let mut heli = Helicopter::new(Vector3::new(0.0, 10.0, 0.0));
        assert!(heli.engine.start_engine());
        assert!(heli.engine.is_running);
        assert!(heli.engine.oil_pressure > 0.0);
    }

    #[test]
    fn test_main_rotor_thrust() {
        let rotor = MainRotor::new(5.0, 2);
        let thrust = rotor.calculate_thrust(1.225, Vector3::zeros());
        // Тяга должна быть направлена вверх (отрицательная Y в локальных координатах)
        assert!(thrust.y < 0.0);
    }

    #[test]
    fn test_physics_update() {
        let mut heli = Helicopter::new(Vector3::new(0.0, 10.0, 0.0));
        heli.engine.start_engine();
        heli.controls.collective = 0.5;
        heli.controls.throttle = 0.8;

        let initial_altitude = heli.position.y;
        heli.update(0.016); // Один кадр при 60 FPS

        // Вертолёт должен начать подниматься при достаточном коллективе
        assert!(heli.main_rotor.current_rpm > 0.0);
    }
}
