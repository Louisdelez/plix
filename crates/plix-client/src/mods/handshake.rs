//! Mod handshake handling for client-server negotiation
//!
//! Handles receiving ModSetDescriptor from server and building ClientCapabilities response.

use plix_common::protocol::messages::{
    JoinDecision, ModDescriptor, ModSetDescriptor, ModSetResponse, PayloadBegin,
};
use serde::{Deserialize, Serialize};

/// Client's capabilities and cache state sent during mod handshake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Whether this client supports payload synchronization
    pub supports_sync: bool,
    /// SHA-256 hashes of cached payloads (64 hex chars each)
    pub cached_payload_hashes: Vec<String>,
    /// Client engine version for compatibility check
    pub engine_version: Option<String>,
}

impl Default for ClientCapabilities {
    fn default() -> Self {
        Self {
            supports_sync: true,
            cached_payload_hashes: Vec::new(),
            engine_version: None,
        }
    }
}

impl ClientCapabilities {
    /// Create new capabilities with sync support
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the engine version
    pub fn with_engine_version(mut self, version: impl Into<String>) -> Self {
        self.engine_version = Some(version.into());
        self
    }

    /// Add cached payload hashes from the local cache
    pub fn with_cached_hashes(mut self, hashes: Vec<String>) -> Self {
        self.cached_payload_hashes = hashes;
        self
    }

    /// Check if a specific payload hash is cached
    pub fn has_cached_payload(&self, hash: &str) -> bool {
        self.cached_payload_hashes.iter().any(|h| h == hash)
    }

    /// Convert to protocol ModSetResponse
    pub fn to_response(&self) -> ModSetResponse {
        ModSetResponse {
            supports_sync: self.supports_sync,
            cached_hashes: self.cached_payload_hashes.clone(),
            engine_version: self.engine_version.clone(),
        }
    }
}

/// State of the client-side handshake process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeState {
    /// Waiting to receive ModSetDescriptor from server
    AwaitingModSet,
    /// Received ModSet, need to send response
    ReceivedModSet,
    /// Sent response, waiting for JoinDecision
    AwaitingDecision,
    /// Received decision, waiting for payload transfer (if needed)
    AwaitingPayloads,
    /// All payloads received (or none needed), handshake complete
    Complete,
    /// Server rejected our connection
    Rejected,
}

/// Client-side mod handshake manager
pub struct ModHandshake {
    /// Current handshake state
    state: HandshakeState,
    /// Our capabilities
    capabilities: ClientCapabilities,
    /// Server's mod set (received during handshake)
    server_mods: Option<ModSetDescriptor>,
    /// Mods that need to be synced
    mods_to_sync: Vec<String>,
    /// Rejection reason if rejected
    rejection_reason: Option<String>,
}

impl ModHandshake {
    /// Create a new handshake manager with given capabilities
    pub fn new(capabilities: ClientCapabilities) -> Self {
        Self {
            state: HandshakeState::AwaitingModSet,
            capabilities,
            server_mods: None,
            mods_to_sync: Vec::new(),
            rejection_reason: None,
        }
    }

    /// Get current handshake state
    pub fn state(&self) -> HandshakeState {
        self.state
    }

    /// Check if handshake is complete (success or rejection)
    pub fn is_finished(&self) -> bool {
        matches!(
            self.state,
            HandshakeState::Complete | HandshakeState::Rejected
        )
    }

    /// Check if we were rejected
    pub fn is_rejected(&self) -> bool {
        self.state == HandshakeState::Rejected
    }

    /// Get rejection reason if rejected
    pub fn rejection_reason(&self) -> Option<&str> {
        self.rejection_reason.as_deref()
    }

    /// Handle receiving ModSetDescriptor from server
    ///
    /// Returns the ModSetResponse to send back to the server.
    pub fn handle_mod_set(&mut self, descriptor: ModSetDescriptor) -> ModSetResponse {
        self.server_mods = Some(descriptor);
        self.state = HandshakeState::ReceivedModSet;

        let response = self.capabilities.to_response();
        self.state = HandshakeState::AwaitingDecision;

        response
    }

    /// Handle receiving JoinDecision from server
    ///
    /// Returns true if we should proceed with connection, false if rejected.
    pub fn handle_decision(&mut self, decision: JoinDecision) -> bool {
        if decision.allowed {
            self.mods_to_sync = decision.mods_to_sync;

            if self.mods_to_sync.is_empty() {
                self.state = HandshakeState::Complete;
            } else {
                self.state = HandshakeState::AwaitingPayloads;
            }

            true
        } else {
            self.state = HandshakeState::Rejected;
            self.rejection_reason = decision.rejection_reason;
            false
        }
    }

    /// Handle receiving PayloadBegin for a mod
    pub fn handle_payload_begin(&mut self, payload: &PayloadBegin) {
        // Remove from pending sync list if present
        if let Some(pos) = self.mods_to_sync.iter().position(|m| m == &payload.mod_id) {
            // Don't remove yet - wait until payload is complete
        }
    }

    /// Mark a mod's payload as complete
    pub fn mark_payload_complete(&mut self, mod_id: &str) {
        self.mods_to_sync.retain(|m| m != mod_id);

        if self.mods_to_sync.is_empty() {
            self.state = HandshakeState::Complete;
        }
    }

    /// Get the list of mods that still need syncing
    pub fn pending_mods(&self) -> &[String] {
        &self.mods_to_sync
    }

    /// Get the server's mod descriptors (if received)
    pub fn server_mods(&self) -> Option<&[ModDescriptor]> {
        self.server_mods.as_ref().map(|d| d.mods.as_slice())
    }

    /// Get total payload size to sync
    pub fn total_payload_size(&self) -> u64 {
        self.server_mods
            .as_ref()
            .map(|d| {
                d.mods
                    .iter()
                    .filter(|m| self.mods_to_sync.contains(&m.id))
                    .map(|m| m.client_payload_size)
                    .sum()
            })
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::protocol::messages::ModJoinPolicy;

    #[test]
    fn test_default_capabilities() {
        let caps = ClientCapabilities::default();
        assert!(caps.supports_sync);
        assert!(caps.cached_payload_hashes.is_empty());
        assert!(caps.engine_version.is_none());
    }

    #[test]
    fn test_capabilities_builder() {
        let caps = ClientCapabilities::new()
            .with_engine_version("0.37.0")
            .with_cached_hashes(vec!["abc123".to_string(), "def456".to_string()]);

        assert_eq!(caps.engine_version, Some("0.37.0".to_string()));
        assert_eq!(caps.cached_payload_hashes.len(), 2);
        assert!(caps.has_cached_payload("abc123"));
        assert!(!caps.has_cached_payload("xyz789"));
    }

    #[test]
    fn test_handshake_no_mods() {
        let caps = ClientCapabilities::new();
        let mut handshake = ModHandshake::new(caps);

        assert_eq!(handshake.state(), HandshakeState::AwaitingModSet);

        // Receive empty mod set
        let descriptor = ModSetDescriptor {
            mods: vec![],
            total_payload_size: 0,
            policy: ModJoinPolicy {
                allow_server_only: true,
                allow_payload_sync: true,
                require_payload_sync: false,
            },
        };

        let response = handshake.handle_mod_set(descriptor);
        assert!(response.supports_sync);
        assert_eq!(handshake.state(), HandshakeState::AwaitingDecision);

        // Receive approval with no sync needed
        let decision = JoinDecision {
            allowed: true,
            rejection_reason: None,
            mods_to_sync: vec![],
        };

        assert!(handshake.handle_decision(decision));
        assert_eq!(handshake.state(), HandshakeState::Complete);
        assert!(handshake.is_finished());
    }

    #[test]
    fn test_handshake_rejected() {
        let caps = ClientCapabilities::new();
        let mut handshake = ModHandshake::new(caps);

        let descriptor = ModSetDescriptor {
            mods: vec![],
            total_payload_size: 0,
            policy: ModJoinPolicy::default(),
        };

        handshake.handle_mod_set(descriptor);

        let decision = JoinDecision {
            allowed: false,
            rejection_reason: Some("Test rejection".to_string()),
            mods_to_sync: vec![],
        };

        assert!(!handshake.handle_decision(decision));
        assert!(handshake.is_rejected());
        assert_eq!(handshake.rejection_reason(), Some("Test rejection"));
    }

    #[test]
    fn test_handshake_with_sync() {
        let caps = ClientCapabilities::new();
        let mut handshake = ModHandshake::new(caps);

        let descriptor = ModSetDescriptor {
            mods: vec![ModDescriptor {
                id: "test-mod".to_string(),
                version: "1.0.0".to_string(),
                client_payload_hash: Some("abc123".to_string()),
                client_payload_size: 1024,
                has_network_channels: false,
            }],
            total_payload_size: 1024,
            policy: ModJoinPolicy::default(),
        };

        handshake.handle_mod_set(descriptor);

        let decision = JoinDecision {
            allowed: true,
            rejection_reason: None,
            mods_to_sync: vec!["test-mod".to_string()],
        };

        assert!(handshake.handle_decision(decision));
        assert_eq!(handshake.state(), HandshakeState::AwaitingPayloads);
        assert_eq!(handshake.pending_mods(), &["test-mod"]);

        // Mark complete
        handshake.mark_payload_complete("test-mod");
        assert_eq!(handshake.state(), HandshakeState::Complete);
    }
}
