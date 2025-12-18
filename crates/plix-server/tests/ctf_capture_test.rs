//! CTF flag capture tests (T021-T023)
//!
//! Tests for flag capture logic: successful capture, blocked capture, and flag reset

use plix_common::math::Vec3;
use plix_common::time::Tick;
use plix_common::types::{FlagState, FlagZone, FlagZoneType, PlayerId, TeamId};
use plix_server::ctf::{CtfConfig, CtfCoordinator, CtfEvent, CtfRules, CtfState};

fn test_zones() -> Vec<FlagZone> {
    vec![
        // Team 0 flag base
        FlagZone::new(
            TeamId::TEAM_0,
            FlagZoneType::FlagBase,
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 4.0, 10.0),
        ),
        // Team 1 flag base
        FlagZone::new(
            TeamId::TEAM_1,
            FlagZoneType::FlagBase,
            Vec3::new(50.0, 0.0, 50.0),
            Vec3::new(60.0, 4.0, 60.0),
        ),
        // Team 0 capture zone
        FlagZone::new(
            TeamId::TEAM_0,
            FlagZoneType::CaptureZone,
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(12.0, 4.0, 12.0),
        ),
        // Team 1 capture zone
        FlagZone::new(
            TeamId::TEAM_1,
            FlagZoneType::CaptureZone,
            Vec3::new(48.0, 0.0, 48.0),
            Vec3::new(60.0, 4.0, 60.0),
        ),
    ]
}

/// T021: Unit test - capture succeeds (own flag at base)
#[test]
fn test_capture_succeeds_with_own_flag_at_base() {
    let mut state = CtfState::new(test_zones(), CtfConfig::default());
    let team1_player = PlayerId(1);

    // Team 1 player picks up team 0's flag
    let enemy_base_pos = Vec3::new(5.0, 2.0, 5.0);
    assert!(CtfRules::pickup(
        team1_player,
        TeamId::TEAM_1,
        enemy_base_pos,
        &mut state
    ));

    // Move to team 1's capture zone
    let capture_pos = Vec3::new(55.0, 2.0, 55.0);

    // Team 1's flag is at base - capture should succeed
    assert!(state.flag(TeamId::TEAM_1).is_at_base());
    assert!(CtfRules::can_capture(
        team1_player,
        TeamId::TEAM_1,
        capture_pos,
        &state
    ));

    // Execute capture
    let new_score = CtfRules::capture(team1_player, TeamId::TEAM_1, &mut state);
    assert_eq!(new_score, 1);
    assert_eq!(state.score(TeamId::TEAM_1), 1);
}

/// T022: Unit test - capture blocked (own flag not at base)
#[test]
fn test_capture_blocked_when_own_flag_missing() {
    let mut state = CtfState::new(test_zones(), CtfConfig::default());
    let team0_player = PlayerId(1);
    let team1_player = PlayerId(2);

    // Team 0 player picks up team 1's flag
    let team1_flag_pos = Vec3::new(55.0, 2.0, 55.0);
    assert!(CtfRules::pickup(
        team0_player,
        TeamId::TEAM_0,
        team1_flag_pos,
        &mut state
    ));

    // Team 1 player picks up team 0's flag
    let team0_flag_pos = Vec3::new(5.0, 2.0, 5.0);
    assert!(CtfRules::pickup(
        team1_player,
        TeamId::TEAM_1,
        team0_flag_pos,
        &mut state
    ));

    // Team 1 player tries to capture - their own flag is NOT at base
    let capture_pos = Vec3::new(55.0, 2.0, 55.0);
    assert!(!state.flag(TeamId::TEAM_1).is_at_base());
    assert!(!CtfRules::can_capture(
        team1_player,
        TeamId::TEAM_1,
        capture_pos,
        &state
    ));
}

/// T023: Unit test - both flags reset after capture
#[test]
fn test_flags_reset_after_capture() {
    let mut state = CtfState::new(test_zones(), CtfConfig::default());
    let team1_player = PlayerId(1);

    // Team 1 player picks up team 0's flag
    let enemy_base_pos = Vec3::new(5.0, 2.0, 5.0);
    assert!(CtfRules::pickup(
        team1_player,
        TeamId::TEAM_1,
        enemy_base_pos,
        &mut state
    ));

    // Verify flag is being carried
    assert!(state.flag(TeamId::TEAM_0).is_carried());

    // Execute capture
    let capture_pos = Vec3::new(55.0, 2.0, 55.0);
    assert!(CtfRules::can_capture(
        team1_player,
        TeamId::TEAM_1,
        capture_pos,
        &state
    ));
    CtfRules::capture(team1_player, TeamId::TEAM_1, &mut state);

    // Both flags should be at base now
    assert!(state.flag(TeamId::TEAM_0).is_at_base());
    assert!(state.flag(TeamId::TEAM_1).is_at_base());
}

/// Test: Capture outside capture zone fails
#[test]
fn test_capture_outside_zone_fails() {
    let mut state = CtfState::new(test_zones(), CtfConfig::default());
    let team1_player = PlayerId(1);

    // Team 1 player picks up team 0's flag
    let enemy_base_pos = Vec3::new(5.0, 2.0, 5.0);
    assert!(CtfRules::pickup(
        team1_player,
        TeamId::TEAM_1,
        enemy_base_pos,
        &mut state
    ));

    // Position outside capture zone (in the middle of the arena)
    let middle_pos = Vec3::new(30.0, 2.0, 30.0);

    // Cannot capture outside the capture zone
    assert!(!CtfRules::can_capture(
        team1_player,
        TeamId::TEAM_1,
        middle_pos,
        &state
    ));
}

/// Test: Non-carrier cannot capture
#[test]
fn test_non_carrier_cannot_capture() {
    let mut state = CtfState::new(test_zones(), CtfConfig::default());
    let carrier = PlayerId(1);
    let non_carrier = PlayerId(2);

    // Carrier picks up flag
    let enemy_base_pos = Vec3::new(5.0, 2.0, 5.0);
    assert!(CtfRules::pickup(
        carrier,
        TeamId::TEAM_1,
        enemy_base_pos,
        &mut state
    ));

    // Non-carrier at capture zone
    let capture_pos = Vec3::new(55.0, 2.0, 55.0);

    // Non-carrier cannot capture
    assert!(!CtfRules::can_capture(
        non_carrier,
        TeamId::TEAM_1,
        capture_pos,
        &state
    ));
}

/// Test: Coordinator generates capture event
#[test]
fn test_coordinator_capture_event() {
    let state = CtfState::new(test_zones(), CtfConfig::default());
    let mut coordinator = CtfCoordinator::new(state);
    let team1_player = PlayerId(1);
    let current_tick = Tick(100);

    // First pickup at enemy base
    let enemy_base_pos = Vec3::new(5.0, 2.0, 5.0);
    let pickup_events =
        coordinator.on_player_position(team1_player, TeamId::TEAM_1, enemy_base_pos, current_tick);
    assert_eq!(pickup_events.len(), 1);
    assert!(matches!(
        pickup_events[0],
        CtfEvent::FlagPickup {
            player_id,
            flag_team
        } if player_id == team1_player && flag_team == TeamId::TEAM_0
    ));

    // Then capture at own base
    let capture_pos = Vec3::new(55.0, 2.0, 55.0);
    let capture_events =
        coordinator.on_player_position(team1_player, TeamId::TEAM_1, capture_pos, current_tick);
    assert_eq!(capture_events.len(), 1);
    assert!(matches!(
        capture_events[0],
        CtfEvent::FlagCapture {
            capturing_team,
            capturing_player,
            new_score
        } if capturing_team == TeamId::TEAM_1 && capturing_player == team1_player && new_score == 1
    ));
}
