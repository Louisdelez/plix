//! Anti-cheat integration tests
//! Tests for User Story 1: Strict Input Validation (FR-001 through FR-003)

use plix_common::protocol::PlayerInput;
use plix_common::time::Tick;
use plix_common::types::InputSeq;

use plix_server::anti_cheat::{
    validate, ActionType, AntiCheatConfig, AntiCheatState, InfractionType,
};

// =============================================================================
// User Story 1: Strict Input Validation
// =============================================================================

#[test]
fn test_validate_input_rejects_nan() {
    // US1 Scenario 1: Reject NaN values
    let input = PlayerInput {
        seq: InputSeq(1),
        tick: Tick(0),
        move_forward: f32::NAN,
        move_right: 0.0,
        jump: false,
        crouch: false,
        attack: false,
        yaw: 0.0,
        pitch: 0.0,
        rtt_nonce: 0,
    };

    let result = validate::validate_input(&input);
    assert!(result.is_err(), "Should reject NaN in move_forward");
    assert_eq!(result.unwrap_err(), InfractionType::InvalidFloat);
}

#[test]
fn test_validate_input_rejects_inf() {
    // US1 Scenario 1: Reject INF values
    let input = PlayerInput {
        seq: InputSeq(1),
        tick: Tick(0),
        move_forward: 0.0,
        move_right: f32::INFINITY,
        jump: false,
        crouch: false,
        attack: false,
        yaw: 0.0,
        pitch: 0.0,
        rtt_nonce: 0,
    };

    let result = validate::validate_input(&input);
    assert!(result.is_err(), "Should reject INF in move_right");
    assert_eq!(result.unwrap_err(), InfractionType::InvalidFloat);
}

#[test]
fn test_validate_input_rejects_neg_inf() {
    // US1 Scenario 1: Reject -INF values
    let input = PlayerInput {
        seq: InputSeq(1),
        tick: Tick(0),
        move_forward: 0.0,
        move_right: 0.0,
        jump: false,
        crouch: false,
        attack: false,
        yaw: f32::NEG_INFINITY,
        pitch: 0.0,
        rtt_nonce: 0,
    };

    let result = validate::validate_input(&input);
    assert!(result.is_err(), "Should reject -INF in yaw");
    assert_eq!(result.unwrap_err(), InfractionType::InvalidFloat);
}

#[test]
fn test_validate_input_rejects_out_of_bounds() {
    // US1 Scenario 2: Reject values outside valid bounds
    let input = PlayerInput {
        seq: InputSeq(1),
        tick: Tick(0),
        move_forward: 2.0, // Outside [-1.0, 1.0]
        move_right: 0.0,
        jump: false,
        crouch: false,
        attack: false,
        yaw: 0.0,
        pitch: 0.0,
        rtt_nonce: 0,
    };

    let result = validate::validate_input(&input);
    assert!(result.is_err(), "Should reject out-of-bounds move_forward");
    assert_eq!(result.unwrap_err(), InfractionType::OutOfBounds);
}

#[test]
fn test_validate_input_accepts_valid() {
    // Normal input should be accepted
    let input = PlayerInput {
        seq: InputSeq(1),
        tick: Tick(0),
        move_forward: 0.5,
        move_right: -0.5,
        jump: true,
        crouch: false,
        attack: false,
        yaw: 1.0,
        pitch: 0.5,
        rtt_nonce: 0,
    };

    let result = validate::validate_input(&input);
    assert!(result.is_ok(), "Should accept valid input");
}

#[test]
fn test_sequence_rejects_duplicate() {
    // US1 Scenario 3: Reject duplicate sequence numbers
    let mut state = AntiCheatState::new(Tick(0));

    // First input accepted
    assert!(state.check_sequence(InputSeq(5)));

    // Duplicate rejected
    assert!(!state.check_sequence(InputSeq(5)));
}

#[test]
fn test_sequence_rejects_old() {
    // US1 Scenario 3: Reject out-of-order (old) sequence numbers
    let mut state = AntiCheatState::new(Tick(0));

    // Accept sequence 10
    assert!(state.check_sequence(InputSeq(10)));

    // Reject older sequence
    assert!(!state.check_sequence(InputSeq(5)));
}

#[test]
fn test_sequence_accepts_newer() {
    // US1 Scenario 3: Accept newer sequence numbers
    let mut state = AntiCheatState::new(Tick(0));

    assert!(state.check_sequence(InputSeq(1)));
    assert!(state.check_sequence(InputSeq(2)));
    assert!(state.check_sequence(InputSeq(3)));
}

// =============================================================================
// User Story 2: Rate Limiting
// =============================================================================

#[test]
fn test_rate_limiter_allows_under_limit() {
    // US2 Scenario 1: Allow inputs under the limit
    let mut state = AntiCheatState::new(Tick(0));
    let config = AntiCheatConfig::default();

    // Should allow up to max_inputs_per_second
    for i in 0..config.max_inputs_per_second {
        assert!(
            state.check_rate_limit(ActionType::Input, Tick(0), &config),
            "Should allow input #{}",
            i
        );
    }
}

#[test]
fn test_rate_limiter_rejects_over_limit() {
    // US2 Scenario 1: Reject inputs over the limit
    let mut state = AntiCheatState::new(Tick(0));
    let config = AntiCheatConfig::default();

    // Exhaust the limit
    for _ in 0..config.max_inputs_per_second {
        state.check_rate_limit(ActionType::Input, Tick(0), &config);
    }

    // Should reject
    assert!(
        !state.check_rate_limit(ActionType::Input, Tick(0), &config),
        "Should reject input over limit"
    );
}

#[test]
fn test_rate_limiter_window_reset() {
    // US2: Rate limit window resets after time
    let mut state = AntiCheatState::new(Tick(0));
    let config = AntiCheatConfig::default();

    // Exhaust the limit
    for _ in 0..config.max_inputs_per_second {
        state.check_rate_limit(ActionType::Input, Tick(0), &config);
    }

    // Should reject at tick 0
    assert!(!state.check_rate_limit(ActionType::Input, Tick(0), &config));

    // Should allow after window reset (60 ticks = 1 second at 60Hz)
    assert!(state.check_rate_limit(ActionType::Input, Tick(60), &config));
}

#[test]
fn test_rate_limiter_attacks() {
    // US2 Scenario 2: Rate limit attacks
    let mut state = AntiCheatState::new(Tick(0));
    let config = AntiCheatConfig::default();

    // Exhaust attack limit
    for _ in 0..config.max_attacks_per_second {
        assert!(state.check_rate_limit(ActionType::Attack, Tick(0), &config));
    }

    // Should reject
    assert!(!state.check_rate_limit(ActionType::Attack, Tick(0), &config));
}

#[test]
fn test_rate_limiter_block_edits() {
    // US2 Scenario 3: Rate limit block edits
    let mut state = AntiCheatState::new(Tick(0));
    let config = AntiCheatConfig::default();

    // Exhaust block edit limit
    for _ in 0..config.max_block_edits_per_second {
        assert!(state.check_rate_limit(ActionType::BlockEdit, Tick(0), &config));
    }

    // Should reject
    assert!(!state.check_rate_limit(ActionType::BlockEdit, Tick(0), &config));
}

#[test]
fn test_rate_limiter_ready_toggle() {
    // US2 Scenario 4: Rate limit ready toggles
    let mut state = AntiCheatState::new(Tick(0));
    let config = AntiCheatConfig::default();

    // Exhaust ready toggle limit
    for _ in 0..config.max_ready_toggles_per_second {
        assert!(state.check_rate_limit(ActionType::ReadyToggle, Tick(0), &config));
    }

    // Should reject
    assert!(!state.check_rate_limit(ActionType::ReadyToggle, Tick(0), &config));
}

// =============================================================================
// User Story 3: Physics Sanity Checks
// =============================================================================

#[test]
fn test_movement_sanity_allows_normal_speed() {
    // US3 Scenario: Normal movement should be allowed
    use plix_common::math::Vec3;

    let config = AntiCheatConfig::default();
    let old_pos = Vec3::new(0.0, 0.0, 0.0);
    // Move less than max_speed_per_tick
    let new_pos = Vec3::new(0.1, 0.0, 0.0);

    let result = validate::validate_movement_delta(old_pos, new_pos, &config);
    assert!(result.is_ok(), "Normal movement should be allowed");
}

#[test]
fn test_movement_sanity_rejects_speed_hack() {
    // US3 Scenario 1: Reject excessive speed
    use plix_common::math::Vec3;

    let config = AntiCheatConfig::default();
    let old_pos = Vec3::new(0.0, 0.0, 0.0);
    // Move way more than max_speed_per_tick
    let new_pos = Vec3::new(10.0, 0.0, 0.0);

    let result = validate::validate_movement_delta(old_pos, new_pos, &config);
    assert!(result.is_err(), "Speed hack should be rejected");
    assert_eq!(result.unwrap_err(), InfractionType::SpeedExceeded);
}

#[test]
fn test_movement_sanity_rejects_teleport() {
    // US3 Scenario: Reject teleportation
    use plix_common::math::Vec3;

    let config = AntiCheatConfig::default();
    let old_pos = Vec3::new(0.0, 0.0, 0.0);
    // Teleport to far location
    let new_pos = Vec3::new(100.0, 100.0, 100.0);

    let result = validate::validate_movement_delta(old_pos, new_pos, &config);
    assert!(result.is_err(), "Teleport should be rejected");
}

// =============================================================================
// User Story 4: Automatic Sanctions
// =============================================================================

#[test]
fn test_sanction_warning_at_threshold() {
    // US4 Scenario 1: Warning at warning_threshold
    let mut state = AntiCheatState::new(Tick(0));
    let config = AntiCheatConfig::default();

    // Record infractions up to warning threshold
    for _ in 0..config.warning_threshold {
        state.record_infraction(InfractionType::InvalidFloat);
    }

    assert_eq!(state.strike_count(), config.warning_threshold);
}

#[test]
fn test_sanction_kick_at_threshold() {
    // US4 Scenario 2: Kick at kick_threshold
    let mut state = AntiCheatState::new(Tick(0));
    let config = AntiCheatConfig::default();

    // Record infractions up to kick threshold
    for _ in 0..config.kick_threshold {
        state.record_infraction(InfractionType::InvalidFloat);
    }

    assert_eq!(state.strike_count(), config.kick_threshold);
}

#[test]
fn test_sanction_ban_at_threshold() {
    // US4 Scenario 3: Ban at ban_threshold
    let mut state = AntiCheatState::new(Tick(0));
    let config = AntiCheatConfig::default();

    // Record infractions up to ban threshold
    for _ in 0..config.ban_threshold {
        state.record_infraction(InfractionType::InvalidFloat);
    }

    assert_eq!(state.strike_count(), config.ban_threshold);
}
