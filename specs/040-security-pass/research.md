# Research: Feature 040 Security Pass

**Date**: 2025-12-19
**Status**: Complete

## Overview

This document captures research findings for the Security Pass feature, resolving technical unknowns identified in the implementation plan.

---

## 1. Bincode Decode Safety

### Decision
Add pre-decode size validation to all `decode()` functions before bincode deserialization.

### Rationale
- Bincode deserializes directly without built-in size limits
- Current `encode()` has post-encode size check, but `decode()` has no equivalent
- Pre-decode validation prevents allocation attacks from oversized payloads

### Implementation Pattern

Current pattern in `crates/plix-common/src/protocol/codec.rs`:
```rust
pub const MAX_PAYLOAD_SIZE: usize = 1389;

pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, ProtocolError> {
    bincode::deserialize(bytes).map_err(|e| ProtocolError::DecodeError(e.to_string()))
}
```

Recommended pattern:
```rust
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, ProtocolError> {
    if bytes.len() > MAX_PAYLOAD_SIZE {
        return Err(ProtocolError::MessageTooLarge(bytes.len()));
    }
    bincode::deserialize(bytes).map_err(|e| ProtocolError::DecodeError(e.to_string()))
}
```

### Alternatives Considered
- **Bincode config limits**: Bincode 2.x supports size limits in config, but requires migrating serialization calls
- **Custom deserializer wrapper**: More complex, less maintainable

### Files to Modify
- `crates/plix-common/src/protocol/codec.rs`: Add pre-decode size check
- `crates/plix-common/src/protocol/messages.rs`: Add per-message-type size limits

---

## 2. Token Bucket Rate Limiting

### Decision
Implement new token bucket rate limiter alongside existing fixed-window anti-cheat system.

### Rationale
- Existing `AntiCheatState` uses fixed-window (60-tick) rate limiting
- Token bucket provides smoother rate enforcement and burst handling
- FR-023 explicitly requires token bucket for mod channel messages
- Can coexist with fixed-window for different use cases

### Implementation Pattern

Existing fixed-window in `crates/plix-server/src/anti_cheat/state.rs`:
```rust
pub struct AntiCheatState {
    window_start_tick: u64,
    input_count: u32,
    // ... per-action counts
}
```

New token bucket pattern:
```rust
pub struct TokenBucket {
    capacity: u32,
    tokens: u32,
    refill_rate: u32,      // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    pub fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed();
        let new_tokens = (elapsed.as_secs_f32() * self.refill_rate as f32) as u32;
        self.tokens = (self.tokens + new_tokens).min(self.capacity);
        self.last_refill = Instant::now();
    }
}
```

### Alternatives Considered
- **Replace fixed-window entirely**: Too risky, existing anti-cheat is battle-tested
- **Leaky bucket**: Less intuitive for burst handling

### Files to Create
- `crates/plix-server/src/security/rate_limiter.rs`: TokenBucket implementation

---

## 3. Fuzz Testing Infrastructure

### Decision
Use cargo-fuzz with libFuzzer backend. Create separate `fuzz/` directory in workspace root.

### Rationale
- cargo-fuzz is the standard Rust fuzzing tool
- libFuzzer is mature and well-integrated with Rust ecosystem
- Corpus-based fuzzing with mutation provides good coverage
- Feature-gated to avoid production impact

### Implementation Pattern

Directory structure:
```text
fuzz/
├── Cargo.toml           # Separate crate for fuzzing
├── fuzz_targets/
│   ├── fuzz_decode_client_message.rs
│   ├── fuzz_decode_server_message.rs
│   └── fuzz_decode_modsync_chunk.rs
└── corpus/
    ├── client_messages/  # Seed corpus
    └── server_messages/
```

Fuzz target template:
```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use plix_common::protocol::{ClientMessage, decode};

fuzz_target!(|data: &[u8]| {
    // Must not panic
    let _ = decode::<ClientMessage>(data);
});
```

### Alternatives Considered
- **AFL**: Less Rust integration, requires more setup
- **Honggfuzz**: Good alternative but cargo-fuzz is more widely used
- **In-tree fuzz crate**: Would pollute main build, feature-gating is cleaner

### Files to Create
- `fuzz/Cargo.toml`: Fuzz crate definition
- `fuzz/fuzz_targets/*.rs`: Individual fuzz targets

---

## 4. Zip Library Safety

### Decision
Add explicit safety checks around existing `zip` crate (v2.2) usage.

### Rationale
- Current code in `crates/plix-mod-distribution/src/installer.rs` extracts without safety checks
- The `zip` crate doesn't automatically prevent path traversal or zip bombs
- Manual validation is required for:
  - Path traversal (`../`, absolute paths)
  - File count limits
  - Decompression ratio limits

### Implementation Pattern

Current unsafe pattern:
```rust
for i in 0..archive.len() {
    let file = archive.by_index(i)?;
    let dest_path = install_dir.join(file.name());
    // No validation!
}
```

Safe pattern:
```rust
const MAX_FILES: usize = 10_000;
const MAX_RATIO: f64 = 20.0;

let mut total_uncompressed: u64 = 0;
let mut file_count: usize = 0;

for i in 0..archive.len() {
    file_count += 1;
    if file_count > MAX_FILES {
        return Err(SecurityError::TooManyFiles);
    }

    let file = archive.by_index(i)?;

    // Path traversal check
    let name = file.name();
    if name.contains("..") || name.starts_with('/') {
        return Err(SecurityError::PathTraversal);
    }

    // Zip bomb check
    total_uncompressed += file.size();
    let compressed = archive.by_index(i)?.compressed_size();
    if total_uncompressed as f64 / compressed as f64 > MAX_RATIO {
        return Err(SecurityError::DecompressionBomb);
    }

    let dest_path = install_dir.join(name);
    // Canonicalize and verify still under install_dir
    let canonical = dest_path.canonicalize()?;
    if !canonical.starts_with(install_dir.canonicalize()?) {
        return Err(SecurityError::PathTraversal);
    }
}
```

### Alternatives Considered
- **Different zip library**: No Rust zip library has built-in safety limits
- **Sandboxed extraction**: Overkill for this use case

### Files to Modify
- `crates/plix-mod-distribution/src/installer.rs`: Add safety checks
- `crates/plix-mod-distribution/src/bundle.rs`: Add validation helpers

---

## 5. Rate-Limited Logging

### Decision
Create a rate-limited logging wrapper using `tracing` with per-category throttling.

### Rationale
- Current tracing usage has no rate limiting
- Log spam from attacks can cause log-based DoS
- Per-category limits allow tuning (security logs more frequent than spam rejection)

### Implementation Pattern

Existing simple pattern:
```rust
use tracing::{info, warn};
warn!("Invalid message from client {}", client_id);
```

Rate-limited wrapper:
```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct RateLimitedLogger {
    last_log: HashMap<String, Instant>,
    suppressed: HashMap<String, u64>,
    min_interval: Duration,
}

impl RateLimitedLogger {
    pub fn warn(&mut self, category: &str, msg: impl AsRef<str>) {
        let now = Instant::now();

        if let Some(last) = self.last_log.get(category) {
            if now.duration_since(*last) < self.min_interval {
                *self.suppressed.entry(category.to_string()).or_insert(0) += 1;
                return;
            }
        }

        let suppressed = self.suppressed.remove(category).unwrap_or(0);
        if suppressed > 0 {
            tracing::warn!("[{}] {} (suppressed {} similar)", category, msg.as_ref(), suppressed);
        } else {
            tracing::warn!("[{}] {}", category, msg.as_ref());
        }

        self.last_log.insert(category.to_string(), now);
    }
}
```

### Alternatives Considered
- **tracing-subscriber filter**: Doesn't support rate limiting, only level filtering
- **External crate (governor)**: Adds dependency for simple use case

### Files to Create
- `crates/plix-server/src/security/observability.rs`: RateLimitedLogger

---

## Summary

| Research Area | Decision | Key Pattern |
|---------------|----------|-------------|
| Bincode decode | Pre-decode size validation | `if bytes.len() > MAX { return Err }` |
| Rate limiting | Token bucket (new) + fixed-window (existing) | `TokenBucket::try_consume()` |
| Fuzzing | cargo-fuzz with libFuzzer | Separate `fuzz/` crate |
| Zip safety | Manual validation around `zip` crate | Path + count + ratio checks |
| Logging | Rate-limited wrapper | Per-category throttling with suppression count |

All technical unknowns are now resolved. Ready for Phase 1 design artifacts.
