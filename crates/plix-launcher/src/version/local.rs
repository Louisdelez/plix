//! Local state management

use crate::error::LauncherError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Local launcher state persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalState {
    /// State format version
    #[serde(default = "default_state_version")]
    pub state_version: u32,

    /// Currently installed version (None if fresh install)
    pub installed_version: Option<String>,

    /// Last successful update timestamp
    pub last_update: Option<u64>,

    /// Checksums of installed files (path -> sha256)
    #[serde(default)]
    pub file_checksums: HashMap<String, String>,

    /// Installation directory path
    pub install_path: Option<String>,
}

fn default_state_version() -> u32 {
    1
}

impl Default for LocalState {
    fn default() -> Self {
        Self {
            state_version: default_state_version(),
            installed_version: None,
            last_update: None,
            file_checksums: HashMap::new(),
            install_path: None,
        }
    }
}

impl LocalState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self::default()
    }

    /// Load state from file, returning default if not exists
    pub fn load(path: &Path) -> Result<Self, LauncherError> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let state: LocalState = toml::from_str(&content)?;
            Ok(state)
        } else {
            Ok(Self::default())
        }
    }

    /// Save state to file using atomic write
    pub fn save(&self, path: &Path) -> Result<(), LauncherError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| LauncherError::Config(format!("Failed to serialize state: {}", e)))?;

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

    /// Check if any version is installed
    pub fn has_installed_version(&self) -> bool {
        self.installed_version.is_some()
    }

    /// Update state after successful installation
    pub fn update_after_install(
        &mut self,
        version: String,
        install_path: String,
        checksums: HashMap<String, String>,
    ) {
        self.installed_version = Some(version);
        self.install_path = Some(install_path);
        self.file_checksums = checksums;
        self.last_update = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_state() {
        let state = LocalState::new();
        assert_eq!(state.state_version, 1);
        assert!(state.installed_version.is_none());
        assert!(state.file_checksums.is_empty());
    }

    #[test]
    fn test_state_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.toml");

        let mut state = LocalState::new();
        state.installed_version = Some("1.2.3".to_string());
        state
            .file_checksums
            .insert("plix-client".to_string(), "abc123".to_string());

        state.save(&path).unwrap();
        assert!(path.exists());

        let loaded = LocalState::load(&path).unwrap();
        assert_eq!(loaded.installed_version, Some("1.2.3".to_string()));
        assert_eq!(
            loaded.file_checksums.get("plix-client"),
            Some(&"abc123".to_string())
        );
    }

    #[test]
    fn test_state_load_nonexistent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.toml");

        let state = LocalState::load(&path).unwrap();
        assert!(state.installed_version.is_none());
    }

    #[test]
    fn test_has_installed_version() {
        let mut state = LocalState::new();
        assert!(!state.has_installed_version());

        state.installed_version = Some("1.0.0".to_string());
        assert!(state.has_installed_version());
    }

    #[test]
    fn test_update_after_install() {
        let mut state = LocalState::new();
        let mut checksums = HashMap::new();
        checksums.insert("file1".to_string(), "hash1".to_string());

        state.update_after_install(
            "2.0.0".to_string(),
            "/path/to/install".to_string(),
            checksums,
        );

        assert_eq!(state.installed_version, Some("2.0.0".to_string()));
        assert_eq!(state.install_path, Some("/path/to/install".to_string()));
        assert!(state.last_update.is_some());
        assert_eq!(
            state.file_checksums.get("file1"),
            Some(&"hash1".to_string())
        );
    }
}
