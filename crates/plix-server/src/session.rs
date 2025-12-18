//! Player session management

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use plix_common::identity::SessionId;
use plix_common::inventory::Hotbar;
use plix_common::math::{Rotation, Vec3};
use plix_common::metrics::RollingWindow;
use plix_common::protocol::{BlockEditRequest, PlayerInput};
use plix_common::time::Tick;
use plix_common::types::{InputSeq, PlayerId, TeamId};

use crate::anti_cheat::AntiCheatState;
use crate::crafting::CraftCooldown;
use crate::economy::PlayerWallet;
use crate::weapons::PlayerWeaponState;

/// Maximum pending inputs per player
const MAX_PENDING_INPUTS: usize = 64;

/// Maximum pending block edit requests per player
const MAX_PENDING_EDITS: usize = 4;

/// Per-session network metrics for monitoring connection quality.
///
/// Tracks RTT (Round-Trip Time), jitter, and packet loss for each player connection.
#[derive(Debug)]
pub struct SessionNetMetrics {
    /// Rolling window of RTT measurements
    rtt_window: RollingWindow<Duration>,
    /// Last sequence number received (for loss detection)
    last_seq: u16,
    /// Total packets received
    packets_received: u64,
    /// Total packets detected as lost (sequence gaps)
    packets_lost: u64,
}

impl SessionNetMetrics {
    /// Create new session metrics with default settings.
    pub fn new() -> Self {
        Self {
            rtt_window: RollingWindow::with_defaults(),
            last_seq: 0,
            packets_received: 0,
            packets_lost: 0,
        }
    }

    /// Record an RTT measurement.
    #[inline]
    pub fn record_rtt(&mut self, rtt: Duration) {
        self.rtt_window.push(rtt);
    }

    /// Record a received packet and detect loss from sequence gaps.
    ///
    /// Returns the number of packets detected as lost (0 if none).
    pub fn record_packet(&mut self, seq: u16) -> u32 {
        self.packets_received += 1;

        if self.packets_received == 1 {
            // First packet - no loss detection yet
            self.last_seq = seq;
            return 0;
        }

        // Calculate expected next sequence (with wrapping)
        let expected = self.last_seq.wrapping_add(1);

        // Detect gaps (accounting for wrapping)
        let lost = if seq == expected {
            0
        } else if seq > expected {
            // Simple gap (seq jumped forward)
            (seq - expected) as u32
        } else if expected > 0xFF00 && seq < 0x0100 {
            // Wrapped around: expected was near max, seq is low
            // Example: expected=0xFFFE, seq=0x0002 means 4 packets arrived
            // Lost = (0xFFFF - expected) + seq
            ((0xFFFF_u32 - expected as u32) + seq as u32) as u32
        } else {
            // Out of order or duplicate - don't count as loss
            0
        };

        self.packets_lost += lost as u64;
        self.last_seq = seq;
        lost
    }

    /// Get the average RTT in milliseconds.
    pub fn rtt_avg_ms(&mut self) -> f64 {
        self.rtt_window.stats_ms().avg
    }

    /// Get RTT statistics (avg, p95, max, min, count) in milliseconds.
    pub fn rtt_stats_ms(&mut self) -> plix_common::metrics::Stats {
        self.rtt_window.stats_ms()
    }

    /// Calculate jitter as the standard deviation of RTT values.
    ///
    /// Returns 0.0 if not enough samples.
    pub fn jitter_ms(&mut self) -> f64 {
        let stats = self.rtt_window.stats_ms();
        if stats.count < 2 {
            return 0.0;
        }

        // Use p95 - avg as a simple jitter approximation
        // A more accurate approach would compute actual std dev,
        // but this is a reasonable proxy for network instability
        (stats.p95 - stats.avg).max(0.0)
    }

    /// Calculate packet loss percentage.
    pub fn loss_pct(&self) -> f64 {
        let total = self.packets_received + self.packets_lost;
        if total == 0 {
            return 0.0;
        }
        (self.packets_lost as f64 / total as f64) * 100.0
    }

    /// Get total packets received.
    pub fn packets_received(&self) -> u64 {
        self.packets_received
    }

    /// Get total packets lost.
    pub fn packets_lost(&self) -> u64 {
        self.packets_lost
    }
}

impl Default for SessionNetMetrics {
    fn default() -> Self {
        Self::new()
    }
}

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
    /// Session ID for this connection (logging and correlation)
    pub session_id: SessionId,

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
    /// Tick when invulnerability expires (None if vulnerable)
    pub invulnerable_until_tick: Option<Tick>,

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

    // TDM spectate
    /// Who this player is spectating while dead (killer ID)
    pub spectate_target: Option<PlayerId>,

    // Match flow
    /// Ready state for match start
    pub is_ready: bool,

    // Anti-cheat
    /// Per-player anti-cheat state
    pub anti_cheat: AntiCheatState,

    // Metrics
    /// Latest RTT nonce from client (for echo in snapshot)
    pub last_rtt_nonce: u64,
    /// Per-session network metrics (RTT, jitter, loss)
    pub net_metrics: SessionNetMetrics,

    // Inventory
    /// Player's hotbar inventory
    pub hotbar: Hotbar,

    // Weapons
    /// Per-player weapon state (cooldowns, recoil)
    pub weapon_state: PlayerWeaponState,

    // Crafting
    /// Per-player crafting cooldown tracker
    pub craft_cooldown: CraftCooldown,

    // Economy
    /// Per-player wallet for match economy
    pub wallet: PlayerWallet,

    // Identity (Feature 025)
    /// Last tick when player renamed (for rate limiting) [T047]
    pub last_rename_tick: Option<Tick>,
}

impl ServerPlayer {
    /// Create a new player with default state
    pub fn new(
        id: PlayerId,
        name: String,
        team: TeamId,
        addr: SocketAddr,
        session_id: SessionId,
    ) -> Self {
        Self {
            id,
            name,
            team,
            addr,
            session_id,
            position: Vec3::ZERO,
            rotation: Rotation::ZERO,
            velocity: Vec3::ZERO,
            health: 100,
            is_dead: false,
            respawn_tick: None,
            last_attack_tick: Tick::ZERO,
            last_edit_tick: None,
            invulnerable_until_tick: None,
            last_input_seq: InputSeq::default(),
            pending_inputs: Vec::with_capacity(MAX_PENDING_INPUTS),
            pending_edits: Vec::with_capacity(MAX_PENDING_EDITS),
            kills: 0,
            deaths: 0,
            spectate_target: None,
            is_ready: false,
            anti_cheat: AntiCheatState::default(),
            last_rtt_nonce: 0,
            net_metrics: SessionNetMetrics::new(),
            hotbar: Hotbar::with_default_size(),
            weapon_state: PlayerWeaponState::default(),
            craft_cooldown: CraftCooldown::new(),
            wallet: PlayerWallet::new(),
            last_rename_tick: None,
        }
    }

    /// Clear ready state for new match
    pub fn clear_ready(&mut self) {
        self.is_ready = false;
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
    ///
    /// Resets kills, deaths, and wallet balance for new match.
    pub fn reset_stats(&mut self) {
        self.kills = 0;
        self.deaths = 0;
        self.wallet.reset();
    }

    /// Spawn player at position with optional invulnerability
    ///
    /// If `invuln_ticks` is Some, sets invulnerability for that many ticks from `current_tick`.
    /// Use `None` for initial spawn (no invulnerability) or `Some(config.respawn_invuln_ticks)` for respawn.
    pub fn spawn(
        &mut self,
        position: Vec3,
        yaw: f32,
        current_tick: Tick,
        invuln_ticks: Option<u32>,
    ) {
        self.position = position;
        self.rotation = Rotation::new(yaw, 0.0);
        self.velocity = Vec3::ZERO;
        self.health = 100;
        self.is_dead = false;
        self.respawn_tick = None;
        self.spectate_target = None; // Clear spectate on respawn (TDM)
                                     // Set invulnerability if specified (T038 - US4)
        self.invulnerable_until_tick = invuln_ticks.map(|ticks| Tick(current_tick.0 + ticks));
        // Reset anti-cheat position tracking for new spawn location
        self.anti_cheat.update_position(position);
    }

    /// Check if player is currently invulnerable
    #[inline]
    pub fn is_invulnerable(&self, current_tick: Tick) -> bool {
        self.invulnerable_until_tick
            .map(|until| current_tick.0 < until.0)
            .unwrap_or(false)
    }

    /// Check if player can rename (rate limit: 60 seconds = 3600 ticks at 60 TPS) [T048]
    ///
    /// Returns true if player has never renamed or enough time has passed since last rename.
    pub const RENAME_COOLDOWN_TICKS: u32 = 3600; // 60 seconds at 60 TPS

    #[inline]
    pub fn can_rename(&self, current_tick: Tick) -> bool {
        self.last_rename_tick
            .map(|last| current_tick.0 >= last.0 + Self::RENAME_COOLDOWN_TICKS)
            .unwrap_or(true)
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
    pub fn add_player(
        &mut self,
        name: String,
        team: TeamId,
        addr: SocketAddr,
        session_id: SessionId,
    ) -> Option<PlayerId> {
        if self.players.len() >= self.max_players as usize {
            return None;
        }

        let id = PlayerId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        if self.next_id == 0 || self.next_id == 0xFFFF {
            self.next_id = 1;
        }

        let player = ServerPlayer::new(id, name, team, addr, session_id);
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

    /// Count ready players
    pub fn ready_count(&self) -> usize {
        self.players.values().filter(|p| p.is_ready).count()
    }

    /// Check if all players are ready
    pub fn all_ready(&self) -> bool {
        !self.players.is_empty() && self.players.values().all(|p| p.is_ready)
    }

    /// Clear ready state for all players
    pub fn clear_all_ready(&mut self) {
        for player in self.players.values_mut() {
            player.clear_ready();
        }
    }

    /// Reset all player stats (kills/deaths)
    pub fn reset_all_stats(&mut self) {
        for player in self.players.values_mut() {
            player.reset_stats();
        }
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
            .add_player("TestPlayer".into(), TeamId::TEAM_0, addr, SessionId::new(1))
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
            .add_player("P1".into(), TeamId::TEAM_0, addr1, SessionId::new(1))
            .is_some());
        assert!(manager
            .add_player("P2".into(), TeamId::TEAM_1, addr2, SessionId::new(2))
            .is_some());
        assert!(manager
            .add_player("P3".into(), TeamId::TEAM_0, addr3, SessionId::new(3))
            .is_none());

        assert!(manager.is_full());
    }

    #[test]
    fn test_player_damage() {
        let id = PlayerId(1);
        let addr = "127.0.0.1:1".parse().unwrap();
        let mut player =
            ServerPlayer::new(id, "Test".into(), TeamId::TEAM_0, addr, SessionId::new(1));

        assert!(!player.take_damage(50, Tick(100)));
        assert_eq!(player.health, 50);

        assert!(player.take_damage(50, Tick(100)));
        assert!(player.is_dead);
        assert_eq!(player.deaths, 1);
    }

    // SessionNetMetrics tests (T023-T025)

    #[test]
    fn test_session_net_metrics_rtt() {
        let mut metrics = SessionNetMetrics::new();

        metrics.record_rtt(Duration::from_millis(50));
        metrics.record_rtt(Duration::from_millis(60));
        metrics.record_rtt(Duration::from_millis(70));

        let avg = metrics.rtt_avg_ms();
        assert!((avg - 60.0).abs() < 1.0, "RTT avg should be ~60ms");
    }

    #[test]
    fn test_session_net_metrics_jitter() {
        let mut metrics = SessionNetMetrics::new();

        // Add varying RTT values
        metrics.record_rtt(Duration::from_millis(50));
        metrics.record_rtt(Duration::from_millis(55));
        metrics.record_rtt(Duration::from_millis(60));
        metrics.record_rtt(Duration::from_millis(100)); // Spike

        let jitter = metrics.jitter_ms();
        // Jitter should be non-zero due to the spike
        assert!(jitter > 0.0, "Jitter should be > 0 with varying RTT");
    }

    #[test]
    fn test_session_net_metrics_packet_loss() {
        let mut metrics = SessionNetMetrics::new();

        // Receive packets 1, 2, 5 (missing 3, 4)
        assert_eq!(metrics.record_packet(1), 0); // First packet, no loss
        assert_eq!(metrics.record_packet(2), 0); // Sequential, no loss
        assert_eq!(metrics.record_packet(5), 2); // Gap of 2 (missing 3, 4)

        assert_eq!(metrics.packets_received(), 3);
        assert_eq!(metrics.packets_lost(), 2);

        let loss = metrics.loss_pct();
        // 2 lost out of 5 total = 40%
        assert!((loss - 40.0).abs() < 1.0, "Loss should be ~40%");
    }

    #[test]
    fn test_session_net_metrics_no_loss() {
        let mut metrics = SessionNetMetrics::new();

        // Sequential packets - no loss
        for i in 1..=10 {
            metrics.record_packet(i);
        }

        assert_eq!(metrics.packets_lost(), 0);
        assert!((metrics.loss_pct() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_session_net_metrics_wrapping_sequence() {
        let mut metrics = SessionNetMetrics::new();

        // Start near max sequence
        metrics.record_packet(0xFFFC);
        metrics.record_packet(0xFFFD);
        metrics.record_packet(0xFFFE);
        metrics.record_packet(0xFFFF);
        // Wrap around
        metrics.record_packet(0x0000);
        metrics.record_packet(0x0001);

        // Should detect no loss during proper wraparound
        assert_eq!(metrics.packets_lost(), 0);
    }
}
