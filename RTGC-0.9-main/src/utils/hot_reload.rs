//! Hot reload functionality for RTGC engine
//! Provides dynamic reloading of assets during development

use std::time::Duration;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs::FileTime;

/// Configuration for hot reload system
pub struct HotReloadConfig {
    pub enabled: bool,
    pub poll_interval: Duration,
    pub watch_paths: Vec<PathBuf>,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            poll_interval: Duration::from_millis(500),
            watch_paths: vec!["assets".into()],
        }
    }
}

/// Tracks a watched file's state
#[derive(Debug)]
struct WatchedFile {
    path: PathBuf,
    last_modified: FileTime,
    reload_count: u32,
}

/// Manages hot reloading of assets
pub struct HotReloadManager {
    config: HotReloadConfig,
    watched_files: HashMap<PathBuf, WatchedFile>,
    last_poll_time: std::time::Instant,
    pending_changes: Vec<PathBuf>,
}

impl HotReloadManager {
    /// Creates a new HotReloadManager
    pub fn new(config: HotReloadConfig) -> Self {
        Self {
            config,
            watched_files: HashMap::new(),
            last_poll_time: std::time::Instant::now(),
            pending_changes: Vec::new(),
        }
    }

    /// Adds a file to watch
    pub fn watch_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path = path.as_ref().to_path_buf();
        
        if !path.exists() {
            return Err(format!("File does not exist: {:?}", path));
        }

        let metadata = std::fs::metadata(&path)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;
        
        let last_modified = FileTime::from_last_modification_time(&metadata);

        self.watched_files.insert(path.clone(), WatchedFile {
            path: path.clone(),
            last_modified,
            reload_count: 0,
        });

        tracing::debug!("Watching file: {:?}", path);
        Ok(())
    }

    /// Removes a file from watching
    pub fn unwatch_file<P: AsRef<Path>>(&mut self, path: P) -> bool {
        self.watched_files.remove(path.as_ref()).is_some()
    }

    /// Polls for changes in watched files
    pub fn poll(&mut self) -> bool {
        if !self.config.enabled {
            return false;
        }

        // Check if enough time has passed since last poll
        if self.last_poll_time.elapsed() < self.config.poll_interval {
            return false;
        }

        self.last_poll_time = std::time::Instant::now();
        self.pending_changes.clear();

        // Check all watched files for changes
        for (path, watched) in &mut self.watched_files {
            if let Ok(metadata) = std::fs::metadata(path) {
                let current_modified = FileTime::from_last_modification_time(&metadata);
                
                if current_modified != watched.last_modified {
                    tracing::info!("File changed: {:?}", path);
                    watched.last_modified = current_modified;
                    watched.reload_count += 1;
                    self.pending_changes.push(path.clone());
                }
            }
        }

        !self.pending_changes.is_empty()
    }

    /// Gets list of files that have changed since last poll
    pub fn get_changed_files(&self) -> &[PathBuf] {
        &self.pending_changes
    }

    /// Gets reload count for a specific file
    pub fn get_reload_count<P: AsRef<Path>>(&self, path: P) -> Option<u32> {
        self.watched_files.get(path.as_ref()).map(|w| w.reload_count)
    }

    /// Clears all watched files
    pub fn clear_watched(&mut self) {
        self.watched_files.clear();
        self.pending_changes.clear();
    }

    /// Enables or disables hot reloading
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// Checks if hot reloading is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Gets statistics about watched files
    pub fn get_stats(&self) -> HotReloadStats {
        HotReloadStats {
            total_watched: self.watched_files.len(),
            pending_changes: self.pending_changes.len(),
            total_reloads: self.watched_files.values().map(|w| w.reload_count).sum(),
        }
    }
}

/// Statistics about hot reload system
#[derive(Debug, Clone)]
pub struct HotReloadStats {
    pub total_watched: usize,
    pub pending_changes: usize,
    pub total_reloads: u32,
}

impl Default for HotReloadManager {
    fn default() -> Self {
        Self::new(HotReloadConfig::default())
    }
}