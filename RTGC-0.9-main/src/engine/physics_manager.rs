//! Менеджер физики - инкапсуляция физической подсистемы
//!
//! Этот модуль управляет всеми физическими объектами и симуляцией,
//! предоставляя контролируемый интерфейс для взаимодействия с физическим миром.

use crate::error::EngineError;
use crate::physics::{Helicopter, PhysicsWorld, TrackedVehicle, Vehicle};
use tracing::warn;

/// Менеджер физических объектов
pub struct PhysicsManager {
    /// Физический мир
    pub physics_world: PhysicsWorld,

    /// Активное транспортное средство (колёсное)
    vehicle: Option<Vehicle>,

    /// Активный вертолёт
    helicopter: Option<Helicopter>,

    /// Активная гусеничная машина
    tracked_vehicle: Option<TrackedVehicle>,

    /// Входы управления для транспортного средства
    vehicle_inputs: VehicleInputs,

    /// Входы управления для гусеничной машины
    tracked_inputs: TrackedVehicleInputs,
}

/// Входы управления транспортным средством
#[derive(Debug, Clone)]
pub struct VehicleInputs {
    pub throttle: f32,
    pub steering: f32,
    pub brake: f32,
}

impl Default for VehicleInputs {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            steering: 0.0,
            brake: 0.0,
        }
    }
}

/// Входы управления гусеничной машиной
#[derive(Debug, Clone)]
pub struct TrackedVehicleInputs {
    pub throttle: f32,
    pub brake: f32,
    pub turn: f32,
}

impl Default for TrackedVehicleInputs {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            brake: 0.0,
            turn: 0.0,
        }
    }
}

impl PhysicsManager {
    /// Создаёт новый менеджер физики
    pub fn new(physics_world: PhysicsWorld) -> Self {
        Self {
            physics_world,
            vehicle: None,
            helicopter: None,
            tracked_vehicle: None,
            vehicle_inputs: VehicleInputs::default(),
            tracked_inputs: TrackedVehicleInputs::default(),
        }
    }

    /// Обновляет физику с фиксированным шагом
    pub fn step(&mut self, dt: f32) -> Result<(), EngineError> {
        // Проверка на NaN/Inf перед шагом симуляции
        if !dt.is_finite() || dt <= 0.0 {
            warn!(target: "physics", "Invalid dt value: {}, skipping physics step", dt);
            return Ok(());
        }

        // Обновление транспортного средства
        if let Some(ref mut vehicle) = self.vehicle {
            let inputs = &self.vehicle_inputs;
            vehicle.set_throttle(inputs.throttle);
            vehicle.set_steering(inputs.steering);
            vehicle.set_brake(inputs.brake);

            if !vehicle.validate_state() {
                warn!(target: "physics", "Vehicle state invalid, resetting");
                vehicle.reset_to_safe_state();
            } else {
                vehicle.update(
                    dt,
                    |_x, _z| 0.0,
                    |_x, _z| crate::world::SurfaceType::default(),
                );
            }
        }

        // Обновление вертолёта
        if let Some(ref mut heli) = self.helicopter {
            if !heli.validate_state() {
                warn!(target: "physics", "Helicopter state invalid, resetting");
                heli.reset_to_safe_state();
            } else {
                heli.update(dt); // Используем базовый update
            }
        }

        // Обновление гусеничной машины
        if let Some(ref mut tracked) = self.tracked_vehicle {
            let inputs = &self.tracked_inputs;
            tracked.set_throttle(inputs.throttle);
            tracked.set_brake(inputs.brake);
            tracked.set_turn(inputs.turn);

            if !tracked.validate_state() {
                warn!(target: "physics", "TrackedVehicle state invalid, resetting");
                tracked.reset_to_safe_state();
            }
        }

        // Шаг физического мира
        self.physics_world.step(dt);

        Ok(())
    }

    /// Устанавливает транспортное средство
    pub fn set_vehicle(&mut self, vehicle: Vehicle) {
        self.vehicle = Some(vehicle);
    }

    /// Устанавливает вертолёт
    pub fn set_helicopter(&mut self, helicopter: Helicopter) {
        self.helicopter = Some(helicopter);
    }

    /// Устанавливает гусеничную машину
    pub fn set_tracked_vehicle(&mut self, tracked: TrackedVehicle) {
        self.tracked_vehicle = Some(tracked);
    }

    /// Получает ссылку на транспортное средство
    pub fn get_vehicle(&self) -> Option<&Vehicle> {
        self.vehicle.as_ref()
    }

    /// Получает мутабельную ссылку на транспортное средство
    pub fn get_vehicle_mut(&mut self) -> Option<&mut Vehicle> {
        self.vehicle.as_mut()
    }

    /// Получает ссылку на вертолёт
    pub fn get_helicopter(&self) -> Option<&Helicopter> {
        self.helicopter.as_ref()
    }

    /// Получает мутабельную ссылку на вертолёт
    pub fn get_helicopter_mut(&mut self) -> Option<&mut Helicopter> {
        self.helicopter.as_mut()
    }

    /// Получает ссылку на гусеничную машину
    pub fn get_tracked_vehicle(&self) -> Option<&TrackedVehicle> {
        self.tracked_vehicle.as_ref()
    }

    /// Устанавливает входы управления транспортным средством
    pub fn set_vehicle_inputs(&mut self, throttle: f32, steering: f32, brake: f32) {
        self.vehicle_inputs = VehicleInputs {
            throttle: throttle.clamp(-1.0, 1.0),
            steering: steering.clamp(-1.0, 1.0),
            brake: brake.clamp(0.0, 1.0),
        };
    }

    /// Устанавливает входы управления гусеничной машиной
    pub fn set_tracked_inputs(&mut self, throttle: f32, brake: f32, turn: f32) {
        self.tracked_inputs = TrackedVehicleInputs {
            throttle: throttle.clamp(-1.0, 1.0),
            brake: brake.clamp(0.0, 1.0),
            turn: turn.clamp(-1.0, 1.0),
        };
    }

    /// Проверяет наличие активного физического объекта
    pub fn has_active_vehicle(&self) -> bool {
        self.vehicle.is_some() || self.helicopter.is_some() || self.tracked_vehicle.is_some()
    }

    /// Сбрасывает все физические объекты
    pub fn clear_all(&mut self) {
        self.vehicle = None;
        self.helicopter = None;
        self.tracked_vehicle = None;
        self.vehicle_inputs = VehicleInputs::default();
        self.tracked_inputs = TrackedVehicleInputs::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_manager_creation() {
        let physics_world = PhysicsWorld::new();
        let manager = PhysicsManager::new(physics_world);

        assert!(!manager.has_active_vehicle());
        assert!(manager.get_vehicle().is_none());
        assert!(manager.get_helicopter().is_none());
    }

    #[test]
    fn test_vehicle_inputs_clamping() {
        let physics_world = PhysicsWorld::new();
        let mut manager = PhysicsManager::new(physics_world);

        // Установка значений за пределами диапазона
        manager.set_vehicle_inputs(2.0, -5.0, 10.0);

        // Проверка клamping
        assert_eq!(manager.vehicle_inputs.throttle, 1.0);
        assert_eq!(manager.vehicle_inputs.steering, -1.0);
        assert_eq!(manager.vehicle_inputs.brake, 1.0);
    }

    #[test]
    fn test_physics_step_with_invalid_dt() {
        let physics_world = PhysicsWorld::new();
        let mut manager = PhysicsManager::new(physics_world);

        // Шаг с NaN должен быть пропущен без ошибки
        let result = manager.step(f32::NAN);
        assert!(result.is_ok());

        // Шаг с отрицательным dt должен быть пропущен
        let result = manager.step(-1.0);
        assert!(result.is_ok());

        // Шаг с inf должен быть пропущен
        let result = manager.step(f32::INFINITY);
        assert!(result.is_ok());
    }
}
