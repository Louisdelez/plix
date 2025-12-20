//! Security Subsystem - Protection against protocol abuse and attacks
//!
//! This module provides the security infrastructure for the Plix server:
//!
//! - [`strikes`] - Strike tracking for violation-based disconnects
//! - [`rate_limiter`] - Token bucket rate limiting
//! - [`handshake`] - Handshake timeout and abuse detection
//! - [`observability`] - Security metrics and rate-limited logging
//! - [`SecurityContext`] - Per-connection security state bundle
//!
//! # Architecture
//!
//! Each client connection has:
//! - One [`SecurityContext`] which bundles:
//!   - [`StrikeTracker`] for accumulating violations
//!   - [`RateLimiter`] for message rate enforcement
//!
//! The server has:
//! - One [`HandshakeTracker`] for pending connection management
//! - One [`SecurityMetrics`] for global observability
//! - One [`RateLimitedLogger`] for throttled security logging
//!
//! # Usage
//!
//! ```ignore
//! use plix_server::security::{SecurityContext, StrikeCategory, RateLimitedMessageType};
//!
//! let mut ctx = SecurityContext::new(connection_id);
//!
//! // Check rate limit before processing
//! if !ctx.check_rate_limit(RateLimitedMessageType::Global) {
//!     // Already recorded strike, check if should disconnect
//!     if ctx.should_disconnect() {
//!         // Disconnect client
//!     }
//! }
//! ```

pub mod handshake;
pub mod observability;
pub mod rate_limiter;
pub mod strikes;

// Re-export commonly used types
pub use handshake::{HandshakeError, HandshakeState, HandshakeTracker};
pub use observability::{MetricsSnapshot, RateLimitedLogger, SecurityMetrics};
pub use rate_limiter::{RateLimitedMessageType, RateLimiter, TokenBucket};
pub use strikes::{StrikeCategory, StrikeTracker};

use plix_common::protocol::ClientMessage;

/// Per-connection security context bundling all security state.
///
/// This is the primary interface for security checks in the network loop.
/// Each connection should have exactly one SecurityContext.
#[derive(Debug)]
pub struct SecurityContext {
    /// Connection identifier for logging
    connection_id: u64,
    /// Strike tracker for violation accumulation
    strikes: StrikeTracker,
    /// Rate limiter for message frequency
    rate_limiter: RateLimiter,
}

impl SecurityContext {
    /// Create a new security context for a connection.
    pub fn new(connection_id: u64) -> Self {
        Self {
            connection_id,
            strikes: StrikeTracker::new(connection_id),
            rate_limiter: RateLimiter::new(),
        }
    }

    /// Check if a message is allowed by rate limiting.
    ///
    /// If rate limited, automatically records a strike.
    /// Returns `true` if message is allowed, `false` if rate limited.
    pub fn check_rate_limit(&mut self, msg_type: RateLimitedMessageType) -> bool {
        if self.rate_limiter.try_consume(msg_type) {
            true
        } else {
            self.strikes.record_strike(StrikeCategory::RateExceeded);
            false
        }
    }

    /// Check rate limit for a client message, determining type automatically.
    ///
    /// Returns `true` if allowed, `false` if rate limited.
    pub fn check_message_rate(&mut self, msg: &ClientMessage) -> bool {
        let msg_type = Self::classify_message(msg);
        self.check_rate_limit(msg_type)
    }

    /// Classify a ClientMessage into a rate limit category.
    fn classify_message(msg: &ClientMessage) -> RateLimitedMessageType {
        match msg {
            ClientMessage::BlockEdit(_) => RateLimitedMessageType::BlockEdit,
            ClientMessage::ChatSend { .. } => RateLimitedMessageType::Chat,
            ClientMessage::UseActiveItem => RateLimitedMessageType::Attack,
            // All other messages use global rate limit only
            _ => RateLimitedMessageType::Global,
        }
    }

    /// Record a decode error and return whether to disconnect.
    pub fn record_decode_error(&mut self) -> bool {
        self.strikes.record_strike(StrikeCategory::DecodeError)
    }

    /// Record a size violation and return whether to disconnect.
    pub fn record_size_violation(&mut self) -> bool {
        self.strikes.record_strike(StrikeCategory::SizeViolation)
    }

    /// Record a handshake abuse and return whether to disconnect.
    pub fn record_handshake_abuse(&mut self) -> bool {
        self.strikes.record_strike(StrikeCategory::HandshakeAbuse)
    }

    /// Record a payload sync abuse and return whether to disconnect.
    pub fn record_payload_sync_abuse(&mut self) -> bool {
        self.strikes.record_strike(StrikeCategory::PayloadSyncAbuse)
    }

    /// Record a hash mismatch and return whether to disconnect.
    pub fn record_hash_mismatch(&mut self) -> bool {
        self.strikes.record_strike(StrikeCategory::HashMismatch)
    }

    /// Record a parser abuse and return whether to disconnect.
    pub fn record_parser_abuse(&mut self) -> bool {
        self.strikes.record_strike(StrikeCategory::ParserAbuse)
    }

    /// Record a strike with the given category and return whether to disconnect.
    pub fn record_strike(&mut self, category: StrikeCategory) -> bool {
        self.strikes.record_strike(category)
    }

    /// Check if connection should be disconnected due to violations.
    pub fn should_disconnect(&self) -> bool {
        self.strikes.should_disconnect()
    }

    /// Get the reason for disconnect if applicable.
    pub fn disconnect_reason(&self) -> Option<&str> {
        self.strikes.disconnect_reason()
    }

    /// Apply time-based decay to strikes. Call periodically (e.g., once per second).
    pub fn decay(&mut self) {
        self.strikes.decay();
    }

    /// Get current strike count.
    pub fn strike_count(&self) -> u32 {
        self.strikes.get_strikes()
    }

    /// Get the connection ID.
    pub fn connection_id(&self) -> u64 {
        self.connection_id
    }

    /// Get the number of rate-limited messages.
    pub fn rate_limited_count(&self) -> u64 {
        self.rate_limiter.blocked_count()
    }

    /// Get reference to the strike tracker.
    pub fn strikes(&self) -> &StrikeTracker {
        &self.strikes
    }

    /// Get mutable reference to the strike tracker.
    pub fn strikes_mut(&mut self) -> &mut StrikeTracker {
        &mut self.strikes
    }

    /// Get reference to the rate limiter.
    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }

    /// Get mutable reference to the rate limiter.
    pub fn rate_limiter_mut(&mut self) -> &mut RateLimiter {
        &mut self.rate_limiter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::limits::STRIKES_BEFORE_DISCONNECT;
    use plix_common::protocol::{BlockEditKind, BlockEditRequest};
    use plix_common::types::{BlockPos, BlockType};

    fn test_block_edit() -> BlockEditRequest {
        BlockEditRequest {
            kind: BlockEditKind::Place,
            target_pos: BlockPos::new(0, 0, 0),
            block_type: BlockType::STONE,
        }
    }

    #[test]
    fn test_security_context_new() {
        let ctx = SecurityContext::new(42);
        assert_eq!(ctx.connection_id(), 42);
        assert_eq!(ctx.strike_count(), 0);
        assert!(!ctx.should_disconnect());
    }

    #[test]
    fn test_security_context_rate_limit() {
        let mut ctx = SecurityContext::new(1);

        // Should allow messages up to rate limit
        for _ in 0..10 {
            assert!(ctx.check_rate_limit(RateLimitedMessageType::Chat));
        }

        // 11th chat message should be rate limited
        assert!(!ctx.check_rate_limit(RateLimitedMessageType::Chat));
        assert_eq!(ctx.strike_count(), 1);
    }

    #[test]
    fn test_security_context_decode_error() {
        let mut ctx = SecurityContext::new(1);

        for _ in 0..STRIKES_BEFORE_DISCONNECT - 1 {
            assert!(!ctx.record_decode_error());
        }

        assert!(ctx.record_decode_error());
        assert!(ctx.should_disconnect());
    }

    #[test]
    fn test_security_context_classify_message() {
        assert_eq!(
            SecurityContext::classify_message(&ClientMessage::BlockEdit(test_block_edit())),
            RateLimitedMessageType::BlockEdit
        );
        assert_eq!(
            SecurityContext::classify_message(&ClientMessage::ChatSend {
                text: String::new()
            }),
            RateLimitedMessageType::Chat
        );
        assert_eq!(
            SecurityContext::classify_message(&ClientMessage::UseActiveItem),
            RateLimitedMessageType::Attack
        );
        assert_eq!(
            SecurityContext::classify_message(&ClientMessage::Disconnect),
            RateLimitedMessageType::Global
        );
    }
}
