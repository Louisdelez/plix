//! Plix Master Server library.
//!
//! This crate provides the core functionality for the master server:
//! - `ServerRegistry`: In-memory storage for server entries with TTL expiration
//! - `api`: HTTP routes for heartbeat and server listing
//! - `validation`: Field validation for server registration
//! - `rate_limit`: IP-based rate limiting

mod api;
mod rate_limit;
mod state;
mod types;
mod validation;

pub use api::create_app;
pub use state::ServerRegistry;
pub use types::{HeartbeatRequest, HeartbeatResponse};
