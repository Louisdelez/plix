//! ABI (Application Binary Interface) for WASM mod-engine communication
//!
//! This module defines the stable interface between the game engine (host)
//! and WASM mod modules. All mods must conform to ABI version 1.

pub mod response;
pub mod types;

pub use response::ResponseBuffer;
pub use types::{AbiRequest, AbiResponse};

/// Current ABI version
pub const ABI_VERSION: u32 = 1;

/// Current API version (same as engine)
pub const API_VERSION: u32 = 1;

/// Operation codes for host function calls
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HostCallOp {
    // === Logging (0x01-0x0F) ===
    /// Log a message
    Log = 0x01,

    // === Capabilities (0x10-0x1F) ===
    /// Check if mod has capability
    HasCapability = 0x10,
    /// Get API version
    GetApiVersion = 0x11,
    /// Get ABI version
    GetAbiVersion = 0x12,

    // === Events (0x20-0x2F) ===
    /// Subscribe to an event type
    SubscribeEvent = 0x20,
    /// Cancel the current event
    CancelEvent = 0x21,

    // === World API (0x30-0x3F) ===
    /// Get block at position
    WorldGetBlock = 0x30,
    /// Set block at position
    WorldSetBlock = 0x31,
    /// Raycast in world
    WorldRaycast = 0x32,
    /// Query AABB
    WorldQueryAabb = 0x33,

    // === Entity API (0x40-0x4F) ===
    /// Get entity transform
    EntityGetTransform = 0x40,
    /// Get entity health
    EntityGetHealth = 0x41,
    /// Apply damage to entity
    EntityApplyDamage = 0x42,
    /// Apply impulse to entity
    EntityApplyImpulse = 0x43,

    // === Net API (0x50-0x5F) ===
    /// Send message to player
    NetSend = 0x50,
    /// Broadcast message to all players
    NetBroadcast = 0x51,

    // === Timer API (0x60-0x6F) ===
    /// Set a one-shot timeout
    TimerSetTimeout = 0x60,
    /// Set a repeating interval
    TimerSetInterval = 0x61,
    /// Clear a timer
    TimerClear = 0x62,
}

impl HostCallOp {
    /// Try to convert a u8 to a HostCallOp
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Log),
            0x10 => Some(Self::HasCapability),
            0x11 => Some(Self::GetApiVersion),
            0x12 => Some(Self::GetAbiVersion),
            0x20 => Some(Self::SubscribeEvent),
            0x21 => Some(Self::CancelEvent),
            0x30 => Some(Self::WorldGetBlock),
            0x31 => Some(Self::WorldSetBlock),
            0x32 => Some(Self::WorldRaycast),
            0x33 => Some(Self::WorldQueryAabb),
            0x40 => Some(Self::EntityGetTransform),
            0x41 => Some(Self::EntityGetHealth),
            0x42 => Some(Self::EntityApplyDamage),
            0x43 => Some(Self::EntityApplyImpulse),
            0x50 => Some(Self::NetSend),
            0x51 => Some(Self::NetBroadcast),
            0x60 => Some(Self::TimerSetTimeout),
            0x61 => Some(Self::TimerSetInterval),
            0x62 => Some(Self::TimerClear),
            _ => None,
        }
    }

    /// Get the operation name for logging
    pub fn name(&self) -> &'static str {
        match self {
            Self::Log => "log",
            Self::HasCapability => "has_capability",
            Self::GetApiVersion => "get_api_version",
            Self::GetAbiVersion => "get_abi_version",
            Self::SubscribeEvent => "subscribe_event",
            Self::CancelEvent => "cancel_event",
            Self::WorldGetBlock => "world.get_block",
            Self::WorldSetBlock => "world.set_block",
            Self::WorldRaycast => "world.raycast",
            Self::WorldQueryAabb => "world.query_aabb",
            Self::EntityGetTransform => "entity.get_transform",
            Self::EntityGetHealth => "entity.get_health",
            Self::EntityApplyDamage => "entity.apply_damage",
            Self::EntityApplyImpulse => "entity.apply_impulse",
            Self::NetSend => "net.send",
            Self::NetBroadcast => "net.broadcast",
            Self::TimerSetTimeout => "timer.set_timeout",
            Self::TimerSetInterval => "timer.set_interval",
            Self::TimerClear => "timer.clear",
        }
    }
}

/// Event type IDs as defined in the ABI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EventTypeId {
    ServerStart = 0x01,
    ServerStop = 0x02,
    PlayerJoin = 0x03,
    PlayerLeave = 0x04,
    PlayerChat = 0x05,
    BlockPlaced = 0x06,
    BlockBroken = 0x07,
    EntitySpawned = 0x08,
    EntityDespawned = 0x09,
}

impl EventTypeId {
    /// Convert from plix_mod_core EventType
    pub fn from_event_type(event_type: plix_mod_core::EventType) -> Option<Self> {
        use plix_mod_core::EventType;
        match event_type {
            EventType::ServerStart => Some(Self::ServerStart),
            EventType::ServerStop => Some(Self::ServerStop),
            EventType::PlayerJoin => Some(Self::PlayerJoin),
            EventType::PlayerLeave => Some(Self::PlayerLeave),
            EventType::PlayerChat => Some(Self::PlayerChat),
            EventType::BlockPlaced => Some(Self::BlockPlaced),
            EventType::BlockBroken => Some(Self::BlockBroken),
            // EntitySpawned and EntityDespawned don't exist in mod_core yet
            _ => None,
        }
    }

    /// Convert to plix_mod_core EventType
    pub fn to_event_type(self) -> Option<plix_mod_core::EventType> {
        use plix_mod_core::EventType;
        match self {
            Self::ServerStart => Some(EventType::ServerStart),
            Self::ServerStop => Some(EventType::ServerStop),
            Self::PlayerJoin => Some(EventType::PlayerJoin),
            Self::PlayerLeave => Some(EventType::PlayerLeave),
            Self::PlayerChat => Some(EventType::PlayerChat),
            Self::BlockPlaced => Some(EventType::BlockPlaced),
            Self::BlockBroken => Some(EventType::BlockBroken),
            // EntitySpawned and EntityDespawned don't exist in mod_core yet
            Self::EntitySpawned | Self::EntityDespawned => None,
        }
    }

    /// Try to convert from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::ServerStart),
            0x02 => Some(Self::ServerStop),
            0x03 => Some(Self::PlayerJoin),
            0x04 => Some(Self::PlayerLeave),
            0x05 => Some(Self::PlayerChat),
            0x06 => Some(Self::BlockPlaced),
            0x07 => Some(Self::BlockBroken),
            0x08 => Some(Self::EntitySpawned),
            0x09 => Some(Self::EntityDespawned),
            _ => None,
        }
    }
}

/// Capability IDs as defined in the ABI (matches plix_mod_core::Capability bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum CapabilityId {
    WorldRead = 0x01,
    WorldWrite = 0x02,
    EntityRead = 0x04,
    EntityWrite = 0x08,
    NetSend = 0x10,
    EventCancelChat = 0x20,
    EventCancelBlocks = 0x40,
}

impl CapabilityId {
    /// Convert to plix_mod_core::Capability
    pub fn to_capability(self) -> plix_mod_core::Capability {
        use plix_mod_core::Capability;
        match self {
            Self::WorldRead => Capability::WORLD_READ,
            Self::WorldWrite => Capability::WORLD_WRITE,
            Self::EntityRead => Capability::ENTITY_READ,
            Self::EntityWrite => Capability::ENTITY_WRITE,
            Self::NetSend => Capability::NET_SEND,
            Self::EventCancelChat => Capability::EVENT_CANCEL_CHAT,
            Self::EventCancelBlocks => Capability::EVENT_CANCEL_BLOCKS,
        }
    }

    /// Try to convert from u32
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0x01 => Some(Self::WorldRead),
            0x02 => Some(Self::WorldWrite),
            0x04 => Some(Self::EntityRead),
            0x08 => Some(Self::EntityWrite),
            0x10 => Some(Self::NetSend),
            0x20 => Some(Self::EventCancelChat),
            0x40 => Some(Self::EventCancelBlocks),
            _ => None,
        }
    }
}

/// Log level IDs as defined in the ABI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl LogLevel {
    /// Convert from i32
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Error),
            1 => Some(Self::Warn),
            2 => Some(Self::Info),
            3 => Some(Self::Debug),
            4 => Some(Self::Trace),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_call_op_from_u8() {
        assert_eq!(HostCallOp::from_u8(0x01), Some(HostCallOp::Log));
        assert_eq!(HostCallOp::from_u8(0x30), Some(HostCallOp::WorldGetBlock));
        assert_eq!(HostCallOp::from_u8(0xFF), None);
    }

    #[test]
    fn test_event_type_id_from_u8() {
        assert_eq!(EventTypeId::from_u8(0x01), Some(EventTypeId::ServerStart));
        assert_eq!(EventTypeId::from_u8(0x05), Some(EventTypeId::PlayerChat));
        assert_eq!(EventTypeId::from_u8(0xFF), None);
    }

    #[test]
    fn test_capability_id_to_capability() {
        assert_eq!(
            CapabilityId::WorldRead.to_capability(),
            plix_mod_core::Capability::WORLD_READ
        );
        assert_eq!(
            CapabilityId::EventCancelChat.to_capability(),
            plix_mod_core::Capability::EVENT_CANCEL_CHAT
        );
    }

    #[test]
    fn test_log_level_from_i32() {
        assert_eq!(LogLevel::from_i32(0), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_i32(2), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_i32(99), None);
    }
}
