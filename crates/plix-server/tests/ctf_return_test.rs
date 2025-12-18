//! CTF flag return tests (T033-T034)
//!
//! Tests for flag return mechanics: teammate touch and auto-return timer

use plix_common::math::Vec3;
use plix_common::time::Tick;
use plix_common::types::{FlagZone, FlagZoneType, PlayerId, TeamId};
use plix_server::ctf::{CtfConfig, CtfCoordinator, CtfEvent, CtfRules, CtfState, ReturnReason};

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

/// T033: Unit test - teammate touch returns dropped flag
#[test]
fn test_teammate_touch_returns_dropped_flag() {
    let mut state = CtfState::new(test_zones(), CtfConfig::default());
    let enemy_carrier = PlayerId(1);
    let teammate = PlayerId(2);
    let current_tick = Tick(100);

    // Enemy picks up team 0's flag
    let pickup_pos = Vec3::new(5.0, 2.0, 5.0);
    assert!(CtfRules::pickup(
        enemy_carrier,
        TeamId::TEAM_1,
        pickup_pos,
        &mut state
    ));

    // Enemy drops flag (dies)
    let drop_pos = Vec3::new(25.0, 2.0, 25.0);
    CtfRules::drop(enemy_carrier, drop_pos, current_tick, &mut state);
    assert!(state.flag(TeamId::TEAM_0).state.is_dropped());

    // Team 0 teammate touches the dropped flag
    assert!(CtfRules::can_return(TeamId::TEAM_0, drop_pos, &state));
    assert!(CtfRules::return_flag(TeamId::TEAM_0, &mut state));

    // Flag should be back at base
    assert!(state.flag(TeamId::TEAM_0).is_at_base());
}

/// T034: Unit test - auto-return on timer expiry
#[test]
fn test_auto_return_on_timer_expiry() {
    let config = CtfConfig::default();
    let return_delay = config.flag_return_delay_ticks;
    let mut state = CtfState::new(test_zones(), config);
    let carrier = PlayerId(1);
    let drop_tick = Tick(100);

    // Carrier picks up and drops flag
    let pickup_pos = Vec3::new(5.0, 2.0, 5.0);
    assert!(CtfRules::pickup(
        carrier,
        TeamId::TEAM_1,
        pickup_pos,
        &mut state
    ));

    let drop_pos = Vec3::new(25.0, 2.0, 25.0);
    CtfRules::drop(carrier, drop_pos, drop_tick, &mut state);

    // Before timer expiry - flag should stay dropped
    let before_expiry = Tick(drop_tick.0 + return_delay - 1);
    let returned = CtfRules::update_return_timers(before_expiry, &mut state);
    assert!(returned.is_empty());
    assert!(state.flag(TeamId::TEAM_0).state.is_dropped());

    // At timer expiry - flag should auto-return
    let at_expiry = Tick(drop_tick.0 + return_delay);
    let returned = CtfRules::update_return_timers(at_expiry, &mut state);
    assert_eq!(returned.len(), 1);
    assert_eq!(returned[0], TeamId::TEAM_0);
    assert!(state.flag(TeamId::TEAM_0).is_at_base());
}

/// Test: Teammate cannot return flag that's at base
#[test]
fn test_cannot_return_flag_at_base() {
    let state = CtfState::new(test_zones(), CtfConfig::default());

    // Flag is at base - cannot "return" it
    let base_pos = Vec3::new(5.0, 2.0, 5.0);
    assert!(!CtfRules::can_return(TeamId::TEAM_0, base_pos, &state));
}

/// Test: Enemy cannot return dropped flag
#[test]
fn test_enemy_cannot_return_dropped_flag() {
    let mut state = CtfState::new(test_zones(), CtfConfig::default());
    let carrier = PlayerId(1);
    let current_tick = Tick(100);

    // Team 1 player picks up team 0's flag
    let pickup_pos = Vec3::new(5.0, 2.0, 5.0);
    assert!(CtfRules::pickup(
        carrier,
        TeamId::TEAM_1,
        pickup_pos,
        &mut state
    ));

    // Flag is dropped
    let drop_pos = Vec3::new(25.0, 2.0, 25.0);
    CtfRules::drop(carrier, drop_pos, current_tick, &mut state);

    // Team 1 cannot return team 0's flag (they're enemies)
    assert!(!CtfRules::can_return(TeamId::TEAM_1, drop_pos, &state));
}

/// Test: Coordinator generates return event on teammate touch
#[test]
fn test_coordinator_teammate_return_event() {
    let state = CtfState::new(test_zones(), CtfConfig::default());
    let mut coordinator = CtfCoordinator::new(state);
    let enemy_carrier = PlayerId(1);
    let teammate = PlayerId(2);
    let current_tick = Tick(100);

    // Enemy picks up and drops team 0's flag
    let pickup_pos = Vec3::new(5.0, 2.0, 5.0);
    coordinator.on_player_position(enemy_carrier, TeamId::TEAM_1, pickup_pos, current_tick);

    let drop_pos = Vec3::new(25.0, 2.0, 25.0);
    coordinator.on_player_death(enemy_carrier, drop_pos, current_tick);

    // Team 0 teammate moves to dropped flag position
    let events = coordinator.on_player_position(teammate, TeamId::TEAM_0, drop_pos, current_tick);

    // Should have return event
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        CtfEvent::FlagReturn {
            flag_team,
            reason
        } if flag_team == TeamId::TEAM_0 && reason == ReturnReason::TeammateTouch
    ));
}

/// Test: Coordinator generates return event on timer expiry
#[test]
fn test_coordinator_timer_return_event() {
    let config = CtfConfig::default();
    let return_delay = config.flag_return_delay_ticks;
    let state = CtfState::new(test_zones(), config);
    let mut coordinator = CtfCoordinator::new(state);
    let carrier = PlayerId(1);
    let drop_tick = Tick(100);

    // Enemy picks up and drops team 0's flag
    let pickup_pos = Vec3::new(5.0, 2.0, 5.0);
    coordinator.on_player_position(carrier, TeamId::TEAM_1, pickup_pos, drop_tick);

    let drop_pos = Vec3::new(25.0, 2.0, 25.0);
    coordinator.on_player_death(carrier, drop_pos, drop_tick);

    // Tick at timer expiry
    let expiry_tick = Tick(drop_tick.0 + return_delay);
    let events = coordinator.tick(expiry_tick);

    // Should have return event with timeout reason
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        CtfEvent::FlagReturn {
            flag_team,
            reason
        } if flag_team == TeamId::TEAM_0 && reason == ReturnReason::Timeout
    ));
}

/// Test: Out of bounds flag returns immediately
#[test]
fn test_out_of_bounds_returns_flag() {
    let config = CtfConfig::default();
    let mut state = CtfState::new(test_zones(), config);
    let mut coordinator = CtfCoordinator::new(state.clone());
    let carrier = PlayerId(1);
    let current_tick = Tick(100);

    // Setup: carrier picks up and drops flag
    let pickup_pos = Vec3::new(5.0, 2.0, 5.0);
    assert!(CtfRules::pickup(
        carrier,
        TeamId::TEAM_1,
        pickup_pos,
        &mut state
    ));

    // Drop at out-of-bounds position (simulating edge case)
    let drop_pos = Vec3::new(-10.0, 2.0, 25.0);
    CtfRules::drop(carrier, drop_pos, current_tick, &mut state);

    // Check out of bounds with arena bounds
    let arena_min = Vec3::new(0.0, 0.0, 0.0);
    let arena_max = Vec3::new(64.0, 32.0, 64.0);

    let returned = CtfRules::check_out_of_bounds(TeamId::TEAM_0, arena_min, arena_max, &mut state);
    assert!(returned);
    assert!(state.flag(TeamId::TEAM_0).is_at_base());
}

/// Test: Coordinator out of bounds check
#[test]
fn test_coordinator_out_of_bounds_event() {
    let state = CtfState::new(test_zones(), CtfConfig::default());
    let mut coordinator = CtfCoordinator::new(state);
    let carrier = PlayerId(1);
    let current_tick = Tick(100);

    // Setup: carrier picks up and drops flag
    let pickup_pos = Vec3::new(5.0, 2.0, 5.0);
    coordinator.on_player_position(carrier, TeamId::TEAM_1, pickup_pos, current_tick);

    // Simulate death at out-of-bounds position
    let oob_pos = Vec3::new(-10.0, 2.0, 25.0);
    coordinator.on_player_death(carrier, oob_pos, current_tick);

    // Check bounds
    let arena_min = Vec3::new(0.0, 0.0, 0.0);
    let arena_max = Vec3::new(64.0, 32.0, 64.0);
    let events = coordinator.check_out_of_bounds(arena_min, arena_max);

    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        CtfEvent::FlagReturn {
            flag_team,
            reason
        } if flag_team == TeamId::TEAM_0 && reason == ReturnReason::OutOfBounds
    ));
}
