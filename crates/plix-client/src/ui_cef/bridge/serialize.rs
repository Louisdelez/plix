//! JSON serialization utilities for bridge messages (Feature 031)

use super::messages::{BridgeError, BridgePush, BridgeRequest, BridgeResponse};
use serde::Serialize;
use serde_json::Value;

/// Maximum string lengths for sanitization
pub const MAX_SERVER_NAME: usize = 64;
pub const MAX_SEARCH_QUERY: usize = 32;
pub const MAX_DISPLAY_NAME: usize = 32;
pub const MAX_ERROR_MESSAGE: usize = 256;
pub const MAX_CORRELATION_ID: usize = 64;

/// Parse a bridge request from JSON string
///
/// # Errors
///
/// Returns `BridgeError` if:
/// - JSON is malformed (EBRG002)
/// - Required fields are missing (EBRG002)
pub fn parse_request(json: &str) -> Result<BridgeRequest, BridgeError> {
    serde_json::from_str(json).map_err(|e| BridgeError::invalid_format(&e.to_string()))
}

/// Serialize a bridge response to JSON string
pub fn serialize_response(response: &BridgeResponse) -> String {
    // Response serialization should never fail with valid data
    serde_json::to_string(response).unwrap_or_else(|e| {
        // Fallback error response
        format!(
            r#"{{"id":"{}","type":"{}","ok":false,"error":{{"code":"EBRG002","message":"Serialization failed: {}"}}}}"#,
            sanitize_string(&response.id, MAX_CORRELATION_ID),
            sanitize_string(&response.msg_type, 32),
            sanitize_string(&e.to_string(), MAX_ERROR_MESSAGE)
        )
    })
}

/// Serialize a push event to JSON string
pub fn serialize_push(push: &BridgePush) -> String {
    serde_json::to_string(push).unwrap_or_else(|_| {
        // Fallback - should never happen
        r#"{"id":null,"type":"Error","payload":{"message":"Serialization failed"}}"#.to_string()
    })
}

/// Create and serialize a push event
pub fn create_push<T: Serialize>(push_type: &str, payload: &T) -> String {
    let value = serde_json::to_value(payload).unwrap_or(Value::Null);
    let push = BridgePush::new(push_type, value);
    serialize_push(&push)
}

/// Sanitize a string for display
///
/// - Truncates to max length (by chars, not bytes)
/// - Removes control characters
/// - Escapes HTML entities
pub fn sanitize_string(s: &str, max_len: usize) -> String {
    let mut result = String::with_capacity(s.len().min(max_len));
    let mut char_count = 0;

    for c in s.chars() {
        if char_count >= max_len {
            break;
        }

        // Skip control characters (except newline, tab)
        if c.is_control() && c != '\n' && c != '\t' {
            continue;
        }

        // Escape HTML entities
        match c {
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#x27;"),
            _ => result.push(c),
        };

        char_count += 1;
    }

    result
}

/// Sanitize a string without HTML escaping (for JSON values)
pub fn sanitize_string_plain(s: &str, max_len: usize) -> String {
    let mut result = String::with_capacity(s.len().min(max_len));
    let mut char_count = 0;

    for c in s.chars() {
        if char_count >= max_len {
            break;
        }

        // Skip control characters (except newline, tab)
        if c.is_control() && c != '\n' && c != '\t' {
            continue;
        }

        result.push(c);
        char_count += 1;
    }

    result
}

/// Validate and sanitize a server address
pub fn validate_address(addr: &str) -> Result<String, BridgeError> {
    let addr = addr.trim();

    // Basic format validation: host:port
    if addr.is_empty() {
        return Err(BridgeError::validation_failed("Address is required"));
    }

    // Check for port
    if !addr.contains(':') {
        return Err(BridgeError::validation_failed(
            "Address must include port (e.g., 127.0.0.1:7777)",
        ));
    }

    // Limit length
    if addr.len() > MAX_SERVER_NAME {
        return Err(BridgeError::validation_failed("Address too long"));
    }

    Ok(addr.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_request() {
        let json = r#"{"id":"req-1","type":"GetConfig","payload":{}}"#;
        let request = parse_request(json).unwrap();
        assert_eq!(request.id, "req-1");
        assert_eq!(request.msg_type, "GetConfig");
    }

    #[test]
    fn test_parse_request_with_payload() {
        let json = r#"{"id":"req-2","type":"SetConfig","payload":{"sensitivity":0.005}}"#;
        let request = parse_request(json).unwrap();
        assert_eq!(request.payload["sensitivity"], 0.005);
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse_request("not json");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "EBRG002");
    }

    #[test]
    fn test_parse_missing_fields() {
        let result = parse_request(r#"{"id":"req-1"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_response() {
        let response = BridgeResponse::success_empty("req-1", "Quit");
        let json = serialize_response(&response);

        assert!(json.contains("\"id\":\"req-1\""));
        assert!(json.contains("\"ok\":true"));
        assert!(json.contains("\"type\":\"Quit\""));
    }

    #[test]
    fn test_sanitize_string_truncates() {
        let long = "a".repeat(100);
        let result = sanitize_string(&long, 10);
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_sanitize_string_removes_control_chars() {
        let with_control = "hello\x00world\x1f!";
        let result = sanitize_string_plain(with_control, 100);
        assert_eq!(result, "helloworld!");
    }

    #[test]
    fn test_sanitize_string_preserves_newlines() {
        let with_newline = "line1\nline2";
        let result = sanitize_string_plain(with_newline, 100);
        assert_eq!(result, "line1\nline2");
    }

    #[test]
    fn test_sanitize_string_escapes_html() {
        let html = "<script>alert('xss')</script>";
        let result = sanitize_string(html, 100);
        assert!(!result.contains('<'));
        assert!(result.contains("&lt;"));
    }

    #[test]
    fn test_validate_address_valid() {
        assert!(validate_address("127.0.0.1:7777").is_ok());
        assert!(validate_address("game.example.com:7777").is_ok());
    }

    #[test]
    fn test_validate_address_invalid() {
        assert!(validate_address("").is_err());
        assert!(validate_address("127.0.0.1").is_err()); // Missing port
        assert!(validate_address(&"a".repeat(100)).is_err()); // Too long
    }

    #[test]
    fn test_create_push() {
        #[derive(Serialize)]
        struct TestPayload {
            value: i32,
        }

        let json = create_push("TestEvent", &TestPayload { value: 42 });
        assert!(json.contains("\"type\":\"TestEvent\""));
        assert!(json.contains("\"value\":42"));
        assert!(json.contains("\"id\":null"));
    }
}
