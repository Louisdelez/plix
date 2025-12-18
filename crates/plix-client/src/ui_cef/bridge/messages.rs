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
}

impl PushType {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConnectionStatus => "ConnectionStatus",
            Self::FavoritesUpdated => "FavoritesUpdated",
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
    }
}
