//! Dungeon runtime state tracking
//!
//! Tracks the current state of a dungeon instance:
//! - Boss alive/dead status
//! - Chest availability
//! - Players who have cleared this run
//! - Respawn timer

use std::collections::HashSet;
use std::time::{Duration, Instant};

use plix_common::content::ids::DungeonId;
use plix_common::types::PlayerId;

/// State of a dungeon instance.
#[derive(Debug, Clone)]
pub struct DungeonState {
    /// Dungeon definition ID
    pub dungeon_id: DungeonId,
    /// Whether the boss is currently alive
    pub boss_alive: bool,
    /// Whether the reward chest is available
    pub chest_available: bool,
    /// Players who have looted the chest this run
    pub cleared_by: HashSet<PlayerId>,
    /// Players currently inside the dungeon
    pub players_inside: HashSet<PlayerId>,
    /// When the boss was killed (for respawn timing)
    pub boss_killed_at: Option<Instant>,
    /// When the boss will respawn
    pub boss_respawn_at: Option<Instant>,
    /// Boss respawn duration (from definition)
    pub respawn_duration: Duration,
}

impl DungeonState {
    /// Create a new dungeon state with boss alive.
    pub fn new(dungeon_id: DungeonId, respawn_secs: f32) -> Self {
        Self {
            dungeon_id,
            boss_alive: true,
            chest_available: false,
            cleared_by: HashSet::new(),
            players_inside: HashSet::new(),
            boss_killed_at: None,
            boss_respawn_at: None,
            respawn_duration: Duration::from_secs_f32(respawn_secs),
        }
    }

    /// Handle the boss being killed.
    pub fn on_boss_killed(&mut self, now: Instant) {
        self.boss_alive = false;
        self.chest_available = true;
        self.boss_killed_at = Some(now);
        self.boss_respawn_at = Some(now + self.respawn_duration);
        self.cleared_by.clear(); // Reset for new run
    }

    /// Handle the boss respawning.
    pub fn on_boss_respawn(&mut self) {
        self.boss_alive = true;
        self.chest_available = false;
        self.cleared_by.clear();
        self.boss_killed_at = None;
        self.boss_respawn_at = None;
    }

    /// Check if it's time for the boss to respawn.
    pub fn should_respawn(&self, now: Instant) -> bool {
        match self.boss_respawn_at {
            Some(respawn_time) => now >= respawn_time,
            None => false,
        }
    }

    /// Get time until boss respawn (if applicable).
    pub fn time_until_respawn(&self, now: Instant) -> Option<Duration> {
        self.boss_respawn_at.map(|respawn| {
            if respawn > now {
                respawn.duration_since(now)
            } else {
                Duration::ZERO
            }
        })
    }

    /// Record a player opening the reward chest.
    /// Returns false if player already cleared this run.
    pub fn record_chest_loot(&mut self, player_id: PlayerId) -> bool {
        if !self.chest_available {
            return false;
        }
        self.cleared_by.insert(player_id)
    }

    /// Check if a player has already looted the chest this run.
    pub fn has_player_looted(&self, player_id: PlayerId) -> bool {
        self.cleared_by.contains(&player_id)
    }

    /// Record a player entering the dungeon.
    pub fn player_entered(&mut self, player_id: PlayerId) {
        self.players_inside.insert(player_id);
    }

    /// Record a player leaving the dungeon.
    pub fn player_left(&mut self, player_id: PlayerId) {
        self.players_inside.remove(&player_id);
    }

    /// Check if a player is inside the dungeon.
    pub fn is_player_inside(&self, player_id: PlayerId) -> bool {
        self.players_inside.contains(&player_id)
    }

    /// Get the number of players currently inside.
    pub fn player_count(&self) -> usize {
        self.players_inside.len()
    }

    /// Get players currently inside the dungeon.
    pub fn players_inside(&self) -> Vec<PlayerId> {
        self.players_inside.iter().copied().collect()
    }

    /// Get the current dungeon phase.
    pub fn phase(&self) -> DungeonPhase {
        if self.boss_alive {
            DungeonPhase::BossActive
        } else if self.chest_available {
            DungeonPhase::ChestAvailable
        } else {
            DungeonPhase::Cleared
        }
    }
}

/// Current phase of the dungeon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonPhase {
    /// Boss is alive and can be fought
    BossActive,
    /// Boss is dead, chest is available
    ChestAvailable,
    /// Dungeon has been fully cleared (waiting for respawn)
    Cleared,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = DungeonState::new(DungeonId::new("test_dungeon"), 300.0);
        assert!(state.boss_alive);
        assert!(!state.chest_available);
        assert!(state.cleared_by.is_empty());
        assert_eq!(state.phase(), DungeonPhase::BossActive);
    }

    #[test]
    fn test_boss_killed() {
        let mut state = DungeonState::new(DungeonId::new("test_dungeon"), 300.0);
        let now = Instant::now();

        state.on_boss_killed(now);

        assert!(!state.boss_alive);
        assert!(state.chest_available);
        assert!(state.boss_respawn_at.is_some());
        assert_eq!(state.phase(), DungeonPhase::ChestAvailable);
    }

    #[test]
    fn test_chest_loot() {
        let mut state = DungeonState::new(DungeonId::new("test_dungeon"), 300.0);
        state.on_boss_killed(Instant::now());

        // First loot should succeed
        assert!(state.record_chest_loot(PlayerId(1)));
        assert!(state.has_player_looted(PlayerId(1)));

        // Second attempt should fail
        assert!(!state.record_chest_loot(PlayerId(1)));

        // Different player should succeed
        assert!(state.record_chest_loot(PlayerId(2)));
    }

    #[test]
    fn test_respawn_timer() {
        let mut state = DungeonState::new(DungeonId::new("test_dungeon"), 0.1); // 100ms respawn
        let now = Instant::now();

        state.on_boss_killed(now);
        assert!(!state.should_respawn(now));

        // Wait for respawn
        let later = now + Duration::from_millis(150);
        assert!(state.should_respawn(later));

        state.on_boss_respawn();
        assert!(state.boss_alive);
        assert!(!state.chest_available);
        assert!(state.cleared_by.is_empty());
    }

    #[test]
    fn test_player_tracking() {
        let mut state = DungeonState::new(DungeonId::new("test_dungeon"), 300.0);

        state.player_entered(PlayerId(1));
        assert!(state.is_player_inside(PlayerId(1)));
        assert_eq!(state.player_count(), 1);

        state.player_entered(PlayerId(2));
        assert_eq!(state.player_count(), 2);

        state.player_left(PlayerId(1));
        assert!(!state.is_player_inside(PlayerId(1)));
        assert_eq!(state.player_count(), 1);
    }
}
