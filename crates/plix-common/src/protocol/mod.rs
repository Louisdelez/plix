//! Network protocol definitions for Plix
//!
//! This module defines all message types exchanged between client and server.

mod codec;
mod messages;

pub use codec::{decode, encode, ProtocolError};
pub use messages::*;

/// Protocol version (must match between client and server)
pub const PROTOCOL_VERSION: u8 = 0;

/// Maximum packet payload size (MTU 1400 - header overhead)
pub const MAX_PAYLOAD_SIZE: usize = 1389;

/// Maximum player name length
pub const MAX_NAME_LENGTH: usize = 32;
