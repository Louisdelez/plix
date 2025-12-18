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

impl Default for CefConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            devtools: false,
            initial_page: default_initial_page(),
            frame_rate: default_frame_rate(),
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
        };

        let toml_str = toml::to_string(&config).unwrap();
        let parsed: CefConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.enabled, config.enabled);
        assert_eq!(parsed.devtools, config.devtools);
        assert_eq!(parsed.initial_page, config.initial_page);
        assert_eq!(parsed.frame_rate, config.frame_rate);
    }
}
