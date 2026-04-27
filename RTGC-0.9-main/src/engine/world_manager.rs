//! Менеджер игрового мира - инкапсуляция мировой подсистемы
//!
//! Этот модуль управляет открытым миром, погодой, циклом дня и ночи,
//! а также генерацией миссий.

use crate::error::EngineError;
use crate::game::{Mission, MissionGenerator};
use crate::weather::dynamic_weather::WeatherSystem;
use crate::world::{DayNightCycle, OpenWorld, RoadNetwork, Settlement};
use nalgebra::Vector3;
use tracing::{info, warn};

/// Менеджер игрового мира
pub struct WorldManager {
    /// Открытый мир
    open_world: Option<OpenWorld>,

    /// Система погоды
    weather_system: WeatherSystem,

    /// Цикл дня и ночи
    day_night_cycle: DayNightCycle,

    /// Поселения в мире
    settlements: Vec<Settlement>,

    /// Дорожная сеть
    road_network: Option<RoadNetwork>,

    /// Генератор миссий
    mission_generator: Option<MissionGenerator>,

    /// Текущая активная миссия
    current_mission: Option<Mission>,

    /// Seed мира
    world_seed: u64,
}

impl WorldManager {
    /// Создаёт новый менеджер мира
    pub fn new(world_seed: u64) -> Self {
        let weather_system = WeatherSystem::new(world_seed);
        let day_night_cycle = DayNightCycle::new(55.0, 82.9);

        Self {
            open_world: None,
            weather_system,
            day_night_cycle,
            settlements: Vec::new(),
            road_network: None,
            mission_generator: None,
            current_mission: None,
            world_seed,
        }
    }

    /// Инициализирует открытый мир
    pub fn initialize_world(&mut self) -> Result<(), EngineError> {
        info!(target: "world", "Initializing open world with seed: {}", self.world_seed);

        self.open_world = Some(OpenWorld::new(self.world_seed));

        // Initialize settlements and road network
        self.settlements = self.generate_settlements();
        self.road_network = Some(RoadNetwork::generate(
            &self.settlements,
            self.world_seed,
            &|_x, _z| 0.0,
        ));

        // Initialize mission generator
        self.mission_generator = Some(MissionGenerator::new(
            self.settlements.clone(),
            self.road_network.clone().unwrap_or_default(),
            self.world_seed,
        ));

        Ok(())
    }

    /// Генерирует поселения на основе seed
    fn generate_settlements(&self) -> Vec<Settlement> {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;

        let mut rng = ChaCha8Rng::seed_from_u64(self.world_seed);
        let mut settlements = Vec::new();

        // Generate 5-10 settlements around the player start position
        let num_settlements = rng.gen_range(5..=10);

        for i in 0..num_settlements {
            let grid_x = rng.gen_range(-3..=3);
            let grid_z = rng.gen_range(-3..=3);
            let center_x = grid_x as f32 * 1000.0;
            let center_z = grid_z as f32 * 1000.0;

            if let Some(settlement) = Settlement::generate(
                self.world_seed + i as u64,
                grid_x,
                grid_z,
                center_x,
                center_z,
            ) {
                settlements.push(settlement);
            }
        }

        settlements
    }

    /// Обновляет все системы мира
    pub fn update(&mut self, dt: f32) -> Result<(), EngineError> {
        // Проверка на NaN/Inf
        if !dt.is_finite() || dt <= 0.0 {
            warn!(target: "world", "Invalid dt value: {}, skipping world update", dt);
            return Ok(());
        }

        // Обновление погоды
        let current_hour = self.day_night_cycle.get_hour();
        self.weather_system.update(dt, current_hour);

        // Обновление цикла дня и ночи
        self.day_night_cycle.advance_time(dt);

        // Обновление миссии (если активна) - removed update call, missions are static data
        // if let Some(ref mut mission) = self.current_mission {
        //     mission.update(dt);
        // }

        Ok(())
    }

    /// Получает ссылку на открытый мир
    pub fn get_open_world(&self) -> Option<&OpenWorld> {
        self.open_world.as_ref()
    }

    /// Получает мутабельную ссылку на открытый мир
    pub fn get_open_world_mut(&mut self) -> Option<&mut OpenWorld> {
        self.open_world.as_mut()
    }

    /// Получает ссылку на погоду
    pub fn get_weather(&self) -> &WeatherSystem {
        &self.weather_system
    }

    /// Получает ссылку на цикл дня и ночи
    pub fn get_day_night_cycle(&self) -> &DayNightCycle {
        &self.day_night_cycle
    }

    /// Получает текущее время суток (часы)
    pub fn get_current_hour(&self) -> f32 {
        self.day_night_cycle.get_hour()
    }

    /// Загружает миссию
    pub fn load_mission(&mut self, mission: Mission) {
        self.current_mission = Some(mission);
        info!(target: "world", "Mission loaded");
    }

    /// Завершает текущую миссию
    pub fn complete_current_mission(&mut self) -> Option<Mission> {
        self.current_mission.take()
    }

    /// Проверяет, активна ли миссия
    pub fn has_active_mission(&self) -> bool {
        self.current_mission.is_some()
    }

    /// Генерирует новую миссию (если доступен генератор)
    pub fn generate_mission(&mut self, player_pos: Vector3<f32>) -> Option<Mission> {
        self.mission_generator
            .as_mut()?
            .generate_mission(player_pos)
    }

    /// Сбрасывает мир
    pub fn reset(&mut self) {
        self.open_world = None;
        self.settlements.clear();
        self.road_network = None;
        self.current_mission = None;
        info!(target: "world", "World reset");
    }

    /// Получает seed мира
    pub fn get_seed(&self) -> u64 {
        self.world_seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_manager_creation() {
        let manager = WorldManager::new(42);

        assert_eq!(manager.get_seed(), 42);
        assert!(manager.get_open_world().is_none());
        assert!(!manager.has_active_mission());
    }

    #[test]
    fn test_world_initialization() {
        let mut manager = WorldManager::new(123);

        let result = manager.initialize_world();
        assert!(result.is_ok());
        assert!(manager.get_open_world().is_some());
    }

    #[test]
    fn test_world_update_with_invalid_dt() {
        let mut manager = WorldManager::new(42);

        // Обновление с NaN должно быть пропущено
        let result = manager.update(f32::NAN);
        assert!(result.is_ok());

        // Обновление с отрицательным dt должно быть пропущено
        let result = manager.update(-1.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mission_lifecycle() {
        let mut manager = WorldManager::new(42);

        assert!(!manager.has_active_mission());

        // Создаём тестовую миссию
        let mission = Mission {
            id: 1,
            name: "Test Mission".to_string(),
            description: "Test".to_string(),
            objectives: vec![],
            rewards: vec![],
            difficulty: 1,
        };

        manager.load_mission(mission);
        assert!(manager.has_active_mission());

        let completed = manager.complete_current_mission();
        assert!(completed.is_some());
        assert!(!manager.has_active_mission());
    }
}
