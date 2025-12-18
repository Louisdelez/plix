//! CTF flag pickup tests (T019-T020)
//!
//! Tests for flag pickup logic: valid pickups (enemy flag) and invalid pickups (same team)

use plix_common::math::Vec3;
use plix_common::types::{FlagZone, FlagZoneType, PlayerId, TeamId};
use plix_server::ctf::{CtfConfig, CtfRules, CtfState};

fn test_zones() -> Vec<FlagZone> {
    vec![
        // Team 0 flag base (where team 0's flag sits)
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

/// T019: Unit test - flag pickup valid (enemy player in flag zone)
#[test]
fn test_enemy_can_pickup_flag_in_base() {
    let mut state = CtfState::new(test_zones(), CtfConfig::default());
    let team1_player = PlayerId(1);

    // Team 1 player at team 0's flag base
    let position = Vec3::new(5.0, 2.0, 5.0);

    // Team 1 player can pick up team 0's flag
    assert!(CtfRules::can_pickup(team1_player, TeamId::TEAM_1, position, &state).is_some());

    // Actually pickup
    assert!(CtfRules::pickup(
        team1_player,
        TeamId::TEAM_1,
        position,
        &mut state
    ));

    // Flag should now be carried
    assert!(state.flag(TeamId::TEAM_0).is_carried());
    assert_eq!(state.flag(TeamId::TEAM_0).carrier(), Some(team1_player));
}

/// T020: Unit test - flag pickup invalid (same team)
#[test]
fn test_same_team_cannot_pickup_own_flag() {
    let state = CtfState::new(test_zones(), CtfConfig::default());
    let team0_player = PlayerId(1);

    // Team 0 player at team 0's flag base
    let position = Vec3::new(5.0, 2.0, 5.0);

    // Team 0 player cannot pick up their own flag
    assert!(CtfRules::can_pickup(team0_player, TeamId::TEAM_0, position, &state).is_none());
}

/// Test: Player cannot pickup flag if already carrying one
#[test]
fn test_cannot_pickup_second_flag() {
    let mut state = CtfState::new(test_zones(), CtfConfig::default());
    let team1_player = PlayerId(1);

    // First pickup team 0's flag
    let pos_team0_base = Vec3::new(5.0, 2.0, 5.0);
    assert!(CtfRules::pickup(
        team1_player,
        TeamId::TEAM_1,
        pos_team0_base,
        &mut state
    ));

    // Now try to pickup at team 1's base (shouldn't matter, already carrying)
    let pos_team1_base = Vec3::new(55.0, 2.0, 55.0);
    assert!(!CtfRules::pickup(
        team1_player,
        TeamId::TEAM_1,
        pos_team1_base,
        &mut state
    ));
}

/// Test: Cannot pickup flag that's being carried
#[test]
fn test_cannot_pickup_carried_flag() {
    let mut state = CtfState::new(test_zones(), CtfConfig::default());
    let team1_player1 = PlayerId(1);
    let team1_player2 = PlayerId(2);

    // Player 1 picks up team 0's flag
    let position = Vec3::new(5.0, 2.0, 5.0);
    assert!(CtfRules::pickup(
        team1_player1,
        TeamId::TEAM_1,
        position,
        &mut state
    ));

    // Player 2 cannot pick up the same flag (it's being carried)
    assert!(CtfRules::can_pickup(team1_player2, TeamId::TEAM_1, position, &state).is_none());
}

/// Test: Player must be in flag zone to pickup
#[test]
fn test_cannot_pickup_outside_zone() {
    let state = CtfState::new(test_zones(), CtfConfig::default());
    let team1_player = PlayerId(1);

    // Team 1 player outside of team 0's flag base
    let position = Vec3::new(25.0, 2.0, 25.0);

    // Cannot pickup - not in the flag zone
    assert!(CtfRules::can_pickup(team1_player, TeamId::TEAM_1, position, &state).is_none());
}
