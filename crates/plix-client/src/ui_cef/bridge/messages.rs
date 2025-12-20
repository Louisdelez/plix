//! Bridge message types for CEF UI (Feature 031)
//!
//! Defines the message protocol between JavaScript UI and Rust game engine.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Bridge message request from JavaScript
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeRequest {
    /// Correlation ID for request/response matching
    pub id: String,

    /// Message type identifier
    #[serde(rename = "type")]
    pub msg_type: String,

    /// Type-specific payload
    #[serde(default)]
    pub payload: Value,
}

/// Bridge message response to JavaScript
#[derive(Debug, Clone, Serialize)]
pub struct BridgeResponse {
    /// Correlation ID (matches request)
    pub id: String,

    /// Message type identifier
    #[serde(rename = "type")]
    pub msg_type: String,

    /// Success indicator
    pub ok: bool,

    /// Response payload (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,

    /// Error details (on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BridgeError>,
}

impl BridgeResponse {
    /// Create a success response
    pub fn success(id: &str, msg_type: &str, payload: Value) -> Self {
        Self {
            id: id.to_string(),
            msg_type: msg_type.to_string(),
            ok: true,
            payload: Some(payload),
            error: None,
        }
    }

    /// Create a success response with empty payload
    pub fn success_empty(id: &str, msg_type: &str) -> Self {
        Self {
            id: id.to_string(),
            msg_type: msg_type.to_string(),
            ok: true,
            payload: Some(Value::Object(serde_json::Map::new())),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: &str, msg_type: &str, error: BridgeError) -> Self {
        Self {
            id: id.to_string(),
            msg_type: msg_type.to_string(),
            ok: false,
            payload: None,
            error: Some(error),
        }
    }
}

/// Bridge push event (server-initiated, no request ID)
#[derive(Debug, Clone, Serialize)]
pub struct BridgePush {
    /// Always null for push events
    pub id: Option<()>,

    /// Event type identifier
    #[serde(rename = "type")]
    pub push_type: String,

    /// Event payload
    pub payload: Value,
}

impl BridgePush {
    /// Create a push event
    pub fn new(push_type: &str, payload: Value) -> Self {
        Self {
            id: None,
            push_type: push_type.to_string(),
            payload,
        }
    }
}

/// Error details for failed responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeError {
    /// Error code (e.g., "EBRG001")
    pub code: String,

    /// User-friendly error message
    pub message: String,
}

impl BridgeError {
    /// Create a new error
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
        }
    }

    // --- Bridge errors (EBRG) ---

    /// Version mismatch error
    pub fn version_mismatch(expected: &str, got: &str) -> Self {
        Self::new(
            "EBRG001",
            &format!(
                "Bridge version {} not supported. Expected {}",
                got, expected
            ),
        )
    }

    /// Invalid message format
    pub fn invalid_format(details: &str) -> Self {
        Self::new("EBRG002", &format!("Invalid message format: {}", details))
    }

    /// Unknown message type
    pub fn unknown_type(msg_type: &str) -> Self {
        Self::new("EBRG003", &format!("Unknown message type: {}", msg_type))
    }

    // --- Config errors (ECFG) ---

    /// Validation failed
    pub fn validation_failed(details: &str) -> Self {
        Self::new("ECFG001", details)
    }

    /// Save failed
    pub fn save_failed(details: &str) -> Self {
        Self::new("ECFG002", &format!("Failed to save config: {}", details))
    }

    // --- Server errors (ESRV) ---

    /// Master server unreachable
    pub fn master_unreachable() -> Self {
        Self::new(
            "ESRV001",
            "Could not reach server list. Check your connection.",
        )
    }

    /// Empty server list
    pub fn empty_server_list() -> Self {
        Self::new("ESRV002", "No servers available.")
    }

    // --- Connection errors (ECON) ---

    /// Connection timeout
    pub fn connection_timeout() -> Self {
        Self::new("ECON001", "Connection timed out.")
    }

    /// Connection refused
    pub fn connection_refused() -> Self {
        Self::new("ECON002", "Connection refused by server.")
    }

    /// Version incompatible
    pub fn version_incompatible() -> Self {
        Self::new("ECON003", "Server is running an incompatible version.")
    }

    /// Server full
    pub fn server_full() -> Self {
        Self::new("ECON004", "Server is full.")
    }

    // --- Chat errors (ECHAT) - Feature 032 ---

    /// Chat message too long
    pub fn chat_too_long(max_len: usize) -> Self {
        Self::new(
            "ECHAT001",
            &format!("Message exceeds {} characters", max_len),
        )
    }

    /// Chat rate limited
    pub fn chat_rate_limited() -> Self {
        Self::new("ECHAT002", "Please wait before sending another message")
    }

    /// Chat message empty
    pub fn chat_empty() -> Self {
        Self::new("ECHAT003", "Message cannot be empty")
    }

    // --- Embed errors (EEMB) - Feature 033 ---

    /// Invalid URL or video/channel ID
    pub fn embed_invalid_url() -> Self {
        Self::new("EEMB001", "Invalid URL or video ID")
    }

    /// Provider is disabled in config
    pub fn embed_provider_disabled(provider: &str) -> Self {
        Self::new("EEMB002", &format!("{} provider is disabled", provider))
    }

    /// Navigation to non-whitelisted domain blocked
    pub fn embed_blocked_domain() -> Self {
        Self::new("EEMB003", "Navigation to external domain blocked")
    }

    /// Rate limited (action within cooldown period)
    pub fn embed_rate_limited() -> Self {
        Self::new("EEMB004", "Please wait before loading another video")
    }

    /// Embeds feature is disabled
    pub fn embed_disabled() -> Self {
        Self::new("EEMB002", "Embeds feature is disabled")
    }

    // --- Accessibility errors (EACC) - Feature 042 ---

    /// Invalid action name for keybinding
    pub fn invalid_action(action: &str) -> Self {
        Self::new("EACC001", &format!("Invalid action: {}", action))
    }

    /// Invalid key name for keybinding
    pub fn invalid_key(key: &str) -> Self {
        Self::new("EACC002", &format!("Invalid key: {}", key))
    }

    /// Invalid accessibility setting
    pub fn invalid_setting(setting: &str) -> Self {
        Self::new("EACC003", &format!("Invalid accessibility setting: {}", setting))
    }

    /// Invalid setting value
    pub fn invalid_value(details: &str) -> Self {
        Self::new("EACC004", &format!("Invalid value: {}", details))
    }

    // --- Quest errors (EQST) - Feature 043 ---

    /// Quest not found
    pub fn quest_not_found(quest_id: &str) -> Self {
        Self::new("EQST001", &format!("Quest not found: {}", quest_id))
    }

    /// Quest prerequisites not met
    pub fn quest_prerequisites_not_met(details: &str) -> Self {
        Self::new("EQST002", &format!("Prerequisites not met: {}", details))
    }

    /// Quest already active
    pub fn quest_already_active(quest_id: &str) -> Self {
        Self::new("EQST003", &format!("Quest already active: {}", quest_id))
    }

    /// Quest not active (for abandon/pin)
    pub fn quest_not_active(quest_id: &str) -> Self {
        Self::new("EQST004", &format!("Quest not active: {}", quest_id))
    }

    /// Max active quests reached
    pub fn quest_max_active() -> Self {
        Self::new("EQST005", "Maximum active quests reached")
    }
}

/// Message types for request/response
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Handshake - validate bridge version
    Handshake,

    /// Get current game configuration
    GetConfig,

    /// Update game configuration
    SetConfig,

    /// Fetch server list from master
    FetchServers,

    /// Toggle server favorite status
    ToggleFavorite,

    /// Connect to a server
    Connect,

    /// Request clean game exit
    Quit,

    // === Feature 032: In-Game UI ===
    /// Send a chat message (Feature 032)
    ChatSend,

    /// Notify game that chat input opened (Feature 032)
    ChatOpen,

    /// Notify game that chat input closed (Feature 032)
    ChatClose,

    /// Clear local chat history (Feature 032)
    ChatClear,

    // === Feature 033: Media Embeds ===
    /// Open/show the embed panel
    EmbedOpenPanel,

    /// Close/hide the embed panel
    EmbedClosePanel,

    /// Notify game that embed panel received focus
    EmbedFocus,

    /// Notify game that embed panel lost focus
    EmbedUnfocus,

    /// Request to load media content
    EmbedLoad,

    /// Stop/clear the current embed
    EmbedStop,

    // === Feature 042: Accessibility ===
    /// Get current keybindings list
    GetKeybinds,

    /// Start listening for keybind capture
    StartKeybindCapture,

    /// Cancel keybind capture
    CancelKeybindCapture,

    /// Request to rebind an action
    RebindAction,

    /// Request to swap keybindings between two actions
    SwapKeybinds,

    /// Reset all keybindings to defaults
    ResetKeybinds,

    /// Get current accessibility settings
    GetAccessibilitySettings,

    /// Update an accessibility setting
    SetAccessibility,

    // === Feature 043: Quest UI ===
    /// Request full quest sync from server
    QuestSyncRequest,

    /// Accept a quest
    QuestAccept,

    /// Abandon a quest
    QuestAbandon,

    /// Pin/unpin a quest to HUD
    QuestPin,
}

impl MessageType {
    /// Parse message type from string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Handshake" => Some(Self::Handshake),
            "GetConfig" => Some(Self::GetConfig),
            "SetConfig" => Some(Self::SetConfig),
            "FetchServers" => Some(Self::FetchServers),
            "ToggleFavorite" => Some(Self::ToggleFavorite),
            "Connect" => Some(Self::Connect),
            "Quit" => Some(Self::Quit),
            // Feature 032: In-Game UI
            "ChatSend" => Some(Self::ChatSend),
            "ChatOpen" => Some(Self::ChatOpen),
            "ChatClose" => Some(Self::ChatClose),
            "ChatClear" => Some(Self::ChatClear),
            // Feature 033: Media Embeds
            "EmbedOpenPanel" => Some(Self::EmbedOpenPanel),
            "EmbedClosePanel" => Some(Self::EmbedClosePanel),
            "EmbedFocus" => Some(Self::EmbedFocus),
            "EmbedUnfocus" => Some(Self::EmbedUnfocus),
            "EmbedLoad" => Some(Self::EmbedLoad),
            "EmbedStop" => Some(Self::EmbedStop),
            // Feature 042: Accessibility
            "GetKeybinds" => Some(Self::GetKeybinds),
            "StartKeybindCapture" => Some(Self::StartKeybindCapture),
            "CancelKeybindCapture" => Some(Self::CancelKeybindCapture),
            "RebindAction" => Some(Self::RebindAction),
            "SwapKeybinds" => Some(Self::SwapKeybinds),
            "ResetKeybinds" => Some(Self::ResetKeybinds),
            "GetAccessibilitySettings" => Some(Self::GetAccessibilitySettings),
            "SetAccessibility" => Some(Self::SetAccessibility),
            // Feature 043: Quest UI
            "QuestSyncRequest" => Some(Self::QuestSyncRequest),
            "QuestAccept" => Some(Self::QuestAccept),
            "QuestAbandon" => Some(Self::QuestAbandon),
            "QuestPin" => Some(Self::QuestPin),
            _ => None,
        }
    }

    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Handshake => "Handshake",
            Self::GetConfig => "GetConfig",
            Self::SetConfig => "SetConfig",
            Self::FetchServers => "FetchServers",
            Self::ToggleFavorite => "ToggleFavorite",
            Self::Connect => "Connect",
            Self::Quit => "Quit",
            // Feature 032: In-Game UI
            Self::ChatSend => "ChatSend",
            Self::ChatOpen => "ChatOpen",
            Self::ChatClose => "ChatClose",
            Self::ChatClear => "ChatClear",
            // Feature 033: Media Embeds
            Self::EmbedOpenPanel => "EmbedOpenPanel",
            Self::EmbedClosePanel => "EmbedClosePanel",
            Self::EmbedFocus => "EmbedFocus",
            Self::EmbedUnfocus => "EmbedUnfocus",
            Self::EmbedLoad => "EmbedLoad",
            Self::EmbedStop => "EmbedStop",
            // Feature 042: Accessibility
            Self::GetKeybinds => "GetKeybinds",
            Self::StartKeybindCapture => "StartKeybindCapture",
            Self::CancelKeybindCapture => "CancelKeybindCapture",
            Self::RebindAction => "RebindAction",
            Self::SwapKeybinds => "SwapKeybinds",
            Self::ResetKeybinds => "ResetKeybinds",
            Self::GetAccessibilitySettings => "GetAccessibilitySettings",
            Self::SetAccessibility => "SetAccessibility",
            // Feature 043: Quest UI
            Self::QuestSyncRequest => "QuestSyncRequest",
            Self::QuestAccept => "QuestAccept",
            Self::QuestAbandon => "QuestAbandon",
            Self::QuestPin => "QuestPin",
        }
    }
}

/// Push event types (server-initiated)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushType {
    /// Connection status update
    ConnectionStatus,

    /// Favorites list updated
    FavoritesUpdated,

    // === Feature 032: In-Game UI ===
    /// HUD state update (HP, RTT, FPS)
    HudState,

    /// Chat message received
    ChatMessage,

    /// Chat toast notification (when chat is closed)
    ChatToast,

    /// Scoreboard state update
    ScoreboardState,

    /// UI configuration (sent on startup)
    UiConfig,

    // === Feature 033: Media Embeds ===
    /// Embed panel state update
    EmbedState,

    /// Embed error notification
    EmbedError,

    // === Feature 042: Accessibility ===
    /// Current keybindings list
    KeybindsList,

    /// Keybind conflict detected
    KeybindConflict,

    /// Keybind capture timed out
    KeybindCaptureTimeout,

    /// Current accessibility settings
    AccessibilitySettings,

    /// Display a subtitle
    SubtitleShow,

    /// Clear all subtitles
    SubtitleClear,

    // === Feature 043: Quest UI ===
    /// Full quest sync (active + completed quests)
    QuestSync,

    /// Quest progress update (step progress, step advanced, etc.)
    QuestUpdate,

    /// Quest notification (toast/popup for events)
    QuestNotification,

    /// Quest tracker HUD update (pinned quest)
    QuestTrackerUpdate,
}

impl PushType {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConnectionStatus => "ConnectionStatus",
            Self::FavoritesUpdated => "FavoritesUpdated",
            // Feature 032: In-Game UI
            Self::HudState => "HudState",
            Self::ChatMessage => "ChatMessage",
            Self::ChatToast => "ChatToast",
            Self::ScoreboardState => "ScoreboardState",
            Self::UiConfig => "UiConfig",
            // Feature 033: Media Embeds
            Self::EmbedState => "EmbedState",
            Self::EmbedError => "EmbedError",
            // Feature 042: Accessibility
            Self::KeybindsList => "KeybindsList",
            Self::KeybindConflict => "KeybindConflict",
            Self::KeybindCaptureTimeout => "KeybindCaptureTimeout",
            Self::AccessibilitySettings => "AccessibilitySettings",
            Self::SubtitleShow => "SubtitleShow",
            Self::SubtitleClear => "SubtitleClear",
            // Feature 043: Quest UI
            Self::QuestSync => "QuestSync",
            Self::QuestUpdate => "QuestUpdate",
            Self::QuestNotification => "QuestNotification",
            Self::QuestTrackerUpdate => "QuestTrackerUpdate",
        }
    }
}

/// Connection status states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    Connecting,
    Connected,
    Failed,
    Disconnected,
}

/// Connection status payload
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStatusPayload {
    pub state: ConnectionState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Favorites updated payload
#[derive(Debug, Clone, Serialize)]
pub struct FavoritesUpdatedPayload {
    pub favorites: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_roundtrip() {
        for msg_type in [
            MessageType::Handshake,
            MessageType::GetConfig,
            MessageType::SetConfig,
            MessageType::FetchServers,
            MessageType::ToggleFavorite,
            MessageType::Connect,
            MessageType::Quit,
            // Feature 032: In-Game UI
            MessageType::ChatSend,
            MessageType::ChatOpen,
            MessageType::ChatClose,
            MessageType::ChatClear,
            // Feature 033: Media Embeds
            MessageType::EmbedOpenPanel,
            MessageType::EmbedClosePanel,
            MessageType::EmbedFocus,
            MessageType::EmbedUnfocus,
            MessageType::EmbedLoad,
            MessageType::EmbedStop,
            // Feature 042: Accessibility
            MessageType::GetKeybinds,
            MessageType::StartKeybindCapture,
            MessageType::CancelKeybindCapture,
            MessageType::RebindAction,
            MessageType::SwapKeybinds,
            MessageType::ResetKeybinds,
            MessageType::GetAccessibilitySettings,
            MessageType::SetAccessibility,
            // Feature 043: Quest UI
            MessageType::QuestSyncRequest,
            MessageType::QuestAccept,
            MessageType::QuestAbandon,
            MessageType::QuestPin,
        ] {
            let s = msg_type.as_str();
            assert_eq!(MessageType::parse(s), Some(msg_type));
        }
    }

    #[test]
    fn test_unknown_message_type() {
        assert_eq!(MessageType::parse("Unknown"), None);
        assert_eq!(MessageType::parse(""), None);
    }

    #[test]
    fn test_bridge_request_deserialize() {
        let json = r#"{"id":"req-1","type":"Handshake","payload":{"bridge_version":"1.0"}}"#;
        let request: BridgeRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.id, "req-1");
        assert_eq!(request.msg_type, "Handshake");
        assert_eq!(request.payload["bridge_version"], "1.0");
    }

    #[test]
    fn test_bridge_response_success() {
        let response = BridgeResponse::success(
            "req-1",
            "GetConfig",
            serde_json::json!({"sensitivity": 0.003}),
        );

        assert!(response.ok);
        assert!(response.error.is_none());
        assert!(response.payload.is_some());

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"ok\":true"));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_bridge_response_error() {
        let error = BridgeError::validation_failed("Sensitivity out of range");
        let response = BridgeResponse::error("req-1", "SetConfig", error);

        assert!(!response.ok);
        assert!(response.payload.is_none());
        assert!(response.error.is_some());

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"ok\":false"));
        assert!(json.contains("ECFG001"));
    }

    #[test]
    fn test_bridge_push() {
        let push = BridgePush::new(
            "ConnectionStatus",
            serde_json::json!({"state": "connecting", "message": "Connecting..."}),
        );

        let json = serde_json::to_string(&push).unwrap();
        assert!(json.contains("\"id\":null"));
        assert!(json.contains("\"type\":\"ConnectionStatus\""));
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(BridgeError::version_mismatch("1.x", "2.0").code, "EBRG001");
        assert_eq!(BridgeError::invalid_format("bad json").code, "EBRG002");
        assert_eq!(BridgeError::unknown_type("Foo").code, "EBRG003");
        assert_eq!(BridgeError::validation_failed("bad value").code, "ECFG001");
        assert_eq!(BridgeError::save_failed("io error").code, "ECFG002");
        assert_eq!(BridgeError::master_unreachable().code, "ESRV001");
        assert_eq!(BridgeError::empty_server_list().code, "ESRV002");
        assert_eq!(BridgeError::connection_timeout().code, "ECON001");
        assert_eq!(BridgeError::connection_refused().code, "ECON002");
        assert_eq!(BridgeError::version_incompatible().code, "ECON003");
        assert_eq!(BridgeError::server_full().code, "ECON004");
        // Feature 032: Chat errors
        assert_eq!(BridgeError::chat_too_long(200).code, "ECHAT001");
        assert_eq!(BridgeError::chat_rate_limited().code, "ECHAT002");
        assert_eq!(BridgeError::chat_empty().code, "ECHAT003");
        // Feature 033: Embed errors
        assert_eq!(BridgeError::embed_invalid_url().code, "EEMB001");
        assert_eq!(
            BridgeError::embed_provider_disabled("YouTube").code,
            "EEMB002"
        );
        assert_eq!(BridgeError::embed_blocked_domain().code, "EEMB003");
        assert_eq!(BridgeError::embed_rate_limited().code, "EEMB004");
        assert_eq!(BridgeError::embed_disabled().code, "EEMB002");
        // Feature 042: Accessibility errors
        assert_eq!(BridgeError::invalid_action("Unknown").code, "EACC001");
        assert_eq!(BridgeError::invalid_key("BadKey").code, "EACC002");
        assert_eq!(BridgeError::invalid_setting("bad").code, "EACC003");
        assert_eq!(BridgeError::invalid_value("out of range").code, "EACC004");
    }
}
