# Security Module API Contracts

**Date**: 2025-12-19
**Module**: `plix-server::security`

## Overview

Internal API contracts for the security subsystem. These are Rust module APIs, not network protocols.

---

## 1. Limits API

**Module**: `plix_common::limits`

### Constants

```rust
// Protocol limits
pub const MAX_PACKET_BYTES: usize = 65536;
pub const MAX_MESSAGE_BYTES: usize = 65536;
pub const MAX_STRING_BYTES: usize = 1024;
pub const MAX_LIST_LEN: usize = 256;

// Handshake limits
pub const MAX_CACHED_PAYLOAD_HASHES: usize = 256;
pub const HANDSHAKE_TIMEOUT_SECS: u32 = 10;
pub const MAX_PENDING_PER_SOURCE: usize = 10;

// Payload sync limits
pub const MAX_PAYLOAD_BYTES: usize = 26_214_400;
pub const MAX_CHUNKS_PER_TRANSFER: usize = 1024;
pub const MAX_RESEND_PER_WINDOW: usize = 16;
pub const TRANSFER_TIMEOUT_SECS: u32 = 30;
pub const CHUNK_TIMEOUT_SECS: u32 = 5;

// Rate limits
pub const GLOBAL_MSG_PER_SEC: u32 = 200;
pub const MOD_CHANNEL_TOKENS: u32 = 50;
pub const MOD_CHANNEL_REFILL_PER_SEC: u32 = 10;
pub const EXPENSIVE_MSG_PER_SEC: u32 = 10;

// Parser limits
pub const MAX_INDEX_JSON_BYTES: usize = 5_242_880;
pub const MAX_MOD_TOML_BYTES: usize = 262_144;
pub const MAX_ZIP_FILES: usize = 10_000;
pub const MAX_ZIP_RATIO: f64 = 20.0;
pub const MAX_BUNDLE_BYTES: usize = 52_428_800;

// Strike limits
pub const STRIKES_BEFORE_DISCONNECT: u32 = 5;
pub const STRIKES_DECAY_SECS: u32 = 60;
pub const BAN_COOLDOWN_SECS: u32 = 300;
```

### Usage Contract

All decode and parse functions MUST use these constants, never local magic numbers.

---

## 2. StrikeTracker API

**Module**: `plix_server::security::strikes`

### Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrikeCategory {
    DecodeError,
    SizeViolation,
    RateExceeded,
    HandshakeAbuse,
    PayloadSyncAbuse,
    HashMismatch,
    ParserAbuse,
}

pub struct StrikeTracker {
    // private fields
}
```

### Constructor

```rust
impl StrikeTracker {
    /// Create a new tracker for a connection
    pub fn new(connection_id: u64) -> Self;
}
```

### Methods

```rust
impl StrikeTracker {
    /// Record a strike in the given category.
    /// Returns true if the connection should be disconnected.
    pub fn record_strike(&mut self, category: StrikeCategory) -> bool;

    /// Apply time-based strike decay.
    /// Call periodically (e.g., once per second).
    pub fn decay(&mut self);

    /// Get current strike count.
    pub fn get_strikes(&self) -> u32;

    /// Get strikes by category.
    pub fn get_strikes_by_category(&self) -> &HashMap<StrikeCategory, u32>;

    /// Check if connection should be disconnected.
    pub fn should_disconnect(&self) -> bool;

    /// Get disconnect reason if should_disconnect is true.
    pub fn disconnect_reason(&self) -> Option<&str>;
}
```

### Invariants

- `record_strike()` always increments strike count
- `should_disconnect()` returns true when `strikes >= STRIKES_BEFORE_DISCONNECT`
- `decay()` never increases strike count
- After `should_disconnect()` returns true, it remains true

---

## 3. RateLimiter API

**Module**: `plix_server::security::rate_limiter`

### Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RateLimitedMessageType {
    Global,
    ModChannel,
    BlockEdit,
    Chat,
    Attack,
}

pub struct RateLimiter {
    // private fields
}
```

### Constructor

```rust
impl RateLimiter {
    /// Create a new rate limiter with default configuration.
    pub fn new() -> Self;

    /// Create with custom configuration.
    pub fn with_config(config: RateLimiterConfig) -> Self;
}
```

### Methods

```rust
impl RateLimiter {
    /// Attempt to consume a token for the given message type.
    /// Returns true if allowed, false if rate limited.
    pub fn try_consume(&mut self, msg_type: RateLimitedMessageType) -> bool;

    /// Check if a message would be allowed without consuming tokens.
    pub fn would_allow(&self, msg_type: RateLimitedMessageType) -> bool;

    /// Get current token count for a bucket.
    pub fn tokens(&self, msg_type: RateLimitedMessageType) -> u32;

    /// Get total rate-limited message count.
    pub fn blocked_count(&self) -> u64;

    /// Reset all buckets to full capacity.
    pub fn reset(&mut self);
}
```

### Invariants

- `try_consume()` refills tokens based on elapsed time before checking
- Tokens never exceed capacity after refill
- `blocked_count()` is monotonically increasing
- Global limit is always checked in addition to per-type limits

---

## 4. HandshakeTracker API

**Module**: `plix_server::security::handshake`

### Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeState {
    AwaitingConnect,
    AwaitingModSet,
    AwaitingPayloadSync,
    Complete,
}

pub struct HandshakeTracker {
    // private fields
}

#[derive(Debug)]
pub enum HandshakeError {
    TooManyPending,
    InvalidTransition,
    Timeout,
    CacheHashLimitExceeded,
}
```

### Constructor

```rust
impl HandshakeTracker {
    /// Create a new handshake tracker.
    pub fn new() -> Self;
}
```

### Methods

```rust
impl HandshakeTracker {
    /// Register a new pending handshake.
    /// Returns error if per-source limit exceeded.
    pub fn register(
        &mut self,
        connection_id: u64,
        source_addr: SocketAddr,
    ) -> Result<(), HandshakeError>;

    /// Update handshake state.
    /// Returns error if transition is invalid.
    pub fn update_state(
        &mut self,
        connection_id: u64,
        new_state: HandshakeState,
    ) -> Result<(), HandshakeError>;

    /// Record cached hashes count from client.
    /// Returns error if limit exceeded.
    pub fn set_cached_hashes_count(
        &mut self,
        connection_id: u64,
        count: usize,
    ) -> Result<(), HandshakeError>;

    /// Check all pending handshakes for timeout.
    /// Returns list of timed-out connection IDs.
    pub fn check_timeouts(&mut self) -> Vec<u64>;

    /// Mark handshake as complete and remove from tracking.
    pub fn complete(&mut self, connection_id: u64);

    /// Abort handshake and remove from tracking.
    pub fn abort(&mut self, connection_id: u64);

    /// Get current pending count for a source.
    pub fn pending_count(&self, source_addr: &SocketAddr) -> usize;

    /// Get total pending handshakes.
    pub fn total_pending(&self) -> usize;
}
```

### Invariants

- `register()` fails if `pending_count(source_addr) >= MAX_PENDING_PER_SOURCE`
- `check_timeouts()` returns IDs where `elapsed >= HANDSHAKE_TIMEOUT_SECS`
- `complete()` and `abort()` always remove from pending (no-op if not found)
- State transitions are validated: only valid progressions allowed

---

## 5. SecurityMetrics API

**Module**: `plix_server::security::observability`

### Types

```rust
pub struct SecurityMetrics {
    // Atomic counters
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub invalid_messages_total: u64,
    pub disconnects_strikes_total: u64,
    pub payload_sync_aborts_total: u64,
    pub registry_parse_failures_total: u64,
    pub rate_limited_total: u64,
    pub handshake_timeouts_total: u64,
    pub zip_safety_rejections_total: u64,
    pub strikes_by_category: HashMap<StrikeCategory, u64>,
}
```

### Constructor

```rust
impl SecurityMetrics {
    /// Create global metrics instance.
    pub fn new() -> Self;
}
```

### Methods

```rust
impl SecurityMetrics {
    // Recording methods (thread-safe, atomic)
    pub fn record_invalid_message(&self);
    pub fn record_disconnect(&self, category: StrikeCategory);
    pub fn record_payload_sync_abort(&self);
    pub fn record_registry_parse_failure(&self);
    pub fn record_rate_limited(&self);
    pub fn record_handshake_timeout(&self);
    pub fn record_zip_safety_rejection(&self);

    /// Get snapshot of all metrics.
    pub fn snapshot(&self) -> MetricsSnapshot;

    /// Reset all counters (testing only).
    #[cfg(test)]
    pub fn reset(&self);
}
```

### Invariants

- All recording methods are thread-safe (use atomics)
- Counters are monotonically increasing (no decrements)
- `snapshot()` returns a consistent point-in-time view

---

## 6. RateLimitedLogger API

**Module**: `plix_server::security::observability`

### Types

```rust
pub struct RateLimitedLogger {
    // private fields
}
```

### Constructor

```rust
impl RateLimitedLogger {
    /// Create with default interval (1 second).
    pub fn new() -> Self;

    /// Create with custom interval.
    pub fn with_interval(interval: Duration) -> Self;

    /// Enable debug mode for verbose logging.
    pub fn with_debug_mode(self, enabled: bool) -> Self;
}
```

### Methods

```rust
impl RateLimitedLogger {
    /// Log a warning with rate limiting.
    /// Category is used for per-category throttling.
    pub fn warn(&mut self, category: &str, message: &str);

    /// Log debug message (only if debug_mode enabled).
    pub fn debug(&mut self, category: &str, message: &str);

    /// Log error (always logged, no rate limiting).
    pub fn error(&mut self, category: &str, message: &str);

    /// Get suppressed message count for a category.
    pub fn suppressed_count(&self, category: &str) -> u64;

    /// Reset all suppression state.
    pub fn reset(&mut self);
}
```

### Invariants

- `warn()` is rate-limited per category
- `error()` is never rate-limited
- `debug()` respects debug_mode flag
- Suppressed count is included in next logged message

---

## 7. Bounded Decode API

**Module**: `plix_common::protocol::codec`

### Updated Functions

```rust
/// Decode a message from bytes with size validation.
/// Returns error if bytes exceed MAX_MESSAGE_BYTES.
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, ProtocolError>;

/// Decode with explicit size limit.
pub fn decode_with_limit<T: DeserializeOwned>(
    bytes: &[u8],
    max_bytes: usize,
) -> Result<T, ProtocolError>;
```

### New Error Variants

```rust
pub enum ProtocolError {
    // Existing variants...

    /// Message exceeds size limit
    MessageTooLarge(usize),

    /// String field exceeds limit
    StringTooLong { field: &'static str, len: usize },

    /// List field exceeds limit
    ListTooLong { field: &'static str, len: usize },

    /// Numeric value out of bounds
    ValueOutOfBounds { field: &'static str, value: i64 },
}
```

### Contract

- All `decode()` calls MUST check size before deserialization
- Errors MUST be typed, not panics
- Error context MUST include which limit was exceeded

---

## Integration Points

### Netloop Integration

```rust
// In message receive handler
fn handle_message(&mut self, conn_id: u64, bytes: &[u8]) {
    // 1. Rate limit check
    if !self.rate_limiter.try_consume(RateLimitedMessageType::Global) {
        self.metrics.record_rate_limited();
        self.strikes.record_strike(StrikeCategory::RateExceeded);
        return;
    }

    // 2. Size check before decode
    if bytes.len() > MAX_MESSAGE_BYTES {
        self.metrics.record_invalid_message();
        if self.strikes.record_strike(StrikeCategory::SizeViolation) {
            self.disconnect(conn_id, "Too many violations");
        }
        return;
    }

    // 3. Decode with error handling
    match decode::<ClientMessage>(bytes) {
        Ok(msg) => self.process_message(conn_id, msg),
        Err(e) => {
            self.logger.warn("decode_error", &format!("{:?}", e));
            if self.strikes.record_strike(StrikeCategory::DecodeError) {
                self.disconnect(conn_id, "Invalid message format");
            }
        }
    }
}
```

### Handshake Integration

```rust
// In connection accept handler
fn on_accept(&mut self, conn_id: u64, addr: SocketAddr) {
    match self.handshake_tracker.register(conn_id, addr) {
        Ok(()) => {
            // Start handshake
        }
        Err(HandshakeError::TooManyPending) => {
            self.metrics.record_handshake_timeout();
            self.disconnect(conn_id, "Too many pending connections");
        }
    }
}

// In tick handler
fn tick(&mut self) {
    for conn_id in self.handshake_tracker.check_timeouts() {
        self.metrics.record_handshake_timeout();
        self.disconnect(conn_id, "Handshake timeout");
    }
}
```
