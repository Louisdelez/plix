# Security Limits Reference

All security limits are defined in `plix-common/src/limits.rs`. This document explains each limit and its rationale.

## Protocol Limits

| Constant | Value | Rationale |
|----------|-------|-----------|
| `MAX_PACKET_BYTES` | 64 KB | Maximum UDP/TCP packet size. Prevents memory exhaustion from oversized packets. |
| `MAX_MESSAGE_BYTES` | 64 KB | Maximum decoded message size. Same as packet limit for consistency. |
| `MAX_STRING_BYTES` | 1 KB | Maximum string field length. Prevents memory abuse via long strings. |
| `MAX_LIST_LEN` | 256 | Maximum list/array elements. Prevents O(n) abuse in list processing. |

## Handshake Limits

| Constant | Value | Rationale |
|----------|-------|-----------|
| `MAX_CACHED_PAYLOAD_HASHES` | 256 | Maximum hashes a client can claim to have cached. Matches MAX_LIST_LEN. |
| `HANDSHAKE_TIMEOUT_SECS` | 10s | Time to complete handshake. Prevents slow-handshake DoS. |
| `MAX_PENDING_PER_SOURCE` | 10 | Maximum pending connections per IP. Prevents connection exhaustion from single source. |

## Payload Sync Limits

| Constant | Value | Rationale |
|----------|-------|-----------|
| `MAX_PAYLOAD_BYTES` | 25 MB | Maximum total payload size. Based on expected mod sizes. |
| `MAX_CHUNKS_PER_TRANSFER` | 1024 | Maximum chunks per mod transfer. With 256 KB chunks, supports up to 256 MB. |
| `MAX_RESEND_PER_WINDOW` | 16 | Maximum resend requests in a time window. Prevents resend spam. |
| `TRANSFER_TIMEOUT_SECS` | 30s | Overall transfer timeout. Prevents stuck transfers. |
| `CHUNK_TIMEOUT_SECS` | 5s | Individual chunk ack timeout. Detects stalled transfers. |

## Rate Limits

| Constant | Value | Rationale |
|----------|-------|-----------|
| `GLOBAL_MSG_PER_SEC` | 200 | Maximum messages per client per second. Based on 60 Hz tick rate plus headroom. |
| `MOD_CHANNEL_TOKENS` | 50 | Initial bucket capacity for mod messages. Allows burst then rate limits. |
| `MOD_CHANNEL_REFILL_PER_SEC` | 10 | Refill rate for mod channel bucket. Steady-state limit. |
| `EXPENSIVE_MSG_PER_SEC` | 10 | Limit for expensive operations (block edits, etc.). Prevents server overload. |

## Parser Limits

| Constant | Value | Rationale |
|----------|-------|-----------|
| `MAX_INDEX_JSON_BYTES` | 5 MB | Maximum registry index size. Based on expected registry sizes. |
| `MAX_MOD_TOML_BYTES` | 256 KB | Maximum mod.toml size. More than enough for any manifest. |
| `MAX_ZIP_FILES` | 10,000 | Maximum files in a bundle. Prevents file count exhaustion. |
| `MAX_ZIP_RATIO` | 20:1 | Maximum decompression ratio. Detects zip bombs. |
| `MAX_BUNDLE_BYTES` | 50 MB | Maximum bundle size. Double the payload limit for overhead. |

## Strike Limits

| Constant | Value | Rationale |
|----------|-------|-----------|
| `STRIKES_BEFORE_DISCONNECT` | 5 | Violations before disconnect. Allows for transient errors. |
| `STRIKES_DECAY_SECS` | 60s | Time for one strike to decay. Allows recovery. |
| `BAN_COOLDOWN_SECS` | 300s | Cooldown after disconnect. Prevents rapid reconnection. |

## Adjusting Limits

When adjusting limits, consider:

1. **Memory impact**: Larger limits mean more memory per connection.
2. **CPU impact**: Higher rate limits mean more processing.
3. **Legitimate use**: Ensure limits don't block valid usage.
4. **Attack surface**: Lower limits provide better DoS protection.

### Example: Adjusting MAX_STRING_BYTES

```rust
// Current: 1 KB - safe for player names, chat messages
pub const MAX_STRING_BYTES: usize = 1024;

// If you need longer strings (e.g., for descriptions):
pub const MAX_STRING_BYTES: usize = 4096; // 4 KB

// Impact: 4x memory per string field
// Consider: Is this needed in hot paths?
```

### Example: Adjusting Rate Limits

```rust
// Current: 200 msg/sec - works for 60 Hz with headroom
pub const GLOBAL_MSG_PER_SEC: u32 = 200;

// For higher tick rates (e.g., 120 Hz):
pub const GLOBAL_MSG_PER_SEC: u32 = 400;

// Impact: 2x potential message processing
// Consider: Can your server handle this load?
```

## Error Codes

Security errors use these codes for logging and debugging:

| Code | Meaning |
|------|---------|
| EMREG002 | Registry index size exceeded |
| EMREG003 | Invalid registry format |
| EMSYNC001 | Cached hash limit exceeded |
| EMSYNC002 | Chunk index out of bounds |
| EMSYNC003 | Resend rate exceeded |
| EMZIP001 | Path traversal detected |
| EMZIP002 | File count exceeded |
| EMZIP003 | Compression ratio exceeded |
