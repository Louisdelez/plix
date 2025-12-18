//! Field validation for server registration.
//!
//! Validates heartbeat request fields for size limits and allowed characters.

use crate::types::HeartbeatRequest;

/// Maximum length for server name
pub const MAX_NAME_LENGTH: usize = 64;

/// Maximum length for region
pub const MAX_REGION_LENGTH: usize = 32;

/// Maximum length for a single tag
pub const MAX_TAG_LENGTH: usize = 32;

/// Maximum number of tags
pub const MAX_TAGS: usize = 10;

/// Maximum length for a single game mode
pub const MAX_GAME_MODE_LENGTH: usize = 32;

/// Maximum number of game modes
pub const MAX_GAME_MODES: usize = 10;

/// Maximum length for host
pub const MAX_HOST_LENGTH: usize = 255;

/// Validation error types
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Name validation failed
    InvalidName(String),
    /// Region validation failed
    InvalidRegion(String),
    /// Host validation failed
    InvalidHost(String),
    /// Port validation failed
    InvalidPort(String),
    /// Tags validation failed
    InvalidTags(String),
    /// Game modes validation failed
    InvalidGameModes(String),
    /// Protocol version validation failed
    InvalidProtocolVersion(String),
    /// Player count validation failed
    InvalidPlayerCount(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidName(msg) => write!(f, "name: {}", msg),
            ValidationError::InvalidRegion(msg) => write!(f, "region: {}", msg),
            ValidationError::InvalidHost(msg) => write!(f, "host: {}", msg),
            ValidationError::InvalidPort(msg) => write!(f, "port: {}", msg),
            ValidationError::InvalidTags(msg) => write!(f, "tags: {}", msg),
            ValidationError::InvalidGameModes(msg) => write!(f, "game_modes: {}", msg),
            ValidationError::InvalidProtocolVersion(msg) => write!(f, "protocol_version: {}", msg),
            ValidationError::InvalidPlayerCount(msg) => write!(f, "player_count: {}", msg),
        }
    }
}

/// Check if a string contains only allowed characters.
/// Allowed: alphanumeric, spaces, hyphens, underscores.
fn is_valid_name_chars(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_')
}

/// Check if a string contains only allowed characters for region/tags.
/// Allowed: alphanumeric, hyphens, underscores.
fn is_valid_identifier_chars(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Validate a heartbeat request.
///
/// Returns Ok(()) if all fields are valid, or Err with the first validation error.
pub fn validate_heartbeat(request: &HeartbeatRequest) -> Result<(), ValidationError> {
    // Validate name
    if request.name.is_empty() {
        return Err(ValidationError::InvalidName("cannot be empty".to_string()));
    }
    if request.name.len() > MAX_NAME_LENGTH {
        return Err(ValidationError::InvalidName(format!(
            "exceeds {} characters",
            MAX_NAME_LENGTH
        )));
    }
    if !is_valid_name_chars(&request.name) {
        return Err(ValidationError::InvalidName(
            "contains invalid characters (allowed: alphanumeric, space, hyphen, underscore)"
                .to_string(),
        ));
    }

    // Validate host
    if request.host.is_empty() {
        return Err(ValidationError::InvalidHost("cannot be empty".to_string()));
    }
    if request.host.len() > MAX_HOST_LENGTH {
        return Err(ValidationError::InvalidHost(format!(
            "exceeds {} characters",
            MAX_HOST_LENGTH
        )));
    }

    // Validate port
    if request.port == 0 {
        return Err(ValidationError::InvalidPort(
            "must be between 1 and 65535".to_string(),
        ));
    }

    // Validate region
    if request.region.is_empty() {
        return Err(ValidationError::InvalidRegion(
            "cannot be empty".to_string(),
        ));
    }
    if request.region.len() > MAX_REGION_LENGTH {
        return Err(ValidationError::InvalidRegion(format!(
            "exceeds {} characters",
            MAX_REGION_LENGTH
        )));
    }
    if !is_valid_identifier_chars(&request.region) {
        return Err(ValidationError::InvalidRegion(
            "contains invalid characters (allowed: alphanumeric, hyphen, underscore)".to_string(),
        ));
    }

    // Validate tags
    if request.tags.len() > MAX_TAGS {
        return Err(ValidationError::InvalidTags(format!(
            "exceeds {} tags",
            MAX_TAGS
        )));
    }
    for (i, tag) in request.tags.iter().enumerate() {
        if tag.is_empty() {
            return Err(ValidationError::InvalidTags(format!(
                "tag {} cannot be empty",
                i
            )));
        }
        if tag.len() > MAX_TAG_LENGTH {
            return Err(ValidationError::InvalidTags(format!(
                "tag {} exceeds {} characters",
                i, MAX_TAG_LENGTH
            )));
        }
        if !is_valid_identifier_chars(tag) {
            return Err(ValidationError::InvalidTags(format!(
                "tag {} contains invalid characters",
                i
            )));
        }
    }

    // Validate game modes
    if request.game_modes.len() > MAX_GAME_MODES {
        return Err(ValidationError::InvalidGameModes(format!(
            "exceeds {} game modes",
            MAX_GAME_MODES
        )));
    }
    for (i, mode) in request.game_modes.iter().enumerate() {
        if mode.is_empty() {
            return Err(ValidationError::InvalidGameModes(format!(
                "game mode {} cannot be empty",
                i
            )));
        }
        if mode.len() > MAX_GAME_MODE_LENGTH {
            return Err(ValidationError::InvalidGameModes(format!(
                "game mode {} exceeds {} characters",
                i, MAX_GAME_MODE_LENGTH
            )));
        }
        if !is_valid_identifier_chars(mode) {
            return Err(ValidationError::InvalidGameModes(format!(
                "game mode {} contains invalid characters",
                i
            )));
        }
    }

    // Validate protocol version
    if request.protocol_version.is_empty() {
        return Err(ValidationError::InvalidProtocolVersion(
            "cannot be empty".to_string(),
        ));
    }

    // Validate player counts
    if request.max_players == 0 {
        return Err(ValidationError::InvalidPlayerCount(
            "max_players must be at least 1".to_string(),
        ));
    }
    if request.player_count > request.max_players {
        return Err(ValidationError::InvalidPlayerCount(
            "player_count cannot exceed max_players".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> HeartbeatRequest {
        HeartbeatRequest {
            name: "Test Server".to_string(),
            host: "192.168.1.100".to_string(),
            port: 7777,
            region: "eu-west".to_string(),
            tags: vec!["ctf".to_string(), "competitive".to_string()],
            player_count: 10,
            max_players: 32,
            game_modes: vec!["ctf".to_string()],
            protocol_version: "0.1.0".to_string(),
        }
    }

    #[test]
    fn test_valid_request() {
        let request = valid_request();
        assert!(validate_heartbeat(&request).is_ok());
    }

    #[test]
    fn test_empty_name_rejected() {
        let mut request = valid_request();
        request.name = "".to_string();
        let result = validate_heartbeat(&request);
        assert!(matches!(result, Err(ValidationError::InvalidName(_))));
    }

    #[test]
    fn test_name_too_long_rejected() {
        let mut request = valid_request();
        request.name = "a".repeat(MAX_NAME_LENGTH + 1);
        let result = validate_heartbeat(&request);
        assert!(matches!(result, Err(ValidationError::InvalidName(_))));
        assert!(result.unwrap_err().to_string().contains("exceeds"));
    }

    #[test]
    fn test_name_max_length_accepted() {
        let mut request = valid_request();
        request.name = "a".repeat(MAX_NAME_LENGTH);
        assert!(validate_heartbeat(&request).is_ok());
    }

    #[test]
    fn test_name_invalid_chars_rejected() {
        let mut request = valid_request();
        request.name = "Test<script>".to_string();
        let result = validate_heartbeat(&request);
        assert!(matches!(result, Err(ValidationError::InvalidName(_))));
    }

    #[test]
    fn test_name_allowed_chars() {
        let mut request = valid_request();
        request.name = "My Server-Name_123".to_string();
        assert!(validate_heartbeat(&request).is_ok());
    }

    #[test]
    fn test_region_too_long_rejected() {
        let mut request = valid_request();
        request.region = "a".repeat(MAX_REGION_LENGTH + 1);
        let result = validate_heartbeat(&request);
        assert!(matches!(result, Err(ValidationError::InvalidRegion(_))));
    }

    #[test]
    fn test_region_with_spaces_rejected() {
        let mut request = valid_request();
        request.region = "eu west".to_string();
        let result = validate_heartbeat(&request);
        assert!(matches!(result, Err(ValidationError::InvalidRegion(_))));
    }

    #[test]
    fn test_too_many_tags_rejected() {
        let mut request = valid_request();
        request.tags = (0..=MAX_TAGS).map(|i| format!("tag{}", i)).collect();
        let result = validate_heartbeat(&request);
        assert!(matches!(result, Err(ValidationError::InvalidTags(_))));
    }

    #[test]
    fn test_tag_too_long_rejected() {
        let mut request = valid_request();
        request.tags = vec!["a".repeat(MAX_TAG_LENGTH + 1)];
        let result = validate_heartbeat(&request);
        assert!(matches!(result, Err(ValidationError::InvalidTags(_))));
    }

    #[test]
    fn test_empty_tag_rejected() {
        let mut request = valid_request();
        request.tags = vec!["valid".to_string(), "".to_string()];
        let result = validate_heartbeat(&request);
        assert!(matches!(result, Err(ValidationError::InvalidTags(_))));
    }

    #[test]
    fn test_port_zero_rejected() {
        let mut request = valid_request();
        request.port = 0;
        let result = validate_heartbeat(&request);
        assert!(matches!(result, Err(ValidationError::InvalidPort(_))));
    }

    #[test]
    fn test_player_count_exceeds_max_rejected() {
        let mut request = valid_request();
        request.player_count = 50;
        request.max_players = 32;
        let result = validate_heartbeat(&request);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidPlayerCount(_))
        ));
    }

    #[test]
    fn test_max_players_zero_rejected() {
        let mut request = valid_request();
        request.max_players = 0;
        let result = validate_heartbeat(&request);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidPlayerCount(_))
        ));
    }

    #[test]
    fn test_empty_protocol_version_rejected() {
        let mut request = valid_request();
        request.protocol_version = "".to_string();
        let result = validate_heartbeat(&request);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidProtocolVersion(_))
        ));
    }
}
