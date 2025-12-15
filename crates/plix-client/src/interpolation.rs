//! Entity interpolation for remote players

use std::collections::VecDeque;

use plix_common::math::{Rotation, Vec3};
use plix_common::protocol::{AnimationState, PlayerSnapshot};
use plix_common::time::Tick;
use plix_common::types::PlayerId;

/// Interpolation delay in ticks
const INTERPOLATION_DELAY: u32 = 6; // 100ms at 60 Hz

/// Maximum snapshots to buffer
const MAX_SNAPSHOTS: usize = 32;

/// Maximum extrapolation distance (blocks) - T060
/// Clamps render positions to prevent wild extrapolation during packet loss
const MAX_EXTRAPOLATION_DIST: f32 = 2.0;

/// Snapshot with timing info
#[derive(Debug, Clone)]
struct TimedSnapshot {
    tick: Tick,
    snapshot: PlayerSnapshot,
}

/// Remote player state with interpolation
#[derive(Debug)]
pub struct RemotePlayer {
    /// Player ID
    pub id: PlayerId,
    /// Snapshot buffer
    snapshots: VecDeque<TimedSnapshot>,
    /// Current display position
    pub display_position: Vec3,
    /// Current display rotation
    pub display_rotation: Rotation,
    /// Current animation state
    pub display_animation: AnimationState,
    /// Is player dead (skip rendering)
    pub is_dead: bool,
    /// Player health
    pub health: u8,
}

impl RemotePlayer {
    /// Create a new remote player
    pub fn new(id: PlayerId) -> Self {
        Self {
            id,
            snapshots: VecDeque::with_capacity(MAX_SNAPSHOTS),
            display_position: Vec3::ZERO,
            display_rotation: Rotation::ZERO,
            display_animation: AnimationState::Idle,
            is_dead: false,
            health: 100,
        }
    }

    /// Add a new snapshot
    pub fn add_snapshot(&mut self, tick: Tick, snapshot: PlayerSnapshot) {
        // Only add if newer than last
        if let Some(last) = self.snapshots.back() {
            if !tick.is_newer_than(last.tick) {
                return;
            }
        }

        // Update dead/health state from snapshot
        self.is_dead = snapshot.is_dead;
        self.health = snapshot.health;

        if self.snapshots.len() >= MAX_SNAPSHOTS {
            self.snapshots.pop_front();
        }

        self.snapshots.push_back(TimedSnapshot { tick, snapshot });
    }

    /// Update interpolated state for render tick
    pub fn interpolate(&mut self, render_tick: Tick, tick_rate: u8) {
        // Target tick is render_tick - delay
        let target_tick = Tick(render_tick.0.saturating_sub(INTERPOLATION_DELAY));

        // Find two snapshots to interpolate between
        let mut before: Option<&TimedSnapshot> = None;
        let mut after: Option<&TimedSnapshot> = None;

        for snapshot in &self.snapshots {
            if snapshot.tick.0 <= target_tick.0 {
                before = Some(snapshot);
            } else if after.is_none() {
                after = Some(snapshot);
            }
        }

        match (before, after) {
            (Some(b), Some(a)) => {
                // Interpolate between snapshots
                let total_ticks = a.tick.0.saturating_sub(b.tick.0);
                let elapsed_ticks = target_tick.0.saturating_sub(b.tick.0);

                let t = if total_ticks > 0 {
                    (elapsed_ticks as f32 / total_ticks as f32).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                let interpolated = b.snapshot.position.lerp(a.snapshot.position, t);

                // T060: Clamp interpolated position to valid space
                // Ensure Y doesn't go below ground level (0)
                self.display_position =
                    Vec3::new(interpolated.x, interpolated.y.max(0.0), interpolated.z);

                self.display_rotation =
                    lerp_rotation(&b.snapshot.rotation, &a.snapshot.rotation, t);
                self.display_animation = a.snapshot.animation;
            }
            (Some(b), None) => {
                // Extrapolate from last known - T060: clamp to valid space
                // No extrapolation beyond last known position to prevent wild rendering
                self.display_position = b.snapshot.position;
                self.display_rotation = b.snapshot.rotation;
                self.display_animation = b.snapshot.animation;
            }
            (None, Some(a)) => {
                // Use first known
                self.display_position = a.snapshot.position;
                self.display_rotation = a.snapshot.rotation;
                self.display_animation = a.snapshot.animation;
            }
            (None, None) => {
                // No snapshots, keep current
            }
        }

        // Prune old snapshots
        while self.snapshots.len() > 2 {
            if let Some(front) = self.snapshots.front() {
                if front.tick.0 < target_tick.0.saturating_sub(60) {
                    self.snapshots.pop_front();
                } else {
                    break;
                }
            }
        }
    }
}

/// Interpolation manager for all remote players
#[derive(Debug, Default)]
pub struct InterpolationManager {
    players: Vec<RemotePlayer>,
}

impl InterpolationManager {
    /// Create a new interpolation manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Update from world snapshot
    pub fn process_snapshot(
        &mut self,
        tick: Tick,
        snapshots: &[PlayerSnapshot],
        local_id: PlayerId,
    ) {
        for snapshot in snapshots {
            if snapshot.id == local_id {
                continue; // Skip local player
            }

            let player = self.get_or_create(snapshot.id);
            player.add_snapshot(tick, snapshot.clone());
        }
    }

    /// Update all player interpolation
    pub fn update(&mut self, render_tick: Tick, tick_rate: u8) {
        for player in &mut self.players {
            player.interpolate(render_tick, tick_rate);
        }
    }

    /// Get or create remote player
    fn get_or_create(&mut self, id: PlayerId) -> &mut RemotePlayer {
        if let Some(idx) = self.players.iter().position(|p| p.id == id) {
            &mut self.players[idx]
        } else {
            self.players.push(RemotePlayer::new(id));
            self.players.last_mut().unwrap()
        }
    }

    /// Remove a player
    pub fn remove_player(&mut self, id: PlayerId) {
        self.players.retain(|p| p.id != id);
    }

    /// Get all players
    pub fn players(&self) -> &[RemotePlayer] {
        &self.players
    }

    /// Clear all players
    pub fn clear(&mut self) {
        self.players.clear();
    }
}

/// Linearly interpolate rotations
fn lerp_rotation(a: &Rotation, b: &Rotation, t: f32) -> Rotation {
    // Simple lerp for angles (doesn't handle wrap-around well but ok for small differences)
    Rotation {
        yaw: a.yaw + (b.yaw - a.yaw) * t,
        pitch: a.pitch + (b.pitch - a.pitch) * t,
    }
}
