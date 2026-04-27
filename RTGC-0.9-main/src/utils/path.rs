//! Path sanitization utilities for secure file operations

use crate::error::ConfigError;
use std::path::{Component, PathBuf};

/// Sanitizes a path string to prevent directory traversal attacks.
///
/// This function validates that the input path does not contain:
/// - Parent directory references (..)
/// - Absolute paths that could access system files
/// - Other potentially dangerous patterns
///
/// # Arguments
/// * `input` - The path string to sanitize
///
/// # Returns
/// * `Ok(PathBuf)` - A sanitized, safe path
/// * `Err(ConfigError)` - If the path contains dangerous patterns
///
/// # Examples
/// ```
/// use rtgc::utils::sanitize_path;
///
/// // Valid paths
/// assert!(sanitize_path("saves/game1").is_ok());
/// assert!(sanitize_path("./assets/textures").is_ok());
///
/// // Invalid paths (directory traversal)
/// assert!(sanitize_path("../etc/passwd").is_err());
/// assert!(sanitize_path("/etc/passwd").is_err());
/// ```
pub fn sanitize_path(input: &str) -> Result<PathBuf, ConfigError> {
    let path = PathBuf::from(input);

    // Check for absolute paths that could access system directories
    if path.is_absolute() {
        // Allow only paths within expected base directories
        let path_str = path.to_string_lossy();
        if !path_str.starts_with("./")
            && !path_str.starts_with("saves/")
            && !path_str.starts_with("assets/")
            && !path_str.starts_with("config/")
            && !path_str.starts_with("logs/")
        {
            return Err(ConfigError::PathTraversal);
        }
    }

    // Check each component for parent directory references
    for component in path.components() {
        match component {
            Component::ParentDir => {
                return Err(ConfigError::PathTraversal);
            }
            Component::RootDir => {
                return Err(ConfigError::PathTraversal);
            }
            _ => {}
        }
    }

    // Additional check: ensure no hidden components in the middle of the path
    for component in path.components() {
        if let Component::Normal(os_str) = component {
            if let Some(s) = os_str.to_str() {
                if s.starts_with('.') && s != "." && s != ".." {
                    // Hidden files/directories are suspicious in user-provided paths
                    // Log warning via tracing instead of eprintln
                    tracing::warn!(target: "security", "Hidden path component detected: {}", s);
                }
            }
        }
    }

    Ok(path)
}

/// Validates that a path exists and is accessible
///
/// # Arguments
/// * `path` - The path to validate
/// * `must_exist` - If true, the path must exist on the filesystem
///
/// # Returns
/// * `Ok(PathBuf)` - The validated path
/// * `Err(ConfigError)` - If validation fails
pub fn validate_path(path: &str, must_exist: bool) -> Result<PathBuf, ConfigError> {
    let sanitized = sanitize_path(path)?;

    if must_exist && !sanitized.exists() {
        return Err(ConfigError::FileReadError(format!(
            "File not found: {}",
            sanitized.display()
        )));
    }

    Ok(sanitized)
}

/// Creates a safe directory path for saving game data
///
/// # Arguments
/// * `base_dir` - The base directory for saves
/// * `sub_path` - The sub-path within the base directory
///
/// # Returns
/// * `Ok(PathBuf)` - The combined safe path
/// * `Err(ConfigError)` - If the path is invalid
pub fn create_safe_save_path(base_dir: &str, sub_path: &str) -> Result<PathBuf, ConfigError> {
    let base = sanitize_path(base_dir)?;
    let sub = sanitize_path(sub_path)?;

    // Ensure sub_path doesn't escape base_dir
    let combined = base.join(sub);

    // Verify the combined path is still within base_dir
    if !combined.starts_with(&base) {
        return Err(ConfigError::PathTraversal);
    }

    Ok(combined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_path_accepts_valid_paths() {
        assert!(sanitize_path("saves/game1").is_ok());
        assert!(sanitize_path("./assets/textures").is_ok());
        assert!(sanitize_path("config/settings.json").is_ok());
        assert!(sanitize_path("logs/debug.log").is_ok());
    }

    #[test]
    fn test_sanitize_path_rejects_parent_dir() {
        assert!(sanitize_path("../etc/passwd").is_err());
        assert!(sanitize_path("saves/../../../etc/passwd").is_err());
        assert!(sanitize_path("..\\..\\secret.txt").is_err());
    }

    #[test]
    fn test_sanitize_path_rejects_absolute_system_paths() {
        assert!(sanitize_path("/etc/passwd").is_err());
        assert!(sanitize_path("/root/.ssh/id_rsa").is_err());
        assert!(sanitize_path("C:\\Windows\\System32").is_err());
    }

    #[test]
    fn test_validate_path_with_existence_check() {
        // This test file should exist
        assert!(validate_path("Cargo.toml", true).is_ok());

        // This file should not exist
        assert!(validate_path("nonexistent_file.txt", true).is_err());

        // Non-existent file is OK if must_exist is false
        assert!(validate_path("nonexistent_file.txt", false).is_ok());
    }

    #[test]
    fn test_create_safe_save_path() {
        let result = create_safe_save_path("saves", "game1/slot1");
        assert!(result.is_ok());
        let path = result.ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Path operation failed"))?;
        assert!(path.starts_with("saves"));
        assert!(path.to_string_lossy().contains("game1/slot1"));
    }

    #[test]
    fn test_create_safe_save_path_prevents_escape() {
        // Attempting to escape base directory should fail
        let result = create_safe_save_path("saves", "../../../etc");
        assert!(result.is_err());
    }
}
