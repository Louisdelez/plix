//! Matchmaking preferences persistence.

use serde::{Deserialize, Serialize};

use crate::profile::{load_profile, save_profile, PlayerProfile, ProfileError};

/// Default preferred mode.
fn default_mode() -> String {
    "tdm".to_string()
}

/// Default preferred region.
fn default_region() -> String {
    "any".to_string()
}

/// User preferences for quick join matchmaking.
///
/// Persisted to `~/.config/plix/profile.toml` under `[matchmaking]` section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatchmakingPreferences {
    /// Preferred game mode (default: "tdm")
    #[serde(default = "default_mode")]
    pub preferred_mode: String,

    /// Preferred region (default: "any")
    #[serde(default = "default_region")]
    pub preferred_region: String,
}

impl Default for MatchmakingPreferences {
    fn default() -> Self {
        Self {
            preferred_mode: default_mode(),
            preferred_region: default_region(),
        }
    }
}

impl MatchmakingPreferences {
    /// Create new preferences with specified values.
    pub fn new(mode: impl Into<String>, region: impl Into<String>) -> Self {
        Self {
            preferred_mode: mode.into().to_lowercase(),
            preferred_region: region.into().to_lowercase(),
        }
    }

    /// Update preferred mode.
    pub fn set_mode(&mut self, mode: impl Into<String>) {
        self.preferred_mode = mode.into().to_lowercase();
    }

    /// Update preferred region.
    pub fn set_region(&mut self, region: impl Into<String>) {
        self.preferred_region = region.into().to_lowercase();
    }
}

/// T045: Load matchmaking preferences from profile.
///
/// Loads the player profile and returns the matchmaking preferences section.
pub fn load_preferences() -> MatchmakingPreferences {
    let profile = load_profile();
    profile.matchmaking
}

/// T046: Save matchmaking preferences to profile.
///
/// Updates the matchmaking section of the player profile and saves it.
pub fn save_preferences(prefs: &MatchmakingPreferences) -> Result<(), ProfileError> {
    let mut profile = load_profile();
    profile.matchmaking = prefs.clone();
    save_profile(&profile)
}

/// Update a specific preference and save.
///
/// Helper that loads, modifies, and saves in one operation.
pub fn update_preference_mode(mode: impl Into<String>) -> Result<(), ProfileError> {
    let mut profile = load_profile();
    profile.matchmaking.set_mode(mode);
    save_profile(&profile)
}

/// Update region preference and save.
pub fn update_preference_region(region: impl Into<String>) -> Result<(), ProfileError> {
    let mut profile = load_profile();
    profile.matchmaking.set_region(region);
    save_profile(&profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_preferences() {
        let prefs = MatchmakingPreferences::default();
        assert_eq!(prefs.preferred_mode, "tdm");
        assert_eq!(prefs.preferred_region, "any");
    }

    #[test]
    fn test_new_preferences() {
        let prefs = MatchmakingPreferences::new("FFA", "EU");
        assert_eq!(prefs.preferred_mode, "ffa");
        assert_eq!(prefs.preferred_region, "eu");
    }

    #[test]
    fn test_set_mode() {
        let mut prefs = MatchmakingPreferences::default();
        prefs.set_mode("CTF");
        assert_eq!(prefs.preferred_mode, "ctf");
    }

    #[test]
    fn test_set_region() {
        let mut prefs = MatchmakingPreferences::default();
        prefs.set_region("ASIA");
        assert_eq!(prefs.preferred_region, "asia");
    }

    #[test]
    fn test_serde_roundtrip() {
        let prefs = MatchmakingPreferences::new("ffa", "us");
        let toml_str = toml::to_string(&prefs).expect("serialize");
        let parsed: MatchmakingPreferences = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(prefs, parsed);
    }

    #[test]
    fn test_serde_with_defaults() {
        // Empty TOML should deserialize to defaults
        let toml_str = "";
        let parsed: MatchmakingPreferences = toml::from_str(toml_str).expect("deserialize");
        assert_eq!(parsed.preferred_mode, "tdm");
        assert_eq!(parsed.preferred_region, "any");
    }

    #[test]
    fn test_serde_partial() {
        // Partial TOML should fill in defaults
        let toml_str = r#"preferred_mode = "ctf""#;
        let parsed: MatchmakingPreferences = toml::from_str(toml_str).expect("deserialize");
        assert_eq!(parsed.preferred_mode, "ctf");
        assert_eq!(parsed.preferred_region, "any"); // default
    }
}
