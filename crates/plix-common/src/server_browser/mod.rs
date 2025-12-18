//! Server browser types shared between master server and clients.
//!
//! This module provides the common data structures for the server directory system:
//! - `ServerEntry`: Represents a game server in the directory
//! - `ServerListResponse`: Response from master server listing active servers

mod types;

pub use types::{ServerEntry, ServerListResponse};
