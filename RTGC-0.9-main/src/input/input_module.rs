//! Input System - Enhanced action-based input with gamepad support
//! DEBUG: Исправлен импорт Gamepad

use std::collections::HashMap;
use winit::event::{ElementState, KeyEvent, MouseButton as WinitMouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

pub use crate::input::mapping::{InputAction, InputMapping, MouseButton};
// DEBUG: Импорт Gamepad из mod.rs
pub use crate::input::gamepad::{GamepadAxis, GamepadButton, GamepadState};
// Gamepad type alias for backwards compatibility
pub use crate::input::Gamepad;

/// State of an input action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionState {
    /// Action is not active
    Released,
    /// Action was just pressed this frame
    JustPressed,
    /// Action is being held
    Held,
    /// Action was just released this frame
    JustReleased,
}

impl Default for ActionState {
    fn default() -> Self {
        Self::Released
    }
}

/// Combined input state from all sources
#[derive(Debug, Clone)]
pub struct InputState {
    /// Current state of each action
    action_states: HashMap<InputAction, ActionState>,
    /// Previous frame's action states
    previous_action_states: HashMap<InputAction, ActionState>,
    /// Key states
    key_states: HashMap<PhysicalKey, ActionState>,
    /// Mouse button states
    mouse_states: HashMap<MouseButton, ActionState>,
    /// Mouse position
    mouse_position: (f64, f64),
    /// Mouse delta
    mouse_delta: (f64, f64),
    /// Scroll delta
    scroll_delta: (f64, f64),
    /// Connected gamepads
    gamepads: Vec<GamepadState>,
    /// Input mapping configuration
    mapping: InputMapping,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            action_states: HashMap::new(),
            previous_action_states: HashMap::new(),
            key_states: HashMap::new(),
            mouse_states: HashMap::new(),
            mouse_position: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            scroll_delta: (0.0, 0.0),
            gamepads: Vec::new(),
            mapping: InputMapping::new(),
        }
    }

    /// Begin a new frame - update previous states
    pub fn begin_frame(&mut self) {
        // Store previous action states
        self.previous_action_states = self.action_states.clone();

        // Update key states: JustPressed -> Held, JustReleased -> Released
        for state in self.key_states.values_mut() {
            *state = match state {
                ActionState::JustPressed => ActionState::Held,
                ActionState::JustReleased => ActionState::Released,
                _ => *state,
            };
        }

        // Update mouse states
        for state in self.mouse_states.values_mut() {
            *state = match state {
                ActionState::JustPressed => ActionState::Held,
                ActionState::JustReleased => ActionState::Released,
                _ => *state,
            };
        }

        // Reset deltas
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = (0.0, 0.0);
    }

    /// Handle keyboard input
    pub fn handle_keyboard(&mut self, event: &KeyEvent) {
        let physical_key = event.physical_key;

        match event.state {
            ElementState::Pressed => {
                self.key_states
                    .insert(physical_key, ActionState::JustPressed);

                // Check if this key maps to an action
                if let Some(key_code) = Self::physical_to_key_code(physical_key) {
                    if let Some(action) = self.mapping.get_action_for_key(&key_code) {
                        self.action_states.insert(action, ActionState::JustPressed);
                    }
                }
            }
            ElementState::Released => {
                self.key_states
                    .insert(physical_key, ActionState::JustReleased);

                if let Some(key_code) = Self::physical_to_key_code(physical_key) {
                    if let Some(action) = self.mapping.get_action_for_key(&key_code) {
                        self.action_states.insert(action, ActionState::JustReleased);
                    }
                }
            }
        }
    }

    /// Handle mouse button input
    pub fn handle_mouse_button(&mut self, button: WinitMouseButton, pressed: bool) {
        let mb = MouseButton::from(button);
        let state = if pressed {
            ActionState::JustPressed
        } else {
            ActionState::JustReleased
        };

        self.mouse_states.insert(mb, state);

        if let Some(action) = self.mapping.get_action_for_mouse(&mb) {
            self.action_states.insert(action, state);
        }
    }

    /// Handle mouse movement
    pub fn handle_mouse_motion(&mut self, position: (f64, f64), delta: (f64, f64)) {
        self.mouse_position = position;
        self.mouse_delta = delta;
    }

    /// Handle mouse scroll
    pub fn handle_scroll(&mut self, delta: (f64, f64)) {
        self.scroll_delta = delta;
    }

    /// Handle gamepad input
    pub fn handle_gamepad(&mut self, gamepad_state: GamepadState) {
        // Find or add gamepad
        if let Some(existing) = self.gamepads.iter_mut().find(|g| g.id == gamepad_state.id) {
            *existing = gamepad_state.clone();
        } else {
            self.gamepads.push(gamepad_state.clone());
        }

        // Map gamepad buttons to actions
        for (button, pressed) in &gamepad_state.buttons {
            let action = self.mapping.get_action_for_gamepad_button(*button);
            if let Some(action) = action {
                let state = if *pressed {
                    ActionState::JustPressed
                } else {
                    ActionState::JustReleased
                };
                self.action_states.insert(action, state);
            }
        }
    }

    /// Check if an action is just pressed (this frame only)
    pub fn is_action_just_pressed(&self, action: InputAction) -> bool {
        matches!(
            self.action_states.get(&action),
            Some(ActionState::JustPressed)
        )
    }

    /// Check if an action is currently held
    pub fn is_action_held(&self, action: InputAction) -> bool {
        matches!(
            self.action_states.get(&action),
            Some(ActionState::JustPressed | ActionState::Held)
        )
    }

    /// Check if an action was just released (this frame only)
    pub fn is_action_released(&self, action: InputAction) -> bool {
        matches!(
            self.action_states.get(&action),
            Some(ActionState::JustReleased)
        )
    }

    /// Get the current state of an action
    pub fn get_action_state(&self, action: InputAction) -> Option<ActionState> {
        self.action_states.get(&action).copied()
    }

    /// Check if a key is just pressed
    pub fn is_key_just_pressed(&self, key_code: KeyCode) -> bool {
        let physical_key = PhysicalKey::Code(key_code);
        matches!(
            self.key_states.get(&physical_key),
            Some(ActionState::JustPressed)
        )
    }

    /// Check if a key is held
    pub fn is_key_held(&self, key_code: KeyCode) -> bool {
        let physical_key = PhysicalKey::Code(key_code);
        matches!(
            self.key_states.get(&physical_key),
            Some(ActionState::JustPressed | ActionState::Held)
        )
    }

    /// Check if a mouse button is just pressed
    pub fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        matches!(
            self.mouse_states.get(&button),
            Some(ActionState::JustPressed)
        )
    }

    /// Check if a mouse button is held
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        matches!(
            self.mouse_states.get(&button),
            Some(ActionState::JustPressed | ActionState::Held)
        )
    }

    /// Get mouse position
    pub fn mouse_position(&self) -> (f64, f64) {
        self.mouse_position
    }

    /// Get mouse delta
    pub fn mouse_delta(&self) -> (f64, f64) {
        self.mouse_delta
    }

    /// Get scroll delta
    pub fn scroll_delta(&self) -> (f64, f64) {
        self.scroll_delta
    }

    /// Get the input mapping
    pub fn mapping(&self) -> &InputMapping {
        &self.mapping
    }

    /// Get mutable input mapping
    pub fn mapping_mut(&mut self) -> &mut InputMapping {
        &mut self.mapping
    }

    /// Get connected gamepads
    pub fn gamepads(&self) -> &[GamepadState] {
        &self.gamepads
    }

    /// Get primary gamepad (first connected)
    pub fn primary_gamepad(&self) -> Option<&GamepadState> {
        self.gamepads.first()
    }

    fn physical_to_key_code(physical: PhysicalKey) -> Option<KeyCode> {
        match physical {
            PhysicalKey::Code(code) => Some(code),
            PhysicalKey::Unidentified(_) => None,
        }
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

/// Input manager for handling all input sources
#[derive(Clone)]
pub struct InputManager {
    state: InputState,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            state: InputState::new(),
        }
    }

    /// Begin a new frame
    pub fn begin_frame(&mut self) {
        self.state.begin_frame();
    }

    /// Update input state (alias for begin_frame for backwards compatibility)
    pub fn update(&mut self) {
        self.state.begin_frame();
    }

    /// Handle keyboard input
    pub fn handle_keyboard(&mut self, event: &KeyEvent) {
        self.state.handle_keyboard(event);
    }

    /// Set key state directly
    pub fn set_key_state(&mut self, key: PhysicalKey, pressed: bool) {
        let state = if pressed {
            ActionState::JustPressed
        } else {
            ActionState::JustReleased
        };
        self.state.key_states.insert(key, state);
    }

    /// Handle mouse button
    pub fn handle_mouse_button(&mut self, button: WinitMouseButton, pressed: bool) {
        self.state.handle_mouse_button(button, pressed);
    }

    /// Set mouse button state directly
    pub fn set_mouse_button_state(&mut self, button: MouseButton, pressed: bool) {
        let state = if pressed {
            ActionState::JustPressed
        } else {
            ActionState::JustReleased
        };
        self.state.mouse_states.insert(button, state);
    }

    /// Handle mouse motion
    pub fn handle_mouse_motion(&mut self, position: (f64, f64), delta: (f64, f64)) {
        self.state.handle_mouse_motion(position, delta);
    }

    /// Handle scroll
    pub fn handle_scroll(&mut self, delta: (f64, f64)) {
        self.state.handle_scroll(delta);
    }

    /// Handle gamepad state
    pub fn handle_gamepad(&mut self, state: GamepadState) {
        self.state.handle_gamepad(state);
    }

    /// Check if action is just pressed
    pub fn is_action_just_pressed(&self, action: InputAction) -> bool {
        self.state.is_action_just_pressed(action)
    }

    /// Check if action is held
    pub fn is_action_held(&self, action: InputAction) -> bool {
        self.state.is_action_held(action)
    }

    /// Check if action is released
    pub fn is_action_released(&self, action: InputAction) -> bool {
        self.state.is_action_released(action)
    }

    /// Get action state
    pub fn get_action_state(&self, action: InputAction) -> Option<ActionState> {
        self.state.get_action_state(action)
    }

    /// Get input state reference
    pub fn state(&self) -> &InputState {
        &self.state
    }

    /// Get mutable input state
    pub fn state_mut(&mut self) -> &mut InputState {
        &mut self.state
    }

    /// Get action map for player input (convenience method)
    pub fn action_map(&self) -> Option<crate::network::protocol::PlayerInput> {
        let mut input = crate::network::protocol::PlayerInput::default();

        // Map input actions to player input
        if self.state.is_action_just_pressed(InputAction::MoveForward)
            || self.state.is_action_held(InputAction::MoveForward)
        {
            input.throttle = 1.0;
        }
        if self.state.is_action_just_pressed(InputAction::MoveBackward)
            || self.state.is_action_held(InputAction::MoveBackward)
        {
            input.throttle = -1.0;
        }
        if self.state.is_action_just_pressed(InputAction::MoveLeft)
            || self.state.is_action_held(InputAction::MoveLeft)
        {
            input.steering = -1.0;
        }
        if self.state.is_action_just_pressed(InputAction::MoveRight)
            || self.state.is_action_held(InputAction::MoveRight)
        {
            input.steering = 1.0;
        }
        if self.state.is_action_just_pressed(InputAction::Brake)
            || self.state.is_action_held(InputAction::Brake)
        {
            input.handbrake = true;
        }

        Some(input)
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_states() {
        let mut input = InputState::new();

        // Initially no action state
        assert_eq!(input.get_action_state(InputAction::MoveForward), None);

        // Simulate key press
        input.handle_keyboard(&KeyEvent {
            physical_key: PhysicalKey::Code(KeyCode::KeyW),
            logical_key: winit::keyboard::Key::Character("w"),
            location: winit::keyboard::KeyLocation::Standard,
            state: ElementState::Pressed,
            repeat: false,
            text: None,
            platform_specific: Default::default(),
        });

        assert!(input.is_action_just_pressed(InputAction::MoveForward));
        assert!(input.is_action_held(InputAction::MoveForward));

        // Begin new frame - should transition to Held
        input.begin_frame();
        assert!(!input.is_action_just_pressed(InputAction::MoveForward));
        assert!(input.is_action_held(InputAction::MoveForward));
    }
}
