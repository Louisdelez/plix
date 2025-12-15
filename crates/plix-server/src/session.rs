//! Player session management

use std::collections::HashMap;
use std::net::SocketAddr;

use plix_common::math::{Rotation, Vec3};
use plix_common::protocol::{BlockEditRequest, PlayerInput};
use plix_common::time::Tick;
use plix_common::types::{InputSeq, PlayerId, TeamId};

/// Maximum pending inputs per player
const MAX_PENDING_INPUTS: usize = 64;

/// Maximum pending block edit requests per player
const MAX_PENDING_EDITS: usize = 4;

/// Server-side player state
#[derive(Debug)]
pub struct ServerPlayer {
    /// Unique player ID
    pub id: PlayerId,
    /// Player display name
    pub name: String,
    /// Assigned team
    pub team: TeamId,
    /// Network address
    pub addr: SocketAddr,

    // Transform
    /// World position
    pub position: Vec3,
    /// Look rotation
    pub rotation: Rotation,
    /// Velocity
    pub velocity: Vec3,

    // Combat state
    /// Current health (0-100)
    pub health: u8,
    /// Is player dead
    pub is_dead: bool,
    /// Tick when player can respawn
    pub respawn_tick: Option<Tick>,
    /// Last attack tick (for cooldown)
    pub last_attack_tick: Tick,
    /// Last block edit tick (for rate limiting)
    pub last_edit_tick: Option<Tick>,

    // Input processing
    /// Last processed input sequence
    pub last_input_seq: InputSeq,
    /// Pending inputs awaiting processing
    pub pending_inputs: Vec<PlayerInput>,
    /// Pending block edit requests
    pub pending_edits: Vec<BlockEditRequest>,

    // Stats
    /// Kills this round
    pub kills: u16,
    /// Deaths this round
    pub deaths: u16,
}

impl ServerPlayer {
    /// Create a new player with default state
    pub fn new(id: PlayerId, name: String, team: TeamId, addr: SocketAddr) -> Self {
        Self {
            id,
            name,
            team,
            addr,
            position: Vec3::ZERO,
            rotation: Rotation::ZERO,
            velocity: Vec3::ZERO,
            health: 100,
            is_dead: false,
            respawn_tick: None,
            last_attack_tick: Tick::ZERO,
            last_edit_tick: None,
            last_input_seq: InputSeq::default(),
            pending_inputs: Vec::with_capacity(MAX_PENDING_INPUTS),
            pending_edits: Vec::with_capacity(MAX_PENDING_EDITS),
            kills: 0,
            deaths: 0,
        }
    }

    /// Queue an input for processing
    pub fn queue_input(&mut self, input: PlayerInput) {
        // Only accept newer inputs
        if input.seq.is_newer_than(self.last_input_seq) {
            if self.pending_inputs.len() >= MAX_PENDING_INPUTS {
                self.pending_inputs.remove(0);
            }
            self.pending_inputs.push(input);
        }
    }

    /// Get and remove the next input to process
    pub fn pop_input(&mut self) -> Option<PlayerInput> {
        if self.pending_inputs.is_empty() {
            return None;
        }
        let input = self.pending_inputs.remove(0);
        self.last_input_seq = input.seq;
        Some(input)
    }

    /// Queue a block edit request
    pub fn queue_edit(&mut self, request: BlockEditRequest) {
        if self.pending_edits.len() >= MAX_PENDING_EDITS {
            // Drop oldest if queue is full
            self.pending_edits.remove(0);
        }
        self.pending_edits.push(request);
    }

    /// Get and remove pending block edit requests
    pub fn drain_edits(&mut self) -> Vec<BlockEditRequest> {
        std::mem::take(&mut self.pending_edits)
    }

    /// Reset stats for new round
    pub fn reset_stats(&mut self) {
        self.kills = 0;
        self.deaths = 0;
    }

    /// Spawn player at position
    pub fn spawn(&mut self, position: Vec3, yaw: f32) {
        self.position = position;
        self.rotation = Rotation::new(yaw, 0.0);
        self.velocity = Vec3::ZERO;
        self.health = 100;
        self.is_dead = false;
        self.respawn_tick = None;
    }

    /// Kill the player
    pub fn die(&mut self, respawn_tick: Tick) {
        self.is_dead = true;
        self.health = 0;
        self.respawn_tick = Some(respawn_tick);
        self.deaths += 1;
    }

    /// Take damage, returns true if died
    pub fn take_damage(&mut self, amount: u8, respawn_tick: Tick) -> bool {
        if self.is_dead {
            return false;
        }

        self.health = self.health.saturating_sub(amount);
        if self.health == 0 {
            self.die(respawn_tick);
            true
        } else {
            false
        }
    }
}

/// Session manager for all connected players
#[derive(Debug, Default)]
pub struct SessionManager {
    /// Players by ID
    players: HashMap<PlayerId, ServerPlayer>,
    /// Player ID by address
    addr_to_id: HashMap<SocketAddr, PlayerId>,
    /// Next player ID to assign
    next_id: u16,
    /// Maximum allowed players
    max_players: u8,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(max_players: u8) -> Self {
        Self {
            players: HashMap::new(),
            addr_to_id: HashMap::new(),
            next_id: 1,
            max_players,
        }
    }

    /// Add a new player
    pub fn add_player(&mut self, name: String, team: TeamId, addr: SocketAddr) -> Option<PlayerId> {
        if self.players.len() >= self.max_players as usize {
            return None;
        }

        let id = PlayerId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        if self.next_id == 0 || self.next_id == 0xFFFF {
            self.next_id = 1;
        }

        let player = ServerPlayer::new(id, name, team, addr);
        self.players.insert(id, player);
        self.addr_to_id.insert(addr, id);

        Some(id)
    }

    /// Remove a player
    pub fn remove_player(&mut self, id: PlayerId) -> Option<ServerPlayer> {
        if let Some(player) = self.players.remove(&id) {
            self.addr_to_id.remove(&player.addr);
            Some(player)
        } else {
            None
        }
    }

    /// Get player by ID
    pub fn get(&self, id: PlayerId) -> Option<&ServerPlayer> {
        self.players.get(&id)
    }

    /// Get mutable player by ID
    pub fn get_mut(&mut self, id: PlayerId) -> Option<&mut ServerPlayer> {
        self.players.get_mut(&id)
    }

    /// Get player by address
    pub fn get_by_addr(&self, addr: &SocketAddr) -> Option<&ServerPlayer> {
        self.addr_to_id
            .get(addr)
            .and_then(|id| self.players.get(id))
    }

    /// Get mutable player by address
    pub fn get_by_addr_mut(&mut self, addr: &SocketAddr) -> Option<&mut ServerPlayer> {
        if let Some(&id) = self.addr_to_id.get(addr) {
            self.players.get_mut(&id)
        } else {
            None
        }
    }

    /// Iterate over all players
    pub fn iter(&self) -> impl Iterator<Item = &ServerPlayer> {
        self.players.values()
    }

    /// Iterate mutably over all players
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ServerPlayer> {
        self.players.values_mut()
    }

    /// Get player count
    pub fn count(&self) -> usize {
        self.players.len()
    }

    /// Check if server is full
    pub fn is_full(&self) -> bool {
        self.players.len() >= self.max_players as usize
    }

    /// Get all player IDs
    pub fn player_ids(&self) -> Vec<PlayerId> {
        self.players.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_remove_player() {
        let mut manager = SessionManager::new(16);

        let addr = "127.0.0.1:12345".parse().unwrap();
        let id = manager
            .add_player("TestPlayer".into(), TeamId::TEAM_0, addr)
            .unwrap();

        assert_eq!(manager.count(), 1);
        assert!(manager.get(id).is_some());
        assert!(manager.get_by_addr(&addr).is_some());

        manager.remove_player(id);
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_max_players() {
        let mut manager = SessionManager::new(2);

        let addr1 = "127.0.0.1:1".parse().unwrap();
        let addr2 = "127.0.0.1:2".parse().unwrap();
        let addr3 = "127.0.0.1:3".parse().unwrap();

        assert!(manager
            .add_player("P1".into(), TeamId::TEAM_0, addr1)
            .is_some());
        assert!(manager
            .add_player("P2".into(), TeamId::TEAM_1, addr2)
            .is_some());
        assert!(manager
            .add_player("P3".into(), TeamId::TEAM_0, addr3)
            .is_none());

        assert!(manager.is_full());
    }

    #[test]
    fn test_player_damage() {
        let id = PlayerId(1);
        let addr = "127.0.0.1:1".parse().unwrap();
        let mut player = ServerPlayer::new(id, "Test".into(), TeamId::TEAM_0, addr);

        assert!(!player.take_damage(50, Tick(100)));
        assert_eq!(player.health, 50);

        assert!(player.take_damage(50, Tick(100)));
        assert!(player.is_dead);
        assert_eq!(player.deaths, 1);
    }
}
