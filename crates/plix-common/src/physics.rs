//! Shared movement physics
//!
//! Used by both client (prediction) and server (authoritative).
//! All functions in this module MUST be deterministic.

use crate::math::Vec3;
use serde::{Deserialize, Serialize};

/// Movement physics configuration constants.
/// These values are shared between client and server for deterministic simulation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MovementConfig {
    /// Maximum horizontal movement speed (m/s)
    pub max_speed: f32,

    /// Gravity acceleration (m/s²)
    pub gravity: f32,

    /// Jump impulse velocity (m/s)
    pub jump_impulse: f32,

    /// Ground friction coefficient
    pub ground_friction: f32,

    /// Air control multiplier (0.0-1.0)
    pub air_control: f32,

    /// Maximum step-up height (blocks)
    pub step_height: f32,

    /// Player collision half-width (m) - radius of the capsule
    pub player_half_width: f32,

    /// Player collision height (m)
    pub player_height: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            max_speed: 6.0,
            gravity: 20.0,
            jump_impulse: 7.07, // sqrt(2 * 20 * 1.25) for 1.25 block jump height
            ground_friction: 10.0,
            air_control: 0.3,
            step_height: 1.0, // Full block step for voxel terrain
            player_half_width: 0.4,
            player_height: 1.8,
        }
    }
}

impl MovementConfig {
    /// Create a new movement config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Get player half-height (for AABB calculations)
    pub fn player_half_height(&self) -> f32 {
        self.player_height / 2.0
    }
}

/// Player movement state.
/// Tracks position, velocity, and grounded status.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MovementState {
    /// Current position (feet position)
    pub position: Vec3,

    /// Current velocity (m/s)
    pub velocity: Vec3,

    /// Whether player is on ground
    pub is_grounded: bool,

    /// Whether jump input is currently buffered
    pub jump_buffered: bool,

    /// Ticks remaining on jump buffer (0 = no buffer)
    pub jump_buffer_ticks: u8,

    /// Whether jump was pressed last tick (for edge detection)
    pub jump_was_pressed: bool,
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            is_grounded: true,
            jump_buffered: false,
            jump_buffer_ticks: 0,
            jump_was_pressed: false,
        }
    }
}

impl MovementState {
    /// Create a new movement state at the given position
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Create a movement state with position and grounded status
    pub fn with_grounded(position: Vec3, is_grounded: bool) -> Self {
        Self {
            position,
            is_grounded,
            ..Default::default()
        }
    }
}

/// Collision result after physics simulation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionResult {
    /// Final resolved position
    pub position: Vec3,

    /// Final velocity (may be zeroed on collision axes)
    pub velocity: Vec3,

    /// Whether player is on ground after resolution
    pub grounded: bool,

    /// Whether step-up occurred during this frame
    pub stepped: bool,

    /// Collision normal (if any collision occurred)
    pub hit_normal: Option<Vec3>,
}

impl Default for CollisionResult {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            grounded: false,
            stepped: false,
            hit_normal: None,
        }
    }
}

/// Epsilon value for floating-point comparisons and clamping
pub const PHYSICS_EPSILON: f32 = 0.001;

/// Minimum velocity threshold - velocities below this are snapped to zero
pub const VELOCITY_THRESHOLD: f32 = 0.01;

/// Maximum ticks for jump buffer (100ms at 60Hz = 6 ticks)
pub const JUMP_BUFFER_TICKS: u8 = 6;

/// Maximum movement per collision step (for tunneling prevention)
pub const MAX_MOVE_PER_STEP: f32 = 0.5;

/// Linear interpolation helper
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Clamp a value to the given range
#[inline]
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

/// Snap small values to zero to prevent floating-point drift
#[inline]
pub fn snap_to_zero(value: f32, threshold: f32) -> f32 {
    if value.abs() < threshold {
        0.0
    } else {
        value
    }
}

/// Snap small velocity components to zero
#[inline]
pub fn snap_velocity_to_zero(velocity: Vec3, threshold: f32) -> Vec3 {
    Vec3::new(
        snap_to_zero(velocity.x, threshold),
        snap_to_zero(velocity.y, threshold),
        snap_to_zero(velocity.z, threshold),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movement_config_defaults() {
        let config = MovementConfig::default();
        assert_eq!(config.max_speed, 6.0);
        assert_eq!(config.gravity, 20.0);
        assert!((config.jump_impulse - 7.07).abs() < 0.01);
        assert_eq!(config.ground_friction, 10.0);
        assert_eq!(config.air_control, 0.3);
        assert_eq!(config.step_height, 1.0); // Full block step for voxel terrain
        assert_eq!(config.player_half_width, 0.4);
        assert_eq!(config.player_height, 1.8);
    }

    #[test]
    fn test_movement_state_default() {
        let state = MovementState::default();
        assert_eq!(state.position, Vec3::ZERO);
        assert_eq!(state.velocity, Vec3::ZERO);
        assert!(state.is_grounded);
        assert!(!state.jump_buffered);
    }

    #[test]
    fn test_snap_to_zero() {
        assert_eq!(snap_to_zero(0.001, 0.01), 0.0);
        assert_eq!(snap_to_zero(0.1, 0.01), 0.1);
        assert_eq!(snap_to_zero(-0.005, 0.01), 0.0);
        assert_eq!(snap_to_zero(-0.1, 0.01), -0.1);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }
}
