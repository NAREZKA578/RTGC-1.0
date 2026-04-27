//! Input module for RTGC-0.8
//! DEBUG: Добавлены экспорты для gamepad

pub mod mapping;
pub mod input_module;
pub mod gamepad;
pub mod action_map;

pub use input_module::InputManager;
// DEBUG: Экспорт Gamepad для input_module.rs
pub use gamepad::{GamepadButton, GamepadAxis, GamepadState};
// Тип Gamepad это псевдоним для GamepadState
pub use gamepad::GamepadState as Gamepad;
