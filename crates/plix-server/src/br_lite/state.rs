//! BR Lite state types

use std::collections::{HashMap, HashSet};

use glam::Vec2;
use plix_common::types::PlayerId;
use serde::{Deserialize, Serialize};

use super::loot::{ActiveEffect, LootItem};

/// Zone phase mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PhaseMode {
    /// Zone radius is fixed
    #[default]
    Stable,
    /// Zone radius is interpolating toward target
    Shrinking,
}

impl PhaseMode {
    /// Convert to u8 for protocol
    pub fn to_u8(self) -> u8 {
        match self {
            PhaseMode::Stable => 0,
            PhaseMode::Shrinking => 1,
        }
    }

    /// Convert from u8
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => PhaseMode::Stable,
            _ => PhaseMode::Shrinking,
        }
    }
}

/// State of the shrinking safe zone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneState {
    /// Zone center (XZ coordinates)
    pub center: Vec2,
    /// Current zone radius
    pub current_radius: f32,
    /// Target radius for current phase
    pub target_radius: f32,
    /// Radius at start of current shrink (for interpolation)
    pub start_radius: f32,
    /// Current phase index (0-indexed)
    pub phase_index: usize,
    /// Current phase mode (Stable or Shrinking)
    pub phase_mode: PhaseMode,
    /// Ticks remaining in current phase/mode
    pub phase_timer: u32,
    /// Total ticks in current mode (for interpolation)
    pub phase_duration_ticks: u32,
    /// Damage per tick when outside zone
    pub damage_per_tick: u32,
}

impl Default for ZoneState {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            current_radius: 32.0,
            target_radius: 32.0,
            start_radius: 32.0,
            phase_index: 0,
            phase_mode: PhaseMode::Stable,
            phase_timer: 0,
            phase_duration_ticks: 0,
            damage_per_tick: 5,
        }
    }
}

impl ZoneState {
    /// Create a new zone state with initial parameters
    pub fn new(center: Vec2, initial_radius: f32) -> Self {
        Self {
            center,
            current_radius: initial_radius,
            target_radius: initial_radius,
            start_radius: initial_radius,
            phase_index: 0,
            phase_mode: PhaseMode::Stable,
            phase_timer: 0,
            phase_duration_ticks: 0,
            damage_per_tick: 5,
        }
    }

    /// Get time remaining in current phase as seconds (assuming 60Hz tick rate)
    pub fn time_remaining_secs(&self) -> u16 {
        (self.phase_timer / 60) as u16
    }
}

/// Per-player BR state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerBrState {
    /// Player is alive and playing
    Alive,
    /// Player has been eliminated
    Eliminated,
    /// Player is spectating (optional after elimination)
    Spectating {
        /// Player being spectated (None = free camera)
        target: Option<PlayerId>,
    },
}

impl Default for PlayerBrState {
    fn default() -> Self {
        PlayerBrState::Alive
    }
}

impl PlayerBrState {
    /// Check if player is alive
    pub fn is_alive(&self) -> bool {
        matches!(self, PlayerBrState::Alive)
    }

    /// Check if player is eliminated
    pub fn is_eliminated(&self) -> bool {
        matches!(
            self,
            PlayerBrState::Eliminated | PlayerBrState::Spectating { .. }
        )
    }
}

/// Complete BR Lite match state
#[derive(Debug, Clone)]
pub struct BrLiteState {
    /// Current zone state
    pub zone: ZoneState,
    /// Set of alive player IDs
    pub alive_players: HashSet<PlayerId>,
    /// Set of eliminated player IDs
    pub eliminated_players: HashSet<PlayerId>,
    /// Per-player BR state
    pub player_states: HashMap<PlayerId, PlayerBrState>,
    /// Winner (if match ended)
    pub winner: Option<PlayerId>,
    /// Loot items in the arena
    pub loot_items: Vec<LootItem>,
    /// Active effects per player
    pub active_effects: HashMap<PlayerId, Vec<ActiveEffect>>,
    /// Total eliminations this match
    pub total_eliminations: u32,
    /// Match has started (in Playing phase)
    pub match_started: bool,
    /// Match has ended
    pub match_ended: bool,
}

impl Default for BrLiteState {
    fn default() -> Self {
        Self::new()
    }
}

impl BrLiteState {
    /// Create a new empty BR Lite state
    pub fn new() -> Self {
        Self {
            zone: ZoneState::default(),
            alive_players: HashSet::new(),
            eliminated_players: HashSet::new(),
            player_states: HashMap::new(),
            winner: None,
            loot_items: Vec::new(),
            active_effects: HashMap::new(),
            total_eliminations: 0,
            match_started: false,
            match_ended: false,
        }
    }

    /// Initialize zone with center and radius
    pub fn init_zone(&mut self, center: Vec2, initial_radius: f32) {
        self.zone = ZoneState::new(center, initial_radius);
    }

    /// Add a player as alive
    pub fn add_player(&mut self, player_id: PlayerId) {
        self.alive_players.insert(player_id);
        self.player_states.insert(player_id, PlayerBrState::Alive);
        self.active_effects.insert(player_id, Vec::new());
    }

    /// Remove a player from the match (disconnect before start)
    pub fn remove_player(&mut self, player_id: PlayerId) {
        self.alive_players.remove(&player_id);
        self.eliminated_players.remove(&player_id);
        self.player_states.remove(&player_id);
        self.active_effects.remove(&player_id);
    }

    /// Get count of alive players
    pub fn alive_count(&self) -> usize {
        self.alive_players.len()
    }

    /// Eliminate a player (permanent death)
    /// Returns Some(winner_id) if this elimination results in a winner
    pub fn eliminate(&mut self, player_id: PlayerId) -> Option<PlayerId> {
        if !self.alive_players.remove(&player_id) {
            return None; // Player was not alive
        }

        self.eliminated_players.insert(player_id);
        self.player_states
            .insert(player_id, PlayerBrState::Eliminated);
        self.total_eliminations += 1;

        // Clear active effects for eliminated player
        self.active_effects.remove(&player_id);

        // Check victory condition
        self.check_victory()
    }

    /// Check victory condition and set winner if applicable
    /// Returns Some(winner_id) if there's a winner
    pub fn check_victory(&mut self) -> Option<PlayerId> {
        if self.match_ended {
            return self.winner;
        }

        match self.alive_players.len() {
            1 => {
                // Last player standing wins
                let winner = *self.alive_players.iter().next().unwrap();
                self.winner = Some(winner);
                self.match_ended = true;
                Some(winner)
            }
            0 => {
                // Edge case: all players eliminated simultaneously
                // Winner is the last eliminated player (lowest ID as tiebreaker)
                if let Some(&winner) = self.eliminated_players.iter().min_by_key(|id| id.0) {
                    self.winner = Some(winner);
                    self.match_ended = true;
                    Some(winner)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Reset state for a new match
    pub fn reset(&mut self) {
        self.zone = ZoneState::default();
        self.alive_players.clear();
        self.eliminated_players.clear();
        self.player_states.clear();
        self.winner = None;
        self.loot_items.clear();
        self.active_effects.clear();
        self.total_eliminations = 0;
        self.match_started = false;
        self.match_ended = false;
    }

    /// Get player's BR state
    pub fn get_player_state(&self, player_id: PlayerId) -> Option<&PlayerBrState> {
        self.player_states.get(&player_id)
    }

    /// Check if a player is alive
    pub fn is_alive(&self, player_id: PlayerId) -> bool {
        self.alive_players.contains(&player_id)
    }

    /// Enter spectator mode for an eliminated player
    pub fn enter_spectate(&mut self, player_id: PlayerId, target: Option<PlayerId>) {
        if self.eliminated_players.contains(&player_id) {
            self.player_states
                .insert(player_id, PlayerBrState::Spectating { target });
        }
    }
}

/// Debug info for server queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrLiteDebugInfo {
    pub alive_count: u8,
    pub phase_index: u8,
    pub zone_radius: f32,
    pub zone_mode: String,
    pub phase_time_remaining: u32,
    pub loot_remaining: u8,
    pub total_eliminations: u8,
}

impl BrLiteState {
    /// Get debug info for server queries
    pub fn debug_info(&self) -> BrLiteDebugInfo {
        BrLiteDebugInfo {
            alive_count: self.alive_count() as u8,
            phase_index: self.zone.phase_index as u8,
            zone_radius: self.zone.current_radius,
            zone_mode: format!("{:?}", self.zone.phase_mode),
            phase_time_remaining: self.zone.phase_timer,
            loot_remaining: self.loot_items.iter().filter(|l| !l.collected).count() as u8,
            total_eliminations: self.total_eliminations as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_state_creation() {
        let zone = ZoneState::new(Vec2::new(32.0, 32.0), 30.0);
        assert_eq!(zone.center, Vec2::new(32.0, 32.0));
        assert_eq!(zone.current_radius, 30.0);
        assert_eq!(zone.phase_mode, PhaseMode::Stable);
    }

    #[test]
    fn test_player_br_state() {
        assert!(PlayerBrState::Alive.is_alive());
        assert!(!PlayerBrState::Alive.is_eliminated());
        assert!(PlayerBrState::Eliminated.is_eliminated());
        assert!(PlayerBrState::Spectating { target: None }.is_eliminated());
    }

    #[test]
    fn test_br_lite_state_add_player() {
        let mut state = BrLiteState::new();
        state.add_player(PlayerId(1));
        state.add_player(PlayerId(2));

        assert_eq!(state.alive_count(), 2);
        assert!(state.is_alive(PlayerId(1)));
        assert!(state.is_alive(PlayerId(2)));
    }

    #[test]
    fn test_elimination_and_victory() {
        let mut state = BrLiteState::new();
        state.match_started = true;
        state.add_player(PlayerId(1));
        state.add_player(PlayerId(2));
        state.add_player(PlayerId(3));

        // Eliminate first player
        let result = state.eliminate(PlayerId(3));
        assert!(result.is_none()); // No winner yet
        assert_eq!(state.alive_count(), 2);

        // Eliminate second player
        let result = state.eliminate(PlayerId(2));
        assert!(result.is_some()); // Winner!
        assert_eq!(result.unwrap(), PlayerId(1));
        assert!(state.match_ended);
    }

    #[test]
    fn test_simultaneous_death_edge_case() {
        let mut state = BrLiteState::new();
        state.match_started = true;
        state.add_player(PlayerId(5));
        state.add_player(PlayerId(3));

        // Both die - lowest ID wins
        state.eliminate(PlayerId(5));
        let result = state.eliminate(PlayerId(3));

        assert!(result.is_some());
        assert_eq!(result.unwrap(), PlayerId(3)); // Lowest ID wins
    }

    #[test]
    fn test_reset() {
        let mut state = BrLiteState::new();
        state.add_player(PlayerId(1));
        state.add_player(PlayerId(2));
        state.match_started = true;
        state.eliminate(PlayerId(2));

        state.reset();

        assert_eq!(state.alive_count(), 0);
        assert!(state.winner.is_none());
        assert!(!state.match_started);
        assert!(!state.match_ended);
        assert_eq!(state.total_eliminations, 0);
    }

    #[test]
    fn test_phase_mode_serialization() {
        assert_eq!(PhaseMode::Stable.to_u8(), 0);
        assert_eq!(PhaseMode::Shrinking.to_u8(), 1);
        assert_eq!(PhaseMode::from_u8(0), PhaseMode::Stable);
        assert_eq!(PhaseMode::from_u8(1), PhaseMode::Shrinking);
    }
}
