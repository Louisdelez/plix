//! Plix Server - Authoritative game server
//!
//! This crate implements the authoritative game server:
//! - Fixed tick loop (60 Hz default)
//! - Player session management
//! - Server-side simulation (movement, collision, combat)
//! - Block physics (gravity, liquids)
//! - Snapshot replication with delta encoding
//! - Input validation and anti-cheat
//! - Match state management

pub mod anti_cheat;
pub mod block_physics;
pub mod br_lite;
pub mod crafting;
pub mod ctf;
pub mod economy;
pub mod identity;
pub mod inventory;
pub mod loot;
pub mod master_announce;
pub mod match_state;
pub mod metrics;
pub mod netloop;
pub mod persist;
pub mod replication;
pub mod session;
pub mod sim;
pub mod tick;
pub mod training;
pub mod validation;
pub mod weapons;

use std::net::SocketAddr;
use std::path::PathBuf;

use thiserror::Error;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tracing::{debug, info, warn};

use plix_arena::{load_arena, SpawnManager};
use plix_common::chunk::{Chunk, CHUNK_SIZE};
use plix_common::combat::CombatConfig;
use plix_common::identity::SessionId;
use plix_common::math::Vec3;
use plix_common::protocol::{
    BlockEditApplied, BlockEditKind, BlockEditRejected, BlockEditRequest, ClientMessage, GameEvent,
    MatchPhase, PlayerSnapshot, ProjectileDespawnReason, ProjectileImpactType, RenameRejectReason,
    ServerMessage, WorldSnapshot,
};
use plix_common::time::Tick;
use plix_common::types::BlockType;
use plix_common::types::{BlockPos, InputSeq, PlayerId, TeamId};
use plix_common::world::ChunkedWorld;
use plix_net::UdpTransport;

use crate::anti_cheat::{
    sanctions, validate, ActionType, AntiCheatConfig, BanList, InfractionType, SanctionType,
};
use crate::crafting::{get_craft_config, CraftMetrics, CraftSystem};
use crate::ctf::{CtfConfig, CtfCoordinator, CtfState};
use crate::identity::NameRegistry;
use crate::inventory::{
    decrement_active_consumable, try_pickup, use_active_item, InventoryConfig, PickupResult,
    UseResult,
};
use crate::loot::{drop_player_items, mode_drops_items, LootManager};
use crate::match_state::{MatchConfig, MatchStateMachine};
use crate::metrics::ServerMetricsCollector;
use crate::persist::{SaveScheduler, SaveSchedulerConfig, WorldStore};
use crate::session::SessionManager;
use crate::sim::block_edit::BlockEditSystem;
use crate::sim::combat::CombatSystem;
use crate::sim::movement::MovementSystem;
use crate::tick::TickConfig;
use crate::training::{TrainingConfig, TrainingCoordinator, TrainingEvent};
use crate::weapons::{
    get_weapon_def,
    projectiles::{ProjectileImpactType as ServerImpactType, ProjectileTarget, WorldCollision},
    ranged::RangedSystem,
    ProjectileManager,
};
use plix_common::inventory::{ItemId, WeaponType};
use plix_common::protocol::{CraftRejectReason, InventorySnapshot, SlotSnapshot};

/// Server errors
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to load arena: {0}")]
    ArenaLoad(#[from] plix_arena::loader::LoadError),

    #[error("Protocol error: {0}")]
    Protocol(#[from] plix_common::protocol::ProtocolError),

    #[error("Persistence error: {0}")]
    Persist(#[from] plix_common::persist::PersistError),
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
    /// Enable world persistence
    pub persistence_enabled: bool,
    /// World ID for persistence (if enabled)
    pub world_id: Option<String>,
    /// Auto-save interval in seconds (default: 300 = 5 min)
    pub auto_save_interval_secs: u64,
    /// Block physics configuration
    pub block_physics: plix_common::block_physics::BlockPhysicsConfig,
    /// Master server announcement configuration
    pub master_config: master_announce::MasterConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 7777,
            tick_rate: 60,
            max_players: 16,
            arena_name: "test_arena".to_string(),
            assets_dir: PathBuf::from("assets"),
            persistence_enabled: false,
            world_id: None,
            auto_save_interval_secs: 300,
            block_physics: plix_common::block_physics::BlockPhysicsConfig::default(),
            master_config: master_announce::MasterConfig::default(),
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
    /// World persistence store (optional)
    world_store: Option<WorldStore>,
    /// Auto-save scheduler
    save_scheduler: SaveScheduler,
    /// Modified blocks tracked as chunks for persistence
    modified_world: ChunkedWorld,
    /// Block physics system (gravity, liquids)
    block_physics: block_physics::BlockPhysicsSystem,
    /// CTF game mode coordinator (None if not CTF mode)
    ctf_coordinator: Option<CtfCoordinator>,
    /// Training mode coordinator (None if not Training mode)
    training_coordinator: Option<TrainingCoordinator>,
    /// Loot entity manager for world loot drops
    loot_manager: LootManager,
    /// Inventory configuration (pickup range, etc.)
    inventory_config: InventoryConfig,
    /// Projectile manager for ranged combat
    projectile_manager: ProjectileManager,
    /// Crafting metrics for observability
    craft_metrics: CraftMetrics,
    /// Economy shop registry
    shop_registry: economy::ShopRegistry,
    /// Economy metrics for observability
    economy_metrics: economy::EconomyMetrics,
    /// Name registry for unique display names
    name_registry: NameRegistry,
    /// Session ID counter for unique session IDs
    next_session_id: u64,
    /// Heartbeat state for master server announcements
    heartbeat_state: std::sync::Arc<master_announce::heartbeat::HeartbeatState>,
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

        // Extract game mode before arena is moved
        let game_mode = loaded_arena.definition.metadata.game_mode;

        info!(
            game_mode = %game_mode,
            "Game mode detected from arena config"
        );

        // Extract flag zones for CTF mode (before arena is moved)
        let flag_zones = loaded_arena.flag_zones();

        // Extract training config and bot spawns for Training mode (before arena is moved)
        let training_arena_config = loaded_arena.training_config().cloned();
        let training_bot_spawns = loaded_arena.training_bot_spawns();

        // Serialize arena metadata only (not full block data - too large for UDP)
        // Client loads arena locally; block edits replicate as deltas
        let arena_data = plix_common::protocol::encode(&loaded_arena.definition)?;

        // Create spawn manager
        let spawn_manager = SpawnManager::new(loaded_arena.definition.spawn_points.clone());

        // Create movement system with arena collision
        let mut movement = MovementSystem::new(loaded_arena);

        // Create session manager
        let sessions = SessionManager::new(config.max_players);

        // Create match state machine with game mode from arena config
        let match_config = match game_mode {
            plix_common::GameMode::Ffa => MatchConfig::ffa_default(),
            plix_common::GameMode::Tdm => MatchConfig::tdm_default(),
            plix_common::GameMode::Ctf => MatchConfig::ctf_default(),
            plix_common::GameMode::BrLite => MatchConfig::br_lite_default(),
            plix_common::GameMode::Training => MatchConfig::training_default(),
        };
        let match_state =
            MatchStateMachine::new(match_config.clone(), config.arena_name.clone(), game_mode);

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

        // Initialize persistence if enabled
        let mut modified_world = ChunkedWorld::new();
        let (world_store, save_scheduler) = if config.persistence_enabled {
            let base_dir = crate::persist::default_worlds_dir();
            let world_id = config
                .world_id
                .clone()
                .unwrap_or_else(|| format!("server_{}", config.arena_name));

            // Try to open existing world or create new one
            let metadata = plix_common::persist::WorldMetadata::new_arena(
                world_id.clone(),
                format!("Server: {}", config.arena_name),
                config.arena_name.clone(),
            );

            let store = WorldStore::open_or_create(&base_dir, metadata)?;

            // Load any previously saved chunks and apply to arena
            let saved_chunks = store.list_saved_chunks()?;
            let mut loaded_count = 0;
            for coord in saved_chunks {
                if let Ok(Some(chunk)) = store.load_chunk(coord) {
                    // Apply saved blocks to arena
                    for lx in 0..CHUNK_SIZE {
                        for ly in 0..CHUNK_SIZE {
                            for lz in 0..CHUNK_SIZE {
                                let block = chunk.get_block(lx, ly, lz);
                                if block != BlockType::AIR {
                                    let world_x = coord.x * CHUNK_SIZE as i32 + lx as i32;
                                    let world_y = coord.y * CHUNK_SIZE as i32 + ly as i32;
                                    let world_z = coord.z * CHUNK_SIZE as i32 + lz as i32;
                                    if world_x >= 0 && world_y >= 0 && world_z >= 0 {
                                        let block_pos = plix_common::types::BlockPos::new(
                                            world_x, world_y, world_z,
                                        );
                                        movement.arena_mut().set_block(block_pos, block);
                                    }
                                }
                            }
                        }
                    }
                    // Keep chunk in modified_world for future saves
                    modified_world.insert_chunk(coord, chunk);
                    loaded_count += 1;
                }
            }
            if loaded_count > 0 {
                info!(chunks = loaded_count, "Loaded saved world modifications");
            }

            let scheduler_config = SaveSchedulerConfig {
                interval: Duration::from_secs(config.auto_save_interval_secs),
                max_chunks_per_cycle: 100,
                enabled: true,
            };
            let scheduler = SaveScheduler::new(scheduler_config);

            info!(
                world_id = %world_id,
                interval_secs = config.auto_save_interval_secs,
                "World persistence enabled"
            );

            (Some(store), scheduler)
        } else {
            // Persistence disabled - scheduler with no-op config
            let scheduler = SaveScheduler::new(SaveSchedulerConfig::disabled());
            (None, scheduler)
        };

        // Create block physics system
        let block_physics = block_physics::BlockPhysicsSystem::new(config.block_physics.clone());

        // Create CTF coordinator if in CTF mode
        let ctf_coordinator = if game_mode == plix_common::GameMode::Ctf {
            let ctf_config = CtfConfig::from_arena(
                Some(match_config.score_limit),
                None, // Use default flag return delay
                Some((match_config.respawn_delay_ticks / 60) as u32), // Convert ticks to seconds
                Some(match_config.time_limit_seconds),
            );
            let ctf_state = CtfState::new(flag_zones, ctf_config);
            info!(
                capture_limit = ctf_state.config.capture_limit,
                flag_return_delay = ctf_state.config.flag_return_delay_ticks,
                "CTF coordinator initialized"
            );
            Some(CtfCoordinator::new(ctf_state))
        } else {
            None
        };

        // Create Training coordinator if in Training mode
        let training_coordinator = if game_mode == plix_common::GameMode::Training {
            // Build TrainingConfig from arena config or use defaults
            let mut training_config = TrainingConfig::default();
            if let Some(ref arena_cfg) = training_arena_config {
                if let Some(count) = arena_cfg.bot_count {
                    training_config.bot_count = count;
                }
                if let Some(ref behavior) = arena_cfg.bot_behavior {
                    training_config.bot_behavior = match behavior.to_lowercase().as_str() {
                        "roam" => crate::training::BotBehaviorType::Roam,
                        "strafe" => crate::training::BotBehaviorType::Strafe,
                        _ => crate::training::BotBehaviorType::Dummy,
                    };
                }
                if let Some(delay_secs) = arena_cfg.bot_respawn_delay_secs {
                    training_config.bot_respawn_delay_ticks = (delay_secs * 60.0) as u32;
                }
                if let Some(delay_secs) = arena_cfg.player_respawn_delay_secs {
                    training_config.player_respawn_delay_ticks = (delay_secs * 60.0) as u32;
                }
                if let Some(inv) = arena_cfg.invincibility_player {
                    training_config.invincibility_player = inv;
                }
                if let Some(inv) = arena_cfg.invincibility_bots {
                    training_config.invincibility_bots = inv;
                }
            }

            // Use bot spawns from arena, or fall back to player spawns
            let bot_spawns = if !training_bot_spawns.is_empty() {
                training_bot_spawns
            } else {
                // Fall back to player spawn points
                spawn_manager
                    .get_all_spawns()
                    .iter()
                    .map(|s| s.position_vec3())
                    .collect()
            };

            info!(
                bot_count = training_config.bot_count,
                bot_behavior = ?training_config.bot_behavior,
                bot_spawns = bot_spawns.len(),
                "Training coordinator initialized"
            );

            Some(TrainingCoordinator::new(
                training_config,
                bot_spawns,
                Tick::ZERO,
            ))
        } else {
            None
        };

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
            world_store,
            save_scheduler,
            modified_world,
            block_physics,
            ctf_coordinator,
            training_coordinator,
            loot_manager: LootManager::new(),
            inventory_config: InventoryConfig::default(),
            projectile_manager: ProjectileManager::new(),
            craft_metrics: CraftMetrics::new(),
            shop_registry: economy::ShopRegistry::new(economy::shop::default_shop_offers()),
            economy_metrics: economy::EconomyMetrics::new(),
            name_registry: NameRegistry::new(),
            next_session_id: 1,
            heartbeat_state: master_announce::heartbeat::HeartbeatState::new(),
        })
    }

    /// Run the server main loop
    pub async fn run(mut self) -> Result<(), ServerError> {
        info!(
            tick_rate = self.tick_config.rate,
            max_players = self.config.max_players,
            "Server starting main loop"
        );

        // Start heartbeat task for master server announcements
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let heartbeat_state = self.heartbeat_state.clone();

        if self.config.master_config.enabled {
            let heartbeat_task = master_announce::HeartbeatTask::new(
                self.config.master_config.clone(),
                self.transport.local_addr()?.ip().to_string(),
                self.config.port,
                self.config.max_players,
                heartbeat_state,
                shutdown_rx,
            );
            tokio::spawn(heartbeat_task.run());
        }

        let tick_duration = Duration::from_secs_f64(1.0 / self.tick_config.rate as f64);
        let mut tick_interval = interval(tick_duration);
        tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                // Handle graceful shutdown (Ctrl+C)
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutdown signal received, saving world...");
                    // Signal heartbeat task to stop
                    let _ = shutdown_tx.send(true);
                    self.shutdown().await;
                    info!("Server shutdown complete");
                    return Ok(());
                }

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

    /// Graceful shutdown - flush persistence and notify clients
    async fn shutdown(&mut self) {
        // Flush all dirty chunks to disk
        if let Some(ref mut store) = self.world_store {
            match self.save_scheduler.flush(&self.modified_world, store) {
                Ok(saved) => {
                    if saved > 0 {
                        info!(chunks = saved, "Flushed world changes to disk");
                    }
                }
                Err(e) => {
                    warn!(error = %e, "Failed to flush world on shutdown");
                }
            }
        }

        // Notify all connected players
        let player_addrs: Vec<SocketAddr> = self.sessions.iter().map(|p| p.addr).collect();
        let msg = ServerMessage::Kicked {
            reason: "Server shutting down".to_string(),
        };
        for addr in player_addrs {
            let _ = self.send_message(addr, msg.clone()).await;
        }
    }

    /// Process a single server tick
    async fn tick(&mut self) -> Result<(), ServerError> {
        self.current_tick = self.current_tick.next();

        // Update heartbeat state with current player count
        self.heartbeat_state
            .set_player_count(self.sessions.count() as u8);

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
            // Tick projectiles (movement, collision, expiration)
            self.tick_projectiles().await;
            // Process block edits (triggers physics event detection)
            self.process_block_edits().await;
            // Run block physics simulation
            self.process_block_physics().await;

            // Process training mode tick (bot behavior and respawns)
            if let Some(ref mut training) = self.training_coordinator {
                let events = training.tick(self.current_tick);
                for event in events {
                    match event {
                        TrainingEvent::BotRespawned { bot_id, position } => {
                            debug!(
                                bot_id = bot_id.0,
                                position = ?position,
                                "Bot respawned"
                            );
                            self.broadcast_event(GameEvent::BotRespawned { bot_id, position })
                                .await;
                        }
                    }
                }
            }
        }

        // Evaluate sanctions for all players (US4)
        self.evaluate_all_sanctions().await;

        // Periodically clean up expired bans (every ~60 seconds)
        if self.current_tick.0 % 3600 == 0 {
            self.ban_list.cleanup_expired();
        }

        // Process auto-save (if persistence enabled and interval elapsed)
        if let Some(ref store) = self.world_store {
            if let Err(e) = self.save_scheduler.tick(&self.modified_world, store) {
                warn!(error = %e, "Auto-save error");
            }
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
        // T043: Use training respawn delay for training mode, else use match config
        let respawn_delay = if let Some(ref training) = self.training_coordinator {
            training.player_respawn_delay_ticks()
        } else {
            self.match_config.respawn_delay_ticks
        };
        let current_tick = self.current_tick;

        // Collect respawned players for event broadcasting
        let mut respawned_players: Vec<PlayerId> = Vec::new();

        // Get game mode for FFA vs TDM spawn selection (T024)
        let game_mode = self.match_state.game_mode();

        // Process each player's input
        for player in self.sessions.iter_mut() {
            if player.is_dead {
                // Check for respawn
                if let Some(respawn_tick) = player.respawn_tick {
                    if current_tick.0 >= respawn_tick.0 {
                        // Get spawn point based on game mode (T024)
                        let spawn = match game_mode {
                            plix_common::GameMode::Ffa => {
                                // FFA: Use any available spawn (ignoring team)
                                self.spawn_manager.get_ffa_spawn_point()
                            }
                            plix_common::GameMode::Tdm | plix_common::GameMode::Ctf => {
                                // TDM/CTF: Use team-specific spawn
                                self.spawn_manager.get_spawn_point(player.team)
                            }
                            plix_common::GameMode::BrLite => {
                                // BR Lite: No respawn (permanent elimination)
                                None
                            }
                            plix_common::GameMode::Training => {
                                // Training: Use any available spawn (fast respawn)
                                self.spawn_manager.get_ffa_spawn_point()
                            }
                        };

                        // Respawn player with invulnerability (T038 - US4)
                        if let Some(spawn) = spawn {
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

        // Process combat (players)
        for &(attacker_id, attacker_pos, attacker_forward, last_attack_tick) in &attack_requests {
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

                // T044: Check if player is invincible in training mode
                let player_invincible = if let Some(ref training) = self.training_coordinator {
                    training.is_player_invincible()
                } else {
                    false
                };

                // Apply damage and knockback to target
                let new_health;
                let killed;
                if let Some(target) = self.sessions.get_mut(target_id) {
                    if player_invincible {
                        // Player is invincible - no damage, just acknowledge hit
                        killed = false;
                        new_health = target.health;
                    } else {
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
                        } else {
                            // Set spectate target to killer (TDM - US2 spectate mode)
                            target.spectate_target = Some(attacker_id);
                        }
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

                    // Economy: Award kill reward to attacker (T021)
                    self.award_kill_reward(attacker_id).await;

                    // US5: Death drops - spawn loot from player's hotbar (T072-T075)
                    let game_mode = self.match_state.game_mode();
                    if mode_drops_items(game_mode) {
                        if let Some(victim) = self.sessions.get(target_id) {
                            let death_pos = victim.position;
                            let spawned_loot = drop_player_items(
                                &victim.hotbar,
                                death_pos,
                                &mut self.loot_manager,
                                self.current_tick,
                            );

                            // Send LootSpawned events for each dropped item
                            for loot_id in &spawned_loot {
                                if let Some(loot) = self.loot_manager.get(*loot_id) {
                                    let event = GameEvent::LootSpawned {
                                        loot_id: *loot_id,
                                        position: loot.position,
                                        item_id: loot.item.item_id,
                                        quantity: loot.item.quantity,
                                    };
                                    self.broadcast_event(event).await;
                                }
                            }

                            debug!(
                                victim_id = target_id.0,
                                items_dropped = spawned_loot.len(),
                                "Player death drops spawned"
                            );
                        }

                        // Clear the victim's hotbar after dropping items
                        if let Some(victim) = self.sessions.get_mut(target_id) {
                            for slot in 0..victim.hotbar.size() {
                                victim.hotbar.set(slot as u8, None);
                            }
                        }
                    }

                    // Get team info for TDM team scoring (T013, T014)
                    let attacker_team = self.sessions.get(attacker_id).map(|p| p.team);
                    let victim_team = self.sessions.get(target_id).map(|p| p.team);
                    let is_enemy_kill = match (attacker_team, victim_team) {
                        (Some(at), Some(vt)) => at != vt,
                        _ => false, // Can't determine teams, no team score
                    };

                    // FFA mode: Log kill event (T018)
                    let game_mode = self.match_state.game_mode();
                    if game_mode == plix_common::GameMode::Ffa {
                        info!(
                            killer = %attacker_id,
                            victim = %target_id,
                            "FFA kill"
                        );
                    }

                    // Award team point for enemy kills - TDM only (T015)
                    // FFA mode skips team scoring entirely
                    if game_mode == plix_common::GameMode::Tdm && is_enemy_kill {
                        if let Some(team) = attacker_team {
                            self.match_state.award_team_kill(team);
                            // Debug log for team score update (T016)
                            debug!(
                                "Team score: Red={}, Blue={}",
                                self.match_state.get_team_score(TeamId::TEAM_0),
                                self.match_state.get_team_score(TeamId::TEAM_1)
                            );
                        }
                    }

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

                    // Check score limits based on game mode
                    match game_mode {
                        plix_common::GameMode::Tdm => {
                            // TDM mode: Check team score limit - may trigger Playing→EndScreen
                            if is_enemy_kill {
                                if let Some(team) = attacker_team {
                                    if self.match_state.check_team_score_limit(team, current_tick) {
                                        // Match ended due to team score limit
                                        info!(
                                            team_winner = ?team,
                                            team_score = self.match_state.get_team_score(team),
                                            "Match ended due to team score limit"
                                        );
                                        if let Some(result) = self.match_state.match_end_result() {
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
                        plix_common::GameMode::Ffa => {
                            // FFA mode: Check individual score limit - may trigger Playing→EndScreen (T029)
                            if let Some(attacker) = self.sessions.get(attacker_id) {
                                if self.match_state.check_score_limit(
                                    attacker_id,
                                    attacker.kills,
                                    current_tick,
                                ) {
                                    // Match ended due to individual score limit (FFA)
                                    if let Some(result) = self.match_state.match_end_result() {
                                        info!(
                                            winner = ?result.winner,
                                            kills = attacker.kills,
                                            "FFA match ended due to score limit"
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
                        plix_common::GameMode::Ctf => {
                            // CTF mode: Victory is determined by capture limit, not kills
                            // Kill tracking handled above, but victory check happens in CTF coordinator
                        }
                        plix_common::GameMode::BrLite => {
                            // BR Lite mode: Victory is determined by last player standing
                            // Elimination handling done in BR Lite coordinator
                        }
                        plix_common::GameMode::Training => {
                            // Training mode: No victory condition (unlimited practice)
                            // Stats tracking done separately
                        }
                    }
                }
            }
        }

        // Training mode: Process bot attacks if no player was hit
        // (Check bots as potential targets for attacks that missed players)
        if self.training_coordinator.is_some() {
            // Collect attacks that didn't hit players to try against bots
            for (attacker_id, attacker_pos, attacker_forward, last_attack_tick) in attack_requests {
                // Check cooldown using config value
                let ticks_since_attack = current_tick.0.wrapping_sub(last_attack_tick.0);
                if ticks_since_attack < self.combat_config.attack_cooldown_ticks {
                    continue;
                }

                // Record attack attempt for stats
                if let Some(ref mut training) = self.training_coordinator {
                    training.record_attack();
                }

                let effective_range = self.combat_config.effective_range();

                // Find closest bot in range and in front
                let mut best_bot: Option<(plix_common::types::BotId, Vec3)> = None;
                let mut best_dist = effective_range;

                if let Some(ref training) = self.training_coordinator {
                    for bot in &training.bots {
                        if bot.is_dead {
                            continue;
                        }

                        let to_bot = bot.position - attacker_pos;
                        let distance = to_bot.length();

                        if distance > effective_range {
                            continue;
                        }

                        // Check if bot is roughly in front (within 90 degree cone)
                        let to_bot_norm = to_bot.normalize_or_zero();
                        let dot = attacker_forward.dot(to_bot_norm);

                        if dot < 0.0 {
                            // Behind attacker
                            continue;
                        }

                        if distance < best_dist {
                            best_dist = distance;
                            best_bot = Some((bot.id, bot.position));
                        }
                    }
                }

                // If we found a bot to hit
                if let Some((bot_id, _bot_pos)) = best_bot {
                    let damage = crate::sim::combat::MELEE_DAMAGE;

                    // Process hit through coordinator
                    let killed = if let Some(ref mut training) = self.training_coordinator {
                        training.process_hit(bot_id, damage, current_tick)
                    } else {
                        false
                    };

                    // Update attacker's last attack tick
                    if let Some(attacker) = self.sessions.get_mut(attacker_id) {
                        attacker.last_attack_tick = current_tick;
                    }

                    // Send BotHit event to attacker
                    if let Some(attacker) = self.sessions.get(attacker_id) {
                        debug!(
                            attacker_id = attacker_id.0,
                            bot_id = bot_id.0,
                            damage = damage,
                            killed = killed,
                            "Bot hit"
                        );
                        let event = GameEvent::BotHit {
                            bot_id,
                            damage,
                            killed,
                        };
                        let _ = self
                            .send_message(attacker.addr, ServerMessage::Event(event))
                            .await;
                    }
                }
            }
        }

        // Broadcast respawn events
        for player_id in respawned_players {
            let event = GameEvent::PlayerRespawned { id: player_id };
            self.broadcast_event(event).await;
        }

        // Check for automatic item pickups (US2 - T039)
        self.check_pickups().await;
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
                spectate_target: p.spectate_target,
                display_name: p.name.clone(),
            })
            .collect();

        let match_state = self.match_state.state().clone();

        // Get bot snapshots for training mode
        let bot_snapshots = if let Some(ref training) = self.training_coordinator {
            training.bot_snapshots()
        } else {
            Vec::new()
        };

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
                bots: bot_snapshots.clone(),
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
                account_id: _, // v2 placeholder - ignored in v1
                auth_token: _, // v2 placeholder - ignored in v1
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
            ClientMessage::TrainingReset => {
                self.handle_training_reset(from).await;
            }
            ClientMessage::TrainingStatsRequest => {
                self.handle_training_stats_request(from).await;
            }
            ClientMessage::SelectHotbarSlot { slot } => {
                self.handle_select_slot(from, slot).await;
            }
            ClientMessage::UseActiveItem => {
                self.handle_use_active_item(from).await;
            }
            ClientMessage::CraftRequest { recipe_id } => {
                self.handle_craft_request(from, recipe_id).await;
            }

            // Economy messages
            ClientMessage::BuyRequest { offer_id } => {
                self.handle_buy_request(from, offer_id).await;
            }
            ClientMessage::BalanceRequest => {
                self.handle_balance_request(from).await;
            }
            ClientMessage::ShopListRequest => {
                self.handle_shop_list_request(from).await;
            }

            // Identity messages (Feature 025)
            ClientMessage::RenameRequest { new_name } => {
                self.handle_rename_request(from, new_name).await;
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

    /// Handle training reset request (T049-T052)
    async fn handle_training_reset(&mut self, from: SocketAddr) {
        // Only process if in training mode
        if self.match_state.game_mode() != plix_common::GameMode::Training {
            debug!(from = %from, "Training reset ignored: not in training mode");
            return;
        }

        // Get player for rate limiting and respawning
        let player_id = match self.sessions.get_by_addr(&from) {
            Some(player) => player.id,
            None => return,
        };

        // T052: Rate limiting (once per second)
        if let Some(player) = self.sessions.get_by_addr_mut(&from) {
            if !player.anti_cheat.check_rate_limit(
                ActionType::TrainingReset,
                self.current_tick,
                &self.anti_cheat_config,
            ) {
                player
                    .anti_cheat
                    .record_infraction(InfractionType::TrainingResetRateExceeded);
                debug!(
                    player_id = player.id.0,
                    "Training reset rejected: rate limit exceeded"
                );
                return;
            }
        }

        // T048: Reset training coordinator (bots, stats)
        let spawn_pos = if let Some(ref mut training) = self.training_coordinator {
            training.reset(self.current_tick)
        } else {
            return;
        };

        // T050: Reposition player to spawn point
        if let Some(player) = self.sessions.get_mut(player_id) {
            player.position = spawn_pos;
            player.velocity = glam::Vec3::ZERO;
            player.is_dead = false;
            player.health = 100;
            player.respawn_tick = None;
            info!(
                player_id = player_id.0,
                pos = ?spawn_pos,
                "Training session reset"
            );
        }

        // T051: Broadcast TrainingReset event to all clients
        self.broadcast_event(GameEvent::TrainingReset { player_id })
            .await;
    }

    /// Handle training stats request (T057-T060)
    async fn handle_training_stats_request(&mut self, from: SocketAddr) {
        // Only process if in training mode
        if self.match_state.game_mode() != plix_common::GameMode::Training {
            debug!(from = %from, "Training stats request ignored: not in training mode");
            return;
        }

        // Get training coordinator stats
        let training = match &self.training_coordinator {
            Some(t) => t,
            None => return,
        };

        // T059: Calculate accuracy
        let accuracy = training.stats().accuracy();

        // T060: Calculate session duration
        let duration_secs = training
            .stats()
            .duration_secs(self.current_tick, self.tick_config.rate as u8);

        // T058: Build and send stats response
        let attacks = training.stats().attacks;
        let hits = training.stats().hits;
        let kills = training.stats().kills;

        let msg = ServerMessage::TrainingStats {
            hits,
            kills,
            attacks,
            accuracy_pct: accuracy,
            session_duration_secs: duration_secs,
        };
        let _ = self.send_message(from, msg).await;

        debug!(
            from = %from,
            attacks = attacks,
            hits = hits,
            kills = kills,
            accuracy = %format!("{:.1}%", accuracy),
            duration = %format!("{:.1}s", duration_secs),
            "Sent training stats"
        );
    }

    /// Handle rename request from player (Feature 025 - US4 - T050-T053)
    async fn handle_rename_request(&mut self, from: SocketAddr, new_name: String) {
        // T050: Get player info for processing
        let player_id = match self.sessions.get_by_addr(&from) {
            Some(p) => p.id,
            None => return,
        };

        // T051: Check rate limit (60 seconds between renames)
        {
            let player = match self.sessions.get(player_id) {
                Some(p) => p,
                None => return,
            };

            if !player.can_rename(self.current_tick) {
                // Send rate limited rejection
                let msg = ServerMessage::Event(GameEvent::RenameResult {
                    success: false,
                    new_name: None,
                    reason: Some(RenameRejectReason::RateLimited),
                });
                let _ = self.send_message(from, msg).await;
                debug!(player_id = player_id.0, "Rename rejected: rate limited");
                return;
            }
        }

        // T052: Validate the new name
        if !plix_common::identity::validate_display_name(&new_name) {
            let msg = ServerMessage::Event(GameEvent::RenameResult {
                success: false,
                new_name: None,
                reason: Some(RenameRejectReason::InvalidName),
            });
            let _ = self.send_message(from, msg).await;
            debug!(
                player_id = player_id.0,
                requested_name = %new_name,
                "Rename rejected: invalid name format"
            );
            return;
        }

        // T053: Try to register the new name via NameRegistry
        let final_name = match self.name_registry.rename(player_id, &new_name) {
            Some(name) => name,
            None => {
                // Name unavailable (all suffix slots full)
                let msg = ServerMessage::Event(GameEvent::RenameResult {
                    success: false,
                    new_name: None,
                    reason: Some(RenameRejectReason::NameUnavailable),
                });
                let _ = self.send_message(from, msg).await;
                debug!(
                    player_id = player_id.0,
                    requested_name = %new_name,
                    "Rename rejected: name unavailable"
                );
                return;
            }
        };

        // Get old name for broadcast
        let old_name = {
            let player = self.sessions.get(player_id).unwrap();
            player.name.clone()
        };

        // Update player state
        {
            let player = self.sessions.get_mut(player_id).unwrap();
            player.name = final_name.clone();
            player.last_rename_tick = Some(self.current_tick);
        }

        info!(
            player_id = player_id.0,
            session_id = %self.sessions.get(player_id).unwrap().session_id.0,
            old_name = %old_name,
            new_name = %final_name,
            "Player renamed"
        );

        // Send success to requesting player
        let msg = ServerMessage::Event(GameEvent::RenameResult {
            success: true,
            new_name: Some(final_name.clone()),
            reason: None,
        });
        let _ = self.send_message(from, msg).await;

        // Broadcast PlayerRenamed event to all players
        self.broadcast_event(GameEvent::PlayerRenamed {
            player_id,
            old_name,
            new_name: final_name,
        })
        .await;
    }

    /// Handle hotbar slot selection (US1 - T025)
    async fn handle_select_slot(&mut self, from: SocketAddr, slot: u8) {
        if let Some(player) = self.sessions.get_by_addr_mut(&from) {
            // Check rate limit for slot selection (T028)
            if !player.anti_cheat.check_rate_limit(
                ActionType::SlotSelect,
                self.current_tick,
                &self.anti_cheat_config,
            ) {
                debug!(
                    player_id = player.id.0,
                    slot = slot,
                    "Slot selection rejected: rate limit exceeded"
                );
                return;
            }

            // Validate slot bounds and set active slot (T021)
            if player.hotbar.set_active_slot(slot) {
                debug!(
                    player_id = player.id.0,
                    slot = slot,
                    "Player selected hotbar slot"
                );
            } else {
                debug!(
                    player_id = player.id.0,
                    slot = slot,
                    hotbar_size = player.hotbar.size(),
                    "Slot selection rejected: invalid slot index"
                );
            }
        }
    }

    /// Handle active item usage (US3 - T052)
    async fn handle_use_active_item(&mut self, from: SocketAddr) {
        // Only allow item usage during Playing phase
        if self.match_state.phase() != MatchPhase::Playing {
            return;
        }

        let player_id = match self.sessions.get_by_addr(&from) {
            Some(p) => p.id,
            None => return,
        };

        // Check rate limit for inventory usage (T053)
        if let Some(player) = self.sessions.get_by_addr_mut(&from) {
            if !player.anti_cheat.check_rate_limit(
                ActionType::InventoryUse,
                self.current_tick,
                &self.anti_cheat_config,
            ) {
                debug!(
                    player_id = player.id.0,
                    "Item usage rejected: rate limit exceeded"
                );
                return;
            }

            // Don't allow dead players to use items
            if player.is_dead {
                return;
            }
        }

        // Get the use result from the player's hotbar
        let use_result = {
            let player = match self.sessions.get(player_id) {
                Some(p) => p,
                None => return,
            };
            use_active_item(&player.hotbar)
        };

        // Process the result
        match use_result {
            UseResult::NoItem => {
                // No item - player uses default melee (already handled by attack system)
                debug!(player_id = player_id.0, "No item equipped - default melee");
            }
            UseResult::WeaponAttack { item_id, damage: _ } => {
                // Weapon attack - route to appropriate weapon system
                self.handle_weapon_attack(player_id, item_id).await;
            }
            UseResult::ConsumableUsed {
                item_id,
                heal_amount,
                stack_depleted,
            } => {
                // Apply healing to player
                if let Some(player) = self.sessions.get_mut(player_id) {
                    let old_health = player.health;
                    player.health = (player.health as u16 + heal_amount).min(100) as u8;
                    let actual_heal = player.health - old_health;

                    // Decrement the consumable stack
                    decrement_active_consumable(&mut player.hotbar);

                    debug!(
                        player_id = player_id.0,
                        item_id = item_id.0,
                        heal = actual_heal,
                        new_health = player.health,
                        stack_depleted = stack_depleted,
                        "Consumable used"
                    );

                    // Send ItemUsed event (T054)
                    let event = GameEvent::ItemUsed {
                        player_id,
                        item_id,
                        slot: player.hotbar.active_slot(),
                    };
                    self.broadcast_event(event).await;
                }
            }
            UseResult::ToolUsed { item_id } => {
                // Tool usage - block placement is handled separately via BlockEdit
                // This just acknowledges the tool is equipped
                debug!(
                    player_id = player_id.0,
                    item_id = item_id.0,
                    "Tool used (block placement via BlockEdit)"
                );
            }
            UseResult::CannotUse => {
                debug!(player_id = player_id.0, "Cannot use item");
            }
        }
    }

    /// Handle crafting request from a player (T035-T037)
    async fn handle_craft_request(&mut self, from: SocketAddr, recipe_id: String) {
        // Only allow crafting during Playing phase
        if self.match_state.phase() != MatchPhase::Playing {
            return;
        }

        // Get crafting config for current game mode
        let craft_config = get_craft_config(self.match_state.game_mode());
        let crafting_enabled = craft_config.enabled;
        let cooldown_ticks = craft_config.cooldown_ticks;

        // Get player, check rate limit, and check cooldown
        let (player_id, is_alive, on_cooldown) = match self.sessions.get_by_addr_mut(&from) {
            Some(player) => {
                // Check rate limit
                if !player.anti_cheat.check_rate_limit(
                    ActionType::InventoryUse,
                    self.current_tick,
                    &self.anti_cheat_config,
                ) {
                    debug!(
                        player_id = player.id.0,
                        "Craft request rejected: rate limit exceeded"
                    );
                    return;
                }
                let on_cooldown = player
                    .craft_cooldown
                    .is_on_cooldown(self.current_tick, cooldown_ticks);
                (player.id, !player.is_dead, on_cooldown)
            }
            None => return,
        };

        // If on cooldown, reject early with CooldownActive reason
        if on_cooldown {
            let event = GameEvent::CraftResult {
                success: false,
                recipe_id: recipe_id.clone(),
                output_item: None,
                output_quantity: None,
                fail_reason: Some(CraftRejectReason::CooldownActive),
            };
            if let Err(e) = self.send_message(from, ServerMessage::Event(event)).await {
                debug!(
                    player_id = player_id.0,
                    error = ?e,
                    "Failed to send CraftResult (cooldown) event"
                );
            }
            debug!(
                player_id = player_id.0,
                recipe = %recipe_id,
                "Craft rejected: cooldown active"
            );
            return;
        }

        // Perform the craft operation
        let craft_system = CraftSystem::new();
        let craft_result = {
            let player = match self.sessions.get_mut(player_id) {
                Some(p) => p,
                None => return,
            };
            craft_system.try_craft(&recipe_id, &mut player.hotbar, is_alive, crafting_enabled)
        };

        // Send CraftResult event to the requesting player (T036)
        let event = GameEvent::CraftResult {
            success: craft_result.success,
            recipe_id: craft_result.recipe_id.clone(),
            output_item: craft_result.output_item,
            output_quantity: craft_result.output_quantity,
            fail_reason: craft_result.fail_reason.map(CraftRejectReason::from),
        };
        if let Err(e) = self.send_message(from, ServerMessage::Event(event)).await {
            debug!(
                player_id = player_id.0,
                error = ?e,
                "Failed to send CraftResult event"
            );
        }

        // If successful, record cooldown, update metrics, and send inventory update (T037)
        if craft_result.success {
            // Record cooldown for successful craft
            if let Some(player) = self.sessions.get_mut(player_id) {
                player.craft_cooldown.record_craft(self.current_tick);
            }

            // Record success metric (T078)
            self.craft_metrics.record_success(&craft_result.recipe_id);

            let inventory_snapshot = {
                let player = match self.sessions.get(player_id) {
                    Some(p) => p,
                    None => return,
                };
                self.create_inventory_snapshot(&player.hotbar, player.hotbar.active_slot())
            };

            let msg = ServerMessage::InventoryUpdate(inventory_snapshot);
            if let Err(e) = self.send_message(from, msg).await {
                debug!(
                    player_id = player_id.0,
                    error = ?e,
                    "Failed to send InventoryUpdate after craft"
                );
            }

            // Log success at info level (T080)
            info!(
                player_id = player_id.0,
                recipe = %craft_result.recipe_id,
                output_item = ?craft_result.output_item,
                output_quantity = ?craft_result.output_quantity,
                "Craft successful"
            );
        } else {
            // Record failure metric (T079)
            let reason_str = craft_result
                .fail_reason
                .map(|r| format!("{:?}", r))
                .unwrap_or_else(|| "Unknown".to_string());
            self.craft_metrics
                .record_failure(&craft_result.recipe_id, &reason_str);

            // Log failure at debug level (T081)
            debug!(
                player_id = player_id.0,
                recipe = %craft_result.recipe_id,
                fail_reason = ?craft_result.fail_reason,
                "Craft failed"
            );
        }
    }

    // ========================================================================
    // Economy Message Handlers
    // ========================================================================

    /// Handle buy request from a player
    async fn handle_buy_request(&mut self, from: SocketAddr, offer_id: String) {
        // Only allow purchases during Playing phase
        if self.match_state.phase() != MatchPhase::Playing {
            return;
        }

        // Get economy config for current game mode
        let economy_config = economy::get_economy_config(self.match_state.game_mode(), None);

        // Get player and validate - extract data then drop borrow
        let (player_id, is_alive, rate_limited) = match self.sessions.get_by_addr_mut(&from) {
            Some(player) => {
                // Check rate limit
                let rate_limited = !player.anti_cheat.check_rate_limit(
                    ActionType::ShopBuy,
                    self.current_tick,
                    &self.anti_cheat_config,
                );
                (player.id, !player.is_dead, rate_limited)
            }
            None => return,
        };

        // Handle rate limit rejection after dropping borrow
        if rate_limited {
            debug!(
                player_id = player_id.0,
                "Buy request rejected: rate limited"
            );
            // Send rate limited response
            let event = GameEvent::PurchaseResult {
                success: false,
                offer_id: offer_id.clone(),
                output_item: None,
                output_quantity: None,
                fail_reason: Some(plix_common::economy::PurchaseRejectReason::RateLimited),
            };
            if let Err(e) = self.send_message(from, ServerMessage::Event(event)).await {
                debug!(player_id = player_id.0, error = ?e, "Failed to send rate limited response");
            }
            self.economy_metrics
                .record_failure(plix_common::economy::PurchaseRejectReason::RateLimited);
            return;
        }

        // Perform the purchase
        let purchase_result = {
            let player = match self.sessions.get_mut(player_id) {
                Some(p) => p,
                None => return,
            };
            economy::try_purchase(
                &offer_id,
                &mut player.wallet,
                &mut player.hotbar,
                &self.shop_registry,
                &economy_config,
                is_alive,
                player_id.0,
            )
        };

        // Send PurchaseResult event
        let event = GameEvent::PurchaseResult {
            success: purchase_result.success,
            offer_id: purchase_result.offer_id.clone(),
            output_item: purchase_result.item_id,
            output_quantity: purchase_result.quantity,
            fail_reason: purchase_result.fail_reason,
        };
        if let Err(e) = self.send_message(from, ServerMessage::Event(event)).await {
            debug!(player_id = player_id.0, error = ?e, "Failed to send PurchaseResult");
        }

        // If successful, send BalanceUpdate and InventoryUpdate
        if purchase_result.success {
            // Send BalanceUpdate
            let balance_event = GameEvent::BalanceUpdate {
                balance: purchase_result.new_balance,
            };
            if let Err(e) = self
                .send_message(from, ServerMessage::Event(balance_event))
                .await
            {
                debug!(player_id = player_id.0, error = ?e, "Failed to send BalanceUpdate");
            }

            // Send InventoryUpdate
            let inventory_snapshot = {
                let player = match self.sessions.get(player_id) {
                    Some(p) => p,
                    None => return,
                };
                self.create_inventory_snapshot(&player.hotbar, player.hotbar.active_slot())
            };
            if let Err(e) = self
                .send_message(from, ServerMessage::InventoryUpdate(inventory_snapshot))
                .await
            {
                debug!(player_id = player_id.0, error = ?e, "Failed to send InventoryUpdate after purchase");
            }

            // Record metrics
            self.economy_metrics.record_purchase(
                self.shop_registry
                    .get(&purchase_result.offer_id)
                    .map(|o| o.price)
                    .unwrap_or(0),
            );
        } else if let Some(reason) = purchase_result.fail_reason {
            self.economy_metrics.record_failure(reason);
        }
    }

    /// Handle balance request from a player
    async fn handle_balance_request(&mut self, from: SocketAddr) {
        let balance = match self.sessions.get_by_addr(&from) {
            Some(player) => player.wallet.get_balance(),
            None => return,
        };

        let event = GameEvent::BalanceUpdate { balance };
        if let Err(e) = self.send_message(from, ServerMessage::Event(event)).await {
            debug!(error = ?e, "Failed to send BalanceUpdate");
        }
    }

    /// Handle shop list request from a player
    async fn handle_shop_list_request(&mut self, from: SocketAddr) {
        let offers: Vec<plix_common::protocol::ShopOfferInfo> = self
            .shop_registry
            .list_for_mode(self.match_state.game_mode())
            .into_iter()
            .map(|o| plix_common::protocol::ShopOfferInfo {
                offer_id: o.offer_id.clone(),
                item_id: o.item_id,
                quantity: o.quantity,
                price: o.price,
            })
            .collect();

        let event = GameEvent::ShopList { offers };
        if let Err(e) = self.send_message(from, ServerMessage::Event(event)).await {
            debug!(error = ?e, "Failed to send ShopList");
        }
    }

    /// Award kill reward to a player (T021)
    ///
    /// Called when a player kills another player. Awards coins based on economy config
    /// and sends a BalanceUpdate event to the killer.
    async fn award_kill_reward(&mut self, killer_id: PlayerId) {
        // Get economy config for current game mode
        let economy_config = economy::get_economy_config(self.match_state.game_mode(), None);

        // Award coins and get new balance
        let new_balance = {
            let player = match self.sessions.get_mut(killer_id) {
                Some(p) => p,
                None => return,
            };
            match economy::award_coins(
                economy::EarningEvent::Kill,
                &mut player.wallet,
                &economy_config,
            ) {
                Some(balance) => balance,
                None => return, // Economy disabled or zero reward
            }
        };

        // Get player address for event delivery
        let player_addr = match self.sessions.get(killer_id) {
            Some(p) => p.addr,
            None => return,
        };

        // Send BalanceUpdate to killer
        let event = GameEvent::BalanceUpdate {
            balance: new_balance,
        };
        if let Err(e) = self
            .send_message(player_addr, ServerMessage::Event(event))
            .await
        {
            debug!(player_id = killer_id.0, error = ?e, "Failed to send kill reward BalanceUpdate");
        }

        // Record metrics
        self.economy_metrics
            .record_earning(economy_config.kill_reward);

        debug!(
            player_id = killer_id.0,
            reward = economy_config.kill_reward,
            new_balance = new_balance,
            "Kill reward awarded"
        );
    }

    /// Create an inventory snapshot from a hotbar
    fn create_inventory_snapshot(
        &self,
        hotbar: &plix_common::inventory::Hotbar,
        active_slot: u8,
    ) -> InventorySnapshot {
        let slots = hotbar
            .slots()
            .iter()
            .map(|slot| match slot {
                Some(stack) => SlotSnapshot::new(stack.item_id, stack.quantity),
                None => SlotSnapshot::empty(),
            })
            .collect();

        InventorySnapshot { slots, active_slot }
    }

    /// Handle weapon attack based on weapon type
    async fn handle_weapon_attack(&mut self, player_id: PlayerId, item_id: ItemId) {
        // Get weapon definition
        let weapon_def = match get_weapon_def(item_id) {
            Some(def) => def,
            None => {
                debug!(
                    player_id = player_id.0,
                    item_id = item_id.0,
                    "Unknown weapon type"
                );
                return;
            }
        };

        // Get player state
        let (player_pos, player_forward, is_moving) = {
            let player = match self.sessions.get(player_id) {
                Some(p) => p,
                None => return,
            };
            let forward = player.rotation.forward();
            let is_moving = player.velocity.length_squared() > 0.01;
            (player.position, forward, is_moving)
        };

        // Route based on weapon type
        match weapon_def.weapon_type {
            WeaponType::Melee => {
                // Melee attacks are handled by the combat system via the Input attack flag
                // This path is for explicit UseActiveItem calls
                debug!(
                    player_id = player_id.0,
                    item_id = item_id.0,
                    "Melee weapon attack via UseActiveItem"
                );
            }
            WeaponType::Ranged => {
                // Handle ranged weapon (e.g., Bow)
                self.handle_ranged_attack(
                    player_id,
                    player_pos,
                    player_forward,
                    is_moving,
                    weapon_def,
                )
                .await;
            }
        }
    }

    /// Handle ranged weapon attack (spawn projectile)
    async fn handle_ranged_attack(
        &mut self,
        player_id: PlayerId,
        position: Vec3,
        forward: Vec3,
        is_moving: bool,
        weapon_def: &'static crate::weapons::defs::WeaponDef,
    ) {
        // Use weapon damage as a proxy for item_id matching (temporary)
        // TODO: Add item_id to WeaponDef or use a different approach
        let item_id = if weapon_def.damage == 15 {
            ItemId::BOW
        } else {
            ItemId::SWORD
        };

        // Check cooldown
        let is_ready = {
            let player = match self.sessions.get(player_id) {
                Some(p) => p,
                None => return,
            };
            player
                .weapon_state
                .cooldowns
                .is_ready(item_id, self.current_tick)
        };

        if !is_ready {
            let remaining = {
                let player = match self.sessions.get(player_id) {
                    Some(p) => p,
                    None => return,
                };
                player
                    .weapon_state
                    .cooldowns
                    .remaining(item_id, self.current_tick)
            };

            // Send cooldown event to player
            if let Some(player) = self.sessions.get(player_id) {
                let event = GameEvent::WeaponCooldown {
                    item_id,
                    remaining_ticks: remaining,
                };
                let _ = self
                    .send_message(player.addr, ServerMessage::Event(event))
                    .await;
            }
            return;
        }

        // Check projectile limit
        if self.projectile_manager.is_full() {
            // Trigger cooldown anyway per spec
            if let Some(player) = self.sessions.get_mut(player_id) {
                player.weapon_state.cooldowns.trigger(
                    item_id,
                    self.current_tick,
                    weapon_def.cooldown_ticks,
                );
            }

            // Send limit reached event
            if let Some(player) = self.sessions.get(player_id) {
                let event = GameEvent::ProjectileLimitReached {
                    current_count: self.projectile_manager.active_count() as u8,
                    max_count: 128,
                };
                let _ = self
                    .send_message(player.addr, ServerMessage::Event(event))
                    .await;
            }
            debug!(
                player_id = player_id.0,
                "Projectile spawn rejected: limit reached"
            );
            return;
        }

        // Calculate spread
        let spread = {
            let player = match self.sessions.get(player_id) {
                Some(p) => p,
                None => return,
            };
            RangedSystem::calculate_spread(
                weapon_def,
                &player.weapon_state.recoil,
                is_moving,
                self.current_tick,
            )
        };

        // Apply spread to direction
        let direction = RangedSystem::apply_spread(forward, spread, self.current_tick.0);

        // Calculate velocity
        let velocity = RangedSystem::calculate_velocity(direction, weapon_def.projectile_speed);

        // Spawn projectile
        let projectile_id = match self.projectile_manager.spawn(
            player_id,
            position + Vec3::Y * 1.6, // Eye height offset
            velocity,
            weapon_def.damage,
            weapon_def.projectile_ttl_ticks,
            self.current_tick,
        ) {
            Some(id) => id,
            None => {
                debug!(player_id = player_id.0, "Failed to spawn projectile");
                return;
            }
        };

        // Update player weapon state
        if let Some(player) = self.sessions.get_mut(player_id) {
            // Trigger cooldown
            player.weapon_state.cooldowns.trigger(
                item_id,
                self.current_tick,
                weapon_def.cooldown_ticks,
            );

            // Add recoil
            player
                .weapon_state
                .recoil
                .add_shot(weapon_def.recoil_per_shot_deg, self.current_tick);
        }

        // Broadcast projectile spawn event
        self.broadcast_event(GameEvent::ProjectileSpawn {
            id: projectile_id,
            owner: player_id,
            position: position + Vec3::Y * 1.6,
            velocity,
            spawn_tick: self.current_tick,
        })
        .await;

        debug!(
            player_id = player_id.0,
            projectile_id = ?projectile_id,
            spread = spread,
            "Projectile spawned"
        );
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

        // Generate unique session ID (T022)
        let session_id = SessionId::new(self.next_session_id);
        self.next_session_id += 1;

        // Create a temporary PlayerId for name registration
        // We need to get the player_id first to register the name
        let temp_player_id = PlayerId(self.sessions.count() as u16 + 1);

        // Register and validate display name via NameRegistry (T023)
        let display_name = match self.name_registry.register(temp_player_id, &name) {
            Some(assigned_name) => assigned_name,
            None => {
                // Max suffix exhaustion - reject connection
                let msg = ServerMessage::Rejected {
                    reason: "Server full for this name (too many players with similar names)"
                        .to_string(),
                };
                let _ = self.send_message(from, msg).await;
                return;
            }
        };

        // Add player with assigned display name and session ID
        let player_id = match self
            .sessions
            .add_player(display_name.clone(), team, from, session_id)
        {
            Some(id) => id,
            None => {
                // Rollback name registration
                self.name_registry.unregister(temp_player_id);
                let msg = ServerMessage::Rejected {
                    reason: "Failed to create player session".to_string(),
                };
                let _ = self.send_message(from, msg).await;
                return;
            }
        };

        // Update name registry with actual player ID if different
        if player_id != temp_player_id {
            // Need to re-register with correct ID
            self.name_registry.unregister(temp_player_id);
            let _ = self.name_registry.register(player_id, &display_name);
        }

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

        // Structured logging with session_id and display_name (T028)
        info!(
            player_id = player_id.0,
            session_id = session_id.0,
            display_name = %display_name,
            requested_name = %name,
            team = team.0,
            from = %from,
            "Player connected"
        );

        // Send Connected response with display_name and session_id (T024)
        let msg = ServerMessage::Connected {
            player_id,
            tick: self.current_tick,
            tick_rate: self.tick_config.rate,
            arena_data: self.arena_data.clone(),
            display_name: display_name.clone(),
            session_id,
        };
        let _ = self.send_message(from, msg).await;

        // Broadcast player joined event with assigned display name
        let event = GameEvent::PlayerJoined {
            id: player_id,
            name: display_name,
            team,
        };
        self.broadcast_event(event).await;
    }

    /// Handle a disconnect request
    async fn handle_disconnect(&mut self, from: SocketAddr) {
        if let Some(player) = self.sessions.get_by_addr(&from) {
            let player_id = player.id;
            let session_id = player.session_id;
            let name = player.name.clone();

            // Unregister name from registry (T027)
            self.name_registry.unregister(player_id);

            info!(
                player_id = player_id.0,
                session_id = session_id.0,
                name = %name,
                "Player disconnected"
            );

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

                // Sync to modified_world for persistence
                if self.config.persistence_enabled {
                    let chunk_coord = request.target_pos.chunk_pos();
                    let (lx, ly, lz) = request.target_pos.local_pos();

                    // Get or create chunk
                    if self.modified_world.get_chunk(chunk_coord).is_none() {
                        self.modified_world
                            .insert_chunk(chunk_coord, Chunk::new(chunk_coord));
                    }
                    if let Some(chunk) = self.modified_world.get_chunk_mut(chunk_coord) {
                        chunk.set_block(lx, ly, lz, new_block_type);
                    }

                    // Mark chunk dirty for auto-save
                    self.save_scheduler.mark_dirty(chunk_coord);
                }

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

                // Detect physics events at the edited position
                self.block_physics
                    .detect_events_at(request.target_pos, self.movement.arena());
                // Also check block above for gravity cascade (support was removed)
                let above = BlockPos::new(
                    request.target_pos.x,
                    request.target_pos.y + 1,
                    request.target_pos.z,
                );
                self.block_physics
                    .detect_events_at(above, self.movement.arena());

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

    /// Process block physics simulation (gravity, liquids)
    async fn process_block_physics(&mut self) {
        // Run physics tick on the arena world
        let processed = self.block_physics.tick(self.movement.arena_mut());

        // If any blocks moved, broadcast the changes
        // Note: Individual block changes are already captured by the dirty tracking
        // and will be sent in the next snapshot or via BlockEditApplied events
        if processed > 0 {
            debug!(
                processed = processed,
                queue_depth = self.block_physics.queue_depth(),
                "Block physics tick"
            );

            // Sync physics changes to modified_world for persistence
            // Note: This is a simplified approach - ideally we'd track exactly which
            // blocks changed during physics tick, but for now we rely on the arena
            // being the source of truth and modified_world being updated on explicit edits
        }
    }

    /// Check for automatic item pickups for all players (T039)
    async fn check_pickups(&mut self) {
        // Collect player IDs and positions first to avoid borrow conflicts
        let player_data: Vec<(PlayerId, Vec3)> = self
            .sessions
            .iter()
            .filter(|p| !p.is_dead)
            .map(|p| (p.id, p.position))
            .collect();

        for (player_id, player_pos) in player_data {
            // Get mutable access to player's hotbar
            let hotbar = match self.sessions.get_mut(player_id) {
                Some(p) => &mut p.hotbar,
                None => continue,
            };

            // Try to pick up nearest loot
            let result = try_pickup(
                player_pos,
                hotbar,
                &mut self.loot_manager,
                &self.inventory_config,
            );

            // Handle pickup result and send events (T040)
            match result {
                PickupResult::Success { loot_id } => {
                    // Loot was picked up - send LootPickedUp event
                    // Note: item info was consumed when loot_manager.remove() was called
                    // For now, broadcast generic pickup event
                    debug!(
                        player_id = player_id.0,
                        loot_id = loot_id.0,
                        "Player picked up loot"
                    );
                }
                PickupResult::Partial { loot_id, remaining } => {
                    debug!(
                        player_id = player_id.0,
                        loot_id = loot_id.0,
                        remaining = remaining,
                        "Player partially picked up loot"
                    );
                }
                PickupResult::HotbarFull { .. } | PickupResult::NoLootInRange => {
                    // Nothing to do
                }
            }
        }
    }

    /// Tick projectiles - move, check collisions, expire, broadcast events
    async fn tick_projectiles(&mut self) {
        // Build list of valid targets (alive, non-invulnerable players)
        let targets: Vec<ProjectileTarget> = self
            .sessions
            .iter()
            .filter(|p| !p.is_dead && !p.is_invulnerable(self.current_tick))
            .map(|p| ProjectileTarget {
                id: p.id,
                position: p.position,
                radius: 0.4,      // Player capsule radius
                half_height: 0.9, // Player capsule half-height
            })
            .collect();

        // Create world wrapper for block collision
        let world_wrapper = WorldWrapper {
            world: &self.modified_world,
        };

        // Tick projectiles and get events
        let (impacts, despawns) =
            self.projectile_manager
                .tick(&targets, &world_wrapper, self.current_tick);

        // Get respawn delay for potential deaths
        let respawn_delay = if let Some(ref training) = self.training_coordinator {
            training.player_respawn_delay_ticks()
        } else {
            self.match_config.respawn_delay_ticks
        };

        // Process impacts
        for impact in impacts {
            // Determine impact type for protocol message
            let (protocol_impact_type, target_player) = match impact.impact_type {
                ServerImpactType::Player(player_id) => {
                    (ProjectileImpactType::Player, Some(player_id))
                }
                ServerImpactType::Block(_) => (ProjectileImpactType::Block, None),
            };

            // Broadcast impact event
            self.broadcast_event(GameEvent::ProjectileImpact {
                id: impact.id,
                position: impact.position,
                impact_type: protocol_impact_type,
                target: target_player,
            })
            .await;

            // Apply damage if hit a player
            if let ServerImpactType::Player(target_id) = impact.impact_type {
                // Skip if target is invulnerable (double-check)
                let is_invulnerable = self
                    .sessions
                    .get(target_id)
                    .map(|p| p.is_invulnerable(self.current_tick))
                    .unwrap_or(false);

                if !is_invulnerable {
                    // Apply damage
                    let (new_health, killed) =
                        if let Some(target) = self.sessions.get_mut(target_id) {
                            let respawn_tick = Tick(self.current_tick.0 + respawn_delay);
                            let killed = target.take_damage(impact.damage as u8, respawn_tick);
                            (target.health, killed)
                        } else {
                            continue;
                        };

                    // Update attacker's kill count if killed
                    if killed {
                        if let Some(attacker) = self.sessions.get_mut(impact.owner) {
                            attacker.kills += 1;
                        }
                    }

                    // Get addresses for event delivery
                    let attacker_addr = self.sessions.get(impact.owner).map(|p| p.addr);
                    let target_addr = self.sessions.get(target_id).map(|p| p.addr);

                    // Send HitConfirmed to attacker
                    if let Some(addr) = attacker_addr {
                        let event = GameEvent::HitConfirmed {
                            attacker: impact.owner,
                            target: target_id,
                            damage: impact.damage as u8,
                        };
                        let _ = self.send_message(addr, ServerMessage::Event(event)).await;
                    }

                    // Send DamageTaken to victim
                    if let Some(addr) = target_addr {
                        let event = GameEvent::DamageTaken {
                            victim: target_id,
                            attacker: impact.owner,
                            damage: impact.damage as u8,
                            new_health,
                        };
                        let _ = self.send_message(addr, ServerMessage::Event(event)).await;
                    }

                    // Broadcast PlayerDied if killed
                    if killed {
                        let event = GameEvent::PlayerDied {
                            victim: target_id,
                            killer: Some(impact.owner),
                        };
                        self.broadcast_event(event).await;

                        // Economy: Award kill reward to attacker (T021)
                        self.award_kill_reward(impact.owner).await;

                        // Update deaths and spectate target
                        if let Some(target) = self.sessions.get_mut(target_id) {
                            target.deaths += 1;
                            target.spectate_target = Some(impact.owner);
                        }

                        // Update match scores
                        if let Some(attacker) = self.sessions.get(impact.owner) {
                            self.match_state.update_player_score(
                                impact.owner,
                                &attacker.name,
                                attacker.kills,
                                attacker.deaths,
                            );
                            self.broadcast_event(GameEvent::ScoreUpdate {
                                player_id: impact.owner,
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
                            self.broadcast_event(GameEvent::ScoreUpdate {
                                player_id: target_id,
                                kills: victim.kills,
                                deaths: victim.deaths,
                            })
                            .await;
                        }
                    }
                }
            }
        }

        // Process despawns
        for (projectile_id, reason) in despawns {
            let protocol_reason = match reason {
                crate::weapons::projectiles::ProjectileDespawnReason::Timeout => {
                    ProjectileDespawnReason::Timeout
                }
                crate::weapons::projectiles::ProjectileDespawnReason::LimitPurge => {
                    ProjectileDespawnReason::LimitPurge
                }
                crate::weapons::projectiles::ProjectileDespawnReason::OwnerLeft => {
                    ProjectileDespawnReason::OwnerLeft
                }
            };

            self.broadcast_event(GameEvent::ProjectileDespawn {
                id: projectile_id,
                reason: protocol_reason,
            })
            .await;
        }
    }
}

/// Wrapper to implement WorldCollision for ChunkedWorld
struct WorldWrapper<'a> {
    world: &'a ChunkedWorld,
}

impl<'a> WorldCollision for WorldWrapper<'a> {
    fn is_solid(&self, pos: BlockPos) -> bool {
        self.world.get_block(pos) != BlockType::AIR
    }
}
