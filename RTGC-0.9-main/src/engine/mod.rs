//! Модуль подсистем движка
//! 
//! Предоставляет инкапсулированные подсистемы для разделения ответственности
//! 
//! # Архитектура
//! 
//! Движок разделён на следующие подмодули:
//! 
//! - [`core`] - Ядро движка, основной класс Engine и игровой цикл
//! - [`state`] - Управление состоянием приложения (единый источник истины)
//! - [`subsystems`] - Контейнеры для подсистем (графика, физика, UI, мир)
//! - [`physics_manager`] - Инкапсуляция физической симуляции
//! - [`world_manager`] - Управление открытым миром, погодой, миссиями
//! - [`vehicle_manager`] - Управление транспортными средствами
//! - [`input_manager`] - Обработка ввода с клавиатуры, мыши и геймпадов
//! - [`game_loop_manager`] - Основной цикл обновлений игровых систем

pub mod core;
pub mod state;
pub mod subsystems;
pub mod physics_manager;
pub mod world_manager;
pub mod vehicle_manager;
pub mod input_manager;
pub mod game_loop_manager;
pub mod render_manager;

pub use crate::config::Config;
pub use core::*;
pub use state::*;
pub use subsystems::*;
pub use physics_manager::*;
pub use world_manager::*;
pub use vehicle_manager::*;
pub use input_manager::*;
pub use game_loop_manager::*;
pub use render_manager::*;
