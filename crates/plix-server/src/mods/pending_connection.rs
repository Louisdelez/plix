//! Pending mod connection tracking (Feature 037)
//!
//! Tracks clients that have passed initial validation but are waiting
//! for mod handshake to complete before being admitted to the game.

use super::sync_session::{ModInfo, SyncSession, SyncState};
use plix_common::identity::SessionId;
use plix_common::protocol::messages::{JoinDecision, ModSetResponse, PayloadAck};
use plix_common::types::TeamId;
use plix_mod_distribution::{JoinPolicy, SyncConfig};
use std::net::SocketAddr;
use std::time::Instant;
use tracing::{info, warn};

/// A connection pending mod handshake completion
pub struct PendingModConnection {
    /// Client address
    pub addr: SocketAddr,
    /// Requested player name
    pub name: String,
    /// Assigned display name (after validation)
    pub display_name: String,
    /// Assigned team
    pub team: TeamId,
    /// Session ID
    pub session_id: SessionId,
    /// Mod sync session
    pub sync_session: SyncSession,
    /// When this pending connection was created
    pub created_at: Instant,
}

impl PendingModConnection {
    /// Create a new pending connection
    pub fn new(
        addr: SocketAddr,
        name: String,
        display_name: String,
        team: TeamId,
        session_id: SessionId,
        join_policy: JoinPolicy,
        sync_config: SyncConfig,
        mods_with_payloads: Vec<ModInfo>,
    ) -> Self {
        let sync_session = SyncSession::new(join_policy, sync_config, mods_with_payloads);

        Self {
            addr,
            name,
            display_name,
            team,
            session_id,
            sync_session,
            created_at: Instant::now(),
        }
    }

    /// Handle receiving ModSetResponse from client
    /// Returns the JoinDecision to send back
    pub fn handle_response(&mut self, response: ModSetResponse) -> JoinDecision {
        info!(
            addr = %self.addr,
            supports_sync = response.supports_sync,
            cached_count = response.cached_hashes.len(),
            "Received mod set response"
        );

        self.sync_session.handle_response(response)
    }

    /// Handle PayloadAck from client
    pub fn handle_payload_ack(&mut self, ack: PayloadAck) {
        self.sync_session.handle_ack(ack);
    }

    /// Handle client confirming payload complete
    pub fn handle_payload_complete(&mut self, mod_id: &str, success: bool, error: Option<&str>) {
        self.sync_session
            .handle_payload_complete(mod_id, success, error);
    }

    /// Check if handshake is complete (ready to join game)
    pub fn is_ready_to_join(&self) -> bool {
        self.sync_session.state() == SyncState::Complete
    }

    /// Check if handshake failed
    pub fn is_failed(&self) -> bool {
        self.sync_session.state() == SyncState::Failed
    }

    /// Check if connection has timed out (30 second default)
    pub fn is_timed_out(&self) -> bool {
        self.created_at.elapsed().as_secs() > 30
    }

    /// Get sync session state
    pub fn sync_state(&self) -> SyncState {
        self.sync_session.state()
    }

    /// Get failure info if failed
    pub fn failure_info(
        &self,
    ) -> Option<(plix_common::protocol::messages::ModSyncRejectReason, String)> {
        self.sync_session.failure_info()
    }
}
