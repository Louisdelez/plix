//! Master server announcement module.
//!
//! Provides functionality for game servers to register themselves with a master server
//! through periodic heartbeat requests.

pub mod config;
pub mod heartbeat;

pub use config::MasterConfig;
pub use heartbeat::HeartbeatTask;
