//! Input mapping - Action mapping and key rebinding

use std::collections::HashMap;
use winit::keyboard::KeyCode;

/// Represents an input action that can be mapped to keys/buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    // Movement
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,

    // Camera
    LookUp,
    LookDown,
    LookLeft,
    LookRight,

    // Vehicle controls
    ThrottleUp,
    ThrottleDown,
    Throttle,
    SteerLeft,
    SteerRight,
    YawLeft,
    YawRight,
    PitchUp,
    PitchDown,
    RollLeft,
    RollRight,
    Brake,

    // Interaction
    Interact,
    Jump,
    Crouch,
    Sprint,

    // UI
    Menu,
    Pause,
    Map,
    Inventory,

    // Misc
    FirePrimary,
    FireSecondary,
    Reload,
}

impl InputAction {
    /// Returns the default key binding for this action
    pub fn default_key_code(&self) -> Option<KeyCode> {
        match self {
            InputAction::MoveForward => Some(KeyCode::KeyW),
            InputAction::MoveBackward => Some(KeyCode::KeyS),
            InputAction::MoveLeft => Some(KeyCode::KeyA),
            InputAction::MoveRight => Some(KeyCode::KeyD),

            InputAction::LookUp
            | InputAction::LookDown
            | InputAction::LookLeft
            | InputAction::LookRight => None, // Mouse only

            InputAction::ThrottleUp => Some(KeyCode::ShiftLeft),
            InputAction::ThrottleDown => Some(KeyCode::ControlLeft),
            InputAction::YawLeft => Some(KeyCode::KeyQ),
            InputAction::YawRight => Some(KeyCode::KeyE),
            InputAction::PitchUp => Some(KeyCode::ArrowUp),
            InputAction::PitchDown => Some(KeyCode::ArrowDown),
            InputAction::RollLeft => Some(KeyCode::ArrowLeft),
            InputAction::RollRight => Some(KeyCode::ArrowRight),
            InputAction::Brake => Some(KeyCode::Space),

            InputAction::Interact => Some(KeyCode::KeyF),
            InputAction::Jump => Some(KeyCode::Space),
            InputAction::Crouch => Some(KeyCode::KeyC),
            InputAction::Sprint => Some(KeyCode::ShiftLeft),

            InputAction::Menu => Some(KeyCode::Escape),
            InputAction::Pause => Some(KeyCode::Escape),
            InputAction::Map => Some(KeyCode::KeyM),
            InputAction::Inventory => Some(KeyCode::KeyI),

            InputAction::FirePrimary => Some(KeyCode::KeyZ),
            InputAction::FireSecondary => Some(KeyCode::KeyX),
            InputAction::Reload => Some(KeyCode::KeyR),

            InputAction::Throttle | InputAction::SteerLeft | InputAction::SteerRight => None,
        }
    }

    /// Returns a human-readable name for this action
    pub fn name(&self) -> &'static str {
        match self {
            InputAction::MoveForward => "Move Forward",
            InputAction::MoveBackward => "Move Backward",
            InputAction::MoveLeft => "Move Left",
            InputAction::MoveRight => "Move Right",

            InputAction::LookUp => "Look Up",
            InputAction::LookDown => "Look Down",
            InputAction::LookLeft => "Look Left",
            InputAction::LookRight => "Look Right",

            InputAction::ThrottleUp => "Throttle Up",
            InputAction::ThrottleDown => "Throttle Down",
            InputAction::YawLeft => "Yaw Left",
            InputAction::YawRight => "Yaw Right",
            InputAction::PitchUp => "Pitch Up",
            InputAction::PitchDown => "Pitch Down",
            InputAction::RollLeft => "Roll Left",
            InputAction::RollRight => "Roll Right",
            InputAction::Brake => "Brake",

            InputAction::Interact => "Interact",
            InputAction::Jump => "Jump",
            InputAction::Crouch => "Crouch",
            InputAction::Sprint => "Sprint",

            InputAction::Menu => "Menu",
            InputAction::Pause => "Pause",
            InputAction::Map => "Map",
            InputAction::Inventory => "Inventory",

            InputAction::FirePrimary => "Fire Primary",
            InputAction::FireSecondary => "Fire Secondary",
            InputAction::Reload => "Reload",

            InputAction::Throttle => "Throttle",
            InputAction::SteerLeft => "Steer Left",
            InputAction::SteerRight => "Steer Right",
        }
    }
}

/// Represents a mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(button: winit::event::MouseButton) -> Self {
        match button {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Other(id) => MouseButton::Other(id),
            winit::event::MouseButton::Back => MouseButton::Other(3),
            winit::event::MouseButton::Forward => MouseButton::Other(4),
        }
    }
}

/// Input mapping configuration
#[derive(Debug, Clone)]
pub struct InputMapping {
    /// Key bindings: Action -> KeyCode
    key_bindings: HashMap<InputAction, KeyCode>,
    /// Reverse lookup: KeyCode -> Action
    key_to_action: HashMap<KeyCode, InputAction>,
    /// Mouse button bindings
    mouse_bindings: HashMap<MouseButton, InputAction>,
    /// Gamepad button bindings
    gamepad_bindings: HashMap<crate::input::gamepad::GamepadButton, InputAction>,
}

impl InputMapping {
    /// Creates a new input mapping with default bindings
    pub fn new() -> Self {
        let mut mapping = Self {
            key_bindings: HashMap::new(),
            key_to_action: HashMap::new(),
            mouse_bindings: HashMap::new(),
            gamepad_bindings: HashMap::new(),
        };

        // Set up default key bindings
        for action in [
            InputAction::MoveForward,
            InputAction::MoveBackward,
            InputAction::MoveLeft,
            InputAction::MoveRight,
            InputAction::ThrottleUp,
            InputAction::ThrottleDown,
            InputAction::YawLeft,
            InputAction::YawRight,
            InputAction::PitchUp,
            InputAction::PitchDown,
            InputAction::RollLeft,
            InputAction::RollRight,
            InputAction::Brake,
            InputAction::Interact,
            InputAction::Jump,
            InputAction::Crouch,
            InputAction::Sprint,
            InputAction::Menu,
            InputAction::Pause,
            InputAction::Map,
            InputAction::Inventory,
            InputAction::FirePrimary,
            InputAction::FireSecondary,
            InputAction::Reload,
        ] {
            if let Some(key_code) = action.default_key_code() {
                mapping.bind_key(action, key_code);
            }
        }

        // Default mouse bindings
        mapping.bind_mouse(MouseButton::Left, InputAction::FirePrimary);
        mapping.bind_mouse(MouseButton::Right, InputAction::FireSecondary);

        mapping
    }

    /// Binds an action to a key code
    pub fn bind_key(&mut self, action: InputAction, key_code: KeyCode) {
        // Remove old binding if exists
        if let Some(old_key) = self.key_bindings.get(&action) {
            self.key_to_action.remove(old_key);
        }

        // Remove any action bound to this key
        if let Some(old_action) = self.key_to_action.get(&key_code) {
            self.key_bindings.remove(old_action);
        }

        self.key_bindings.insert(action, key_code);
        self.key_to_action.insert(key_code, action);
    }

    /// Binds an action to a mouse button
    pub fn bind_mouse(&mut self, button: MouseButton, action: InputAction) {
        self.mouse_bindings.insert(button, action);
    }

    /// Gets the action bound to a key code
    pub fn get_action_for_key(&self, key_code: &KeyCode) -> Option<InputAction> {
        self.key_to_action.get(key_code).copied()
    }

    /// Gets the action bound to a mouse button
    pub fn get_action_for_mouse(&self, button: &MouseButton) -> Option<InputAction> {
        self.mouse_bindings.get(button).copied()
    }

    /// Gets the action bound to a gamepad button
    pub fn get_action_for_gamepad_button(
        &self,
        button: crate::input::gamepad::GamepadButton,
    ) -> Option<InputAction> {
        self.gamepad_bindings.get(&button).copied()
    }

    /// Binds a gamepad button to an action
    pub fn bind_gamepad_button(
        &mut self,
        action: InputAction,
        button: crate::input::gamepad::GamepadButton,
    ) {
        self.gamepad_bindings.insert(button, action);
    }

    /// Gets the key code bound to an action
    pub fn get_key_for_action(&self, action: &InputAction) -> Option<KeyCode> {
        self.key_bindings.get(action).copied()
    }

    /// Checks if an action is currently bound
    pub fn is_action_bound(&self, action: &InputAction) -> bool {
        self.key_bindings.contains_key(action)
            || self.mouse_bindings.values().any(|&a| a == *action)
    }

    /// Resets a binding to its default
    pub fn reset_binding(&mut self, action: InputAction) {
        if let Some(default_key) = action.default_key_code() {
            self.bind_key(action, default_key);
        } else {
            // Unbind if no default
            if let Some(old_key) = self.key_bindings.get(&action) {
                self.key_to_action.remove(old_key);
            }
            self.key_bindings.remove(&action);
        }
    }

    /// Resets all bindings to defaults
    pub fn reset_all(&mut self) {
        *self = Self::new();
    }

    /// Exports bindings to a serializable format
    pub fn export(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();

        for (action, key_code) in &self.key_bindings {
            let key_str = format!("{:?}", key_code);
            result.insert(format!("action_{:?}", action), key_str);
        }

        result
    }

    /// Imports bindings from a serializable format
    pub fn import(&mut self, data: &HashMap<String, String>) -> Result<(), String> {
        for (key, value) in data {
            if let Some(action_str) = key.strip_prefix("action_") {
                let action = self.parse_action(action_str)?;
                let key_code = self.parse_key_code(value)?;
                self.bind_key(action, key_code);
            }
        }
        Ok(())
    }

    fn parse_action(&self, s: &str) -> Result<InputAction, String> {
        // Simplified parsing - in production use serde
        match s {
            "MoveForward" => Ok(InputAction::MoveForward),
            "MoveBackward" => Ok(InputAction::MoveBackward),
            "MoveLeft" => Ok(InputAction::MoveLeft),
            "MoveRight" => Ok(InputAction::MoveRight),
            "ThrottleUp" => Ok(InputAction::ThrottleUp),
            "ThrottleDown" => Ok(InputAction::ThrottleDown),
            "YawLeft" => Ok(InputAction::YawLeft),
            "YawRight" => Ok(InputAction::YawRight),
            "PitchUp" => Ok(InputAction::PitchUp),
            "PitchDown" => Ok(InputAction::PitchDown),
            "RollLeft" => Ok(InputAction::RollLeft),
            "RollRight" => Ok(InputAction::RollRight),
            "Brake" => Ok(InputAction::Brake),
            "Interact" => Ok(InputAction::Interact),
            "Jump" => Ok(InputAction::Jump),
            "Crouch" => Ok(InputAction::Crouch),
            "Sprint" => Ok(InputAction::Sprint),
            "Menu" => Ok(InputAction::Menu),
            "Pause" => Ok(InputAction::Pause),
            "Map" => Ok(InputAction::Map),
            "Inventory" => Ok(InputAction::Inventory),
            "FirePrimary" => Ok(InputAction::FirePrimary),
            "FireSecondary" => Ok(InputAction::FireSecondary),
            "Reload" => Ok(InputAction::Reload),
            _ => Err(format!("Unknown action: {}", s)),
        }
    }

    fn parse_key_code(&self, s: &str) -> Result<KeyCode, String> {
        // Simplified parsing - in production use serde
        match s {
            "KeyW" => Ok(KeyCode::KeyW),
            "KeyA" => Ok(KeyCode::KeyA),
            "KeyS" => Ok(KeyCode::KeyS),
            "KeyD" => Ok(KeyCode::KeyD),
            "Space" => Ok(KeyCode::Space),
            "ShiftLeft" => Ok(KeyCode::ShiftLeft),
            "ControlLeft" => Ok(KeyCode::ControlLeft),
            "KeyQ" => Ok(KeyCode::KeyQ),
            "KeyE" => Ok(KeyCode::KeyE),
            "ArrowUp" => Ok(KeyCode::ArrowUp),
            "ArrowDown" => Ok(KeyCode::ArrowDown),
            "ArrowLeft" => Ok(KeyCode::ArrowLeft),
            "ArrowRight" => Ok(KeyCode::ArrowRight),
            "KeyF" => Ok(KeyCode::KeyF),
            "KeyC" => Ok(KeyCode::KeyC),
            "Escape" => Ok(KeyCode::Escape),
            "KeyM" => Ok(KeyCode::KeyM),
            "KeyI" => Ok(KeyCode::KeyI),
            "KeyR" => Ok(KeyCode::KeyR),
            "MouseLeft" => Ok(KeyCode::KeyZ),
            "MouseRight" => Ok(KeyCode::KeyX),
            _ => Err(format!("Unknown key code: {}", s)),
        }
    }
}

impl Default for InputMapping {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mapping() {
        let mapping = InputMapping::new();
        assert!(mapping.is_action_bound(&InputAction::MoveForward));
        assert_eq!(
            mapping.get_key_for_action(&InputAction::MoveForward),
            Some(KeyCode::KeyW)
        );
    }

    #[test]
    fn test_rebinding() {
        let mut mapping = InputMapping::new();
        mapping.bind_key(InputAction::MoveForward, KeyCode::ArrowUp);
        assert_eq!(
            mapping.get_key_for_action(&InputAction::MoveForward),
            Some(KeyCode::ArrowUp)
        );
        assert!(
            !mapping.is_action_bound(&InputAction::MoveForward)
                || mapping.get_key_for_action(&InputAction::MoveForward) == Some(KeyCode::ArrowUp)
        );
    }
}
