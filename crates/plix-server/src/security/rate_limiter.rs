//! Rate Limiting - Token bucket implementation for message rate enforcement
//!
//! Provides smooth rate limiting with burst allowance using the token bucket algorithm.

use std::collections::HashMap;
use std::time::Instant;

use plix_common::limits::{
    EXPENSIVE_MSG_PER_SEC, GLOBAL_MSG_PER_SEC, MOD_CHANNEL_REFILL_PER_SEC, MOD_CHANNEL_TOKENS,
};

/// Message types that can be rate limited
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RateLimitedMessageType {
    /// Global rate limit (all messages)
    Global,
    /// Mod channel messages
    ModChannel,
    /// Block edit messages
    BlockEdit,
    /// Chat messages
    Chat,
    /// Attack/combat messages
    Attack,
}

/// Token bucket for rate limiting
///
/// Tokens are consumed for each message and refilled over time.
/// Allows bursts up to capacity while enforcing average rate.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: u32,
    tokens: u32,
    refill_rate: u32, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    pub fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            capacity,
            tokens: capacity, // Start full
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Attempt to consume a token.
    ///
    /// Returns `true` if successful, `false` if rate limited.
    pub fn try_consume(&mut self) -> bool {
        self.try_consume_n(1)
    }

    /// Attempt to consume N tokens.
    ///
    /// Returns `true` if successful, `false` if rate limited.
    pub fn try_consume_n(&mut self, n: u32) -> bool {
        self.refill();

        if self.tokens >= n {
            self.tokens -= n;
            true
        } else {
            false
        }
    }

    /// Check if consuming would succeed without actually consuming.
    pub fn would_allow(&self) -> bool {
        self.would_allow_n(1)
    }

    /// Check if consuming N tokens would succeed.
    pub fn would_allow_n(&self, n: u32) -> bool {
        let elapsed = self.last_refill.elapsed().as_secs_f32();
        let potential_tokens = self.tokens + (elapsed * self.refill_rate as f32) as u32;
        potential_tokens.min(self.capacity) >= n
    }

    /// Get current token count (after refill)
    pub fn tokens(&mut self) -> u32 {
        self.refill();
        self.tokens
    }

    /// Get current token count without refilling
    pub fn tokens_raw(&self) -> u32 {
        self.tokens
    }

    /// Reset bucket to full capacity
    pub fn reset(&mut self) {
        self.tokens = self.capacity;
        self.last_refill = Instant::now();
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed();
        let new_tokens = (elapsed.as_secs_f32() * self.refill_rate as f32) as u32;

        if new_tokens > 0 {
            self.tokens = (self.tokens + new_tokens).min(self.capacity);
            self.last_refill = Instant::now();
        }
    }
}

/// Per-connection rate limiter with global and per-type buckets
#[derive(Debug)]
pub struct RateLimiter {
    global: TokenBucket,
    per_type: HashMap<RateLimitedMessageType, TokenBucket>,
    blocked_count: u64,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    /// Create a new rate limiter with default configuration
    pub fn new() -> Self {
        let mut per_type = HashMap::new();

        // Configure per-type buckets
        per_type.insert(
            RateLimitedMessageType::ModChannel,
            TokenBucket::new(MOD_CHANNEL_TOKENS, MOD_CHANNEL_REFILL_PER_SEC),
        );
        per_type.insert(
            RateLimitedMessageType::BlockEdit,
            TokenBucket::new(20, 10), // 20 burst, 10/sec
        );
        per_type.insert(
            RateLimitedMessageType::Chat,
            TokenBucket::new(10, 2), // 10 burst, 2/sec
        );
        per_type.insert(
            RateLimitedMessageType::Attack,
            TokenBucket::new(EXPENSIVE_MSG_PER_SEC, EXPENSIVE_MSG_PER_SEC),
        );

        Self {
            global: TokenBucket::new(GLOBAL_MSG_PER_SEC, GLOBAL_MSG_PER_SEC),
            per_type,
            blocked_count: 0,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: RateLimiterConfig) -> Self {
        let mut limiter = Self::new();
        limiter.global = TokenBucket::new(config.global_capacity, config.global_refill);
        limiter
    }

    /// Attempt to consume a token for the given message type.
    ///
    /// Checks both global and per-type limits.
    /// Returns `true` if allowed, `false` if rate limited.
    pub fn try_consume(&mut self, msg_type: RateLimitedMessageType) -> bool {
        // Always check global first
        if !self.global.try_consume() {
            self.blocked_count += 1;
            return false;
        }

        // Check per-type if not Global
        if msg_type != RateLimitedMessageType::Global {
            if let Some(bucket) = self.per_type.get_mut(&msg_type) {
                if !bucket.try_consume() {
                    // Refund global token since per-type failed
                    self.global.tokens = (self.global.tokens + 1).min(self.global.capacity);
                    self.blocked_count += 1;
                    return false;
                }
            }
        }

        true
    }

    /// Check if a message would be allowed without consuming tokens.
    pub fn would_allow(&self, msg_type: RateLimitedMessageType) -> bool {
        if !self.global.would_allow() {
            return false;
        }

        if msg_type != RateLimitedMessageType::Global {
            if let Some(bucket) = self.per_type.get(&msg_type) {
                if !bucket.would_allow() {
                    return false;
                }
            }
        }

        true
    }

    /// Get current token count for a bucket
    pub fn tokens(&mut self, msg_type: RateLimitedMessageType) -> u32 {
        if msg_type == RateLimitedMessageType::Global {
            self.global.tokens()
        } else {
            self.per_type
                .get_mut(&msg_type)
                .map(|b| b.tokens())
                .unwrap_or(0)
        }
    }

    /// Get total blocked message count
    pub fn blocked_count(&self) -> u64 {
        self.blocked_count
    }

    /// Reset all buckets to full capacity
    pub fn reset(&mut self) {
        self.global.reset();
        for bucket in self.per_type.values_mut() {
            bucket.reset();
        }
        self.blocked_count = 0;
    }
}

/// Configuration for rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    pub global_capacity: u32,
    pub global_refill: u32,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            global_capacity: GLOBAL_MSG_PER_SEC,
            global_refill: GLOBAL_MSG_PER_SEC,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_starts_full() {
        let bucket = TokenBucket::new(100, 10);
        assert_eq!(bucket.tokens_raw(), 100);
    }

    #[test]
    fn test_token_bucket_consume() {
        let mut bucket = TokenBucket::new(100, 10);
        assert!(bucket.try_consume());
        assert_eq!(bucket.tokens_raw(), 99);
    }

    #[test]
    fn test_token_bucket_consume_n() {
        let mut bucket = TokenBucket::new(100, 10);
        assert!(bucket.try_consume_n(50));
        assert_eq!(bucket.tokens_raw(), 50);
    }

    #[test]
    fn test_token_bucket_empty_blocks() {
        let mut bucket = TokenBucket::new(5, 1);

        // Drain the bucket
        for _ in 0..5 {
            assert!(bucket.try_consume());
        }

        // Next should fail
        assert!(!bucket.try_consume());
    }

    #[test]
    fn test_token_bucket_would_allow() {
        let mut bucket = TokenBucket::new(5, 1);

        // Drain
        for _ in 0..5 {
            bucket.try_consume();
        }

        assert!(!bucket.would_allow());
    }

    #[test]
    fn test_token_bucket_reset() {
        let mut bucket = TokenBucket::new(100, 10);
        bucket.try_consume_n(50);
        bucket.reset();
        assert_eq!(bucket.tokens_raw(), 100);
    }

    #[test]
    fn test_rate_limiter_global() {
        let mut limiter = RateLimiter::new();

        // Should allow up to global limit
        for _ in 0..GLOBAL_MSG_PER_SEC {
            assert!(limiter.try_consume(RateLimitedMessageType::Global));
        }

        // Next should be blocked
        assert!(!limiter.try_consume(RateLimitedMessageType::Global));
        assert_eq!(limiter.blocked_count(), 1);
    }

    #[test]
    fn test_rate_limiter_per_type() {
        let mut limiter = RateLimiter::new();

        // Chat has 10 burst capacity
        for _ in 0..10 {
            assert!(limiter.try_consume(RateLimitedMessageType::Chat));
        }

        // 11th should be blocked
        assert!(!limiter.try_consume(RateLimitedMessageType::Chat));
    }

    #[test]
    fn test_rate_limiter_reset() {
        let mut limiter = RateLimiter::new();

        // Consume some
        for _ in 0..50 {
            limiter.try_consume(RateLimitedMessageType::Global);
        }

        limiter.reset();

        assert_eq!(limiter.tokens(RateLimitedMessageType::Global), GLOBAL_MSG_PER_SEC);
        assert_eq!(limiter.blocked_count(), 0);
    }

    #[test]
    fn test_rate_limiter_blocked_count_increments() {
        let mut limiter = RateLimiter::new();

        // Drain global
        for _ in 0..GLOBAL_MSG_PER_SEC {
            limiter.try_consume(RateLimitedMessageType::Global);
        }

        // Try more
        limiter.try_consume(RateLimitedMessageType::Global);
        limiter.try_consume(RateLimitedMessageType::Global);
        limiter.try_consume(RateLimitedMessageType::Global);

        assert_eq!(limiter.blocked_count(), 3);
    }
}
