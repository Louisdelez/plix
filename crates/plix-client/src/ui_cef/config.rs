//! CEF UI configuration (Feature 030)
//!
//! # Security
//!
//! External network requests from JavaScript are blocked by design:
//! - Only local asset URLs (file://, assets/ui/*) are allowed
//! - fetch() and XMLHttpRequest to external domains are blocked by CEF
//! - The bridge is the only way for UI to interact with the network
//!
//! This is configured through CEF's request handler in the shell (Feature 030).

use serde::{Deserialize, Serialize};

/// CEF UI configuration options
///
/// Stored in client config file under `[ui]` section.
///
/// # TOML Example
///
/// ```toml
/// [ui]
/// cef_enabled = true
/// cef_devtools = false
/// cef_initial_page = "index.html"
/// cef_frame_rate = 60
/// cef_hud = true
/// cef_chat = true
/// cef_scoreboard = true
/// debug_bridge = false
/// # Feature 033: Embeds
/// cef_embeds = true
/// cef_embeds_youtube = true
/// cef_embeds_twitch = true
/// cef_embeds_spotify = false
/// cef_embeds_autoplay = false
/// cef_embeds_chat = false
/// cef_embeds_twitch_parent = "localhost"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CefConfig {
    /// Enable CEF UI (default: true if feature available)
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Enable CEF DevTools (default: false)
    #[serde(default)]
    pub devtools: bool,

    /// Path to initial HTML page (relative to assets/ui/)
    #[serde(default = "default_initial_page")]
    pub initial_page: String,

    /// CEF frame rate limit (default: 60, range: 1-120)
    #[serde(default = "default_frame_rate")]
    pub frame_rate: u32,

    /// Enable CEF HUD overlay (HP, RTT, FPS, crosshair) - Feature 032
    #[serde(default = "default_enabled")]
    pub cef_hud: bool,

    /// Enable CEF chat overlay - Feature 032
    #[serde(default = "default_enabled")]
    pub cef_chat: bool,

    /// Enable CEF scoreboard overlay - Feature 032
    #[serde(default = "default_enabled")]
    pub cef_scoreboard: bool,

    /// Enable bridge message debug logging - Feature 032
    #[serde(default)]
    pub debug_bridge: bool,

    // === Feature 033: Media Embeds ===
    /// Enable media embeds panel (default: true)
    #[serde(default = "default_enabled")]
    pub cef_embeds: bool,

    /// Enable YouTube provider (default: true)
    #[serde(default = "default_enabled")]
    pub cef_embeds_youtube: bool,

    /// Enable Twitch provider (default: true)
    #[serde(default = "default_enabled")]
    pub cef_embeds_twitch: bool,

    /// Enable Spotify provider (default: false, stubbed)
    #[serde(default)]
    pub cef_embeds_spotify: bool,

    /// Autoplay videos on load (default: false)
    #[serde(default)]
    pub cef_embeds_autoplay: bool,

    /// Show Twitch chat alongside stream (default: false)
    #[serde(default)]
    pub cef_embeds_chat: bool,

    /// Twitch embed parent domain (default: "localhost")
    #[serde(default = "default_twitch_parent")]
    pub cef_embeds_twitch_parent: String,
}

fn default_enabled() -> bool {
    true
}

fn default_initial_page() -> String {
    "index.html".to_string()
}

fn default_frame_rate() -> u32 {
    60
}

fn default_twitch_parent() -> String {
    "localhost".to_string()
}

impl Default for CefConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            devtools: false,
            initial_page: default_initial_page(),
            frame_rate: default_frame_rate(),
            cef_hud: default_enabled(),
            cef_chat: default_enabled(),
            cef_scoreboard: default_enabled(),
            debug_bridge: false,
            // Feature 033: Embeds
            cef_embeds: default_enabled(),
            cef_embeds_youtube: default_enabled(),
            cef_embeds_twitch: default_enabled(),
            cef_embeds_spotify: false,
            cef_embeds_autoplay: false,
            cef_embeds_chat: false,
            cef_embeds_twitch_parent: default_twitch_parent(),
        }
    }
}

impl CefConfig {
    /// Validate configuration values
    ///
    /// Clamps frame_rate to valid range and validates initial_page.
    pub fn validate(&mut self) {
        // Clamp frame rate to valid range
        self.frame_rate = self.frame_rate.clamp(1, 120);

        // Validate initial_page (remove path traversal attempts)
        if self.initial_page.contains("..") {
            self.initial_page = default_initial_page();
        }

        // Remove leading slashes (must be relative path)
        if self.initial_page.starts_with('/') || self.initial_page.starts_with('\\') {
            self.initial_page = self
                .initial_page
                .trim_start_matches(['/', '\\'])
                .to_string();
        }
    }

    /// Check if initial_page path is valid
    pub fn is_valid_page_path(&self) -> bool {
        !self.initial_page.contains("..")
            && !self.initial_page.starts_with('/')
            && !self.initial_page.starts_with('\\')
            && !self.initial_page.is_empty()
    }

    /// Get the full path to the initial page (relative to assets/ui/)
    pub fn initial_page_path(&self) -> String {
        format!("assets/ui/{}", self.initial_page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CefConfig::default();
        assert!(config.enabled);
        assert!(!config.devtools);
        assert_eq!(config.initial_page, "index.html");
        assert_eq!(config.frame_rate, 60);
        // Feature 032 defaults
        assert!(config.cef_hud);
        assert!(config.cef_chat);
        assert!(config.cef_scoreboard);
        assert!(!config.debug_bridge);
        // Feature 033 defaults
        assert!(config.cef_embeds);
        assert!(config.cef_embeds_youtube);
        assert!(config.cef_embeds_twitch);
        assert!(!config.cef_embeds_spotify);
        assert!(!config.cef_embeds_autoplay);
        assert!(!config.cef_embeds_chat);
        assert_eq!(config.cef_embeds_twitch_parent, "localhost");
    }

    #[test]
    fn test_validate_frame_rate() {
        let mut config = CefConfig {
            frame_rate: 0,
            ..Default::default()
        };
        config.validate();
        assert_eq!(config.frame_rate, 1);

        config.frame_rate = 200;
        config.validate();
        assert_eq!(config.frame_rate, 120);

        config.frame_rate = 60;
        config.validate();
        assert_eq!(config.frame_rate, 60);
    }

    #[test]
    fn test_validate_initial_page() {
        let mut config = CefConfig {
            initial_page: "../../../etc/passwd".to_string(),
            ..Default::default()
        };
        config.validate();
        assert_eq!(config.initial_page, "index.html"); // Reset to default

        let mut config = CefConfig {
            initial_page: "/absolute/path.html".to_string(),
            ..Default::default()
        };
        config.validate();
        assert_eq!(config.initial_page, "absolute/path.html"); // Stripped leading slash
    }

    #[test]
    fn test_is_valid_page_path() {
        let config = CefConfig::default();
        assert!(config.is_valid_page_path());

        let config = CefConfig {
            initial_page: "../bad.html".to_string(),
            ..Default::default()
        };
        assert!(!config.is_valid_page_path());

        let config = CefConfig {
            initial_page: "/absolute.html".to_string(),
            ..Default::default()
        };
        assert!(!config.is_valid_page_path());

        let config = CefConfig {
            initial_page: "".to_string(),
            ..Default::default()
        };
        assert!(!config.is_valid_page_path());
    }

    #[test]
    fn test_initial_page_path() {
        let config = CefConfig::default();
        assert_eq!(config.initial_page_path(), "assets/ui/index.html");

        let config = CefConfig {
            initial_page: "menus/main.html".to_string(),
            ..Default::default()
        };
        assert_eq!(config.initial_page_path(), "assets/ui/menus/main.html");
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = CefConfig {
            enabled: false,
            devtools: true,
            initial_page: "custom.html".to_string(),
            frame_rate: 30,
            cef_hud: false,
            cef_chat: true,
            cef_scoreboard: false,
            debug_bridge: true,
            // Feature 033 fields
            cef_embeds: false,
            cef_embeds_youtube: false,
            cef_embeds_twitch: true,
            cef_embeds_spotify: true,
            cef_embeds_autoplay: true,
            cef_embeds_chat: true,
            cef_embeds_twitch_parent: "example.com".to_string(),
        };

        let toml_str = toml::to_string(&config).unwrap();
        let parsed: CefConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.enabled, config.enabled);
        assert_eq!(parsed.devtools, config.devtools);
        assert_eq!(parsed.initial_page, config.initial_page);
        assert_eq!(parsed.frame_rate, config.frame_rate);
        // Feature 032 fields
        assert_eq!(parsed.cef_hud, config.cef_hud);
        assert_eq!(parsed.cef_chat, config.cef_chat);
        assert_eq!(parsed.cef_scoreboard, config.cef_scoreboard);
        assert_eq!(parsed.debug_bridge, config.debug_bridge);
        // Feature 033 fields
        assert_eq!(parsed.cef_embeds, config.cef_embeds);
        assert_eq!(parsed.cef_embeds_youtube, config.cef_embeds_youtube);
        assert_eq!(parsed.cef_embeds_twitch, config.cef_embeds_twitch);
        assert_eq!(parsed.cef_embeds_spotify, config.cef_embeds_spotify);
        assert_eq!(parsed.cef_embeds_autoplay, config.cef_embeds_autoplay);
        assert_eq!(parsed.cef_embeds_chat, config.cef_embeds_chat);
        assert_eq!(
            parsed.cef_embeds_twitch_parent,
            config.cef_embeds_twitch_parent
        );
    }
}
