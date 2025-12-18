//! BR Lite elimination integration tests
//!
//! Tests for User Story 1: Last Player Standing Victory

use glam::Vec2;
use plix_common::time::Tick;
use plix_common::types::PlayerId;
use plix_server::br_lite::{BrLiteConfig, BrLiteCoordinator, BrLiteEvent, MatchPhase, ZonePhase};

fn test_config() -> BrLiteConfig {
    BrLiteConfig {
        min_players: 2,
        post_match_delay_secs: 5,
        phases: vec![
            ZonePhase::new(60, 30, 0.8, 5),
            ZonePhase::new(45, 25, 0.5, 10),
        ],
        initial_radius: Some(50.0),
        ..Default::default()
    }
}

/// T024: Test elimination marks player correctly
#[test]
fn test_elimination_marks_player_correctly() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    // Add players
    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.add_player(PlayerId(3));

    // Start match
    coordinator.start_match();
    assert_eq!(coordinator.match_phase(), MatchPhase::Playing);
    assert_eq!(coordinator.alive_count(), 3);

    // Eliminate player 3
    let events = coordinator.eliminate_player(PlayerId(3));

    // Verify elimination event
    assert!(!events.is_empty());
    let has_elimination = events.iter().any(|e| {
        matches!(e, BrLiteEvent::Elimination { player_id, alive_count }
            if *player_id == PlayerId(3) && *alive_count == 2)
    });
    assert!(has_elimination, "Expected elimination event for player 3");

    // Verify player state
    assert!(!coordinator.is_alive(PlayerId(3)));
    assert!(coordinator.is_alive(PlayerId(1)));
    assert!(coordinator.is_alive(PlayerId(2)));
    assert_eq!(coordinator.alive_count(), 2);
}

/// T025: Test victory at alive_count == 1
#[test]
fn test_victory_at_one_alive() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    // Add players
    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    // Eliminate player 2
    let events = coordinator.eliminate_player(PlayerId(2));

    // Should have both elimination and victory events
    assert!(events.len() >= 2);

    let has_elimination = events.iter().any(|e| {
        matches!(e, BrLiteEvent::Elimination { player_id, alive_count }
            if *player_id == PlayerId(2) && *alive_count == 1)
    });
    assert!(has_elimination);

    let has_victory = events
        .iter()
        .any(|e| matches!(e, BrLiteEvent::Victory { winner_id } if *winner_id == PlayerId(1)));
    assert!(has_victory, "Expected victory event for player 1");

    // Verify match state
    assert_eq!(coordinator.match_phase(), MatchPhase::PostMatch);
    assert_eq!(coordinator.winner(), Some(PlayerId(1)));
}

/// T026: Test simultaneous death edge case (lowest ID wins)
#[test]
fn test_simultaneous_death_lowest_id_wins() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    // Add players with non-sequential IDs
    coordinator.add_player(PlayerId(5));
    coordinator.add_player(PlayerId(3));
    coordinator.start_match();

    // Eliminate both players rapidly (simulating simultaneous death)
    coordinator.eliminate_player(PlayerId(5));
    let events = coordinator.eliminate_player(PlayerId(3));

    // Should have victory event with lowest ID (3) as winner
    let has_victory = events
        .iter()
        .any(|e| matches!(e, BrLiteEvent::Victory { winner_id } if *winner_id == PlayerId(3)));
    assert!(has_victory, "Expected player 3 (lowest ID) to win");

    assert_eq!(coordinator.winner(), Some(PlayerId(3)));
}

/// T027: Test disconnect = elimination
#[test]
fn test_disconnect_causes_elimination() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    // Add players
    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.add_player(PlayerId(3));
    coordinator.start_match();

    // Player 2 disconnects during match
    let events = coordinator.remove_player(PlayerId(2));

    // Should generate elimination event
    let has_elimination = events.iter().any(
        |e| matches!(e, BrLiteEvent::Elimination { player_id, .. } if *player_id == PlayerId(2)),
    );
    assert!(has_elimination, "Disconnect should cause elimination");

    assert!(!coordinator.is_alive(PlayerId(2)));
    assert_eq!(coordinator.alive_count(), 2);
}

/// Additional test: Disconnect in lobby doesn't eliminate
#[test]
fn test_disconnect_in_lobby_removes_without_elimination() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    // Add players but don't start match
    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    assert_eq!(coordinator.match_phase(), MatchPhase::Lobby);

    // Player 2 disconnects in lobby
    let events = coordinator.remove_player(PlayerId(2));

    // Should NOT generate elimination event (not in match yet)
    assert!(events.is_empty());
    assert_eq!(coordinator.alive_count(), 1);
}

/// Additional test: Victory event only fires once
#[test]
fn test_victory_fires_once() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();

    // Eliminate to trigger victory
    let events1 = coordinator.eliminate_player(PlayerId(2));
    assert!(events1
        .iter()
        .any(|e| matches!(e, BrLiteEvent::Victory { .. })));

    // Try to eliminate already-dead player (should be no-op)
    let events2 = coordinator.eliminate_player(PlayerId(2));
    assert!(
        events2.is_empty(),
        "Eliminating already-dead player should be no-op"
    );
}

/// Additional test: Match doesn't start without minimum players
#[test]
fn test_match_requires_minimum_players() {
    let config = BrLiteConfig {
        min_players: 4,
        ..test_config()
    };
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));

    // Try to start with 2 players (min is 4)
    let event = coordinator.try_start_match();
    assert!(event.is_none());
    assert_eq!(coordinator.match_phase(), MatchPhase::Lobby);

    // Add more players
    coordinator.add_player(PlayerId(3));
    coordinator.add_player(PlayerId(4));

    // Now it should work
    let event = coordinator.try_start_match();
    assert!(event.is_some());
    assert_eq!(coordinator.match_phase(), MatchPhase::Playing);
}

/// Additional test: Post-match timer works correctly
#[test]
fn test_post_match_timer() {
    let config = BrLiteConfig {
        post_match_delay_secs: 2, // 2 seconds = 120 ticks
        ..test_config()
    };
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();
    coordinator.eliminate_player(PlayerId(2));

    assert_eq!(coordinator.match_phase(), MatchPhase::PostMatch);
    assert!(!coordinator.is_post_match_complete());

    // Tick through post-match period
    let positions = std::collections::HashMap::new();
    for _ in 0..120 {
        coordinator.tick(Tick(0), &positions);
    }

    assert!(coordinator.is_post_match_complete());
}

/// Additional test: Reset properly clears state
#[test]
fn test_reset_clears_state() {
    let config = test_config();
    let mut coordinator = BrLiteCoordinator::new(config, Vec2::new(64.0, 64.0));

    coordinator.add_player(PlayerId(1));
    coordinator.add_player(PlayerId(2));
    coordinator.start_match();
    coordinator.eliminate_player(PlayerId(2));

    // Reset
    coordinator.reset();

    assert_eq!(coordinator.match_phase(), MatchPhase::Lobby);
    assert_eq!(coordinator.alive_count(), 0);
    assert!(coordinator.winner().is_none());
}
