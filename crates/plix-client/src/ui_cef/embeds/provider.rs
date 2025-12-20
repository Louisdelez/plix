//! Embed provider definitions and whitelist (Feature 033)

use crate::ui_cef::CefConfig;

/// Supported media providers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedProvider {
    YouTube,
    Twitch,
    Spotify,
}

impl EmbedProvider {
    /// Parse provider from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "youtube" => Some(Self::YouTube),
            "twitch" => Some(Self::Twitch),
            "spotify" => Some(Self::Spotify),
            _ => None,
        }
    }

    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::YouTube => "youtube",
            Self::Twitch => "twitch",
            Self::Spotify => "spotify",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::YouTube => "YouTube",
            Self::Twitch => "Twitch",
            Self::Spotify => "Spotify",
        }
    }

    /// Check if provider is enabled in config
    pub fn is_enabled(&self, config: &CefConfig) -> bool {
        match self {
            Self::YouTube => config.cef_embeds_youtube,
            Self::Twitch => config.cef_embeds_twitch,
            Self::Spotify => config.cef_embeds_spotify,
        }
    }

    /// Get whitelisted domains for this provider
    pub fn whitelist_domains(&self) -> &'static [&'static str] {
        match self {
            Self::YouTube => &[
                "youtube.com",
                "www.youtube.com",
                "youtu.be",
                "youtube-nocookie.com",
                "www.youtube-nocookie.com",
            ],
            Self::Twitch => &["twitch.tv", "www.twitch.tv", "player.twitch.tv"],
            Self::Spotify => &["open.spotify.com"],
        }
    }

    /// Check if a domain is whitelisted for any provider
    pub fn is_domain_whitelisted(domain: &str) -> bool {
        let domain_lower = domain.to_lowercase();

        for provider in [Self::YouTube, Self::Twitch, Self::Spotify] {
            for allowed in provider.whitelist_domains() {
                if domain_lower == *allowed || domain_lower.ends_with(&format!(".{}", allowed)) {
                    return true;
                }
            }
        }

        false
    }

    /// Get combined whitelist of all provider domains
    pub fn all_whitelist_domains() -> Vec<&'static str> {
        let mut domains = Vec::new();
        for provider in [Self::YouTube, Self::Twitch, Self::Spotify] {
            domains.extend_from_slice(provider.whitelist_domains());
        }
        domains
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            EmbedProvider::parse("youtube"),
            Some(EmbedProvider::YouTube)
        );
        assert_eq!(
            EmbedProvider::parse("YouTube"),
            Some(EmbedProvider::YouTube)
        );
        assert_eq!(
            EmbedProvider::parse("YOUTUBE"),
            Some(EmbedProvider::YouTube)
        );
        assert_eq!(EmbedProvider::parse("twitch"), Some(EmbedProvider::Twitch));
        assert_eq!(
            EmbedProvider::parse("spotify"),
            Some(EmbedProvider::Spotify)
        );
        assert_eq!(EmbedProvider::parse("unknown"), None);
    }

    #[test]
    fn test_as_str() {
        assert_eq!(EmbedProvider::YouTube.as_str(), "youtube");
        assert_eq!(EmbedProvider::Twitch.as_str(), "twitch");
        assert_eq!(EmbedProvider::Spotify.as_str(), "spotify");
    }

    #[test]
    fn test_is_enabled() {
        let config = CefConfig::default();

        assert!(EmbedProvider::YouTube.is_enabled(&config));
        assert!(EmbedProvider::Twitch.is_enabled(&config));
        assert!(!EmbedProvider::Spotify.is_enabled(&config)); // Disabled by default
    }

    #[test]
    fn test_whitelist_domains() {
        let youtube_domains = EmbedProvider::YouTube.whitelist_domains();
        assert!(youtube_domains.contains(&"youtube.com"));
        assert!(youtube_domains.contains(&"youtu.be"));
        assert!(youtube_domains.contains(&"youtube-nocookie.com"));

        let twitch_domains = EmbedProvider::Twitch.whitelist_domains();
        assert!(twitch_domains.contains(&"twitch.tv"));
        assert!(twitch_domains.contains(&"player.twitch.tv"));

        let spotify_domains = EmbedProvider::Spotify.whitelist_domains();
        assert!(spotify_domains.contains(&"open.spotify.com"));
    }

    #[test]
    fn test_is_domain_whitelisted() {
        assert!(EmbedProvider::is_domain_whitelisted("youtube.com"));
        assert!(EmbedProvider::is_domain_whitelisted("www.youtube.com"));
        assert!(EmbedProvider::is_domain_whitelisted("youtube-nocookie.com"));
        assert!(EmbedProvider::is_domain_whitelisted("twitch.tv"));
        assert!(EmbedProvider::is_domain_whitelisted("open.spotify.com"));

        assert!(!EmbedProvider::is_domain_whitelisted("evil.com"));
        assert!(!EmbedProvider::is_domain_whitelisted("youtube.evil.com"));
        assert!(!EmbedProvider::is_domain_whitelisted(
            "malicious-youtube.com"
        ));
    }
}
