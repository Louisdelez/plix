//! Server browser handlers for CEF UI (Feature 031)
//!
//! Handles FetchServers and Connect messages.

use super::favorites::Favorites;
use crate::ui_cef::bridge::messages::{
    BridgeError, BridgeResponse, ConnectionState, ConnectionStatusPayload, PushType,
};
use crate::ui_cef::bridge::serialize::{
    create_push, sanitize_string_plain, validate_address, MAX_SERVER_NAME,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Current protocol version (must match server)
pub const PROTOCOL_VERSION: u32 = 1;

/// Server entry for UI display (matches data-model.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntryUI {
    /// Server address (ip:port or hostname:port)
    pub address: String,

    /// Display name
    pub name: String,

    /// Geographic region
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// Game mode tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Current player count
    pub players: u32,

    /// Maximum player capacity
    pub max_players: u32,

    /// Protocol version number
    pub protocol_version: u32,

    /// Whether server is in favorites
    pub is_favorite: bool,

    /// Whether protocol version matches client
    pub is_compatible: bool,
}

impl ServerEntryUI {
    /// Create from server info with favorites check
    #[allow(clippy::too_many_arguments)]
    pub fn from_server_info(
        address: &str,
        name: &str,
        region: Option<&str>,
        tags: &[String],
        players: u32,
        max_players: u32,
        protocol_version: u32,
        favorites: &Favorites,
    ) -> Self {
        Self {
            address: sanitize_string_plain(address, MAX_SERVER_NAME),
            name: sanitize_string_plain(name, MAX_SERVER_NAME),
            region: region.map(|r| sanitize_string_plain(r, 32)),
            tags: tags
                .iter()
                .take(5)
                .map(|t| sanitize_string_plain(t, 32))
                .collect(),
            players,
            max_players,
            protocol_version,
            is_favorite: favorites.is_favorite(address),
            is_compatible: protocol_version == PROTOCOL_VERSION,
        }
    }
}

/// Handle FetchServers message
///
/// In a real implementation, this would call the server browser API from Feature 026.
/// For now, we return a placeholder response.
pub fn handle_fetch_servers(
    id: &str,
    favorites: &Favorites,
    // In real implementation: server_browser: &ServerBrowser
) -> BridgeResponse {
    // TODO: Call actual server browser API (Feature 026)
    // For now, return empty list or mock data

    // Mock server list for development
    let servers = vec![ServerEntryUI::from_server_info(
        "127.0.0.1:7777",
        "Local Server",
        Some("Local"),
        &["FFA".to_string(), "Vanilla".to_string()],
        0,
        16,
        PROTOCOL_VERSION,
        favorites,
    )];

    BridgeResponse::success(id, "FetchServers", json!({ "servers": servers }))
}

/// Handle Connect message
///
/// Validates the address and initiates connection.
/// Connection progress is reported via ConnectionStatus push events.
pub fn handle_connect(id: &str, payload: &Value) -> Result<ConnectResult, BridgeError> {
    // Extract and validate address
    let address = payload
        .get("address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| BridgeError::invalid_format("Missing 'address' field"))?;

    let address = validate_address(address)?;

    // Return result for caller to handle connection
    Ok(ConnectResult {
        response: BridgeResponse::success_empty(id, "Connect"),
        address,
    })
}

/// Result of handle_connect
pub struct ConnectResult {
    /// Success response to send immediately
    pub response: BridgeResponse,

    /// Validated address to connect to
    pub address: String,
}

/// Create a ConnectionStatus push event
pub fn create_connection_status_push(
    state: ConnectionState,
    address: Option<&str>,
    message: Option<&str>,
) -> String {
    let payload = ConnectionStatusPayload {
        state,
        address: address.map(|a| sanitize_string_plain(a, MAX_SERVER_NAME)),
        message: message.map(|m| sanitize_string_plain(m, 256)),
    };

    create_push(PushType::ConnectionStatus.as_str(), &payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_entry_ui() {
        let favorites = Favorites::default();
        let entry = ServerEntryUI::from_server_info(
            "127.0.0.1:7777",
            "Test Server",
            Some("EU"),
            &["FFA".to_string()],
            5,
            16,
            PROTOCOL_VERSION,
            &favorites,
        );

        assert_eq!(entry.address, "127.0.0.1:7777");
        assert_eq!(entry.name, "Test Server");
        assert_eq!(entry.region, Some("EU".to_string()));
        assert_eq!(entry.players, 5);
        assert!(!entry.is_favorite);
        assert!(entry.is_compatible);
    }

    #[test]
    fn test_server_entry_ui_favorite() {
        let mut favorites = Favorites::default();
        favorites.toggle("127.0.0.1:7777");

        let entry = ServerEntryUI::from_server_info(
            "127.0.0.1:7777",
            "Test Server",
            None,
            &[],
            0,
            16,
            PROTOCOL_VERSION,
            &favorites,
        );

        assert!(entry.is_favorite);
    }

    #[test]
    fn test_server_entry_ui_incompatible() {
        let favorites = Favorites::default();
        let entry = ServerEntryUI::from_server_info(
            "127.0.0.1:7777",
            "Old Server",
            None,
            &[],
            0,
            16,
            PROTOCOL_VERSION + 1, // Different version
            &favorites,
        );

        assert!(!entry.is_compatible);
    }

    #[test]
    fn test_handle_fetch_servers() {
        let favorites = Favorites::default();
        let response = handle_fetch_servers("req-1", &favorites);

        assert!(response.ok);
        let payload = response.payload.unwrap();
        assert!(payload.get("servers").is_some());
    }

    #[test]
    fn test_handle_connect_valid() {
        let payload = json!({"address": "127.0.0.1:7777"});
        let result = handle_connect("req-1", &payload);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.response.ok);
        assert_eq!(result.address, "127.0.0.1:7777");
    }

    #[test]
    fn test_handle_connect_missing_address() {
        let payload = json!({});
        let result = handle_connect("req-1", &payload);

        assert!(result.is_err());
    }

    #[test]
    fn test_handle_connect_invalid_address() {
        let payload = json!({"address": "no-port"});
        let result = handle_connect("req-1", &payload);

        assert!(result.is_err());
    }

    #[test]
    fn test_create_connection_status_push() {
        let json = create_connection_status_push(
            ConnectionState::Connecting,
            Some("127.0.0.1:7777"),
            Some("Connecting..."),
        );

        assert!(json.contains("ConnectionStatus"));
        assert!(json.contains("connecting"));
        assert!(json.contains("127.0.0.1:7777"));
    }
}
