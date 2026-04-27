//! Configuration module for RTGC engine
//! Provides centralized configuration for all engine subsystems
//! Supports TOML format for better readability and maintenance

use crate::error::{ConfigError, Result};
use crate::utils::sanitize_path as utils_sanitize_path;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

type ConfigResult = std::result::Result<(), ConfigError>;

pub const DEFAULT_TARGET_FPS: f32 = 60.0;
pub const DEFAULT_FRAME_TIME_CLAMP: f32 = 0.1;
pub const DEFAULT_WINDOW_WIDTH: u32 = 1280;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 720;

/// Main configuration structure containing all subsystem configs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub graphics: GraphicsConfig,
    pub physics: PhysicsConfig,
    pub world: WorldConfig,
    pub input: InputConfig,
    pub audio: AudioConfig,
    pub network: NetworkConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            graphics: GraphicsConfig::default(),
            physics: PhysicsConfig::default(),
            world: WorldConfig::default(),
            input: InputConfig::default(),
            audio: AudioConfig::default(),
            network: NetworkConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            ConfigError::FileReadError(format!("Failed to read config: {}", e))
        })?;
        
        let config: Config = toml::from_str(&content).map_err(|e| {
            ConfigError::ParseError(format!("Failed to parse config: {}", e))
        })?;
        
        config.validate()?;
        info!("Config loaded from {:?}", path.as_ref());
        Ok(config)
    }
    
    /// Validate all configuration sections
    pub fn validate(&self) -> ConfigResult {
        self.graphics.validate()?;
        self.physics.validate()?;
        self.world.validate()?;
        self.input.validate()?;
        self.audio.validate()?;
        self.network.validate()?;
        Ok(())
    }

    /// Save configuration to a TOML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> ConfigResult {
        // Validate before saving
        self.validate()?;

        let content = toml::to_string_pretty(self).map_err(|e| {
            ConfigError::SerializationError(format!("Failed to serialize config: {}", e))
        })?;
        let path_ref = path.as_ref();
        std::fs::write(path_ref, content).map_err(|e| {
            ConfigError::FileWriteError(format!("Failed to write config file: {}", e))
        })?;
        info!("Configuration saved to {:?}", path_ref);
        Ok(())
    }
}

/// Graphics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub max_fps: Option<u32>,
    pub msaa_samples: u32,
    pub shadow_resolution: u32,
    pub max_anisotropy: f32,
    pub lod_bias: f32,
    pub texture_streaming_budget_mb: u32,
    pub backend: String, // "vulkan", "dx12", "opengl", "dx11"
    pub enable_validation: bool,
}

impl GraphicsConfig {
    /// Validate graphics configuration values
    pub fn validate(&self) -> ConfigResult {
        // Validate window dimensions
        if self.window_width == 0 || self.window_width > 7680 {
            return Err(ConfigError::InvalidValue(format!(
                "window_width must be between 1 and 7680, got {}",
                self.window_width
            )));
        }
        if self.window_height == 0 || self.window_height > 4320 {
            return Err(ConfigError::InvalidValue(format!(
                "window_height must be between 1 and 4320, got {}",
                self.window_height
            )));
        }

        // Validate FPS
        if let Some(fps) = self.max_fps {
            if fps == 0 || fps > 1000 {
                return Err(ConfigError::InvalidFps(fps));
            }
        }

        // Validate MSAA
        if self.msaa_samples != 0
            && self.msaa_samples != 1
            && self.msaa_samples != 2
            && self.msaa_samples != 4
            && self.msaa_samples != 8
        {
            return Err(ConfigError::InvalidValue(format!(
                "msaa_samples must be 0, 1, 2, 4, or 8, got {}",
                self.msaa_samples
            )));
        }

        // Validate shadow resolution
        if self.shadow_resolution < 256 || self.shadow_resolution > 8192 {
            return Err(ConfigError::InvalidValue(format!(
                "shadow_resolution must be between 256 and 8192, got {}",
                self.shadow_resolution
            )));
        }

        // Validate anisotropy
        if self.max_anisotropy < 1.0 || self.max_anisotropy > 16.0 {
            return Err(ConfigError::InvalidValue(format!(
                "max_anisotropy must be between 1.0 and 16.0, got {}",
                self.max_anisotropy
            )));
        }

        // Validate texture streaming budget (max 4GB)
        if self.texture_streaming_budget_mb > 4096 {
            return Err(ConfigError::MemoryBudgetExceeded(
                self.texture_streaming_budget_mb,
            ));
        }

        // Validate backend
        let valid_backends = ["vulkan", "dx12", "opengl", "dx11"];
        if !valid_backends.contains(&self.backend.to_lowercase().as_str()) {
            return Err(ConfigError::InvalidBackend(self.backend.clone()));
        }

        Ok(())
    }
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            window_width: 1920,
            window_height: 1080,
            fullscreen: false,
            vsync: true,
            max_fps: Some(60),
            msaa_samples: 4,
            shadow_resolution: 2048,
            max_anisotropy: 16.0,
            lod_bias: 0.0,
            texture_streaming_budget_mb: 512,
            backend: "vulkan".to_string(),
            enable_validation: cfg!(debug_assertions),
        }
    }
}

/// Physics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub substeps: u32,
    pub gravity: [f32; 3],
    pub solver_iterations: u32,
    pub contact_offset: f32,
    pub rest_offset: f32,
    pub max_depenetration_velocity: f32,
    pub enable_ccd: bool,
    pub thread_count: u32,
    pub async_physics: bool,
    pub timestep: f32,
}

impl PhysicsConfig {
    /// Validate physics configuration values
    pub fn validate(&self) -> ConfigResult {
        // Validate substeps (must be >= 1 to avoid division by zero)
        if self.substeps == 0 || self.substeps > 64 {
            return Err(ConfigError::InvalidValue(format!(
                "substeps must be between 1 and 64, got {}",
                self.substeps
            )));
        }

        // Validate solver iterations
        if self.solver_iterations == 0 || self.solver_iterations > 256 {
            return Err(ConfigError::InvalidValue(format!(
                "solver_iterations must be between 1 and 256, got {}",
                self.solver_iterations
            )));
        }

        // Validate contact offset
        if self.contact_offset <= 0.0 || self.contact_offset > 1.0 {
            return Err(ConfigError::InvalidValue(format!(
                "contact_offset must be between 0.0 and 1.0, got {}",
                self.contact_offset
            )));
        }

        // Validate rest offset
        if self.rest_offset < 0.0 || self.rest_offset > 1.0 {
            return Err(ConfigError::InvalidValue(format!(
                "rest_offset must be between 0.0 and 1.0, got {}",
                self.rest_offset
            )));
        }

        // Validate max depenetration velocity
        if self.max_depenetration_velocity <= 0.0 || self.max_depenetration_velocity > 10000.0 {
            return Err(ConfigError::InvalidValue(format!(
                "max_depenetration_velocity must be between 0.0 and 10000.0, got {}",
                self.max_depenetration_velocity
            )));
        }

        // Validate thread count
        if self.thread_count == 0 || self.thread_count > 64 {
            return Err(ConfigError::InvalidValue(format!(
                "thread_count must be between 1 and 64, got {}",
                self.thread_count
            )));
        }

        // Validate timestep
        if self.timestep <= 0.0 || self.timestep > 1.0 {
            return Err(ConfigError::InvalidValue(format!(
                "timestep must be between 0.0 and 1.0, got {}",
                self.timestep
            )));
        }

        Ok(())
    }
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        // Use std::thread::available_parallelism() instead of num_cpus::get()
        let thread_count = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1);

        Self {
            substeps: 4,
            gravity: [0.0, -9.81, 0.0],
            solver_iterations: 8,
            contact_offset: 0.01,
            rest_offset: 0.0,
            max_depenetration_velocity: 100.0,
            enable_ccd: true,
            thread_count,
            async_physics: true,
            timestep: 1.0 / 60.0,
        }
    }
}

/// World configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    pub chunk_size: u32,
    pub render_distance: u32,
    pub terrainlod_distances: Vec<f32>,
    pub max_entities: u32,
    pub streaming_enabled: bool,
    pub save_directory: String,
}

impl WorldConfig {
    /// Validate world configuration values and sanitize paths
    pub fn validate(&self) -> ConfigResult {
        // Validate chunk size
        if self.chunk_size < 8 || self.chunk_size > 512 {
            return Err(ConfigError::InvalidValue(format!(
                "chunk_size must be between 8 and 512, got {}",
                self.chunk_size
            )));
        }

        // Validate render distance
        if self.render_distance == 0 || self.render_distance > 100 {
            return Err(ConfigError::InvalidValue(format!(
                "render_distance must be between 1 and 100, got {}",
                self.render_distance
            )));
        }

        // Validate LOD distances
        for (i, &dist) in self.terrainlod_distances.iter().enumerate() {
            if dist <= 0.0 || dist > 10000.0 {
                return Err(ConfigError::InvalidValue(format!(
                    "terrainlod_distances[{}] must be between 0.0 and 10000.0, got {}",
                    i, dist
                )));
            }
        }

        // Validate max entities
        if self.max_entities == 0 || self.max_entities > 1000000 {
            return Err(ConfigError::InvalidValue(format!(
                "max_entities must be between 1 and 1000000, got {}",
                self.max_entities
            )));
        }

        // Sanitize and validate save directory path using centralized utility
        utils_sanitize_path(&self.save_directory).map_err(|_| ConfigError::PathTraversal)?;

        Ok(())
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            chunk_size: 64,
            render_distance: 10,
            terrainlod_distances: vec![50.0, 100.0, 200.0, 500.0],
            max_entities: 10000,
            streaming_enabled: true,
            save_directory: "saves".to_string(),
        }
    }
}

/// Input configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub mouse_sensitivity: f32,
    pub invert_y: bool,
    pub gamepad_enabled: bool,
    pub vibration_enabled: bool,
    pub vibration_strength: f32,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 1.0,
            invert_y: false,
            gamepad_enabled: true,
            vibration_enabled: true,
            vibration_strength: 0.5,
        }
    }
}

impl InputConfig {
    /// Validate input configuration values
    pub fn validate(&self) -> ConfigResult {
        // Validate mouse sensitivity
        if self.mouse_sensitivity < 0.0 || self.mouse_sensitivity > 10.0 {
            return Err(ConfigError::InvalidValue(format!(
                "mouse_sensitivity must be between 0.0 and 10.0, got {}",
                self.mouse_sensitivity
            )));
        }

        // Validate vibration strength
        if self.vibration_strength < 0.0 || self.vibration_strength > 1.0 {
            return Err(ConfigError::InvalidValue(format!(
                "vibration_strength must be between 0.0 and 1.0, got {}",
                self.vibration_strength
            )));
        }

        Ok(())
    }
}

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub voice_volume: f32,
    pub environmental_audio: bool,
    pub doppler_effect: bool,
    pub max_audio_sources: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 1.0,
            voice_volume: 1.0,
            environmental_audio: true,
            doppler_effect: true,
            max_audio_sources: 32,
        }
    }
}

impl AudioConfig {
    /// Validate audio configuration values
    pub fn validate(&self) -> ConfigResult {
        // Validate volume levels (0.0 to 1.0, but allow slightly above for headroom)
        let volumes = [
            ("master_volume", self.master_volume),
            ("music_volume", self.music_volume),
            ("sfx_volume", self.sfx_volume),
            ("voice_volume", self.voice_volume),
        ];

        for (name, value) in volumes.iter() {
            if *value < 0.0 || *value > 2.0 {
                return Err(ConfigError::InvalidValue(format!(
                    "{} must be between 0.0 and 2.0, got {}",
                    name, value
                )));
            }
        }

        // Validate max audio sources
        if self.max_audio_sources == 0 || self.max_audio_sources > 512 {
            return Err(ConfigError::InvalidValue(format!(
                "max_audio_sources must be between 1 and 512, got {}",
                self.max_audio_sources
            )));
        }

        Ok(())
    }
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub player_name: String,
    pub voice_chat_enabled: bool,
    pub voice_chat_push_to_talk: bool,
    pub push_to_talk_key: String,
    pub max_players: u32,
    pub server_port: u16,
    pub tick_rate: f32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            player_name: "Driver".to_string(),
            voice_chat_enabled: true,
            voice_chat_push_to_talk: true,
            push_to_talk_key: "ControlLeft".to_string(),
            max_players: 32,
            server_port: 27015,
            tick_rate: 60.0,
        }
    }
}

impl NetworkConfig {
    /// Validate network configuration values
    pub fn validate(&self) -> ConfigResult {
        if self.max_players == 0 || self.max_players > 128 {
            return Err(ConfigError::InvalidValue(format!(
                "max_players must be between 1 and 128, got {}",
                self.max_players
            )));
        }

        if self.server_port == 0 || self.server_port > 65535 {
            return Err(ConfigError::InvalidValue(format!(
                "server_port must be between 1 and 65535, got {}",
                self.server_port
            )));
        }

        if self.tick_rate <= 0.0 || self.tick_rate > 1000.0 {
            return Err(ConfigError::InvalidValue(format!(
                "tick_rate must be between 0.0 and 1000.0, got {}",
                self.tick_rate
            )));
        }

        Ok(())
    }
}

// Примечание: функция sanitize_path удалена, теперь используется централизованная
// версия из crate::utils::sanitize_path для единообразия во всём проекте

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.graphics.window_width, 1920);
        assert_eq!(config.graphics.window_height, 1080);
        assert_eq!(config.physics.substeps, 4);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_string = toml::to_string(&config).expect("Failed to serialize config");
        let loaded: Config = toml::from_str(&toml_string).expect("Failed to deserialize config");
        assert_eq!(config.graphics.window_width, loaded.graphics.window_width);
    }
}
