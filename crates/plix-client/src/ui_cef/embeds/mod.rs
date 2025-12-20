//! Media Embeds Manager (Feature 033)
//!
//! Provides YouTube and Twitch embed support in a CEF panel overlay.
//!
//! # Architecture
//!
//! ```text
//! F8 Key
//!    │
//!    ▼
//! EmbedsManager ──► EmbedPanel (visible, focused)
//!    │                    │
//!    │                    ▼
//!    │              EmbedSlot (provider, content_id, embed_url)
//!    │                    │
//!    ▼                    ▼
//! InputFocus ◄───── CEF Iframe (YouTube/Twitch player)
//!    │
//!    ▼
//! Block gameplay input if EmbedFocus
//! ```

mod config;
mod navigation_guard;
mod normalizer;
mod provider;

pub use config::EmbedConfig;
pub use navigation_guard::{NavigationDecision, NavigationGuard};
pub use normalizer::UrlNormalizer;
pub use provider::EmbedProvider;

use crate::ui_cef::bridge::messages::BridgeError;
use crate::ui_cef::bridge::serialize::{EmbedLoadResponsePayload, EmbedStatePayload};
use crate::ui_cef::CefConfig;
use std::time::{Duration, Instant};

/// Rate limit cooldown period
const RATE_LIMIT_COOLDOWN: Duration = Duration::from_secs(2);

/// Embed slot state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SlotState {
    /// No content loaded
    #[default]
    Empty,
    /// Content being loaded
    Loading,
    /// Content active and playing
    Playing,
    /// Load failed
    Error,
}

impl SlotState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Loading => "loading",
            Self::Playing => "playing",
            Self::Error => "error",
        }
    }
}

/// Embed slot - represents a single active embed instance
#[derive(Debug, Clone, Default)]
pub struct EmbedSlot {
    /// Current provider
    pub provider: Option<EmbedProvider>,
    /// Content ID (video ID, channel name)
    pub content_id: Option<String>,
    /// Canonical embed URL
    pub embed_url: Option<String>,
    /// Twitch chat URL (optional)
    pub chat_url: Option<String>,
    /// Slot state
    pub state: SlotState,
}

impl EmbedSlot {
    /// Clear the slot
    pub fn clear(&mut self) {
        self.provider = None;
        self.content_id = None;
        self.embed_url = None;
        self.chat_url = None;
        self.state = SlotState::Empty;
    }
}

/// Embed panel state
#[derive(Debug, Clone, Default)]
pub struct EmbedPanel {
    /// Panel visibility
    pub visible: bool,
    /// Panel has input focus
    pub focused: bool,
    /// Current embed slot
    pub slot: EmbedSlot,
}

/// Embeds manager - coordinates embed panel and slot state
pub struct EmbedsManager {
    /// Panel state
    panel: EmbedPanel,
    /// Last load action timestamp (for rate limiting)
    last_load_at: Option<Instant>,
    /// URL normalizer
    normalizer: UrlNormalizer,
    /// State changed flag (triggers EmbedState push)
    state_dirty: bool,
}

impl Default for EmbedsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbedsManager {
    /// Create a new embeds manager
    pub fn new() -> Self {
        Self {
            panel: EmbedPanel::default(),
            last_load_at: None,
            normalizer: UrlNormalizer::new(),
            state_dirty: false,
        }
    }

    /// Check if panel is visible
    pub fn is_visible(&self) -> bool {
        self.panel.visible
    }

    /// Check if panel has focus
    pub fn is_focused(&self) -> bool {
        self.panel.focused
    }

    /// Get current provider
    pub fn provider(&self) -> Option<EmbedProvider> {
        self.panel.slot.provider
    }

    /// Get current embed URL
    pub fn embed_url(&self) -> Option<&str> {
        self.panel.slot.embed_url.as_deref()
    }

    /// Check if state changed and needs push
    pub fn take_state_dirty(&mut self) -> bool {
        let dirty = self.state_dirty;
        self.state_dirty = false;
        dirty
    }

    /// Open/show the embed panel
    ///
    /// # Errors
    ///
    /// Returns `BridgeError` if embeds feature is disabled.
    pub fn open_panel(&mut self, config: &CefConfig) -> Result<(), BridgeError> {
        if !config.cef_embeds {
            return Err(BridgeError::embed_disabled());
        }

        if !self.panel.visible {
            self.panel.visible = true;
            self.state_dirty = true;
        }

        Ok(())
    }

    /// Close/hide the embed panel
    pub fn close_panel(&mut self) {
        if self.panel.visible {
            self.panel.visible = false;
            self.panel.focused = false;
            self.state_dirty = true;
        }
    }

    /// Toggle panel visibility
    pub fn toggle_panel(&mut self, config: &CefConfig) -> Result<(), BridgeError> {
        if self.panel.visible {
            self.close_panel();
            Ok(())
        } else {
            self.open_panel(config)
        }
    }

    /// Give focus to the embed panel
    pub fn focus(&mut self) {
        if !self.panel.focused {
            self.panel.focused = true;
            self.state_dirty = true;
        }
    }

    /// Release focus from the embed panel
    pub fn unfocus(&mut self) {
        if self.panel.focused {
            self.panel.focused = false;
            self.state_dirty = true;
        }
    }

    /// Load content from URL or ID
    ///
    /// # Errors
    ///
    /// Returns `BridgeError` if:
    /// - Rate limited (EEMB004)
    /// - Invalid URL (EEMB001)
    /// - Provider disabled (EEMB002)
    pub fn load(
        &mut self,
        provider_name: &str,
        url_or_id: &str,
        config: &CefConfig,
    ) -> Result<EmbedLoadResponsePayload, BridgeError> {
        // Check rate limit
        if let Some(last) = self.last_load_at {
            if last.elapsed() < RATE_LIMIT_COOLDOWN {
                return Err(BridgeError::embed_rate_limited());
            }
        }

        // Parse provider
        let provider =
            EmbedProvider::parse(provider_name).ok_or_else(BridgeError::embed_invalid_url)?;

        // Check if provider is enabled
        if !provider.is_enabled(config) {
            return Err(BridgeError::embed_provider_disabled(
                provider.display_name(),
            ));
        }

        // Normalize URL
        let (content_id, embed_url) = self.normalizer.normalize(provider, url_or_id, config)?;

        // Build chat URL for Twitch if enabled
        let chat_url = if provider == EmbedProvider::Twitch && config.cef_embeds_chat {
            Some(format!(
                "https://www.twitch.tv/embed/{}/chat?parent={}",
                content_id, config.cef_embeds_twitch_parent
            ))
        } else {
            None
        };

        // Update slot
        self.panel.slot.provider = Some(provider);
        self.panel.slot.content_id = Some(content_id);
        self.panel.slot.embed_url = Some(embed_url.clone());
        self.panel.slot.chat_url = chat_url;
        self.panel.slot.state = SlotState::Playing;

        // Update rate limit
        self.last_load_at = Some(Instant::now());
        self.state_dirty = true;

        // Auto-open panel if hidden
        if !self.panel.visible {
            self.panel.visible = true;
        }

        Ok(EmbedLoadResponsePayload { embed_url })
    }

    /// Stop/clear the current embed
    pub fn stop(&mut self) {
        if self.panel.slot.state != SlotState::Empty {
            self.panel.slot.clear();
            self.state_dirty = true;
        }
    }

    /// Get current state payload for push
    pub fn state_payload(&self) -> EmbedStatePayload {
        EmbedStatePayload {
            visible: self.panel.visible,
            focused: self.panel.focused,
            provider: self.panel.slot.provider.map(|p| p.as_str().to_string()),
            embed_url: self.panel.slot.embed_url.clone(),
            chat_url: self.panel.slot.chat_url.clone(),
            state: self.panel.slot.state.as_str().to_string(),
        }
    }

    /// Force state dirty (for resync after UI reload)
    pub fn force_state_dirty(&mut self) {
        self.state_dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> CefConfig {
        CefConfig::default()
    }

    #[test]
    fn test_new_manager() {
        let manager = EmbedsManager::new();
        assert!(!manager.is_visible());
        assert!(!manager.is_focused());
        assert!(manager.provider().is_none());
    }

    #[test]
    fn test_open_close_panel() {
        let mut manager = EmbedsManager::new();
        let config = test_config();

        // Open
        assert!(manager.open_panel(&config).is_ok());
        assert!(manager.is_visible());
        assert!(manager.take_state_dirty());

        // Close
        manager.close_panel();
        assert!(!manager.is_visible());
        assert!(manager.take_state_dirty());
    }

    #[test]
    fn test_open_panel_disabled() {
        let mut manager = EmbedsManager::new();
        let mut config = test_config();
        config.cef_embeds = false;

        let result = manager.open_panel(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "EEMB002");
    }

    #[test]
    fn test_focus_unfocus() {
        let mut manager = EmbedsManager::new();

        manager.focus();
        assert!(manager.is_focused());
        assert!(manager.take_state_dirty());

        manager.unfocus();
        assert!(!manager.is_focused());
        assert!(manager.take_state_dirty());
    }

    #[test]
    fn test_stop() {
        let mut manager = EmbedsManager::new();
        let config = test_config();

        // Load something first
        let _ = manager.load("youtube", "dQw4w9WgXcQ", &config);
        assert!(manager.provider().is_some());

        // Stop
        manager.stop();
        assert!(manager.provider().is_none());
        assert!(manager.take_state_dirty());
    }

    #[test]
    fn test_rate_limiting() {
        let mut manager = EmbedsManager::new();
        let config = test_config();

        // First load should succeed
        let result1 = manager.load("youtube", "dQw4w9WgXcQ", &config);
        assert!(result1.is_ok());

        // Immediate second load should fail
        let result2 = manager.load("youtube", "abc123", &config);
        assert!(result2.is_err());
        assert_eq!(result2.unwrap_err().code, "EEMB004");
    }

    #[test]
    fn test_provider_disabled() {
        let mut manager = EmbedsManager::new();
        let mut config = test_config();
        config.cef_embeds_spotify = false;

        let result = manager.load("spotify", "track:123", &config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "EEMB002");
    }

    #[test]
    fn test_state_payload() {
        let mut manager = EmbedsManager::new();
        let config = test_config();

        manager.open_panel(&config).unwrap();
        manager.focus();

        let payload = manager.state_payload();
        assert!(payload.visible);
        assert!(payload.focused);
        assert_eq!(payload.state, "empty");
    }
}
