//! Master server configuration for heartbeat announcements.

use serde::{Deserialize, Serialize};

/// Configuration for announcing this server to a master server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterConfig {
    /// Whether master server announcement is enabled
    pub enabled: bool,
    /// Master server URL (e.g., "http://localhost:8080")
    pub url: String,
    /// Server display name
    pub name: String,
    /// Geographic region (e.g., "eu-west", "us-east")
    pub region: String,
    /// Server tags for filtering (e.g., ["competitive", "casual"])
    pub tags: Vec<String>,
    /// Game modes supported (e.g., ["ffa", "ctf", "tdm"])
    pub game_modes: Vec<String>,
    /// Heartbeat interval in seconds (default: 20)
    pub heartbeat_interval_secs: u64,
}

impl Default for MasterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: String::new(),
            name: "Plix Server".to_string(),
            region: "unknown".to_string(),
            tags: Vec::new(),
            game_modes: vec!["ffa".to_string()],
            heartbeat_interval_secs: 20,
        }
    }
}

impl MasterConfig {
    /// Create a new master config with the specified URL and name.
    pub fn new(url: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            enabled: true,
            url: url.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the region.
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    /// Set the tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set the game modes.
    pub fn with_game_modes(mut self, game_modes: Vec<String>) -> Self {
        self.game_modes = game_modes;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_disabled() {
        let config = MasterConfig::default();
        assert!(!config.enabled);
        assert!(config.url.is_empty());
    }

    #[test]
    fn test_new_config_enabled() {
        let config = MasterConfig::new("http://localhost:8080", "My Server");
        assert!(config.enabled);
        assert_eq!(config.url, "http://localhost:8080");
        assert_eq!(config.name, "My Server");
    }

    #[test]
    fn test_builder_pattern() {
        let config = MasterConfig::new("http://master.example.com", "Test Server")
            .with_region("eu-west")
            .with_tags(vec!["competitive".to_string(), "ranked".to_string()])
            .with_game_modes(vec!["ctf".to_string()]);

        assert_eq!(config.region, "eu-west");
        assert_eq!(config.tags, vec!["competitive", "ranked"]);
        assert_eq!(config.game_modes, vec!["ctf"]);
    }
}
