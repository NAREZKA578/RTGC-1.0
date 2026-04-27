//! Action Map - Enhanced input action system with 15+ actions and state tracking

use std::collections::HashMap;
use winit::keyboard::KeyCode;
use winit::event::ElementState;
use crate::input::mapping::MouseButton;
use crate::input::gamepad::GamepadButton;

/// State of an input action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionState {
    /// Action is not pressed
    Released,
    /// Action was just pressed this frame
    JustPressed,
    /// Action is being held down
    Held,
    /// Action was just released this frame
    JustReleased,
}

impl Default for ActionState {
    fn default() -> Self {
        Self::Released
    }
}

/// Extended input actions (15+ actions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtendedAction {
    // Movement (4)
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,

    // Camera (4)
    LookUp,
    LookDown,
    LookLeft,
    LookRight,

    // Vehicle controls (8)
    ThrottleUp,
    ThrottleDown,
    YawLeft,
    YawRight,
    PitchUp,
    PitchDown,
    RollLeft,
    RollRight,

    // Character actions (4)
    Sprint,
    Crouch,
    Jump,
    Interact,

    // UI/Menu (4)
    OpenMenu,
    OpenMap,
    OpenInventory,
    Pause,

    // Combat/Interaction (4)
    FirePrimary,
    FireSecondary,
    Reload,
    UseItem,

    // Misc (3)
    Brake,
    Handbrake,
    Horn,
}

impl ExtendedAction {
    pub fn name(&self) -> &'static str {
        match self {
            ExtendedAction::MoveForward => "Move Forward",
            ExtendedAction::MoveBackward => "Move Backward",
            ExtendedAction::MoveLeft => "Move Left",
            ExtendedAction::MoveRight => "Move Right",
            ExtendedAction::LookUp => "Look Up",
            ExtendedAction::LookDown => "Look Down",
            ExtendedAction::LookLeft => "Look Left",
            ExtendedAction::LookRight => "Look Right",
            ExtendedAction::ThrottleUp => "Throttle Up",
            ExtendedAction::ThrottleDown => "Throttle Down",
            ExtendedAction::YawLeft => "Yaw Left",
            ExtendedAction::YawRight => "Yaw Right",
            ExtendedAction::PitchUp => "Pitch Up",
            ExtendedAction::PitchDown => "Pitch Down",
            ExtendedAction::RollLeft => "Roll Left",
            ExtendedAction::RollRight => "Roll Right",
            ExtendedAction::Sprint => "Sprint",
            ExtendedAction::Crouch => "Crouch",
            ExtendedAction::Jump => "Jump",
            ExtendedAction::Interact => "Interact",
            ExtendedAction::OpenMenu => "Open Menu",
            ExtendedAction::OpenMap => "Open Map",
            ExtendedAction::OpenInventory => "Open Inventory",
            ExtendedAction::Pause => "Pause",
            ExtendedAction::FirePrimary => "Fire Primary",
            ExtendedAction::FireSecondary => "Fire Secondary",
            ExtendedAction::Reload => "Reload",
            ExtendedAction::UseItem => "Use Item",
            ExtendedAction::Brake => "Brake",
            ExtendedAction::Handbrake => "Handbrake",
            ExtendedAction::Horn => "Horn",
        }
    }

    pub fn default_key_code(&self) -> Option<KeyCode> {
        match self {
            ExtendedAction::MoveForward => Some(KeyCode::KeyW),
            ExtendedAction::MoveBackward => Some(KeyCode::KeyS),
            ExtendedAction::MoveLeft => Some(KeyCode::KeyA),
            ExtendedAction::MoveRight => Some(KeyCode::KeyD),
            ExtendedAction::LookUp | ExtendedAction::LookDown |
            ExtendedAction::LookLeft | ExtendedAction::LookRight => None,
            ExtendedAction::ThrottleUp => Some(KeyCode::ShiftLeft),
            ExtendedAction::ThrottleDown => Some(KeyCode::ControlLeft),
            ExtendedAction::YawLeft => Some(KeyCode::KeyQ),
            ExtendedAction::YawRight => Some(KeyCode::KeyE),
            ExtendedAction::PitchUp => Some(KeyCode::ArrowUp),
            ExtendedAction::PitchDown => Some(KeyCode::ArrowDown),
            ExtendedAction::RollLeft => Some(KeyCode::ArrowLeft),
            ExtendedAction::RollRight => Some(KeyCode::ArrowRight),
            ExtendedAction::Sprint => Some(KeyCode::ShiftLeft),
            ExtendedAction::Crouch => Some(KeyCode::KeyC),
            ExtendedAction::Jump => Some(KeyCode::Space),
            ExtendedAction::Interact => Some(KeyCode::KeyF),
            ExtendedAction::OpenMenu => Some(KeyCode::Escape),
            ExtendedAction::OpenMap => Some(KeyCode::KeyM),
            ExtendedAction::OpenInventory => Some(KeyCode::KeyI),
            ExtendedAction::Pause => Some(KeyCode::Pause),
            ExtendedAction::FirePrimary => Some(KeyCode::KeyZ),
            ExtendedAction::FireSecondary => Some(KeyCode::KeyX),
            ExtendedAction::Reload => Some(KeyCode::KeyR),
            ExtendedAction::UseItem => Some(KeyCode::KeyU),
            ExtendedAction::Brake => Some(KeyCode::Space),
            ExtendedAction::Handbrake => Some(KeyCode::KeyH),
            ExtendedAction::Horn => Some(KeyCode::KeyK),
        }
    }

    pub fn default_gamepad_button(&self) -> Option<GamepadButton> {
        match self {
            ExtendedAction::FirePrimary => Some(GamepadButton::RightTrigger),
            ExtendedAction::FireSecondary => Some(GamepadButton::LeftTrigger),
            ExtendedAction::Jump => Some(GamepadButton::A),
            ExtendedAction::Crouch => Some(GamepadButton::B),
            ExtendedAction::Interact => Some(GamepadButton::X),
            ExtendedAction::OpenMenu => Some(GamepadButton::Start),
            ExtendedAction::OpenMap => Some(GamepadButton::Back),
            _ => None,
        }
    }
}

/// Manages action states and input mapping
pub struct ActionMap {
    /// Current state of each action
    action_states: HashMap<ExtendedAction, ActionState>,
    /// Previous state of each action (for edge detection)
    previous_states: HashMap<ExtendedAction, ActionState>,
    /// Key to action mapping
    key_to_action: HashMap<KeyCode, ExtendedAction>,
    /// Mouse button to action mapping
    mouse_to_action: HashMap<MouseButton, ExtendedAction>,
    /// Gamepad button to action mapping
    gamepad_to_action: HashMap<GamepadButton, ExtendedAction>,
    /// Reverse mappings
    action_to_key: HashMap<ExtendedAction, KeyCode>,
    action_to_mouse: HashMap<ExtendedAction, MouseButton>,
    action_to_gamepad: HashMap<ExtendedAction, GamepadButton>,
}

impl ActionMap {
    pub fn new() -> Self {
        let mut map = Self {
            action_states: HashMap::new(),
            previous_states: HashMap::new(),
            key_to_action: HashMap::new(),
            mouse_to_action: HashMap::new(),
            gamepad_to_action: HashMap::new(),
            action_to_key: HashMap::new(),
            action_to_mouse: HashMap::new(),
            action_to_gamepad: HashMap::new(),
        };

        // Initialize default bindings
        map.initialize_default_bindings();

        map
    }

    fn initialize_default_bindings(&mut self) {
        for action in [
            ExtendedAction::MoveForward,
            ExtendedAction::MoveBackward,
            ExtendedAction::MoveLeft,
            ExtendedAction::MoveRight,
            ExtendedAction::ThrottleUp,
            ExtendedAction::ThrottleDown,
            ExtendedAction::YawLeft,
            ExtendedAction::YawRight,
            ExtendedAction::PitchUp,
            ExtendedAction::PitchDown,
            ExtendedAction::RollLeft,
            ExtendedAction::RollRight,
            ExtendedAction::Sprint,
            ExtendedAction::Crouch,
            ExtendedAction::Jump,
            ExtendedAction::Interact,
            ExtendedAction::OpenMenu,
            ExtendedAction::OpenMap,
            ExtendedAction::OpenInventory,
            ExtendedAction::Pause,
            ExtendedAction::FirePrimary,
            ExtendedAction::FireSecondary,
            ExtendedAction::Reload,
            ExtendedAction::UseItem,
            ExtendedAction::Brake,
            ExtendedAction::Handbrake,
            ExtendedAction::Horn,
        ] {
            if let Some(key) = action.default_key_code() {
                self.bind_key(action, key);
            }
            if let Some(button) = action.default_gamepad_button() {
                self.bind_gamepad_button(action, button);
            }
        }

        // Default mouse bindings
        self.bind_mouse(ExtendedAction::FirePrimary, MouseButton::Left);
        self.bind_mouse(ExtendedAction::FireSecondary, MouseButton::Right);
    }

    /// Bind a key to an action
    pub fn bind_key(&mut self, action: ExtendedAction, key: KeyCode) {
        // Remove old binding if exists
        if let Some(old_key) = self.action_to_key.get(&action) {
            self.key_to_action.remove(old_key);
        }
        // Remove any action bound to this key
        if let Some(old_action) = self.key_to_action.get(&key) {
            self.action_to_key.remove(old_action);
        }

        self.key_to_action.insert(key, action);
        self.action_to_key.insert(action, key);
    }

    /// Bind a mouse button to an action
    pub fn bind_mouse(&mut self, action: ExtendedAction, button: MouseButton) {
        self.mouse_to_action.insert(button, action);
        self.action_to_mouse.insert(action, button);
    }

    /// Bind a gamepad button to an action
    pub fn bind_gamepad_button(&mut self, action: ExtendedAction, button: GamepadButton) {
        self.gamepad_to_action.insert(button, action);
        self.action_to_gamepad.insert(action, button);
    }

    /// Process keyboard input
    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) {
        if let Some(&action) = self.key_to_action.get(&key) {
            match state {
                ElementState::Pressed => {
                    let current_state = self.action_states.entry(action).or_insert(ActionState::Released);
                    if *current_state == ActionState::Released || *current_state == ActionState::JustReleased {
                        *current_state = ActionState::JustPressed;
                    } else {
                        *current_state = ActionState::Held;
                    }
                }
                ElementState::Released => {
                    let current_state = self.action_states.entry(action).or_insert(ActionState::Released);
                    *current_state = ActionState::JustReleased;
                }
            }
        }
    }

    /// Process mouse input
    pub fn process_mouse(&mut self, button: MouseButton, state: ElementState) {
        if let Some(&action) = self.mouse_to_action.get(&button) {
            match state {
                ElementState::Pressed => {
                    let current_state = self.action_states.entry(action).or_insert(ActionState::Released);
                    if *current_state == ActionState::Released || *current_state == ActionState::JustReleased {
                        *current_state = ActionState::JustPressed;
                    } else {
                        *current_state = ActionState::Held;
                    }
                }
                ElementState::Released => {
                    let current_state = self.action_states.entry(action).or_insert(ActionState::Released);
                    *current_state = ActionState::JustReleased;
                }
            }
        }
    }

    /// Process gamepad input
    pub fn process_gamepad(&mut self, button: GamepadButton, pressed: bool) {
        if let Some(&action) = self.gamepad_to_action.get(&button) {
            let current_state = self.action_states.entry(action).or_insert(ActionState::Released);
            if pressed {
                if *current_state == ActionState::Released || *current_state == ActionState::JustReleased {
                    *current_state = ActionState::JustPressed;
                } else {
                    *current_state = ActionState::Held;
                }
            } else {
                *current_state = ActionState::JustReleased;
            }
        }
    }

    /// Update action states (call at end of frame)
    pub fn update(&mut self) {
        // Save current states as previous
        self.previous_states = self.action_states.clone();

        // Transition states
        for (&_action, state) in &mut self.action_states {
            match state {
                ActionState::JustPressed => *state = ActionState::Held,
                ActionState::JustReleased => *state = ActionState::Released,
                _ => {}
            }
        }
    }

    /// Check if action is just pressed
    pub fn is_action_just_pressed(&self, action: ExtendedAction) -> bool {
        self.action_states.get(&action) == Some(&ActionState::JustPressed)
    }

    /// Check if action is held
    pub fn is_action_held(&self, action: ExtendedAction) -> bool {
        matches!(
            self.action_states.get(&action),
            Some(ActionState::Held | ActionState::JustPressed)
        )
    }

    /// Check if action is just released
    pub fn is_action_released(&self, action: ExtendedAction) -> bool {
        self.action_states.get(&action) == Some(&ActionState::JustReleased)
    }

    /// Get action state
    pub fn get_action_state(&self, action: ExtendedAction) -> ActionState {
        *self.action_states.get(&action).unwrap_or(&ActionState::Released)
    }

    /// Get all active actions
    pub fn get_active_actions(&self) -> Vec<ExtendedAction> {
        self.action_states
            .iter()
            .filter(|(_, state)| matches!(state, ActionState::Held | ActionState::JustPressed))
            .map(|(&action, _)| action)
            .collect()
    }

    /// Reset all action states
    pub fn reset(&mut self) {
        self.action_states.clear();
        self.previous_states.clear();
    }

    /// Get debug info for all actions
    pub fn get_debug_info(&self) -> Vec<(String, String)> {
        let mut info = Vec::new();
        for &action in self.action_to_key.keys() {
            let state = self.get_action_state(action);
            let key = self.action_to_key.get(&action);
            info.push((
                action.name().to_string(),
                format!("{:?} ({:?})", state, key),
            ));
        }
        info
    }
}

impl Default for ActionMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_map_basic() {
        let mut map = ActionMap::new();
        
        // Initially should be released
        assert_eq!(map.get_action_state(ExtendedAction::MoveForward), ActionState::Released);
        
        // Simulate key press
        map.process_keyboard(KeyCode::KeyW, ElementState::Pressed);
        assert!(map.is_action_just_pressed(ExtendedAction::MoveForward));
        
        // Update should transition to held
        map.update();
        assert!(map.is_action_held(ExtendedAction::MoveForward));
        
        // Simulate key release
        map.process_keyboard(KeyCode::KeyW, ElementState::Released);
        assert!(map.is_action_released(ExtendedAction::MoveForward));
    }

    #[test]
    fn test_action_map_gamepad() {
        let mut map = ActionMap::new();
        
        // Simulate gamepad button press
        map.process_gamepad(GamepadButton::A, true);
        assert!(map.is_action_just_pressed(ExtendedAction::Jump));
        
        map.update();
        assert!(map.is_action_held(ExtendedAction::Jump));
        
        map.process_gamepad(GamepadButton::A, false);
        assert!(map.is_action_released(ExtendedAction::Jump));
    }
}
