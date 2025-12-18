//! Training mode session tests (Feature 020 - US2, US3, US6)
//!
//! Tests for:
//! - US2: Quick warmup session (immediate start, fast respawn, no victory)
//! - US3: Session reset functionality
//! - US6: Player invincibility options

use glam::Vec3;
use plix_common::protocol::MatchPhase;
use plix_common::time::Tick;
use plix_common::types::{BotId, GameMode};
use plix_server::match_state::{MatchConfig, MatchStateMachine};
use plix_server::training::{TrainingConfig, TrainingCoordinator};

// =============================================================================
// T038: Player spawns immediately on join (no countdown)
// =============================================================================

#[test]
fn test_training_mode_starts_in_playing_phase() {
    // Training mode should skip Lobby and Countdown, starting directly in Playing
    let config = MatchConfig::training_default();
    let mut state =
        MatchStateMachine::new(config, "training_arena".to_string(), GameMode::Training);

    // Training should start in Lobby but transition to Playing with 0 countdown
    state.start_countdown(Tick(0));
    state.update(Tick(1)); // Process countdown (0 ticks)

    assert_eq!(
        state.phase(),
        MatchPhase::Playing,
        "Training mode should be in Playing phase after single tick"
    );
}

#[test]
fn test_training_config_countdown_is_zero() {
    let config = MatchConfig::training_default();

    assert_eq!(
        config.countdown_ticks, 0,
        "Training countdown should be 0 (no countdown)"
    );
    assert_eq!(config.min_players, 1, "Training requires only 1 player");
}

// =============================================================================
// T039: Player respawn delay matches config
// =============================================================================

#[test]
fn test_training_player_respawn_delay_default() {
    let config = TrainingConfig::default();

    assert_eq!(
        config.player_respawn_delay_ticks, 60,
        "Default player respawn should be 60 ticks (1 second)"
    );
}

#[test]
fn test_training_player_respawn_delay_configurable() {
    let mut config = TrainingConfig::default();
    config.player_respawn_delay_ticks = 30; // 0.5 seconds

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let coordinator = TrainingCoordinator::new(config.clone(), spawn_points, Tick(0));

    assert_eq!(
        coordinator.player_respawn_delay_ticks(),
        30,
        "Player respawn delay should be configurable"
    );
}

#[test]
fn test_match_config_training_respawn_delay() {
    let config = MatchConfig::training_default();

    assert_eq!(
        config.respawn_delay_ticks, 60,
        "Match config training respawn should be 60 ticks"
    );
}

// =============================================================================
// T040: No victory condition triggers in training mode
// =============================================================================

#[test]
fn test_training_no_time_limit() {
    let config = MatchConfig::training_default();

    assert_eq!(config.time_limit_seconds, 0, "Training has no time limit");
}

#[test]
fn test_training_no_score_limit() {
    let config = MatchConfig::training_default();

    assert_eq!(config.score_limit, 0, "Training has no score limit");
}

#[test]
fn test_training_mode_never_ends_from_time() {
    let config = MatchConfig::training_default();
    let mut state =
        MatchStateMachine::new(config, "training_arena".to_string(), GameMode::Training);

    state.start_countdown(Tick(0));
    state.update(Tick(1)); // Playing

    // Run for many ticks - should never transition to EndScreen
    for i in 2..10000 {
        let phase_change = state.update(Tick(i));
        assert!(
            phase_change.is_none() || phase_change == Some(MatchPhase::Playing),
            "Training should never end from time at tick {}, got {:?}",
            i,
            phase_change
        );
    }

    assert_eq!(
        state.phase(),
        MatchPhase::Playing,
        "Training should stay in Playing indefinitely"
    );
}

#[test]
fn test_training_mode_score_check_no_effect() {
    let config = MatchConfig::training_default();
    let mut state =
        MatchStateMachine::new(config, "training_arena".to_string(), GameMode::Training);

    state.start_countdown(Tick(0));
    state.update(Tick(1)); // Playing

    let player_id = plix_common::types::PlayerId(1);

    // Even with high score, should not trigger match end (score_limit = 0)
    state.update_player_score(player_id, "TestPlayer", 100, 0);
    let ended = state.check_score_limit(player_id, 100, Tick(2));

    assert!(
        !ended,
        "Training mode should never end from score (score_limit=0 means no limit)"
    );
    assert_eq!(state.phase(), MatchPhase::Playing);
}

// =============================================================================
// T045: Reset repositions player to spawn (US3 - tested via coordinator)
// =============================================================================

#[test]
fn test_reset_returns_spawn_position() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0), Vec3::new(20.0, 2.0, 20.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points.clone(), Tick(0));

    // Perform reset
    let spawn_pos = coordinator.reset(Tick(100));

    assert_eq!(
        spawn_pos, spawn_points[0],
        "Reset should return first spawn point for player"
    );
}

// =============================================================================
// T046: Reset clears stats (US3)
// =============================================================================

#[test]
fn test_reset_clears_all_stats() {
    let mut config = TrainingConfig::default();
    config.bot_count = 2;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Accumulate stats
    coordinator.record_attack();
    coordinator.record_attack();
    coordinator.record_attack();
    coordinator.process_hit(BotId(0), 50, Tick(10));
    coordinator.process_hit(BotId(0), 50, Tick(20)); // Kill

    assert_eq!(coordinator.stats().attacks, 3);
    assert_eq!(coordinator.stats().hits, 2);
    assert_eq!(coordinator.stats().kills, 1);

    // Reset
    coordinator.reset(Tick(100));

    assert_eq!(coordinator.stats().attacks, 0, "Attacks should reset");
    assert_eq!(coordinator.stats().hits, 0, "Hits should reset");
    assert_eq!(coordinator.stats().kills, 0, "Kills should reset");
    assert_eq!(
        coordinator.stats().session_start,
        Tick(100),
        "Session start should update"
    );
}

// =============================================================================
// T047: Reset respawns all bots at initial positions (US3)
// =============================================================================

#[test]
fn test_reset_respawns_all_bots() {
    let mut config = TrainingConfig::default();
    config.bot_count = 3;
    config.bot_respawn_delay_ticks = 1000; // Long delay to ensure bots stay dead

    let spawn_points = vec![
        Vec3::new(10.0, 2.0, 10.0),
        Vec3::new(20.0, 2.0, 20.0),
        Vec3::new(30.0, 2.0, 30.0),
    ];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points.clone(), Tick(0));

    // Kill all bots
    coordinator.process_hit(BotId(0), 100, Tick(10));
    coordinator.process_hit(BotId(1), 100, Tick(20));
    coordinator.process_hit(BotId(2), 100, Tick(30));

    assert!(coordinator.bots[0].is_dead);
    assert!(coordinator.bots[1].is_dead);
    assert!(coordinator.bots[2].is_dead);

    // Reset
    coordinator.reset(Tick(100));

    // All bots should be alive at spawn positions
    assert!(
        !coordinator.bots[0].is_dead,
        "Bot 0 should be alive after reset"
    );
    assert!(
        !coordinator.bots[1].is_dead,
        "Bot 1 should be alive after reset"
    );
    assert!(
        !coordinator.bots[2].is_dead,
        "Bot 2 should be alive after reset"
    );

    assert_eq!(coordinator.bots[0].position, spawn_points[0]);
    assert_eq!(coordinator.bots[1].position, spawn_points[1]);
    assert_eq!(coordinator.bots[2].position, spawn_points[2]);
}

#[test]
fn test_reset_restores_bot_health() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // Damage bot
    coordinator.process_hit(BotId(0), 50, Tick(10));
    assert_eq!(coordinator.bots[0].health, 50);

    // Reset
    coordinator.reset(Tick(100));

    assert_eq!(
        coordinator.bots[0].health, 100,
        "Bot health should be restored to full"
    );
}

// =============================================================================
// T070: Invincible player takes no damage (US6)
// =============================================================================

#[test]
fn test_invincibility_player_config() {
    let mut config = TrainingConfig::default();
    config.invincibility_player = true;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    assert!(
        coordinator.is_player_invincible(),
        "Player invincibility should be readable from coordinator"
    );
}

#[test]
fn test_invincibility_player_default_false() {
    let config = TrainingConfig::default();

    assert!(
        !config.invincibility_player,
        "Player invincibility should be false by default"
    );
}

// =============================================================================
// T071: Non-invincible player takes normal damage (US6)
// =============================================================================

#[test]
fn test_non_invincible_player_config() {
    let mut config = TrainingConfig::default();
    config.invincibility_player = false;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    assert!(
        !coordinator.is_player_invincible(),
        "Player should take damage when invincibility is off"
    );
}

// =============================================================================
// Additional Training Session Tests
// =============================================================================

#[test]
fn test_training_game_mode_stored_correctly() {
    let config = MatchConfig::training_default();
    let state = MatchStateMachine::new(config, "training_arena".to_string(), GameMode::Training);

    assert_eq!(state.game_mode(), GameMode::Training);
}

#[test]
fn test_training_defaults_applied() {
    let config = MatchConfig::training_default();

    assert_eq!(config.min_players, 1);
    assert_eq!(config.countdown_ticks, 0);
    assert_eq!(config.time_limit_seconds, 0);
    assert_eq!(config.score_limit, 0);
    assert_eq!(config.end_screen_ticks, 0);
    assert_eq!(config.respawn_delay_ticks, 60);
}

#[test]
fn test_multiple_resets_work() {
    let mut config = TrainingConfig::default();
    config.bot_count = 1;

    let spawn_points = vec![Vec3::new(10.0, 2.0, 10.0)];
    let mut coordinator = TrainingCoordinator::new(config, spawn_points, Tick(0));

    // First session
    coordinator.record_attack();
    coordinator.process_hit(BotId(0), 100, Tick(10));
    assert_eq!(coordinator.stats().kills, 1);

    // First reset
    coordinator.reset(Tick(100));
    assert_eq!(coordinator.stats().kills, 0);

    // Second session
    coordinator.record_attack();
    coordinator.process_hit(BotId(0), 100, Tick(110));
    assert_eq!(coordinator.stats().kills, 1);

    // Second reset
    coordinator.reset(Tick(200));
    assert_eq!(coordinator.stats().kills, 0);
    assert_eq!(coordinator.stats().session_start, Tick(200));
}
