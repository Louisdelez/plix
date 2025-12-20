//! In-Game UI Overlay Coordinator (Feature 032)
//!
//! This module coordinates the CEF in-game UI overlays:
//! - HUD (HP, RTT, FPS, crosshair)
//! - Chat (message history, input, toasts)
//! - Scoreboard (player list)
//!
//! All overlays are rendered to the same CEF surface and share a common
//! bridge dispatcher for communication with the JavaScript UI.

pub mod chat;
pub mod hud;
pub mod scoreboard;

pub use chat::ChatClient;
pub use hud::{HudState, HudStatePublisher};
pub use scoreboard::{ScoreboardClient, ScoreboardRow, ScoreboardState};

use crate::ui_cef::bridge::messages::PushType;
use crate::ui_cef::bridge::serialize::create_push;
use crate::ui_cef::CefConfig;
use serde::Serialize;

/// Coordinator for all in-game UI overlays
///
/// Manages HUD, Chat, and Scoreboard clients, routing updates to CEF
/// when appropriate based on configuration and visibility state.
pub struct IngameOverlay {
    /// HUD state publisher (throttled updates)
    pub hud: HudStatePublisher,

    /// Chat client (message history, input state)
    pub chat: ChatClient,

    /// Scoreboard client (player list)
    pub scoreboard: ScoreboardClient,

    /// Configuration reference
    config: IngameConfig,

    /// Queue of push messages to send to UI
    push_queue: Vec<String>,

    /// Whether the UI is ready to receive messages
    ui_ready: bool,
}

/// Configuration subset for in-game overlay
#[derive(Debug, Clone)]
pub struct IngameConfig {
    /// Enable CEF HUD overlay
    pub cef_hud: bool,
    /// Enable CEF chat overlay
    pub cef_chat: bool,
    /// Enable CEF scoreboard overlay
    pub cef_scoreboard: bool,
    /// Enable bridge debug logging
    pub debug_bridge: bool,
}

impl From<&CefConfig> for IngameConfig {
    fn from(config: &CefConfig) -> Self {
        Self {
            cef_hud: config.cef_hud,
            cef_chat: config.cef_chat,
            cef_scoreboard: config.cef_scoreboard,
            debug_bridge: config.debug_bridge,
        }
    }
}

impl Default for IngameConfig {
    fn default() -> Self {
        Self {
            cef_hud: true,
            cef_chat: true,
            cef_scoreboard: true,
            debug_bridge: false,
        }
    }
}

impl IngameOverlay {
    /// Create a new in-game overlay coordinator
    pub fn new(config: IngameConfig) -> Self {
        Self {
            hud: HudStatePublisher::new(),
            chat: ChatClient::new(),
            scoreboard: ScoreboardClient::new(),
            config,
            push_queue: Vec::new(),
            ui_ready: false,
        }
    }

    /// Create from CEF config
    pub fn from_cef_config(config: &CefConfig) -> Self {
        Self::new(IngameConfig::from(config))
    }

    /// Mark UI as ready to receive messages
    pub fn set_ui_ready(&mut self, ready: bool) {
        self.ui_ready = ready;

        // If becoming ready, send initial config
        if ready {
            self.send_ui_config();
        }
    }

    /// Check if UI is ready
    pub fn is_ui_ready(&self) -> bool {
        self.ui_ready
    }

    /// Get configuration
    pub fn config(&self) -> &IngameConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: IngameConfig) {
        self.config = config;
        if self.ui_ready {
            self.send_ui_config();
        }
    }

    /// Send UI configuration to JavaScript
    fn send_ui_config(&mut self) {
        #[derive(Serialize)]
        struct UiConfigPayload {
            #[serde(rename = "cefHudEnabled")]
            cef_hud_enabled: bool,
            #[serde(rename = "cefChatEnabled")]
            cef_chat_enabled: bool,
            #[serde(rename = "cefScoreboardEnabled")]
            cef_scoreboard_enabled: bool,
            #[serde(rename = "cefCrosshairEnabled")]
            cef_crosshair_enabled: bool,
            keybinds: KeybindInfo,
        }

        #[derive(Serialize)]
        struct KeybindInfo {
            #[serde(rename = "chatOpen")]
            chat_open: &'static str,
            scoreboard: &'static str,
        }

        let payload = UiConfigPayload {
            cef_hud_enabled: self.config.cef_hud,
            cef_chat_enabled: self.config.cef_chat,
            cef_scoreboard_enabled: self.config.cef_scoreboard,
            cef_crosshair_enabled: self.config.cef_hud, // Crosshair tied to HUD
            keybinds: KeybindInfo {
                chat_open: "Enter",
                scoreboard: "Tab",
            },
        };

        let json = create_push(PushType::UiConfig.as_str(), &payload);
        self.push_queue.push(json);
    }

    /// Queue a push message for sending to UI
    pub fn queue_push<T: Serialize>(&mut self, push_type: PushType, payload: &T) {
        if !self.ui_ready {
            // Drop messages if UI not ready (could queue with timeout)
            return;
        }

        let json = create_push(push_type.as_str(), payload);

        if self.config.debug_bridge {
            tracing::debug!(push_type = %push_type.as_str(), "Bridge TX: {}", json);
        }

        self.push_queue.push(json);
    }

    /// Drain queued push messages
    ///
    /// Returns an iterator over JSON strings to send to CEF.
    pub fn drain_push_queue(&mut self) -> impl Iterator<Item = String> + '_ {
        self.push_queue.drain(..)
    }

    /// Check if there are queued messages
    pub fn has_queued_messages(&self) -> bool {
        !self.push_queue.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingame_overlay_new() {
        let overlay = IngameOverlay::new(IngameConfig::default());
        assert!(!overlay.is_ui_ready());
        assert!(!overlay.has_queued_messages());
    }

    #[test]
    fn test_ingame_overlay_ui_ready() {
        let mut overlay = IngameOverlay::new(IngameConfig::default());

        overlay.set_ui_ready(true);
        assert!(overlay.is_ui_ready());
        // Should have queued UiConfig message
        assert!(overlay.has_queued_messages());

        let messages: Vec<_> = overlay.drain_push_queue().collect();
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("UiConfig"));
    }

    #[test]
    fn test_ingame_config_from_cef_config() {
        let cef_config = CefConfig {
            cef_hud: false,
            cef_chat: true,
            cef_scoreboard: false,
            debug_bridge: true,
            ..Default::default()
        };

        let ingame_config = IngameConfig::from(&cef_config);
        assert!(!ingame_config.cef_hud);
        assert!(ingame_config.cef_chat);
        assert!(!ingame_config.cef_scoreboard);
        assert!(ingame_config.debug_bridge);
    }

    #[test]
    fn test_queue_push_requires_ui_ready() {
        let mut overlay = IngameOverlay::new(IngameConfig::default());

        // Should not queue when UI not ready
        #[derive(Serialize)]
        struct TestPayload {
            value: i32,
        }
        overlay.queue_push(PushType::HudState, &TestPayload { value: 42 });
        assert!(!overlay.has_queued_messages());

        // Should queue when UI ready
        overlay.set_ui_ready(true);
        overlay.drain_push_queue().count(); // Clear the UiConfig message

        overlay.queue_push(PushType::HudState, &TestPayload { value: 42 });
        assert!(overlay.has_queued_messages());
    }
}
