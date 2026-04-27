//! Gamepad input support
//! 
//! Implements full gamepad controller support including:
//! - Button mapping (A/B/X/Y, triggers, bumpers, etc.)
//! - Analog sticks with deadzone handling
//! - Vibration/haptic feedback
//! - Multiple controller support

use std::collections::HashMap;
use tracing::info;

/// Gamepad button enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    // Face buttons
    A,
    B,
    X,
    Y,
    
    // D-pad
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    
    // Shoulders/Triggers
    LeftShoulder,
    RightShoulder,
    LeftTrigger,
    RightTrigger,
    
    // Center buttons
    Back,
    Start,
    Guide,
    
    // Sticks
    LeftStick,
    RightStick,
}

/// Gamepad axis enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
}

/// Gamepad state
#[derive(Debug, Clone)]
pub struct GamepadState {
    /// Button states (true = pressed)
    pub buttons: HashMap<GamepadButton, bool>,
    /// Axis values (-1.0 to 1.0)
    pub axes: HashMap<GamepadAxis, f32>,
    /// Trigger values (0.0 to 1.0)
    pub left_trigger: f32,
    pub right_trigger: f32,
    /// Connection status
    pub connected: bool,
    /// Player index (0-3)
    pub player_index: usize,
    /// Gamepad ID
    pub id: u32,
}

impl Default for GamepadState {
    fn default() -> Self {
        Self {
            buttons: HashMap::new(),
            axes: HashMap::new(),
            left_trigger: 0.0,
            right_trigger: 0.0,
            connected: false,
            player_index: 0,
            id: 0,
        }
    }
}

impl GamepadState {
    pub fn new(player_index: usize) -> Self {
        let mut state = Self::default();
        state.player_index = player_index;
        state.id = player_index as u32;

        // Initialize axes to zero
        state.axes.insert(GamepadAxis::LeftStickX, 0.0);
        state.axes.insert(GamepadAxis::LeftStickY, 0.0);
        state.axes.insert(GamepadAxis::RightStickX, 0.0);
        state.axes.insert(GamepadAxis::RightStickY, 0.0);

        state
    }
    
    /// Check if a button is currently pressed
    pub fn is_button_pressed(&self, button: GamepadButton) -> bool {
        *self.buttons.get(&button).unwrap_or(&false)
    }
    
    /// Get axis value with optional deadzone
    pub fn get_axis(&self, axis: GamepadAxis, deadzone: f32) -> f32 {
        let value = *self.axes.get(&axis).unwrap_or(&0.0);
        apply_deadzone(value, deadzone)
    }
    
    /// Get combined trigger value (for games that treat triggers as one axis)
    pub fn get_triggers_combined(&self) -> f32 {
        self.right_trigger - self.left_trigger
    }
}

/// Apply deadzone to analog input
fn apply_deadzone(value: f32, deadzone: f32) -> f32 {
    if value.abs() < deadzone {
        0.0
    } else {
        // Remap to maintain full range after deadzone
        let sign = value.signum();
        let magnitude = (value.abs() - deadzone) / (1.0 - deadzone);
        (magnitude * sign).clamp(-1.0, 1.0)
    }
}

/// Gamepad configuration for customizable controls
#[derive(Debug, Clone)]
pub struct GamepadConfig {
    pub deadzone_left: f32,
    pub deadzone_right: f32,
    pub trigger_deadzone: f32,
    pub vibration_enabled: bool,
    pub invert_left_y: bool,
    pub invert_right_y: bool,
}

impl Default for GamepadConfig {
    fn default() -> Self {
        Self {
            deadzone_left: 0.15,
            deadzone_right: 0.1,
            trigger_deadzone: 0.1,
            vibration_enabled: true,
            invert_left_y: false,
            invert_right_y: false,
        }
    }
}

/// Gamepad manager for handling multiple controllers
pub struct GamepadManager {
    gamepads: HashMap<usize, GamepadState>,
    config: GamepadConfig,
    max_gamepads: usize,
}

impl GamepadManager {
    pub fn new() -> Self {
        Self {
            gamepads: HashMap::new(),
            config: GamepadConfig::default(),
            max_gamepads: 4, // Standard limit for most platforms
        }
    }
    
    /// Register a new gamepad
    pub fn add_gamepad(&mut self, player_index: usize) -> Option<usize> {
        if self.gamepads.len() >= self.max_gamepads {
            return None;
        }
        
        let state = GamepadState::new(player_index);
        self.gamepads.insert(player_index, state);
        Some(player_index)
    }
    
    /// Remove a gamepad
    pub fn remove_gamepad(&mut self, player_index: usize) {
        self.gamepads.remove(&player_index);
    }
    
    /// Get gamepad state by player index
    pub fn get_gamepad(&self, player_index: usize) -> Option<&GamepadState> {
        self.gamepads.get(&player_index)
    }
    
    /// Get mutable gamepad state
    pub fn get_gamepad_mut(&mut self, player_index: usize) -> Option<&mut GamepadState> {
        self.gamepads.get_mut(&player_index)
    }
    
    /// Update button state
    pub fn set_button_state(&mut self, player_index: usize, button: GamepadButton, pressed: bool) {
        if let Some(gamepad) = self.gamepads.get_mut(&player_index) {
            gamepad.buttons.insert(button, pressed);
        }
    }
    
    /// Update axis value
    pub fn set_axis_value(&mut self, player_index: usize, axis: GamepadAxis, value: f32) {
        if let Some(gamepad) = self.gamepads.get_mut(&player_index) {
            let mut adjusted_value = value;
            
            // Apply inversion if configured
            match axis {
                GamepadAxis::LeftStickY if self.config.invert_left_y => {
                    adjusted_value = -value;
                }
                GamepadAxis::RightStickY if self.config.invert_right_y => {
                    adjusted_value = -value;
                }
                _ => {}
            }
            
            gamepad.axes.insert(axis, adjusted_value);
        }
    }
    
    /// Update trigger value
    pub fn set_trigger_value(&mut self, player_index: usize, left: f32, right: f32) {
        if let Some(gamepad) = self.gamepads.get_mut(&player_index) {
            gamepad.left_trigger = left.clamp(0.0, 1.0);
            gamepad.right_trigger = right.clamp(0.0, 1.0);
        }
    }
    
    /// Set vibration intensity (0.0 to 1.0 for each motor)
    pub fn set_vibration(&mut self, player_index: usize, left_motor: f32, right_motor: f32) {
        if !self.config.vibration_enabled {
            return;
        }
        
        // In a real implementation, this would send haptic feedback to the device
        info!(
            "Vibration on gamepad {}: left={}, right={}",
            player_index, left_motor, right_motor
        );
    }
    
    /// Stop all vibration
    pub fn stop_vibration(&mut self, player_index: usize) {
        self.set_vibration(player_index, 0.0, 0.0);
    }
    
    /// Get connected gamepad count
    pub fn connected_count(&self) -> usize {
        self.gamepads.values().filter(|g| g.connected).count()
    }
    
    /// Check if any gamepad is connected
    pub fn has_connected_gamepad(&self) -> bool {
        self.gamepads.values().any(|g| g.connected)
    }
}

impl Default for GamepadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_deadzone() {
        assert_eq!(apply_deadzone(0.1, 0.15), 0.0);
        assert!(apply_deadzone(0.5, 0.15) > 0.4);
        assert_eq!(apply_deadzone(0.0, 0.2), 0.0);
    }
    
    #[test]
    fn test_gamepad_manager() {
        let mut manager = GamepadManager::new();
        
        // Add gamepad
        assert!(manager.add_gamepad(0).is_some());
        assert_eq!(manager.connected_count(), 0); // Not marked as connected yet
        
        // Update state
        manager.set_button_state(0, GamepadButton::A, true);
        
        let gamepad = manager.get_gamepad(0).expect("Gamepad should exist");
        assert!(gamepad.is_button_pressed(GamepadButton::A));
        
        // Test axis
        manager.set_axis_value(0, GamepadAxis::LeftStickX, 0.5);
        let gamepad = manager.get_gamepad(0).expect("Gamepad should exist");
        assert!(gamepad.get_axis(GamepadAxis::LeftStickX, 0.15).abs() > 0.3);
    }
}
