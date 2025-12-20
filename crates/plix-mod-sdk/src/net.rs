//! Network API - send messages between server and clients
//!
//! Mods can send custom messages to clients or broadcast to all players.
//! Messages are sent on named channels with a maximum payload of 8KB.
//! Requires NET_SEND capability.
//!
//! # Example
//!
//! ```rust,ignore
//! use plix_mod_sdk::net::{send, broadcast, MessageTarget};
//!
//! // Send to a specific player
//! send("mod:mymod:ui", MessageTarget::Client(player_id), &payload)?;
//!
//! // Broadcast to all players
//! broadcast("mod:mymod:notification", &message)?;
//! ```
//!
//! # Channel Naming Convention
//!
//! Channels should follow the format `mod:<mod_id>:<name>` to avoid conflicts.
//!
//! # Constraints
//!
//! - Maximum payload size: 8KB
//! - Rate limit: 20 messages/second per mod

use crate::abi::{self, HostCallOp};
use crate::codec::{self, decode, encode, NetBroadcastRequest, NetSendRequest};
use crate::error::{err_invalid, err_rate, Result};
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Maximum network message payload size in bytes
pub const MAX_PAYLOAD_SIZE: usize = 8 * 1024; // 8KB

/// Maximum messages per second per mod
pub const MAX_MESSAGES_PER_SECOND: u32 = 20;

/// Target for a network message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageTarget {
    /// Send to the server (for client-side mods, not applicable for server mods)
    Server,
    /// Send to a specific client by player ID
    Client(u64),
    /// Send to all connected clients
    AllClients,
    /// Send to all clients on a specific team
    Team(u8),
}

impl MessageTarget {
    /// Convert to wire format (type + id)
    fn to_wire(&self) -> (u8, u64) {
        match *self {
            MessageTarget::Server => (0, 0),
            MessageTarget::Client(id) => (1, id),
            MessageTarget::AllClients => (2, 0),
            MessageTarget::Team(id) => (3, id as u64),
        }
    }
}

/// Send a message to a specific target
///
/// Requires NET_SEND capability.
///
/// # Arguments
///
/// * `channel` - Channel name (recommend format: `mod:<mod_id>:<name>`)
/// * `target` - Message target (client, all clients, team)
/// * `payload` - Message payload (max 8KB)
///
/// # Errors
///
/// - `PermissionDenied` if NET_SEND capability not granted
/// - `InvalidArgument` if payload > 8KB or channel is empty
/// - `RateLimited` if sending too many messages (>20/sec)
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::net::{send, MessageTarget};
///
/// // Send UI update to a player
/// let data = serde_json::to_vec(&ui_state)?;
/// send("mod:mymod:ui", MessageTarget::Client(player_id), &data)?;
///
/// // Send to a team
/// send("mod:mymod:team", MessageTarget::Team(1), &message)?;
/// ```
pub fn send(channel: &str, target: MessageTarget, payload: &[u8]) -> Result<()> {
    validate_send(channel, payload)?;

    let (target_type, target_id) = target.to_wire();

    let request = NetSendRequest {
        channel: channel.into(),
        target_type,
        target_id,
        payload: payload.to_vec(),
    };
    let request_bytes = encode(&request)?;

    let response_bytes = abi::host_call(HostCallOp::NetSend, &request_bytes)
        .map_err(|code| crate::error::ModError::from_abi(code as u16, None))?;

    let response: codec::AbiResponse = decode(&response_bytes)?;
    response.into_result()?;

    Ok(())
}

/// Broadcast a message to all connected clients
///
/// Requires NET_SEND capability.
///
/// # Arguments
///
/// * `channel` - Channel name (recommend format: `mod:<mod_id>:<name>`)
/// * `payload` - Message payload (max 8KB)
///
/// # Errors
///
/// - `PermissionDenied` if NET_SEND capability not granted
/// - `InvalidArgument` if payload > 8KB or channel is empty
/// - `RateLimited` if sending too many messages (>20/sec)
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::net::broadcast;
///
/// // Announce to all players
/// let announcement = "Server event starting!";
/// broadcast("mod:mymod:announcements", announcement.as_bytes())?;
/// ```
pub fn broadcast(channel: &str, payload: &[u8]) -> Result<()> {
    validate_send(channel, payload)?;

    let request = NetBroadcastRequest {
        channel: channel.into(),
        payload: payload.to_vec(),
    };
    let request_bytes = encode(&request)?;

    let response_bytes = abi::host_call(HostCallOp::NetBroadcast, &request_bytes)
        .map_err(|code| crate::error::ModError::from_abi(code as u16, None))?;

    let response: codec::AbiResponse = decode(&response_bytes)?;
    response.into_result()?;

    Ok(())
}

/// Validate send parameters
fn validate_send(channel: &str, payload: &[u8]) -> Result<()> {
    if channel.is_empty() {
        return Err(err_invalid("Channel name cannot be empty"));
    }

    if payload.len() > MAX_PAYLOAD_SIZE {
        return Err(err_invalid(alloc::format!(
            "Payload size {} exceeds maximum {}",
            payload.len(),
            MAX_PAYLOAD_SIZE
        )));
    }

    Ok(())
}

/// Helper to create a mod-namespaced channel name
///
/// # Example
///
/// ```rust,ignore
/// let channel = mod_channel("mymod", "ui");
/// // Returns "mod:mymod:ui"
/// ```
pub fn mod_channel(mod_id: &str, name: &str) -> String {
    alloc::format!("mod:{}:{}", mod_id, name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_target_to_wire() {
        assert_eq!(MessageTarget::Server.to_wire(), (0, 0));
        assert_eq!(MessageTarget::Client(123).to_wire(), (1, 123));
        assert_eq!(MessageTarget::AllClients.to_wire(), (2, 0));
        assert_eq!(MessageTarget::Team(2).to_wire(), (3, 2));
    }

    #[test]
    fn test_mod_channel() {
        assert_eq!(mod_channel("mymod", "ui"), "mod:mymod:ui");
        assert_eq!(mod_channel("test", "events"), "mod:test:events");
    }

    #[test]
    fn test_validate_empty_channel() {
        let result = validate_send("", &[1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_payload_too_large() {
        let payload = alloc::vec![0u8; MAX_PAYLOAD_SIZE + 1];
        let result = validate_send("test", &payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_ok() {
        let payload = alloc::vec![0u8; 100];
        let result = validate_send("mod:test:channel", &payload);
        assert!(result.is_ok());
    }
}
