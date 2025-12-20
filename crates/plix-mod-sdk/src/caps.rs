//! Capability checking API
//!
//! Mods declare required capabilities in their manifest. The runtime grants
//! or denies capabilities based on server policy. Use this module to check
//! if your mod has a specific capability before making API calls.
//!
//! # Example
//!
//! ```rust,ignore
//! use plix_mod_sdk::caps::{has_capability, Capability};
//!
//! if has_capability(Capability::WorldWrite) {
//!     // Safe to call set_block
//!     world::set_block(pos, block_id)?;
//! } else {
//!     warn("World write capability not granted");
//! }
//! ```

use crate::abi;

/// Capability flags that control what API calls a mod can make
///
/// These match the capabilities defined in plix-mod-core and the ABI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Capability {
    /// Read world state (get_block, raycast, query_aabb)
    WorldRead = 0x01,
    /// Modify world state (set_block)
    WorldWrite = 0x02,
    /// Read entity state (get_transform, get_health)
    EntityRead = 0x04,
    /// Modify entities (apply_damage, apply_impulse)
    EntityWrite = 0x08,
    /// Send network messages
    NetSend = 0x10,
    /// Cancel chat events
    EventCancelChat = 0x20,
    /// Cancel block placement/breaking events
    EventCancelBlocks = 0x40,
}

impl Capability {
    /// Get the string representation of this capability
    pub fn as_str(&self) -> &'static str {
        match self {
            Capability::WorldRead => "world.read",
            Capability::WorldWrite => "world.write",
            Capability::EntityRead => "entity.read",
            Capability::EntityWrite => "entity.write",
            Capability::NetSend => "net.send",
            Capability::EventCancelChat => "event.cancel.chat",
            Capability::EventCancelBlocks => "event.cancel.blocks",
        }
    }

    /// Parse a capability from its string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "world.read" | "world_read" => Some(Capability::WorldRead),
            "world.write" | "world_write" => Some(Capability::WorldWrite),
            "entity.read" | "entity_read" => Some(Capability::EntityRead),
            "entity.write" | "entity_write" => Some(Capability::EntityWrite),
            "net.send" | "net_send" => Some(Capability::NetSend),
            "event.cancel.chat" | "event_cancel_chat" => Some(Capability::EventCancelChat),
            "event.cancel.blocks" | "event_cancel_blocks" => Some(Capability::EventCancelBlocks),
            _ => None,
        }
    }
}

/// Check if the mod has a specific capability
///
/// Returns `true` if the capability was granted by the server.
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::caps::{has_capability, Capability};
///
/// if has_capability(Capability::WorldRead) {
///     let block = world::get_block(pos)?;
///     info!("Block at pos: {}", block);
/// }
/// ```
pub fn has_capability(cap: Capability) -> bool {
    abi::has_capability(cap as u32)
}

/// Check if the mod has all of the specified capabilities
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::caps::{has_all_capabilities, Capability};
///
/// if has_all_capabilities(&[Capability::WorldRead, Capability::WorldWrite]) {
///     // Can both read and write world
/// }
/// ```
pub fn has_all_capabilities(caps: &[Capability]) -> bool {
    caps.iter().all(|&cap| has_capability(cap))
}

/// Check if the mod has any of the specified capabilities
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::caps::{has_any_capability, Capability};
///
/// if has_any_capability(&[Capability::EntityRead, Capability::WorldRead]) {
///     // Can read either entities or world
/// }
/// ```
pub fn has_any_capability(caps: &[Capability]) -> bool {
    caps.iter().any(|&cap| has_capability(cap))
}

/// Require a capability, returning an error if not granted
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::caps::{require_capability, Capability};
///
/// fn do_world_write() -> Result<()> {
///     require_capability(Capability::WorldWrite)?;
///     // ... safe to write
///     Ok(())
/// }
/// ```
pub fn require_capability(cap: Capability) -> crate::error::Result<()> {
    if has_capability(cap) {
        Ok(())
    } else {
        Err(crate::error::err_perm(cap.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_values() {
        assert_eq!(Capability::WorldRead as u32, 0x01);
        assert_eq!(Capability::WorldWrite as u32, 0x02);
        assert_eq!(Capability::EntityRead as u32, 0x04);
        assert_eq!(Capability::EntityWrite as u32, 0x08);
        assert_eq!(Capability::NetSend as u32, 0x10);
        assert_eq!(Capability::EventCancelChat as u32, 0x20);
        assert_eq!(Capability::EventCancelBlocks as u32, 0x40);
    }

    #[test]
    fn test_capability_from_str() {
        assert_eq!(Capability::from_str("world.read"), Some(Capability::WorldRead));
        assert_eq!(Capability::from_str("world_read"), Some(Capability::WorldRead));
        assert_eq!(Capability::from_str("net.send"), Some(Capability::NetSend));
        assert_eq!(Capability::from_str("unknown"), None);
    }

    #[test]
    fn test_capability_as_str() {
        assert_eq!(Capability::WorldRead.as_str(), "world.read");
        assert_eq!(Capability::NetSend.as_str(), "net.send");
        assert_eq!(Capability::EventCancelChat.as_str(), "event.cancel.chat");
    }
}
