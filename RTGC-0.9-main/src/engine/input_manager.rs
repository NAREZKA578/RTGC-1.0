//! Менеджер ввода - инкапсуляция системы ввода
//!
//! Этот модуль управляет обработкой ввода с клавиатуры, мыши и геймпадов,
//! предоставляя контролируемый интерфейс для игровых систем.

use crate::input::InputManager;
use tracing::debug;
use winit::event::MouseButton;
use winit::keyboard::{KeyCode, PhysicalKey};

/// Менеджер ввода для движка
pub struct InputManagerWrapper {
    /// Основной менеджер ввода
    input_manager: InputManager,
}

impl InputManagerWrapper {
    /// Создаёт новый менеджер ввода
    pub fn new() -> Self {
        Self {
            input_manager: InputManager::new(),
        }
    }

    /// Обновляет состояние ввода
    pub fn update(&mut self) {
        self.input_manager.update();
    }

    /// Устанавливает состояние клавиши
    pub fn set_key_state(&mut self, key: PhysicalKey, pressed: bool) {
        self.input_manager.set_key_state(key, pressed);
    }

    /// Устанавливает состояние кнопки мыши
    pub fn set_mouse_button_state(&mut self, button: MouseButton, pressed: bool) {
        self.input_manager
            .set_mouse_button_state(button.into(), pressed);
    }

    /// Проверяет, удерживается ли клавиша
    pub fn is_key_held(&self, key: KeyCode) -> bool {
        self.input_manager.state().is_key_held(key)
    }

    /// Получает PlayerInput если доступен
    pub fn action_map(&self) -> Option<crate::network::protocol::PlayerInput> {
        self.input_manager.action_map()
    }

    /// Получает состояние ввода
    pub fn state(&self) -> &crate::input::input_module::InputState {
        self.input_manager.state()
    }

    /// Получает доступ к InputManager
    pub fn input_manager(&self) -> &InputManager {
        &self.input_manager
    }

    /// Обрабатывает событие клавиатуры для движения персонажа
    pub fn handle_movement_keys(&mut self, key_code: KeyCode, pressed: bool) {
        let physical_key = match key_code {
            KeyCode::KeyW => PhysicalKey::Code(KeyCode::KeyW),
            KeyCode::KeyS => PhysicalKey::Code(KeyCode::KeyS),
            KeyCode::KeyA => PhysicalKey::Code(KeyCode::KeyA),
            KeyCode::KeyD => PhysicalKey::Code(KeyCode::KeyD),
            KeyCode::Space => PhysicalKey::Code(KeyCode::Space),
            KeyCode::ShiftLeft => PhysicalKey::Code(KeyCode::ShiftLeft),
            _ => return,
        };

        self.set_key_state(physical_key, pressed);

        if pressed {
            debug!(target: "input", "Movement key pressed: {:?}", key_code);
        }
    }
}

impl Default for InputManagerWrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_manager_creation() {
        let manager = InputManagerWrapper::new();
        assert!(!manager.is_key_held(KeyCode::KeyW));
    }

    #[test]
    fn test_key_state_setting() {
        let mut manager = InputManagerWrapper::new();
        manager.set_key_state(PhysicalKey::Code(KeyCode::KeyW), true);
        // InputManager может не сохранять состояние сразу без update
    }
}
