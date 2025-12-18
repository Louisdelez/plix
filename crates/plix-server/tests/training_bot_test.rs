//! Training mode bot tests (Feature 020 - US1)
//!
//! Tests for bot spawning, damage, respawn mechanics in training mode.

use glam::Vec3;
use plix_common::time::Tick;
use plix_common::types::BotId;
use plix_server::training::{
    BotBehavior, BotBehaviorType, TrainingBot, TrainingConfig, TrainingCoordinator, TrainingEvent,
};

// =============================================================================
// T021: Bot spawn count matches config
// =============================================================================

#[test]
fn test_bot_spawn_count_matches_config() {
    let mut config = TrainingConfig::default();
    config.bot_count = 3;

    let spawn_points = vec![
        Vec3::new(10.0, 2.0, 10.0),
        Vec3::new(20.0, 2.0, 20.0),
        Vec3::new(30.0, 2.0, 30.0),
    ];

    let coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    assert_eq!(coordinator.bots.len(), 3, "Should spawn exactly 3 bots");
}

#[test]
fn test_bot_spawn_count_zero() {
    let mut config = TrainingConfig::default();
    config.bot_count = 0;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    assert_eq!(
        coordinator.bots.len(),
        0,
        "Should spawn no bots when count is 0"
    );
}

#[test]
fn test_bot_spawn_count_maximum() {
    let mut config = TrainingConfig::default();
    config.bot_count = 20; // Max allowed

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0), Vec3::new(20.0, 2.0, 20.0)];

    let coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    assert_eq!(coordinator.bots.len(), 20, "Should spawn maximum 20 bots");
}

#[test]
fn test_bots_spawn_at_correct_positions() {
    let mut config = TrainingConfig::default();
    config.bot_count = 3;

    let spawn_points = vec![
        Vec3::new(10.0, 2.0, 10.0),
        Vec3::new(20.0, 2.0, 20.0),
        Vec3::new(30.0, 2.0, 30.0),
    ];

    let coordinator = TrainingCoordinator::new(config, spawn_points.clone(), Tick(0));

    assert_eq!(coordinator.bots[0].position, spawn_points[0]);
    assert_eq!(coordinator.bots[1].position, spawn_points[1]);
    assert_eq!(coordinator.bots[2].position, spawn_points[2]);
}

// =============================================================================
// T022: Bot respawn after delay
// =============================================================================

#[test]
fn test_bot_respawn_after_delay() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.bot_respawn_delay_ticks = 60; // 1 second at 60Hz

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points.clone(), Tick(0));

    // Kill the bot at tick 100
    let killed = coordinator.process_hit(BotId(0), 100, Tick(100));
    assert!(killed, "Bot should be killed");
    assert!(coordinator.bots[0].is_dead, "Bot should be dead");

    // Tick before respawn - no respawn
    let events = coordinator.tick(Tick(159));
    assert!(events.is_empty(), "No respawn event before delay");
    assert!(coordinator.bots[0].is_dead, "Bot still dead before delay");

    // Tick at respawn time
    let events = coordinator.tick(Tick(160));
    assert_eq!(events.len(), 1, "Should have respawn event");
    assert!(!coordinator.bots[0].is_dead, "Bot should be alive");

    // Verify respawn event
    let TrainingEvent::BotRespawned { bot_id, position } = &events[0];
    assert_eq!(*bot_id, BotId(0));
    assert_eq!(
        *position, spawn_points[0],
        "Should respawn at original position"
    );
}

#[test]
fn test_bot_respawn_restores_health() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.bot_respawn_delay_ticks = 30;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Kill the bot
    coordinator.process_hit(BotId(0), 100, Tick(0));
    assert_eq!(coordinator.bots[0].health, 0);

    // Respawn
    coordinator.tick(Tick(30));
    assert_eq!(
        coordinator.bots[0].health, 100,
        "Health should be restored to full"
    );
}

// =============================================================================
// T023: Bot hit registers damage
// =============================================================================

#[test]
fn test_bot_hit_registers_damage() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Hit bot for 30 damage
    let killed = coordinator.process_hit(BotId(0), 30, Tick(0));

    assert!(!killed, "Bot should not be killed by 30 damage");
    assert_eq!(
        coordinator.bots[0].health, 70,
        "Health should be reduced to 70"
    );
    assert!(!coordinator.bots[0].is_dead, "Bot should still be alive");
}

#[test]
fn test_bot_hit_records_stats() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    assert_eq!(coordinator.stats().hits, 0);

    // Hit bot
    coordinator.process_hit(BotId(0), 30, Tick(0));

    assert_eq!(coordinator.stats().hits, 1, "Hit should be recorded");
}

#[test]
fn test_bot_kill_records_stats() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    assert_eq!(coordinator.stats().kills, 0);

    // Kill bot
    coordinator.process_hit(BotId(0), 100, Tick(0));

    assert_eq!(coordinator.stats().hits, 1, "Hit should be recorded");
    assert_eq!(coordinator.stats().kills, 1, "Kill should be recorded");
}

#[test]
fn test_multiple_hits_accumulate_damage() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Hit bot multiple times
    coordinator.process_hit(BotId(0), 25, Tick(0));
    assert_eq!(coordinator.bots[0].health, 75);

    coordinator.process_hit(BotId(0), 25, Tick(1));
    assert_eq!(coordinator.bots[0].health, 50);

    coordinator.process_hit(BotId(0), 25, Tick(2));
    assert_eq!(coordinator.bots[0].health, 25);

    let killed = coordinator.process_hit(BotId(0), 25, Tick(3));
    assert!(killed, "Fourth hit should kill bot");
    assert_eq!(coordinator.bots[0].health, 0);
    assert!(coordinator.bots[0].is_dead);
}

// =============================================================================
// T024: Invincible bot takes no damage but hit counted
// =============================================================================

#[test]
fn test_invincible_bot_no_damage() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.invincibility_bots = true;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Hit invincible bot
    let killed = coordinator.process_hit(BotId(0), 100, Tick(0));

    assert!(!killed, "Invincible bot should not be killed");
    assert_eq!(
        coordinator.bots[0].health, 100,
        "Health should be unchanged"
    );
    assert!(!coordinator.bots[0].is_dead, "Bot should not be dead");
}

#[test]
fn test_invincible_bot_hit_still_counted() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.invincibility_bots = true;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Hit invincible bot multiple times
    coordinator.process_hit(BotId(0), 50, Tick(0));
    coordinator.process_hit(BotId(0), 50, Tick(1));
    coordinator.process_hit(BotId(0), 50, Tick(2));

    assert_eq!(coordinator.stats().hits, 3, "All hits should be counted");
    assert_eq!(coordinator.stats().kills, 0, "No kills on invincible bots");
    assert_eq!(coordinator.bots[0].health, 100, "Health unchanged");
}

#[test]
fn test_invincible_bot_accuracy_tracking() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.invincibility_bots = true;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Record attacks and hits
    coordinator.record_attack(); // Miss
    coordinator.record_attack(); // Hit
    coordinator.process_hit(BotId(0), 50, Tick(0));
    coordinator.record_attack(); // Hit
    coordinator.process_hit(BotId(0), 50, Tick(1));

    assert_eq!(coordinator.stats().attacks, 3);
    assert_eq!(coordinator.stats().hits, 2);

    // Accuracy should be 2/3 = 66.67%
    let accuracy = coordinator.stats().accuracy();
    assert!(
        (accuracy - 66.666).abs() < 1.0,
        "Accuracy should be ~66.67%"
    );
}

// =============================================================================
// Additional Bot Tests
// =============================================================================

#[test]
fn test_hit_invalid_bot_id() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Try to hit non-existent bot
    let killed = coordinator.process_hit(BotId(99), 50, Tick(0));

    assert!(!killed, "Should not kill non-existent bot");
    // Hit is still recorded for accuracy tracking
    assert_eq!(
        coordinator.stats().hits,
        1,
        "Hit recorded even for invalid target"
    );
}

#[test]
fn test_hit_dead_bot_no_effect() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.bot_respawn_delay_ticks = 1000; // Long delay

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Kill bot
    coordinator.process_hit(BotId(0), 100, Tick(0));
    assert!(coordinator.bots[0].is_dead);

    // Try to hit dead bot
    let killed = coordinator.process_hit(BotId(0), 50, Tick(1));

    assert!(!killed, "Dead bot can't be killed again");
    assert_eq!(coordinator.stats().hits, 2, "Both hits recorded");
    assert_eq!(coordinator.stats().kills, 1, "Only one kill");
}

#[test]
fn test_bot_snapshots_reflect_state() {
    let mut config = TrainingConfig::default();
    config.bot_count = 2;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0), Vec3::new(20.0, 2.0, 20.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points.clone(), Tick(0));

    // Damage one bot
    coordinator.process_hit(BotId(0), 50, Tick(0));

    let snapshots = coordinator.bot_snapshots();

    assert_eq!(snapshots.len(), 2);

    // First bot: damaged
    assert_eq!(snapshots[0].id, BotId(0));
    assert_eq!(snapshots[0].position, spawn_points[0]);
    assert_eq!(snapshots[0].health, 50);
    assert!(!snapshots[0].is_dead);

    // Second bot: healthy
    assert_eq!(snapshots[1].id, BotId(1));
    assert_eq!(snapshots[1].position, spawn_points[1]);
    assert_eq!(snapshots[1].health, 100);
    assert!(!snapshots[1].is_dead);
}
