//! Client-side prediction for local player
//!
//! Uses shared physics from plix-common for deterministic simulation
//! that matches the server exactly.

use plix_common::math::{Rotation, Vec3};
use plix_common::physics::MovementConfig;
use plix_common::protocol::PlayerInput;

// Use shared physics constants from plix-common to ensure client-server determinism
fn config() -> MovementConfig {
    MovementConfig::default()
}

/// Linear interpolation helper
#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Predicted player state
#[derive(Debug, Clone)]
pub struct PredictedState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Rotation,
    pub is_grounded: bool,
}

impl Default for PredictedState {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            rotation: Rotation::ZERO,
            is_grounded: true,
        }
    }
}

/// Prediction system
#[derive(Debug, Default)]
pub struct PredictionSystem;

impl PredictionSystem {
    /// Create a new prediction system
    pub fn new() -> Self {
        Self
    }

    /// Predict state from input (simplified, no collision)
    /// Uses shared physics constants from plix-common for determinism
    pub fn predict(&self, state: &PredictedState, input: &PlayerInput, dt: f32) -> PredictedState {
        let cfg = config();
        let mut new_state = state.clone();

        // Update rotation
        new_state.rotation.yaw = input.yaw;
        new_state.rotation.pitch = input.pitch;

        // Calculate movement direction
        let forward = new_state.rotation.forward();
        let right = new_state.rotation.right();

        let forward_flat = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let right_flat = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

        let move_dir = forward_flat * input.move_forward + right_flat * input.move_right;

        // Apply movement using shared config
        let target_vel = move_dir * cfg.max_speed;

        if state.is_grounded {
            // Ground friction
            let friction = cfg.ground_friction * dt;
            new_state.velocity.x = lerp(new_state.velocity.x, target_vel.x, friction.min(1.0));
            new_state.velocity.z = lerp(new_state.velocity.z, target_vel.z, friction.min(1.0));

            if input.jump {
                new_state.velocity.y = cfg.jump_impulse;
                new_state.is_grounded = false;
            }
        } else {
            // Air control (reduced)
            new_state.velocity.x += (target_vel.x - new_state.velocity.x) * cfg.air_control * dt;
            new_state.velocity.z += (target_vel.z - new_state.velocity.z) * cfg.air_control * dt;
        }

        // Apply gravity
        if !new_state.is_grounded {
            new_state.velocity.y -= cfg.gravity * dt;
        }

        // Update position
        new_state.position += new_state.velocity * dt;

        // Simple ground check (y = 0)
        if new_state.position.y < 0.0 {
            new_state.position.y = 0.0;
            new_state.velocity.y = 0.0;
            new_state.is_grounded = true;
        }

        new_state
    }

    /// Replay a sequence of inputs from a base state
    pub fn replay<'a>(
        &self,
        base_state: &PredictedState,
        inputs: impl Iterator<Item = &'a PlayerInput>,
        dt: f32,
    ) -> PredictedState {
        let mut state = base_state.clone();
        for input in inputs {
            state = self.predict(&state, input, dt);
        }
        state
    }
}
