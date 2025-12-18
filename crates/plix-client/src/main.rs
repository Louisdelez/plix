//! Plix Client - Entry point
//!
//! Usage: plix-client [OPTIONS]

use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use clap::Parser;
use tracing::{debug, error, info, warn};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use plix_arena::load_arena;
use plix_client::config::{
    load_config, save_config, Action, GameConfig, Key as ConfigKey, FOV_MAX, FOV_MIN, FOV_STEP,
    SENSITIVITY_MAX, SENSITIVITY_MIN, SENSITIVITY_STEP,
};
use plix_client::input::{InputManager, Key};
use plix_client::interpolation::InterpolationManager;
use plix_client::profile::{load_profile, save_profile, PlayerProfile};
use plix_client::raycast::{raycast_blocks, MAX_RAYCAST_DISTANCE};
use plix_client::render::{Camera, PlayerInstance, RenderEngine, UIQuad};
use plix_client::server_browser::BrowserState;
use plix_client::ui::hud::{CombatEvent, Hud};
use plix_client::ui::net_debug::{NetDebugData, NetDebugOverlay};
use plix_client::ui::state::UiState;
use plix_client::ui::{
    Crosshair, KeybindsMenu, KeybindsMenuItem, PauseMenu, PauseMenuItem, ServerBrowserMenu,
    SettingsMenu, SettingsMenuItem,
};
use plix_client::ui_cef::{CefConfig, CefShell};
use plix_client::world::ClientWorld;
use plix_common::math::Vec3;
use plix_common::protocol::{
    BlockEditKind, BlockEditRequest, ClientMessage, GameEvent, ServerMessage, WorldSnapshot,
    PROTOCOL_VERSION,
};
use plix_common::time::Tick;
use plix_common::types::BlockType;
use plix_common::types::PlayerId;

/// Plix game client
#[derive(Parser, Debug)]
#[command(name = "plix-client")]
#[command(about = "Multiplayer game client for Plix")]
#[command(version)]
struct Args {
    /// Server address to connect to (e.g., "127.0.0.1:7777")
    #[arg(long, default_value = "127.0.0.1:7777")]
    server: String,

    /// Player name
    #[arg(long, default_value = "Player")]
    name: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Headless mode (no window, connect and exit)
    #[arg(long)]
    headless: bool,

    /// Master server URL for server browser (e.g., "http://localhost:8080")
    #[arg(long, default_value = "http://localhost:8080")]
    master_url: String,

    // === CEF UI Shell flags (Feature 030) ===
    /// Enable CEF-based HTML UI (overrides config)
    #[arg(long, conflicts_with = "no_cef_ui")]
    cef_ui: bool,

    /// Disable CEF-based HTML UI, use native UI (overrides config)
    #[arg(long, conflicts_with = "cef_ui")]
    no_cef_ui: bool,

    /// Enable CEF DevTools for debugging HTML/CSS/JS
    #[arg(long)]
    cef_devtools: bool,
}

/// Network connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

/// Game state
struct GameState {
    window: Arc<Window>,
    engine: RenderEngine,
    camera: Camera,
    input: InputManager,
    player_position: Vec3,
    last_frame: Instant,
    frame_count: u32,
    fps: f32,
    last_fps_update: Instant,
    cursor_grabbed: bool,
    arena_loaded: bool,
    // Network state
    socket: Option<UdpSocket>,
    connection_state: ConnectionState,
    player_id: Option<PlayerId>,
    server_tick: Tick,
    tick_rate: u8,
    last_input_send: Instant,
    rtt_ms: u32,
    last_snapshot_time: Option<Instant>,
    // Interpolation
    interpolation: InterpolationManager,
    // Match state for HUD
    match_phase: String,
    // Combat HUD
    hud: Hud,
    // Local player health
    local_health: u8,
    // Client-side world state for block editing (T033)
    world: Option<ClientWorld>,
    // UI state (T011)
    ui_state: UiState,
    // Game configuration (T028)
    config: GameConfig,
    // T037: Crosshair display
    crosshair: Crosshair,
    // T044: Pause menu state
    pause_menu: PauseMenu,
    // Settings menu state
    settings_menu: SettingsMenu,
    // T082: Keybinds menu state
    keybinds_menu: KeybindsMenu,
    // Flag for quit request (T051)
    quit_requested: bool,
    // T045-T047: RTT nonce tracking for accurate RTT measurement
    rtt_nonces: HashMap<u64, Instant>,
    next_rtt_nonce: u64,
    rtt_samples: Vec<Duration>,
    // T040-T050: Network debug overlay
    net_debug_overlay: NetDebugOverlay,
    // T049: Server browser state
    browser_state: BrowserState,
    browser_menu: ServerBrowserMenu,
    // Cached server display names for browser menu
    browser_server_names: Vec<String>,
    // Player name for reconnection
    player_name: String,
    // CEF UI Shell (Feature 030)
    cef_shell: CefShell,
}

impl GameState {
    async fn new(
        window: Arc<Window>,
        server_addr: &str,
        player_name: &str,
        master_url: &str,
        cef_ui_flag: Option<bool>, // --cef-ui or --no-cef-ui override
        cef_devtools_flag: bool,   // --cef-devtools flag
    ) -> Self {
        let size = window.inner_size();
        let mut engine = RenderEngine::new(window.clone()).await;

        // Try to load the test arena
        let mut arena_loaded = false;
        let mut spawn_position = Vec3::new(0.0, 1.7, 5.0);
        let mut client_world: Option<ClientWorld> = None;

        // Try multiple paths for arena file
        let arena_paths = [
            PathBuf::from("assets/arenas/test_arena.toml"),
            PathBuf::from("../assets/arenas/test_arena.toml"),
            PathBuf::from("../../assets/arenas/test_arena.toml"),
        ];

        for path in &arena_paths {
            if path.exists() {
                match load_arena(path) {
                    Ok(arena) => {
                        info!(
                            name = %arena.definition.metadata.name,
                            size = ?arena.size(),
                            "Arena loaded successfully"
                        );

                        // Get first spawn point for camera position
                        if let Some(spawn) = arena.definition.spawn_points.first() {
                            spawn_position = Vec3::new(
                                spawn.position[0],
                                spawn.position[1] + 1.6, // Eye height
                                spawn.position[2],
                            );
                            info!(
                                position = ?spawn_position,
                                "Camera positioned at spawn point"
                            );
                        }

                        // T033/T034 [US1]: Create ClientWorld with chunked storage
                        let world = ClientWorld::new(arena);

                        // T035 [US1]: Load chunked world into render engine
                        engine.load_chunked_world(world.chunked_world());
                        info!(
                            chunk_count = world.chunk_count(),
                            mesh_count = engine.chunk_mesh_count(),
                            "Chunked world loaded"
                        );

                        client_world = Some(world);
                        arena_loaded = true;
                        break;
                    }
                    Err(e) => {
                        warn!(error = %e, path = %path.display(), "Failed to load arena");
                    }
                }
            }
        }

        if !arena_loaded {
            warn!("No arena loaded, using fallback scene");
        }

        // Load game configuration (T029)
        let mut config = load_config();
        info!(
            sensitivity = config.sensitivity,
            fov = config.fov_degrees,
            "Config loaded"
        );

        // T012: Apply CEF CLI flag overrides
        if let Some(cef_enabled) = cef_ui_flag {
            config.ui.enabled = cef_enabled;
        }
        if cef_devtools_flag {
            config.ui.devtools = true;
        }

        // T013: Log CEF availability status
        let cef_shell = CefShell::new(config.ui.clone());
        if cef_shell.should_fallback() {
            info!(
                enabled = config.ui.enabled,
                status = ?cef_shell.status(),
                "CEF UI unavailable, using native UI fallback"
            );
        } else {
            info!(
                enabled = config.ui.enabled,
                devtools = config.ui.devtools,
                status = ?cef_shell.status(),
                "CEF UI shell initialized"
            );
        }

        // Use FOV from config
        let mut camera = Camera::new(config.fov_degrees, size.width as f32 / size.height as f32);
        camera.position = spawn_position;

        // Connect to server
        let (socket, connection_state, player_id, server_tick, tick_rate) =
            Self::connect_to_server(server_addr, player_name);

        // Client uses locally loaded arena; block edits replicate as deltas
        if connection_state == ConnectionState::Connected {
            info!(player_id = ?player_id, "Connected to server");
        }

        Self {
            window,
            engine,
            camera,
            input: InputManager::new(),
            player_position: spawn_position,
            last_frame: Instant::now(),
            frame_count: 0,
            fps: 0.0,
            last_fps_update: Instant::now(),
            cursor_grabbed: false,
            arena_loaded,
            socket,
            connection_state,
            player_id,
            server_tick,
            tick_rate,
            last_input_send: Instant::now(),
            rtt_ms: 0,
            last_snapshot_time: None,
            interpolation: InterpolationManager::new(),
            match_phase: "Connecting".to_string(),
            hud: Hud::new(),
            local_health: 100,
            world: client_world,
            // T011: Initialize UI state
            ui_state: UiState::default(),
            // T028: Store loaded config
            config,
            // T037: Initialize crosshair
            crosshair: Crosshair::new(),
            // T044: Initialize menus
            pause_menu: PauseMenu::new(),
            settings_menu: SettingsMenu::new(),
            // T082: Initialize keybinds menu
            keybinds_menu: KeybindsMenu::new(),
            quit_requested: false,
            // T045-T047: RTT nonce tracking
            rtt_nonces: HashMap::new(),
            next_rtt_nonce: 1, // Start at 1, 0 means no nonce
            rtt_samples: Vec::with_capacity(32),
            // T040-T050: Network debug overlay
            net_debug_overlay: NetDebugOverlay::new(),
            // T049: Server browser state
            browser_state: BrowserState::new(master_url),
            browser_menu: ServerBrowserMenu::new(),
            browser_server_names: Vec::new(),
            player_name: player_name.to_string(),
            // T012/T013: CEF shell initialized earlier with CLI overrides applied
            cef_shell,
        }
    }

    fn connect_to_server(
        server_addr: &str,
        player_name: &str,
    ) -> (
        Option<UdpSocket>,
        ConnectionState,
        Option<PlayerId>,
        Tick,
        u8,
    ) {
        let addr: SocketAddr = match server_addr.parse() {
            Ok(a) => a,
            Err(e) => {
                error!(error = %e, "Invalid server address");
                return (None, ConnectionState::Failed, None, Tick::ZERO, 60);
            }
        };

        // Create and bind socket
        let socket = match UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(e) => {
                error!(error = %e, "Failed to bind socket");
                return (None, ConnectionState::Failed, None, Tick::ZERO, 60);
            }
        };

        if let Err(e) = socket.connect(addr) {
            error!(error = %e, "Failed to connect socket");
            return (None, ConnectionState::Failed, None, Tick::ZERO, 60);
        }

        info!(local = %socket.local_addr().unwrap(), server = %addr, "Socket connected");

        // Send connect message
        let msg = ClientMessage::Connect {
            protocol_version: PROTOCOL_VERSION,
            name: player_name.to_string(),
            account_id: None, // v2 placeholder
            auth_token: None, // v2 placeholder
        };
        let data = match plix_common::protocol::encode(&msg) {
            Ok(d) => d,
            Err(e) => {
                error!(error = %e, "Failed to encode connect message");
                return (None, ConnectionState::Failed, None, Tick::ZERO, 60);
            }
        };
        if let Err(e) = socket.send(&data) {
            error!(error = %e, "Failed to send connect message");
            return (None, ConnectionState::Failed, None, Tick::ZERO, 60);
        }

        // Set timeout for connect response
        socket
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .ok();

        // Wait for response
        let mut buf = vec![0u8; 65536];
        match socket.recv(&mut buf) {
            Ok(len) => {
                match plix_common::protocol::decode::<ServerMessage>(&buf[..len]) {
                    Ok(ServerMessage::Connected {
                        player_id,
                        tick,
                        tick_rate,
                        arena_data: _, // Ignored - client loads arena locally
                        display_name,
                        session_id,
                    }) => {
                        info!(
                            player_id = player_id.0,
                            session_id = session_id.0,
                            display_name = %display_name,
                            tick = tick.0,
                            tick_rate = tick_rate,
                            "Connected to server"
                        );

                        // Set non-blocking for game loop
                        socket.set_nonblocking(true).ok();
                        socket.set_read_timeout(None).ok();

                        (
                            Some(socket),
                            ConnectionState::Connected,
                            Some(player_id),
                            tick,
                            tick_rate,
                        )
                    }
                    Ok(ServerMessage::Rejected { reason }) => {
                        error!(reason = %reason, "Connection rejected");
                        (None, ConnectionState::Failed, None, Tick::ZERO, 60)
                    }
                    Ok(_) => {
                        error!("Unexpected response from server");
                        (None, ConnectionState::Failed, None, Tick::ZERO, 60)
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to decode server response");
                        (None, ConnectionState::Failed, None, Tick::ZERO, 60)
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to receive connect response (timeout?)");
                (None, ConnectionState::Failed, None, Tick::ZERO, 60)
            }
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;

        // Process network messages
        self.process_network();

        // Update HUD combat event indicators
        self.hud.update_events();

        // T013: Only process gameplay input when in InGame state
        if self.connection_state == ConnectionState::Connected
            && self.ui_state.should_process_gameplay_input()
        {
            // Send input to server periodically (60 Hz)
            let input_interval = std::time::Duration::from_millis(16); // ~60 Hz
            if now.duration_since(self.last_input_send) >= input_interval {
                self.send_input();
                self.last_input_send = now;
            }

            // Process block edit input (T035, T047) - checks for LMB/RMB clicks
            self.process_block_edit_input();
        }

        // Update interpolation for remote players
        if let Some(local_id) = self.player_id {
            self.interpolation.update(self.server_tick, self.tick_rate);

            // Convert remote players to render instances (skip dead players)
            let mut player_instances = Vec::new();
            for player in self.interpolation.players() {
                if player.id == local_id {
                    continue; // Skip local player
                }
                if player.is_dead {
                    continue; // Skip dead players (T038)
                }
                player_instances.push(PlayerInstance {
                    position: player.display_position,
                    color: [0.8, 0.2, 0.2], // Red for remote players
                });
            }
            self.engine.set_players(player_instances);
        }

        // FPS counter
        self.frame_count += 1;
        let fps_elapsed = (now - self.last_fps_update).as_secs_f32();
        if fps_elapsed >= 0.5 {
            // Update every 500ms as per spec
            self.fps = self.frame_count as f32 / fps_elapsed;
            self.frame_count = 0;
            self.last_fps_update = now;

            // T047, T050: Update debug overlay data
            self.net_debug_overlay.update(NetDebugData {
                rtt: Duration::from_millis(self.rtt_ms as u64),
                jitter: self.compute_jitter(),
                packet_loss: 0.0, // Client doesn't track packet loss currently
                pending_inputs: self.rtt_nonces.len(),
                bytes_sent_per_sec: 0, // TODO: Track bandwidth
                bytes_recv_per_sec: 0,
                server_tick: self.server_tick.0,
                client_tick: self.server_tick.0, // Client tracks server tick
                tick_offset: 0,
                fps: self.fps,
                player_id: self.player_id.map(|id| id.0).unwrap_or(0),
            });

            // Update window title with HUD info (format from spec)
            // Format: "PLIX | FPS: {fps:.0} | Ping: {rtt}ms | ID: {id} | {phase}"
            let ping_str = if self.connection_state == ConnectionState::Connected {
                format!("{}ms", self.rtt_ms)
            } else {
                "--".to_string()
            };
            let id_str = match self.player_id {
                Some(id) => format!("{}", id.0),
                None => "--".to_string(),
            };
            let status = match self.connection_state {
                ConnectionState::Disconnected => "Disconnected",
                ConnectionState::Connecting => "Connecting...",
                ConnectionState::Connected => &self.match_phase,
                ConnectionState::Failed => "Failed",
            };
            // Include health and combat indicators in title (T042)
            let health_str = format!("HP: {}", self.local_health);
            let hit_indicator = if self.hud.hit_confirmed { " [HIT]" } else { "" };
            let dmg_indicator = if self.hud.damage_taken { " [DMG]" } else { "" };

            // T086, T090: Show rebinding state in title
            let ui_indicator = match &self.ui_state {
                UiState::Rebinding(action) => format!(" [Press key for {:?}...]", action),
                UiState::ConfirmSwap {
                    action,
                    conflicting_action,
                    ..
                } => format!(
                    " [Swap {:?} <-> {:?}? Enter=Yes, ESC=No]",
                    action, conflicting_action
                ),
                UiState::Keybinds => " [Keybinds]".to_string(),
                UiState::Settings => " [Settings]".to_string(),
                UiState::PauseMenu => " [Paused]".to_string(),
                _ => String::new(),
            };

            // T050: Show extended debug info in title when overlay is visible
            let debug_str = if self.net_debug_overlay.is_visible() {
                let jitter = self.compute_jitter();
                format!(
                    " | Jitter: {:.0}ms | Pending: {} | Tick: {}",
                    jitter.as_secs_f64() * 1000.0,
                    self.rtt_nonces.len(),
                    self.server_tick.0
                )
            } else {
                String::new()
            };

            let title = format!(
                "PLIX | FPS: {:.0} | Ping: {} | {} | ID: {} | {}{}{}{}{}",
                self.fps,
                ping_str,
                health_str,
                id_str,
                status,
                hit_indicator,
                dmg_indicator,
                ui_indicator,
                debug_str
            );
            self.window.set_title(&title);
        }

        // Apply input to player position (client-side movement for responsiveness)
        // T013: Only apply movement when gameplay input should be processed
        let rotation = self.input.rotation();
        if self.ui_state.should_process_gameplay_input() {
            let forward = rotation.forward();
            let right = rotation.right();

            // Generate input to get current state
            let input = self.input.generate_input(self.server_tick);

            let move_speed = 5.0 * dt;
            self.player_position.x += forward.x * input.move_forward * move_speed;
            self.player_position.z += forward.z * input.move_forward * move_speed;
            self.player_position.x += right.x * input.move_right * move_speed;
            self.player_position.z += right.z * input.move_right * move_speed;
        }

        // Update camera
        self.camera
            .follow_player(self.player_position, rotation, 0.0);
        self.camera
            .set_aspect(self.engine.size().width, self.engine.size().height);
    }

    fn process_network(&mut self) {
        if self.socket.is_none() {
            return;
        }

        // Collect messages first to avoid borrow issues
        let mut messages = Vec::new();
        let mut buf = vec![0u8; 65536];

        // Get socket reference for receiving
        let socket = self.socket.as_ref().unwrap();
        loop {
            match socket.recv(&mut buf) {
                Ok(len) => {
                    messages.push(buf[..len].to_vec());
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    break; // No more messages
                }
                Err(e) => {
                    warn!(error = %e, "Network receive error");
                    break;
                }
            }
        }

        // Now process collected messages
        for data in messages {
            if let Err(e) = self.handle_server_message(&data) {
                warn!(error = %e, "Failed to handle server message");
            }
        }
    }

    fn handle_server_message(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let msg: ServerMessage = plix_common::protocol::decode(data)?;

        match msg {
            ServerMessage::Snapshot(snapshot) => {
                self.handle_snapshot(snapshot);
            }
            ServerMessage::Kicked { reason } => {
                warn!(reason = %reason, "Kicked from server");
                self.connection_state = ConnectionState::Disconnected;
            }
            ServerMessage::Event(event) => {
                debug!(event = ?event, "Received game event");
                self.handle_game_event(event);
            }
            _ => {}
        }

        Ok(())
    }

    /// Calculate jitter from RTT samples (standard deviation)
    fn compute_jitter(&self) -> Duration {
        if self.rtt_samples.len() < 2 {
            return Duration::ZERO;
        }

        let total: Duration = self.rtt_samples.iter().sum();
        let avg_nanos = total.as_nanos() / self.rtt_samples.len() as u128;

        let variance: u128 = self
            .rtt_samples
            .iter()
            .map(|d| {
                let diff = d.as_nanos() as i128 - avg_nanos as i128;
                (diff * diff) as u128
            })
            .sum::<u128>()
            / self.rtt_samples.len() as u128;

        let std_dev = (variance as f64).sqrt() as u64;
        Duration::from_nanos(std_dev)
    }

    fn handle_snapshot(&mut self, snapshot: WorldSnapshot) {
        // Update server tick
        self.server_tick = snapshot.tick;

        // T047: Calculate RTT from echoed nonce
        if snapshot.rtt_nonce_echo != 0 {
            if let Some(sent_time) = self.rtt_nonces.remove(&snapshot.rtt_nonce_echo) {
                let rtt = sent_time.elapsed();
                self.rtt_samples.push(rtt);

                // Keep only last 16 samples for smoothing
                if self.rtt_samples.len() > 16 {
                    self.rtt_samples.remove(0);
                }

                // Update smoothed RTT (average of samples)
                if !self.rtt_samples.is_empty() {
                    let total: Duration = self.rtt_samples.iter().sum();
                    let avg = total / self.rtt_samples.len() as u32;
                    self.rtt_ms = avg.as_millis() as u32;
                }
            }
        }
        self.last_snapshot_time = Some(Instant::now());

        // Update match phase for HUD
        self.match_phase = format!("{:?}", snapshot.match_state.phase);

        // Process players through interpolation manager
        if let Some(local_id) = self.player_id {
            self.interpolation
                .process_snapshot(snapshot.tick, &snapshot.players, local_id);

            // Update local player position and health from server (server authority)
            for player in &snapshot.players {
                if player.id == local_id {
                    // Blend towards server position (simple reconciliation)
                    let server_pos = player.position;
                    self.player_position = self.player_position.lerp(server_pos, 0.1);
                    // Update local health from snapshot (T024, T041)
                    self.local_health = player.health;
                    break;
                }
            }
        }
    }

    fn handle_game_event(&mut self, event: GameEvent) {
        let local_id = self.player_id;

        match event {
            // T022: Handle HitConfirmed - show hit indicator for attacker
            GameEvent::HitConfirmed {
                attacker, damage, ..
            } => {
                if Some(attacker) == local_id {
                    info!(damage = damage, "Hit confirmed!");
                    self.hud.push_event(CombatEvent::HitConfirmed { damage });
                }
            }
            // T023: Handle DamageTaken - show damage for victim
            GameEvent::DamageTaken {
                victim,
                damage,
                new_health,
                ..
            } => {
                if Some(victim) == local_id {
                    info!(damage = damage, new_health = new_health, "Damage taken!");
                    self.hud
                        .push_event(CombatEvent::DamageTaken { damage, new_health });
                    self.local_health = new_health;
                }
            }
            // T037: Handle PlayerDied - show kill feed
            GameEvent::PlayerDied { victim, killer } => {
                info!(victim = victim.0, killer = ?killer, "Player died");
                if let Some(killer_id) = killer {
                    self.hud.push_event(CombatEvent::Kill {
                        killer: killer_id,
                        victim,
                    });
                }
            }
            // T039: Handle PlayerRespawned - optional notification
            GameEvent::PlayerRespawned { id } => {
                info!(player = id.0, "Player respawned");
                self.hud.push_event(CombatEvent::Respawned { player: id });
            }
            // Other events (join/leave, round/match state)
            GameEvent::PlayerJoined { id, name, team } => {
                info!(id = id.0, name = %name, team = team.0, "Player joined");
            }
            GameEvent::PlayerLeft { id } => {
                info!(id = id.0, "Player left");
                self.interpolation.remove_player(id);
            }
            // T036: Handle BlockEditApplied - update ClientWorld and trigger mesh rebuild
            GameEvent::BlockEditApplied(edit) => {
                info!(pos = ?edit.pos, new_block = ?edit.new_block, "Block edit applied");
                if let Some(world) = &mut self.world {
                    world.apply_edit(&edit);
                    // Mark that we need to rebuild the mesh
                    self.rebuild_world_mesh();
                }
                // T058/T059: Push HUD event for debug feedback
                if edit.new_block == BlockType::AIR {
                    self.hud.push_event(CombatEvent::BlockRemoved);
                } else {
                    self.hud.push_event(CombatEvent::BlockPlaced);
                }
            }
            // T036: Handle BlockEditRejected - show debug feedback
            GameEvent::BlockEditRejected(rejection) => {
                info!(pos = ?rejection.pos, reason = ?rejection.reason, "Block edit rejected");
                // T060: Push HUD event for rejected edit
                self.hud.push_event(CombatEvent::BlockEditRejected {
                    reason: format!("{:?}", rejection.reason),
                });
            }
            _ => {}
        }
    }

    /// Rebuild world mesh from current ClientWorld state (T037)
    ///
    /// T035 [US1]: Updated to rebuild only dirty chunks instead of entire mesh.
    fn rebuild_world_mesh(&mut self) {
        if let Some(world) = &mut self.world {
            if world.is_dirty() {
                // Collect dirty chunks
                let dirty: Vec<_> = world.dirty_chunks().iter().copied().collect();

                // Rebuild only affected chunks
                self.engine
                    .rebuild_chunk_meshes(&dirty, world.chunked_world());

                world.clear_dirty();
                debug!(chunks_rebuilt = dirty.len(), "Chunk meshes rebuilt");
            }
        }
    }

    /// Process block edit input (T035, T047)
    fn process_block_edit_input(&mut self) {
        let Some(socket) = &self.socket else {
            return;
        };

        let Some(world) = &self.world else {
            return;
        };

        // Get camera position and direction for raycast
        let eye_pos = self.camera.position;
        let look_dir = self.input.rotation().forward();

        // Check for remove block (LMB) - T035
        if self.input.remove_block_pressed() {
            if let Some(hit) =
                raycast_blocks(world.arena(), eye_pos, look_dir, MAX_RAYCAST_DISTANCE)
            {
                let request = BlockEditRequest {
                    kind: BlockEditKind::Remove,
                    target_pos: hit.block_pos,
                    block_type: BlockType::AIR, // Ignored for remove
                };
                self.send_block_edit_request(socket, request);
            }
        }

        // Check for place block (RMB) - T047
        if self.input.place_block_pressed() {
            if let Some(hit) =
                raycast_blocks(world.arena(), eye_pos, look_dir, MAX_RAYCAST_DISTANCE)
            {
                let adjacent_pos = hit.adjacent_pos();
                // Check adjacent position is in bounds
                if world.is_in_bounds(adjacent_pos) {
                    let request = BlockEditRequest {
                        kind: BlockEditKind::Place,
                        target_pos: adjacent_pos,
                        block_type: BlockType::STONE, // Default block type
                    };
                    self.send_block_edit_request(socket, request);
                }
            }
        }
    }

    /// Send a block edit request to the server
    fn send_block_edit_request(&self, socket: &UdpSocket, request: BlockEditRequest) {
        let msg = ClientMessage::BlockEdit(request.clone());
        match plix_common::protocol::encode(&msg) {
            Ok(data) => {
                if let Err(e) = socket.send(&data) {
                    warn!(error = %e, "Failed to send block edit request");
                } else {
                    debug!(kind = ?request.kind, pos = ?request.target_pos, "Block edit request sent");
                }
            }
            Err(e) => {
                warn!(error = %e, "Failed to encode block edit request");
            }
        }
    }

    fn send_input(&mut self) {
        let Some(socket) = &self.socket else {
            return;
        };

        let mut input = self.input.generate_input(self.server_tick);

        // T045: Generate RTT nonce and store timestamp
        let nonce = self.next_rtt_nonce;
        self.next_rtt_nonce = self.next_rtt_nonce.wrapping_add(1);
        if self.next_rtt_nonce == 0 {
            self.next_rtt_nonce = 1; // Skip 0
        }
        input.rtt_nonce = nonce;

        // T046: Store nonce with timestamp for RTT calculation
        let now = Instant::now();
        self.rtt_nonces.insert(nonce, now);

        // Limit stored nonces to prevent memory growth (keep last 64)
        if self.rtt_nonces.len() > 64 {
            // Remove oldest entries - collect keys to remove first
            let mut entries: Vec<_> = self.rtt_nonces.iter().map(|(&k, &t)| (k, t)).collect();
            entries.sort_by_key(|(_, t)| *t);
            let to_remove: Vec<_> = entries.iter().take(32).map(|(k, _)| *k).collect();
            for nonce in to_remove {
                self.rtt_nonces.remove(&nonce);
            }
        }

        let msg = ClientMessage::Input(input);

        match plix_common::protocol::encode(&msg) {
            Ok(data) => {
                if let Err(e) = socket.send(&data) {
                    warn!(error = %e, "Failed to send input");
                }
            }
            Err(e) => {
                warn!(error = %e, "Failed to encode input");
            }
        }
    }

    fn render(&mut self) {
        self.engine.update_camera(&self.camera);

        // Collect UI quads based on current UI state
        let mut ui_quads: Vec<UIQuad> = Vec::new();
        let size = self.engine.size();

        // T038: Show crosshair only when InGame
        if self.ui_state.should_show_crosshair() {
            ui_quads.extend(self.crosshair.render());
        }

        // T046: Render menus based on UI state
        match &self.ui_state {
            UiState::PauseMenu => {
                ui_quads.extend(
                    self.pause_menu
                        .render(size.width as f32, size.height as f32),
                );
            }
            UiState::Settings => {
                ui_quads.extend(
                    self.settings_menu
                        .render(size.width as f32, size.height as f32),
                );
            }
            UiState::Keybinds => {
                ui_quads.extend(self.keybinds_menu.render(
                    size.width as f32,
                    size.height as f32,
                    &self.config.keybinds,
                ));
            }
            UiState::Rebinding(_) => {
                ui_quads.extend(self.keybinds_menu.render_rebinding(
                    size.width as f32,
                    size.height as f32,
                    &self.config.keybinds,
                ));
            }
            UiState::ConfirmSwap { .. } => {
                ui_quads.extend(self.keybinds_menu.render_confirm_swap(
                    size.width as f32,
                    size.height as f32,
                    &self.config.keybinds,
                ));
            }
            UiState::ServerBrowser | UiState::ServerBrowserRefreshing => {
                ui_quads.extend(self.browser_menu.render(
                    size.width as f32,
                    size.height as f32,
                    &self.browser_server_names,
                ));
            }
            _ => {}
        }

        match self.engine.render_with_ui(&ui_quads) {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => self.engine.resize(self.engine.size()),
            Err(wgpu::SurfaceError::OutOfMemory) => {
                error!("Out of GPU memory");
            }
            Err(e) => {
                error!(error = %e, "Render error");
            }
        }
    }

    fn handle_key(&mut self, key: KeyCode, pressed: bool) -> bool {
        // T051: Check for quit request
        if self.quit_requested {
            return true;
        }

        // T087: Handle key capture in Rebinding state (except ESC which cancels)
        if let UiState::Rebinding(action) = self.ui_state {
            if pressed && key != KeyCode::Escape {
                if let Some(config_key) = ConfigKey::from_keycode(key) {
                    self.handle_rebind_key_press(action, config_key);
                }
                return self.quit_requested;
            }
        }

        // T091: Handle Enter/ESC in ConfirmSwap state
        if let UiState::ConfirmSwap {
            action,
            new_key,
            conflicting_action,
        } = self.ui_state
        {
            if pressed {
                match key {
                    KeyCode::Enter => {
                        // T092: Execute swap
                        self.execute_keybind_swap(action, new_key, conflicting_action);
                        return self.quit_requested;
                    }
                    KeyCode::Escape => {
                        // Cancel swap, go back to keybinds menu
                        self.set_ui_state(UiState::Keybinds);
                        return self.quit_requested;
                    }
                    _ => {}
                }
            }
            return self.quit_requested;
        }

        match key {
            // ESC toggles pause menu (T040)
            KeyCode::Escape => {
                if pressed {
                    match self.ui_state {
                        UiState::InGame => {
                            self.pause_menu.reset(); // Reset to Resume on open
                            self.set_ui_state(UiState::PauseMenu);
                        }
                        UiState::PauseMenu => {
                            self.set_ui_state(UiState::InGame);
                        }
                        UiState::Settings => {
                            self.set_ui_state(UiState::PauseMenu);
                        }
                        // T094: ESC in keybinds goes back to settings
                        UiState::Keybinds => {
                            self.set_ui_state(UiState::Settings);
                        }
                        // T094: ESC in rebinding cancels and goes back to keybinds
                        UiState::Rebinding(_) => {
                            self.set_ui_state(UiState::Keybinds);
                        }
                        // T049: ESC in server browser goes back to pause menu
                        UiState::ServerBrowser | UiState::ServerBrowserRefreshing => {
                            self.set_ui_state(UiState::PauseMenu);
                        }
                        // In other states, ESC goes back to pause menu
                        _ => {
                            self.set_ui_state(UiState::PauseMenu);
                        }
                    }
                }
            }
            // T047: Up/Down arrows for menu navigation
            KeyCode::ArrowUp => {
                if pressed {
                    match self.ui_state {
                        UiState::PauseMenu => self.pause_menu.move_up(),
                        UiState::Settings => self.settings_menu.move_up(),
                        UiState::Keybinds => self.keybinds_menu.move_up(),
                        UiState::ServerBrowser => self.browser_menu.move_up(),
                        _ => {}
                    }
                }
            }
            KeyCode::ArrowDown => {
                if pressed {
                    match self.ui_state {
                        UiState::PauseMenu => self.pause_menu.move_down(),
                        UiState::Settings => self.settings_menu.move_down(),
                        UiState::Keybinds => self.keybinds_menu.move_down(),
                        UiState::ServerBrowser => self.browser_menu.move_down(),
                        _ => {}
                    }
                }
            }
            // T048: Enter key to activate selected menu item
            KeyCode::Enter => {
                if pressed {
                    match self.ui_state {
                        UiState::PauseMenu => {
                            self.handle_pause_menu_action();
                        }
                        UiState::Settings => {
                            self.handle_settings_menu_action();
                        }
                        // T085: Enter in keybinds menu to start rebinding
                        UiState::Keybinds => {
                            self.handle_keybinds_menu_action();
                        }
                        // T049: Connect to selected server
                        UiState::ServerBrowser => {
                            self.handle_server_browser_connect();
                        }
                        _ => {}
                    }
                }
            }
            // T049: R key to refresh server list in browser
            KeyCode::KeyR => {
                if pressed && self.ui_state == UiState::ServerBrowser {
                    self.start_server_list_refresh();
                }
            }
            // T059, T066: Left/Right arrows to adjust setting values
            KeyCode::ArrowLeft => {
                if pressed {
                    self.adjust_setting(-1);
                }
            }
            KeyCode::ArrowRight => {
                if pressed {
                    self.adjust_setting(1);
                }
            }
            // T013: Gate gameplay keys on should_process_gameplay_input
            KeyCode::KeyW => {
                if self.ui_state.should_process_gameplay_input() {
                    self.input.set_key(Key::W, pressed);
                }
            }
            KeyCode::KeyS => {
                if self.ui_state.should_process_gameplay_input() {
                    self.input.set_key(Key::S, pressed);
                }
            }
            KeyCode::KeyA => {
                if self.ui_state.should_process_gameplay_input() {
                    self.input.set_key(Key::A, pressed);
                }
            }
            KeyCode::KeyD => {
                if self.ui_state.should_process_gameplay_input() {
                    self.input.set_key(Key::D, pressed);
                }
            }
            KeyCode::Space => {
                if self.ui_state.should_process_gameplay_input() {
                    self.input.set_key(Key::Space, pressed);
                }
            }
            KeyCode::ControlLeft | KeyCode::ControlRight => {
                if self.ui_state.should_process_gameplay_input() {
                    self.input.set_key(Key::Ctrl, pressed);
                }
            }
            // T048: F3 toggles debug overlay
            KeyCode::F3 => {
                if pressed {
                    self.net_debug_overlay.toggle();
                    info!(
                        visible = self.net_debug_overlay.is_visible(),
                        "Debug overlay toggled"
                    );
                }
            }
            _ => {}
        }
        self.quit_requested
    }

    /// T049, T050, T051: Handle pause menu item activation
    fn handle_pause_menu_action(&mut self) {
        match self.pause_menu.selected_item() {
            PauseMenuItem::Resume => {
                // T049: Return to game
                self.set_ui_state(UiState::InGame);
            }
            PauseMenuItem::Servers => {
                // T049: Open server browser
                self.browser_menu.reset();
                self.set_ui_state(UiState::ServerBrowser);
            }
            PauseMenuItem::Settings => {
                // T050: Open settings menu
                self.settings_menu.reset();
                self.set_ui_state(UiState::Settings);
            }
            PauseMenuItem::Quit => {
                // T051: Request clean exit
                info!("Quit requested via pause menu");
                self.quit_requested = true;
            }
        }
    }

    /// Handle settings menu item activation
    fn handle_settings_menu_action(&mut self) {
        match self.settings_menu.selected_item() {
            SettingsMenuItem::Back => {
                self.set_ui_state(UiState::PauseMenu);
            }
            // Sensitivity and FOV are adjusted with Left/Right arrows, Enter does nothing
            SettingsMenuItem::Sensitivity | SettingsMenuItem::Fov => {}
            // T071: Toggle fullscreen on Enter
            SettingsMenuItem::Fullscreen => {
                self.toggle_fullscreen();
            }
            // T095: Toggle audio mute on Enter
            SettingsMenuItem::Audio => {
                self.toggle_audio_mute();
            }
            // T084: Navigate to keybinds menu
            SettingsMenuItem::Keybinds => {
                self.keybinds_menu.reset();
                self.set_ui_state(UiState::Keybinds);
            }
        }
    }

    /// T085: Handle keybinds menu item activation
    fn handle_keybinds_menu_action(&mut self) {
        let selected = self.keybinds_menu.selected_item();

        if selected == KeybindsMenuItem::Back {
            self.set_ui_state(UiState::Settings);
            return;
        }

        // Enter Rebinding state for the selected action
        if let Some(action) = selected.to_action() {
            info!(action = ?action, "Starting rebind for action");
            self.set_ui_state(UiState::Rebinding(action));
        }
    }

    /// T087, T088, T089: Handle key press during rebinding
    fn handle_rebind_key_press(&mut self, action: Action, new_key: ConfigKey) {
        // T088: Check for conflict
        if let Some(conflicting_action) = self.config.keybinds.action_for_key(new_key) {
            if conflicting_action != action {
                // T089: Conflict detected, enter ConfirmSwap state
                info!(
                    action = ?action,
                    new_key = ?new_key,
                    conflicting_action = ?conflicting_action,
                    "Keybind conflict detected"
                );
                self.set_ui_state(UiState::ConfirmSwap {
                    action,
                    new_key,
                    conflicting_action,
                });
                return;
            }
        }

        // No conflict, apply the new binding directly
        self.config.keybinds.set(action, new_key);
        info!(action = ?action, key = ?new_key, "Keybind set");

        // T093: Save config
        if let Err(e) = save_config(&self.config) {
            warn!(error = %e, "Failed to save config");
        }

        // Return to keybinds menu
        self.set_ui_state(UiState::Keybinds);
    }

    /// T092: Execute keybind swap (confirmed by user)
    fn execute_keybind_swap(
        &mut self,
        action: Action,
        new_key: ConfigKey,
        conflicting_action: Action,
    ) {
        // Get the key currently bound to action (to give to conflicting_action)
        let old_key = self.config.keybinds.get(action);

        // Set the new binding
        self.config.keybinds.set(action, new_key);

        // Give the old key to the conflicting action (swap)
        if let Some(old_key) = old_key {
            self.config.keybinds.set(conflicting_action, old_key);
            info!(
                action = ?action,
                new_key = ?new_key,
                conflicting_action = ?conflicting_action,
                old_key = ?old_key,
                "Keybinds swapped"
            );
        }

        // T093: Save config
        if let Err(e) = save_config(&self.config) {
            warn!(error = %e, "Failed to save config");
        }

        // Return to keybinds menu
        self.set_ui_state(UiState::Keybinds);
    }

    /// T049: Start async server list refresh
    fn start_server_list_refresh(&mut self) {
        info!(
            master_url = %self.browser_state.master_url(),
            "Refreshing server list"
        );
        self.browser_menu.set_status("Refreshing...");
        // Note: In a full async implementation, this would spawn a task.
        // For the synchronous game loop, we do a blocking refresh here.
        // In production, you'd want to use channels or Arc<Mutex> for async updates.
        let master_url = self.browser_state.master_url().to_string();
        match plix_client::server_browser::fetch::fetch_servers_blocking(&master_url) {
            Ok(response) => {
                let count = response.servers.len();
                self.browser_server_names = response
                    .servers
                    .iter()
                    .map(|s| {
                        format!(
                            "{} ({}/{}) [{}]",
                            plix_client::server_browser::sanitize_display_string(&s.name, 32),
                            s.player_count,
                            s.max_players,
                            s.region
                        )
                    })
                    .collect();
                self.browser_menu.set_server_count(count);
                self.browser_menu.set_status(format!(
                    "Found {} server(s). R=Refresh, Enter=Connect",
                    count
                ));
                info!(count = count, "Server list refreshed");
            }
            Err(e) => {
                warn!(error = %e, "Failed to refresh server list");
                self.browser_menu
                    .set_status(format!("Error: {}. Press R to retry", e));
                self.browser_server_names.clear();
                self.browser_menu.set_server_count(0);
            }
        }
    }

    /// T049: Handle connect to selected server
    fn handle_server_browser_connect(&mut self) {
        let index = self.browser_menu.selected_index();
        let master_url = self.browser_state.master_url().to_string();

        // Fetch fresh server list to get connection info
        match plix_client::server_browser::fetch::fetch_servers_blocking(&master_url) {
            Ok(response) => {
                if let Some(server) = response.servers.get(index) {
                    let addr = format!("{}:{}", server.host, server.port);
                    info!(
                        server_name = %server.name,
                        address = %addr,
                        "Connecting to server from browser"
                    );

                    // Disconnect from current server if connected
                    if self.connection_state == ConnectionState::Connected {
                        if let Some(socket) = &self.socket {
                            let msg = ClientMessage::Disconnect;
                            if let Ok(data) = plix_common::protocol::encode(&msg) {
                                let _ = socket.send(&data);
                            }
                        }
                    }

                    // Connect to new server
                    let (socket, connection_state, player_id, server_tick, tick_rate) =
                        Self::connect_to_server(&addr, &self.player_name);

                    self.socket = socket;
                    self.connection_state = connection_state;
                    self.player_id = player_id;
                    self.server_tick = server_tick;
                    self.tick_rate = tick_rate;

                    if connection_state == ConnectionState::Connected {
                        self.set_ui_state(UiState::InGame);
                        self.browser_menu
                            .set_status(format!("Connected to {}", server.name));
                    } else {
                        self.browser_menu
                            .set_status(format!("Failed to connect to {}", server.name));
                    }
                } else {
                    self.browser_menu.set_status("Invalid server selection");
                }
            }
            Err(e) => {
                warn!(error = %e, "Failed to fetch server for connection");
                self.browser_menu
                    .set_status(format!("Error: {}. Press R to refresh", e));
            }
        }
    }

    /// T059, T060, T061, T066, T067, T068: Adjust setting value by direction (-1 or +1)
    fn adjust_setting(&mut self, direction: i32) {
        if self.ui_state != UiState::Settings {
            return;
        }

        match self.settings_menu.selected_item() {
            SettingsMenuItem::Sensitivity => {
                // T059: Adjust sensitivity
                let delta = direction as f32 * SENSITIVITY_STEP;
                self.config.sensitivity =
                    (self.config.sensitivity + delta).clamp(SENSITIVITY_MIN, SENSITIVITY_MAX);

                // T060: Apply immediately (sensitivity is read from config in handle_mouse_motion)
                info!(
                    sensitivity = self.config.sensitivity,
                    "Sensitivity adjusted"
                );

                // T061: Save config
                if let Err(e) = save_config(&self.config) {
                    warn!(error = %e, "Failed to save config");
                }
            }
            SettingsMenuItem::Fov => {
                // T066: Adjust FOV
                let delta = direction as f32 * FOV_STEP;
                self.config.fov_degrees = (self.config.fov_degrees + delta).clamp(FOV_MIN, FOV_MAX);

                // T067: Apply immediately to camera
                self.camera.set_fov(self.config.fov_degrees);
                info!(fov = self.config.fov_degrees, "FOV adjusted");

                // T068: Save config
                if let Err(e) = save_config(&self.config) {
                    warn!(error = %e, "Failed to save config");
                }
            }
            _ => {}
        }
    }

    /// T070, T071, T072: Toggle fullscreen mode
    fn toggle_fullscreen(&mut self) {
        self.config.fullscreen = !self.config.fullscreen;
        self.apply_fullscreen();

        info!(fullscreen = self.config.fullscreen, "Fullscreen toggled");

        // T072: Save config
        if let Err(e) = save_config(&self.config) {
            warn!(error = %e, "Failed to save config");
        }
    }

    /// T070: Apply fullscreen setting to window
    fn apply_fullscreen(&self) {
        use winit::window::Fullscreen;

        if self.config.fullscreen {
            // Use borderless fullscreen on primary monitor
            self.window
                .set_fullscreen(Some(Fullscreen::Borderless(None)));
        } else {
            self.window.set_fullscreen(None);
        }
    }

    /// T095, T096, T097: Toggle audio mute
    fn toggle_audio_mute(&mut self) {
        self.config.audio_muted = !self.config.audio_muted;

        info!(muted = self.config.audio_muted, "Audio mute toggled");

        // Save config
        if let Err(e) = save_config(&self.config) {
            warn!(error = %e, "Failed to save config");
        }

        // Note: Actual audio mute would be applied to audio subsystem here
        // For now we just store the preference
    }

    fn handle_mouse_button(&mut self, button: winit::event::MouseButton, pressed: bool) {
        use winit::event::MouseButton;

        // T087: Handle mouse button capture in Rebinding state
        if let UiState::Rebinding(action) = self.ui_state {
            if pressed {
                if let Some(config_key) = ConfigKey::from_mouse_button(button) {
                    self.handle_rebind_key_press(action, config_key);
                }
            }
            return;
        }

        // T013: Only process gameplay mouse buttons when appropriate
        if !self.ui_state.should_process_gameplay_input() {
            return;
        }

        // If not grabbed, clicking grabs the cursor
        if pressed && !self.cursor_grabbed {
            self.grab_cursor();
            return;
        }

        match button {
            MouseButton::Left => self.input.set_key(Key::LeftClick, pressed),
            MouseButton::Right => self.input.set_key(Key::RightClick, pressed),
            _ => {}
        }
    }

    fn handle_mouse_motion(&mut self, dx: f64, dy: f64) {
        // T013: Only process mouse look when in game and cursor grabbed
        if self.cursor_grabbed && self.ui_state.should_process_gameplay_input() {
            // Use sensitivity from config
            let sensitivity = self.config.sensitivity;
            self.input.mouse_move(
                dx as f32 * sensitivity / 0.003,
                dy as f32 * sensitivity / 0.003,
            );
        }
    }

    fn grab_cursor(&mut self) {
        self.cursor_grabbed = true;
        let _ = self
            .window
            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_| {
                self.window
                    .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            });
        self.window.set_cursor_visible(false);
    }

    fn release_cursor(&mut self) {
        self.cursor_grabbed = false;
        let _ = self
            .window
            .set_cursor_grab(winit::window::CursorGrabMode::None);
        self.window.set_cursor_visible(true);
    }

    /// T014: Apply cursor state based on current UiState
    fn apply_cursor_state(&mut self) {
        if self.ui_state.should_grab_cursor() {
            self.grab_cursor();
        } else {
            self.release_cursor();
        }
    }

    /// T015: Set UI state and apply cursor state changes
    fn set_ui_state(&mut self, new_state: UiState) {
        if self.ui_state != new_state {
            debug!(old = ?self.ui_state, new = ?new_state, "UI state changed");
            self.ui_state = new_state;
            self.apply_cursor_state();
        }
    }
}

/// Application handler for winit
struct App {
    args: Args,
    state: Option<GameState>,
}

impl App {
    fn new(args: Args) -> Self {
        Self { args, state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attrs = Window::default_attributes()
            .with_title("Plix - Connecting...")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        // Load player profile from disk (T035)
        let profile = load_profile();
        // Use profile display_name unless args.name was explicitly set (not default "Player")
        let player_name = if self.args.name != "Player" {
            self.args.name.clone()
        } else {
            profile.display_name.clone()
        };
        info!(
            profile_name = %profile.display_name,
            using_name = %player_name,
            "Loaded player profile"
        );

        // T012: Determine CEF UI override from CLI flags
        let cef_ui_flag = if self.args.cef_ui {
            Some(true)
        } else if self.args.no_cef_ui {
            Some(false)
        } else {
            None // Use config default
        };

        // Initialize rendering and network on this thread
        let mut state = pollster::block_on(GameState::new(
            window,
            &self.args.server,
            &player_name,
            &self.args.master_url,
            cef_ui_flag,
            self.args.cef_devtools,
        ));

        // T073: Apply fullscreen setting from config on startup
        state.apply_fullscreen();

        self.state = Some(state);

        info!(
            name = %player_name,
            server = %self.args.server,
            master_url = %self.args.master_url,
            "Plix client started"
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                info!("Window closed");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                state.engine.resize(size);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                if state.handle_key(key, key_state == ElementState::Pressed) {
                    event_loop.exit();
                }
            }
            WindowEvent::MouseInput {
                state: btn_state,
                button,
                ..
            } => {
                state.handle_mouse_button(button, btn_state == ElementState::Pressed);
            }
            WindowEvent::RedrawRequested => {
                state.update();
                state.render();
                state.window.request_redraw();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };

        if let DeviceEvent::MouseMotion { delta } = event {
            state.handle_mouse_motion(delta.0, delta.1);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}

fn main() {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("plix={}", args.log_level).parse().unwrap()),
        )
        .init();

    if args.headless {
        // Headless mode - use tokio runtime for networking only
        info!("Running in headless mode");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(run_headless(args));
    } else {
        // Windowed mode
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = App::new(args);
        event_loop.run_app(&mut app).unwrap();
    }
}

async fn run_headless(args: Args) {
    use plix_client::{Client, ClientConfig};

    // Load profile and determine player name (T035)
    let profile = load_profile();
    let player_name = if args.name != "Player" {
        args.name.clone()
    } else {
        profile.display_name.clone()
    };
    info!(profile_name = %profile.display_name, using_name = %player_name, "Loaded player profile");

    let config = ClientConfig {
        name: player_name.clone(),
        server_addr: args.server.parse().ok(),
    };

    let mut client = Client::new(config);

    match args.server.parse::<SocketAddr>() {
        Ok(addr) => match client.connect(addr).await {
            Ok(()) => {
                info!(
                    player_id = ?client.player_id(),
                    "Successfully connected!"
                );
                info!("Headless mode - disconnecting");
                client.disconnect().await;
            }
            Err(e) => {
                error!(error = %e, "Failed to connect");
            }
        },
        Err(e) => {
            error!(server = %args.server, error = %e, "Invalid server address");
        }
    }

    info!("Client shutdown");
}
