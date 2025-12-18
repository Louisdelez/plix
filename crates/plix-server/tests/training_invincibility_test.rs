//! Training mode invincibility tests (Feature 020 - US6)
//!
//! Tests for player and bot invincibility options.

use glam::Vec3;
use plix_common::time::Tick;
use plix_common::types::BotId;
use plix_server::training::{TrainingConfig, TrainingCoordinator};

// =============================================================================
// T070: Invincible player configuration
// =============================================================================

#[test]
fn test_player_invincibility_default_off() {
    let config = TrainingConfig::default();
    assert!(
        !config.invincibility_player,
        "Player invincibility should be off by default"
    );
}

#[test]
fn test_player_invincibility_configurable() {
    let mut config = TrainingConfig::default();
    config.invincibility_player = true;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    assert!(coordinator.is_player_invincible());
}

#[test]
fn test_player_invincibility_from_coordinator() {
    let mut config = TrainingConfig::default();
    config.invincibility_player = false;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    assert!(!coordinator.is_player_invincible());
}

// =============================================================================
// T071: Non-invincible player behavior (normal damage)
// =============================================================================

#[test]
fn test_non_invincible_bots_take_damage() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.invincibility_bots = false;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Bot should take damage
    let killed = coordinator.process_hit(BotId(0), 50, Tick(10));
    assert!(!killed);
    assert_eq!(coordinator.bots[0].health, 50, "Bot should take damage");

    // Bot should die
    let killed = coordinator.process_hit(BotId(0), 50, Tick(20));
    assert!(killed);
    assert!(coordinator.bots[0].is_dead, "Bot should be dead");
}

// =============================================================================
// T072-T074: Bot invincibility
// =============================================================================

#[test]
fn test_bot_invincibility_default_off() {
    let config = TrainingConfig::default();
    assert!(
        !config.invincibility_bots,
        "Bot invincibility should be off by default"
    );
}

#[test]
fn test_invincible_bots_no_damage() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.invincibility_bots = true;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Invincible bot should not take damage
    let killed = coordinator.process_hit(BotId(0), 100, Tick(10));
    assert!(!killed, "Invincible bot should not be killed");
    assert_eq!(
        coordinator.bots[0].health, 100,
        "Invincible bot health unchanged"
    );
    assert!(
        !coordinator.bots[0].is_dead,
        "Invincible bot should not die"
    );
}

#[test]
fn test_invincible_bots_hits_still_counted() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.invincibility_bots = true;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Multiple hits on invincible bot
    coordinator.process_hit(BotId(0), 50, Tick(10));
    coordinator.process_hit(BotId(0), 50, Tick(20));
    coordinator.process_hit(BotId(0), 50, Tick(30));

    // Hits should be counted for accuracy
    assert_eq!(coordinator.stats().hits, 3, "Hits should be recorded");
    // No kills
    assert_eq!(coordinator.stats().kills, 0, "No kills on invincible bots");
    // Bot health unchanged
    assert_eq!(coordinator.bots[0].health, 100);
}

// =============================================================================
// T075-T077: Combined invincibility scenarios
// =============================================================================

#[test]
fn test_both_invincibility_options_independent() {
    // Both on
    let mut config = TrainingConfig::default();
    config.invincibility_player = true;
    config.invincibility_bots = true;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let coordinator = TrainingCoordinator::new(config.clone(), spawn_points.clone(), Tick(0));

    assert!(coordinator.is_player_invincible());
    assert!(coordinator.config.invincibility_bots);

    // Player on, bot off
    let mut config2 = TrainingConfig::default();
    config2.invincibility_player = true;
    config2.invincibility_bots = false;

    let coordinator2 = TrainingCoordinator::new(config2.clone(), spawn_points.clone(), Tick(0));
    assert!(coordinator2.is_player_invincible());
    assert!(!coordinator2.config.invincibility_bots);

    // Player off, bot on
    let mut config3 = TrainingConfig::default();
    config3.invincibility_player = false;
    config3.invincibility_bots = true;

    let coordinator3 = TrainingCoordinator::new(config3.clone(), spawn_points.clone(), Tick(0));
    assert!(!coordinator3.is_player_invincible());
    assert!(coordinator3.config.invincibility_bots);
}

#[test]
fn test_accuracy_tracking_with_invincible_bots() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.invincibility_bots = true;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Simulate attacks and hits
    coordinator.record_attack(); // Miss
    coordinator.record_attack(); // Hit
    coordinator.process_hit(BotId(0), 50, Tick(10));
    coordinator.record_attack(); // Hit
    coordinator.process_hit(BotId(0), 50, Tick(20));

    // 3 attacks, 2 hits = 66.67%
    let accuracy = coordinator.stats().accuracy();
    assert!(
        (accuracy - 66.666).abs() < 1.0,
        "Accuracy should be ~66.67%"
    );

    // Bot still alive
    assert!(!coordinator.bots[0].is_dead);
    assert_eq!(coordinator.bots[0].health, 100);
}

#[test]
fn test_invincibility_preserved_after_reset() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.invincibility_bots = true;
    config.invincibility_player = true;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Hit bot (doesn't die due to invincibility)
    coordinator.process_hit(BotId(0), 100, Tick(10));
    assert!(!coordinator.bots[0].is_dead);

    // Reset
    coordinator.reset(Tick(100));

    // Invincibility settings should still apply
    coordinator.process_hit(BotId(0), 100, Tick(110));
    assert!(
        !coordinator.bots[0].is_dead,
        "Invincibility should persist after reset"
    );
    assert!(
        coordinator.is_player_invincible(),
        "Player invincibility should persist"
    );
}

#[test]
fn test_respawn_delay_config() {
    let mut config = TrainingConfig::default();
    config.bot_respawn_delay_ticks = 120; // 2 seconds
    config.player_respawn_delay_ticks = 30; // 0.5 seconds

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let coordinator = TrainingCoordinator::new(config.clone(), spawn_points, Tick(0));

    assert_eq!(coordinator.config.bot_respawn_delay_ticks, 120);
    assert_eq!(coordinator.player_respawn_delay_ticks(), 30);
}
