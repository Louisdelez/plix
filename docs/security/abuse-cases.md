# Abuse Cases

This document describes known attack patterns and their expected test behavior.

## Decode Abuse

### DC-001: Random Bytes Decode

**Attack**: Send random bytes as a protocol message.
**Expected**: Decode returns `Err(DecodeError)`, no panic.
**Test**: `tests/security/decode_abuse_test.rs::test_random_bytes_decode`

```rust
#[test]
fn test_random_bytes_decode() {
    let random = [0xde, 0xad, 0xbe, 0xef, 0x00, 0xff];
    let result = decode::<ClientMessage>(&random);
    assert!(matches!(result, Err(ProtocolError::DecodeError(_))));
}
```

### DC-002: Truncated Message Decode

**Attack**: Send the first N bytes of a valid message.
**Expected**: Decode returns `Err(DecodeError)`, no panic.
**Test**: `tests/security/decode_abuse_test.rs::test_truncated_message`

```rust
#[test]
fn test_truncated_message() {
    let valid = encode(&ClientMessage::Disconnect).unwrap();
    let truncated = &valid[..valid.len() / 2];
    let result = decode::<ClientMessage>(truncated);
    assert!(matches!(result, Err(ProtocolError::DecodeError(_))));
}
```

### DC-003: Oversized Packet

**Attack**: Send packet larger than MAX_MESSAGE_BYTES.
**Expected**: Rejected with `DecodeSizeLimitExceeded`, strike issued.
**Test**: `tests/security/decode_abuse_test.rs::test_oversized_packet`

```rust
#[test]
fn test_oversized_packet() {
    let oversized = vec![0u8; MAX_MESSAGE_BYTES + 1];
    let result = decode::<ClientMessage>(&oversized);
    assert!(matches!(result, Err(ProtocolError::DecodeSizeLimitExceeded { .. })));
}
```

## Handshake Abuse

### HS-001: Handshake Timeout

**Attack**: Connect but never send ModSetResponse.
**Expected**: Connection closed after HANDSHAKE_TIMEOUT_SECS.
**Test**: `tests/security/handshake_abuse_test.rs::test_handshake_timeout`

```rust
#[test]
fn test_handshake_timeout() {
    let mut tracker = HandshakeTracker::new();
    tracker.register(1, addr).unwrap();

    // Simulate time passing
    std::thread::sleep(Duration::from_secs(HANDSHAKE_TIMEOUT_SECS as u64 + 1));

    let timed_out = tracker.check_timeouts();
    assert!(timed_out.contains(&1));
}
```

### HS-002: Cached Hash Limit Exceeded

**Attack**: Send ModSetResponse with too many cached hashes.
**Expected**: Rejected with `CachedHashLimitExceeded`, connection closed.
**Test**: `tests/security/handshake_abuse_test.rs::test_cached_hash_limit`

```rust
#[test]
fn test_cached_hash_limit() {
    let response = ModSetResponse {
        supports_sync: true,
        cached_hashes: vec!["hash".to_string(); MAX_CACHED_PAYLOAD_HASHES + 1],
        engine_version: None,
    };

    let decision = session.handle_response(response);
    assert!(!decision.allowed);
}
```

### HS-003: Pending Connection Exhaustion

**Attack**: Open MAX_PENDING_PER_SOURCE connections from same IP.
**Expected**: Additional connections rejected.
**Test**: `tests/security/handshake_abuse_test.rs::test_pending_limit`

```rust
#[test]
fn test_pending_limit() {
    let mut tracker = HandshakeTracker::new();
    let addr = "127.0.0.1:12345".parse().unwrap();

    for i in 0..MAX_PENDING_PER_SOURCE {
        assert!(tracker.register(i as u64, addr).is_ok());
    }

    // Next should fail
    assert!(matches!(
        tracker.register(999, addr),
        Err(HandshakeError::TooManyPending)
    ));
}
```

## Payload Sync Abuse

### PS-001: Invalid Chunk Index

**Attack**: Send PayloadAck with chunk_index > chunk_count.
**Expected**: Session aborted, strike issued.
**Test**: `tests/security/payload_sync_abuse_test.rs::test_invalid_chunk_index`

### PS-002: Resend Spam

**Attack**: Request resends exceeding MAX_RESEND_PER_WINDOW.
**Expected**: Session aborted, strike issued.
**Test**: `tests/security/payload_sync_abuse_test.rs::test_resend_spam`

### PS-003: Hash Mismatch

**Attack**: Report hash mismatch after receiving valid chunks.
**Expected**: Strike issued, session aborted.
**Test**: `tests/security/payload_sync_abuse_test.rs::test_hash_mismatch`

## Parser Abuse

### PA-001: Large Registry Index

**Attack**: Create index.json larger than MAX_INDEX_JSON_BYTES.
**Expected**: Rejected with EMREG002 before parsing.
**Test**: `tests/security/parser_abuse_test.rs::test_large_index`

```rust
#[test]
fn test_large_index() {
    let large = "x".repeat(MAX_INDEX_JSON_BYTES + 1);
    let result = RegistryIndex::from_json(&large);
    assert!(result.is_err());
}
```

### PA-002: Path Traversal in Zip

**Attack**: Bundle with `../../../etc/passwd` entry.
**Expected**: Rejected before extraction.
**Test**: `tests/security/parser_abuse_test.rs::test_path_traversal`

```rust
#[test]
fn test_path_traversal() {
    // Create zip with traversal path
    let bundle = create_malicious_bundle("../../../etc/passwd", b"malicious");
    let result = ModBundle::from_file(&bundle);
    assert!(result.is_err());
}
```

### PA-003: Zip Bomb

**Attack**: Bundle with 1000:1 compression ratio (highly compressed zeros).
**Expected**: Rejected with EMZIP003 before extraction.
**Test**: `tests/security/parser_abuse_test.rs::test_zip_bomb`

```rust
#[test]
fn test_zip_bomb() {
    // Create zip with extreme ratio
    let bundle = create_compressed_bundle(1_000_000_000); // 1GB uncompressed
    let result = ModBundle::from_file(&bundle);
    assert!(result.is_err()); // Ratio exceeds MAX_ZIP_RATIO
}
```

### PA-004: Too Many Files

**Attack**: Bundle with more than MAX_ZIP_FILES entries.
**Expected**: Rejected with EMZIP002 before extraction.
**Test**: `tests/security/parser_abuse_test.rs::test_too_many_files`

## Rate Limit Abuse

### RL-001: Message Flood

**Attack**: Send more than GLOBAL_MSG_PER_SEC messages.
**Expected**: Excess messages dropped, strike issued.
**Test**: Integration test with rate limiter.

### RL-002: Type-Specific Flood

**Attack**: Send bursts of expensive messages (block edits).
**Expected**: Per-type rate limit triggers, strike issued.
**Test**: Integration test with specific message type.

## Test Matrix

| Test ID | Category | Limit Tested | Expected Result |
|---------|----------|--------------|-----------------|
| DC-001 | Decode | N/A | Error, no panic |
| DC-002 | Decode | N/A | Error, no panic |
| DC-003 | Decode | MAX_MESSAGE_BYTES | Size error |
| HS-001 | Handshake | HANDSHAKE_TIMEOUT_SECS | Timeout cleanup |
| HS-002 | Handshake | MAX_CACHED_PAYLOAD_HASHES | Reject |
| HS-003 | Handshake | MAX_PENDING_PER_SOURCE | Reject |
| PS-001 | Payload | Chunk count | Abort session |
| PS-002 | Payload | MAX_RESEND_PER_WINDOW | Abort session |
| PS-003 | Payload | Hash verification | Strike |
| PA-001 | Parser | MAX_INDEX_JSON_BYTES | Reject |
| PA-002 | Parser | Path safety | Reject |
| PA-003 | Parser | MAX_ZIP_RATIO | Reject |
| PA-004 | Parser | MAX_ZIP_FILES | Reject |
| RL-001 | Rate | GLOBAL_MSG_PER_SEC | Drop + strike |
| RL-002 | Rate | Per-type limits | Drop + strike |
