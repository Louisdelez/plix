//! Latency-tolerant hit registration tests (US5)
//!
//! T042, T043 - Verify server uses authoritative positions and produces deterministic results

use plix_common::combat::CombatConfig;
use plix_common::math::Vec3;
use plix_common::time::Tick;
use plix_common::types::PlayerId;
use plix_server::sim::combat::CombatSystem;

/// T042 [P] [US5] Unit test: hit uses server authoritative positions not client-reported
/// The combat system receives server-side positions, not client-reported positions.
/// This test verifies that the positions passed to try_attack are what determines hits.
#[test]
fn test_hit_uses_server_authoritative_positions() {
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    let attacker_id = PlayerId(1);
    let target_id = PlayerId(2);

    // Server's authoritative position for attacker
    let server_attacker_pos = Vec3::new(0.0, 1.0, 0.0);
    let attacker_forward = Vec3::new(0.0, 0.0, 1.0);

    // Server's authoritative position for target (in range at 1.5 blocks)
    let server_target_pos = Vec3::new(0.0, 1.0, 1.5);

    // Hypothetical client-reported position (out of range at 3.0 blocks)
    // This should NOT be used - the server ignores client positions
    let _client_reported_target_pos = Vec3::new(0.0, 1.0, 3.0);

    // Combat system uses server positions ONLY
    let targets = vec![(target_id, server_target_pos, 100u8)];

    let result = combat.try_attack(
        &config,
        attacker_id,
        server_attacker_pos,
        attacker_forward,
        Tick(0),
        Tick(100),
        &targets,
    );

    // Should hit because server positions show target in range
    assert!(
        result.is_some(),
        "Hit should succeed based on server authoritative positions"
    );

    let (hit_id, _) = result.unwrap();
    assert_eq!(hit_id, target_id);
}

/// T042 [P] [US5] Test that no client position data affects hit detection
/// The server never uses client-reported positions for combat
#[test]
fn test_client_positions_ignored() {
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    let attacker_id = PlayerId(1);
    let target_id = PlayerId(2);

    // Server positions: target is OUT of range
    let server_attacker_pos = Vec3::ZERO;
    let server_target_pos = Vec3::new(0.0, 0.0, 10.0); // 10 blocks - out of range
    let attacker_forward = Vec3::new(0.0, 0.0, 1.0);

    // Even if a hypothetical client claimed target was close, server uses its own data
    let targets = vec![(target_id, server_target_pos, 100u8)];

    let result = combat.try_attack(
        &config,
        attacker_id,
        server_attacker_pos,
        attacker_forward,
        Tick(0),
        Tick(100),
        &targets,
    );

    // Should miss because server positions show target out of range
    assert!(
        result.is_none(),
        "Hit should fail when server authoritative positions show target out of range"
    );
}

/// T043 [P] [US5] Determinism test: same inputs produce same outcomes across runs
#[test]
fn test_deterministic_combat_outcomes() {
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    let attacker_id = PlayerId(1);
    let target_id = PlayerId(2);

    let attacker_pos = Vec3::new(5.0, 1.0, 5.0);
    let attacker_forward = Vec3::new(0.0, 0.0, 1.0);
    let target_pos = Vec3::new(5.0, 1.0, 6.5); // 1.5 blocks away

    let targets = vec![(target_id, target_pos, 100u8)];

    // Run the same attack multiple times
    let mut results = Vec::new();
    for _ in 0..10 {
        let result = combat.try_attack(
            &config,
            attacker_id,
            attacker_pos,
            attacker_forward,
            Tick(0),
            Tick(100),
            &targets,
        );
        results.push(result.is_some());
    }

    // All results should be identical
    assert!(
        results.iter().all(|&r| r == results[0]),
        "Combat should produce deterministic results: {:?}",
        results
    );
}

/// T043 [P] [US5] Determinism test: edge case positions produce consistent results
#[test]
fn test_deterministic_edge_case() {
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    let attacker_id = PlayerId(1);
    let target_id = PlayerId(2);

    let attacker_pos = Vec3::ZERO;
    let attacker_forward = Vec3::new(0.0, 0.0, 1.0);

    // Target at exactly effective_range - this is an edge case
    let effective_range = config.effective_range();
    let target_pos = Vec3::new(0.0, 0.0, effective_range);

    let targets = vec![(target_id, target_pos, 100u8)];

    // Run multiple times to check determinism at boundary
    let mut results = Vec::new();
    for _ in 0..100 {
        let result = combat.try_attack(
            &config,
            attacker_id,
            attacker_pos,
            attacker_forward,
            Tick(0),
            Tick(100),
            &targets,
        );
        results.push(result.is_some());
    }

    // All results must be the same (deterministic)
    let all_same = results.iter().all(|&r| r == results[0]);
    assert!(
        all_same,
        "Combat at edge of range must be deterministic. Results varied: {:?}",
        results.iter().filter(|&&r| r).count()
    );
}

/// T044/T045 Verification: epsilon is only latency tolerance, no history buffer
/// This test verifies the epsilon-only approach works correctly
#[test]
fn test_epsilon_only_tolerance() {
    let config = CombatConfig::default();

    // Verify epsilon values
    assert_eq!(config.attack_range, 1.8, "Base attack range should be 1.8");
    assert_eq!(
        config.attack_range_epsilon, 0.15,
        "Attack range epsilon should be 0.15"
    );
    assert!(
        (config.effective_range() - 1.95).abs() < 0.0001,
        "Effective range should be ~1.95 (1.8 + 0.15), got {}",
        config.effective_range()
    );

    // The epsilon provides a small buffer for network latency
    // but does NOT require any history buffer or position rewind
    let combat = CombatSystem::new();

    let attacker_id = PlayerId(1);
    let target_id = PlayerId(2);

    // Target at 1.9 blocks (within epsilon, beyond base range)
    let targets = vec![(target_id, Vec3::new(0.0, 0.0, 1.9), 100u8)];

    let result = combat.try_attack(
        &config,
        attacker_id,
        Vec3::ZERO,
        Vec3::new(0.0, 0.0, 1.0),
        Tick(0),
        Tick(100),
        &targets,
    );

    // Should hit because 1.9 <= 1.95 effective range
    assert!(result.is_some(), "Hit within epsilon should succeed");

    // Target at 2.0 blocks (beyond effective range)
    let targets_far = vec![(target_id, Vec3::new(0.0, 0.0, 2.0), 100u8)];

    let result_far = combat.try_attack(
        &config,
        attacker_id,
        Vec3::ZERO,
        Vec3::new(0.0, 0.0, 1.0),
        Tick(0),
        Tick(100),
        &targets_far,
    );

    // Should miss because 2.0 > 1.95 effective range
    assert!(
        result_far.is_none(),
        "Hit beyond effective range should fail"
    );
}

/// Verify tick-based timing is deterministic
#[test]
fn test_tick_based_determinism() {
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    let attacker_id = PlayerId(1);
    let target_id = PlayerId(2);

    let targets = vec![(target_id, Vec3::new(0.0, 0.0, 1.5), 100u8)];

    // Same tick values should always produce same cooldown result
    let last_attack = Tick(50);
    let current = Tick(70); // 20 ticks since attack (cooldown is 30)

    // Should fail due to cooldown
    for _ in 0..10 {
        let result = combat.try_attack(
            &config,
            attacker_id,
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 1.0),
            last_attack,
            current,
            &targets,
        );
        assert!(
            result.is_none(),
            "Attack on cooldown should always fail deterministically"
        );
    }

    // After cooldown (30 ticks)
    let after_cooldown = Tick(last_attack.0 + config.attack_cooldown_ticks);

    for _ in 0..10 {
        let result = combat.try_attack(
            &config,
            attacker_id,
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 1.0),
            last_attack,
            after_cooldown,
            &targets,
        );
        assert!(
            result.is_some(),
            "Attack after cooldown should always succeed deterministically"
        );
    }
}
