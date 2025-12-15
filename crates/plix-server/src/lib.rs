//! Plix Server - Authoritative game server
//!
//! This crate implements the authoritative game server:
//! - Fixed tick loop (60 Hz default)
//! - Player session management
//! - Server-side simulation (movement, collision, combat)
//! - Snapshot replication with delta encoding
//! - Input validation and anti-cheat
//! - Match state management

pub mod anti_cheat;
pub mod match_state;
pub mod metrics;
pub mod netloop;
pub mod replication;
pub mod session;
pub mod sim;
pub mod tick;
pub mod validation;

use std::net::SocketAddr;
use std::path::PathBuf;

use thiserror::Error;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tracing::{debug, info, warn};

use plix_arena::{load_arena, SpawnManager};
use plix_common::combat::CombatConfig;
use plix_common::math::Vec3;
use plix_common::protocol::{
    BlockEditApplied, BlockEditKind, BlockEditRejected, BlockEditRequest, ClientMessage, GameEvent,
    MatchPhase, PlayerSnapshot, ServerMessage, WorldSnapshot,
};
use plix_common::time::Tick;
use plix_common::types::BlockType;
use plix_common::types::{InputSeq, PlayerId, TeamId};
use plix_net::UdpTransport;

use crate::anti_cheat::{
    sanctions, validate, ActionType, AntiCheatConfig, BanList, InfractionType, SanctionType,
};
use crate::match_state::{MatchConfig, MatchStateMachine};
use crate::metrics::ServerMetricsCollector;
use crate::session::SessionManager;
use crate::sim::block_edit::BlockEditSystem;
use crate::sim::combat::CombatSystem;
use crate::sim::movement::MovementSystem;
use crate::tick::TickConfig;

/// Server errors
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to load arena: {0}")]
    ArenaLoad(#[from] plix_arena::loader::LoadError),

    #[error("Protocol error: {0}")]
    Protocol(#[from] plix_common::protocol::ProtocolError),
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// UDP port to listen on
    pub port: u16,
    /// Server tick rate (Hz)
    pub tick_rate: u8,
    /// Maximum concurrent players
    pub max_players: u8,
    /// Arena name
    pub arena_name: String,
    /// Assets directory path
    pub assets_dir: PathBuf,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 7777,
            tick_rate: 60,
            max_players: 16,
            arena_name: "test_arena".to_string(),
            assets_dir: PathBuf::from("assets"),
        }
    }
}

/// Outgoing message to be sent
struct OutgoingMessage {
    addr: SocketAddr,
    data: Vec<u8>,
}

/// Main server instance
pub struct Server {
    config: ServerConfig,
    transport: UdpTransport,
    sessions: SessionManager,
    match_state: MatchStateMachine,
    match_config: MatchConfig,
    spawn_manager: SpawnManager,
    movement: MovementSystem,
    combat: CombatSystem,
    combat_config: CombatConfig,
    current_tick: Tick,
    tick_config: TickConfig,
    arena_data: Vec<u8>,
    /// Anti-cheat configuration
    anti_cheat_config: AntiCheatConfig,
    /// IP ban list
    ban_list: BanList,
    /// Server metrics collector
    metrics: ServerMetricsCollector,
}

impl Server {
    /// Create a new server with the given configuration
    pub async fn new(config: ServerConfig) -> Result<Self, ServerError> {
        // Bind UDP socket
        let addr: SocketAddr = format!("0.0.0.0:{}", config.port).parse().unwrap();
        let transport = UdpTransport::bind(addr).await?;

        info!(
            port = config.port,
            addr = %transport.local_addr()?,
            "UDP socket bound"
        );

        // Load arena
        let arena_path = config
            .assets_dir
            .join("arenas")
            .join(format!("{}.toml", config.arena_name));
        let loaded_arena = load_arena(&arena_path)?;

        info!(
            arena = %config.arena_name,
            size = ?loaded_arena.size(),
            spawns = loaded_arena.spawn_point_count(),
            "Arena loaded"
        );

        // Serialize arena metadata only (not full block data - too large for UDP)
        // Client loads arena locally; block edits replicate as deltas
        let arena_data = plix_common::protocol::encode(&loaded_arena.definition)?;

        // Create spawn manager
        let spawn_manager = SpawnManager::new(loaded_arena.definition.spawn_points.clone());

        // Create movement system with arena collision
        let movement = MovementSystem::new(loaded_arena);

        // Create session manager
        let sessions = SessionManager::new(config.max_players);

        // Create match state machine
        let match_config = MatchConfig::default();
        let match_state = MatchStateMachine::new(match_config.clone(), config.arena_name.clone());

        // Create combat system and config
        let combat = CombatSystem::new();
        let combat_config = CombatConfig::default();

        // Create tick config
        let tick_config = TickConfig::new(config.tick_rate);

        // Create anti-cheat config with defaults
        let anti_cheat_config = AntiCheatConfig::default();

        // Create ban list
        let ban_list = BanList::new();

        // Create metrics collector
        let metrics = ServerMetricsCollector::new();

        Ok(Self {
            config,
            transport,
            sessions,
            match_state,
            match_config,
            spawn_manager,
            movement,
            combat,
            combat_config,
            current_tick: Tick::ZERO,
            tick_config,
            arena_data,
            anti_cheat_config,
            ban_list,
            metrics,
        })
    }

    /// Run the server main loop
    pub async fn run(mut self) -> Result<(), ServerError> {
        info!(
            tick_rate = self.tick_config.rate,
            max_players = self.config.max_players,
            "Server starting main loop"
        );

        let tick_duration = Duration::from_secs_f64(1.0 / self.tick_config.rate as f64);
        let mut tick_interval = interval(tick_duration);
        tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                // Handle tick
                _ = tick_interval.tick() => {
                    let tick_start = std::time::Instant::now();
                    self.tick().await?;
                    let tick_duration = tick_start.elapsed();

                    // Record tick metrics and maybe log
                    self.metrics.record_tick(tick_duration);
                    if self.metrics.maybe_log(self.sessions.count(), self.sessions.count()) {
                        // Log per-session network metrics
                        let session_metrics: Vec<_> = self.sessions.iter_mut()
                            .map(|p| {
                                let jitter = p.net_metrics.jitter_ms();
                                let loss = p.net_metrics.loss_pct();
                                // RTT is computed client-side; use 0 as placeholder for server-side
                                (p.id.0, p.name.as_str(), 0.0_f64, jitter, loss)
                            })
                            .collect();
                        crate::metrics::ServerMetricsCollector::log_session_metrics(session_metrics.into_iter());
                    }
                }

                // Handle incoming packets (non-blocking poll)
                result = self.transport.recv_from() => {
                    match result {
                        Ok((data, from)) => {
                            // Record packet for metrics
                            self.metrics.record_packet_in(data.len());
                            // Copy data to avoid borrow issues
                            let data = data.to_vec();
                            self.handle_packet(&data, from).await;
                        }
                        Err(e) => {
                            warn!(error = %e, "Failed to receive packet");
                        }
                    }
                }
            }
        }
    }

    /// Process a single server tick
    async fn tick(&mut self) -> Result<(), ServerError> {
        self.current_tick = self.current_tick.next();

        // Track old phase for change detection
        let old_phase = self.match_state.phase();

        // Update match state
        if let Some(new_phase) = self.match_state.update(self.current_tick) {
            info!(phase = ?new_phase, tick = %self.current_tick.0, "Match phase changed");

            // Broadcast MatchPhaseChanged event
            self.broadcast_event(GameEvent::MatchPhaseChanged {
                from: old_phase,
                to: new_phase,
            })
            .await;

            // Handle phase-specific transitions
            match new_phase {
                MatchPhase::Playing => {
                    // Spawn all players and reset stats when entering Playing
                    self.spawn_all_players().await;
                    self.sessions.reset_all_stats();
                    info!("All players spawned for new match");
                }
                MatchPhase::EndScreen => {
                    // Broadcast MatchEnd event (T045 - time limit case)
                    if let Some(result) = self.match_state.match_end_result() {
                        info!(
                            winner = ?result.winner,
                            reason = ?result.reason,
                            "Match ended"
                        );
                        self.broadcast_event(GameEvent::MatchEnd {
                            winner: result.winner,
                            scores: result.scores.clone(),
                            reason: result.reason,
                        })
                        .await;
                    }
                }
                MatchPhase::Resetting => {
                    // World reset handled here
                    self.reset_world().await;
                }
                MatchPhase::Lobby => {
                    // Clear ready states when returning to Lobby
                    self.sessions.clear_all_ready();
                }
                _ => {}
            }
        }

        // Broadcast countdown tick every second during countdown
        if self.match_state.phase() == MatchPhase::Countdown
            && self.match_state.should_broadcast_countdown()
        {
            let remaining = self.match_state.countdown_seconds();
            self.broadcast_event(GameEvent::CountdownTick { remaining })
                .await;
            info!(remaining = remaining, "Countdown tick");
        }

        // Handle countdown cancellation if player disconnects/unreadies
        if self.match_state.phase() == MatchPhase::Countdown {
            let player_count = self.sessions.count();
            let min_players = self.match_config.min_players as usize;
            if player_count < min_players || !self.sessions.all_ready() {
                self.match_state.cancel_countdown(self.current_tick);
                info!("Countdown cancelled - not all players ready");
                self.broadcast_event(GameEvent::MatchPhaseChanged {
                    from: MatchPhase::Countdown,
                    to: MatchPhase::Lobby,
                })
                .await;
            }
        }

        // Process player inputs and simulate
        if self.match_state.phase() == MatchPhase::Playing {
            self.simulate_tick().await;
            // Process block edits
            self.process_block_edits().await;
        }

        // Evaluate sanctions for all players (US4)
        self.evaluate_all_sanctions().await;

        // Periodically clean up expired bans (every ~60 seconds)
        if self.current_tick.0 % 3600 == 0 {
            self.ban_list.cleanup_expired();
        }

        // Send snapshots to all players
        self.send_snapshots().await?;

        Ok(())
    }

    /// Spawn all players at arena spawn points
    async fn spawn_all_players(&mut self) {
        let player_ids: Vec<_> = self.sessions.player_ids();
        for player_id in player_ids {
            if let Some(player) = self.sessions.get_mut(player_id) {
                if let Some(spawn) = self.spawn_manager.get_spawn_point(player.team) {
                    // Initial spawn at match start - no invulnerability
                    player.spawn(
                        spawn.position.into(),
                        spawn.rotation,
                        self.current_tick,
                        None,
                    );
                    player.reset_stats();
                    debug!(
                        player_id = %player_id.0,
                        position = ?spawn.position,
                        "Player spawned"
                    );
                }
            }
        }
    }

    /// Reset the world to baseline arena state
    async fn reset_world(&mut self) {
        // Complete the reset and potentially rotate arena
        if let Some(next_arena) = self.match_state.complete_reset(self.current_tick) {
            info!(arena = %next_arena, "Arena rotation - loading next arena");
            // TODO: Actually load the new arena when arena rotation is fully implemented
            self.broadcast_event(GameEvent::ArenaChanged { name: next_arena })
                .await;
        }

        // Clear ready states
        self.sessions.clear_all_ready();
        self.sessions.reset_all_stats();

        info!("World reset complete, returning to Lobby");
    }

    /// Simulate game state for this tick
    async fn simulate_tick(&mut self) {
        let dt = 1.0 / self.tick_config.rate as f32;

        // Collect player states for combat processing
        let mut attack_requests: Vec<(PlayerId, Vec3, Vec3, Tick)> = Vec::new();
        let respawn_delay = self.match_config.respawn_delay_ticks;
        let current_tick = self.current_tick;

        // Collect respawned players for event broadcasting
        let mut respawned_players: Vec<PlayerId> = Vec::new();

        // Process each player's input
        for player in self.sessions.iter_mut() {
            if player.is_dead {
                // Check for respawn
                if let Some(respawn_tick) = player.respawn_tick {
                    if current_tick.0 >= respawn_tick.0 {
                        // Respawn player with invulnerability (T038 - US4)
                        if let Some(spawn) = self.spawn_manager.get_spawn_point(player.team) {
                            player.spawn(
                                spawn.position.into(),
                                spawn.rotation,
                                current_tick,
                                Some(self.combat_config.respawn_invuln_ticks),
                            );
                            respawned_players.push(player.id);
                        }
                    }
                }
                continue;
            }

            // Process pending input
            if let Some(input) = player.pop_input() {
                // Update rotation
                player.rotation.yaw = input.yaw;
                player.rotation.pitch = input.pitch;

                // Apply movement
                let new_pos =
                    self.movement
                        .move_player(player.position, player.velocity, &input, dt);

                // Movement sanity check (US3)
                if let Some(last_valid) = player.anti_cheat.last_valid_position {
                    if let Err(infraction) = validate::validate_movement_delta(
                        last_valid,
                        new_pos,
                        &self.anti_cheat_config,
                    ) {
                        // Reject movement - keep at authoritative position
                        player.anti_cheat.record_infraction(infraction);
                        debug!(
                            player_id = player.id.0,
                            delta = %(new_pos - last_valid).length(),
                            "Movement rejected: sanity check failed"
                        );
                        // Don't update position, keep old_pos
                    } else {
                        // Valid movement, update position
                        player.position = new_pos;
                        player.anti_cheat.update_position(new_pos);
                    }
                } else {
                    // First movement - initialize last_valid_position
                    player.position = new_pos;
                    player.anti_cheat.update_position(new_pos);
                }

                // Collect attack request (with rate limit check - US2)
                if input.attack {
                    if !player.anti_cheat.check_rate_limit(
                        ActionType::Attack,
                        current_tick,
                        &self.anti_cheat_config,
                    ) {
                        player
                            .anti_cheat
                            .record_infraction(InfractionType::AttackRateExceeded);
                        debug!(
                            player_id = player.id.0,
                            "Attack rejected: rate limit exceeded"
                        );
                    } else {
                        let forward = player.rotation.forward();
                        attack_requests.push((
                            player.id,
                            player.position,
                            forward,
                            player.last_attack_tick,
                        ));
                    }
                }
            }
        }

        // Process combat
        for (attacker_id, attacker_pos, attacker_forward, last_attack_tick) in attack_requests {
            // Build target list - exclude dead and invulnerable players (T040, T041 - US4)
            let targets: Vec<(PlayerId, Vec3, u8)> = self
                .sessions
                .iter()
                .filter(|p| !p.is_dead && p.id != attacker_id && !p.is_invulnerable(current_tick))
                .map(|p| (p.id, p.position, p.health))
                .collect();

            // Try attack
            if let Some((target_id, hit_result)) = self.combat.try_attack(
                &self.combat_config,
                attacker_id,
                attacker_pos,
                attacker_forward,
                last_attack_tick,
                current_tick,
                &targets,
            ) {
                // Update attacker's last attack tick
                if let Some(attacker) = self.sessions.get_mut(attacker_id) {
                    attacker.last_attack_tick = current_tick;
                    attacker.kills += if hit_result.killed { 1 } else { 0 };
                }

                // Apply damage and knockback to target
                let new_health;
                let killed;
                if let Some(target) = self.sessions.get_mut(target_id) {
                    let respawn_tick = Tick(current_tick.0 + respawn_delay);
                    killed = target.take_damage(hit_result.damage, respawn_tick);
                    new_health = target.health;

                    // Apply knockback velocity impulse (T031 - US3)
                    // knockback_dir * knockback_strength gives velocity in m/s
                    // Collision system (Feature 008) handles wall stopping
                    if !killed {
                        let knockback_impulse =
                            hit_result.knockback_dir * self.combat_config.knockback_strength;
                        target.velocity = target.velocity + knockback_impulse;
                    }
                } else {
                    continue;
                }

                // Get addresses for event delivery
                let attacker_addr = self.sessions.get(attacker_id).map(|p| p.addr);
                let target_addr = self.sessions.get(target_id).map(|p| p.addr);

                // Send HitConfirmed to attacker
                if let Some(addr) = attacker_addr {
                    let event = GameEvent::HitConfirmed {
                        attacker: attacker_id,
                        target: target_id,
                        damage: hit_result.damage,
                    };
                    let _ = self.send_message(addr, ServerMessage::Event(event)).await;
                }

                // Send DamageTaken to victim
                if let Some(addr) = target_addr {
                    let event = GameEvent::DamageTaken {
                        victim: target_id,
                        attacker: attacker_id,
                        damage: hit_result.damage,
                        new_health,
                    };
                    let _ = self.send_message(addr, ServerMessage::Event(event)).await;
                }

                // Broadcast PlayerDied if killed
                if killed {
                    let event = GameEvent::PlayerDied {
                        victim: target_id,
                        killer: Some(attacker_id),
                    };
                    self.broadcast_event(event).await;

                    // Update match state scores and broadcast ScoreUpdate events (T039, T040)
                    if let Some(attacker) = self.sessions.get(attacker_id) {
                        self.match_state.update_player_score(
                            attacker_id,
                            &attacker.name,
                            attacker.kills,
                            attacker.deaths,
                        );
                        // Broadcast attacker's score update
                        self.broadcast_event(GameEvent::ScoreUpdate {
                            player_id: attacker_id,
                            kills: attacker.kills,
                            deaths: attacker.deaths,
                        })
                        .await;
                    }
                    if let Some(victim) = self.sessions.get(target_id) {
                        self.match_state.update_player_score(
                            target_id,
                            &victim.name,
                            victim.kills,
                            victim.deaths,
                        );
                        // Broadcast victim's score update
                        self.broadcast_event(GameEvent::ScoreUpdate {
                            player_id: target_id,
                            kills: victim.kills,
                            deaths: victim.deaths,
                        })
                        .await;
                    }

                    // Check score limit - may trigger Playing→EndScreen (T041)
                    if let Some(attacker) = self.sessions.get(attacker_id) {
                        if self.match_state.check_score_limit(
                            attacker_id,
                            attacker.kills,
                            current_tick,
                        ) {
                            // Match ended due to score limit
                            if let Some(result) = self.match_state.match_end_result() {
                                info!(
                                    winner = ?result.winner,
                                    reason = ?result.reason,
                                    "Match ended due to score limit"
                                );
                                self.broadcast_event(GameEvent::MatchEnd {
                                    winner: result.winner,
                                    scores: result.scores.clone(),
                                    reason: result.reason,
                                })
                                .await;
                                self.broadcast_event(GameEvent::MatchPhaseChanged {
                                    from: MatchPhase::Playing,
                                    to: MatchPhase::EndScreen,
                                })
                                .await;
                            }
                        }
                    }
                }
            }
        }

        // Broadcast respawn events
        for player_id in respawned_players {
            let event = GameEvent::PlayerRespawned { id: player_id };
            self.broadcast_event(event).await;
        }
    }

    /// Send world snapshots to all connected players
    async fn send_snapshots(&mut self) -> Result<(), ServerError> {
        // Build player snapshots
        let player_snapshots: Vec<PlayerSnapshot> = self
            .sessions
            .iter()
            .map(|p| PlayerSnapshot {
                id: p.id,
                position: p.position,
                rotation: p.rotation,
                health: p.health,
                is_dead: p.is_dead,
                animation: plix_common::protocol::AnimationState::Idle,
            })
            .collect();

        let match_state = self.match_state.state().clone();

        // Send to each player
        let player_addrs: Vec<(SocketAddr, PlayerId, InputSeq, u64)> = self
            .sessions
            .iter()
            .map(|p| (p.addr, p.id, p.last_input_seq, p.last_rtt_nonce))
            .collect();

        for (addr, _player_id, last_input_seq, rtt_nonce) in player_addrs {
            let snapshot = WorldSnapshot {
                tick: self.current_tick,
                last_input_seq,
                players: player_snapshots.clone(),
                match_state: match_state.clone(),
                rtt_nonce_echo: rtt_nonce,
            };

            let msg = ServerMessage::Snapshot(snapshot);
            // Estimate packet size for metrics (encode to get actual size)
            if let Ok(data) = plix_common::protocol::encode(&msg) {
                self.metrics.record_packet_out(data.len());
            }
            self.send_message(addr, msg).await?;
        }

        Ok(())
    }

    /// Handle an incoming packet
    async fn handle_packet(&mut self, data: &[u8], from: SocketAddr) {
        // Decode client message
        let msg: ClientMessage = match plix_common::protocol::decode(data) {
            Ok(m) => m,
            Err(e) => {
                debug!(from = %from, error = %e, "Failed to decode packet");
                return;
            }
        };

        self.handle_message(from, msg).await;
    }

    /// Handle a client message
    async fn handle_message(&mut self, from: SocketAddr, msg: ClientMessage) {
        match msg {
            ClientMessage::Connect {
                protocol_version,
                name,
            } => {
                self.handle_connect(from, protocol_version, name).await;
            }
            ClientMessage::Disconnect => {
                self.handle_disconnect(from).await;
            }
            ClientMessage::Input(input) => {
                if let Some(player) = self.sessions.get_by_addr_mut(&from) {
                    // Validate input (US1)
                    if let Err(infraction) = validate::validate_input(&input) {
                        player.anti_cheat.record_infraction(infraction);
                        debug!(
                            player_id = player.id.0,
                            infraction = ?infraction,
                            "Input rejected: invalid float or out of bounds"
                        );
                        return;
                    }

                    // Check rate limit (US2)
                    if !player.anti_cheat.check_rate_limit(
                        ActionType::Input,
                        self.current_tick,
                        &self.anti_cheat_config,
                    ) {
                        player
                            .anti_cheat
                            .record_infraction(InfractionType::InputRateExceeded);
                        debug!(
                            player_id = player.id.0,
                            "Input rejected: rate limit exceeded"
                        );
                        return;
                    }

                    // Check sequence (US1)
                    if !player.anti_cheat.check_sequence(input.seq) {
                        // Don't record infraction for mild out-of-order (could be network)
                        debug!(
                            player_id = player.id.0,
                            seq = input.seq.0,
                            "Input ignored: duplicate or out-of-order sequence"
                        );
                        return;
                    }

                    // Store RTT nonce for echo in next snapshot
                    player.last_rtt_nonce = input.rtt_nonce;

                    // Record packet for loss detection metrics
                    player.net_metrics.record_packet(input.seq.0);

                    player.queue_input(input);
                }
            }
            ClientMessage::SnapshotAck { tick: _ } => {
                // Update connection metrics
            }
            ClientMessage::BlockEdit(request) => {
                self.handle_block_edit(from, request).await;
            }
            ClientMessage::ReadyToggle => {
                self.handle_ready_toggle(from).await;
            }
        }
    }

    /// Handle ready toggle message from player
    async fn handle_ready_toggle(&mut self, from: SocketAddr) {
        // Only allow ready toggle in Lobby phase
        if self.match_state.phase() != MatchPhase::Lobby {
            return;
        }

        if let Some(player) = self.sessions.get_by_addr_mut(&from) {
            // Check rate limit (US2)
            if !player.anti_cheat.check_rate_limit(
                ActionType::ReadyToggle,
                self.current_tick,
                &self.anti_cheat_config,
            ) {
                player
                    .anti_cheat
                    .record_infraction(InfractionType::ReadyToggleRateExceeded);
                debug!(
                    player_id = player.id.0,
                    "Ready toggle rejected: rate limit exceeded"
                );
                return;
            }

            player.is_ready = !player.is_ready;
            debug!(
                player_id = %player.id.0,
                is_ready = player.is_ready,
                "Player toggled ready"
            );
        }

        // Check if all players are ready and we have enough players
        let player_count = self.sessions.count();
        let min_players = self.match_config.min_players as usize;

        if self.sessions.all_ready() && player_count >= min_players {
            // Start countdown
            self.match_state.start_countdown(self.current_tick);
            info!(tick = %self.current_tick.0, "All players ready, starting countdown");
        }
    }

    /// Handle a connection request
    async fn handle_connect(&mut self, from: SocketAddr, protocol_version: u8, name: String) {
        info!(from = %from, name = %name, version = protocol_version, "Connection request");

        // Check ban list (US4)
        if let Some(ban_entry) = self.ban_list.is_banned(from.ip()) {
            let remaining_secs = ban_entry.remaining().as_secs();
            let msg = ServerMessage::Rejected {
                reason: format!(
                    "You are temporarily banned: {}. {} seconds remaining.",
                    ban_entry.reason, remaining_secs
                ),
            };
            info!(
                from = %from,
                reason = %ban_entry.reason,
                remaining = remaining_secs,
                "Banned IP attempted to connect"
            );
            let _ = self.send_message(from, msg).await;
            return;
        }

        // Check protocol version
        if protocol_version != plix_common::protocol::PROTOCOL_VERSION {
            let msg = ServerMessage::Rejected {
                reason: format!(
                    "Protocol version mismatch: expected {}, got {}",
                    plix_common::protocol::PROTOCOL_VERSION,
                    protocol_version
                ),
            };
            let _ = self.send_message(from, msg).await;
            return;
        }

        // Check if server is full
        if self.sessions.is_full() {
            let msg = ServerMessage::Rejected {
                reason: "Server is full".to_string(),
            };
            let _ = self.send_message(from, msg).await;
            return;
        }

        // Check for duplicate connection
        if self.sessions.get_by_addr(&from).is_some() {
            debug!(from = %from, "Duplicate connection attempt");
            return;
        }

        // Assign team (balance teams)
        let team_0_count = self
            .sessions
            .iter()
            .filter(|p| p.team == TeamId::TEAM_0)
            .count();
        let team_1_count = self
            .sessions
            .iter()
            .filter(|p| p.team == TeamId::TEAM_1)
            .count();
        let team = if team_0_count <= team_1_count {
            TeamId::TEAM_0
        } else {
            TeamId::TEAM_1
        };

        // Add player
        let player_id = match self.sessions.add_player(name.clone(), team, from) {
            Some(id) => id,
            None => {
                let msg = ServerMessage::Rejected {
                    reason: "Failed to create player session".to_string(),
                };
                let _ = self.send_message(from, msg).await;
                return;
            }
        };

        // Spawn player (initial connect - no invulnerability)
        if let Some(spawn) = self.spawn_manager.get_spawn_point(team) {
            if let Some(player) = self.sessions.get_mut(player_id) {
                player.spawn(
                    spawn.position.into(),
                    spawn.rotation,
                    self.current_tick,
                    None,
                );
            }
        }

        info!(
            player_id = player_id.0,
            name = %name,
            team = team.0,
            from = %from,
            "Player connected"
        );

        // Send Connected response
        let msg = ServerMessage::Connected {
            player_id,
            tick: self.current_tick,
            tick_rate: self.tick_config.rate,
            arena_data: self.arena_data.clone(),
        };
        let _ = self.send_message(from, msg).await;

        // Broadcast player joined event
        let event = GameEvent::PlayerJoined {
            id: player_id,
            name,
            team,
        };
        self.broadcast_event(event).await;
    }

    /// Handle a disconnect request
    async fn handle_disconnect(&mut self, from: SocketAddr) {
        if let Some(player) = self.sessions.get_by_addr(&from) {
            let player_id = player.id;
            let name = player.name.clone();

            info!(player_id = player_id.0, name = %name, "Player disconnected");

            self.sessions.remove_player(player_id);

            // Broadcast player left event
            let event = GameEvent::PlayerLeft { id: player_id };
            self.broadcast_event(event).await;
        }
    }

    /// Handle a block edit request - queues it for processing in tick
    async fn handle_block_edit(&mut self, from: SocketAddr, request: BlockEditRequest) {
        if let Some(player) = self.sessions.get_by_addr_mut(&from) {
            // Check rate limit (US2)
            if !player.anti_cheat.check_rate_limit(
                ActionType::BlockEdit,
                self.current_tick,
                &self.anti_cheat_config,
            ) {
                player
                    .anti_cheat
                    .record_infraction(InfractionType::BlockEditRateExceeded);
                debug!(
                    player_id = player.id.0,
                    "Block edit rejected: rate limit exceeded"
                );
                return;
            }

            debug!(
                player_id = player.id.0,
                kind = ?request.kind,
                target = ?request.target_pos,
                "Block edit request received"
            );
            player.queue_edit(request);
        }
    }

    /// Process all pending block edit requests for this tick
    async fn process_block_edits(&mut self) {
        let current_tick = self.current_tick;
        let match_phase = self.match_state.phase();

        // Collect all pending edits with player info
        let pending: Vec<(PlayerId, SocketAddr, Vec<BlockEditRequest>)> = self
            .sessions
            .iter_mut()
            .map(|p| (p.id, p.addr, p.drain_edits()))
            .filter(|(_, _, edits)| !edits.is_empty())
            .collect();

        for (player_id, player_addr, edits) in pending {
            for request in edits {
                self.process_single_edit(
                    player_id,
                    player_addr,
                    request,
                    current_tick,
                    match_phase,
                )
                .await;
            }
        }
    }

    /// Process a single block edit request
    async fn process_single_edit(
        &mut self,
        player_id: PlayerId,
        player_addr: SocketAddr,
        request: BlockEditRequest,
        current_tick: Tick,
        match_phase: MatchPhase,
    ) {
        // Get player data for validation
        let player = match self.sessions.get(player_id) {
            Some(p) => p,
            None => return,
        };

        // Build list of all players for collision check (place only)
        let all_players: Vec<_> = self.sessions.iter().collect();

        // Validate the request
        let validation_result = BlockEditSystem::validate(
            &request,
            player,
            self.movement.arena(),
            &all_players,
            current_tick,
            match_phase,
        );

        match validation_result {
            Ok(()) => {
                // Apply the edit to the arena
                let new_block_type = match request.kind {
                    BlockEditKind::Remove => BlockType::AIR,
                    BlockEditKind::Place => request.block_type,
                };
                self.movement
                    .arena_mut()
                    .set_block(request.target_pos, new_block_type);

                // Update player's last edit tick for rate limiting
                if let Some(player) = self.sessions.get_mut(player_id) {
                    player.last_edit_tick = Some(current_tick);
                }

                // Note: arena_data not updated here - too large for UDP
                // Late joiners get stale arena; proper fix needs fragmentation (TODO)

                debug!(
                    player_id = player_id.0,
                    kind = ?request.kind,
                    target = ?request.target_pos,
                    "Block edit applied"
                );

                // Broadcast BlockEditApplied to all clients
                let applied = BlockEditApplied {
                    pos: request.target_pos,
                    new_block: new_block_type,
                    tick: current_tick,
                };
                let event = GameEvent::BlockEditApplied(applied);
                self.broadcast_event(event).await;
            }
            Err(reason) => {
                debug!(
                    player_id = player_id.0,
                    kind = ?request.kind,
                    target = ?request.target_pos,
                    reason = ?reason,
                    "Block edit rejected"
                );

                // Send BlockEditRejected only to the requester
                let rejected = BlockEditRejected {
                    pos: request.target_pos,
                    reason,
                };
                let event = GameEvent::BlockEditRejected(rejected);
                let _ = self
                    .send_message(player_addr, ServerMessage::Event(event))
                    .await;
            }
        }
    }

    /// Send a message to an address
    async fn send_message(&self, to: SocketAddr, msg: ServerMessage) -> Result<(), ServerError> {
        let data = plix_common::protocol::encode(&msg)?;
        self.transport.send_to(&data, to).await?;
        Ok(())
    }

    /// Broadcast an event to all players
    async fn broadcast_event(&self, event: GameEvent) {
        let msg = ServerMessage::Event(event);
        let player_addrs: Vec<SocketAddr> = self.sessions.iter().map(|p| p.addr).collect();

        for addr in player_addrs {
            let _ = self.send_message(addr, msg.clone()).await;
        }
    }

    /// Evaluate sanctions for all connected players
    async fn evaluate_all_sanctions(&mut self) {
        let player_ids: Vec<PlayerId> = self.sessions.player_ids();
        for player_id in player_ids {
            self.evaluate_sanctions(player_id).await;
        }
    }

    /// Evaluate and apply sanctions for a player based on their current strike count.
    /// Returns true if the player was kicked or banned.
    async fn evaluate_sanctions(&mut self, player_id: PlayerId) -> bool {
        // Get player info needed for sanctions
        let (strikes, addr, should_warn) = {
            let player = match self.sessions.get(player_id) {
                Some(p) => p,
                None => return false,
            };
            (
                player.anti_cheat.strike_count(),
                player.addr,
                sanctions::should_warn(&player.anti_cheat, self.current_tick),
            )
        };

        // Evaluate what sanction to apply
        match sanctions::evaluate(strikes, &self.anti_cheat_config) {
            Some(SanctionType::TempBan) => {
                self.apply_ban(player_id, addr, strikes).await;
                true
            }
            Some(SanctionType::Kick) => {
                self.apply_kick(player_id, addr, strikes).await;
                true
            }
            Some(SanctionType::Warning) if should_warn => {
                self.apply_warning(player_id, addr, strikes).await;
                false
            }
            _ => false,
        }
    }

    /// Apply a warning sanction
    async fn apply_warning(&mut self, player_id: PlayerId, addr: SocketAddr, strikes: u32) {
        // Update last warning tick
        if let Some(player) = self.sessions.get_mut(player_id) {
            player.anti_cheat.last_warning_tick = Some(self.current_tick);
        }

        let msg = ServerMessage::Warning {
            message: "Anti-cheat warning: unusual activity detected".to_string(),
            strikes,
        };
        let _ = self.send_message(addr, msg).await;

        info!(
            player_id = player_id.0,
            strikes = strikes,
            "Anti-cheat warning sent"
        );
    }

    /// Apply a kick sanction
    async fn apply_kick(&mut self, player_id: PlayerId, addr: SocketAddr, strikes: u32) {
        let name = self
            .sessions
            .get(player_id)
            .map(|p| p.name.clone())
            .unwrap_or_default();

        // Reset strikes after kick (give them another chance)
        if let Some(player) = self.sessions.get_mut(player_id) {
            player.anti_cheat.reset_strikes();
        }

        let msg = ServerMessage::Kicked {
            reason: format!("Kicked for suspicious activity ({} infractions)", strikes),
        };
        let _ = self.send_message(addr, msg).await;

        // Remove player session
        self.sessions.remove_player(player_id);

        // Broadcast player left
        let event = GameEvent::PlayerLeft { id: player_id };
        self.broadcast_event(event).await;

        warn!(
            player_id = player_id.0,
            name = %name,
            strikes = strikes,
            "Player kicked for anti-cheat violations"
        );
    }

    /// Apply a temporary ban sanction
    async fn apply_ban(&mut self, player_id: PlayerId, addr: SocketAddr, strikes: u32) {
        let name = self
            .sessions
            .get(player_id)
            .map(|p| p.name.clone())
            .unwrap_or_default();

        // Add to ban list
        let ban_duration =
            std::time::Duration::from_secs(self.anti_cheat_config.ban_duration_seconds);
        self.ban_list.add_ban(
            addr.ip(),
            format!("Anti-cheat: {} infractions", strikes),
            ban_duration,
        );

        let msg = ServerMessage::Kicked {
            reason: format!(
                "Temporarily banned for suspicious activity. Duration: {} seconds",
                self.anti_cheat_config.ban_duration_seconds
            ),
        };
        let _ = self.send_message(addr, msg).await;

        // Remove player session
        self.sessions.remove_player(player_id);

        // Broadcast player left
        let event = GameEvent::PlayerLeft { id: player_id };
        self.broadcast_event(event).await;

        warn!(
            player_id = player_id.0,
            name = %name,
            strikes = strikes,
            ban_duration = self.anti_cheat_config.ban_duration_seconds,
            ip = %addr.ip(),
            "Player banned for anti-cheat violations"
        );
    }
}
