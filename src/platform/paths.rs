use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct AppPaths {
    pub config_dir: PathBuf,
    pub save_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub fonts_dir: PathBuf,
    pub textures_dir: PathBuf,
    pub audio_dir: PathBuf,
    pub shaders_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl AppPaths {
    pub fn resolve() -> Result<Self> {
        let exe_dir = std::env::current_exe()
            .context("Failed to get executable path")?
            .parent()
            .context("Failed to get parent directory")?
            .to_path_buf();

        let assets_dir = exe_dir.join("assets");
        let config_dir = if let Some(data_dir) = dirs::config_dir() {
            data_dir.join("RTGC")
        } else {
            exe_dir.join("config")
        };

        Ok(Self {
            config_dir: config_dir.clone(),
            save_dir: config_dir.join("saves"),
            assets_dir: assets_dir.clone(),
            fonts_dir: assets_dir.join("fonts"),
            textures_dir: assets_dir.join("textures"),
            audio_dir: assets_dir.join("audio"),
            shaders_dir: assets_dir.join("shaders"),
            data_dir: assets_dir.join("data"),
        })
    }

    pub fn ensure_directories(&self) -> Result<()> {
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.save_dir)?;
        Ok(())
    }
}
