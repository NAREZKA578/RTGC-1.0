use crate::save::player_profile::PlayerProfile;
use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct SaveManager {
    save_dir: PathBuf,
}

impl SaveManager {
    pub fn new(save_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&save_dir).ok();
        Self { save_dir }
    }

    pub fn save(&self, profile: &PlayerProfile) -> Result<()> {
        let filename = format!("player_{}.toml", profile.character.id);
        let path = self.save_dir.join(&filename);
        let data = toml::to_string_pretty(profile).context("Failed to serialize profile")?;
        std::fs::write(&path, data).context("Failed to write save file")?;
        tracing::info!("Game saved to: {:?}", path);
        Ok(())
    }

    pub fn load(&self, character_id: &str) -> Result<PlayerProfile> {
        let filename = format!("player_{}.toml", character_id);
        let path = self.save_dir.join(&filename);
        let data = std::fs::read_to_string(&path).context("Failed to read save file")?;
        let profile = toml::from_str(&data).context("Failed to deserialize profile")?;
        Ok(profile)
    }

    pub fn list_saves(&self) -> Result<Vec<String>> {
        let mut saves = Vec::new();
        if self.save_dir.exists() {
            for entry in std::fs::read_dir(&self.save_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                    if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                        saves.push(name.to_string());
                    }
                }
            }
        }
        Ok(saves)
    }

    pub fn delete(&self, character_id: &str) -> Result<()> {
        let filename = format!("player_{}.toml", character_id);
        let path = self.save_dir.join(&filename);
        if path.exists() {
            std::fs::remove_file(&path).context("Failed to delete save file")?;
        }
        Ok(())
    }
}
