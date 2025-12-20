//! Security abuse test suite for Feature 040
//!
//! These tests validate that the security subsystem properly handles abuse cases:
//! - Handshake timeout and cleanup
//! - Decode error accumulation and disconnect
//! - Rate limiting with strike tracking
//! - Payload sync abuse detection
//! - Cached hash limit enforcement

use std::net::SocketAddr;
use std::sync::Arc;

use plix_common::limits::{
    MAX_CACHED_PAYLOAD_HASHES, MAX_PACKET_BYTES, STRIKES_BEFORE_DISCONNECT,
};
use plix_common::protocol::{encode, ClientMessage, PlayerInput};
use plix_common::time::Tick;
use plix_common::types::InputSeq;
use plix_server::netloop::{ProcessResult, ServerNetLoop};
use plix_server::security::{
    HandshakeError, HandshakeState, HandshakeTracker, RateLimitedMessageType, SecurityContext,
    SecurityMetrics, StrikeCategory,
};

/// Create a test PlayerInput
fn test_input() -> PlayerInput {
    PlayerInput {
        seq: InputSeq::default(),
        tick: Tick(0),
        move_forward: 0.0,
        move_right: 0.0,
        jump: false,
        crouch: false,
        attack: false,
        yaw: 0.0,
        pitch: 0.0,
        rtt_nonce: 0,
    }
}

fn test_addr(port: u16) -> SocketAddr {
    format!("127.0.0.1:{}", port).parse().unwrap()
}

// =============================================================================
// Handshake Abuse Tests
// =============================================================================

#[test]
fn test_handshake_per_source_limit() {
    let mut tracker = HandshakeTracker::new();
    let addr = test_addr(12345);

    // Should allow up to MAX_PENDING_PER_SOURCE
    for i in 0..plix_common::limits::MAX_PENDING_PER_SOURCE {
        assert!(
            tracker.register(i as u64, addr).is_ok(),
            "should allow connection {}",
            i
        );
    }

    // Next should fail
    let result = tracker.register(999, addr);
    assert!(
        matches!(result, Err(HandshakeError::TooManyPending)),
        "should reject excess pending connections"
    );
}

#[test]
fn test_handshake_different_sources_independent() {
    let mut tracker = HandshakeTracker::new();
    let addr1 = test_addr(12345);
    let addr2 = test_addr(12346);

    // Fill up addr1
    for i in 0..plix_common::limits::MAX_PENDING_PER_SOURCE {
        tracker.register(i as u64, addr1).unwrap();
    }

    // addr2 should still work
    assert!(
        tracker
            .register(100, addr2)
            .is_ok(),
        "different source should have independent limit"
    );
}

#[test]
fn test_handshake_state_transitions() {
    let mut tracker = HandshakeTracker::new();
    let addr = test_addr(12345);

    tracker.register(1, addr).unwrap();

    // Valid transition: AwaitingConnect -> AwaitingModSet
    assert!(tracker
        .update_state(1, HandshakeState::AwaitingModSet)
        .is_ok());

    // Invalid transition: AwaitingModSet -> AwaitingConnect (can't go backwards)
    let result = tracker.update_state(1, HandshakeState::AwaitingConnect);
    assert!(matches!(
        result,
        Err(HandshakeError::InvalidTransition { .. })
    ));
}

#[test]
fn test_cached_hash_limit_exceeded() {
    let mut tracker = HandshakeTracker::new();
    let addr = test_addr(12345);

    tracker.register(1, addr).unwrap();

    // Within limit should succeed
    assert!(tracker
        .set_cached_hashes_count(1, MAX_CACHED_PAYLOAD_HASHES)
        .is_ok());

    // Exceeds limit should fail
    let result = tracker.set_cached_hashes_count(1, MAX_CACHED_PAYLOAD_HASHES + 1);
    assert!(matches!(
        result,
        Err(HandshakeError::CacheHashLimitExceeded { .. })
    ));
}

// =============================================================================
// Decode Abuse Tests
// =============================================================================

#[test]
fn test_decode_random_bytes_increments_strikes() {
    let mut ctx = SecurityContext::new(1);

    // Should not disconnect on first error
    assert!(!ctx.record_decode_error());
    assert_eq!(ctx.strike_count(), 1);
}

#[test]
fn test_decode_errors_accumulate_to_disconnect() {
    let mut ctx = SecurityContext::new(1);

    // Record strikes up to threshold - 1
    for i in 0..STRIKES_BEFORE_DISCONNECT - 1 {
        let should_dc = ctx.record_decode_error();
        assert!(!should_dc, "should not disconnect at strike {}", i + 1);
    }

    // Next strike should trigger disconnect
    assert!(ctx.record_decode_error(), "should disconnect at threshold");
    assert!(ctx.should_disconnect());
    assert!(ctx.disconnect_reason().is_some());
}

#[test]
fn test_decode_error_via_netloop() {
    let metrics = Arc::new(SecurityMetrics::new());
    let mut netloop = ServerNetLoop::with_metrics(metrics.clone());

    let addr = test_addr(12345);
    netloop.add_connection(addr).unwrap();

    // Send invalid bytes
    let invalid_bytes = [0xDE, 0xAD, 0xBE, 0xEF];

    for _ in 0..STRIKES_BEFORE_DISCONNECT {
        let result = netloop.process_raw_bytes(addr, &invalid_bytes);
        match result {
            ProcessResult::Dropped => continue,
            ProcessResult::Disconnect { .. } => break,
            ProcessResult::Ok(_) => panic!("should not decode random bytes"),
        }
    }

    // Metrics should reflect the invalid messages
    let snapshot = metrics.snapshot();
    assert!(
        snapshot.invalid_messages_total >= 1,
        "should record invalid messages"
    );
}

// =============================================================================
// Size Violation Tests
// =============================================================================

#[test]
fn test_oversized_packet_dropped() {
    let metrics = Arc::new(SecurityMetrics::new());
    let mut netloop = ServerNetLoop::with_metrics(metrics.clone());

    let addr = test_addr(12345);
    netloop.add_connection(addr).unwrap();

    // Create oversized packet
    let oversized = vec![0u8; MAX_PACKET_BYTES + 1];
    let result = netloop.process_raw_bytes(addr, &oversized);

    assert!(
        matches!(result, ProcessResult::Dropped),
        "oversized packet should be dropped"
    );

    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.invalid_messages_total, 1);
}

#[test]
fn test_oversized_packets_accumulate_strikes() {
    let metrics = Arc::new(SecurityMetrics::new());
    let mut netloop = ServerNetLoop::with_metrics(metrics.clone());

    let addr = test_addr(12345);
    netloop.add_connection(addr).unwrap();

    let oversized = vec![0u8; MAX_PACKET_BYTES + 1];

    // Send enough oversized packets to trigger disconnect
    for _ in 0..STRIKES_BEFORE_DISCONNECT {
        let result = netloop.process_raw_bytes(addr, &oversized);
        if matches!(result, ProcessResult::Disconnect { .. }) {
            // Should disconnect at threshold
            let snapshot = metrics.snapshot();
            assert!(
                snapshot.disconnects_strikes_total >= 1,
                "should record disconnect"
            );
            return;
        }
    }

    panic!("should have disconnected after {} oversized packets", STRIKES_BEFORE_DISCONNECT);
}

// =============================================================================
// Rate Limiting Tests
// =============================================================================

#[test]
fn test_rate_limit_global_allows_burst() {
    let mut ctx = SecurityContext::new(1);

    // Should allow up to GLOBAL_MSG_PER_SEC messages
    let mut allowed = 0;
    for _ in 0..plix_common::limits::GLOBAL_MSG_PER_SEC {
        if ctx.check_rate_limit(RateLimitedMessageType::Global) {
            allowed += 1;
        }
    }

    assert_eq!(
        allowed,
        plix_common::limits::GLOBAL_MSG_PER_SEC,
        "should allow full burst"
    );
}

#[test]
fn test_rate_limit_blocks_excess() {
    let mut ctx = SecurityContext::new(1);

    // Consume all tokens
    for _ in 0..plix_common::limits::GLOBAL_MSG_PER_SEC {
        ctx.check_rate_limit(RateLimitedMessageType::Global);
    }

    // Next should be blocked
    assert!(
        !ctx.check_rate_limit(RateLimitedMessageType::Global),
        "should block after burst exhausted"
    );

    // Should have recorded a strike
    assert_eq!(ctx.strike_count(), 1, "should record strike for rate limit");
}

#[test]
fn test_rate_limit_chat_has_lower_limit() {
    let mut ctx = SecurityContext::new(1);

    // Chat has 10 burst capacity
    for _ in 0..10 {
        assert!(
            ctx.check_rate_limit(RateLimitedMessageType::Chat),
            "chat should allow 10 messages"
        );
    }

    // 11th should be blocked
    assert!(
        !ctx.check_rate_limit(RateLimitedMessageType::Chat),
        "chat should block 11th message"
    );
}

#[test]
fn test_rate_limit_via_netloop() {
    let metrics = Arc::new(SecurityMetrics::new());
    let mut netloop = ServerNetLoop::with_metrics(metrics.clone());

    let addr = test_addr(12345);
    netloop.add_connection(addr).unwrap();

    // Create a valid message
    let msg = ClientMessage::Input(test_input());
    let bytes = encode(&msg).unwrap();

    // Send many messages quickly
    let mut blocked = 0;
    for _ in 0..plix_common::limits::GLOBAL_MSG_PER_SEC + 50 {
        let result = netloop.process_raw_bytes(addr, &bytes);
        if matches!(result, ProcessResult::Dropped) {
            blocked += 1;
        }
    }

    assert!(blocked > 0, "should have blocked some messages");

    let snapshot = metrics.snapshot();
    assert!(snapshot.rate_limited_total > 0, "should record rate limited");
}

// =============================================================================
// Payload Sync Abuse Tests (via SecurityContext)
// =============================================================================

#[test]
fn test_payload_sync_abuse_strike() {
    let mut ctx = SecurityContext::new(1);

    // Record payload sync abuse
    assert!(!ctx.record_payload_sync_abuse());
    assert_eq!(ctx.strike_count(), 1);
}

#[test]
fn test_payload_sync_abuse_accumulates_to_disconnect() {
    let mut ctx = SecurityContext::new(1);

    for _ in 0..STRIKES_BEFORE_DISCONNECT - 1 {
        ctx.record_payload_sync_abuse();
    }

    assert!(ctx.record_payload_sync_abuse(), "should disconnect at threshold");
    assert!(ctx.should_disconnect());
}

// =============================================================================
// Mixed Abuse Tests
// =============================================================================

#[test]
fn test_different_violations_accumulate() {
    let mut ctx = SecurityContext::new(1);

    // Mix of different violations
    ctx.record_decode_error();
    ctx.record_size_violation();
    ctx.record_payload_sync_abuse();
    ctx.record_handshake_abuse();

    assert_eq!(ctx.strike_count(), 4);
    assert!(!ctx.should_disconnect());

    // One more should trigger disconnect
    assert!(ctx.record_decode_error());
    assert!(ctx.should_disconnect());
}

#[test]
fn test_netloop_violation_recording() {
    let mut netloop = ServerNetLoop::new();
    let addr = test_addr(12345);

    netloop.add_connection(addr).unwrap();

    // Record violations
    for _ in 0..STRIKES_BEFORE_DISCONNECT - 1 {
        let result = netloop.record_violation(&addr, StrikeCategory::PayloadSyncAbuse);
        assert!(result.is_none(), "should not disconnect yet");
    }

    // Final violation should return disconnect reason
    let result = netloop.record_violation(&addr, StrikeCategory::PayloadSyncAbuse);
    assert!(result.is_some(), "should return disconnect reason");
}

// =============================================================================
// Metrics Tests
// =============================================================================

#[test]
fn test_metrics_tracking() {
    let metrics = SecurityMetrics::new();

    metrics.record_invalid_message();
    metrics.record_invalid_message();
    metrics.record_rate_limited();
    metrics.record_payload_sync_abort();
    metrics.record_handshake_timeout();
    metrics.record_disconnect(StrikeCategory::DecodeError);

    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.invalid_messages_total, 2);
    assert_eq!(snapshot.rate_limited_total, 1);
    assert_eq!(snapshot.payload_sync_aborts_total, 1);
    assert_eq!(snapshot.handshake_timeouts_total, 1);
    assert_eq!(snapshot.disconnects_strikes_total, 1);
    assert_eq!(
        snapshot.strikes_by_category.get(&StrikeCategory::DecodeError),
        Some(&1)
    );
}

// =============================================================================
// Integration: Complete Abuse Scenario
// =============================================================================

#[test]
fn test_complete_abuse_scenario() {
    let metrics = Arc::new(SecurityMetrics::new());
    let mut netloop = ServerNetLoop::with_metrics(metrics.clone());

    let addr = test_addr(12345);
    netloop.add_connection(addr).unwrap();

    // Phase 1: Send some valid messages
    let valid_msg = ClientMessage::Input(test_input());
    let valid_bytes = encode(&valid_msg).unwrap();

    for _ in 0..5 {
        let result = netloop.process_raw_bytes(addr, &valid_bytes);
        assert!(
            matches!(result, ProcessResult::Ok(_)),
            "valid messages should pass"
        );
    }

    // Phase 2: Mix in some invalid messages
    let invalid_bytes = [0xFF, 0xFF, 0xFF, 0xFF];
    for _ in 0..3 {
        let result = netloop.process_raw_bytes(addr, &invalid_bytes);
        assert!(
            !matches!(result, ProcessResult::Ok(_)),
            "invalid messages should not pass"
        );
    }

    // Phase 3: More valid messages should still work (not disconnected yet)
    let result = netloop.process_raw_bytes(addr, &valid_bytes);
    assert!(
        matches!(result, ProcessResult::Ok(_)),
        "should still accept valid messages"
    );

    // Check metrics reflect the activity
    let snapshot = metrics.snapshot();
    assert!(
        snapshot.invalid_messages_total >= 3,
        "should have recorded invalid messages"
    );
}

#[test]
fn test_connection_cleanup_removes_security_context() {
    let mut netloop = ServerNetLoop::new();
    let addr = test_addr(12345);

    netloop.add_connection(addr).unwrap();
    assert!(netloop.get_security_context(&addr).is_some());

    netloop.remove_connection(&addr);
    assert!(netloop.get_security_context(&addr).is_none());
}

// =============================================================================
// Decay Tests
// =============================================================================

#[test]
fn test_strike_decay_does_not_reset_immediately() {
    let mut ctx = SecurityContext::new(1);

    ctx.record_decode_error();
    ctx.record_decode_error();
    assert_eq!(ctx.strike_count(), 2);

    // Decay immediately - should not reduce (not enough time passed)
    ctx.decay();
    assert!(
        ctx.strike_count() >= 1,
        "immediate decay should not fully reset"
    );
}
