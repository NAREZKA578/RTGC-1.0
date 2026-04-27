//! UI System for RTGC-0.9
//! Handles HUD, menus, tooltips, and all user interface elements

use crate::game::player::{CameraMode, PlayerState};
use crate::game::weather::WeatherState;
use crate::graphics::render_queue::RenderQueue;
use crate::graphics::UiCommand;
use nalgebra::{Vector2, Vector3};

// Type aliases for backwards compatibility
type Vec2 = Vector2<f32>;
type Vec3 = Vector3<f32>;

/// UI visibility flags
#[derive(Debug, Clone, Copy)]
pub struct UIVisibility {
    pub hud: bool,
    pub crosshair: bool,
    pub interaction_prompt: bool,
    pub minimap: bool,
    pub speedometer: bool,
    pub fuel_gauge: bool,
    pub compass: bool,
    pub clock: bool,
    pub notifications: bool,
    pub debug_overlay: bool,
}

impl Default for UIVisibility {
    fn default() -> Self {
        Self {
            hud: true,
            crosshair: true,
            interaction_prompt: true,
            minimap: true,
            speedometer: false,
            fuel_gauge: false,
            compass: true,
            clock: true,
            notifications: true,
            debug_overlay: false,
        }
    }
}

/// HUD state data
#[derive(Debug, Clone)]
pub struct HUDData {
    /// Player health (0.0 - 100.0)
    pub health: f32,
    /// Player stamina (0.0 - 100.0)
    pub stamina: f32,
    /// Current speed (km/h)
    pub speed_kmh: f32,
    /// Fuel level (0.0 - 1.0)
    pub fuel: f32,
    /// Money (rubles)
    pub money: u32,
    /// Current time (hours, 0-24)
    pub time_hours: f32,
    /// Weather description
    pub weather: String,
    /// Location name
    pub location: String,
    /// Player state
    pub player_state: PlayerState,
    /// Camera mode
    pub camera_mode: CameraMode,
    /// Current gear (for vehicles)
    pub gear: i8,
    /// RPM (for vehicles)
    pub rpm: f32,
    /// Engine temperature (0.0 - 1.0, normal is 0.3-0.6)
    pub engine_temp: f32,
    /// Compass heading (0-359 degrees)
    pub heading: f32,
    /// Coordinates (X, Y, Z)
    pub position: Vector2<f32>,
    /// Altitude (meters)
    pub altitude: f32,
}

impl Default for HUDData {
    fn default() -> Self {
        Self {
            health: 100.0,
            stamina: 100.0,
            speed_kmh: 0.0,
            fuel: 1.0,
            money: 50000,
            time_hours: 12.0,
            weather: "Clear".to_string(),
            location: "Novosibirsk".to_string(),
            player_state: PlayerState::OnFoot,
            camera_mode: CameraMode::ThirdPerson {
                distance: 4.0,
                yaw: 0.0,
                pitch: 0.3,
            },
            gear: 0,
            rpm: 0.0,
            engine_temp: 0.5,
            heading: 0.0,
            position: Vector2::zeros(),
            altitude: 0.0,
        }
    }
}

/// Notification message
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub duration: f32,
    pub age: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
    SkillUp,
    Achievement,
}

/// Interaction prompt data
#[derive(Debug, Clone)]
pub struct InteractionPrompt {
    pub visible: bool,
    pub text: String,
    pub key: String,
    pub distance: f32,
}

/// Minimap data
#[derive(Debug, Clone)]
pub struct MinimapData {
    /// Player position on map (normalized 0-1)
    pub player_pos: Vector2<f32>,
    /// Player rotation (radians)
    pub player_rotation: f32,
    /// Zoom level (1.0 = max zoom)
    pub zoom: f32,
    /// Marked waypoints
    pub waypoints: Vec<MapWaypoint>,
    /// Visible vehicles
    pub vehicles: Vec<VehicleMarker>,
    /// Visible NPCs
    pub npcs: Vec<NPCMarker>,
}

#[derive(Debug, Clone)]
pub struct MapWaypoint {
    pub name: String,
    pub position: Vector2<f32>,
    pub waypoint_type: WaypointType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaypointType {
    Mission,
    Custom,
    Home,
    Work,
    FuelStation,
    Shop,
}

#[derive(Debug, Clone)]
pub struct VehicleMarker {
    pub vehicle_id: u32,
    pub position: Vector2<f32>,
    pub vehicle_type: String,
    pub is_player_owned: bool,
}

#[derive(Debug, Clone)]
pub struct NPCMarker {
    pub npc_id: u32,
    pub position: Vector2<f32>,
    pub name: String,
}

impl Default for MinimapData {
    fn default() -> Self {
        Self {
            player_pos: Vec2::new(0.5, 0.5),
            player_rotation: 0.0,
            zoom: 1.0,
            waypoints: Vec::new(),
            vehicles: Vec::new(),
            npcs: Vec::new(),
        }
    }
}

/// Main UI manager
#[derive(Clone)]
pub struct UIManager {
    visibility: UIVisibility,
    hud_data: HUDData,
    notifications: Vec<Notification>,
    interaction_prompt: Option<InteractionPrompt>,
    minimap_data: MinimapData,
    /// Active skill notifications
    skill_notifications: Vec<(String, u32)>, // (skill_name, new_rank)
}

impl UIManager {
    pub fn new() -> Self {
        Self {
            visibility: UIVisibility::default(),
            hud_data: HUDData::default(),
            notifications: Vec::new(),
            interaction_prompt: None,
            minimap_data: MinimapData::default(),
            skill_notifications: Vec::new(),
        }
    }

    /// Update UI systems
    pub fn update(&mut self, dt: f32) {
        // Update notifications
        for notification in &mut self.notifications {
            notification.age += dt;
        }
        self.notifications.retain(|n| n.age < n.duration);

        // Update skill notifications
        // (would fade out over time in actual rendering)

        // Update interaction prompt visibility based on distance
        if let Some(prompt) = &mut self.interaction_prompt {
            if prompt.distance > 3.0 {
                prompt.visible = false;
            }
        }
    }

    /// Add a notification
    pub fn add_notification(&mut self, message: String, notification_type: NotificationType) {
        let duration = match notification_type {
            NotificationType::Info => 3.0,
            NotificationType::Success => 4.0,
            NotificationType::Warning => 5.0,
            NotificationType::Error => 6.0,
            NotificationType::SkillUp => 5.0,
            NotificationType::Achievement => 8.0,
        };

        self.notifications.push(Notification {
            message,
            notification_type,
            duration,
            age: 0.0,
        });
    }

    /// Add skill up notification
    pub fn notify_skill_up(&mut self, skill_name: String, new_rank: u32) {
        self.skill_notifications
            .push((skill_name.clone(), new_rank));
        self.add_notification(
            format!("{} increased to Rank {}!", skill_name, new_rank),
            NotificationType::SkillUp,
        );
    }

    /// Set interaction prompt
    pub fn set_interaction_prompt(&mut self, text: String, distance: f32) {
        self.interaction_prompt = Some(InteractionPrompt {
            visible: true,
            text,
            key: "F".to_string(),
            distance,
        });
        self.visibility.interaction_prompt = true;
    }

    /// Clear interaction prompt
    pub fn clear_interaction_prompt(&mut self) {
        self.interaction_prompt = None;
        self.visibility.interaction_prompt = false;
    }

    /// Update HUD data
    pub fn update_hud(&mut self, data: HUDData) {
        // Check player state before moving data
        let is_in_vehicle = matches!(data.player_state, PlayerState::InVehicle { .. });

        self.hud_data = data;

        // Auto-show speedometer when in vehicle
        if is_in_vehicle {
            self.visibility.speedometer = true;
            self.visibility.fuel_gauge = true;
        } else {
            self.visibility.speedometer = false;
            self.visibility.fuel_gauge = false;
        }
    }

    /// Get current HUD data
    pub fn get_hud_data(&self) -> &HUDData {
        &self.hud_data
    }

    /// Update minimap data
    pub fn update_minimap(&mut self, data: MinimapData) {
        self.minimap_data = data;
    }

    /// Get minimap data
    pub fn get_minimap_data(&self) -> &MinimapData {
        &self.minimap_data
    }

    /// Toggle HUD visibility
    pub fn toggle_hud(&mut self) {
        self.visibility.hud = !self.visibility.hud;
    }

    /// Toggle minimap
    pub fn toggle_minimap(&mut self) {
        self.visibility.minimap = !self.visibility.minimap;
    }

    /// Toggle debug overlay
    pub fn toggle_debug_overlay(&mut self) {
        self.visibility.debug_overlay = !self.visibility.debug_overlay;
    }

    /// Get UI visibility
    pub fn get_visibility(&self) -> UIVisibility {
        self.visibility
    }

    /// Get active notifications
    pub fn get_notifications(&self) -> &[Notification] {
        &self.notifications
    }

    /// Get interaction prompt
    pub fn get_interaction_prompt(&self) -> Option<&InteractionPrompt> {
        self.interaction_prompt.as_ref()
    }

    /// Handle weather change notification
    pub fn notify_weather_change(&mut self, weather: &WeatherState) {
        self.add_notification(
            format!("Weather changed: {}", weather.description()),
            NotificationType::Info,
        );
        self.hud_data.weather = weather.description().to_string();
    }

    /// Handle money change
    pub fn notify_money_change(&mut self, amount: i32, reason: &str) {
        if amount > 0 {
            self.add_notification(
                format!("+{} ₽ ({})", amount, reason),
                NotificationType::Success,
            );
        } else if amount < 0 {
            self.add_notification(
                format!("-{} ₽ ({})", -amount, reason),
                NotificationType::Warning,
            );
        }
    }

    /// Reset UI state (on scene change)
    pub fn reset(&mut self) {
        self.notifications.clear();
        self.interaction_prompt = None;
        self.skill_notifications.clear();
        self.visibility = UIVisibility::default();
    }

    /// Проблема 3: Submit UI commands to render queue
    /// Отрисовка уведомлений и HUD через RenderCommand::UIElement и RenderCommand::UIText
    pub fn submit_ui_commands(
        &self,
        render_queue: &mut RenderQueue,
        screen_width: f32,
        screen_height: f32,
    ) {
        // Отрисовка уведомлений
        if self.visibility.notifications {
            let mut y_offset = 10.0;
            for notification in &self.notifications {
                let alpha = 1.0 - (notification.age / notification.duration).min(1.0);
                let color = match notification.notification_type {
                    NotificationType::Info => [0.2, 0.6, 1.0, alpha],
                    NotificationType::Success => [0.2, 0.8, 0.2, alpha],
                    NotificationType::Warning => [1.0, 0.8, 0.0, alpha],
                    NotificationType::Error => [1.0, 0.2, 0.2, alpha],
                    NotificationType::SkillUp => [1.0, 0.5, 0.0, alpha],
                    NotificationType::Achievement => [1.0, 0.8, 0.5, alpha],
                };

                 // Фон уведомления
                 render_queue.add_ui_command(UiCommand::Rect {
                     position: [10.0, screen_height - y_offset - 30.0],
                     size: [300.0, 25.0],
                     color: [0.0, 0.0, 0.0, 0.7 * alpha],
                 });
                 
                 // Текст уведомления
                 render_queue.add_ui_command(UiCommand::Text {
                     text: notification.message.clone(),
                     position: [20.0, screen_height - y_offset - 25.0],
                     font_size: 14.0,
                     color: [1.0, 1.0, 1.0, alpha],
                 });
                y_offset += 35.0;
            }
        }

        // Отрисовка interaction prompt
        if self.visibility.interaction_prompt {
            if let Some(prompt) = &self.interaction_prompt {
                if prompt.visible {
                     render_queue.add_ui_command(UiCommand::Rect {
                         position: [
                             screen_width / 2.0 - 100.0,
                             screen_height / 2.0 + 50.0,
                         ],
                         size: [200.0, 30.0],
                         color: [0.0, 0.0, 0.0, 0.7],
                     });
                     
                     render_queue.add_ui_command(UiCommand::Text {
                         text: prompt.text.clone(),
                         position: [screen_width / 2.0 - 90.0, screen_height / 2.0 + 55.0],
                         font_size: 16.0,
                         color: [1.0, 1.0, 1.0, 1.0],
                     });
                }
            }
        }

        // Отрисовка HUD элементов (speedometer, fuel, etc.)
        if self.visibility.hud {
            // Speedometer background
            if self.visibility.speedometer {
                 // Speed text
                 render_queue.add_ui_command(UiCommand::Text {
                     text: format!("{:.0} км/ч", self.hud_data.speed_kmh),
                     position: [screen_width - 200.0, 20.0],
                     font_size: 32.0,
                     color: [1.0, 1.0, 1.0, 1.0],
                 });
                
                 // Gear and RPM
                 render_queue.add_ui_command(UiCommand::Text {
                     text: format!("{} {:.0}", 
                         if self.hud_data.gear > 0 { 
                             self.hud_data.gear.to_string() 
                         } else { 
                             "N".to_string() 
                         },
                         self.hud_data.rpm),
                     position: [screen_width - 200.0, 55.0],
                     font_size: 18.0,
                     color: [1.0, 1.0, 1.0, 1.0],
                 });
            }

             // Fuel gauge background
             if self.visibility.fuel_gauge {
                 render_queue.add_ui_command(UiCommand::Rect {
                     position: [screen_width - 210.0, 100.0],
                     size: [200.0, 30.0],
                     color: [0.0, 0.0, 0.0, 0.5],
                 });
                 
                 // Fuel level bar
                 let fuel_width = 180.0 * self.hud_data.fuel;
                 let fuel_color = if self.hud_data.fuel < 0.2 {
                     [1.0, 0.2, 0.2, 1.0]
                 } else if self.hud_data.fuel < 0.5 {
                     [1.0, 0.8, 0.0, 1.0]
                 } else {
                     [0.2, 0.8, 0.2, 1.0]
                 };
                 
                 render_queue.add_ui_command(UiCommand::Rect {
                     position: [screen_width - 200.0, 105.0],
                     size: [fuel_width, 20.0],
                     color: fuel_color,
                 });
                 
                 // Fuel percentage text
                 render_queue.add_ui_command(UiCommand::Text {
                     text: format!("{:.0}%", self.hud_data.fuel * 100.0),
                     position: [screen_width - 200.0, 107.0],
                     font_size: 14.0,
                     color: [1.0, 1.0, 1.0, 1.0],
                 });
                
                // Fuel level bar
                let fuel_width = 180.0 * self.hud_data.fuel;
                let fuel_color = if self.hud_data.fuel < 0.2 {
                    [1.0, 0.2, 0.2, 1.0]
                } else if self.hud_data.fuel < 0.5 {
                    [1.0, 0.8, 0.0, 1.0]
                } else {
                    [0.2, 0.8, 0.2, 1.0]
                };
                
                 render_queue.add_ui_command(UiCommand::Rect {
                     position: [screen_width - 200.0, 105.0],
                     size: [fuel_width, 20.0],
                     color: fuel_color,
                 });
                 
                 // Fuel percentage text
                 render_queue.add_ui_command(UiCommand::Text {
                     text: format!("{:.0}%", self.hud_data.fuel * 100.0),
                     position: [screen_width - 200.0, 107.0],
                     font_size: 14.0,
                     color: [1.0, 1.0, 1.0, 1.0],
                 });
            }

            // Compass background
            if self.visibility.compass {
                 render_queue.add_ui_command(UiCommand::Rect {
                     position: [screen_width / 2.0 - 100.0, 10.0],
                     size: [200.0, 30.0],
                     color: [0.0, 0.0, 0.0, 0.5],
                 });
                 
// Compass heading
                 let heading_dir = match (self.hud_data.heading / 22.5) as i32 % 16 {
                     0 | 16 => "С",
                     1 => "ССВ",
                     2 => "СВ",
                     3 => "ВСВ",
                     4 => "В",
                     5 => "ВЮВ",
                     6 => "ЮВ",
                     7 => "ЮЮВ",
                     8 => "Ю",
                     9 => "ЮЮЗ",
                     10 => "ЮЗ",
                     11 => "ЗЮЗ",
                     12 => "З",
                     13 => "ЗСЗ",
                     14 => "СЗ",
                     15 => "ССЗ",
                     _ => "?",
                 };
                 
                 render_queue.add_ui_command(UiCommand::Text {
                     text: format!("{} {:.0}°", heading_dir, self.hud_data.heading),
                     position: [screen_width / 2.0 - 60.0, 15.0],
                     font_size: 18.0,
                     color: [1.0, 1.0, 1.0, 1.0],
                 });
             }

            // Clock
            if self.visibility.clock {
                 render_queue.add_ui_command(UiCommand::Rect {
                     position: [screen_width - 100.0, screen_height - 40.0],
                     size: [90.0, 30.0],
                     color: [0.0, 0.0, 0.0, 0.5],
                 });
                 
                 // Time text
                 let hours = self.hud_data.time_hours as i32;
                 let minutes = ((self.hud_data.time_hours - hours as f32) * 60.0) as i32;
                 render_queue.add_ui_command(UiCommand::Text {
                     text: format!("{:02}:{:02}", hours, minutes),
                     position: [screen_width - 90.0, screen_height - 35.0],
                     font_size: 16.0,
                     color: [1.0, 1.0, 1.0, 1.0],
                 });
            }
            
             // Money display
             render_queue.add_ui_command(UiCommand::Text {
                 text: format!("{} ₽", self.hud_data.money),
                 position: [10.0, screen_height - 35.0],
                 font_size: 18.0,
                 color: [0.2, 1.0, 0.2, 1.0],
             });
            
             // Weather display
             render_queue.add_ui_command(UiCommand::Text {
                 text: self.hud_data.weather.clone(),
                 position: [10.0, screen_height - 60.0],
                 font_size: 14.0,
                 color: [1.0, 1.0, 1.0, 1.0],
             });
        }
    }
}

impl Default for UIManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_manager_creation() {
        let ui = UIManager::new();
        assert!(ui.get_visibility().hud);
        assert!(ui.get_notifications().is_empty());
    }

    #[test]
    fn test_add_notification() {
        let mut ui = UIManager::new();
        ui.add_notification("Test".to_string(), NotificationType::Info);
        assert_eq!(ui.get_notifications().len(), 1);
    }

    #[test]
    fn test_toggle_hud() {
        let mut ui = UIManager::new();
        assert!(ui.get_visibility().hud);
        ui.toggle_hud();
        assert!(!ui.get_visibility().hud);
    }

    #[test]
    fn test_interaction_prompt() {
        let mut ui = UIManager::new();
        ui.set_interaction_prompt("Enter Vehicle".to_string(), 2.0);
        assert!(ui.get_interaction_prompt().is_some());

        ui.clear_interaction_prompt();
        assert!(ui.get_interaction_prompt().is_none());
    }
}
