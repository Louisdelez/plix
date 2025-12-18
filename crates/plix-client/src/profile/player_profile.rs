//! Player profile persistence
//!
//! Handles saving and loading player profile from local configuration.
//! The profile includes display name and future auth credentials.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

use crate::matchmaking::MatchmakingPreferences;

/// Default profile version
pub const PROFILE_VERSION: u32 = 1;

/// Default display name for new profiles
pub const DEFAULT_DISPLAY_NAME: &str = "Player";

/// Profile-related errors
#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
}

/// Player profile stored locally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    /// Profile version for migrations
    pub version: u32,
    /// Player's chosen display name
    pub display_name: String,
    /// Account ID placeholder (v2 - always None in v1)
    #[serde(default)]
    pub account_id: Option<u64>,
    /// Auth token placeholder (v2 - always None in v1)
    #[serde(default)]
    pub auth_token: Option<String>,
    /// Matchmaking preferences for quick join
    #[serde(default)]
    pub matchmaking: MatchmakingPreferences,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            version: PROFILE_VERSION,
            display_name: DEFAULT_DISPLAY_NAME.to_string(),
            account_id: None,
            auth_token: None,
            matchmaking: MatchmakingPreferences::default(),
        }
    }
}

impl PlayerProfile {
    /// Create a new profile with the given display name
    pub fn new(display_name: String) -> Self {
        Self {
            version: PROFILE_VERSION,
            display_name,
            account_id: None,
            auth_token: None,
            matchmaking: MatchmakingPreferences::default(),
        }
    }
}

/// Get the path to the profile file
///
/// Returns `~/.config/plix/profile.toml` on Unix systems.
pub fn profile_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("plix").join("profile.toml")
}

/// Load profile from disk, returning defaults if not found or corrupted
pub fn load_profile() -> PlayerProfile {
    let path = profile_path();

    // File doesn't exist - return defaults
    if !path.exists() {
        return PlayerProfile::default();
    }

    // Try to read and parse
    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str::<PlayerProfile>(&content) {
            Ok(profile) => profile,
            Err(e) => {
                // Log warning about corrupted profile
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    "Corrupted profile file, using defaults"
                );
                PlayerProfile::default()
            }
        },
        Err(e) => {
            tracing::warn!(
                path = %path.display(),
                error = %e,
                "Could not read profile file, using defaults"
            );
            PlayerProfile::default()
        }
    }
}

/// Save profile to disk with atomic write (temp file + rename)
pub fn save_profile(profile: &PlayerProfile) -> Result<(), ProfileError> {
    let path = profile_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Serialize to TOML
    let content = toml::to_string_pretty(profile)?;

    // Write to temp file first for atomic update
    let temp_path = path.with_extension("toml.tmp");
    fs::write(&temp_path, &content)?;

    // Rename temp file to final path (atomic on most filesystems)
    fs::rename(&temp_path, &path)?;

    tracing::debug!(path = %path.display(), "Profile saved");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // T036: Unit tests for profile loading/saving

    #[test]
    fn test_default_profile() {
        let profile = PlayerProfile::default();
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.display_name, DEFAULT_DISPLAY_NAME);
        assert!(profile.account_id.is_none());
        assert!(profile.auth_token.is_none());
    }

    #[test]
    fn test_profile_new() {
        let profile = PlayerProfile::new("Alice".to_string());
        assert_eq!(profile.display_name, "Alice");
    }

    #[test]
    fn test_profile_serde_roundtrip() {
        let profile = PlayerProfile::new("TestPlayer".to_string());
        let toml_str = toml::to_string(&profile).unwrap();
        let parsed: PlayerProfile = toml::from_str(&toml_str).unwrap();
        assert_eq!(profile.display_name, parsed.display_name);
        assert_eq!(profile.version, parsed.version);
    }

    #[test]
    fn test_profile_path_exists() {
        let path = profile_path();
        // Should end with profile.toml
        assert!(path.to_string_lossy().ends_with("profile.toml"));
    }

    // T008: Save/load roundtrip test for profile with matchmaking section
    #[test]
    fn test_profile_matchmaking_serde_roundtrip() {
        let mut profile = PlayerProfile::new("MatchPlayer".to_string());

        // Set custom matchmaking preferences
        profile.matchmaking.preferred_mode = "ctf".to_string();
        profile.matchmaking.preferred_region = "na".to_string();

        // Serialize to TOML
        let toml_str = toml::to_string(&profile).unwrap();

        // Verify matchmaking section appears in TOML
        assert!(toml_str.contains("[matchmaking]"));
        assert!(toml_str.contains("preferred_mode"));
        assert!(toml_str.contains("preferred_region"));

        // Deserialize and verify
        let parsed: PlayerProfile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.display_name, "MatchPlayer");
        assert_eq!(parsed.matchmaking.preferred_mode, "ctf");
        assert_eq!(parsed.matchmaking.preferred_region, "na");
    }

    #[test]
    fn test_profile_loads_without_matchmaking_section() {
        // Test backward compatibility: old profile without matchmaking section
        let toml_str = r#"
version = 1
display_name = "OldPlayer"
"#;
        let parsed: PlayerProfile = toml::from_str(toml_str).unwrap();
        assert_eq!(parsed.display_name, "OldPlayer");
        // Should get default matchmaking preferences (mode=tdm, region=any)
        assert_eq!(parsed.matchmaking.preferred_mode, "tdm");
        assert_eq!(parsed.matchmaking.preferred_region, "any");
    }

    #[test]
    fn test_default_profile_has_matchmaking() {
        let profile = PlayerProfile::default();
        // Default mode is "tdm", default region is "any"
        assert_eq!(profile.matchmaking.preferred_mode, "tdm");
        assert_eq!(profile.matchmaking.preferred_region, "any");
    }
}
