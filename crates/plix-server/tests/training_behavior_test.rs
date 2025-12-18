//! Training mode bot behavior tests (Feature 020 - US5)
//!
//! Integration tests for bot behavior configuration.

use glam::Vec3;
use plix_common::time::Tick;
use plix_common::types::BotId;
use plix_server::training::{BotBehaviorType, TrainingConfig, TrainingCoordinator};

// =============================================================================
// T061: Dummy bots stay stationary
// =============================================================================

#[test]
fn test_dummy_bots_stationary_in_coordinator() {
    let mut config = TrainingConfig::default();
    config.bot_count = 3;
    config.bot_behavior = BotBehaviorType::Dummy;

    let spawn_points = vec![
        Vec3::new(10.0, 2.0, 10.0),
        Vec3::new(20.0, 2.0, 20.0),
        Vec3::new(30.0, 2.0, 30.0),
    ];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points.clone(), Tick(0));

    // Store initial positions
    let initial_positions: Vec<Vec3> = coordinator.bots.iter().map(|b| b.position).collect();

    // Run many ticks
    for tick in 1..500 {
        coordinator.tick(Tick(tick));
    }

    // Verify positions unchanged
    for (i, bot) in coordinator.bots.iter().enumerate() {
        assert_eq!(
            bot.position, initial_positions[i],
            "Dummy bot {} should not move",
            i
        );
    }
}

// =============================================================================
// T062: Roaming bots move within radius
// =============================================================================

#[test]
fn test_roam_bots_move_within_radius() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.bot_behavior = BotBehaviorType::Roam;

    let spawn_point = Vec3::new(10.0, 2.0, 10.0);
    let spawn_points = vec![spawn_point];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    let max_radius = 5.0; // Default roam radius

    // Run many ticks
    for tick in 1..1000 {
        coordinator.tick(Tick(tick));

        // Verify bot stays within radius
        let bot_pos = coordinator.bots[0].position;
        let delta = bot_pos - spawn_point;
        let horizontal_dist = (delta.x * delta.x + delta.z * delta.z).sqrt();

        assert!(
            horizontal_dist <= max_radius + 0.2,
            "Roaming bot exceeded radius at tick {}: dist={:.2}, max={}",
            tick,
            horizontal_dist,
            max_radius
        );
    }
}

#[test]
fn test_roam_bots_actually_move() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.bot_behavior = BotBehaviorType::Roam;

    let spawn_point = Vec3::new(10.0, 2.0, 10.0);
    let spawn_points = vec![spawn_point];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Run enough ticks to ensure movement
    for tick in 1..200 {
        coordinator.tick(Tick(tick));
    }

    let final_pos = coordinator.bots[0].position;
    let moved = (final_pos - spawn_point).length() > 0.1;

    assert!(moved, "Roaming bot should have moved from spawn point");
}

// =============================================================================
// T063: Strafing bots oscillate
// =============================================================================

#[test]
fn test_strafe_bots_oscillate() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.bot_behavior = BotBehaviorType::Strafe;

    let spawn_point = Vec3::new(10.0, 2.0, 10.0);
    let spawn_points = vec![spawn_point];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    let mut x_values: Vec<f32> = Vec::new();

    // Run ticks and collect X positions
    for tick in 1..300 {
        coordinator.tick(Tick(tick));
        x_values.push(coordinator.bots[0].position.x);
    }

    // Verify oscillation occurred
    let min_x = x_values.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_x = x_values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let oscillation_range = max_x - min_x;

    assert!(
        oscillation_range > 2.0,
        "Strafing bot should oscillate: range={:.2}",
        oscillation_range
    );
}

#[test]
fn test_strafe_bots_maintain_z_and_y() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.bot_behavior = BotBehaviorType::Strafe;

    let spawn_point = Vec3::new(10.0, 2.0, 10.0);
    let spawn_points = vec![spawn_point];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Run ticks
    for tick in 1..200 {
        coordinator.tick(Tick(tick));

        let pos = coordinator.bots[0].position;
        assert_eq!(pos.y, spawn_point.y, "Y should not change during strafe");
        assert_eq!(pos.z, spawn_point.z, "Z should not change during strafe");
    }
}

// =============================================================================
// T064-T069: Behavior configuration and updates
// =============================================================================

#[test]
fn test_behavior_type_applied_to_all_bots() {
    let mut config = TrainingConfig::default();
    config.bot_count = 5;
    config.bot_behavior = BotBehaviorType::Strafe;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0), Vec3::new(20.0, 2.0, 20.0)];
    let coordinator = TrainingCoordinator::new(config.clone(), spawn_points, Tick(0));

    // All bots should have strafe behavior
    for bot in &coordinator.bots {
        assert!(
            matches!(
                bot.behavior,
                plix_server::training::BotBehavior::Strafe { .. }
            ),
            "All bots should have Strafe behavior"
        );
    }
}

#[test]
fn test_behavior_reset_on_session_reset() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.bot_behavior = BotBehaviorType::Strafe;

    let spawn_point = Vec3::new(10.0, 2.0, 10.0);
    let spawn_points = vec![spawn_point];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Run some ticks to change behavior state
    for tick in 1..100 {
        coordinator.tick(Tick(tick));
    }

    let pos_before_reset = coordinator.bots[0].position;

    // Reset session
    coordinator.reset(Tick(100));

    let pos_after_reset = coordinator.bots[0].position;

    // Position should be back at spawn
    assert_eq!(pos_after_reset, spawn_point);

    // Behavior phase should be reset (bot at center X)
    assert!(
        (pos_after_reset.x - spawn_point.x).abs() < 0.1,
        "Strafe phase should reset to 0 (bot at center)"
    );
}

#[test]
fn test_bot_count_configuration() {
    for count in [0, 1, 5, 10, 20] {
        let mut config = TrainingConfig::default();
        config.bot_count = count;

        let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
        let coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

        assert_eq!(
            coordinator.bots.len(),
            count as usize,
            "Should spawn {} bots",
            count
        );
    }
}

#[test]
fn test_dead_bots_dont_move() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;
    config.bot_behavior = BotBehaviorType::Strafe;
    config.bot_respawn_delay_ticks = 1000; // Long delay

    let spawn_point = Vec3::new(10.0, 2.0, 10.0);
    let spawn_points = vec![spawn_point];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Kill the bot
    coordinator.process_hit(BotId(0), 100, Tick(10));
    assert!(coordinator.bots[0].is_dead);

    let pos_after_death = coordinator.bots[0].position;

    // Run ticks - dead bot should not move
    for tick in 11..100 {
        coordinator.tick(Tick(tick));
        assert_eq!(
            coordinator.bots[0].position, pos_after_death,
            "Dead bot should not move"
        );
    }
}
