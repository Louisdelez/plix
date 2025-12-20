//! JS↔Rust message bridge for CEF UI (Feature 031)
//!
//! This module provides the bidirectional communication layer between
//! the JavaScript UI running in CEF and the Rust game engine.
//!
//! # Message Flow
//!
//! ```text
//! JS → Rust: window.plix.send(JSON.stringify({id, type, payload}))
//!            → BridgeDispatcher::dispatch()
//!            → handler function
//!            → BridgeResponse
//!            → JSON string back to JS
//!
//! Rust → JS: BridgePush
//!            → serialize_push()
//!            → window.plix.onMessage(JSON string)
//! ```
//!
//! # Example
//!
//! ```ignore
//! let mut dispatcher = BridgeDispatcher::new();
//! dispatcher.set_display_name("PlayerName");
//!
//! // Handle incoming message from JS
//! if let Some(response_json) = dispatcher.dispatch(json_str) {
//!     // Send response_json back to JS
//! }
//! ```

pub mod handlers;
pub mod messages;
pub mod serialize;

use messages::{BridgeError, BridgeResponse, MessageType};
use serialize::{parse_request, serialize_response};
use tracing::{debug, error, warn};

/// Bridge dispatcher - routes messages to handlers
pub struct BridgeDispatcher {
    /// Player display name for handshake response
    display_name: String,

    /// Flag set when Quit is requested
    quit_requested: bool,
}

impl Default for BridgeDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl BridgeDispatcher {
    /// Create a new dispatcher
    pub fn new() -> Self {
        Self {
            display_name: "Player".to_string(),
            quit_requested: false,
        }
    }

    /// Set the player display name (for handshake response)
    pub fn set_display_name(&mut self, name: &str) {
        self.display_name = name.to_string();
    }

    /// Check if quit has been requested
    pub fn quit_requested(&self) -> bool {
        self.quit_requested
    }

    /// Clear the quit flag
    pub fn clear_quit_request(&mut self) {
        self.quit_requested = false;
    }

    /// Dispatch an incoming JSON message and return the response
    ///
    /// Returns `Some(json_string)` with the response, or `None` if no response
    /// should be sent (which should never happen for request messages).
    pub fn dispatch(&mut self, json: &str) -> Option<String> {
        debug!(json = %json, "Bridge message received");

        // Parse the request
        let request = match parse_request(json) {
            Ok(req) => req,
            Err(e) => {
                error!(error = %e.message, "Failed to parse bridge message");
                // Generate error response with empty ID (best effort)
                let response = BridgeResponse::error("", "Error", e);
                return Some(serialize_response(&response));
            }
        };

        // Parse message type
        let msg_type = match MessageType::parse(&request.msg_type) {
            Some(t) => t,
            None => {
                warn!(msg_type = %request.msg_type, "Unknown bridge message type");
                let response = BridgeResponse::error(
                    &request.id,
                    &request.msg_type,
                    BridgeError::unknown_type(&request.msg_type),
                );
                return Some(serialize_response(&response));
            }
        };

        // Route to handler
        let response = match msg_type {
            MessageType::Handshake => {
                handlers::handle_handshake(&request.id, &request.payload, &self.display_name)
            }
            MessageType::Quit => {
                self.quit_requested = true;
                handlers::handle_quit(&request.id)
            }
            // These handlers need game state access - delegate to menus module
            MessageType::GetConfig
            | MessageType::SetConfig
            | MessageType::FetchServers
            | MessageType::ToggleFavorite
            | MessageType::Connect => {
                // Return a placeholder response - actual handling done by caller
                // who has access to game state
                BridgeResponse::success_empty(&request.id, msg_type.as_str())
            }
            // Feature 032: In-game UI message types
            // These are handled by the ingame overlay coordinator, not here
            MessageType::ChatSend
            | MessageType::ChatOpen
            | MessageType::ChatClose
            | MessageType::ChatClear => {
                // Return placeholder - actual handling in ingame module
                BridgeResponse::success_empty(&request.id, msg_type.as_str())
            }
            // Feature 033: Media Embeds
            // These are handled by the embeds module, not here
            MessageType::EmbedOpenPanel
            | MessageType::EmbedClosePanel
            | MessageType::EmbedFocus
            | MessageType::EmbedUnfocus
            | MessageType::EmbedLoad
            | MessageType::EmbedStop => {
                // Return placeholder - actual handling in embeds module
                BridgeResponse::success_empty(&request.id, msg_type.as_str())
            }
        };

        debug!(
            id = %response.id,
            ok = response.ok,
            "Bridge response generated"
        );

        Some(serialize_response(&response))
    }

    /// Create a dispatch context for use with game state
    ///
    /// This returns the parsed message type and ID so the caller can
    /// handle messages that need game state access.
    pub fn parse_message(&self, json: &str) -> Result<ParsedMessage, String> {
        let request = parse_request(json)
            .map_err(|e| serialize_response(&BridgeResponse::error("", "Error", e)))?;

        let msg_type = MessageType::parse(&request.msg_type).ok_or_else(|| {
            serialize_response(&BridgeResponse::error(
                &request.id,
                &request.msg_type,
                BridgeError::unknown_type(&request.msg_type),
            ))
        })?;

        Ok(ParsedMessage {
            id: request.id,
            msg_type,
            payload: request.payload,
        })
    }
}

/// Parsed message for external handling
pub struct ParsedMessage {
    /// Request correlation ID
    pub id: String,

    /// Message type
    pub msg_type: MessageType,

    /// Message payload
    pub payload: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_handshake() {
        let mut dispatcher = BridgeDispatcher::new();
        dispatcher.set_display_name("TestPlayer");

        let json = r#"{"id":"req-1","type":"Handshake","payload":{"bridge_version":"1.0"}}"#;
        let response = dispatcher.dispatch(json).unwrap();

        assert!(response.contains("\"ok\":true"));
        assert!(response.contains("TestPlayer"));
    }

    #[test]
    fn test_dispatcher_quit() {
        let mut dispatcher = BridgeDispatcher::new();

        assert!(!dispatcher.quit_requested());

        let json = r#"{"id":"req-1","type":"Quit","payload":{}}"#;
        let response = dispatcher.dispatch(json).unwrap();

        assert!(response.contains("\"ok\":true"));
        assert!(dispatcher.quit_requested());

        dispatcher.clear_quit_request();
        assert!(!dispatcher.quit_requested());
    }

    #[test]
    fn test_dispatcher_unknown_type() {
        let mut dispatcher = BridgeDispatcher::new();

        let json = r#"{"id":"req-1","type":"UnknownType","payload":{}}"#;
        let response = dispatcher.dispatch(json).unwrap();

        assert!(response.contains("\"ok\":false"));
        assert!(response.contains("EBRG003"));
    }

    #[test]
    fn test_dispatcher_invalid_json() {
        let mut dispatcher = BridgeDispatcher::new();

        let response = dispatcher.dispatch("not json").unwrap();

        assert!(response.contains("\"ok\":false"));
        assert!(response.contains("EBRG002"));
    }

    #[test]
    fn test_parse_message() {
        let dispatcher = BridgeDispatcher::new();

        let json = r#"{"id":"req-1","type":"GetConfig","payload":{}}"#;
        let parsed = dispatcher.parse_message(json).unwrap();

        assert_eq!(parsed.id, "req-1");
        assert_eq!(parsed.msg_type, MessageType::GetConfig);
    }

    #[test]
    fn test_parse_message_invalid() {
        let dispatcher = BridgeDispatcher::new();

        let result = dispatcher.parse_message("not json");
        assert!(result.is_err());
    }
}
