//! Embed-specific configuration (Feature 033)
//!
//! This module provides a focused view of embed config from CefConfig.

use crate::ui_cef::CefConfig;

/// Embed configuration view
///
/// Provides convenient access to embed-related config fields.
#[derive(Debug, Clone)]
pub struct EmbedConfig {
    /// Embeds feature enabled
    pub enabled: bool,
    /// YouTube provider enabled
    pub youtube_enabled: bool,
    /// Twitch provider enabled
    pub twitch_enabled: bool,
    /// Spotify provider enabled
    pub spotify_enabled: bool,
    /// Autoplay videos
    pub autoplay: bool,
    /// Show Twitch chat
    pub twitch_chat: bool,
    /// Twitch parent domain
    pub twitch_parent: String,
}

impl From<&CefConfig> for EmbedConfig {
    fn from(config: &CefConfig) -> Self {
        Self {
            enabled: config.cef_embeds,
            youtube_enabled: config.cef_embeds_youtube,
            twitch_enabled: config.cef_embeds_twitch,
            spotify_enabled: config.cef_embeds_spotify,
            autoplay: config.cef_embeds_autoplay,
            twitch_chat: config.cef_embeds_chat,
            twitch_parent: config.cef_embeds_twitch_parent.clone(),
        }
    }
}

impl Default for EmbedConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            youtube_enabled: true,
            twitch_enabled: true,
            spotify_enabled: false,
            autoplay: false,
            twitch_chat: false,
            twitch_parent: "localhost".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_cef_config() {
        let cef_config = CefConfig::default();
        let embed_config = EmbedConfig::from(&cef_config);

        assert!(embed_config.enabled);
        assert!(embed_config.youtube_enabled);
        assert!(embed_config.twitch_enabled);
        assert!(!embed_config.spotify_enabled);
        assert!(!embed_config.autoplay);
        assert!(!embed_config.twitch_chat);
        assert_eq!(embed_config.twitch_parent, "localhost");
    }
}
