//! Client network handling

use std::net::SocketAddr;

use plix_common::protocol::{ClientMessage, ServerMessage};
use plix_net::{Connection, ConnectionState};

/// Client network state
#[derive(Debug)]
pub struct ClientNet {
    /// Server address
    server_addr: Option<SocketAddr>,
    /// Connection to server
    connection: Option<Connection>,
    /// Outgoing message queue
    outgoing: Vec<ClientMessage>,
}

impl ClientNet {
    /// Create a new client network handler
    pub fn new() -> Self {
        Self {
            server_addr: None,
            connection: None,
            outgoing: Vec::new(),
        }
    }

    /// Start connecting to a server
    pub fn connect(&mut self, addr: SocketAddr, name: String) {
        self.server_addr = Some(addr);
        let mut connection = Connection::new(addr);
        connection.start_connecting();
        self.connection = Some(connection);

        // Queue connect message
        self.outgoing.push(ClientMessage::Connect {
            protocol_version: plix_common::protocol::PROTOCOL_VERSION,
            name,
        });
    }

    /// Disconnect from server
    pub fn disconnect(&mut self) {
        if let Some(conn) = &mut self.connection {
            conn.disconnect();
        }
        self.outgoing.push(ClientMessage::Disconnect);
    }

    /// Queue a message to send
    pub fn send(&mut self, msg: ClientMessage) {
        self.outgoing.push(msg);
    }

    /// Get and clear outgoing messages
    pub fn drain_outgoing(&mut self) -> Vec<ClientMessage> {
        std::mem::take(&mut self.outgoing)
    }

    /// Handle received message
    pub fn handle_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Connected {
                player_id,
                tick,
                tick_rate,
                arena_data,
            } => {
                if let Some(conn) = &mut self.connection {
                    conn.set_connected();
                }
                // TODO: Store player ID, sync tick, load arena
            }
            ServerMessage::Rejected { reason } => {
                if let Some(conn) = &mut self.connection {
                    conn.fail();
                }
                // TODO: Show rejection reason
            }
            ServerMessage::Kicked { reason } => {
                if let Some(conn) = &mut self.connection {
                    conn.fail();
                }
                // TODO: Show kick reason
            }
            ServerMessage::Warning { message, strikes } => {
                // TODO: Display anti-cheat warning to player
                // For now, just log it (client-side logging not yet implemented)
            }
            ServerMessage::Snapshot(snapshot) => {
                // TODO: Process snapshot
            }
            ServerMessage::Event(event) => {
                // TODO: Process event
            }
        }
    }

    /// Get connection state
    pub fn state(&self) -> ConnectionState {
        self.connection
            .as_ref()
            .map(|c| c.state)
            .unwrap_or(ConnectionState::Disconnected)
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.state() == ConnectionState::Connected
    }

    /// Update network state
    pub fn update(&mut self) {
        if let Some(conn) = &mut self.connection {
            conn.update();
        }
    }
}

impl Default for ClientNet {
    fn default() -> Self {
        Self::new()
    }
}
