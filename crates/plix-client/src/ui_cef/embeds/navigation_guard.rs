//! CEF Navigation Guard (Feature 033)
//!
//! Blocks navigation to non-whitelisted domains.

use super::provider::EmbedProvider;
use tracing::warn;

/// Navigation decision returned by the guard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationDecision {
    /// Allow the navigation
    Allow,
    /// Block the navigation
    Block,
}

/// Navigation guard for embed iframes
///
/// Intercepts all navigation attempts and blocks non-whitelisted domains.
pub struct NavigationGuard {
    /// Enable debug logging
    debug_logging: bool,
}

impl NavigationGuard {
    /// Create a new navigation guard
    pub fn new() -> Self {
        Self {
            debug_logging: false,
        }
    }

    /// Enable or disable debug logging
    pub fn set_debug_logging(&mut self, enabled: bool) {
        self.debug_logging = enabled;
    }

    /// Check if a URL should be allowed
    ///
    /// Returns `NavigationDecision::Allow` if the domain is whitelisted,
    /// `NavigationDecision::Block` otherwise.
    pub fn check(&self, url: &str) -> NavigationDecision {
        // Extract domain from URL
        let domain = match self.extract_domain(url) {
            Some(d) => d,
            None => {
                if self.debug_logging {
                    warn!(url = %url, "Failed to extract domain from URL");
                }
                return NavigationDecision::Block;
            }
        };

        // Block file:// URLs
        if url.starts_with("file://") || url.starts_with("file:") {
            if self.debug_logging {
                warn!(url = %url, "Blocked file:// URL");
            }
            return NavigationDecision::Block;
        }

        // Check whitelist
        if EmbedProvider::is_domain_whitelisted(&domain) {
            NavigationDecision::Allow
        } else {
            if self.debug_logging {
                warn!(url = %url, domain = %domain, "Blocked navigation to non-whitelisted domain");
            }
            NavigationDecision::Block
        }
    }

    /// Extract domain from a URL
    fn extract_domain(&self, url: &str) -> Option<String> {
        // Remove protocol
        let url = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .or_else(|| url.strip_prefix("//"))
            .unwrap_or(url);

        // Get host (before first /)
        let host = url.split('/').next()?;

        // Remove port if present
        let domain = host.split(':').next()?;

        if domain.is_empty() {
            None
        } else {
            Some(domain.to_lowercase())
        }
    }

    /// Check if a URL is a popup attempt (window.open)
    ///
    /// Always returns Block for popups.
    pub fn check_popup(&self, url: &str) -> NavigationDecision {
        if self.debug_logging {
            warn!(url = %url, "Blocked popup attempt");
        }
        NavigationDecision::Block
    }

    /// Check if a URL is a download attempt
    ///
    /// Always returns Block for downloads.
    pub fn check_download(&self, url: &str) -> NavigationDecision {
        if self.debug_logging {
            warn!(url = %url, "Blocked download attempt");
        }
        NavigationDecision::Block
    }
}

impl Default for NavigationGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_youtube() {
        let guard = NavigationGuard::new();

        assert_eq!(
            guard.check("https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ"),
            NavigationDecision::Allow
        );
        assert_eq!(
            guard.check("https://youtube.com/watch?v=abc"),
            NavigationDecision::Allow
        );
        assert_eq!(
            guard.check("https://youtu.be/abc"),
            NavigationDecision::Allow
        );
    }

    #[test]
    fn test_allow_twitch() {
        let guard = NavigationGuard::new();

        assert_eq!(
            guard.check("https://player.twitch.tv/?channel=shroud&parent=localhost"),
            NavigationDecision::Allow
        );
        assert_eq!(
            guard.check("https://www.twitch.tv/embed/shroud/chat"),
            NavigationDecision::Allow
        );
    }

    #[test]
    fn test_allow_spotify() {
        let guard = NavigationGuard::new();

        assert_eq!(
            guard.check("https://open.spotify.com/embed/track/abc"),
            NavigationDecision::Allow
        );
    }

    #[test]
    fn test_block_external() {
        let guard = NavigationGuard::new();

        assert_eq!(
            guard.check("https://evil.com/malware"),
            NavigationDecision::Block
        );
        assert_eq!(
            guard.check("https://youtube.evil.com/phishing"),
            NavigationDecision::Block
        );
        assert_eq!(
            guard.check("https://malicious-youtube.com/fake"),
            NavigationDecision::Block
        );
    }

    #[test]
    fn test_block_file_urls() {
        let guard = NavigationGuard::new();

        assert_eq!(guard.check("file:///etc/passwd"), NavigationDecision::Block);
        assert_eq!(
            guard.check("file://C:/Windows/System32/config"),
            NavigationDecision::Block
        );
    }

    #[test]
    fn test_block_popups() {
        let guard = NavigationGuard::new();

        assert_eq!(
            guard.check_popup("https://youtube.com/"),
            NavigationDecision::Block
        );
    }

    #[test]
    fn test_block_downloads() {
        let guard = NavigationGuard::new();

        assert_eq!(
            guard.check_download("https://example.com/file.exe"),
            NavigationDecision::Block
        );
    }

    #[test]
    fn test_extract_domain() {
        let guard = NavigationGuard::new();

        assert_eq!(
            guard.extract_domain("https://www.youtube.com/watch?v=abc"),
            Some("www.youtube.com".to_string())
        );
        assert_eq!(
            guard.extract_domain("http://example.com:8080/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            guard.extract_domain("//cdn.example.com/file.js"),
            Some("cdn.example.com".to_string())
        );
    }
}
