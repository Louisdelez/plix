//! Server network loop - handles incoming connections and messages

use std::collections::HashMap;
use std::net::SocketAddr;

use plix_common::protocol::{ClientMessage, ServerMessage};
use plix_net::Connection;

/// Server network state
pub struct ServerNetLoop {
    /// Active connections by address
    connections: HashMap<SocketAddr, Connection>,
    /// Pending outgoing messages
    outgoing: Vec<(SocketAddr, ServerMessage)>,
}

impl ServerNetLoop {
    /// Create a new server network loop
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            outgoing: Vec::new(),
        }
    }

    /// Process incoming message from address
    pub fn handle_message(&mut self, from: SocketAddr, msg: ClientMessage) {
        // TODO: Implement message handling
        match msg {
            ClientMessage::Connect {
                protocol_version: _,
                name: _,
            } => {
                // Handle connection request
            }
            ClientMessage::Disconnect => {
                // Handle disconnect
                self.connections.remove(&from);
            }
            ClientMessage::Input(_input) => {
                // Handle player input
            }
            ClientMessage::SnapshotAck { tick: _ } => {
                // Handle snapshot acknowledgment
            }
            ClientMessage::BlockEdit(_request) => {
                // Block edits are handled in main server tick loop
                // This handler exists for exhaustive matching
            }
            ClientMessage::ReadyToggle => {
                // Ready toggle is handled in main server
                // This handler exists for exhaustive matching
            }
        }
    }

    /// Queue a message to send
    pub fn queue_send(&mut self, to: SocketAddr, msg: ServerMessage) {
        self.outgoing.push((to, msg));
    }

    /// Broadcast message to all connections
    pub fn broadcast(&mut self, msg: ServerMessage) {
        for addr in self.connections.keys() {
            self.outgoing.push((*addr, msg.clone()));
        }
    }

    /// Get and clear pending outgoing messages
    pub fn drain_outgoing(&mut self) -> Vec<(SocketAddr, ServerMessage)> {
        std::mem::take(&mut self.outgoing)
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Check if address is connected
    pub fn is_connected(&self, addr: &SocketAddr) -> bool {
        self.connections.contains_key(addr)
    }

    /// Add a new connection
    pub fn add_connection(&mut self, addr: SocketAddr) {
        self.connections.insert(addr, Connection::new(addr));
    }

    /// Remove a connection
    pub fn remove_connection(&mut self, addr: &SocketAddr) {
        self.connections.remove(addr);
    }

    /// Update all connections (check timeouts, etc.)
    pub fn update(&mut self) {
        // Find timed out connections
        let timed_out: Vec<_> = self
            .connections
            .iter()
            .filter(|(_, conn)| conn.is_timed_out())
            .map(|(addr, _)| *addr)
            .collect();

        // Remove timed out connections
        for addr in timed_out {
            self.connections.remove(&addr);
        }
    }
}

impl Default for ServerNetLoop {
    fn default() -> Self {
        Self::new()
    }
}
