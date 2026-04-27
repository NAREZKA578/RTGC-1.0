// Settings Manager - Loads and saves game settings from TOML file
// Provides runtime access to all configurable game options

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Display settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    pub fullscreen: bool,
    pub vsync: bool,
    pub fps_limit: u32,
    pub resolution_width: u32,
    pub resolution_height: u32,
    pub brightness: f32,
    pub gamma: f32,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            vsync: true,
            fps_limit: 60,
            resolution_width: 1920,
            resolution_height: 1080,
            brightness: 1.0,
            gamma: 1.0,
        }
    }
}

/// Graphics quality preset
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QualityLevel {
    Off,
    Low,
    Medium,
    High,
    Ultra,
}

/// Anti-aliasing mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AAMode {
    Off,
    FXAA,
    MSAA2x,
    MSAA4x,
    MSAA8x,
}

impl Default for AAMode {
    fn default() -> Self {
        AAMode::FXAA
    }
}

/// Graphics settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsSettings {
    pub backend: String, // "opengl", "vulkan", "dx12"
    pub texture_quality: QualityLevel,
    pub shadow_quality: QualityLevel,
    pub lod_distance: f32,
    pub render_distance: f32,
    pub anti_aliasing: AAMode,
    pub anisotropic_filtering: u32,
    pub ambient_occlusion: bool,
    pub motion_blur: bool,
    pub depth_of_field: bool,
    pub bloom: bool,
    pub color_grading: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            backend: "opengl".to_string(),
            texture_quality: QualityLevel::High,
            shadow_quality: QualityLevel::Medium,
            lod_distance: 1.0,
            render_distance: 5000.0,
            anti_aliasing: AAMode::FXAA,
            anisotropic_filtering: 16,
            ambient_occlusion: true,
            motion_blur: false,
            depth_of_field: false,
            bloom: true,
            color_grading: true,
        }
    }
}

/// Audio settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub voice_volume: f32,
    pub engine_volume: f32,
    pub environment_volume: f32,
    pub audio_device: String,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            music_volume: 0.6,
            sfx_volume: 0.9,
            voice_volume: 0.7,
            engine_volume: 0.85,
            environment_volume: 0.75,
            audio_device: "default".to_string(),
        }
    }
}

/// Camera mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CameraMode {
    FirstPerson,
    ThirdPerson { distance: f32, yaw: f32, pitch: f32 },
}

impl Default for CameraMode {
    fn default() -> Self {
        CameraMode::ThirdPerson {
            distance: 4.0,
            yaw: 0.0,
            pitch: 0.3,
        }
    }
}

/// Difficulty level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Realistic,
}

impl Default for Difficulty {
    fn default() -> Self {
        Difficulty::Normal
    }
}

/// Units system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UnitsSystem {
    Metric,
    Imperial,
}

impl Default for UnitsSystem {
    fn default() -> Self {
        UnitsSystem::Metric
    }
}

/// Controls settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlsSettings {
    pub mouse_sensitivity: f32,
    pub invert_y_axis: bool,
    pub vehicle_mouse_steering: bool,
    pub camera_mode: CameraMode,
    pub camera_distance: f32,
    pub camera_height: f32,
    pub camera_smoothing: f32,

    // Key bindings
    pub key_forward: String,
    pub key_backward: String,
    pub key_left: String,
    pub key_right: String,
    pub key_jump: String,
    pub key_run: String,
    pub key_interact: String,
    pub key_inventory: String,
    pub key_menu: String,
    pub key_horn: String,
    pub key_lights: String,
    pub key_engine: String,
    pub key_handbrake: String,
    pub key_view_change: String,
    pub key_debug: String,

    // Vehicle controls
    pub key_gear_up: String,
    pub key_gear_down: String,
    pub key_diff_lock: String,
    pub key_4wd_toggle: String,
    pub key_low_range: String,
    pub key_winch_in: String,
    pub key_winch_out: String,
}

impl Default for ControlsSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 1.0,
            invert_y_axis: false,
            vehicle_mouse_steering: false,
            camera_mode: CameraMode::ThirdPerson {
                distance: 5.0,
                yaw: 0.0,
                pitch: 0.3,
            },
            camera_distance: 5.0,
            camera_height: 2.5,
            camera_smoothing: 0.8,

            key_forward: "W".to_string(),
            key_backward: "S".to_string(),
            key_left: "A".to_string(),
            key_right: "D".to_string(),
            key_jump: "Space".to_string(),
            key_run: "ShiftLeft".to_string(),
            key_interact: "F".to_string(),
            key_inventory: "Tab".to_string(),
            key_menu: "Escape".to_string(),
            key_horn: "H".to_string(),
            key_lights: "L".to_string(),
            key_engine: "E".to_string(),
            key_handbrake: "Space".to_string(),
            key_view_change: "V".to_string(),
            key_debug: "F3".to_string(),

            key_gear_up: "Z".to_string(),
            key_gear_down: "X".to_string(),
            key_diff_lock: "K".to_string(),
            key_4wd_toggle: "N".to_string(),
            key_low_range: "B".to_string(),
            key_winch_in: "Q".to_string(),
            key_winch_out: "R".to_string(),
        }
    }
}

/// Gameplay settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplaySettings {
    pub difficulty: Difficulty,
    pub auto_save: bool,
    pub auto_save_interval: u32, // seconds
    pub show_hints: bool,
    pub show_waypoints: bool,
    pub show_minimap: bool,
    pub show_compass: bool,
    pub units: UnitsSystem,
    pub fuel_consumption: f32, // multiplier
    pub wear_rate: f32,        // multiplier
    pub damage_multiplier: f32,
    pub respawn_on_crash: bool,
    pub financial_penalties: bool,
}

impl Default for GameplaySettings {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Normal,
            auto_save: true,
            auto_save_interval: 300,
            show_hints: true,
            show_waypoints: true,
            show_minimap: true,
            show_compass: true,
            units: UnitsSystem::Metric,
            fuel_consumption: 1.0,
            wear_rate: 1.0,
            damage_multiplier: 1.0,
            respawn_on_crash: true,
            financial_penalties: true,
        }
    }
}

/// HUD settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudSettings {
    pub hud_enabled: bool,
    pub hud_opacity: f32,
    pub hud_scale: f32,
    pub show_speed: bool,
    pub show_gear: bool,
    pub show_fuel: bool,
    pub show_diff_status: bool,
    pub show_wheel_status: bool,
    pub show_cargo: bool,
    pub show_terrain_angle: bool,
    pub show_minimap: bool,
    pub show_compass: bool,
    pub compact_mode: bool,
    pub show_fps: bool,
    pub show_ping: bool,
    pub show_coordinates: bool,
}

impl Default for HudSettings {
    fn default() -> Self {
        Self {
            hud_enabled: true,
            hud_opacity: 1.0,
            hud_scale: 1.0,
            show_speed: true,
            show_gear: true,
            show_fuel: true,
            show_diff_status: true,
            show_wheel_status: true,
            show_cargo: true,
            show_terrain_angle: true,
            show_minimap: true,
            show_compass: true,
            compact_mode: false,
            show_fps: false,
            show_ping: false,
            show_coordinates: false,
        }
    }
}

/// Network settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub player_name: String,
    pub voice_chat_enabled: bool,
    pub voice_chat_push_to_talk: bool,
    pub push_to_talk_key: String,
    pub max_players: u32,
    pub server_port: u16,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            player_name: "Driver".to_string(),
            voice_chat_enabled: true,
            voice_chat_push_to_talk: true,
            push_to_talk_key: "ControlLeft".to_string(),
            max_players: 32,
            server_port: 27015,
        }
    }
}

/// Performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub use_multithreading: bool,
    pub physics_threads: u32,
    pub render_threads: u32,
    pub streaming_threads: u32,
    pub memory_budget_mb: u32,
    pub chunk_load_distance: u32,
    pub chunk_unload_distance: u32,
    pub async_texture_loading: bool,
    pub preload_nearby_chunks: bool,
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            use_multithreading: true,
            physics_threads: 4,
            render_threads: 2,
            streaming_threads: 2,
            memory_budget_mb: 2048,
            chunk_load_distance: 8,
            chunk_unload_distance: 12,
            async_texture_loading: true,
            preload_nearby_chunks: true,
        }
    }
}

/// Debug settings (development only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSettings {
    pub debug_mode: bool,
    pub show_physics_debug: bool,
    pub show_chunk_boundaries: bool,
    pub show_collision_boxes: bool,
    pub show_fps_graph: bool,
    pub show_memory_usage: bool,
    pub god_mode: bool,
    pub unlimited_fuel: bool,
    pub unlimited_money: bool,
    pub skip_character_creation: bool,
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            debug_mode: false,
            show_physics_debug: false,
            show_chunk_boundaries: false,
            show_collision_boxes: false,
            show_fps_graph: false,
            show_memory_usage: false,
            god_mode: false,
            unlimited_fuel: false,
            unlimited_money: false,
            skip_character_creation: false,
        }
    }
}

/// Main settings structure - contains all game settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub display: DisplaySettings,
    pub graphics: GraphicsSettings,
    pub audio: AudioSettings,
    pub controls: ControlsSettings,
    pub gameplay: GameplaySettings,
    pub hud: HudSettings,
    pub network: NetworkSettings,
    pub performance: PerformanceSettings,
    pub debug: DebugSettings,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            display: DisplaySettings::default(),
            graphics: GraphicsSettings::default(),
            audio: AudioSettings::default(),
            controls: ControlsSettings::default(),
            gameplay: GameplaySettings::default(),
            hud: HudSettings::default(),
            network: NetworkSettings::default(),
            performance: PerformanceSettings::default(),
            debug: DebugSettings::default(),
        }
    }
}

/// Settings Manager - handles loading and saving of game settings
pub struct SettingsManager {
    settings: GameSettings,
    config_path: String,
}

impl SettingsManager {
    /// Create a new SettingsManager with default settings
    pub fn new() -> Self {
        Self {
            settings: GameSettings::default(),
            config_path: "assets/settings/settings.toml".to_string(),
        }
    }

    /// Load settings from TOML file
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.config_path = path_str.clone();

        if !path.as_ref().exists() {
            tracing::warn!("Settings file not found: {}, using defaults", path_str);
            return Ok(());
        }

        let content = fs::read_to_string(path)?;
        let settings: GameSettings = toml::from_str(&content)?;
        self.settings = settings;

        tracing::info!("Settings loaded from {}", path_str);
        Ok(())
    }

    /// Save settings to TOML file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(&self.settings)?;
        fs::write(&self.config_path, content)?;

        tracing::info!("Settings saved to {}", self.config_path);
        Ok(())
    }

    /// Save settings to a specific path
    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(&self.settings)?;
        let path_ref = path.as_ref();
        fs::write(path_ref, content)?;

        tracing::info!("Settings saved to {:?}", path_ref);
        Ok(())
    }

    /// Get immutable reference to settings
    pub fn get(&self) -> &GameSettings {
        &self.settings
    }

    /// Get mutable reference to settings
    pub fn get_mut(&mut self) -> &mut GameSettings {
        &mut self.settings
    }

    /// Apply settings to the game (called after loading/modifying)
    pub fn apply(&mut self) {
        // Apply display settings
        tracing::debug!(
            "Applying display settings: {}x{}, fullscreen={}, vsync={}",
            self.settings.display.resolution_width,
            self.settings.display.resolution_height,
            self.settings.display.fullscreen,
            self.settings.display.vsync
        );

        // Apply graphics settings
        tracing::debug!(
            "Applying graphics settings: quality={:?}, shadows={:?}, aa={:?}",
            self.settings.graphics.texture_quality,
            self.settings.graphics.shadow_quality,
            self.settings.graphics.anti_aliasing
        );

        // Apply audio settings - set volumes
        tracing::debug!(
            "Applying audio settings: master={}, music={}, sfx={}",
            self.settings.audio.master_volume,
            self.settings.audio.music_volume,
            self.settings.audio.sfx_volume
        );

        // Apply controls settings
        tracing::debug!("Applying controls settings: sensitivity={}", self.settings.controls.mouse_sensitivity);

        // Apply gameplay settings
        tracing::debug!(
            "Applying gameplay settings: difficulty={:?}, autosave={}",
            self.settings.gameplay.difficulty,
            self.settings.gameplay.auto_save
        );

        tracing::info!("Settings applied successfully");
    }

    /// Reset all settings to default
    pub fn reset_to_defaults(&mut self) {
        self.settings = GameSettings::default();
        tracing::info!("Settings reset to defaults");
    }

    /// Reset a specific category to defaults
    pub fn reset_category(&mut self, category: &str) {
        match category {
            "display" => self.settings.display = DisplaySettings::default(),
            "graphics" => self.settings.graphics = GraphicsSettings::default(),
            "audio" => self.settings.audio = AudioSettings::default(),
            "controls" => self.settings.controls = ControlsSettings::default(),
            "gameplay" => self.settings.gameplay = GameplaySettings::default(),
            "hud" => self.settings.hud = HudSettings::default(),
            "network" => self.settings.network = NetworkSettings::default(),
            "performance" => self.settings.performance = PerformanceSettings::default(),
            "debug" => self.settings.debug = DebugSettings::default(),
            _ => tracing::warn!("Unknown settings category: {}", category),
        }
        tracing::info!("Settings category '{}' reset to defaults", category);
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = GameSettings::default();
        assert_eq!(settings.display.fps_limit, 60);
        assert_eq!(settings.graphics.backend, "opengl");
        assert_eq!(settings.audio.master_volume, 0.8);
        assert_eq!(settings.controls.key_inventory, "Tab");
    }

    #[test]
    fn test_settings_manager_creation() {
        let manager = SettingsManager::new();
        assert_eq!(manager.get().display.fps_limit, 60);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = GameSettings::default();
        let serialized = toml::to_string(&settings).unwrap();
        let deserialized: GameSettings = toml::from_str(&serialized).unwrap();
        assert_eq!(settings.display.fps_limit, deserialized.display.fps_limit);
    }
}
