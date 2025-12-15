//! Input validation and anti-cheat

use plix_common::math::Vec3;
use plix_common::protocol::PlayerInput;

/// Maximum movement speed in blocks per second
pub const MAX_MOVE_SPEED: f32 = 10.0;

/// Attack cooldown in ticks (60 Hz = 30 ticks = 500ms)
#[deprecated(note = "Use CombatConfig.attack_cooldown_ticks instead")]
pub const ATTACK_COOLDOWN_TICKS: u32 = 30;

/// Attack range in blocks
#[deprecated(note = "Use CombatConfig.attack_range instead")]
pub const ATTACK_RANGE: f32 = 1.8; // Updated from 2.0 to 1.8

/// Latency tolerance for range check
pub const ATTACK_RANGE_EPSILON: f32 = 0.15;

/// Maximum position change per tick
pub const MAX_POS_DELTA: f32 = MAX_MOVE_SPEED / 60.0;

/// Validation result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationResult {
    /// Input is valid
    Valid,
    /// Input was modified (corrected)
    Modified,
    /// Input was rejected (too suspicious)
    Rejected,
}

/// Input validator
#[derive(Debug, Default)]
pub struct InputValidator {
    /// Violation count per player (for escalation)
    violations: u32,
}

impl InputValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate and potentially modify an input
    pub fn validate(&mut self, input: &mut PlayerInput) -> ValidationResult {
        let mut result = ValidationResult::Valid;

        // Clamp movement values
        if input.move_forward.abs() > 1.0 {
            input.move_forward = input.move_forward.clamp(-1.0, 1.0);
            result = ValidationResult::Modified;
        }

        if input.move_right.abs() > 1.0 {
            input.move_right = input.move_right.clamp(-1.0, 1.0);
            result = ValidationResult::Modified;
        }

        // Clamp look angles
        if input.pitch.abs() > std::f32::consts::FRAC_PI_2 {
            input.pitch = input
                .pitch
                .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
            result = ValidationResult::Modified;
        }

        if result == ValidationResult::Modified {
            self.violations += 1;
        }

        result
    }

    /// Validate position change
    pub fn validate_movement(&mut self, old_pos: Vec3, new_pos: Vec3) -> ValidationResult {
        let delta = new_pos - old_pos;
        let distance = delta.length();

        if distance > MAX_POS_DELTA * 2.0 {
            // Teleport attempt, reject
            self.violations += 10;
            return ValidationResult::Rejected;
        }

        if distance > MAX_POS_DELTA {
            // Slightly too fast, flag but allow
            self.violations += 1;
            return ValidationResult::Modified;
        }

        ValidationResult::Valid
    }

    /// Check if player should be kicked for violations
    pub fn should_kick(&self) -> bool {
        self.violations >= 100
    }

    /// Get violation count
    pub fn violations(&self) -> u32 {
        self.violations
    }

    /// Reset violations
    pub fn reset(&mut self) {
        self.violations = 0;
    }
}

/// Check if attack is valid
pub fn validate_attack(
    attacker_pos: Vec3,
    target_pos: Vec3,
    last_attack_tick: u32,
    current_tick: u32,
) -> bool {
    // Check cooldown
    if current_tick.wrapping_sub(last_attack_tick) < ATTACK_COOLDOWN_TICKS {
        return false;
    }

    // Check range
    let distance = (target_pos - attacker_pos).length();
    if distance > ATTACK_RANGE {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::time::Tick;
    use plix_common::types::InputSeq;

    #[test]
    fn test_input_clamping() {
        let mut validator = InputValidator::new();
        let mut input = PlayerInput {
            seq: InputSeq(0),
            tick: Tick(0),
            move_forward: 2.0, // Too high
            move_right: -0.5,
            jump: false,
            crouch: false,
            attack: false,
            yaw: 0.0,
            pitch: 0.0,
            rtt_nonce: 0,
        };

        let result = validator.validate(&mut input);
        assert_eq!(result, ValidationResult::Modified);
        assert_eq!(input.move_forward, 1.0);
    }

    #[test]
    fn test_movement_validation() {
        let mut validator = InputValidator::new();

        // Normal movement
        let old = Vec3::ZERO;
        let new = Vec3::new(0.1, 0.0, 0.0);
        assert_eq!(
            validator.validate_movement(old, new),
            ValidationResult::Valid
        );

        // Teleport
        let teleport = Vec3::new(10.0, 0.0, 0.0);
        assert_eq!(
            validator.validate_movement(old, teleport),
            ValidationResult::Rejected
        );
    }

    #[test]
    fn test_attack_validation() {
        let attacker = Vec3::ZERO;
        let target_close = Vec3::new(1.5, 0.0, 0.0);
        let target_far = Vec3::new(5.0, 0.0, 0.0);

        // Valid attack (close, cooldown passed)
        assert!(validate_attack(attacker, target_close, 0, 100));

        // Invalid (too far)
        assert!(!validate_attack(attacker, target_far, 0, 100));

        // Invalid (cooldown)
        assert!(!validate_attack(attacker, target_close, 90, 100));
    }
}
