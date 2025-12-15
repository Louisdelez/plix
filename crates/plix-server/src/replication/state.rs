//! Replicated state definition

use plix_common::math::{Rotation, Vec3};
use plix_common::protocol::AnimationState;
use plix_common::types::PlayerId;

/// Player state for replication
#[derive(Debug, Clone)]
pub struct ReplicatedPlayer {
    pub id: PlayerId,
    pub position: Vec3,
    pub rotation: Rotation,
    pub velocity: Vec3,
    pub health: u8,
    pub is_dead: bool,
    pub animation: AnimationState,
}

/// All replicated state
#[derive(Debug, Clone, Default)]
pub struct ReplicatedState {
    pub players: Vec<ReplicatedPlayer>,
}

impl ReplicatedState {
    /// Create empty state
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.players.clear();
    }

    /// Add or update a player
    pub fn update_player(&mut self, player: ReplicatedPlayer) {
        if let Some(existing) = self.players.iter_mut().find(|p| p.id == player.id) {
            *existing = player;
        } else {
            self.players.push(player);
        }
    }

    /// Remove a player
    pub fn remove_player(&mut self, id: PlayerId) {
        self.players.retain(|p| p.id != id);
    }

    /// Get player by ID
    pub fn get_player(&self, id: PlayerId) -> Option<&ReplicatedPlayer> {
        self.players.iter().find(|p| p.id == id)
    }
}
