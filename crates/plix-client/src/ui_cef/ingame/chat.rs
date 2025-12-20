//! Chat Client (Feature 032)
//!
//! Manages client-side chat state including message history,
//! input state, rate limiting, and local commands.

use plix_common::chat::{
    ChatMessageKind, ChatValidationError, CHAT_RATE_LIMIT_MS, MAX_CHAT_HISTORY, MAX_MESSAGE_LENGTH,
};
use serde::Serialize;
use std::collections::VecDeque;
use std::time::Instant;

/// A chat message with local ID for history tracking
#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    /// Unique local message ID
    pub id: u64,
    /// Sender's display name
    pub author: String,
    /// Message content
    pub text: String,
    /// Message type (player or system)
    pub kind: ChatMessageKind,
    /// Unix timestamp in seconds
    pub timestamp: u64,
}

impl ChatMessage {
    /// Create a new player message
    pub fn player(
        id: u64,
        author: impl Into<String>,
        text: impl Into<String>,
        timestamp: u64,
    ) -> Self {
        Self {
            id,
            author: author.into(),
            text: text.into(),
            kind: ChatMessageKind::Player,
            timestamp,
        }
    }

    /// Create a new system message
    pub fn system(id: u64, text: impl Into<String>, timestamp: u64) -> Self {
        Self {
            id,
            author: "Server".to_string(),
            text: text.into(),
            kind: ChatMessageKind::System,
            timestamp,
        }
    }
}

/// Chat message history with fixed capacity
#[derive(Debug)]
pub struct ChatHistory {
    /// Message queue (oldest first)
    messages: VecDeque<ChatMessage>,
    /// Next message ID
    next_id: u64,
}

impl Default for ChatHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatHistory {
    /// Create a new empty history
    pub fn new() -> Self {
        Self {
            messages: VecDeque::with_capacity(MAX_CHAT_HISTORY),
            next_id: 1,
        }
    }

    /// Add a message to history, evicting oldest if at capacity
    pub fn add(&mut self, mut message: ChatMessage) -> u64 {
        let id = self.next_id;
        message.id = id;
        self.next_id += 1;

        // Evict oldest if at capacity
        if self.messages.len() >= MAX_CHAT_HISTORY {
            self.messages.pop_front();
        }

        self.messages.push_back(message);
        id
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get all messages (oldest first)
    pub fn messages(&self) -> impl Iterator<Item = &ChatMessage> {
        self.messages.iter()
    }

    /// Get message count
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

/// Chat client state manager
#[derive(Debug)]
pub struct ChatClient {
    /// Message history
    history: ChatHistory,
    /// Whether chat input is open
    is_open: bool,
    /// Current input text (pending)
    pending_text: String,
    /// Time of last message send (for rate limiting)
    last_send_time: Option<Instant>,
    /// Rate limit interval in milliseconds
    rate_limit_ms: u64,
}

impl Default for ChatClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatClient {
    /// Create a new chat client
    pub fn new() -> Self {
        Self {
            history: ChatHistory::new(),
            is_open: false,
            pending_text: String::new(),
            last_send_time: None,
            rate_limit_ms: CHAT_RATE_LIMIT_MS,
        }
    }

    /// Check if chat input is open
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Open chat input
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Close chat input
    pub fn close(&mut self) {
        self.is_open = false;
        self.pending_text.clear();
    }

    /// Get the pending input text
    pub fn pending_text(&self) -> &str {
        &self.pending_text
    }

    /// Set the pending input text
    pub fn set_pending_text(&mut self, text: impl Into<String>) {
        self.pending_text = text.into();
    }

    /// Validate and prepare a message for sending
    ///
    /// Returns the trimmed text if valid, or an error.
    pub fn validate_send(&self, text: &str) -> Result<String, ChatValidationError> {
        let trimmed = text.trim();

        // Check empty
        if trimmed.is_empty() {
            return Err(ChatValidationError::Empty);
        }

        // Check length
        if trimmed.len() > MAX_MESSAGE_LENGTH {
            return Err(ChatValidationError::TooLong);
        }

        // Check rate limit
        if let Some(last_send) = self.last_send_time {
            let elapsed = last_send.elapsed().as_millis() as u64;
            if elapsed < self.rate_limit_ms {
                return Err(ChatValidationError::RateLimited);
            }
        }

        // Check for control characters
        if trimmed.chars().any(|c| c.is_control() && c != '\n') {
            return Err(ChatValidationError::InvalidCharacters);
        }

        Ok(trimmed.to_string())
    }

    /// Record that a message was sent (for rate limiting)
    pub fn record_send(&mut self) {
        self.last_send_time = Some(Instant::now());
    }

    /// Receive a message and add to history
    ///
    /// Returns true if chat is closed (indicating a toast should be shown).
    pub fn receive(
        &mut self,
        author: impl Into<String>,
        text: impl Into<String>,
        kind: ChatMessageKind,
        timestamp: u64,
    ) -> bool {
        let message = match kind {
            ChatMessageKind::Player => ChatMessage::player(0, author, text, timestamp),
            ChatMessageKind::System => ChatMessage::system(0, text, timestamp),
        };

        self.history.add(message);

        // Return true if chat is closed (should show toast)
        !self.is_open
    }

    /// Clear chat history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Get message history
    pub fn history(&self) -> &ChatHistory {
        &self.history
    }

    /// Check if text is a command (starts with /)
    pub fn is_command(text: &str) -> bool {
        text.trim().starts_with('/')
    }

    /// Parse a command from text
    ///
    /// Returns (command_name, arguments) if text is a command.
    pub fn parse_command(text: &str) -> Option<(&str, &str)> {
        let trimmed = text.trim();
        if !trimmed.starts_with('/') {
            return None;
        }

        let without_slash = &trimmed[1..];
        let mut parts = without_slash.splitn(2, ' ');
        let command = parts.next()?;
        let args = parts.next().unwrap_or("");

        Some((command, args))
    }

    /// Handle a local command
    ///
    /// Returns Some(message) for local commands (/help, /clear),
    /// None if command should be forwarded to server.
    pub fn handle_local_command(&mut self, command: &str, _args: &str) -> Option<ChatMessage> {
        match command.to_lowercase().as_str() {
            "help" => {
                let help_text =
                    "Available commands:\n  /help - Show this help\n  /clear - Clear chat history";
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let msg = ChatMessage::system(0, help_text, timestamp);
                self.history.add(msg.clone());
                Some(msg)
            }
            "clear" => {
                self.clear_history();
                None // No message to display, history was cleared
            }
            _ => None, // Unknown command, forward to server
        }
    }

    /// Set rate limit for testing
    #[cfg(test)]
    pub fn set_rate_limit(&mut self, ms: u64) {
        self.rate_limit_ms = ms;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_chat_message_player() {
        let msg = ChatMessage::player(1, "Alice", "Hello!", 12345);
        assert_eq!(msg.id, 1);
        assert_eq!(msg.author, "Alice");
        assert_eq!(msg.text, "Hello!");
        assert_eq!(msg.kind, ChatMessageKind::Player);
    }

    #[test]
    fn test_chat_message_system() {
        let msg = ChatMessage::system(1, "Player joined", 12345);
        assert_eq!(msg.author, "Server");
        assert_eq!(msg.kind, ChatMessageKind::System);
    }

    #[test]
    fn test_chat_history_add() {
        let mut history = ChatHistory::new();
        let msg = ChatMessage::player(0, "Alice", "Test", 0);

        let id = history.add(msg);
        assert_eq!(id, 1);
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_chat_history_evicts_oldest() {
        let mut history = ChatHistory::new();

        // Add MAX_CHAT_HISTORY + 1 messages
        for i in 0..=MAX_CHAT_HISTORY {
            let msg = ChatMessage::player(0, "User", format!("Message {}", i), 0);
            history.add(msg);
        }

        // Should still be at capacity
        assert_eq!(history.len(), MAX_CHAT_HISTORY);

        // First message should be Message 1 (Message 0 was evicted)
        let first = history.messages().next().unwrap();
        assert_eq!(first.text, "Message 1");
    }

    #[test]
    fn test_chat_history_clear() {
        let mut history = ChatHistory::new();
        history.add(ChatMessage::player(0, "Alice", "Test", 0));
        assert!(!history.is_empty());

        history.clear();
        assert!(history.is_empty());
    }

    #[test]
    fn test_chat_client_open_close() {
        let mut client = ChatClient::new();
        assert!(!client.is_open());

        client.open();
        assert!(client.is_open());

        client.set_pending_text("test");
        assert_eq!(client.pending_text(), "test");

        client.close();
        assert!(!client.is_open());
        assert!(client.pending_text().is_empty()); // Cleared on close
    }

    #[test]
    fn test_chat_client_validate_send() {
        let client = ChatClient::new();

        // Valid
        assert!(client.validate_send("Hello!").is_ok());

        // Empty
        assert_eq!(client.validate_send(""), Err(ChatValidationError::Empty));
        assert_eq!(client.validate_send("   "), Err(ChatValidationError::Empty));

        // Too long
        let long = "a".repeat(MAX_MESSAGE_LENGTH + 1);
        assert_eq!(
            client.validate_send(&long),
            Err(ChatValidationError::TooLong)
        );

        // Control chars
        assert_eq!(
            client.validate_send("hello\x00world"),
            Err(ChatValidationError::InvalidCharacters)
        );
    }

    #[test]
    fn test_chat_client_rate_limit() {
        let mut client = ChatClient::new();
        client.set_rate_limit(100);

        // First message should be allowed
        assert!(client.validate_send("First").is_ok());
        client.record_send();

        // Immediate second should be rate limited
        assert_eq!(
            client.validate_send("Second"),
            Err(ChatValidationError::RateLimited)
        );

        // After delay, should be allowed
        sleep(Duration::from_millis(110));
        assert!(client.validate_send("Third").is_ok());
    }

    #[test]
    fn test_chat_client_receive() {
        let mut client = ChatClient::new();

        // Receive while closed returns true (show toast)
        let show_toast = client.receive("Alice", "Hello!", ChatMessageKind::Player, 0);
        assert!(show_toast);
        assert_eq!(client.history().len(), 1);

        // Receive while open returns false (no toast)
        client.open();
        let show_toast = client.receive("Bob", "Hi!", ChatMessageKind::Player, 0);
        assert!(!show_toast);
        assert_eq!(client.history().len(), 2);
    }

    #[test]
    fn test_is_command() {
        assert!(ChatClient::is_command("/help"));
        assert!(ChatClient::is_command("  /clear  "));
        assert!(!ChatClient::is_command("hello"));
        assert!(!ChatClient::is_command("not /a command"));
    }

    #[test]
    fn test_parse_command() {
        assert_eq!(ChatClient::parse_command("/help"), Some(("help", "")));
        assert_eq!(
            ChatClient::parse_command("/say hello world"),
            Some(("say", "hello world"))
        );
        assert_eq!(ChatClient::parse_command("  /clear  "), Some(("clear", "")));
        assert_eq!(ChatClient::parse_command("not a command"), None);
    }

    #[test]
    fn test_handle_local_command_help() {
        let mut client = ChatClient::new();
        let result = client.handle_local_command("help", "");

        assert!(result.is_some());
        let msg = result.unwrap();
        assert_eq!(msg.kind, ChatMessageKind::System);
        assert!(msg.text.contains("Available commands"));
    }

    #[test]
    fn test_handle_local_command_clear() {
        let mut client = ChatClient::new();
        client.receive("Alice", "Test", ChatMessageKind::Player, 0);
        assert!(!client.history().is_empty());

        let result = client.handle_local_command("clear", "");
        assert!(result.is_none()); // No message returned
        assert!(client.history().is_empty()); // But history was cleared
    }

    #[test]
    fn test_handle_local_command_unknown() {
        let mut client = ChatClient::new();
        let result = client.handle_local_command("unknown", "args");
        assert!(result.is_none()); // Should be forwarded to server
    }
}
