//! Integration tests for the mod API core
//!
//! These tests exercise the full workflow with a dummy mod.

use plix_mod_core::api::net::{ModChannel, ModRateLimiters, RATE_LIMIT_MSG_PER_SEC};
use plix_mod_core::api::timers::{GlobalTimers, MAX_TIMERS_PER_MOD, MIN_TIMER_INTERVAL_MS};
use plix_mod_core::capabilities::{Capability, CapabilityPolicy};
use plix_mod_core::errors::ErrorCode;
use plix_mod_core::events::{EventBus, EventContext, EventType, GameEvent};
use plix_mod_core::manifest::ModManifest;
use plix_mod_core::registry::{ModRegistry, ERROR_THRESHOLD};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Load the dummy mod manifest from fixtures
fn load_dummy_manifest() -> ModManifest {
    let manifest_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dummy_mod/mod.toml");
    ModManifest::load_from_file(&manifest_path).expect("Failed to load dummy mod manifest")
}

// ============================================================================
// T103: Event subscription and receipt
// ============================================================================

#[test]
fn test_integration_event_subscription() {
    let manifest = load_dummy_manifest();
    let mut registry = ModRegistry::new();

    // Load the mod
    registry.load(manifest).expect("Failed to load mod");
    assert!(registry.is_enabled("dummy-mod"));

    // Subscribe to events
    {
        let ctx = registry.get_mut("dummy-mod").unwrap();
        ctx.subscribe(EventType::PlayerJoin);
        ctx.subscribe(EventType::PlayerChat);
        assert!(ctx.is_subscribed(EventType::PlayerJoin));
        assert!(ctx.is_subscribed(EventType::PlayerChat));
        assert!(!ctx.is_subscribed(EventType::BlockPlaced));
    }

    // Create event bus and handler
    let mut bus = EventBus::new();
    let call_count = Arc::new(AtomicU32::new(0));
    let call_count_clone = call_count.clone();

    bus.set_handler(Box::new(move |mod_id, _event| {
        assert_eq!(mod_id, "dummy-mod");
        call_count_clone.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }));

    // Emit events
    bus.emit(GameEvent::player_join(1, "TestPlayer", 1));
    bus.emit(GameEvent::player_chat(1, "Hello!", 2));

    // Dispatch
    let cancelled = bus.dispatch(&mut registry);
    assert_eq!(cancelled, 0);
    assert_eq!(call_count.load(Ordering::SeqCst), 2);
}

// ============================================================================
// T104: Event cancellation
// ============================================================================

#[test]
fn test_integration_event_cancellation_allowed() {
    // Test that chat events can be cancelled with the right capability
    let mut ctx = EventContext::new();
    ctx.current_event = Some(GameEvent::player_chat(1, "test", 1));

    // Dummy mod has event_cancel_chat capability
    let caps = Capability::EVENT_CANCEL_CHAT;
    let result = ctx.cancel_event(caps);
    assert!(result.is_ok());
    assert!(ctx.current_event.as_ref().unwrap().cancelled);
}

#[test]
fn test_integration_event_cancellation_denied() {
    // Test that block events cannot be cancelled without capability
    let mut ctx = EventContext::new();
    ctx.current_event = Some(GameEvent::block_placed(Some(1), glam::IVec3::ZERO, 1, 1));

    // Dummy mod does NOT have event_cancel_blocks capability
    let caps = Capability::EVENT_CANCEL_CHAT; // Wrong capability
    let result = ctx.cancel_event(caps);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, ErrorCode::PermissionDenied);
}

// ============================================================================
// T105: Permission denied scenarios
// ============================================================================

#[test]
fn test_integration_permission_denied_scenarios() {
    let mut policy = CapabilityPolicy::permissive();

    // Deny world write to a specific mod
    policy.deny_from_mod("restricted-mod", Capability::WORLD_WRITE);

    let mut registry = ModRegistry::with_policy(policy);

    // Create a mod manifest with world write
    let toml = r#"
id = "restricted-mod"
name = "Restricted Mod"
version = "1.0.0"
api_version = 1

[capabilities]
world_read = true
world_write = true
"#;
    let manifest = ModManifest::parse(toml).unwrap();
    registry.load(manifest).unwrap();

    // Check effective capabilities
    let ctx = registry.get("restricted-mod").unwrap();
    assert!(ctx.has_capability(Capability::WORLD_READ));
    assert!(!ctx.has_capability(Capability::WORLD_WRITE)); // Denied by policy
}

#[test]
fn test_integration_capability_require_check() {
    use plix_mod_core::capabilities::require;

    // Test require helper
    let caps = Capability::WORLD_READ | Capability::ENTITY_READ;

    assert!(require(caps, Capability::WORLD_READ).is_ok());
    assert!(require(caps, Capability::ENTITY_READ).is_ok());

    let result = require(caps, Capability::WORLD_WRITE);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, ErrorCode::PermissionDenied);
}

// ============================================================================
// T106: Rate limiting
// ============================================================================

#[test]
fn test_integration_rate_limiting() {
    let mut limiters = ModRateLimiters::new();
    let limiter = limiters.get_or_create("test-mod");

    // Should allow up to RATE_LIMIT_MSG_PER_SEC messages
    for i in 0..RATE_LIMIT_MSG_PER_SEC {
        assert!(limiter.try_consume(), "Message {} should be allowed", i + 1);
    }

    // Next message should be rate limited
    assert!(!limiter.try_consume(), "Should be rate limited");
}

#[test]
fn test_integration_rate_limiter_per_mod() {
    let mut limiters = ModRateLimiters::new();

    // Exhaust mod1's tokens
    {
        let limiter1 = limiters.get_or_create("mod1");
        for _ in 0..RATE_LIMIT_MSG_PER_SEC {
            limiter1.try_consume();
        }
        assert!(!limiter1.try_consume());
    }

    // mod2 should still have tokens
    {
        let limiter2 = limiters.get_or_create("mod2");
        assert!(limiter2.try_consume());
    }
}

// ============================================================================
// T107: Timer limits
// ============================================================================

#[test]
fn test_integration_timer_limits() {
    let mut timers = GlobalTimers::new();
    let mod_timers = timers.get_or_create("test-mod");

    // Fill to capacity
    for i in 0..MAX_TIMERS_PER_MOD {
        let result = mod_timers.set_timeout(100, i as u32);
        assert!(result.is_ok(), "Timer {} should succeed", i);
    }

    // Next timer should fail
    let result = mod_timers.set_timeout(100, 999);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, ErrorCode::RateLimited);
}

#[test]
fn test_integration_timer_min_interval_clamping() {
    let mut timers = GlobalTimers::new();
    let mod_timers = timers.get_or_create("test-mod");

    // Set timer with very low interval
    let handle = mod_timers.set_timeout(10, 1).unwrap();
    let timer = mod_timers.get(handle).unwrap();

    // Should be clamped to minimum
    assert_eq!(timer.remaining_ms, MIN_TIMER_INTERVAL_MS);
}

#[test]
fn test_integration_timer_tick_and_fire() {
    let mut timers = GlobalTimers::new();
    timers.get_or_create("mod1").set_timeout(100, 1).unwrap();
    timers.get_or_create("mod2").set_timeout(200, 2).unwrap();

    // Tick 100ms - first timer fires
    let fired = timers.tick(100);
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].0, "mod1");
    assert_eq!(fired[0].1, 1); // callback_id

    // Tick another 100ms - second timer fires
    let fired = timers.tick(100);
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].0, "mod2");
}

// ============================================================================
// T108: Auto-disable after 5 consecutive errors
// ============================================================================

#[test]
fn test_integration_auto_disable_after_errors() {
    let manifest = load_dummy_manifest();
    let mut registry = ModRegistry::new();
    registry.load(manifest).unwrap();

    // Subscribe to an event
    registry
        .get_mut("dummy-mod")
        .unwrap()
        .subscribe(EventType::PlayerJoin);

    let mut bus = EventBus::new();

    // Handler always fails
    bus.set_handler(Box::new(|_, _| Err("Intentional test error".into())));

    // Emit exactly ERROR_THRESHOLD events
    for i in 0..ERROR_THRESHOLD {
        bus.emit(GameEvent::player_join(
            i as u64,
            format!("P{}", i),
            i as u64,
        ));
    }

    // Dispatch - should trigger auto-disable
    bus.dispatch(&mut registry);

    // Mod should be disabled
    assert!(!registry.is_enabled("dummy-mod"));

    // Check disable reason
    let ctx = registry.get("dummy-mod").unwrap();
    assert!(ctx.disable_reason.is_some());
    assert!(ctx
        .disable_reason
        .as_ref()
        .unwrap()
        .contains("consecutive errors"));
}

#[test]
fn test_integration_error_reset_on_success() {
    let manifest = load_dummy_manifest();
    let mut registry = ModRegistry::new();
    registry.load(manifest).unwrap();

    let ctx = registry.get_mut("dummy-mod").unwrap();

    // Record some errors (but not enough to disable)
    for _ in 0..(ERROR_THRESHOLD - 1) {
        assert!(!ctx.record_error());
    }
    assert_eq!(ctx.error_count, ERROR_THRESHOLD - 1);

    // Success resets the counter
    ctx.record_success();
    assert_eq!(ctx.error_count, 0);

    // Now errors need to start over
    for _ in 0..(ERROR_THRESHOLD - 1) {
        assert!(!ctx.record_error());
    }
    // Mod is still enabled
    assert_eq!(ctx.error_count, ERROR_THRESHOLD - 1);
}

// ============================================================================
// Manifest validation tests
// ============================================================================

#[test]
fn test_integration_manifest_valid_loads() {
    let manifest = load_dummy_manifest();
    assert_eq!(manifest.id, "dummy-mod");
    assert_eq!(manifest.name, "Dummy Test Mod");
    assert_eq!(manifest.api_version, 1);
    assert!(manifest.capabilities.world_read);
    assert!(manifest.capabilities.event_cancel_chat);
}

#[test]
fn test_integration_manifest_invalid_fails() {
    let invalid_toml = r#"
id = "INVALID_UPPERCASE"
name = "Test"
version = "1.0.0"
api_version = 1
"#;
    let result = ModManifest::parse(invalid_toml);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidArgument);
}

#[test]
fn test_integration_manifest_future_api_version() {
    let future_toml = r#"
id = "future-mod"
name = "Future Mod"
version = "1.0.0"
api_version = 99
"#;
    let result = ModManifest::parse(future_toml);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::Unsupported);
}

// ============================================================================
// Channel naming tests
// ============================================================================

#[test]
fn test_integration_channel_naming() {
    // Valid channel
    let channel = ModChannel::parse("mod:dummy-mod:events").unwrap();
    assert_eq!(channel.mod_id, "dummy-mod");
    assert_eq!(channel.name, "events");
    assert_eq!(channel.full_name(), "mod:dummy-mod:events");

    // Invalid formats
    assert!(ModChannel::parse("invalid").is_err());
    assert!(ModChannel::parse("mod:only").is_err());
    assert!(ModChannel::parse("notmod:a:b").is_err());
}

// ============================================================================
// Full workflow test
// ============================================================================

#[test]
fn test_integration_full_workflow() {
    // 1. Create registry with policy
    let policy = CapabilityPolicy::permissive();
    let mut registry = ModRegistry::with_policy(policy);

    // 2. Load mod from manifest
    let manifest = load_dummy_manifest();
    registry.load(manifest).expect("Mod should load");

    // 3. Verify mod is active
    assert!(registry.is_enabled("dummy-mod"));
    assert_eq!(registry.active_mod_count(), 1);

    // 4. Check capabilities were applied
    let ctx = registry.get("dummy-mod").unwrap();
    assert!(ctx.has_capability(Capability::WORLD_READ));
    assert!(ctx.has_capability(Capability::WORLD_WRITE));
    assert!(ctx.has_capability(Capability::NET_SEND));
    assert!(ctx.has_capability(Capability::EVENT_CANCEL_CHAT));

    // 5. Subscribe to events
    registry
        .get_mut("dummy-mod")
        .unwrap()
        .subscribe(EventType::PlayerJoin);
    registry
        .get_mut("dummy-mod")
        .unwrap()
        .subscribe(EventType::PlayerChat);

    // 6. Create event bus
    let mut bus = EventBus::new();
    let events_received = Arc::new(AtomicU32::new(0));
    let events_clone = events_received.clone();

    bus.set_handler(Box::new(move |_, _| {
        events_clone.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }));

    // 7. Emit and dispatch events
    bus.emit(GameEvent::player_join(1, "Player1", 1));
    bus.emit(GameEvent::player_chat(1, "Hello", 2));
    bus.emit(GameEvent::server_start(0)); // Not subscribed

    bus.dispatch(&mut registry);

    // 8. Verify only subscribed events were handled
    assert_eq!(events_received.load(Ordering::SeqCst), 2);

    // 9. Unload mod
    registry.unload("dummy-mod").expect("Unload should succeed");
    assert!(!registry.is_loaded("dummy-mod"));
    assert_eq!(registry.mod_count(), 0);
}
