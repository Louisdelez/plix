//! Connection state machine for Plix networking
//!
//! Handles the lifecycle of a network connection:
//! - Handshake (3-way)
//! - Keepalive
//! - Timeout detection
//! - Graceful disconnect

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::channel::{OrderedChannel, ReliableChannel, UnreliableChannel};
use crate::metrics::NetworkMetrics;

/// Connection timeout if no packets received
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

/// Keepalive interval
pub const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(5);

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Initial state, not connected
    Disconnected,
    /// Handshake in progress (client sent Connect, waiting for response)
    Connecting,
    /// Fully connected and operational
    Connected,
    /// Disconnecting gracefully
    Disconnecting,
    /// Connection failed or timed out
    Failed,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// A network connection to a remote peer
#[derive(Debug)]
pub struct Connection {
    /// Remote address
    pub addr: SocketAddr,
    /// Current connection state
    pub state: ConnectionState,
    /// Unreliable channel (inputs, snapshots)
    pub unreliable: UnreliableChannel,
    /// Reliable channel (events)
    pub reliable: ReliableChannel,
    /// Ordered reliable channel (connection, critical state)
    pub ordered: OrderedChannel,
    /// Network metrics
    pub metrics: NetworkMetrics,
    /// Last packet received time
    last_recv: Instant,
    /// Last packet sent time
    last_send: Instant,
    /// Connection established time
    connected_at: Option<Instant>,
}

impl Connection {
    /// Create a new connection to the given address
    pub fn new(addr: SocketAddr) -> Self {
        let now = Instant::now();
        Self {
            addr,
            state: ConnectionState::Disconnected,
            unreliable: UnreliableChannel::new(),
            reliable: ReliableChannel::new(),
            ordered: OrderedChannel::new(),
            metrics: NetworkMetrics::new(),
            last_recv: now,
            last_send: now,
            connected_at: None,
        }
    }

    /// Mark connection as connecting (client initiating handshake)
    pub fn start_connecting(&mut self) {
        self.state = ConnectionState::Connecting;
    }

    /// Mark connection as fully connected
    pub fn set_connected(&mut self) {
        self.state = ConnectionState::Connected;
        self.connected_at = Some(Instant::now());
    }

    /// Start graceful disconnect
    pub fn disconnect(&mut self) {
        self.state = ConnectionState::Disconnecting;
    }

    /// Mark connection as failed
    pub fn fail(&mut self) {
        self.state = ConnectionState::Failed;
    }

    /// Update last received time (call when packet received)
    pub fn on_packet_received(&mut self) {
        self.last_recv = Instant::now();
    }

    /// Update last sent time (call when packet sent)
    pub fn on_packet_sent(&mut self) {
        self.last_send = Instant::now();
    }

    /// Check if connection has timed out
    pub fn is_timed_out(&self) -> bool {
        if self.state == ConnectionState::Connected || self.state == ConnectionState::Connecting {
            Instant::now().duration_since(self.last_recv) > CONNECTION_TIMEOUT
        } else {
            false
        }
    }

    /// Check if we need to send a keepalive
    pub fn needs_keepalive(&self) -> bool {
        if self.state == ConnectionState::Connected {
            Instant::now().duration_since(self.last_send) > KEEPALIVE_INTERVAL
        } else {
            false
        }
    }

    /// Get time since connection was established
    pub fn connected_duration(&self) -> Option<Duration> {
        self.connected_at.map(|t| Instant::now().duration_since(t))
    }

    /// Get time since last packet received
    pub fn time_since_recv(&self) -> Duration {
        Instant::now().duration_since(self.last_recv)
    }

    /// Get time since last packet sent
    pub fn time_since_send(&self) -> Duration {
        Instant::now().duration_since(self.last_send)
    }

    /// Check if any reliable channel has failed packets
    pub fn has_failed_packets(&self) -> bool {
        self.reliable.has_failed_packets() || self.ordered.has_failed_packets()
    }

    /// Get total pending reliable packets
    pub fn pending_reliable_count(&self) -> usize {
        self.reliable.pending_count() + self.ordered.pending_count()
    }

    /// Update connection state based on current conditions
    pub fn update(&mut self) {
        // Check for timeout
        if self.is_timed_out() {
            self.state = ConnectionState::Failed;
            return;
        }

        // Check for failed reliable packets
        if self.has_failed_packets() {
            self.state = ConnectionState::Failed;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_lifecycle() {
        let addr = "127.0.0.1:7777".parse().unwrap();
        let mut conn = Connection::new(addr);

        assert_eq!(conn.state, ConnectionState::Disconnected);

        conn.start_connecting();
        assert_eq!(conn.state, ConnectionState::Connecting);

        conn.set_connected();
        assert_eq!(conn.state, ConnectionState::Connected);
        assert!(conn.connected_at.is_some());

        conn.disconnect();
        assert_eq!(conn.state, ConnectionState::Disconnecting);
    }

    #[test]
    fn test_timeout_detection() {
        let addr = "127.0.0.1:7777".parse().unwrap();
        let mut conn = Connection::new(addr);
        conn.set_connected();

        // Just connected, should not be timed out
        assert!(!conn.is_timed_out());

        // Note: We can't easily test actual timeout without sleeping
        // In real tests, we'd mock Instant
    }
}
