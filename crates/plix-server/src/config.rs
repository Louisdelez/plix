//! Server Configuration
//!
//! TOML-based configuration for the headless server with validation.

use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

/// Server configuration errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Port validation failed.
    #[error("Invalid port: {0}")]
    InvalidPort(String),

    /// Tick rate validation failed.
    #[error("Invalid tick rate {0}: must be between 20 and 60 Hz")]
    InvalidTickRate(u8),

    /// Max players validation failed.
    #[error("Invalid max_players {0}: must be between 1 and 128")]
    InvalidMaxPlayers(u8),

    /// Bind address validation failed.
    #[error("Invalid bind address: {0}")]
    InvalidBindAddress(String),

    /// Arena not found.
    #[error("Arena not found: {0}\n\nHint: Check available arenas in assets/arenas/")]
    ArenaNotFound(String),

    /// Config file read error.
    #[error("Failed to read config file: {0}")]
    FileRead(String),

    /// Config file parse error.
    #[error("Failed to parse config file: {0}")]
    FileParse(String),
}

/// Current configuration version for migration support.
pub const CONFIG_VERSION: &str = "1.0.0";

/// Complete server configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Configuration file version (for migration support)
    #[serde(default = "default_config_version")]
    pub config_version: String,
    /// Network settings
    pub network: NetworkConfig,
    /// Game settings
    pub game: GameConfig,
    /// Server metadata
    pub server: ServerMetaConfig,
    /// Logging settings
    pub logging: LoggingConfig,
    /// Persistence settings
    pub persistence: PersistenceConfig,
}

fn default_config_version() -> String {
    CONFIG_VERSION.to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            config_version: CONFIG_VERSION.to_string(),
            network: NetworkConfig::default(),
            game: GameConfig::default(),
            server: ServerMetaConfig::default(),
            logging: LoggingConfig::default(),
            persistence: PersistenceConfig::default(),
        }
    }
}

impl ServerConfig {
    /// Load configuration from a TOML file.
    pub fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::FileRead(format!("{}: {}", path.display(), e)))?;

        toml::from_str(&content)
            .map_err(|e| ConfigError::FileParse(format!("{}: {}", path.display(), e)))
    }

    /// Validate the configuration.
    ///
    /// Returns a list of all validation errors found.
    pub fn validate(&self) -> Result<(), Vec<ConfigError>> {
        let mut errors = Vec::new();

        // Validate port
        if self.network.port == 0 {
            errors.push(ConfigError::InvalidPort("Port cannot be 0".to_string()));
        }

        // Validate tick rate (20-60 Hz is reasonable)
        if self.game.tick_rate < 20 || self.game.tick_rate > 60 {
            errors.push(ConfigError::InvalidTickRate(self.game.tick_rate));
        }

        // Validate max players (1-128)
        if self.network.max_players == 0 || self.network.max_players > 128 {
            errors.push(ConfigError::InvalidMaxPlayers(self.network.max_players));
        }

        // Validate bind address format
        if self.network.bind_address.is_empty() {
            errors.push(ConfigError::InvalidBindAddress(
                "Bind address cannot be empty".to_string(),
            ));
        } else if self.network.bind_address.parse::<std::net::IpAddr>().is_err() {
            errors.push(ConfigError::InvalidBindAddress(format!(
                "'{}' is not a valid IP address",
                self.network.bind_address
            )));
        }

        // Validate arena name (basic check - full validation requires asset loading)
        if self.game.arena.is_empty() {
            errors.push(ConfigError::ArenaNotFound("Arena name cannot be empty".to_string()));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Network configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// IP address to bind to
    pub bind_address: String,
    /// UDP port for game traffic
    pub port: u16,
    /// Maximum concurrent players
    pub max_players: u8,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 7777,
            max_players: 32,
        }
    }
}

/// Game configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GameConfig {
    /// Server tick rate in Hz
    pub tick_rate: u8,
    /// Arena to load
    pub arena: String,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            tick_rate: 30,
            arena: "ffa_small".to_string(),
        }
    }
}

/// Server metadata configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerMetaConfig {
    /// Server display name
    pub name: String,
    /// Message of the day
    pub motd: String,
}

impl Default for ServerMetaConfig {
    fn default() -> Self {
        Self {
            name: "Plix Server".to_string(),
            motd: String::new(),
        }
    }
}

/// Logging configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

/// Persistence configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PersistenceConfig {
    /// Autosave interval in seconds
    pub autosave_interval_secs: u64,
    /// Graceful shutdown timeout in seconds
    pub shutdown_timeout_secs: u64,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            autosave_interval_secs: 300,
            shutdown_timeout_secs: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.network.port, 7777);
        assert_eq!(config.network.max_players, 32);
        assert_eq!(config.game.tick_rate, 30);
    }

    #[test]
    fn test_validate_default() {
        let config = ServerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_port() {
        let mut config = ServerConfig::default();
        config.network.port = 0;

        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, ConfigError::InvalidPort(_))));
    }

    #[test]
    fn test_validate_invalid_tick_rate() {
        let mut config = ServerConfig::default();
        config.game.tick_rate = 10;

        let errors = config.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ConfigError::InvalidTickRate(_))));
    }

    #[test]
    fn test_validate_invalid_max_players() {
        let mut config = ServerConfig::default();
        config.network.max_players = 0;

        let errors = config.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ConfigError::InvalidMaxPlayers(_))));
    }

    #[test]
    fn test_validate_multiple_errors() {
        let mut config = ServerConfig::default();
        config.network.port = 0;
        config.game.tick_rate = 100;
        config.network.max_players = 200;

        let errors = config.validate().unwrap_err();
        assert!(errors.len() >= 3);
    }

    #[test]
    fn test_toml_parsing() {
        let toml_str = r#"
            [network]
            bind_address = "127.0.0.1"
            port = 8888
            max_players = 16

            [game]
            tick_rate = 60
            arena = "tdm_medium"

            [server]
            name = "Test Server"
            motd = "Welcome!"

            [logging]
            level = "debug"

            [persistence]
            autosave_interval_secs = 600
            shutdown_timeout_secs = 10
        "#;

        let config: ServerConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.network.bind_address, "127.0.0.1");
        assert_eq!(config.network.port, 8888);
        assert_eq!(config.network.max_players, 16);
        assert_eq!(config.game.tick_rate, 60);
        assert_eq!(config.game.arena, "tdm_medium");
        assert_eq!(config.server.name, "Test Server");
        assert_eq!(config.persistence.shutdown_timeout_secs, 10);
    }
}
