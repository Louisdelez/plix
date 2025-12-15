//! Plix Net - UDP transport layer with reliability
//!
//! This crate provides the network transport for Plix:
//! - UDP socket wrapper
//! - Packet framing with sequence/ack
//! - Unreliable, reliable, and ordered reliable channels
//! - Connection management (handshake, keepalive, timeout)
//! - RTT and packet loss metrics

pub mod channel;
pub mod connection;
pub mod metrics;
pub mod packet;
pub mod transport;

pub use connection::{Connection, ConnectionState};
pub use metrics::NetworkMetrics;
pub use packet::{Channel, Packet, PacketHeader};
pub use transport::UdpTransport;
