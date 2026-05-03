use anyhow::Result;
use kira::manager::{AudioManager, AudioManagerSettings, DefaultBackend};

pub struct RtgcAudioManager {
    manager: Option<AudioManager<DefaultBackend>>,
    master_volume: f64,
    music_volume: f64,
    sfx_volume: f64,
}

impl RtgcAudioManager {
    pub fn new() -> Result<Self> {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .map_err(|e| anyhow::anyhow!("Failed to create audio manager: {}", e))?;

        Ok(Self {
            manager: Some(manager),
            master_volume: 1.0,
            music_volume: 0.6,
            sfx_volume: 0.9,
        })
    }

    pub fn set_master_volume(&mut self, volume: f64) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    pub fn play_menu_music(&mut self) {
        // TODO: Load and play menu music
    }

    pub fn play_sfx(&mut self, _sfx_name: &str) {
        // TODO: Play sound effect
    }
}
