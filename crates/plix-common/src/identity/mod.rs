//! Identity types for player identification
//!
//! This module provides core identity types used across client and server:
//! - `DisplayName`: Validated player display name (1-32 chars)
//! - `SessionId`: Unique session identifier for logging/metrics
//! - `AccountId`: Placeholder for future authentication (v2)

mod display_name;
mod session;

pub use display_name::{validate_display_name, DisplayName, DisplayNameError};
pub use session::{AccountId, SessionId};
