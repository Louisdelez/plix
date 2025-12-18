//! Directory structure management

use crate::error::LauncherError;
use std::path::PathBuf;

/// Directory paths used by the launcher
#[derive(Debug, Clone)]
pub struct LauncherDirs {
    /// Config directory (~/.config/plix/)
    pub config_dir: PathBuf,
    /// Data directory (~/.local/share/plix/)
    pub data_dir: PathBuf,
    /// Versions directory (~/.local/share/plix/versions/)
    pub versions_dir: PathBuf,
    /// Current version symlink/directory (~/.local/share/plix/current/)
    pub current_dir: PathBuf,
    /// Launcher state directory (~/.local/share/plix/launcher/)
    pub launcher_dir: PathBuf,
    /// Logs directory (~/.local/share/plix/logs/)
    pub logs_dir: PathBuf,
}

impl LauncherDirs {
    /// Create launcher directories with default XDG paths
    pub fn new() -> Result<Self, LauncherError> {
        let config_dir = dirs_next::config_dir()
            .ok_or_else(|| LauncherError::Config("Could not determine config directory".into()))?
            .join("plix");

        let data_dir = dirs_next::data_dir()
            .ok_or_else(|| LauncherError::Config("Could not determine data directory".into()))?
            .join("plix");

        Ok(Self {
            config_dir: config_dir.clone(),
            data_dir: data_dir.clone(),
            versions_dir: data_dir.join("versions"),
            current_dir: data_dir.join("current"),
            launcher_dir: data_dir.join("launcher"),
            logs_dir: data_dir.join("logs"),
        })
    }

    /// Create launcher directories with a custom data directory
    pub fn with_data_dir(data_dir: PathBuf) -> Result<Self, LauncherError> {
        let config_dir = dirs_next::config_dir()
            .ok_or_else(|| LauncherError::Config("Could not determine config directory".into()))?
            .join("plix");

        Ok(Self {
            config_dir,
            data_dir: data_dir.clone(),
            versions_dir: data_dir.join("versions"),
            current_dir: data_dir.join("current"),
            launcher_dir: data_dir.join("launcher"),
            logs_dir: data_dir.join("logs"),
        })
    }

    /// Path to the launcher config file
    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("launcher.toml")
    }

    /// Path to the launcher state file
    pub fn state_file(&self) -> PathBuf {
        self.launcher_dir.join("state.toml")
    }

    /// Path to the launcher log file
    pub fn log_file(&self) -> PathBuf {
        self.logs_dir.join("launcher.log")
    }

    /// Path to a specific version directory
    pub fn version_dir(&self, version: &str) -> PathBuf {
        self.versions_dir.join(version)
    }

    /// Path to the game binary in the current directory
    pub fn game_binary(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            self.current_dir.join("plix-client.exe")
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.current_dir.join("plix-client")
        }
    }

    /// Ensure all required directories exist
    pub fn ensure_dirs(&self) -> Result<(), LauncherError> {
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.versions_dir)?;
        std::fs::create_dir_all(&self.launcher_dir)?;
        std::fs::create_dir_all(&self.logs_dir)?;
        Ok(())
    }
}

impl Default for LauncherDirs {
    fn default() -> Self {
        Self::new().expect("Failed to determine default directories")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launcher_dirs_new() {
        let dirs = LauncherDirs::new();
        assert!(dirs.is_ok());
        let dirs = dirs.unwrap();
        assert!(dirs.config_dir.ends_with("plix"));
        assert!(dirs.data_dir.ends_with("plix"));
    }

    #[test]
    fn test_launcher_dirs_with_custom_data() {
        let custom_data = PathBuf::from("/tmp/plix-test");
        let dirs = LauncherDirs::with_data_dir(custom_data.clone()).unwrap();
        assert_eq!(dirs.data_dir, custom_data);
        assert_eq!(dirs.versions_dir, custom_data.join("versions"));
    }

    #[test]
    fn test_version_dir() {
        let dirs = LauncherDirs::new().unwrap();
        let ver_dir = dirs.version_dir("1.2.3");
        assert!(ver_dir.ends_with("versions/1.2.3"));
    }
}
