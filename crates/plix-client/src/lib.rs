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

pub mod commands;
pub mod config;
pub mod input;
pub mod interpolation;
pub mod net;
pub mod prediction;
pub mod raycast;
pub mod reconciliation;
pub mod render;
pub mod ui;
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

    /// Connect to a server
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
        };
        self.send_message(&msg).await?;

        // Wait for response (with timeout)
        let mut buf = vec![0u8; 1500];
        let timeout = tokio::time::Duration::from_secs(5);

        match tokio::time::timeout(timeout, self.recv(&mut buf)).await {
            Ok(Ok(len)) => {
                let response: ServerMessage = plix_common::protocol::decode(&buf[..len])?;
                self.handle_connect_response(response)?;
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

    /// Handle connection response from server
    fn handle_connect_response(&mut self, response: ServerMessage) -> Result<(), ClientError> {
        match response {
            ServerMessage::Connected {
                player_id,
                tick,
                tick_rate,
                arena_data: _,
            } => {
                info!(
                    player_id = player_id.0,
                    tick = tick.0,
                    tick_rate = tick_rate,
                    "Connected to server"
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
                    "Unexpected response".to_string(),
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
