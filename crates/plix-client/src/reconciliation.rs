//! Server reconciliation for client-side prediction

use plix_common::math::Vec3;
use plix_common::protocol::{PlayerSnapshot, WorldSnapshot};
use plix_common::types::PlayerId;

use crate::commands::CommandBuffer;
use crate::prediction::{PredictedState, PredictionSystem};

/// Threshold for smooth correction vs snap
const SMOOTH_THRESHOLD: f32 = 1.0; // blocks
const SNAP_THRESHOLD: f32 = 5.0; // blocks

/// Maximum correction delta per frame (0.5 blocks) - T059
/// Prevents jarring visual corrections while still resolving errors
const MAX_CORRECTION_PER_FRAME: f32 = 0.5;

/// Reconciliation system
#[derive(Debug)]
pub struct ReconciliationSystem {
    /// Prediction system
    prediction: PredictionSystem,
    /// Correction amount to apply (smoothing)
    correction: Vec3,
    /// Correction blend factor
    correction_blend: f32,
}

impl ReconciliationSystem {
    /// Create a new reconciliation system
    pub fn new() -> Self {
        Self {
            prediction: PredictionSystem::new(),
            correction: Vec3::ZERO,
            correction_blend: 0.0,
        }
    }

    /// Process server snapshot and reconcile with prediction
    pub fn process_snapshot(
        &mut self,
        snapshot: &WorldSnapshot,
        local_player_id: PlayerId,
        commands: &mut CommandBuffer,
        predicted_state: &mut PredictedState,
        dt: f32,
    ) {
        // Find local player in snapshot
        let server_state = snapshot.players.iter().find(|p| p.id == local_player_id);

        let Some(server_player) = server_state else {
            return;
        };

        // Acknowledge processed inputs
        commands.acknowledge(snapshot.last_input_seq);

        // Create base state from server
        let server_predicted = PredictedState {
            position: server_player.position,
            velocity: Vec3::ZERO, // Server doesn't send velocity
            rotation: server_player.rotation,
            is_grounded: true, // Assume grounded, will be corrected
        };

        // Replay pending inputs
        let replayed = self
            .prediction
            .replay(&server_predicted, commands.pending_commands(), dt);

        // Calculate prediction error
        let error = replayed.position - predicted_state.position;
        let error_dist = error.length();

        if error_dist > SNAP_THRESHOLD {
            // Large error - snap to server state
            *predicted_state = replayed;
            self.correction = Vec3::ZERO;
            self.correction_blend = 0.0;
        } else if error_dist > SMOOTH_THRESHOLD {
            // Medium error - smooth correction
            self.correction = error;
            self.correction_blend = 1.0;
            predicted_state.rotation = replayed.rotation;
        } else {
            // Small error - ignore (within acceptable prediction error)
            predicted_state.rotation = replayed.rotation;
        }
    }

    /// Apply smoothed correction with delta clamping (T058 + T059)
    /// Blends over ~100ms with max 0.5 blocks/frame to prevent jarring corrections
    pub fn apply_correction(&mut self, state: &mut PredictedState, dt: f32) {
        if self.correction_blend > 0.0 {
            let blend_rate = 10.0 * dt; // Smooth over ~100ms
            let apply_amount = self.correction * blend_rate.min(self.correction_blend);

            // T059: Clamp correction delta to max 0.5 blocks per frame
            let apply_length = apply_amount.length();
            let clamped_apply = if apply_length > MAX_CORRECTION_PER_FRAME {
                apply_amount * (MAX_CORRECTION_PER_FRAME / apply_length)
            } else {
                apply_amount
            };

            state.position += clamped_apply;
            self.correction -= clamped_apply;
            self.correction_blend -= blend_rate;

            if self.correction_blend <= 0.0 || self.correction.length() < 0.001 {
                self.correction = Vec3::ZERO;
                self.correction_blend = 0.0;
            }
        }
    }

    /// Check if currently correcting
    pub fn is_correcting(&self) -> bool {
        self.correction_blend > 0.0
    }
}

impl Default for ReconciliationSystem {
    fn default() -> Self {
        Self::new()
    }
}
