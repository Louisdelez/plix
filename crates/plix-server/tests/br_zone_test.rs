//! BR Lite zone integration tests
//!
//! Tests for User Story 2: Shrinking Safe Zone

use glam::{Vec2, Vec3};
use plix_common::time::Tick;
use plix_common::types::PlayerId;
use plix_server::br_lite::{
    BrLiteConfig, BrLiteCoordinator, BrLiteEvent, MatchPhase, PhaseMode, ZonePhase,
};
use std::collections::HashMap;

fn test_config() -> BrLiteConfig {
    BrLiteConfig {
        min_players: 2,
        post_match_delay_secs: 5,
        phases: vec![
            // Phase 0: 1 second stable, 1 second shrink to 80%, 5 damage
            ZonePhase::new(1, 1, 0.8, 5),
            // Phase 1: 1 second stable, 1 second shrink to 50%, 10 damage
            ZonePhase::new(1, 1, 0.5, 10),
        ],
        initial_radius: Some(50.0),
        damage_tick_interval: 60, // 1 second at 60Hz
        ..Default::default()
    }
}

/// T037: Test zone state starts in Stable mode
#[test]
fn test_zone_starts_stable() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    let zone = coordinator.zone_state();
    assert_eq!(zone.phase_mode, PhaseMode::Stable);
    assert_eq!(zone.phase_index, 0);
    assert_eq!(zone.current_radius, 50.0);
    assert_eq!(zone.target_radius, 50.0);
}

/// T038: Test zone transitions from Stable to Shrinking
#[test]
fn test_zone_stable_to_shrinking_transition() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    let positions = HashMap::new();

    // Tick through stable phase (1 second = 60 ticks)
    for i in 0..60 {
        coordinator.tick(Tick(i), &positions);
    }

    // Should still be stable on last tick
    let zone = coordinator.zone_state();
    assert_eq!(zone.phase_mode, PhaseMode::Stable);

    // One more tick transitions to shrinking
    let events = coordinator.tick(Tick(60), &positions);

    let zone = coordinator.zone_state();
    assert_eq!(zone.phase_mode, PhaseMode::Shrinking);
    assert_eq!(zone.target_radius, 40.0); // 80% of 50.0

    // Should have ZoneUpdate event
    let has_zone_update = events
        .iter()
        .any(|e| matches!(e, BrLiteEvent::ZoneUpdate(_)));
    assert!(
        has_zone_update,
        "Expected ZoneUpdate event on phase transition"
    );
}

/// T039: Test zone radius interpolation during shrink
#[test]
fn test_zone_radius_interpolation() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    let positions = HashMap::new();

    // Skip to shrinking phase
    for i in 0..61 {
        coordinator.tick(Tick(i), &positions);
    }

    // Now in shrinking phase, radius should interpolate
    let start_radius = coordinator.zone_state().current_radius;

    // Tick halfway through shrink (30 ticks)
    for i in 61..91 {
        coordinator.tick(Tick(i), &positions);
    }

    let mid_radius = coordinator.zone_state().current_radius;
    // Should be approximately halfway between 50.0 and 40.0 = 45.0
    assert!(
        (mid_radius - 45.0).abs() < 1.0,
        "Expected mid-shrink radius ~45.0, got {}",
        mid_radius
    );

    // Complete shrink
    for i in 91..121 {
        coordinator.tick(Tick(i), &positions);
    }

    let end_radius = coordinator.zone_state().current_radius;
    // Should be 40.0 (80% of 50)
    assert!(
        (end_radius - 40.0).abs() < 0.5,
        "Expected end radius ~40.0, got {}",
        end_radius
    );
}

/// T040: Test is_in_zone returns true for players inside
#[test]
fn test_is_in_zone_inside() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    // Zone center is at (32, 32) with radius 50
    // Player at center should be inside
    assert!(coordinator.is_in_zone(Vec3::new(32.0, 5.0, 32.0)));

    // Player near edge but inside
    assert!(coordinator.is_in_zone(Vec3::new(32.0 + 45.0, 5.0, 32.0)));
}

/// T041: Test is_in_zone returns false for players outside
#[test]
fn test_is_in_zone_outside() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    // Zone center is at (32, 32) with radius 50
    // Player far outside should be outside zone
    assert!(!coordinator.is_in_zone(Vec3::new(100.0, 5.0, 100.0)));

    // Player just past the edge
    assert!(!coordinator.is_in_zone(Vec3::new(32.0 + 55.0, 5.0, 32.0)));
}

/// T042: Test zone damage applies to players outside zone
#[test]
fn test_zone_damage_outside() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    // Player 1 inside zone, player 2 outside zone
    let mut positions = HashMap::new();
    positions.insert(PlayerId(1), Vec3::new(32.0, 5.0, 32.0)); // Inside
    positions.insert(PlayerId(2), Vec3::new(200.0, 5.0, 200.0)); // Outside

    // First tick with damage interval = 60
    let events = coordinator.tick(Tick(60), &positions);

    // Player 2 should receive zone damage
    let has_damage = events.iter().any(|e| {
        matches!(e, BrLiteEvent::ZoneDamage { player_id, damage }
            if *player_id == PlayerId(2) && *damage == 5)
    });
    assert!(has_damage, "Expected zone damage event for player 2");

    // Player 1 should NOT receive damage
    let player1_damaged = events.iter().any(
        |e| matches!(e, BrLiteEvent::ZoneDamage { player_id, .. } if *player_id == PlayerId(1)),
    );
    assert!(!player1_damaged, "Player 1 should not receive zone damage");
}

/// T043: Test zone damage respects interval
#[test]
fn test_zone_damage_interval() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    let mut positions = HashMap::new();
    positions.insert(PlayerId(1), Vec3::new(200.0, 5.0, 200.0)); // Outside

    // First damage at tick 60
    let events1 = coordinator.tick(Tick(60), &positions);
    let damage_count1 = events1
        .iter()
        .filter(|e| matches!(e, BrLiteEvent::ZoneDamage { .. }))
        .count();
    assert_eq!(damage_count1, 1);

    // No damage at tick 61 (too soon)
    let events2 = coordinator.tick(Tick(61), &positions);
    let damage_count2 = events2
        .iter()
        .filter(|e| matches!(e, BrLiteEvent::ZoneDamage { .. }))
        .count();
    assert_eq!(damage_count2, 0);

    // Damage again at tick 120 (60 ticks later)
    let events3 = coordinator.tick(Tick(120), &positions);
    let damage_count3 = events3
        .iter()
        .filter(|e| matches!(e, BrLiteEvent::ZoneDamage { .. }))
        .count();
    assert_eq!(damage_count3, 1);
}

/// T044: Test damage increases in later phases
#[test]
fn test_damage_increases_per_phase() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    // Check initial damage (phase 0)
    let zone = coordinator.zone_state();
    assert_eq!(zone.damage_per_tick, 5);
    assert_eq!(zone.phase_index, 0);

    let positions = HashMap::new();

    // Phase 0: 1 second stable (60 ticks) + 1 second shrink (60 ticks) = 120 ticks total
    // Timer starts at 60, decrements each tick until 0, then transitions
    // So we need: 60 ticks stable + 1 tick transition + 60 ticks shrink + 1 tick transition = 122 ticks
    // Let's just tick enough to get through phase 0 completely
    for _ in 0..150 {
        coordinator.tick(Tick(0), &positions);
    }

    // Now should be in phase 1
    let zone = coordinator.zone_state();
    // After shrinking completes, we move to next phase
    assert_eq!(
        zone.phase_index, 1,
        "Expected phase_index 1 after 150 ticks, got {} (mode: {:?})",
        zone.phase_index, zone.phase_mode
    );
    assert_eq!(zone.damage_per_tick, 10);
}

/// Test zone center is correct
#[test]
fn test_zone_center() {
    let config = BrLiteConfig {
        zone_center: Some([40.0, 40.0]),
        initial_radius: Some(30.0),
        ..test_config()
    };
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    let zone = coordinator.zone_state();
    assert_eq!(zone.center.x, 40.0);
    assert_eq!(zone.center.y, 40.0); // Vec2.y is Z coordinate
    assert_eq!(zone.current_radius, 30.0);
}

/// Test zone uses arena center when not specified
#[test]
fn test_zone_defaults_to_arena_center() {
    let config = BrLiteConfig {
        zone_center: None, // Not specified
        initial_radius: Some(30.0),
        ..test_config()
    };
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    let zone = coordinator.zone_state();
    // Should be arena center (64/2 = 32)
    assert_eq!(zone.center.x, 32.0);
    assert_eq!(zone.center.y, 32.0);
}
