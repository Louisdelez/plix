//! Chat types for in-game communication (Feature 032)
//!
//! This module defines the shared chat types used by both client and server
//! for text communication between players.

use serde::{Deserialize, Serialize};

/// Maximum length of a chat message in characters
pub const MAX_MESSAGE_LENGTH: usize = 200;

/// Maximum length of a player/author name in characters
pub const MAX_AUTHOR_LENGTH: usize = 32;

/// Rate limit for sending chat messages (milliseconds)
pub const CHAT_RATE_LIMIT_MS: u64 = 500;

/// Maximum chat history size (number of messages)
pub const MAX_CHAT_HISTORY: usize = 100;

/// Kind of chat message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatMessageKind {
    /// Normal player chat message
    Player,
    /// System announcement (server message, join/leave, etc.)
    System,
}

impl Default for ChatMessageKind {
    fn default() -> Self {
        Self::Player
    }
}

impl ChatMessageKind {
    /// Get string representation for bridge protocol
    pub fn as_str(&self) -> &'static str {
        match self {
            ChatMessageKind::Player => "player",
            ChatMessageKind::System => "system",
        }
    }

    /// Parse from string (case-insensitive)
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "player" => Some(Self::Player),
            "system" => Some(Self::System),
            _ => None,
        }
    }
}

/// A chat message for protocol transmission
///
/// Used in GameEvent::ChatReceived for server-to-client broadcast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageData {
    /// Sender's display name
    pub author: String,
    /// Message content (max 200 chars)
    pub text: String,
    /// Message type
    pub kind: ChatMessageKind,
    /// Unix timestamp in seconds
    pub timestamp: u64,
}

impl ChatMessageData {
    /// Create a new player message
    pub fn player(author: impl Into<String>, text: impl Into<String>, timestamp: u64) -> Self {
        Self {
            author: author.into(),
            text: text.into(),
            kind: ChatMessageKind::Player,
            timestamp,
        }
    }

    /// Create a new system message
    pub fn system(text: impl Into<String>, timestamp: u64) -> Self {
        Self {
            author: "Server".to_string(),
            text: text.into(),
            kind: ChatMessageKind::System,
            timestamp,
        }
    }

    /// Validate the message content
    ///
    /// Returns error message if validation fails.
    pub fn validate(&self) -> Result<(), &'static str> {
        let trimmed = self.text.trim();

        if trimmed.is_empty() {
            return Err("Message cannot be empty");
        }

        if trimmed.len() > MAX_MESSAGE_LENGTH {
            return Err("Message exceeds maximum length");
        }

        if self.author.is_empty() {
            return Err("Author cannot be empty");
        }

        if self.author.len() > MAX_AUTHOR_LENGTH {
            return Err("Author name exceeds maximum length");
        }

        // Check for control characters (except newline for display)
        if trimmed.chars().any(|c| c.is_control() && c != '\n') {
            return Err("Message contains invalid characters");
        }

        Ok(())
    }
}

/// Validation result for chat input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatValidationError {
    /// Message is empty after trimming
    Empty,
    /// Message exceeds maximum length
    TooLong,
    /// Rate limited (sent too recently)
    RateLimited,
    /// Contains invalid characters
    InvalidCharacters,
}

impl std::fmt::Display for ChatValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatValidationError::Empty => write!(f, "Message cannot be empty"),
            ChatValidationError::TooLong => {
                write!(f, "Message exceeds {} characters", MAX_MESSAGE_LENGTH)
            }
            ChatValidationError::RateLimited => {
                write!(f, "Please wait before sending another message")
            }
            ChatValidationError::InvalidCharacters => {
                write!(f, "Message contains invalid characters")
            }
        }
    }
}

impl std::error::Error for ChatValidationError {}

/// Validate a chat message text before sending
pub fn validate_message_text(text: &str) -> Result<(), ChatValidationError> {
    let trimmed = text.trim();

    if trimmed.is_empty() {
        return Err(ChatValidationError::Empty);
    }

    if trimmed.len() > MAX_MESSAGE_LENGTH {
        return Err(ChatValidationError::TooLong);
    }

    // Check for control characters (except newline)
    if trimmed.chars().any(|c| c.is_control() && c != '\n') {
        return Err(ChatValidationError::InvalidCharacters);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_kind_default() {
        assert_eq!(ChatMessageKind::default(), ChatMessageKind::Player);
    }

    #[test]
    fn test_chat_message_kind_as_str() {
        assert_eq!(ChatMessageKind::Player.as_str(), "player");
        assert_eq!(ChatMessageKind::System.as_str(), "system");
    }

    #[test]
    fn test_chat_message_kind_parse() {
        assert_eq!(
            ChatMessageKind::parse("player"),
            Some(ChatMessageKind::Player)
        );
        assert_eq!(
            ChatMessageKind::parse("Player"),
            Some(ChatMessageKind::Player)
        );
        assert_eq!(
            ChatMessageKind::parse("PLAYER"),
            Some(ChatMessageKind::Player)
        );
        assert_eq!(
            ChatMessageKind::parse("system"),
            Some(ChatMessageKind::System)
        );
        assert_eq!(ChatMessageKind::parse("unknown"), None);
    }

    #[test]
    fn test_chat_message_data_player() {
        let msg = ChatMessageData::player("Alice", "Hello world!", 1234567890);
        assert_eq!(msg.author, "Alice");
        assert_eq!(msg.text, "Hello world!");
        assert_eq!(msg.kind, ChatMessageKind::Player);
        assert_eq!(msg.timestamp, 1234567890);
    }

    #[test]
    fn test_chat_message_data_system() {
        let msg = ChatMessageData::system("Bob has joined", 1234567890);
        assert_eq!(msg.author, "Server");
        assert_eq!(msg.text, "Bob has joined");
        assert_eq!(msg.kind, ChatMessageKind::System);
    }

    #[test]
    fn test_chat_message_data_validate() {
        // Valid message
        let msg = ChatMessageData::player("Alice", "Hello!", 0);
        assert!(msg.validate().is_ok());

        // Empty message
        let msg = ChatMessageData::player("Alice", "", 0);
        assert!(msg.validate().is_err());

        // Whitespace-only message
        let msg = ChatMessageData::player("Alice", "   ", 0);
        assert!(msg.validate().is_err());

        // Empty author
        let msg = ChatMessageData::player("", "Hello!", 0);
        assert!(msg.validate().is_err());

        // Too long message
        let long_text = "a".repeat(MAX_MESSAGE_LENGTH + 1);
        let msg = ChatMessageData::player("Alice", long_text, 0);
        assert!(msg.validate().is_err());

        // Too long author
        let long_author = "a".repeat(MAX_AUTHOR_LENGTH + 1);
        let msg = ChatMessageData::player(long_author, "Hello!", 0);
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_validate_message_text() {
        // Valid
        assert!(validate_message_text("Hello world!").is_ok());
        assert!(validate_message_text("  Hello  ").is_ok());

        // Empty
        assert_eq!(validate_message_text(""), Err(ChatValidationError::Empty));
        assert_eq!(
            validate_message_text("   "),
            Err(ChatValidationError::Empty)
        );

        // Too long
        let long_text = "a".repeat(MAX_MESSAGE_LENGTH + 1);
        assert_eq!(
            validate_message_text(&long_text),
            Err(ChatValidationError::TooLong)
        );

        // Control characters
        assert_eq!(
            validate_message_text("Hello\x00world"),
            Err(ChatValidationError::InvalidCharacters)
        );
    }

    #[test]
    fn test_chat_message_kind_serde() {
        let json = serde_json::to_string(&ChatMessageKind::Player).unwrap();
        assert_eq!(json, "\"player\"");

        let parsed: ChatMessageKind = serde_json::from_str("\"system\"").unwrap();
        assert_eq!(parsed, ChatMessageKind::System);
    }

    #[test]
    fn test_chat_message_data_serde() {
        let msg = ChatMessageData::player("Alice", "Test message", 1234567890);
        let json = serde_json::to_string(&msg).unwrap();

        let parsed: ChatMessageData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.author, "Alice");
        assert_eq!(parsed.text, "Test message");
        assert_eq!(parsed.kind, ChatMessageKind::Player);
        assert_eq!(parsed.timestamp, 1234567890);
    }
}
