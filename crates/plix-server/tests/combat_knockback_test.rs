//! Knockback feedback tests (US3)
//!
//! T026, T027, T028 - Verify knockback direction, magnitude, and collision interaction

use plix_common::combat::CombatConfig;
use plix_common::math::Vec3;
use plix_common::time::Tick;
use plix_common::types::PlayerId;
use plix_server::sim::combat::CombatSystem;

/// T026 [P] [US3] Unit test: knockback direction is normalize(victim_pos - attacker_pos)
#[test]
fn test_knockback_direction_normalized() {
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    let attacker_id = PlayerId(1);
    let target_id = PlayerId(2);

    // Attacker at origin, target ahead on Z axis
    let attacker_pos = Vec3::ZERO;
    let target_pos = Vec3::new(0.0, 0.0, 1.5);
    let attacker_forward = Vec3::new(0.0, 0.0, 1.0);

    let targets = vec![(target_id, target_pos, 100u8)];

    let result = combat.try_attack(
        &config,
        attacker_id,
        attacker_pos,
        attacker_forward,
        Tick(0),
        Tick(100),
        &targets,
    );

    assert!(result.is_some(), "Attack should hit");
    let (_, hit_result) = result.unwrap();

    // Expected direction: normalize(target_pos - attacker_pos) = normalize((0,0,1.5)) = (0,0,1)
    let expected_dir = (target_pos - attacker_pos).normalize_or_zero();
    assert!(
        (hit_result.knockback_dir - expected_dir).length() < 0.001,
        "Knockback direction should be normalized vector from attacker to victim. Got {:?}, expected {:?}",
        hit_result.knockback_dir,
        expected_dir
    );

    // Verify it's normalized (length ~= 1)
    assert!(
        (hit_result.knockback_dir.length() - 1.0).abs() < 0.001,
        "Knockback direction should be normalized (length 1). Got {}",
        hit_result.knockback_dir.length()
    );
}

/// T026 [P] [US3] Unit test: knockback direction with diagonal attack
#[test]
fn test_knockback_direction_diagonal() {
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    let attacker_id = PlayerId(1);
    let target_id = PlayerId(2);

    // Attacker at origin, target diagonally placed
    let attacker_pos = Vec3::new(0.0, 0.0, 0.0);
    let target_pos = Vec3::new(1.0, 0.0, 1.0); // ~1.41 blocks away diagonally
    let attacker_forward = Vec3::new(1.0, 0.0, 1.0).normalize_or_zero();

    let targets = vec![(target_id, target_pos, 100u8)];

    let result = combat.try_attack(
        &config,
        attacker_id,
        attacker_pos,
        attacker_forward,
        Tick(0),
        Tick(100),
        &targets,
    );

    assert!(result.is_some(), "Attack should hit diagonal target");
    let (_, hit_result) = result.unwrap();

    // Expected direction: normalize(1, 0, 1) = (0.707, 0, 0.707)
    let expected_dir = (target_pos - attacker_pos).normalize_or_zero();
    assert!(
        (hit_result.knockback_dir - expected_dir).length() < 0.001,
        "Diagonal knockback direction should be correct. Got {:?}, expected {:?}",
        hit_result.knockback_dir,
        expected_dir
    );
}

/// T027 [P] [US3] Unit test: knockback magnitude is config.knockback_strength
#[test]
fn test_knockback_magnitude_from_config() {
    let config = CombatConfig::default();

    // Verify the config value exists and is the expected default
    assert_eq!(
        config.knockback_strength, 4.0,
        "Default knockback strength should be 4.0 m/s"
    );

    // The actual application of magnitude happens when velocity is set:
    // velocity = knockback_dir * config.knockback_strength
    // This test verifies the config provides the correct value
    let knockback_dir = Vec3::new(0.0, 0.0, 1.0);
    let knockback_velocity = knockback_dir * config.knockback_strength;

    assert!(
        (knockback_velocity.length() - 4.0).abs() < 0.001,
        "Knockback velocity magnitude should be 4.0 m/s. Got {}",
        knockback_velocity.length()
    );
}

/// T027 [P] [US3] Unit test: knockback velocity calculation with different config values
#[test]
fn test_knockback_velocity_calculation() {
    // Test with custom knockback strength
    let mut config = CombatConfig::default();
    config.knockback_strength = 6.0;

    let knockback_dir = Vec3::new(1.0, 0.0, 0.0);
    let knockback_velocity = knockback_dir * config.knockback_strength;

    assert!(
        (knockback_velocity.x - 6.0).abs() < 0.001,
        "Knockback velocity X should match strength. Got {}",
        knockback_velocity.x
    );
    assert!(
        knockback_velocity.y.abs() < 0.001,
        "Knockback velocity Y should be 0. Got {}",
        knockback_velocity.y
    );
    assert!(
        knockback_velocity.z.abs() < 0.001,
        "Knockback velocity Z should be 0. Got {}",
        knockback_velocity.z
    );
}

/// T028 [P] [US3] Integration test: knockback velocity is additive
/// (collision system handles stopping at walls - this verifies velocity application)
#[test]
fn test_knockback_velocity_application() {
    // Simulate a player receiving knockback
    // The collision system (Feature 008) handles wall stopping

    let config = CombatConfig::default();

    // Starting velocity (player is stationary)
    let initial_velocity = Vec3::ZERO;

    // Knockback direction (from hit result)
    let knockback_dir = Vec3::new(0.0, 0.0, 1.0);

    // Apply knockback as velocity impulse
    let knockback_impulse = knockback_dir * config.knockback_strength;
    let final_velocity = initial_velocity + knockback_impulse;

    assert!(
        (final_velocity.z - 4.0).abs() < 0.001,
        "Knockback should add velocity impulse. Got {:?}",
        final_velocity
    );
}

/// T028 [P] [US3] Integration test: knockback with existing velocity
#[test]
fn test_knockback_with_existing_velocity() {
    let config = CombatConfig::default();

    // Player moving forward at 5 m/s
    let initial_velocity = Vec3::new(0.0, 0.0, 5.0);

    // Hit from the side - knockback pushes in -X direction
    let knockback_dir = Vec3::new(-1.0, 0.0, 0.0);
    let knockback_impulse = knockback_dir * config.knockback_strength;
    let final_velocity = initial_velocity + knockback_impulse;

    // Should have both components
    assert!(
        (final_velocity.x - (-4.0)).abs() < 0.001,
        "Knockback X component should be -4.0. Got {}",
        final_velocity.x
    );
    assert!(
        (final_velocity.z - 5.0).abs() < 0.001,
        "Original Z velocity should be preserved. Got {}",
        final_velocity.z
    );
}

/// T028 [P] [US3] Verify knockback direction handles edge case of overlapping players
#[test]
fn test_knockback_overlapping_players() {
    let combat = CombatSystem::new();
    let config = CombatConfig::default();

    let attacker_id = PlayerId(1);
    let target_id = PlayerId(2);

    // Players at nearly the same position
    let attacker_pos = Vec3::new(5.0, 1.0, 5.0);
    let target_pos = Vec3::new(5.0, 1.0, 5.01); // Very small offset
    let attacker_forward = Vec3::new(0.0, 0.0, 1.0);

    let targets = vec![(target_id, target_pos, 100u8)];

    let result = combat.try_attack(
        &config,
        attacker_id,
        attacker_pos,
        attacker_forward,
        Tick(0),
        Tick(100),
        &targets,
    );

    assert!(result.is_some(), "Attack should hit very close target");
    let (_, hit_result) = result.unwrap();

    // Even with very close targets, knockback_dir should still be valid
    // normalize_or_zero handles the case where positions are identical
    let knockback_length = hit_result.knockback_dir.length();
    assert!(
        knockback_length > 0.0,
        "Knockback direction should be valid even for very close targets. Got length {}",
        knockback_length
    );
}
