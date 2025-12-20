//! Event subscription and handling API
//!
//! Mods subscribe to events during initialization and receive callbacks
//! when those events occur. Some events are cancellable (require capability).
//!
//! # Example
//!
//! ```rust,ignore
//! use plix_mod_sdk::events::{subscribe, EventType, EventContext, PlayerChatPayload};
//!
//! fn init() {
//!     subscribe(EventType::PlayerChat).unwrap();
//! }
//!
//! fn on_player_chat(ctx: &EventContext, payload: PlayerChatPayload) {
//!     if payload.text.contains("spam") {
//!         ctx.cancel().unwrap(); // Requires EVENT_CANCEL_CHAT capability
//!     }
//! }
//! ```

use crate::abi;
use crate::error::{err_invalid, err_perm, ErrorCode, ModError, Result};
use alloc::string::String;
use alloc::vec::Vec;
use glam::IVec3;
use serde::{Deserialize, Serialize};

/// Event types that mods can subscribe to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EventType {
    /// Server started
    ServerStart = 0x01,
    /// Server stopping
    ServerStop = 0x02,
    /// Player joined the game
    PlayerJoin = 0x03,
    /// Player left the game
    PlayerLeave = 0x04,
    /// Player sent a chat message (cancellable)
    PlayerChat = 0x05,
    /// Block was placed (cancellable)
    BlockPlaced = 0x06,
    /// Block was broken (cancellable)
    BlockBroken = 0x07,
    /// Entity was spawned
    EntitySpawned = 0x08,
    /// Entity was despawned
    EntityDespawned = 0x09,
}

impl EventType {
    /// Check if this event type can be cancelled
    pub fn is_cancellable(&self) -> bool {
        matches!(
            self,
            EventType::PlayerChat | EventType::BlockPlaced | EventType::BlockBroken
        )
    }

    /// Get the capability required to cancel this event
    pub fn cancel_capability(&self) -> Option<crate::caps::Capability> {
        match self {
            EventType::PlayerChat => Some(crate::caps::Capability::EventCancelChat),
            EventType::BlockPlaced | EventType::BlockBroken => {
                Some(crate::caps::Capability::EventCancelBlocks)
            }
            _ => None,
        }
    }

    /// Get the string name of this event type
    pub fn name(&self) -> &'static str {
        match self {
            EventType::ServerStart => "server_start",
            EventType::ServerStop => "server_stop",
            EventType::PlayerJoin => "player_join",
            EventType::PlayerLeave => "player_leave",
            EventType::PlayerChat => "player_chat",
            EventType::BlockPlaced => "block_placed",
            EventType::BlockBroken => "block_broken",
            EventType::EntitySpawned => "entity_spawned",
            EventType::EntityDespawned => "entity_despawned",
        }
    }

    /// Parse event type from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(EventType::ServerStart),
            0x02 => Some(EventType::ServerStop),
            0x03 => Some(EventType::PlayerJoin),
            0x04 => Some(EventType::PlayerLeave),
            0x05 => Some(EventType::PlayerChat),
            0x06 => Some(EventType::BlockPlaced),
            0x07 => Some(EventType::BlockBroken),
            0x08 => Some(EventType::EntitySpawned),
            0x09 => Some(EventType::EntityDespawned),
            _ => None,
        }
    }
}

/// Subscribe to an event type
///
/// Call this during mod initialization to receive events of this type.
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::events::{subscribe, EventType};
///
/// fn init() {
///     subscribe(EventType::PlayerChat).unwrap();
///     subscribe(EventType::PlayerJoin).unwrap();
/// }
/// ```
pub fn subscribe(event_type: EventType) -> Result<()> {
    let result = abi::subscribe_event(event_type as u8);
    if result == 0 {
        Ok(())
    } else {
        Err(ModError::new(
            ErrorCode::from_u16(result as u16).unwrap_or(ErrorCode::InvalidArgument),
            alloc::format!("Failed to subscribe to event: {}", event_type.name()),
        ))
    }
}

/// Context for the currently processing event
///
/// Provides information about the event and allows cancellation.
#[derive(Debug, Clone)]
pub struct EventContext {
    /// The event type being processed
    pub event_type: EventType,
    /// Server tick when the event occurred
    pub tick: u64,
    /// Whether this event can be cancelled
    pub cancellable: bool,
}

impl EventContext {
    /// Create a new event context
    pub fn new(event_type: EventType, tick: u64) -> Self {
        Self {
            cancellable: event_type.is_cancellable(),
            event_type,
            tick,
        }
    }

    /// Cancel this event
    ///
    /// Only works for cancellable events and requires the appropriate capability.
    /// Returns an error if:
    /// - The event is not cancellable
    /// - The mod lacks the required capability
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// fn on_player_chat(ctx: &EventContext, payload: PlayerChatPayload) {
    ///     if payload.text.contains("banned_word") {
    ///         if let Err(e) = ctx.cancel() {
    ///             warn!("Could not cancel: {}", e);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn cancel(&self) -> Result<()> {
        if !self.cancellable {
            return Err(err_invalid("This event type cannot be cancelled"));
        }

        // Check capability
        if let Some(required_cap) = self.event_type.cancel_capability() {
            if !crate::caps::has_capability(required_cap) {
                return Err(err_perm(required_cap.as_str()));
            }
        }

        let result = abi::cancel_event();
        if result == 0 {
            Ok(())
        } else {
            Err(ModError::new(
                ErrorCode::from_u16(result as u16).unwrap_or(ErrorCode::InvalidArgument),
                "Failed to cancel event",
            ))
        }
    }
}

// === Event Payloads ===

/// Payload for PlayerJoin event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerJoinPayload {
    /// Unique player ID
    pub player_id: u64,
    /// Player display name
    pub name: String,
}

/// Reason why a player left
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaveReason {
    /// Player disconnected normally
    Disconnect,
    /// Player was kicked
    Kicked,
    /// Player timed out
    Timeout,
    /// Server is shutting down
    ServerShutdown,
}

/// Payload for PlayerLeave event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLeavePayload {
    /// Unique player ID
    pub player_id: u64,
    /// Why the player left
    pub reason: LeaveReason,
}

/// Payload for PlayerChat event (cancellable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerChatPayload {
    /// Sender player ID
    pub player_id: u64,
    /// Message content
    pub text: String,
}

/// Payload for BlockPlaced event (cancellable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPlacedPayload {
    /// Placer player ID (None if world-generated)
    pub player_id: Option<u64>,
    /// Block position
    pub pos: IVec3,
    /// Block type ID
    pub block_id: u16,
}

/// Payload for BlockBroken event (cancellable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockBrokenPayload {
    /// Breaker player ID (None if world-generated)
    pub player_id: Option<u64>,
    /// Block position
    pub pos: IVec3,
    /// Previous block type ID
    pub block_id: u16,
}

/// Payload for ServerStart event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStartPayload {
    /// Server tick number
    pub tick: u64,
}

/// Payload for ServerStop event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStopPayload {
    /// Shutdown reason
    pub reason: String,
}

/// Payload for EntitySpawned event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySpawnedPayload {
    /// Entity ID
    pub entity_id: u64,
    /// Entity type name
    pub entity_type: String,
}

/// Payload for EntityDespawned event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDespawnedPayload {
    /// Entity ID
    pub entity_id: u64,
    /// Reason for despawning
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_cancellable() {
        assert!(EventType::PlayerChat.is_cancellable());
        assert!(EventType::BlockPlaced.is_cancellable());
        assert!(EventType::BlockBroken.is_cancellable());
        assert!(!EventType::PlayerJoin.is_cancellable());
        assert!(!EventType::ServerStart.is_cancellable());
    }

    #[test]
    fn test_event_type_from_u8() {
        assert_eq!(EventType::from_u8(0x01), Some(EventType::ServerStart));
        assert_eq!(EventType::from_u8(0x05), Some(EventType::PlayerChat));
        assert_eq!(EventType::from_u8(0xFF), None);
    }

    #[test]
    fn test_event_context_non_cancellable() {
        let ctx = EventContext::new(EventType::PlayerJoin, 100);
        assert!(!ctx.cancellable);
    }

    #[test]
    fn test_player_chat_payload() {
        let payload = PlayerChatPayload {
            player_id: 123,
            text: "Hello world".into(),
        };
        assert_eq!(payload.player_id, 123);
        assert_eq!(payload.text, "Hello world");
    }
}
