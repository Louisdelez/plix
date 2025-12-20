//! Plix Client - Game client with prediction and rendering
//!
//! This crate implements the game client:
//! - Input capture and command buffering
//! - Client-side prediction for local player
//! - Server reconciliation on corrections
//! - Remote player interpolation
//! - Voxel rendering with wgpu
//! - HUD overlay
//! - Configuration persistence
//! - Optional CEF-based HTML UI (Feature 030)
//! - Accessibility features (Feature 042)

pub mod accessibility;
pub mod chunk_manager;
pub mod chunk_mesher;
pub mod commands;
pub mod config;
pub mod console;
pub mod input;
pub mod interpolation;
pub mod matchmaking;
pub mod mods;
pub mod net;
pub mod persist;
pub mod prediction;
pub mod profile;
pub mod raycast;
pub mod reconciliation;
pub mod render;
pub mod server_browser;
pub mod ui;
pub mod ui_cef;
pub mod world;

use std::net::SocketAddr;

use thiserror::Error;
use tokio::net::UdpSocket;
use tracing::{info, warn};

use plix_common::protocol::{ClientMessage, ServerMessage, PROTOCOL_VERSION};
use plix_common::time::Tick;
use plix_common::types::PlayerId;

/// Client errors
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(#[from] plix_common::protocol::ProtocolError),

    #[error("Connection rejected: {0}")]
    Rejected(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
}

/// Client connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Player name
    pub name: String,
    /// Server address
    pub server_addr: Option<SocketAddr>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            name: "Player".to_string(),
            server_addr: None,
        }
    }
}

/// Game client state
pub struct Client {
    config: ClientConfig,
    socket: Option<UdpSocket>,
    state: ConnectionState,
    player_id: Option<PlayerId>,
    server_tick: Tick,
    tick_rate: u8,
}

impl Client {
    /// Create a new client
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config,
            socket: None,
            state: ConnectionState::Disconnected,
            player_id: None,
            server_tick: Tick::ZERO,
            tick_rate: 60,
        }
    }

    /// Get connection state
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Get player ID if connected
    pub fn player_id(&self) -> Option<PlayerId> {
        self.player_id
    }

    /// Connect to a server (Feature 037: includes mod handshake)
    pub async fn connect(&mut self, server_addr: SocketAddr) -> Result<(), ClientError> {
        info!(server = %server_addr, "Connecting to server");

        // Bind local socket
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(server_addr).await?;

        info!(local = %socket.local_addr()?, "Socket bound");

        self.socket = Some(socket);
        self.state = ConnectionState::Connecting;

        // Send connect message
        let msg = ClientMessage::Connect {
            protocol_version: PROTOCOL_VERSION,
            name: self.config.name.clone(),
            account_id: None, // v2 placeholder
            auth_token: None, // v2 placeholder
        };
        self.send_message(&msg).await?;

        // Wait for ModSet response (Feature 037)
        let mut buf = vec![0u8; 65535]; // Larger buffer for mod set
        let timeout = tokio::time::Duration::from_secs(5);

        match tokio::time::timeout(timeout, self.recv(&mut buf)).await {
            Ok(Ok(len)) => {
                let response: ServerMessage = plix_common::protocol::decode(&buf[..len])?;
                self.handle_mod_set(response).await?;
            }
            Ok(Err(e)) => {
                self.state = ConnectionState::Failed;
                return Err(e);
            }
            Err(_) => {
                self.state = ConnectionState::Failed;
                return Err(ClientError::ConnectionFailed(
                    "Connection timeout".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Handle ModSet message from server (Feature 037)
    async fn handle_mod_set(&mut self, response: ServerMessage) -> Result<(), ClientError> {
        use plix_common::protocol::messages::{ModSetDescriptor, ModSetResponse};

        match response {
            ServerMessage::ModSet(mod_set) => {
                info!(
                    mod_count = mod_set.mods.len(),
                    total_size = mod_set.total_payload_size,
                    "Received ModSetDescriptor"
                );

                // Build and send ModSetResponse
                // For now, we support sync but have no cached payloads
                let response = ModSetResponse {
                    supports_sync: true,
                    cached_hashes: Vec::new(), // TODO: integrate with PayloadCache
                    engine_version: Some(env!("CARGO_PKG_VERSION").to_string()),
                };

                let msg = ClientMessage::ModSetResponse(response);
                self.send_message(&msg).await?;

                // Wait for JoinDecision
                let mut buf = vec![0u8; 65535];
                let timeout = tokio::time::Duration::from_secs(5);

                match tokio::time::timeout(timeout, self.recv(&mut buf)).await {
                    Ok(Ok(len)) => {
                        let decision: ServerMessage =
                            plix_common::protocol::decode(&buf[..len])?;
                        self.handle_join_decision(decision).await?;
                    }
                    Ok(Err(e)) => {
                        self.state = ConnectionState::Failed;
                        return Err(e);
                    }
                    Err(_) => {
                        self.state = ConnectionState::Failed;
                        return Err(ClientError::ConnectionFailed(
                            "Join decision timeout".to_string(),
                        ));
                    }
                }

                Ok(())
            }
            // Handle legacy servers that send Connected directly (backwards compat)
            ServerMessage::Connected {
                player_id,
                tick,
                tick_rate,
                arena_data: _,
                display_name,
                session_id,
            } => {
                info!(
                    player_id = player_id.0,
                    session_id = session_id.0,
                    display_name = %display_name,
                    tick = tick.0,
                    tick_rate = tick_rate,
                    "Connected to server (legacy, no mod handshake)"
                );
                self.player_id = Some(player_id);
                self.server_tick = tick;
                self.tick_rate = tick_rate;
                self.state = ConnectionState::Connected;
                Ok(())
            }
            ServerMessage::Rejected { reason } => {
                warn!(reason = %reason, "Connection rejected");
                self.state = ConnectionState::Failed;
                Err(ClientError::Rejected(reason))
            }
            _ => {
                self.state = ConnectionState::Failed;
                Err(ClientError::ConnectionFailed(
                    "Unexpected response (expected ModSet)".to_string(),
                ))
            }
        }
    }

    /// Handle JoinDecision from server (Feature 037)
    async fn handle_join_decision(&mut self, response: ServerMessage) -> Result<(), ClientError> {
        use plix_common::protocol::messages::JoinDecision;

        match response {
            ServerMessage::JoinDecision(decision) => {
                if !decision.allowed {
                    let reason = decision
                        .rejection_reason
                        .unwrap_or_else(|| "Mod policy rejection".to_string());
                    warn!(reason = %reason, "Join rejected by mod policy");
                    self.state = ConnectionState::Failed;
                    return Err(ClientError::Rejected(reason));
                }

                info!(
                    mods_to_sync = decision.mods_to_sync.len(),
                    "Join decision: allowed"
                );

                // If sync is needed, we'd handle payload transfer here
                // For now, we just wait for Connected
                if !decision.mods_to_sync.is_empty() {
                    info!("Payload sync required (not yet implemented)");
                    // TODO: implement payload receiver integration
                }

                // Wait for Connected message
                let mut buf = vec![0u8; 65535];
                let timeout = tokio::time::Duration::from_secs(30); // Longer for sync

                match tokio::time::timeout(timeout, self.recv(&mut buf)).await {
                    Ok(Ok(len)) => {
                        let connected: ServerMessage =
                            plix_common::protocol::decode(&buf[..len])?;
                        self.handle_connected(connected)?;
                    }
                    Ok(Err(e)) => {
                        self.state = ConnectionState::Failed;
                        return Err(e);
                    }
                    Err(_) => {
                        self.state = ConnectionState::Failed;
                        return Err(ClientError::ConnectionFailed(
                            "Connected message timeout".to_string(),
                        ));
                    }
                }

                Ok(())
            }
            // Handle Connected directly (server with no mods to sync)
            ServerMessage::Connected {
                player_id,
                tick,
                tick_rate,
                arena_data: _,
                display_name,
                session_id,
            } => {
                info!(
                    player_id = player_id.0,
                    session_id = session_id.0,
                    display_name = %display_name,
                    "Connected to server"
                );
                self.player_id = Some(player_id);
                self.server_tick = tick;
                self.tick_rate = tick_rate;
                self.state = ConnectionState::Connected;
                Ok(())
            }
            ServerMessage::ModSyncFailed { reason, message } => {
                warn!(reason = ?reason, message = %message, "Mod sync failed");
                self.state = ConnectionState::Failed;
                Err(ClientError::ConnectionFailed(format!(
                    "Mod sync failed: {}",
                    message
                )))
            }
            _ => {
                self.state = ConnectionState::Failed;
                Err(ClientError::ConnectionFailed(
                    "Unexpected response (expected JoinDecision or Connected)".to_string(),
                ))
            }
        }
    }

    /// Handle Connected message (final step of connection)
    fn handle_connected(&mut self, response: ServerMessage) -> Result<(), ClientError> {
        match response {
            ServerMessage::Connected {
                player_id,
                tick,
                tick_rate,
                arena_data: _,
                display_name,
                session_id,
            } => {
                info!(
                    player_id = player_id.0,
                    session_id = session_id.0,
                    display_name = %display_name,
                    tick = tick.0,
                    tick_rate = tick_rate,
                    "Connected to server (mod handshake complete)"
                );
                self.player_id = Some(player_id);
                self.server_tick = tick;
                self.tick_rate = tick_rate;
                self.state = ConnectionState::Connected;
                Ok(())
            }
            ServerMessage::ModSyncFailed { reason, message } => {
                warn!(reason = ?reason, message = %message, "Mod sync failed");
                self.state = ConnectionState::Failed;
                Err(ClientError::ConnectionFailed(format!(
                    "Mod sync failed: {}",
                    message
                )))
            }
            ServerMessage::Rejected { reason } => {
                warn!(reason = %reason, "Connection rejected");
                self.state = ConnectionState::Failed;
                Err(ClientError::Rejected(reason))
            }
            _ => {
                self.state = ConnectionState::Failed;
                Err(ClientError::ConnectionFailed(
                    "Unexpected response (expected Connected)".to_string(),
                ))
            }
        }
    }

    /// Send a client message
    async fn send_message(&self, msg: &ClientMessage) -> Result<(), ClientError> {
        if let Some(socket) = &self.socket {
            let data = plix_common::protocol::encode(msg)?;
            socket.send(&data).await?;
        }
        Ok(())
    }

    /// Receive data
    async fn recv(&self, buf: &mut [u8]) -> Result<usize, ClientError> {
        if let Some(socket) = &self.socket {
            let len = socket.recv(buf).await?;
            Ok(len)
        } else {
            Err(ClientError::ConnectionFailed("No socket".to_string()))
        }
    }

    /// Disconnect from server
    pub async fn disconnect(&mut self) {
        if self.state == ConnectionState::Connected {
            let _ = self.send_message(&ClientMessage::Disconnect).await;
        }
        self.socket = None;
        self.state = ConnectionState::Disconnected;
        self.player_id = None;
        info!("Disconnected from server");
    }
}
