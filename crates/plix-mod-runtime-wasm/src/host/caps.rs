//! Capability and version query host functions
//!
//! Implements plix_has_capability, plix_get_api_version, plix_get_abi_version.
//!
//! ## Capability Discovery
//!
//! Mods can query their granted capabilities at runtime using `plix_has_capability`:
//!
//! ```text
//! // From WASM mod:
//! let has_world_read = plix_has_capability(0x01); // WorldRead
//! let has_net_send = plix_has_capability(0x10);   // NetSend
//! ```
//!
//! ## Capability IDs
//!
//! The capability IDs are defined in `abi/mod.rs` as `CapabilityId`:
//!
//! | ID   | Capability         | Description |
//! |------|-------------------|-------------|
//! | 0x01 | WORLD_READ        | Read block data |
//! | 0x02 | WORLD_WRITE       | Modify blocks |
//! | 0x04 | ENTITY_READ       | Read entity data |
//! | 0x08 | ENTITY_WRITE      | Modify entities |
//! | 0x10 | NET_SEND          | Send network messages |
//! | 0x20 | EVENT_CANCEL_CHAT | Cancel chat events |
//! | 0x40 | EVENT_CANCEL_BLOCKS | Cancel block events |
//!
//! Note: The actual plix_has_capability, plix_get_api_version, and plix_get_abi_version
//! functions are implemented in host/mod.rs as they need direct access to the Caller context.

#[cfg(test)]
mod tests {
    use crate::abi::CapabilityId;
    use plix_mod_core::Capability;

    #[test]
    fn test_has_capability_returns_1_for_granted_cap() {
        // Simulate what plix_has_capability does
        let granted = Capability::WORLD_READ | Capability::NET_SEND;

        // Check WORLD_READ (0x01) - should be granted
        let cap_id = CapabilityId::from_u32(0x01).unwrap();
        let cap = cap_id.to_capability();
        assert!(granted.contains(cap), "WORLD_READ should be granted");

        // Check NET_SEND (0x10) - should be granted
        let cap_id = CapabilityId::from_u32(0x10).unwrap();
        let cap = cap_id.to_capability();
        assert!(granted.contains(cap), "NET_SEND should be granted");
    }

    #[test]
    fn test_has_capability_returns_0_for_denied_cap() {
        // Simulate what plix_has_capability does
        let granted = Capability::WORLD_READ; // Only WORLD_READ granted

        // Check WORLD_WRITE (0x02) - should NOT be granted
        let cap_id = CapabilityId::from_u32(0x02).unwrap();
        let cap = cap_id.to_capability();
        assert!(!granted.contains(cap), "WORLD_WRITE should NOT be granted");

        // Check EVENT_CANCEL_CHAT (0x20) - should NOT be granted
        let cap_id = CapabilityId::from_u32(0x20).unwrap();
        let cap = cap_id.to_capability();
        assert!(
            !granted.contains(cap),
            "EVENT_CANCEL_CHAT should NOT be granted"
        );
    }

    #[test]
    fn test_unknown_capability_id_returns_none() {
        // Unknown capability IDs should return None
        assert!(CapabilityId::from_u32(0x00).is_none());
        assert!(CapabilityId::from_u32(0x80).is_none());
        assert!(CapabilityId::from_u32(0xFF).is_none());
    }
}
