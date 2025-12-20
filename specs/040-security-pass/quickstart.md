# Quickstart: Feature 040 Security Pass

**Date**: 2025-12-19

## Overview

This guide covers how to work with the security subsystem, run fuzz tests, and understand security limits.

---

## 1. Understanding Security Limits

All security limits are defined in `plix-common/src/limits.rs`. These are the authoritative values used throughout the codebase.

### Key Limits Reference

| Category | Constant | Value | Purpose |
|----------|----------|-------|---------|
| Protocol | `MAX_PACKET_BYTES` | 64 KB | Maximum raw packet size |
| Protocol | `MAX_STRING_BYTES` | 1 KB | Maximum string field length |
| Protocol | `MAX_LIST_LEN` | 256 | Maximum list/array elements |
| Handshake | `HANDSHAKE_TIMEOUT_SECS` | 10s | Time to complete handshake |
| Handshake | `MAX_CACHED_PAYLOAD_HASHES` | 256 | Max hashes client can send |
| Payload | `TRANSFER_TIMEOUT_SECS` | 30s | Time to complete transfer |
| Rate | `GLOBAL_MSG_PER_SEC` | 200 | Max messages per client per second |
| Parser | `MAX_INDEX_JSON_BYTES` | 5 MB | Registry index size limit |
| Parser | `MAX_ZIP_FILES` | 10,000 | Max files in bundle |
| Parser | `MAX_ZIP_RATIO` | 20:1 | Max decompression ratio |
| Strikes | `STRIKES_BEFORE_DISCONNECT` | 5 | Violations before kick |

### Using Limits in Code

```rust
use plix_common::limits::{MAX_STRING_BYTES, MAX_LIST_LEN};

fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name.len() > MAX_STRING_BYTES {
        return Err(ValidationError::StringTooLong);
    }
    Ok(())
}

fn validate_items(items: &[Item]) -> Result<(), ValidationError> {
    if items.len() > MAX_LIST_LEN {
        return Err(ValidationError::ListTooLong);
    }
    Ok(())
}
```

---

## 2. Running Fuzz Tests

### Prerequisites

Install cargo-fuzz:
```bash
cargo install cargo-fuzz
```

### Available Fuzz Targets

| Target | What it tests |
|--------|---------------|
| `fuzz_decode_client_message` | ClientMessage decode from random bytes |
| `fuzz_decode_server_message` | ServerMessage decode from random bytes |
| `fuzz_decode_modsync_chunk` | PayloadChunk decode from random bytes |

### Running Fuzz Tests

```bash
# Enter the fuzz directory
cd fuzz

# Run a specific target (runs until stopped with Ctrl+C)
cargo fuzz run fuzz_decode_client_message

# Run with timeout (useful for CI)
cargo fuzz run fuzz_decode_client_message -- -max_total_time=300

# Run with specific corpus
cargo fuzz run fuzz_decode_client_message corpus/client_messages/

# List all targets
cargo fuzz list
```

### Interpreting Results

**Success output:**
```
#12345    DONE   cov: 342 ft: 1234 corp: 56
```
- No panics or crashes after many iterations = PASS

**Failure output:**
```
==12345==ERROR: libFuzzer: deadly signal
SUMMARY: libFuzzer: deadly-signal
```
- A crash was found. The failing input is saved to `fuzz/artifacts/`

### Reproducing Crashes

```bash
# Reproduce a crash
cargo fuzz run fuzz_decode_client_message fuzz/artifacts/fuzz_decode_client_message/crash-abc123

# Minimize the crash input
cargo fuzz tmin fuzz_decode_client_message fuzz/artifacts/fuzz_decode_client_message/crash-abc123
```

---

## 3. Security Metrics & Logging

### Available Metrics

| Metric | Meaning |
|--------|---------|
| `invalid_messages_total` | Malformed messages received |
| `disconnects_strikes_total` | Clients kicked for violations |
| `rate_limited_total` | Messages dropped due to rate limiting |
| `payload_sync_aborts_total` | Payload transfers aborted |
| `handshake_timeouts_total` | Handshakes that timed out |
| `registry_parse_failures_total` | Failed registry index parses |
| `zip_safety_rejections_total` | Bundles rejected for zip safety |

### Accessing Metrics

```rust
use plix_server::security::SecurityMetrics;

let metrics = SecurityMetrics::new();

// Record events
metrics.record_invalid_message();
metrics.record_rate_limited();

// Get snapshot for reporting
let snapshot = metrics.snapshot();
println!("Invalid messages: {}", snapshot.invalid_messages_total);
println!("Disconnects: {}", snapshot.disconnects_strikes_total);
```

### Debug Logging

Enable detailed security logging:

```rust
use plix_server::security::RateLimitedLogger;

let logger = RateLimitedLogger::new()
    .with_debug_mode(true);  // Enable verbose output

logger.debug("handshake", "Received ModSetResponse with 15 cached hashes");
```

Or via environment:
```bash
PLIX_SECURITY_DEBUG=1 cargo run --bin plix-server
```

---

## 4. Running Abuse Tests

### Test Categories

| Test File | Coverage |
|-----------|----------|
| `decode_abuse_test.rs` | Truncated/random bytes, oversized messages |
| `handshake_abuse_test.rs` | Timeout, rapid reconnect, cache hash abuse |
| `payload_sync_abuse_test.rs` | Invalid chunks, resend spam, hash mismatch |
| `parser_abuse_test.rs` | Large files, zip bombs, path traversal |

### Running Tests

```bash
# Run all security tests
cargo test --test security

# Run specific test file
cargo test --test security::decode_abuse_test

# Run with output
cargo test --test security -- --nocapture
```

### Writing New Abuse Tests

```rust
#[test]
fn test_oversized_string_rejected() {
    let mut tracker = StrikeTracker::new(1);

    // Create message with oversized name
    let oversized = "x".repeat(MAX_STRING_BYTES + 1);
    let bytes = encode_test_message_with_name(&oversized);

    // Attempt decode
    let result = decode::<ClientMessage>(&bytes);

    // Verify rejection
    assert!(matches!(result, Err(ProtocolError::StringTooLong { .. })));
}

#[test]
fn test_strike_accumulation() {
    let mut tracker = StrikeTracker::new(1);

    // Record strikes up to threshold
    for _ in 0..STRIKES_BEFORE_DISCONNECT - 1 {
        assert!(!tracker.record_strike(StrikeCategory::DecodeError));
    }

    // Next strike should trigger disconnect
    assert!(tracker.record_strike(StrikeCategory::DecodeError));
    assert!(tracker.should_disconnect());
}
```

---

## 5. Common Tasks

### Adding a New Limit

1. Add constant to `plix-common/src/limits.rs`:
   ```rust
   pub const MAX_NEW_THING: usize = 100;
   ```

2. Use in validation code:
   ```rust
   use plix_common::limits::MAX_NEW_THING;

   if value > MAX_NEW_THING {
       return Err(LimitExceeded);
   }
   ```

3. Document in `docs/security/limits.md`

4. Add test in `tests/security/`

### Adding a New Strike Category

1. Add variant to `StrikeCategory` enum:
   ```rust
   pub enum StrikeCategory {
       // existing...
       NewAbuse,
   }
   ```

2. Update metrics if needed:
   ```rust
   metrics.record_disconnect(StrikeCategory::NewAbuse);
   ```

3. Add test case

### Adding a New Fuzz Target

1. Create `fuzz/fuzz_targets/fuzz_new_target.rs`:
   ```rust
   #![no_main]
   use libfuzzer_sys::fuzz_target;

   fuzz_target!(|data: &[u8]| {
       let _ = parse_new_thing(data);
   });
   ```

2. Add to `fuzz/Cargo.toml`:
   ```toml
   [[bin]]
   name = "fuzz_new_target"
   path = "fuzz_targets/fuzz_new_target.rs"
   test = false
   doc = false
   ```

3. Add seed corpus to `fuzz/corpus/new_target/`

4. Run: `cargo fuzz run fuzz_new_target`

---

## 6. Troubleshooting

### "Rate limited" logs appearing frequently

- Check if legitimate traffic patterns exceed limits
- Consider adjusting `GLOBAL_MSG_PER_SEC` if needed
- Verify per-type limits aren't too restrictive

### Fuzz test times out

- libFuzzer timeout (not a bug): increase with `-max_total_time`
- If decode itself times out: indicates O(n²) or worse complexity

### Strike count doesn't decay

- Verify `decay()` is being called periodically
- Check `STRIKES_DECAY_SECS` configuration

### Zip extraction fails legitimate bundles

- Check bundle size against `MAX_BUNDLE_BYTES`
- Verify file count under `MAX_ZIP_FILES`
- Check compression ratio against `MAX_ZIP_RATIO`

---

## 7. Documentation Links

- [Threat Model](../../docs/security/threat-model.md) - Attack surfaces and mitigations
- [Limits Reference](../../docs/security/limits.md) - All security limits explained
- [Fuzzing Guide](../../docs/security/fuzzing.md) - Detailed fuzzing instructions
- [Abuse Cases](../../docs/security/abuse-cases.md) - Known attack patterns and tests
