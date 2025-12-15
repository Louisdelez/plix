//! Client-side prediction for local player

use plix_common::math::{Rotation, Vec3};
use plix_common::protocol::PlayerInput;

/// Movement constants (should match server)
const MOVE_SPEED: f32 = 5.0;
const JUMP_VELOCITY: f32 = 8.0;
const GRAVITY: f32 = 20.0;

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
    pub fn predict(&self, state: &PredictedState, input: &PlayerInput, dt: f32) -> PredictedState {
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

        // Apply movement
        let target_vel = move_dir * MOVE_SPEED;

        if state.is_grounded {
            new_state.velocity.x = target_vel.x;
            new_state.velocity.z = target_vel.z;

            if input.jump {
                new_state.velocity.y = JUMP_VELOCITY;
                new_state.is_grounded = false;
            }
        }

        // Apply gravity
        if !new_state.is_grounded {
            new_state.velocity.y -= GRAVITY * dt;
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
