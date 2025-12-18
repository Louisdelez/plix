//! Launcher configuration

use crate::error::LauncherError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Launcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    /// Config format version
    #[serde(default = "default_config_version")]
    pub config_version: u32,

    /// URL to fetch manifest from
    #[serde(default = "default_manifest_url")]
    pub manifest_url: String,

    /// Stay open after launching game (default: false)
    #[serde(default)]
    pub stay_open: bool,

    /// HTTP timeout in seconds (default: 30)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Number of download retries (default: 3)
    #[serde(default = "default_retries")]
    pub max_retries: u32,

    /// Verbose logging (default: false)
    #[serde(default)]
    pub verbose: bool,
}

fn default_config_version() -> u32 {
    1
}

fn default_manifest_url() -> String {
    "https://releases.plix.example/manifest.toml".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_retries() -> u32 {
    3
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            config_version: default_config_version(),
            manifest_url: default_manifest_url(),
            stay_open: false,
            timeout_seconds: default_timeout(),
            max_retries: default_retries(),
            verbose: false,
        }
    }
}

impl LauncherConfig {
    /// Load configuration from file, creating default if not exists
    pub fn load(path: &Path) -> Result<Self, LauncherError> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: LauncherConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration or create default file if not exists
    pub fn load_or_create(path: &Path) -> Result<Self, LauncherError> {
        if path.exists() {
            Self::load(path)
        } else {
            let config = Self::default();
            config.save(path)?;
            Ok(config)
        }
    }

    /// Save configuration to file using atomic write
    pub fn save(&self, path: &Path) -> Result<(), LauncherError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| LauncherError::Config(format!("Failed to serialize config: {}", e)))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Atomic write: write to temp file, then rename
        let temp_path = path.with_extension("toml.tmp");
        std::fs::write(&temp_path, &content)?;
        std::fs::rename(&temp_path, path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = LauncherConfig::default();
        assert_eq!(config.config_version, 1);
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert!(!config.verbose);
        assert!(!config.stay_open);
    }

    #[test]
    fn test_config_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("launcher.toml");

        let config = LauncherConfig {
            manifest_url: "https://test.example/manifest.toml".to_string(),
            verbose: true,
            ..Default::default()
        };

        config.save(&path).unwrap();
        assert!(path.exists());

        let loaded = LauncherConfig::load(&path).unwrap();
        assert_eq!(loaded.manifest_url, "https://test.example/manifest.toml");
        assert!(loaded.verbose);
    }

    #[test]
    fn test_config_load_nonexistent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.toml");

        let config = LauncherConfig::load(&path).unwrap();
        assert_eq!(config.manifest_url, default_manifest_url());
    }

    #[test]
    fn test_config_load_or_create() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("new_config.toml");

        assert!(!path.exists());
        let config = LauncherConfig::load_or_create(&path).unwrap();
        assert!(path.exists());
        assert_eq!(config.config_version, 1);
    }
}
