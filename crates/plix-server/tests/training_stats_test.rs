//! Training mode statistics tests (Feature 020 - US4)
//!
//! Tests for statistics tracking and retrieval.

use glam::Vec3;
use plix_common::time::Tick;
use plix_common::types::BotId;
use plix_server::training::{TrainingConfig, TrainingCoordinator, TrainingStats};

// =============================================================================
// T053: Stats show total hits
// =============================================================================

#[test]
fn test_stats_show_total_hits() {
    let mut config = TrainingConfig::default();
    config.bot_count = 3;

    let spawn_points = vec![
        Vec3::new(10.0, 2.0, 10.0),
        Vec3::new(20.0, 2.0, 20.0),
        Vec3::new(30.0, 2.0, 30.0),
    ];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // No hits initially
    assert_eq!(coordinator.stats().hits, 0);

    // Hit different bots
    coordinator.process_hit(BotId(0), 25, Tick(10));
    coordinator.process_hit(BotId(1), 25, Tick(20));
    coordinator.process_hit(BotId(2), 25, Tick(30));

    assert_eq!(coordinator.stats().hits, 3, "Should track all hits");
}

// =============================================================================
// T054: Stats show accuracy percentage
// =============================================================================

#[test]
fn test_stats_show_accuracy_percentage() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Record attacks and hits
    coordinator.record_attack(); // Miss
    coordinator.record_attack(); // Miss
    coordinator.record_attack(); // Hit
    coordinator.process_hit(BotId(0), 25, Tick(10));
    coordinator.record_attack(); // Hit
    coordinator.process_hit(BotId(0), 25, Tick(20));

    assert_eq!(coordinator.stats().attacks, 4);
    assert_eq!(coordinator.stats().hits, 2);

    // Accuracy = 2/4 = 50%
    let accuracy = coordinator.stats().accuracy();
    assert!(
        (accuracy - 50.0).abs() < 0.1,
        "Accuracy should be 50%, got {}",
        accuracy
    );
}

#[test]
fn test_stats_accuracy_100_percent() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // All hits
    coordinator.record_attack();
    coordinator.process_hit(BotId(0), 25, Tick(10));
    coordinator.record_attack();
    coordinator.process_hit(BotId(0), 25, Tick(20));

    let accuracy = coordinator.stats().accuracy();
    assert!((accuracy - 100.0).abs() < 0.1, "Accuracy should be 100%");
}

#[test]
fn test_stats_accuracy_0_percent() {
    let config = TrainingConfig::default();
    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // All misses
    coordinator.record_attack();
    coordinator.record_attack();
    coordinator.record_attack();

    let accuracy = coordinator.stats().accuracy();
    assert!((accuracy - 0.0).abs() < 0.1, "Accuracy should be 0%");
}

#[test]
fn test_stats_accuracy_no_attacks() {
    let config = TrainingConfig::default();
    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // No attacks = 0% accuracy (not NaN or error)
    let accuracy = coordinator.stats().accuracy();
    assert!(
        (accuracy - 0.0).abs() < 0.1,
        "Accuracy should be 0% with no attacks"
    );
}

// =============================================================================
// T055: Stats show session duration
// =============================================================================

#[test]
fn test_stats_show_session_duration() {
    let config = TrainingConfig::default();
    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Session started at tick 0, current tick is 600 (10 seconds at 60Hz)
    let duration = coordinator.stats().duration_secs(Tick(600), 60);
    assert!(
        (duration - 10.0).abs() < 0.1,
        "Duration should be 10 seconds, got {}",
        duration
    );
}

#[test]
fn test_stats_duration_after_reset() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Play for some time
    let _duration_before = coordinator.stats().duration_secs(Tick(1200), 60);

    // Reset at tick 1200 (20 seconds)
    coordinator.reset(Tick(1200));

    // Duration should be 0 immediately after reset
    let duration_after_reset = coordinator.stats().duration_secs(Tick(1200), 60);
    assert!(
        (duration_after_reset - 0.0).abs() < 0.1,
        "Duration should reset to 0"
    );

    // 5 more seconds (300 ticks)
    let duration_later = coordinator.stats().duration_secs(Tick(1500), 60);
    assert!(
        (duration_later - 5.0).abs() < 0.1,
        "Duration should be 5 seconds after reset"
    );
}

// =============================================================================
// Additional Stats Tests
// =============================================================================

#[test]
fn test_stats_show_kills() {
    let mut config = TrainingConfig::default();
    config.bot_count = 3;
    config.bot_respawn_delay_ticks = 1000; // Long delay

    let spawn_points = vec![
        Vec3::new(10.0, 2.0, 10.0),
        Vec3::new(20.0, 2.0, 20.0),
        Vec3::new(30.0, 2.0, 30.0),
    ];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Kill two bots
    coordinator.process_hit(BotId(0), 100, Tick(10)); // Kill
    coordinator.process_hit(BotId(1), 50, Tick(20)); // Damage
    coordinator.process_hit(BotId(2), 100, Tick(30)); // Kill

    assert_eq!(coordinator.stats().kills, 2, "Should track kills");
    assert_eq!(coordinator.stats().hits, 3, "Should track all hits");
}

#[test]
fn test_stats_creation_sets_session_start() {
    let stats = TrainingStats::new(Tick(500));
    assert_eq!(stats.session_start, Tick(500));
    assert_eq!(stats.attacks, 0);
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.kills, 0);
}

#[test]
fn test_stats_record_methods() {
    let mut stats = TrainingStats::new(Tick(0));

    stats.record_attack();
    assert_eq!(stats.attacks, 1);

    stats.record_hit();
    assert_eq!(stats.hits, 1);

    stats.record_kill();
    assert_eq!(stats.kills, 1);

    stats.record_attack();
    stats.record_attack();
    stats.record_hit();

    assert_eq!(stats.attacks, 3);
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.kills, 1);
}

#[test]
fn test_stats_reset() {
    let mut stats = TrainingStats::new(Tick(0));

    stats.record_attack();
    stats.record_attack();
    stats.record_hit();
    stats.record_kill();

    assert_eq!(stats.attacks, 2);
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.kills, 1);

    stats.reset(Tick(1000));

    assert_eq!(stats.attacks, 0);
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.kills, 0);
    assert_eq!(stats.session_start, Tick(1000));
}

#[test]
fn test_coordinator_stats_reference() {
    let config = TrainingConfig::default();
    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // stats() returns a reference
    let stats = coordinator.stats();
    assert_eq!(stats.session_start, Tick(0));
}
