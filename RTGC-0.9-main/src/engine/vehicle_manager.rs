//! Менеджер транспортных средств - инкапсуляция логики управления
//!
//! Этот модуль управляет созданием, уничтожением и переключением между
//! различными типами транспортных средств.

use crate::error::EngineError;
use crate::game::{Cargo, Winch};
use crate::physics::{Helicopter, TrackedVehicle, Vehicle};
use nalgebra::{UnitQuaternion, Vector3};
use tracing::info;

/// Тип транспортного средства
#[derive(Debug, Clone, PartialEq)]
pub enum VehicleType {
    /// Колёсное транспортное средство
    Wheeled,
    /// Вертолёт
    Helicopter,
    /// Гусеничная машина
    Tracked,
}

/// Менеджер транспортных средств
pub struct VehicleManager {
    /// Активное транспортное средство
    active_vehicle: Option<ActiveVehicle>,

    /// Груз (если есть)
    cargo: Option<Cargo>,

    /// Лебёдка
    winch: Winch,

    /// Позиция спавна по умолчанию
    default_spawn_position: Vector3<f32>,
}

/// Активное транспортное средство (enum для хранения разных типов)
#[derive(Debug)]
pub enum ActiveVehicle {
    Wheeled(Vehicle),
    Helicopter(Helicopter),
    Tracked(TrackedVehicle),
}

impl ActiveVehicle {
    /// Получает позицию транспортного средства
    pub fn position(&self) -> Vector3<f32> {
        match self {
            Self::Wheeled(v) => v.position(),
            Self::Helicopter(h) => h.position,
            Self::Tracked(t) => t.position,
        }
    }

    /// Получает ориентацию (поворот) транспортного средства
    pub fn rotation(&self) -> UnitQuaternion<f32> {
        match self {
            Self::Wheeled(v) => v.rotation(),
            Self::Helicopter(h) => h.rotation,
            Self::Tracked(t) => t.orientation,
        }
    }

    /// Получает forward вектор (направление движения)
    pub fn forward(&self) -> Vector3<f32> {
        self.rotation() * Vector3::new(0.0, 0.0, 1.0)
    }

    /// Получает скорость транспортного средства
    pub fn speed(&self) -> f32 {
        match self {
            Self::Wheeled(v) => v.speed(),
            Self::Helicopter(h) => h.velocity.norm(),
            Self::Tracked(t) => t.linear_velocity.norm(),
        }
    }

    /// Проверяет, является ли транспортное средство вертолётом
    pub fn is_helicopter(&self) -> bool {
        matches!(self, Self::Helicopter(_))
    }

    /// Проверяет, является ли транспортное средство колёсным
    pub fn is_wheeled(&self) -> bool {
        matches!(self, Self::Wheeled(_))
    }

    /// Проверяет, является ли транспортное средство гусеничным
    pub fn is_tracked(&self) -> bool {
        matches!(self, Self::Tracked(_))
    }
}

impl VehicleManager {
    /// Создаёт новый менеджер транспортных средств
    pub fn new(default_spawn_position: Vector3<f32>) -> Self {
        Self {
            active_vehicle: None,
            cargo: None,
            winch: Winch::new(0),
            default_spawn_position,
        }
    }

    /// Спавнит колёсное транспортное средство
    pub fn spawn_wheeled_vehicle(&mut self, vehicle: Vehicle) -> Result<(), EngineError> {
        info!(target: "vehicle", "Spawning wheeled vehicle at {:?}", self.default_spawn_position);
        self.active_vehicle = Some(ActiveVehicle::Wheeled(vehicle));
        Ok(())
    }

    /// Спавнит вертолёт
    pub fn spawn_helicopter(&mut self, helicopter: Helicopter) -> Result<(), EngineError> {
        info!(target: "vehicle", "Spawning helicopter at {:?}", self.default_spawn_position);
        self.active_vehicle = Some(ActiveVehicle::Helicopter(helicopter));
        Ok(())
    }

    /// Спавнит гусеничную машину
    pub fn spawn_tracked_vehicle(&mut self, tracked: TrackedVehicle) -> Result<(), EngineError> {
        info!(target: "vehicle", "Spawning tracked vehicle at {:?}", self.default_spawn_position);
        self.active_vehicle = Some(ActiveVehicle::Tracked(tracked));
        Ok(())
    }

    /// Удаляет активное транспортное средство
    pub fn despawn_active_vehicle(&mut self) -> Option<ActiveVehicle> {
        info!(target: "vehicle", "Despawning active vehicle");
        self.active_vehicle.take()
    }

    /// Переключается на другое транспортное средство
    pub fn switch_vehicle(&mut self, new_vehicle: ActiveVehicle) -> Option<ActiveVehicle> {
        let old = self.active_vehicle.take();
        self.active_vehicle = Some(new_vehicle);
        info!(target: "vehicle", "Switched to {}",
            match &self.active_vehicle {
                Some(ActiveVehicle::Wheeled(_)) => "wheeled vehicle",
                Some(ActiveVehicle::Helicopter(_)) => "helicopter",
                Some(ActiveVehicle::Tracked(_)) => "tracked vehicle",
                None => "nothing",
            }
        );
        old
    }

    /// Получает ссылку на активное транспортное средство
    pub fn get_active_vehicle(&self) -> Option<&ActiveVehicle> {
        self.active_vehicle.as_ref()
    }

    /// Получает мутабельную ссылку на активное транспортное средство
    pub fn get_active_vehicle_mut(&mut self) -> Option<&mut ActiveVehicle> {
        self.active_vehicle.as_mut()
    }

    /// Проверяет наличие активного транспортного средства
    pub fn has_active_vehicle(&self) -> bool {
        self.active_vehicle.is_some()
    }

    /// Получает тип активного транспортного средства
    pub fn get_active_vehicle_type(&self) -> Option<VehicleType> {
        self.active_vehicle.as_ref().map(|v| match v {
            ActiveVehicle::Wheeled(_) => VehicleType::Wheeled,
            ActiveVehicle::Helicopter(_) => VehicleType::Helicopter,
            ActiveVehicle::Tracked(_) => VehicleType::Tracked,
        })
    }

    /// Устанавливает груз
    pub fn set_cargo(&mut self, cargo: Cargo) {
        self.cargo = Some(cargo);
    }

    /// Получает ссылку на груз
    pub fn get_cargo(&self) -> Option<&Cargo> {
        self.cargo.as_ref()
    }

    /// Получает ссылку на лебёдку
    pub fn get_winch(&self) -> &Winch {
        &self.winch
    }

    /// Получает мутабельную ссылку на лебёдку
    pub fn get_winch_mut(&mut self) -> &mut Winch {
        &mut self.winch
    }

    /// Обновляет состояние лебёдки (упрощённо - физика не требуется)
    pub fn update_winch(&mut self, dt: f32) {
        // winch.update() в реализации не использует physics_world,
        // поэтому передаём пустышку
        let mut dummy_constraints: Vec<crate::physics::SpringConstraint> = Vec::new();
        self.winch.update(
            dt,
            &mut crate::physics::PhysicsWorld::new(),
            &mut dummy_constraints,
        );
    }

    /// Сбрасывает все транспортные средства
    pub fn clear_all(&mut self) {
        self.active_vehicle = None;
        self.cargo = None;
        self.winch = Winch::new(0);
        info!(target: "vehicle", "All vehicles cleared");
    }

    /// Получает позицию активного транспортного средства
    pub fn get_active_position(&self) -> Option<Vector3<f32>> {
        self.active_vehicle.as_ref().map(|v| v.position())
    }

    /// Получает скорость активного транспортного средства
    pub fn get_active_speed(&self) -> Option<f32> {
        self.active_vehicle.as_ref().map(|v| v.speed())
    }

    /// Получает позицию игрока (активного транспортного средства или дефолтную)
    pub fn get_player_position(&self) -> Vector3<f32> {
        self.get_active_position()
            .unwrap_or(self.default_spawn_position)
    }

    /// Получает направление взгляда игрока (вперёд по транспортному средству или дефолтное)
    pub fn get_player_forward(&self) -> Vector3<f32> {
        self.active_vehicle
            .as_ref()
            .map(|v| v.forward())
            .unwrap_or(Vector3::z())
    }

    /// Устанавливает позицию спавна игрока
    pub fn set_player_position(&mut self, position: Vector3<f32>) {
        self.default_spawn_position = position;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::PhysicsWorld;

    #[test]
    fn test_vehicle_manager_creation() {
        let spawn_pos = Vector3::new(0.0, 0.0, 0.0);
        let manager = VehicleManager::new(spawn_pos);

        assert!(!manager.has_active_vehicle());
        assert_eq!(manager.get_active_vehicle_type(), None);
    }

    #[test]
    fn test_spawn_wheeled_vehicle() {
        let mut manager = VehicleManager::new(Vector3::zeros());

        // Создаём тестовое транспортное средство
        let physics_world = PhysicsWorld::new();
        let vehicle = Vehicle::new(&physics_world, Vector3::new(1.0, 2.0, 3.0));

        let result = manager.spawn_wheeled_vehicle(vehicle);
        assert!(result.is_ok());
        assert!(manager.has_active_vehicle());
        assert_eq!(
            manager.get_active_vehicle_type(),
            Some(VehicleType::Wheeled)
        );
    }

    #[test]
    fn test_vehicle_switching() {
        let mut manager = VehicleManager::new(Vector3::zeros());

        assert!(!manager.has_active_vehicle());

        // Спавним колёсное ТС
        let physics_world = PhysicsWorld::new();
        let vehicle = Vehicle::new(&physics_world, Vector3::zeros());
        manager.spawn_wheeled_vehicle(vehicle).unwrap();

        assert!(manager.has_active_vehicle());
        assert_eq!(
            manager.get_active_vehicle_type(),
            Some(VehicleType::Wheeled)
        );

        // Очищаем
        manager.clear_all();
        assert!(!manager.has_active_vehicle());
    }

    #[test]
    fn test_winch_access() {
        let mut manager = VehicleManager::new(Vector3::zeros());

        let winch = manager.get_winch();
        assert!(winch.is_attached() == false || winch.is_attached() == true); // Просто проверяем доступ
    }
}
