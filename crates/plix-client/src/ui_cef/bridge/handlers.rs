//! Bridge message handlers (Feature 031)
//!
//! Implements handlers for each bridge message type.

use super::messages::{BridgeError, BridgeResponse};
use super::serialize::sanitize_string_plain;
use serde_json::{json, Value};

/// Supported bridge version (major.minor)
pub const BRIDGE_VERSION: &str = "1.0";

/// Handle Handshake message
///
/// Validates bridge version compatibility and returns player info.
pub fn handle_handshake(id: &str, payload: &Value, display_name: &str) -> BridgeResponse {
    // Extract bridge_version from payload
    let client_version = payload
        .get("bridge_version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Check version compatibility (accept 1.x)
    if !client_version.starts_with("1.") {
        return BridgeResponse::error(
            id,
            "Handshake",
            BridgeError::version_mismatch("1.x", client_version),
        );
    }

    // Sanitize display name
    let safe_name = sanitize_string_plain(display_name, 32);

    BridgeResponse::success(
        id,
        "Handshake",
        json!({
            "supported_version": BRIDGE_VERSION,
            "display_name": safe_name
        }),
    )
}

/// Handle Quit message
///
/// Returns success immediately. The game loop should check for quit flag.
pub fn handle_quit(id: &str) -> BridgeResponse {
    BridgeResponse::success_empty(id, "Quit")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_valid_version() {
        let payload = json!({"bridge_version": "1.0"});
        let response = handle_handshake("req-1", &payload, "TestPlayer");

        assert!(response.ok);
        let payload = response.payload.unwrap();
        assert_eq!(payload["supported_version"], "1.0");
        assert_eq!(payload["display_name"], "TestPlayer");
    }

    #[test]
    fn test_handshake_compatible_version() {
        let payload = json!({"bridge_version": "1.5"});
        let response = handle_handshake("req-1", &payload, "TestPlayer");

        assert!(response.ok); // 1.x should be compatible
    }

    #[test]
    fn test_handshake_incompatible_version() {
        let payload = json!({"bridge_version": "2.0"});
        let response = handle_handshake("req-1", &payload, "TestPlayer");

        assert!(!response.ok);
        let error = response.error.unwrap();
        assert_eq!(error.code, "EBRG001");
    }

    #[test]
    fn test_handshake_missing_version() {
        let payload = json!({});
        let response = handle_handshake("req-1", &payload, "TestPlayer");

        assert!(!response.ok); // "unknown" doesn't match 1.x
    }

    #[test]
    fn test_handshake_sanitizes_display_name() {
        let payload = json!({"bridge_version": "1.0"});
        let long_name = "a".repeat(100);
        let response = handle_handshake("req-1", &payload, &long_name);

        assert!(response.ok);
        let payload = response.payload.unwrap();
        let display_name = payload["display_name"].as_str().unwrap();
        assert!(display_name.len() <= 32);
    }

    #[test]
    fn test_quit() {
        let response = handle_quit("req-1");

        assert!(response.ok);
        assert_eq!(response.msg_type, "Quit");
    }
}
