//! CTF flag drop tests (T031-T032, T035)
//!
//! Tests for flag drop on death/disconnect and enemy pickup of dropped flags

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

/// T031: Unit test - death drops flag at carrier position
#[test]
fn test_death_drops_flag_at_position() {
    let mut state = CtfState::new(test_zones(), CtfConfig::default());
    let carrier = PlayerId(1);
    let current_tick = Tick(100);

    // Carrier picks up enemy flag
    let pickup_pos = Vec3::new(5.0, 2.0, 5.0);
    assert!(CtfRules::pickup(
        carrier,
        TeamId::TEAM_1,
        pickup_pos,
        &mut state
    ));
    assert!(state.flag(TeamId::TEAM_0).is_carried());

    // Carrier dies at this position
    let death_pos = Vec3::new(25.0, 2.0, 25.0);
    let dropped_flag = CtfRules::drop(carrier, death_pos, current_tick, &mut state);
    assert_eq!(dropped_flag, Some(TeamId::TEAM_0));

    // Flag should be dropped at death position
    assert!(state.flag(TeamId::TEAM_0).state.is_dropped());
    if let FlagState::Dropped { position, .. } = &state.flag(TeamId::TEAM_0).state {
        assert_eq!(*position, death_pos);
    } else {
        panic!("Flag should be in Dropped state");
    }
}

/// T032: Unit test - enemy can pickup dropped flag
#[test]
fn test_enemy_can_pickup_dropped_flag() {
    let mut state = CtfState::new(test_zones(), CtfConfig::default());
    let carrier = PlayerId(1);
    let other_enemy = PlayerId(2);
    let current_tick = Tick(100);

    // Carrier picks up enemy flag
    let pickup_pos = Vec3::new(5.0, 2.0, 5.0);
    assert!(CtfRules::pickup(
        carrier,
        TeamId::TEAM_1,
        pickup_pos,
        &mut state
    ));

    // Carrier dies
    let drop_pos = Vec3::new(25.0, 2.0, 25.0);
    CtfRules::drop(carrier, drop_pos, current_tick, &mut state);

    // Another enemy player can pick up the dropped flag
    assert!(CtfRules::can_pickup(other_enemy, TeamId::TEAM_1, drop_pos, &state).is_some());
    assert!(CtfRules::pickup(
        other_enemy,
        TeamId::TEAM_1,
        drop_pos,
        &mut state
    ));

    // Flag should now be carried by the other player
    assert!(state.flag(TeamId::TEAM_0).is_carried());
    assert_eq!(state.flag(TeamId::TEAM_0).carrier(), Some(other_enemy));
}

/// T035: Unit test - disconnect drops flag (same as death)
#[test]
fn test_disconnect_drops_flag() {
    let state = CtfState::new(test_zones(), CtfConfig::default());
    let mut coordinator = CtfCoordinator::new(state);
    let carrier = PlayerId(1);
    let current_tick = Tick(100);

    // Carrier picks up enemy flag
    let pickup_pos = Vec3::new(5.0, 2.0, 5.0);
    coordinator.on_player_position(carrier, TeamId::TEAM_1, pickup_pos, current_tick);

    // Carrier disconnects
    let disconnect_pos = Vec3::new(30.0, 2.0, 30.0);
    let events = coordinator.on_player_disconnect(carrier, disconnect_pos, current_tick);

    // Should have a FlagDrop event
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        CtfEvent::FlagDrop {
            player_id,
            flag_team,
            ..
        } if player_id == carrier && flag_team == TeamId::TEAM_0
    ));
}

/// Test: Dropping flag sets return timer
#[test]
fn test_drop_sets_return_timer() {
    let config = CtfConfig::default();
    let return_delay = config.flag_return_delay_ticks;
    let mut state = CtfState::new(test_zones(), config);
    let carrier = PlayerId(1);
    let current_tick = Tick(100);

    // Carrier picks up and then drops flag
    let pickup_pos = Vec3::new(5.0, 2.0, 5.0);
    assert!(CtfRules::pickup(
        carrier,
        TeamId::TEAM_1,
        pickup_pos,
        &mut state
    ));

    let drop_pos = Vec3::new(25.0, 2.0, 25.0);
    CtfRules::drop(carrier, drop_pos, current_tick, &mut state);

    // Check return timer is set correctly
    if let FlagState::Dropped { return_tick, .. } = &state.flag(TeamId::TEAM_0).state {
        assert_eq!(return_tick.0, current_tick.0 + return_delay);
    } else {
        panic!("Flag should be in Dropped state");
    }
}

/// Test: Non-carrier death doesn't drop flag
#[test]
fn test_non_carrier_death_no_drop() {
    let mut state = CtfState::new(test_zones(), CtfConfig::default());
    let non_carrier = PlayerId(1);
    let current_tick = Tick(100);

    // Non-carrier dies
    let death_pos = Vec3::new(25.0, 2.0, 25.0);
    let dropped_flag = CtfRules::drop(non_carrier, death_pos, current_tick, &mut state);

    // Nothing should be dropped
    assert_eq!(dropped_flag, None);
}

/// Test: Coordinator death event generates FlagDrop
#[test]
fn test_coordinator_death_drop_event() {
    let state = CtfState::new(test_zones(), CtfConfig::default());
    let mut coordinator = CtfCoordinator::new(state);
    let carrier = PlayerId(1);
    let current_tick = Tick(100);

    // Carrier picks up flag
    let pickup_pos = Vec3::new(5.0, 2.0, 5.0);
    coordinator.on_player_position(carrier, TeamId::TEAM_1, pickup_pos, current_tick);

    // Carrier dies
    let death_pos = Vec3::new(25.0, 2.0, 25.0);
    let events = coordinator.on_player_death(carrier, death_pos, current_tick);

    // Should have drop event with correct info
    assert_eq!(events.len(), 1);
    if let CtfEvent::FlagDrop {
        player_id,
        flag_team,
        position,
        return_tick,
    } = &events[0]
    {
        assert_eq!(*player_id, carrier);
        assert_eq!(*flag_team, TeamId::TEAM_0);
        assert_eq!(*position, death_pos);
        assert_eq!(
            return_tick.0,
            current_tick.0 + coordinator.config().flag_return_delay_ticks
        );
    } else {
        panic!("Expected FlagDrop event");
    }
}
