//! Network API: mod-to-mod messaging with rate limiting

use crate::capabilities::{require, Capability};
use crate::errors::{err_invalid, err_rate, ModApiError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Maximum payload size (8KB)
pub const MAX_PAYLOAD_SIZE: usize = 8 * 1024;

/// Rate limit: messages per second
pub const RATE_LIMIT_MSG_PER_SEC: u32 = 20;

/// Message target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageTarget {
    /// Send to server
    Server,
    /// Send to specific client
    Client(u64),
    /// Broadcast to all clients
    AllClients,
    /// Send to all clients on a team
    Team(u8),
}

/// Channel direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelDirection {
    /// Server can send to clients
    ServerToClient,
    /// Clients can send to server
    ClientToServer,
    /// Both directions allowed
    Bidirectional,
}

/// Mod network channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModChannel {
    /// Owning mod ID
    pub mod_id: String,
    /// Channel name
    pub name: String,
    /// Allowed direction
    pub direction: ChannelDirection,
}

impl ModChannel {
    /// Create a new channel
    pub fn new(
        mod_id: impl Into<String>,
        name: impl Into<String>,
        direction: ChannelDirection,
    ) -> Self {
        Self {
            mod_id: mod_id.into(),
            name: name.into(),
            direction,
        }
    }

    /// Get the full channel identifier (mod:id:name format)
    pub fn full_name(&self) -> String {
        format!("mod:{}:{}", self.mod_id, self.name)
    }

    /// Parse a channel name in mod:id:name format
    pub fn parse(channel_str: &str) -> Result<Self, ModApiError> {
        let parts: Vec<&str> = channel_str.split(':').collect();
        if parts.len() != 3 {
            return Err(err_invalid(format!(
                "Invalid channel format '{}'. Expected 'mod:<mod_id>:<name>'",
                channel_str
            )));
        }

        if parts[0] != "mod" {
            return Err(err_invalid(format!(
                "Channel must start with 'mod:'. Got '{}'",
                channel_str
            )));
        }

        let mod_id = parts[1];
        let name = parts[2];

        if mod_id.is_empty() {
            return Err(err_invalid("Channel mod_id cannot be empty"));
        }

        if name.is_empty() {
            return Err(err_invalid("Channel name cannot be empty"));
        }

        Ok(Self {
            mod_id: mod_id.to_string(),
            name: name.to_string(),
            direction: ChannelDirection::Bidirectional, // Default
        })
    }
}

/// Token bucket rate limiter
#[derive(Debug)]
pub struct TokenBucket {
    /// Maximum tokens (burst capacity)
    capacity: u32,
    /// Current available tokens
    tokens: f32,
    /// Tokens added per second
    refill_rate: f32,
    /// Last refill time
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    pub fn new(capacity: u32, refill_rate: f32) -> Self {
        Self {
            capacity,
            tokens: capacity as f32,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Create a bucket with the default rate limit (20 msg/s)
    pub fn default_rate_limit() -> Self {
        Self::new(RATE_LIMIT_MSG_PER_SEC, RATE_LIMIT_MSG_PER_SEC as f32)
    }

    /// Try to consume a token, returns true if allowed
    pub fn try_consume(&mut self) -> bool {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let tokens_to_add = elapsed.as_secs_f32() * self.refill_rate;

        self.tokens = (self.tokens + tokens_to_add).min(self.capacity as f32);
        self.last_refill = now;
    }

    /// Get current token count
    pub fn available_tokens(&mut self) -> f32 {
        self.refill();
        self.tokens
    }

    /// Reset the bucket to full
    pub fn reset(&mut self) {
        self.tokens = self.capacity as f32;
        self.last_refill = Instant::now();
    }
}

impl Default for TokenBucket {
    fn default() -> Self {
        Self::default_rate_limit()
    }
}

/// Trait for network messaging
///
/// This trait is implemented by the game engine to provide actual message sending.
pub trait NetApi {
    /// Send a message on a mod channel
    ///
    /// # Capability Required
    /// - `net.send` (NET_SEND)
    ///
    /// # Parameters
    /// - `channel`: Channel name (format: `mod:<mod_id>:<name>`)
    /// - `target`: Message target
    /// - `payload`: Message bytes (max 8KB)
    ///
    /// # Errors
    /// - EMOD002: Missing capability
    /// - EMOD001: Payload exceeds 8KB
    /// - EMOD001: Invalid channel format
    /// - EMOD005: Rate limited (>20 msg/s)
    fn send_message(
        &mut self,
        channel: &str,
        target: MessageTarget,
        payload: &[u8],
    ) -> Result<(), ModApiError>;
}

/// Wrapper that adds capability and rate limit checks
pub struct CapabilityCheckedNet<'a, N: NetApi> {
    inner: &'a mut N,
    capabilities: Capability,
    rate_limiter: &'a mut TokenBucket,
}

impl<'a, N: NetApi> CapabilityCheckedNet<'a, N> {
    /// Create a new capability-checked wrapper
    pub fn new(
        inner: &'a mut N,
        capabilities: Capability,
        rate_limiter: &'a mut TokenBucket,
    ) -> Self {
        Self {
            inner,
            capabilities,
            rate_limiter,
        }
    }

    /// Send message with all checks
    pub fn send_message(
        &mut self,
        channel: &str,
        target: MessageTarget,
        payload: &[u8],
    ) -> Result<(), ModApiError> {
        // Check capability
        require(self.capabilities, Capability::NET_SEND)?;

        // Validate channel format
        let _parsed = ModChannel::parse(channel)?;

        // Check payload size
        if payload.len() > MAX_PAYLOAD_SIZE {
            return Err(err_invalid(format!(
                "Payload size {} exceeds maximum of {} bytes",
                payload.len(),
                MAX_PAYLOAD_SIZE
            )));
        }

        // Check rate limit
        if !self.rate_limiter.try_consume() {
            return Err(err_rate(format!(
                "Rate limited: exceeded {} messages per second",
                RATE_LIMIT_MSG_PER_SEC
            )));
        }

        self.inner.send_message(channel, target, payload)
    }
}

/// Per-mod rate limiter storage
#[derive(Debug, Default)]
pub struct ModRateLimiters {
    limiters: HashMap<String, TokenBucket>,
}

impl ModRateLimiters {
    /// Create a new rate limiter storage
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create a rate limiter for a mod
    pub fn get_or_create(&mut self, mod_id: &str) -> &mut TokenBucket {
        self.limiters
            .entry(mod_id.to_string())
            .or_insert_with(TokenBucket::default_rate_limit)
    }

    /// Remove a mod's rate limiter
    pub fn remove(&mut self, mod_id: &str) {
        self.limiters.remove(mod_id);
    }

    /// Reset all rate limiters
    pub fn reset_all(&mut self) {
        for limiter in self.limiters.values_mut() {
            limiter.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_channel_parse_valid() {
        let channel = ModChannel::parse("mod:my-mod:chat").unwrap();
        assert_eq!(channel.mod_id, "my-mod");
        assert_eq!(channel.name, "chat");
    }

    #[test]
    fn test_channel_parse_invalid_format() {
        assert!(ModChannel::parse("invalid").is_err());
        assert!(ModChannel::parse("mod:only").is_err());
        assert!(ModChannel::parse("mod:a:b:c:d").is_err());
    }

    #[test]
    fn test_channel_parse_invalid_prefix() {
        let result = ModChannel::parse("notmod:foo:bar");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("start with 'mod:'"));
    }

    #[test]
    fn test_channel_parse_empty_parts() {
        assert!(ModChannel::parse("mod::name").is_err());
        assert!(ModChannel::parse("mod:id:").is_err());
    }

    #[test]
    fn test_channel_full_name() {
        let channel = ModChannel::new("test-mod", "events", ChannelDirection::Bidirectional);
        assert_eq!(channel.full_name(), "mod:test-mod:events");
    }

    #[test]
    fn test_token_bucket_basic() {
        let mut bucket = TokenBucket::new(5, 5.0);

        // Should be able to consume 5 tokens
        for _ in 0..5 {
            assert!(bucket.try_consume());
        }

        // 6th should fail
        assert!(!bucket.try_consume());
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(5, 1000.0); // Very fast refill

        // Consume all tokens
        for _ in 0..5 {
            bucket.try_consume();
        }

        // Wait a tiny bit for refill
        std::thread::sleep(Duration::from_millis(10));

        // Should have some tokens back
        assert!(bucket.available_tokens() > 0.0);
    }

    #[test]
    fn test_mod_rate_limiters() {
        let mut limiters = ModRateLimiters::new();

        // First access creates limiter
        let limiter1 = limiters.get_or_create("mod1");
        assert!(limiter1.try_consume());

        // Second access returns same limiter
        let limiter2 = limiters.get_or_create("mod1");
        // Already consumed one token
        assert!(limiter2.available_tokens() < 20.0);

        // Different mod gets different limiter
        let limiter3 = limiters.get_or_create("mod2");
        assert!(limiter3.available_tokens() >= 19.0);
    }

    // Mock NetApi for testing
    struct MockNet {
        messages_sent: Vec<(String, MessageTarget, Vec<u8>)>,
    }

    impl Default for MockNet {
        fn default() -> Self {
            Self {
                messages_sent: Vec::new(),
            }
        }
    }

    impl NetApi for MockNet {
        fn send_message(
            &mut self,
            channel: &str,
            target: MessageTarget,
            payload: &[u8],
        ) -> Result<(), ModApiError> {
            self.messages_sent
                .push((channel.to_string(), target, payload.to_vec()));
            Ok(())
        }
    }

    #[test]
    fn test_capability_checked_send() {
        let mut net = MockNet::default();
        let mut rate_limiter = TokenBucket::default_rate_limit();
        let caps = Capability::NET_SEND;

        let mut checked = CapabilityCheckedNet::new(&mut net, caps, &mut rate_limiter);

        let result = checked.send_message("mod:test:channel", MessageTarget::Server, b"hello");
        assert!(result.is_ok());
        assert_eq!(net.messages_sent.len(), 1);
    }

    #[test]
    fn test_capability_checked_without_permission() {
        let mut net = MockNet::default();
        let mut rate_limiter = TokenBucket::default_rate_limit();
        let caps = Capability::empty();

        let mut checked = CapabilityCheckedNet::new(&mut net, caps, &mut rate_limiter);

        let result = checked.send_message("mod:test:channel", MessageTarget::Server, b"hello");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::PermissionDenied
        );
    }

    #[test]
    fn test_payload_size_limit() {
        let mut net = MockNet::default();
        let mut rate_limiter = TokenBucket::default_rate_limit();
        let caps = Capability::NET_SEND;

        let mut checked = CapabilityCheckedNet::new(&mut net, caps, &mut rate_limiter);

        let big_payload = vec![0u8; MAX_PAYLOAD_SIZE + 1];
        let result = checked.send_message("mod:test:channel", MessageTarget::Server, &big_payload);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn test_rate_limiting() {
        let mut net = MockNet::default();
        let mut rate_limiter = TokenBucket::new(3, 0.0); // 3 tokens, no refill
        let caps = Capability::NET_SEND;

        let mut checked = CapabilityCheckedNet::new(&mut net, caps, &mut rate_limiter);

        // First 3 should succeed
        for _ in 0..3 {
            let result = checked.send_message("mod:test:channel", MessageTarget::Server, b"hi");
            assert!(result.is_ok());
        }

        // 4th should fail
        let result = checked.send_message("mod:test:channel", MessageTarget::Server, b"hi");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::RateLimited
        );
    }
}
