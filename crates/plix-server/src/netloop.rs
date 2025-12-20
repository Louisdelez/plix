//! Server network loop - handles incoming connections and messages
//!
//! This module provides security-hardened message handling with:
//! - Packet size validation (MAX_PACKET_BYTES)
//! - Per-connection rate limiting
//! - Strike-based violation tracking
//! - Automatic disconnect on abuse

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use plix_common::limits::MAX_PACKET_BYTES;
use plix_common::protocol::{ClientMessage, ServerMessage};
use plix_net::Connection;

use crate::security::{
    HandshakeTracker, RateLimitedLogger, RateLimitedMessageType, SecurityContext, SecurityMetrics,
    StrikeCategory,
};

/// Result of processing raw bytes from a client
#[derive(Debug)]
pub enum ProcessResult {
    /// Message was processed successfully
    Ok(ClientMessage),
    /// Message was dropped (rate limited, invalid, etc.) but connection continues
    Dropped,
    /// Connection should be terminated
    Disconnect { reason: String },
}

/// Server network state with security integration
pub struct ServerNetLoop {
    /// Active connections by address
    connections: HashMap<SocketAddr, Connection>,
    /// Per-connection security contexts
    security_contexts: HashMap<SocketAddr, SecurityContext>,
    /// Pending outgoing messages
    outgoing: Vec<(SocketAddr, ServerMessage)>,
    /// Global handshake tracker
    handshake_tracker: HandshakeTracker,
    /// Shared security metrics
    metrics: Arc<SecurityMetrics>,
    /// Rate-limited security logger
    logger: RateLimitedLogger,
    /// Next connection ID counter
    next_connection_id: u64,
}

impl ServerNetLoop {
    /// Create a new server network loop
    pub fn new() -> Self {
        Self::with_metrics(Arc::new(SecurityMetrics::new()))
    }

    /// Create with shared metrics
    pub fn with_metrics(metrics: Arc<SecurityMetrics>) -> Self {
        Self {
            connections: HashMap::new(),
            security_contexts: HashMap::new(),
            outgoing: Vec::new(),
            handshake_tracker: HandshakeTracker::new(),
            metrics,
            logger: RateLimitedLogger::new(),
            next_connection_id: 1,
        }
    }

    /// Process raw bytes received from a client.
    ///
    /// This is the primary entry point for incoming data with full security checks:
    /// 1. Packet size validation
    /// 2. Decode with error handling
    /// 3. Rate limiting
    /// 4. Strike accumulation
    ///
    /// Returns ProcessResult indicating whether message was accepted, dropped, or
    /// connection should be terminated.
    pub fn process_raw_bytes(&mut self, from: SocketAddr, bytes: &[u8]) -> ProcessResult {
        // 1. Check packet size BEFORE any allocation
        if bytes.len() > MAX_PACKET_BYTES {
            self.metrics.record_invalid_message();
            if let Some(ctx) = self.security_contexts.get_mut(&from) {
                if ctx.record_size_violation() {
                    let reason = ctx
                        .disconnect_reason()
                        .unwrap_or("size violation")
                        .to_string();
                    self.metrics
                        .record_disconnect(StrikeCategory::SizeViolation);
                    self.logger.warn(
                        "size",
                        &format!("packet too large from {}: {} bytes", from, bytes.len()),
                    );
                    return ProcessResult::Disconnect { reason };
                }
            }
            return ProcessResult::Dropped;
        }

        // 2. Attempt decode
        let msg = match plix_common::protocol::decode::<ClientMessage>(bytes) {
            Ok(msg) => msg,
            Err(e) => {
                self.metrics.record_invalid_message();
                if let Some(ctx) = self.security_contexts.get_mut(&from) {
                    if ctx.record_decode_error() {
                        let reason = ctx
                            .disconnect_reason()
                            .unwrap_or("decode error")
                            .to_string();
                        self.metrics.record_disconnect(StrikeCategory::DecodeError);
                        self.logger.warn(
                            "decode",
                            &format!("decode error from {}: {}", from, e),
                        );
                        return ProcessResult::Disconnect { reason };
                    }
                }
                self.logger.warn("decode", &format!("decode error from {}: {}", from, e));
                return ProcessResult::Dropped;
            }
        };

        // 3. Check rate limit
        if let Some(ctx) = self.security_contexts.get_mut(&from) {
            if !ctx.check_message_rate(&msg) {
                self.metrics.record_rate_limited();
                if ctx.should_disconnect() {
                    let reason = ctx
                        .disconnect_reason()
                        .unwrap_or("rate limit exceeded")
                        .to_string();
                    self.metrics.record_disconnect(StrikeCategory::RateExceeded);
                    self.logger.warn(
                        "rate",
                        &format!("rate limit exceeded from {}", from),
                    );
                    return ProcessResult::Disconnect { reason };
                }
                return ProcessResult::Dropped;
            }
        }

        ProcessResult::Ok(msg)
    }

    /// Process incoming message from address (legacy interface).
    ///
    /// Prefer `process_raw_bytes` for new code as it includes security checks.
    pub fn handle_message(&mut self, from: SocketAddr, msg: ClientMessage) {
        // Apply rate limiting if security context exists
        if let Some(ctx) = self.security_contexts.get_mut(&from) {
            if !ctx.check_message_rate(&msg) {
                self.metrics.record_rate_limited();
                return; // Drop the message
            }
        }

        match msg {
            ClientMessage::Connect {
                protocol_version: _,
                name: _,
                account_id: _,
                auth_token: _,
            } => {
                // Handle connection request
            }
            ClientMessage::Disconnect => {
                // Handle disconnect
                self.connections.remove(&from);
                self.security_contexts.remove(&from);
            }
            ClientMessage::Input(_input) => {
                // Handle player input
            }
            ClientMessage::SnapshotAck { tick: _ } => {
                // Handle snapshot acknowledgment
            }
            ClientMessage::BlockEdit(_request) => {
                // Block edits are handled in main server tick loop
            }
            ClientMessage::ReadyToggle => {
                // Ready toggle is handled in main server
            }
            ClientMessage::TrainingReset => {
                // Training reset is handled in main server
            }
            ClientMessage::TrainingStatsRequest => {
                // Training stats request is handled in main server
            }
            ClientMessage::SelectHotbarSlot { .. } => {
                // Slot selection is handled in main server
            }
            ClientMessage::UseActiveItem => {
                // Item usage is handled in main server
            }
            ClientMessage::CraftRequest { .. } => {
                // Craft requests are handled in main server
            }
            ClientMessage::BuyRequest { .. } => {
                // Buy requests are handled in main server
            }
            ClientMessage::BalanceRequest => {
                // Balance requests are handled in main server
            }
            ClientMessage::ShopListRequest => {
                // Shop list requests are handled in main server
            }
            ClientMessage::RenameRequest { .. } => {
                // Rename requests are handled in main server
            }
            ClientMessage::ChatSend { .. } => {
                // Chat messages are handled in main server
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

    /// Add a new connection with security context
    pub fn add_connection(&mut self, addr: SocketAddr) -> Result<u64, &'static str> {
        // Check handshake tracker limits
        let connection_id = self.next_connection_id;
        if let Err(e) = self.handshake_tracker.register(connection_id, addr) {
            self.logger.warn(
                "handshake",
                &format!("connection rejected from {}: {}", addr, e),
            );
            return Err("too many pending connections");
        }

        self.connections.insert(addr, Connection::new(addr));
        self.security_contexts
            .insert(addr, SecurityContext::new(connection_id));
        self.next_connection_id = self.next_connection_id.wrapping_add(1);
        if self.next_connection_id == 0 {
            self.next_connection_id = 1;
        }

        Ok(connection_id)
    }

    /// Remove a connection and cleanup security state
    pub fn remove_connection(&mut self, addr: &SocketAddr) {
        if let Some(ctx) = self.security_contexts.remove(addr) {
            self.handshake_tracker.abort(ctx.connection_id());
        }
        self.connections.remove(addr);
    }

    /// Complete handshake for a connection
    pub fn complete_handshake(&mut self, addr: &SocketAddr) {
        if let Some(ctx) = self.security_contexts.get(addr) {
            self.handshake_tracker.complete(ctx.connection_id());
        }
    }

    /// Record a security violation for a connection
    pub fn record_violation(
        &mut self,
        addr: &SocketAddr,
        category: StrikeCategory,
    ) -> Option<String> {
        if let Some(ctx) = self.security_contexts.get_mut(addr) {
            if ctx.record_strike(category) {
                self.metrics.record_disconnect(category);
                return ctx.disconnect_reason().map(|s| s.to_string());
            }
        }
        None
    }

    /// Get security context for an address
    pub fn get_security_context(&self, addr: &SocketAddr) -> Option<&SecurityContext> {
        self.security_contexts.get(addr)
    }

    /// Get mutable security context for an address
    pub fn get_security_context_mut(&mut self, addr: &SocketAddr) -> Option<&mut SecurityContext> {
        self.security_contexts.get_mut(addr)
    }

    /// Get reference to shared metrics
    pub fn metrics(&self) -> &Arc<SecurityMetrics> {
        &self.metrics
    }

    /// Get reference to handshake tracker
    pub fn handshake_tracker(&self) -> &HandshakeTracker {
        &self.handshake_tracker
    }

    /// Get mutable reference to handshake tracker
    pub fn handshake_tracker_mut(&mut self) -> &mut HandshakeTracker {
        &mut self.handshake_tracker
    }

    /// Update all connections (check timeouts, decay strikes, etc.)
    pub fn update(&mut self) {
        // Check handshake timeouts
        let timed_out_handshakes = self.handshake_tracker.check_timeouts();
        for _conn_id in timed_out_handshakes {
            self.metrics.record_handshake_timeout();
        }

        // Find timed out connections
        let timed_out: Vec<_> = self
            .connections
            .iter()
            .filter(|(_, conn)| conn.is_timed_out())
            .map(|(addr, _)| *addr)
            .collect();

        // Remove timed out connections
        for addr in timed_out {
            self.remove_connection(&addr);
        }

        // Decay strikes for all connections
        for ctx in self.security_contexts.values_mut() {
            ctx.decay();
        }
    }
}

impl Default for ServerNetLoop {
    fn default() -> Self {
        Self::new()
    }
}
