//! Input validation functions for anti-cheat.
//!
//! These functions validate client inputs for:
//! - NaN/INF values (invalid floats)
//! - Out-of-bounds values
//! - Movement sanity (speed/teleport detection)

use plix_common::math::Vec3;
use plix_common::protocol::PlayerInput;

use super::config::AntiCheatConfig;
use super::InfractionType;

/// Check if all float fields in an input are finite (not NaN or INF).
#[inline]
pub fn floats_are_finite(input: &PlayerInput) -> bool {
    input.move_forward.is_finite()
        && input.move_right.is_finite()
        && input.yaw.is_finite()
        && input.pitch.is_finite()
}

/// Validate a PlayerInput for NaN/INF and bounds.
/// Returns Ok(()) if valid, Err(InfractionType) if invalid.
pub fn validate_input(input: &PlayerInput) -> Result<(), InfractionType> {
    // Check all f32 fields for NaN/INF
    if !floats_are_finite(input) {
        return Err(InfractionType::InvalidFloat);
    }

    // Check movement bounds [-1.0, 1.0]
    if input.move_forward < -1.0 || input.move_forward > 1.0 {
        return Err(InfractionType::OutOfBounds);
    }
    if input.move_right < -1.0 || input.move_right > 1.0 {
        return Err(InfractionType::OutOfBounds);
    }

    // Check pitch bounds [-PI/2, PI/2]
    let half_pi = std::f32::consts::FRAC_PI_2;
    if input.pitch < -half_pi || input.pitch > half_pi {
        return Err(InfractionType::OutOfBounds);
    }

    Ok(())
}

/// Validate movement delta between positions.
/// Returns Ok(()) if valid, Err(InfractionType) if speed/teleport detected.
pub fn validate_movement_delta(
    old_pos: Vec3,
    new_pos: Vec3,
    config: &AntiCheatConfig,
) -> Result<(), InfractionType> {
    let delta = new_pos - old_pos;
    let distance = delta.length();

    // Allow a tolerance of 2x max_speed_per_tick for network jitter
    let max_allowed = config.max_speed_per_tick * 2.0;

    if distance > max_allowed {
        return Err(InfractionType::SpeedExceeded);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::time::Tick;
    use plix_common::types::InputSeq;

    fn make_input(move_forward: f32, move_right: f32, yaw: f32, pitch: f32) -> PlayerInput {
        PlayerInput {
            seq: InputSeq(1),
            tick: Tick(0),
            move_forward,
            move_right,
            jump: false,
            crouch: false,
            attack: false,
            yaw,
            pitch,
            rtt_nonce: 0,
        }
    }

    #[test]
    fn test_floats_are_finite() {
        let valid = make_input(0.5, -0.5, 1.0, 0.5);
        assert!(floats_are_finite(&valid));

        let nan = make_input(f32::NAN, 0.0, 0.0, 0.0);
        assert!(!floats_are_finite(&nan));

        let inf = make_input(0.0, f32::INFINITY, 0.0, 0.0);
        assert!(!floats_are_finite(&inf));
    }

    #[test]
    fn test_validate_input_valid() {
        let input = make_input(0.5, -0.5, 1.0, 0.5);
        assert!(validate_input(&input).is_ok());
    }

    #[test]
    fn test_validate_input_nan() {
        let input = make_input(f32::NAN, 0.0, 0.0, 0.0);
        assert_eq!(validate_input(&input), Err(InfractionType::InvalidFloat));
    }

    #[test]
    fn test_validate_input_out_of_bounds() {
        let input = make_input(2.0, 0.0, 0.0, 0.0);
        assert_eq!(validate_input(&input), Err(InfractionType::OutOfBounds));
    }

    #[test]
    fn test_validate_movement_normal() {
        let config = AntiCheatConfig::default();
        let old = Vec3::ZERO;
        let new = Vec3::new(0.1, 0.0, 0.0);
        assert!(validate_movement_delta(old, new, &config).is_ok());
    }

    #[test]
    fn test_validate_movement_speed_exceeded() {
        let config = AntiCheatConfig::default();
        let old = Vec3::ZERO;
        let new = Vec3::new(10.0, 0.0, 0.0);
        assert_eq!(
            validate_movement_delta(old, new, &config),
            Err(InfractionType::SpeedExceeded)
        );
    }
}
