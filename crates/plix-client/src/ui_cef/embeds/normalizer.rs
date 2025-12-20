//! URL normalization for embed providers (Feature 033)
//!
//! Converts various URL formats to canonical embed URLs.

use super::provider::EmbedProvider;
use crate::ui_cef::bridge::messages::BridgeError;
use crate::ui_cef::CefConfig;

/// URL normalizer for embed providers
pub struct UrlNormalizer;

impl UrlNormalizer {
    /// Create a new URL normalizer
    pub fn new() -> Self {
        Self
    }

    /// Normalize a URL or ID to a canonical embed URL
    ///
    /// Returns (content_id, embed_url) on success.
    ///
    /// # Errors
    ///
    /// Returns `BridgeError::embed_invalid_url()` if the URL cannot be parsed.
    pub fn normalize(
        &self,
        provider: EmbedProvider,
        url_or_id: &str,
        config: &CefConfig,
    ) -> Result<(String, String), BridgeError> {
        let url_or_id = url_or_id.trim();

        match provider {
            EmbedProvider::YouTube => self.normalize_youtube(url_or_id, config),
            EmbedProvider::Twitch => self.normalize_twitch(url_or_id, config),
            EmbedProvider::Spotify => self.normalize_spotify(url_or_id),
        }
    }

    /// Normalize YouTube URL
    fn normalize_youtube(
        &self,
        url_or_id: &str,
        config: &CefConfig,
    ) -> Result<(String, String), BridgeError> {
        let video_id = self.extract_youtube_id(url_or_id)?;

        // Validate video ID (should be 11 characters, alphanumeric + _ -)
        if video_id.len() != 11
            || !video_id
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(BridgeError::embed_invalid_url());
        }

        let autoplay = if config.cef_embeds_autoplay { "1" } else { "0" };
        let embed_url = format!(
            "https://www.youtube-nocookie.com/embed/{}?autoplay={}&controls=1",
            video_id, autoplay
        );

        Ok((video_id, embed_url))
    }

    /// Extract YouTube video ID from various URL formats
    fn extract_youtube_id(&self, url_or_id: &str) -> Result<String, BridgeError> {
        // Direct ID (11 chars)
        if url_or_id.len() == 11
            && url_or_id
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Ok(url_or_id.to_string());
        }

        // URL patterns
        // youtube.com/watch?v=VIDEO_ID
        if let Some(id) = self.extract_query_param(url_or_id, "v") {
            return Ok(id);
        }

        // youtu.be/VIDEO_ID
        if url_or_id.contains("youtu.be/") {
            if let Some(id) = url_or_id.split("youtu.be/").nth(1) {
                let id = id.split(['?', '&', '#']).next().unwrap_or("");
                if !id.is_empty() {
                    return Ok(id.to_string());
                }
            }
        }

        // youtube.com/shorts/VIDEO_ID
        if url_or_id.contains("/shorts/") {
            if let Some(id) = url_or_id.split("/shorts/").nth(1) {
                let id = id.split(['?', '&', '#', '/']).next().unwrap_or("");
                if !id.is_empty() {
                    return Ok(id.to_string());
                }
            }
        }

        // youtube.com/embed/VIDEO_ID
        if url_or_id.contains("/embed/") {
            if let Some(id) = url_or_id.split("/embed/").nth(1) {
                let id = id.split(['?', '&', '#', '/']).next().unwrap_or("");
                if !id.is_empty() {
                    return Ok(id.to_string());
                }
            }
        }

        Err(BridgeError::embed_invalid_url())
    }

    /// Extract a query parameter from a URL
    fn extract_query_param(&self, url: &str, param: &str) -> Option<String> {
        let query = url.split('?').nth(1)?;
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if parts.next() == Some(param) {
                if let Some(value) = parts.next() {
                    // Remove any fragment
                    let value = value.split('#').next().unwrap_or(value);
                    if !value.is_empty() {
                        return Some(value.to_string());
                    }
                }
            }
        }
        None
    }

    /// Normalize Twitch URL
    fn normalize_twitch(
        &self,
        url_or_id: &str,
        config: &CefConfig,
    ) -> Result<(String, String), BridgeError> {
        let channel = self.extract_twitch_channel(url_or_id)?;

        // Validate channel name (alphanumeric + underscore, 4-25 chars typically)
        if channel.is_empty()
            || channel.len() > 25
            || !channel.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            return Err(BridgeError::embed_invalid_url());
        }

        let parent = &config.cef_embeds_twitch_parent;
        let embed_url = format!(
            "https://player.twitch.tv/?channel={}&parent={}",
            channel, parent
        );

        Ok((channel, embed_url))
    }

    /// Extract Twitch channel name from various URL formats
    fn extract_twitch_channel(&self, url_or_id: &str) -> Result<String, BridgeError> {
        // Direct channel name (no slashes, no dots)
        if !url_or_id.contains('/') && !url_or_id.contains('.') && !url_or_id.is_empty() {
            return Ok(url_or_id.to_string());
        }

        // twitch.tv/CHANNEL
        if url_or_id.contains("twitch.tv/") {
            if let Some(path) = url_or_id.split("twitch.tv/").nth(1) {
                let channel = path.split(['?', '&', '#', '/']).next().unwrap_or("");
                if !channel.is_empty() && channel != "embed" && channel != "popout" {
                    return Ok(channel.to_string());
                }
            }
        }

        // player.twitch.tv/?channel=CHANNEL
        if let Some(channel) = self.extract_query_param(url_or_id, "channel") {
            return Ok(channel);
        }

        Err(BridgeError::embed_invalid_url())
    }

    /// Normalize Spotify URL (stub)
    fn normalize_spotify(&self, url_or_id: &str) -> Result<(String, String), BridgeError> {
        // Spotify is stubbed - just validate it looks like a Spotify ID/URL
        let content_id = if url_or_id.contains("open.spotify.com/") {
            // Extract track/album/playlist ID
            if let Some(path) = url_or_id.split("open.spotify.com/").nth(1) {
                let parts: Vec<&str> = path.split('/').collect();
                if parts.len() >= 2 {
                    // track/TRACK_ID or album/ALBUM_ID etc
                    parts
                        .get(1)
                        .map(|s| s.split('?').next().unwrap_or(""))
                        .unwrap_or("")
                } else {
                    ""
                }
            } else {
                ""
            }
        } else {
            // Assume direct ID
            url_or_id
        };

        if content_id.is_empty() {
            return Err(BridgeError::embed_invalid_url());
        }

        // Spotify embed URL (stubbed - would need actual format)
        let embed_url = format!("https://open.spotify.com/embed/track/{}", content_id);

        Ok((content_id.to_string(), embed_url))
    }
}

impl Default for UrlNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> CefConfig {
        CefConfig::default()
    }

    #[test]
    fn test_youtube_direct_id() {
        let normalizer = UrlNormalizer::new();
        let config = test_config();

        let (id, url) = normalizer
            .normalize(EmbedProvider::YouTube, "dQw4w9WgXcQ", &config)
            .unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
        assert!(url.contains("youtube-nocookie.com/embed/dQw4w9WgXcQ"));
    }

    #[test]
    fn test_youtube_watch_url() {
        let normalizer = UrlNormalizer::new();
        let config = test_config();

        let (id, _) = normalizer
            .normalize(
                EmbedProvider::YouTube,
                "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
                &config,
            )
            .unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_youtube_short_url() {
        let normalizer = UrlNormalizer::new();
        let config = test_config();

        let (id, _) = normalizer
            .normalize(
                EmbedProvider::YouTube,
                "https://youtu.be/dQw4w9WgXcQ",
                &config,
            )
            .unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_youtube_shorts_url() {
        let normalizer = UrlNormalizer::new();
        let config = test_config();

        let (id, _) = normalizer
            .normalize(
                EmbedProvider::YouTube,
                "https://youtube.com/shorts/dQw4w9WgXcQ",
                &config,
            )
            .unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_youtube_invalid() {
        let normalizer = UrlNormalizer::new();
        let config = test_config();

        assert!(normalizer
            .normalize(EmbedProvider::YouTube, "invalid", &config)
            .is_err());
        assert!(normalizer
            .normalize(EmbedProvider::YouTube, "", &config)
            .is_err());
    }

    #[test]
    fn test_twitch_direct_channel() {
        let normalizer = UrlNormalizer::new();
        let config = test_config();

        let (channel, url) = normalizer
            .normalize(EmbedProvider::Twitch, "shroud", &config)
            .unwrap();
        assert_eq!(channel, "shroud");
        assert!(url.contains("player.twitch.tv/?channel=shroud"));
    }

    #[test]
    fn test_twitch_url() {
        let normalizer = UrlNormalizer::new();
        let config = test_config();

        let (channel, _) = normalizer
            .normalize(
                EmbedProvider::Twitch,
                "https://www.twitch.tv/shroud",
                &config,
            )
            .unwrap();
        assert_eq!(channel, "shroud");
    }

    #[test]
    fn test_twitch_invalid() {
        let normalizer = UrlNormalizer::new();
        let config = test_config();

        assert!(normalizer
            .normalize(EmbedProvider::Twitch, "", &config)
            .is_err());
    }

    #[test]
    fn test_autoplay_config() {
        let normalizer = UrlNormalizer::new();
        let mut config = test_config();

        // Default: autoplay=0
        let (_, url) = normalizer
            .normalize(EmbedProvider::YouTube, "dQw4w9WgXcQ", &config)
            .unwrap();
        assert!(url.contains("autoplay=0"));

        // With autoplay
        config.cef_embeds_autoplay = true;
        let (_, url) = normalizer
            .normalize(EmbedProvider::YouTube, "dQw4w9WgXcQ", &config)
            .unwrap();
        assert!(url.contains("autoplay=1"));
    }
}
