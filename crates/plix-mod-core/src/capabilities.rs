//! Capability-based permission system for mods
//!
//! Mods declare required capabilities in their manifest.
//! Server administrators can override (grant/deny) capabilities per mod.

use crate::errors::{err_perm, ModApiError};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

bitflags! {
    /// Permission flags that mods request and engine grants/denies
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Capability: u32 {
        /// Read world state (get_block, raycast, query_aabb)
        const WORLD_READ = 0b0000001;
        /// Modify world state (set_block)
        const WORLD_WRITE = 0b0000010;
        /// Read entity state (get_transform, get_health, etc.)
        const ENTITY_READ = 0b0000100;
        /// Modify entities (apply_damage, apply_impulse, spawn/despawn)
        const ENTITY_WRITE = 0b0001000;
        /// Send network messages
        const NET_SEND = 0b0010000;
        /// Cancel chat events
        const EVENT_CANCEL_CHAT = 0b0100000;
        /// Cancel block placement/breaking events
        const EVENT_CANCEL_BLOCKS = 0b1000000;
    }
}

impl Default for Capability {
    fn default() -> Self {
        Capability::empty()
    }
}

// Manual Serialize/Deserialize for Capability (as u32)
impl Serialize for Capability {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.bits().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Capability {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u32::deserialize(deserializer)?;
        Ok(Capability::from_bits_truncate(bits))
    }
}

impl Capability {
    /// Parse capability from string (e.g., "world.read" -> WORLD_READ)
    ///
    /// Note: We don't implement `FromStr` trait because our return type is `Option<Self>`
    /// rather than `Result<Self, Error>`, which is more ergonomic for this use case.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "world.read" | "world_read" => Some(Capability::WORLD_READ),
            "world.write" | "world_write" => Some(Capability::WORLD_WRITE),
            "entity.read" | "entity_read" => Some(Capability::ENTITY_READ),
            "entity.write" | "entity_write" => Some(Capability::ENTITY_WRITE),
            "net.send" | "net_send" => Some(Capability::NET_SEND),
            "event.cancel.chat" | "event_cancel_chat" => Some(Capability::EVENT_CANCEL_CHAT),
            "event.cancel.blocks" | "event_cancel_blocks" => Some(Capability::EVENT_CANCEL_BLOCKS),
            _ => None,
        }
    }

    /// Get the string representation of a single capability flag
    pub fn as_str(&self) -> &'static str {
        match *self {
            Capability::WORLD_READ => "world.read",
            Capability::WORLD_WRITE => "world.write",
            Capability::ENTITY_READ => "entity.read",
            Capability::ENTITY_WRITE => "entity.write",
            Capability::NET_SEND => "net.send",
            Capability::EVENT_CANCEL_CHAT => "event.cancel.chat",
            Capability::EVENT_CANCEL_BLOCKS => "event.cancel.blocks",
            _ => "unknown",
        }
    }
}

/// Check if capabilities include the required capability, returning EMOD002 if not
pub fn require(granted: Capability, required: Capability) -> Result<(), ModApiError> {
    if granted.contains(required) {
        Ok(())
    } else {
        Err(err_perm(required.as_str()))
    }
}

/// Server policy for capability overrides
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityPolicy {
    /// Default capabilities granted to all mods (unless denied)
    pub default_granted: Capability,
    /// Default capabilities denied to all mods
    pub default_denied: Capability,
    /// Per-mod capability overrides (grant)
    pub mod_grants: HashMap<String, Capability>,
    /// Per-mod capability overrides (deny)
    pub mod_denies: HashMap<String, Capability>,
}

impl CapabilityPolicy {
    /// Create a new policy with all defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a permissive policy that grants all capabilities by default
    pub fn permissive() -> Self {
        Self {
            default_granted: Capability::all(),
            default_denied: Capability::empty(),
            mod_grants: HashMap::new(),
            mod_denies: HashMap::new(),
        }
    }

    /// Grant a capability to a specific mod
    pub fn grant_to_mod(&mut self, mod_id: impl Into<String>, cap: Capability) {
        let entry = self
            .mod_grants
            .entry(mod_id.into())
            .or_insert(Capability::empty());
        *entry |= cap;
    }

    /// Deny a capability from a specific mod
    pub fn deny_from_mod(&mut self, mod_id: impl Into<String>, cap: Capability) {
        let entry = self
            .mod_denies
            .entry(mod_id.into())
            .or_insert(Capability::empty());
        *entry |= cap;
    }

    /// Calculate effective capabilities for a mod
    ///
    /// Formula: (manifest_caps ∩ default_granted) ∪ mod_grants - default_denied - mod_denies
    pub fn effective_capabilities(&self, mod_id: &str, manifest_caps: Capability) -> Capability {
        // Start with capabilities requested by manifest AND granted by default
        let mut effective = manifest_caps & self.default_granted;

        // Add any per-mod grants (even if not in manifest - admin override)
        if let Some(grants) = self.mod_grants.get(mod_id) {
            effective |= *grants;
        }

        // Remove default denies
        effective -= self.default_denied;

        // Remove per-mod denies
        if let Some(denies) = self.mod_denies.get(mod_id) {
            effective -= *denies;
        }

        effective
    }
}

/// Parsed capabilities from a manifest's [capabilities] section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestCapabilities {
    #[serde(default)]
    pub world_read: bool,
    #[serde(default)]
    pub world_write: bool,
    #[serde(default)]
    pub entity_read: bool,
    #[serde(default)]
    pub entity_write: bool,
    #[serde(default)]
    pub net_send: bool,
    #[serde(default)]
    pub event_cancel_chat: bool,
    #[serde(default)]
    pub event_cancel_blocks: bool,
}

impl ManifestCapabilities {
    /// Convert to bitflags
    pub fn to_capability(&self) -> Capability {
        let mut caps = Capability::empty();
        if self.world_read {
            caps |= Capability::WORLD_READ;
        }
        if self.world_write {
            caps |= Capability::WORLD_WRITE;
        }
        if self.entity_read {
            caps |= Capability::ENTITY_READ;
        }
        if self.entity_write {
            caps |= Capability::ENTITY_WRITE;
        }
        if self.net_send {
            caps |= Capability::NET_SEND;
        }
        if self.event_cancel_chat {
            caps |= Capability::EVENT_CANCEL_CHAT;
        }
        if self.event_cancel_blocks {
            caps |= Capability::EVENT_CANCEL_BLOCKS;
        }
        caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_from_str() {
        assert_eq!(
            Capability::from_str("world.read"),
            Some(Capability::WORLD_READ)
        );
        assert_eq!(
            Capability::from_str("world_read"),
            Some(Capability::WORLD_READ)
        );
        assert_eq!(
            Capability::from_str("entity.write"),
            Some(Capability::ENTITY_WRITE)
        );
        assert_eq!(
            Capability::from_str("event.cancel.chat"),
            Some(Capability::EVENT_CANCEL_CHAT)
        );
        assert_eq!(Capability::from_str("unknown"), None);
    }

    #[test]
    fn test_require_success() {
        let granted = Capability::WORLD_READ | Capability::WORLD_WRITE;
        assert!(require(granted, Capability::WORLD_READ).is_ok());
        assert!(require(granted, Capability::WORLD_WRITE).is_ok());
    }

    #[test]
    fn test_require_failure() {
        let granted = Capability::WORLD_READ;
        let result = require(granted, Capability::WORLD_WRITE);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, crate::errors::ErrorCode::PermissionDenied);
        assert!(err.message.contains("world.write"));
    }

    #[test]
    fn test_policy_default() {
        let policy = CapabilityPolicy::new();
        let manifest_caps = Capability::WORLD_READ | Capability::WORLD_WRITE;

        // With empty default_granted, no capabilities are effective
        let effective = policy.effective_capabilities("test-mod", manifest_caps);
        assert!(effective.is_empty());
    }

    #[test]
    fn test_policy_permissive() {
        let policy = CapabilityPolicy::permissive();
        let manifest_caps = Capability::WORLD_READ | Capability::WORLD_WRITE;

        let effective = policy.effective_capabilities("test-mod", manifest_caps);
        assert!(effective.contains(Capability::WORLD_READ));
        assert!(effective.contains(Capability::WORLD_WRITE));
    }

    #[test]
    fn test_policy_mod_deny() {
        let mut policy = CapabilityPolicy::permissive();
        policy.deny_from_mod("restricted-mod", Capability::WORLD_WRITE);

        let manifest_caps = Capability::WORLD_READ | Capability::WORLD_WRITE;
        let effective = policy.effective_capabilities("restricted-mod", manifest_caps);

        assert!(effective.contains(Capability::WORLD_READ));
        assert!(!effective.contains(Capability::WORLD_WRITE));
    }

    #[test]
    fn test_policy_mod_grant() {
        let mut policy = CapabilityPolicy::new();
        policy.grant_to_mod("privileged-mod", Capability::NET_SEND);

        // Even with empty manifest caps, admin can grant
        let effective = policy.effective_capabilities("privileged-mod", Capability::empty());
        assert!(effective.contains(Capability::NET_SEND));
    }

    #[test]
    fn test_manifest_capabilities_conversion() {
        let manifest_caps = ManifestCapabilities {
            world_read: true,
            world_write: true,
            entity_read: false,
            entity_write: false,
            net_send: true,
            event_cancel_chat: false,
            event_cancel_blocks: false,
        };

        let caps = manifest_caps.to_capability();
        assert!(caps.contains(Capability::WORLD_READ));
        assert!(caps.contains(Capability::WORLD_WRITE));
        assert!(caps.contains(Capability::NET_SEND));
        assert!(!caps.contains(Capability::ENTITY_READ));
        assert!(!caps.contains(Capability::EVENT_CANCEL_CHAT));
    }
}
