# Data Model: Feature 040 Security Pass

**Date**: 2025-12-19
**Status**: Complete

## Overview

This document defines the data structures and entities for the Security Pass feature. All entities are in-memory runtime state with no persistence requirements.

---

## Core Entities

### 1. SecurityLimits

Centralized configuration holding all security boundaries. Defined as compile-time constants with optional runtime overrides.

```
SecurityLimits
├── Protocol Limits
│   ├── max_packet_bytes: usize = 65536 (64 KB)
│   ├── max_message_bytes: usize = 65536
│   ├── max_string_bytes: usize = 1024 (1 KB)
│   └── max_list_len: usize = 256
│
├── Handshake Limits
│   ├── max_cached_payload_hashes: usize = 256
│   ├── handshake_timeout_secs: u32 = 10
│   └── max_pending_per_source: usize = 10
│
├── Payload Sync Limits
│   ├── max_payload_bytes: usize = 26214400 (25 MB)
│   ├── max_chunks_per_transfer: usize = 1024
│   ├── max_resend_per_window: usize = 16
│   ├── transfer_timeout_secs: u32 = 30
│   └── chunk_timeout_secs: u32 = 5
│
├── Rate Limits
│   ├── global_msg_per_sec: u32 = 200
│   ├── mod_channel_tokens: u32 = 50
│   ├── mod_channel_refill_per_sec: u32 = 10
│   └── expensive_msg_per_sec: u32 = 10
│
├── Parser Limits
│   ├── max_index_json_bytes: usize = 5242880 (5 MB)
│   ├── max_mod_toml_bytes: usize = 262144 (256 KB)
│   ├── max_zip_files: usize = 10000
│   ├── max_zip_ratio: f64 = 20.0
│   └── max_bundle_bytes: usize = 52428800 (50 MB)
│
└── Strike Limits
    ├── strikes_before_disconnect: u32 = 5
    ├── strikes_decay_secs: u32 = 60
    └── ban_cooldown_secs: u32 = 300
```

**Validation Rules**:
- All byte limits must be > 0
- Timeout values must be >= 1 second
- Rate limits must be >= 1
- max_zip_ratio must be >= 1.0

**Relationships**:
- Used by: All decode/parse functions, StrikeTracker, RateLimiter

---

### 2. StrikeTracker

Per-connection state tracking accumulated violations. Each client connection has exactly one StrikeTracker.

```
StrikeTracker
├── Identification
│   └── connection_id: u64
│
├── Strike State
│   ├── strikes: u32 (current count)
│   ├── last_strike_time: Instant
│   └── categories: HashMap<StrikeCategory, u32>
│
└── Decision State
    ├── should_disconnect: bool
    └── disconnect_reason: Option<String>
```

**Strike Categories**:
```
StrikeCategory (enum)
├── DecodeError        # Invalid message format
├── SizeViolation      # Message too large
├── RateExceeded       # Too many messages
├── HandshakeAbuse     # Handshake protocol violations
├── PayloadSyncAbuse   # Payload sync protocol violations
├── HashMismatch       # Integrity verification failed
└── ParserAbuse        # Parser limit exceeded
```

**Lifecycle**:
1. Created on connection accept
2. Updated on each violation via `record_strike(category)`
3. Strikes decay over time (configurable)
4. When `strikes >= strikes_before_disconnect`, set `should_disconnect = true`
5. Destroyed on connection close

**Methods**:
- `record_strike(category: StrikeCategory) -> bool`: Returns true if should disconnect
- `decay()`: Reduce strikes based on elapsed time
- `get_strikes() -> u32`: Current strike count
- `should_disconnect() -> bool`: Check disconnect threshold

---

### 3. RateLimiter

Token bucket implementation for message rate limiting. Supports per-client and per-type limits.

```
RateLimiter
├── Global Bucket
│   ├── capacity: u32 = 200
│   ├── tokens: u32 (current)
│   ├── refill_rate: u32 = 200 per second
│   └── last_refill: Instant
│
├── Per-Type Buckets
│   └── buckets: HashMap<MessageType, TokenBucket>
│       ├── ModChannel → capacity: 50, refill: 10/sec
│       ├── BlockEdit → capacity: 20, refill: 10/sec
│       └── Chat → capacity: 10, refill: 2/sec
│
└── State
    ├── last_check: Instant
    └── blocked_count: u64 (metrics)
```

**TokenBucket (inner struct)**:
```
TokenBucket
├── capacity: u32
├── tokens: u32
├── refill_rate: u32 (per second)
└── last_refill: Instant
```

**Methods**:
- `try_consume_global() -> bool`: Check global rate limit
- `try_consume_type(msg_type: MessageType) -> bool`: Check per-type limit
- `try_consume(msg_type: MessageType) -> bool`: Check both, returns false if either fails
- `refill()`: Update token counts based on elapsed time

**Relationships**:
- One RateLimiter per connection
- Reports to SecurityMetrics when blocking

---

### 4. HandshakeTracker

Tracks pending handshakes for timeout and abuse detection.

```
HandshakeTracker
├── pending: HashMap<ConnectionId, PendingHandshake>
├── per_source_count: HashMap<SocketAddr, usize>
└── max_pending_per_source: usize
```

**PendingHandshake (inner struct)**:
```
PendingHandshake
├── connection_id: u64
├── source_addr: SocketAddr
├── start_time: Instant
├── state: HandshakeState
└── cached_hashes_count: usize
```

**HandshakeState (enum)**:
```
HandshakeState
├── AwaitingConnect
├── AwaitingModSet
├── AwaitingPayloadSync
└── Complete
```

**Methods**:
- `register(conn_id, addr) -> Result<(), TooManyPending>`: Add pending handshake
- `update_state(conn_id, new_state)`: Transition state
- `check_timeouts() -> Vec<ConnectionId>`: Return timed-out connections
- `complete(conn_id)`: Remove from pending
- `abort(conn_id)`: Remove and cleanup

**Validation Rules**:
- Per-source count must not exceed `max_pending_per_source`
- Elapsed time must not exceed `handshake_timeout_secs`
- `cached_hashes_count` must not exceed `max_cached_payload_hashes`

---

### 5. SecurityMetrics

Counters and observable state for security monitoring.

```
SecurityMetrics
├── Counters (atomic u64)
│   ├── invalid_messages_total
│   ├── disconnects_strikes_total
│   ├── payload_sync_aborts_total
│   ├── registry_parse_failures_total
│   ├── rate_limited_total
│   ├── handshake_timeouts_total
│   └── zip_safety_rejections_total
│
└── Per-Category Breakdown
    └── strikes_by_category: HashMap<StrikeCategory, u64>
```

**Methods**:
- `record_invalid_message()`: Increment counter
- `record_disconnect(reason: StrikeCategory)`: Increment + categorize
- `record_rate_limited()`: Increment counter
- `snapshot() -> MetricsSnapshot`: Get current values for reporting

---

### 6. RateLimitedLogger

Wrapper for rate-limited security logging.

```
RateLimitedLogger
├── last_log: HashMap<String, Instant>
├── suppressed: HashMap<String, u64>
├── min_interval: Duration
└── debug_mode: bool
```

**Log Categories**:
```
LogCategory (string constants)
├── "decode_error"
├── "size_violation"
├── "rate_exceeded"
├── "handshake_abuse"
├── "payload_sync_abuse"
├── "parser_abuse"
└── "zip_safety"
```

**Methods**:
- `warn(category: &str, message: &str)`: Rate-limited warning
- `debug(category: &str, message: &str)`: Only if debug_mode enabled
- `error(category: &str, message: &str)`: Always logged (no rate limiting for errors)

---

### 7. FuzzCorpus

Collection of valid message samples for seeding fuzz tests. (Development-only, not runtime)

```
FuzzCorpus
├── client_messages/
│   ├── connect.bin
│   ├── input.bin
│   ├── block_edit.bin
│   └── mod_set_response.bin
│
├── server_messages/
│   ├── connected.bin
│   ├── snapshot.bin
│   └── payload_chunk.bin
│
└── invalid_samples/
    ├── truncated.bin
    ├── oversized.bin
    └── random_bytes.bin
```

**Generation**:
- Created during test runs or manually crafted
- Valid samples captured from actual protocol exchanges
- Invalid samples designed to test boundary conditions

---

## Entity Relationships

```
Connection (1) ────── (1) StrikeTracker
    │
    ├──────────────── (1) RateLimiter
    │                      ├── GlobalBucket
    │                      └── PerTypeBuckets
    │
    └──────────────── (0..1) PendingHandshake
                           (via HandshakeTracker)

SecurityLimits (1) ─────── used by ─────── (many) Decode/Parse functions

SecurityMetrics (1) ────── receives from ─── (many) StrikeTrackers
                    ────── receives from ─── (many) RateLimiters
                    ────── receives from ─── HandshakeTracker

RateLimitedLogger (1) ──── used by ─────── (all) Security subsystems
```

---

## State Transitions

### StrikeTracker Lifecycle
```
[New Connection] → Created(strikes=0)
                       │
                       ▼
                   Active ◄──────────────┐
                       │                 │
                 record_strike()         │ decay()
                       │                 │
                       ▼                 │
              strikes < threshold? ──yes─┘
                       │
                      no
                       │
                       ▼
              should_disconnect=true
                       │
                       ▼
                  [Disconnect]
```

### HandshakeTracker State Machine
```
[Accept] → AwaitingConnect
               │
               ▼ (Connect received)
          AwaitingModSet
               │
               ▼ (ModSetResponse received)
          AwaitingPayloadSync
               │
               ▼ (Sync complete or not needed)
             Complete
               │
               ▼
          [Remove from tracker]

At any state:
  - Timeout → [Abort + Disconnect]
  - Error → [Abort + Strike + Disconnect]
```

---

## Validation Summary

| Entity | Key Validations |
|--------|-----------------|
| SecurityLimits | All values > 0, ratios >= 1.0, timeouts >= 1s |
| StrikeTracker | strikes <= max, categories valid enum |
| RateLimiter | tokens <= capacity, refill bounded |
| HandshakeTracker | per-source <= max, elapsed <= timeout |
| SecurityMetrics | Counters monotonically increasing |
| RateLimitedLogger | min_interval > 0 |
