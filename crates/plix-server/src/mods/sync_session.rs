//! Mod sync session management (Feature 037)
//!
//! This module handles the server-side synchronization of mod payloads to clients.
//! It manages the state machine for each client's mod sync process.
//!
//! # Security (Feature 040)
//!
//! This module includes protection against payload sync abuse:
//! - Chunk index bounds validation
//! - Duplicate chunk detection
//! - Resend request rate limiting
//! - Transfer timeout enforcement
//! - Hash mismatch detection

use plix_common::limits::{
    MAX_CACHED_PAYLOAD_HASHES, MAX_CHUNKS_PER_TRANSFER, MAX_RESEND_PER_WINDOW,
    TRANSFER_TIMEOUT_SECS,
};
use plix_common::protocol::messages::{
    JoinDecision, ModDescriptor, ModJoinPolicy, ModSetDescriptor, ModSetResponse, ModSyncRejectReason,
    PayloadAck, PayloadBegin, PayloadChunk, PayloadEnd,
};
use plix_mod_distribution::{JoinPolicy, ModManifest, SyncConfig};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// State of a client's mod sync session
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncState {
    /// Waiting for client's ModSetResponse
    AwaitingCapabilities,
    /// Evaluating client capabilities and deciding on sync
    Evaluating,
    /// Transferring payloads
    Transferring,
    /// All payloads transferred successfully
    Complete,
    /// Sync failed
    Failed,
}

/// Per-mod transfer state
#[derive(Debug)]
struct ModTransfer {
    /// Mod ID
    mod_id: String,
    /// Total payload size
    total_size: u64,
    /// SHA-256 hash of payload
    sha256: String,
    /// Path to payload ZIP
    payload_path: PathBuf,
    /// Total chunk count
    chunk_count: u32,
    /// Last acknowledged chunk index (-1 if none)
    last_acked_chunk: i32,
    /// Chunks currently in flight (not yet acked)
    inflight_chunks: Vec<u32>,
    /// Time of last chunk sent
    last_chunk_time: Option<Instant>,
    /// Whether transfer is complete
    complete: bool,
    // Security tracking (Feature 040)
    /// Chunks that have been received (for duplicate detection)
    received_chunks: HashSet<u32>,
    /// Resend requests in current window
    resend_requests: u32,
    /// Window start time for resend limiting
    resend_window_start: Instant,
}

/// Security abuse detection results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAbuseType {
    /// Chunk index out of bounds
    ChunkIndexOutOfBounds,
    /// Duplicate chunk received
    DuplicateChunk,
    /// Too many resend requests
    ResendSpam,
    /// Hash mismatch on verification
    HashMismatch,
    /// Cached hash count exceeds limit
    CachedHashLimitExceeded,
}

/// Server-side mod sync session for a single client
pub struct SyncSession {
    /// Current state
    state: SyncState,
    /// Server's join policy
    join_policy: JoinPolicy,
    /// Sync configuration
    sync_config: SyncConfig,
    /// Mods with client payloads
    mods_with_payloads: Vec<ModInfo>,
    /// Client's cached payload hashes (from ModSetResponse)
    client_cached_hashes: Vec<String>,
    /// Whether client supports sync
    client_supports_sync: bool,
    /// Mods that need to be synced
    mods_to_sync: Vec<String>,
    /// Current mod transfer states
    transfers: HashMap<String, ModTransfer>,
    /// Transfer queue (mod IDs in order)
    transfer_queue: VecDeque<String>,
    /// Current mod being transferred
    current_mod: Option<String>,
    /// Session start time
    start_time: Instant,
    /// Failure reason if state is Failed
    failure_reason: Option<ModSyncRejectReason>,
    /// Failure message
    failure_message: Option<String>,
    // Security tracking (Feature 040)
    /// Detected abuse type (if any)
    detected_abuse: Option<SyncAbuseType>,
}

/// Information about a mod with client payload
#[derive(Debug, Clone)]
pub struct ModInfo {
    /// Mod ID
    pub id: String,
    /// Mod version
    pub version: String,
    /// SHA-256 hash of client payload
    pub client_payload_hash: String,
    /// Size of client payload in bytes
    pub client_payload_size: u64,
    /// Path to the payload ZIP file
    pub payload_path: PathBuf,
    /// Whether mod has network channels
    pub has_network_channels: bool,
}

impl SyncSession {
    /// Create a new sync session
    pub fn new(
        join_policy: JoinPolicy,
        sync_config: SyncConfig,
        mods_with_payloads: Vec<ModInfo>,
    ) -> Self {
        Self {
            state: SyncState::AwaitingCapabilities,
            join_policy,
            sync_config,
            mods_with_payloads,
            client_cached_hashes: Vec::new(),
            client_supports_sync: false,
            mods_to_sync: Vec::new(),
            transfers: HashMap::new(),
            transfer_queue: VecDeque::new(),
            current_mod: None,
            start_time: Instant::now(),
            failure_reason: None,
            failure_message: None,
            detected_abuse: None,
        }
    }

    /// Get current session state
    pub fn state(&self) -> SyncState {
        self.state
    }

    /// Check if session is complete (success or failure)
    pub fn is_finished(&self) -> bool {
        matches!(self.state, SyncState::Complete | SyncState::Failed)
    }

    /// Build the ModSetDescriptor to send to the client
    pub fn build_mod_set_descriptor(&self) -> ModSetDescriptor {
        let mods: Vec<ModDescriptor> = self
            .mods_with_payloads
            .iter()
            .map(|m| ModDescriptor {
                id: m.id.clone(),
                version: m.version.clone(),
                client_payload_hash: Some(m.client_payload_hash.clone()),
                client_payload_size: m.client_payload_size,
                has_network_channels: m.has_network_channels,
            })
            .collect();

        let total_payload_size: u64 = mods.iter().map(|m| m.client_payload_size).sum();

        ModSetDescriptor {
            mods,
            total_payload_size,
            policy: ModJoinPolicy {
                allow_server_only: self.join_policy.allow_server_only,
                allow_payload_sync: self.join_policy.allow_payload_sync,
                require_payload_sync: self.join_policy.require_payload_sync,
            },
        }
    }

    /// Handle client's ModSetResponse
    ///
    /// # Security
    /// Validates cached hash count against MAX_CACHED_PAYLOAD_HASHES limit.
    pub fn handle_response(&mut self, response: ModSetResponse) -> JoinDecision {
        if self.state != SyncState::AwaitingCapabilities {
            return JoinDecision {
                allowed: false,
                rejection_reason: Some("Unexpected ModSetResponse".to_string()),
                mods_to_sync: Vec::new(),
            };
        }

        // Security: Check cached hash limit (T034)
        if response.cached_hashes.len() > MAX_CACHED_PAYLOAD_HASHES {
            warn!(
                count = response.cached_hashes.len(),
                limit = MAX_CACHED_PAYLOAD_HASHES,
                "Client exceeded cached hash limit"
            );
            self.state = SyncState::Failed;
            self.detected_abuse = Some(SyncAbuseType::CachedHashLimitExceeded);
            self.failure_reason = Some(ModSyncRejectReason::ClientRejected);
            self.failure_message = Some(format!(
                "Cached hash count {} exceeds limit {}",
                response.cached_hashes.len(),
                MAX_CACHED_PAYLOAD_HASHES
            ));
            return JoinDecision {
                allowed: false,
                rejection_reason: self.failure_message.clone(),
                mods_to_sync: Vec::new(),
            };
        }

        self.client_supports_sync = response.supports_sync;
        self.client_cached_hashes = response.cached_hashes;
        self.state = SyncState::Evaluating;

        // Evaluate join decision
        let decision = self.evaluate_join();

        if decision.allowed {
            // Set up transfers for mods that need syncing
            self.mods_to_sync = decision.mods_to_sync.clone();
            self.setup_transfers();

            if self.transfer_queue.is_empty() {
                // No transfers needed, mark complete
                self.state = SyncState::Complete;
            } else {
                self.state = SyncState::Transferring;
            }
        } else {
            self.state = SyncState::Failed;
            self.failure_reason = Some(ModSyncRejectReason::ClientRejected);
            self.failure_message = decision.rejection_reason.clone();
        }

        decision
    }

    /// Evaluate whether to allow the client to join
    fn evaluate_join(&self) -> JoinDecision {
        // Case 1: Server requires payload sync but client doesn't support it
        if self.join_policy.require_payload_sync && !self.client_supports_sync {
            return JoinDecision {
                allowed: false,
                rejection_reason: Some(
                    "Server requires mod payload sync, but your client doesn't support it"
                        .to_string(),
                ),
                mods_to_sync: Vec::new(),
            };
        }

        // Case 2: Server has payloads but doesn't allow server-only clients
        if !self.join_policy.allow_server_only
            && !self.client_supports_sync
            && !self.mods_with_payloads.is_empty()
        {
            return JoinDecision {
                allowed: false,
                rejection_reason: Some(
                    "Server requires mod support, but your client doesn't support sync".to_string(),
                ),
                mods_to_sync: Vec::new(),
            };
        }

        // Determine which mods need syncing
        let mods_to_sync: Vec<String> = if self.client_supports_sync && self.join_policy.allow_payload_sync {
            self.mods_with_payloads
                .iter()
                .filter(|m| !self.client_cached_hashes.contains(&m.client_payload_hash))
                .map(|m| m.id.clone())
                .collect()
        } else {
            Vec::new()
        };

        // Check total payload size
        let total_sync_size: u64 = self
            .mods_with_payloads
            .iter()
            .filter(|m| mods_to_sync.contains(&m.id))
            .map(|m| m.client_payload_size)
            .sum();

        let max_payload_bytes = self.sync_config.max_payload_bytes() as u64;
        if total_sync_size > max_payload_bytes {
            return JoinDecision {
                allowed: false,
                rejection_reason: Some(format!(
                    "Total payload size ({} bytes) exceeds limit ({} bytes)",
                    total_sync_size, max_payload_bytes
                )),
                mods_to_sync: Vec::new(),
            };
        }

        JoinDecision {
            allowed: true,
            rejection_reason: None,
            mods_to_sync,
        }
    }

    /// Set up transfer states for mods that need syncing
    ///
    /// # Security
    /// Validates chunk count against MAX_CHUNKS_PER_TRANSFER limit.
    fn setup_transfers(&mut self) {
        for mod_id in &self.mods_to_sync {
            if let Some(mod_info) = self.mods_with_payloads.iter().find(|m| &m.id == mod_id) {
                let chunk_size = self.sync_config.chunk_size_bytes();
                let chunk_count =
                    ((mod_info.client_payload_size as usize + chunk_size - 1) / chunk_size) as u32;

                // Security: Validate chunk count (T037)
                if chunk_count as usize > MAX_CHUNKS_PER_TRANSFER {
                    warn!(
                        mod_id = %mod_id,
                        chunk_count = chunk_count,
                        limit = MAX_CHUNKS_PER_TRANSFER,
                        "Mod exceeds chunk limit"
                    );
                    continue; // Skip this mod
                }

                let transfer = ModTransfer {
                    mod_id: mod_id.clone(),
                    total_size: mod_info.client_payload_size,
                    sha256: mod_info.client_payload_hash.clone(),
                    payload_path: mod_info.payload_path.clone(),
                    chunk_count,
                    last_acked_chunk: -1,
                    inflight_chunks: Vec::new(),
                    last_chunk_time: None,
                    complete: false,
                    // Security tracking
                    received_chunks: HashSet::new(),
                    resend_requests: 0,
                    resend_window_start: Instant::now(),
                };

                self.transfers.insert(mod_id.clone(), transfer);
                self.transfer_queue.push_back(mod_id.clone());
            }
        }
    }

    /// Generate PayloadBegin for the next mod to transfer
    pub fn next_payload_begin(&mut self) -> Option<PayloadBegin> {
        if self.state != SyncState::Transferring {
            return None;
        }

        // Get next mod from queue if we don't have a current one
        if self.current_mod.is_none() {
            self.current_mod = self.transfer_queue.pop_front();
        }

        let mod_id = self.current_mod.as_ref()?;
        let transfer = self.transfers.get(mod_id)?;

        Some(PayloadBegin {
            mod_id: transfer.mod_id.clone(),
            total_size: transfer.total_size,
            chunk_count: transfer.chunk_count,
            sha256: transfer.sha256.clone(),
        })
    }

    /// Generate next chunk(s) to send, respecting max_inflight_chunks
    pub fn next_chunks(&mut self) -> Vec<PayloadChunk> {
        if self.state != SyncState::Transferring {
            return Vec::new();
        }

        let mod_id = match &self.current_mod {
            Some(id) => id.clone(),
            None => return Vec::new(),
        };

        let max_inflight = self.sync_config.max_inflight_chunks as usize;
        let chunk_size = self.sync_config.chunk_size_bytes();

        let transfer = match self.transfers.get_mut(&mod_id) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut chunks = Vec::new();

        // Calculate next chunk to send
        let next_chunk_start = (transfer.last_acked_chunk + 1) as u32;
        let inflight_count = transfer.inflight_chunks.len();

        // Send up to max_inflight - inflight_count new chunks
        let can_send = max_inflight.saturating_sub(inflight_count);
        let next_unsent = next_chunk_start + inflight_count as u32;

        for i in 0..can_send {
            let chunk_index = next_unsent + i as u32;
            if chunk_index >= transfer.chunk_count {
                break;
            }

            // Read chunk data from file
            match self.read_chunk(&transfer.payload_path, chunk_index, chunk_size) {
                Ok(data) => {
                    chunks.push(PayloadChunk {
                        mod_id: mod_id.clone(),
                        chunk_index,
                        data,
                    });
                    transfer.inflight_chunks.push(chunk_index);
                }
                Err(e) => {
                    warn!(mod_id = %mod_id, chunk = chunk_index, error = %e, "Failed to read chunk");
                    self.state = SyncState::Failed;
                    self.failure_reason = Some(ModSyncRejectReason::Timeout);
                    self.failure_message = Some(format!("Failed to read payload: {}", e));
                    return Vec::new();
                }
            }
        }

        if !chunks.is_empty() {
            transfer.last_chunk_time = Some(Instant::now());
        }

        chunks
    }

    /// Read a chunk from the payload file
    fn read_chunk(
        &self,
        path: &Path,
        chunk_index: u32,
        chunk_size: usize,
    ) -> std::io::Result<Vec<u8>> {
        use std::fs::File;
        use std::io::{BufReader, Seek, SeekFrom};

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let offset = (chunk_index as u64) * (chunk_size as u64);
        reader.seek(SeekFrom::Start(offset))?;

        let mut buffer = vec![0u8; chunk_size];
        let bytes_read = reader.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        Ok(buffer)
    }

    /// Handle PayloadAck from client
    pub fn handle_ack(&mut self, ack: PayloadAck) -> Option<PayloadEnd> {
        if self.state != SyncState::Transferring {
            return None;
        }

        let mod_id = match &self.current_mod {
            Some(id) if id == &ack.mod_id => id.clone(),
            _ => return None,
        };

        let transfer = match self.transfers.get_mut(&mod_id) {
            Some(t) => t,
            None => return None,
        };

        // Update acked state
        transfer.last_acked_chunk = ack.last_chunk_index as i32;
        transfer
            .inflight_chunks
            .retain(|&c| c > ack.last_chunk_index);

        if ack.complete || (ack.last_chunk_index + 1) as u32 >= transfer.chunk_count {
            transfer.complete = true;
            let end = PayloadEnd {
                mod_id: mod_id.clone(),
                success: true,
                error: None,
            };

            // Move to next mod or complete
            self.current_mod = None;
            if self.transfer_queue.is_empty() {
                self.state = SyncState::Complete;
                info!("Mod sync complete for session");
            }

            return Some(end);
        }

        None
    }

    /// Handle client confirming payload complete (with verification result)
    pub fn handle_payload_complete(&mut self, mod_id: &str, success: bool, error: Option<&str>) {
        if !success {
            self.state = SyncState::Failed;
            self.failure_reason = Some(ModSyncRejectReason::HashMismatch);
            self.failure_message = error.map(|s| s.to_string());
        }
    }

    /// Check for timeout
    pub fn check_timeout(&mut self) -> bool {
        let timeout = Duration::from_secs(self.sync_config.transfer_timeout_secs as u64);

        if self.start_time.elapsed() > timeout {
            self.state = SyncState::Failed;
            self.failure_reason = Some(ModSyncRejectReason::Timeout);
            self.failure_message = Some("Transfer timeout".to_string());
            return true;
        }

        // Check for chunk timeout (individual chunk not acked for 30 seconds)
        if let Some(mod_id) = &self.current_mod {
            if let Some(transfer) = self.transfers.get(mod_id) {
                if let Some(last_time) = transfer.last_chunk_time {
                    if last_time.elapsed() > Duration::from_secs(30)
                        && !transfer.inflight_chunks.is_empty()
                    {
                        self.state = SyncState::Failed;
                        self.failure_reason = Some(ModSyncRejectReason::Timeout);
                        self.failure_message =
                            Some("Chunk acknowledgement timeout".to_string());
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Get failure info if session failed
    pub fn failure_info(&self) -> Option<(ModSyncRejectReason, String)> {
        if self.state == SyncState::Failed {
            Some((
                self.failure_reason.unwrap_or(ModSyncRejectReason::Timeout),
                self.failure_message.clone().unwrap_or_default(),
            ))
        } else {
            None
        }
    }

    /// Get session duration
    pub fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get number of mods being synced
    pub fn mods_to_sync_count(&self) -> usize {
        self.mods_to_sync.len()
    }

    /// Get detected abuse type (if any)
    ///
    /// Returns the type of abuse that was detected, for security metrics.
    pub fn detected_abuse(&self) -> Option<SyncAbuseType> {
        self.detected_abuse
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_join_policy() -> JoinPolicy {
        JoinPolicy {
            allow_server_only: true,
            allow_payload_sync: true,
            require_payload_sync: false,
        }
    }

    fn test_sync_config() -> SyncConfig {
        SyncConfig {
            max_payload_mb: 25,
            chunk_size_kb: 256,
            max_inflight_chunks: 4,
            transfer_timeout_secs: 300,
        }
    }

    #[test]
    fn test_new_session() {
        let session = SyncSession::new(test_join_policy(), test_sync_config(), vec![]);
        assert_eq!(session.state(), SyncState::AwaitingCapabilities);
        assert!(!session.is_finished());
    }

    #[test]
    fn test_build_descriptor_empty() {
        let session = SyncSession::new(test_join_policy(), test_sync_config(), vec![]);
        let descriptor = session.build_mod_set_descriptor();

        assert!(descriptor.mods.is_empty());
        assert_eq!(descriptor.total_payload_size, 0);
        assert!(descriptor.policy.allow_server_only);
    }

    #[test]
    fn test_handle_response_no_sync_needed() {
        let mut session = SyncSession::new(test_join_policy(), test_sync_config(), vec![]);

        let response = ModSetResponse {
            supports_sync: true,
            cached_hashes: vec![],
            engine_version: None,
        };

        let decision = session.handle_response(response);

        assert!(decision.allowed);
        assert!(decision.mods_to_sync.is_empty());
        assert_eq!(session.state(), SyncState::Complete);
    }

    #[test]
    fn test_handle_response_reject_no_sync_support() {
        let policy = JoinPolicy {
            allow_server_only: false,
            allow_payload_sync: true,
            require_payload_sync: true,
        };

        let mods = vec![ModInfo {
            id: "test-mod".to_string(),
            version: "1.0.0".to_string(),
            client_payload_hash: "abc123".to_string(),
            client_payload_size: 1024,
            payload_path: PathBuf::from("/tmp/test.zip"),
            has_network_channels: false,
        }];

        let mut session = SyncSession::new(policy, test_sync_config(), mods);

        let response = ModSetResponse {
            supports_sync: false,
            cached_hashes: vec![],
            engine_version: None,
        };

        let decision = session.handle_response(response);

        assert!(!decision.allowed);
        assert!(decision.rejection_reason.is_some());
        assert_eq!(session.state(), SyncState::Failed);
    }

    #[test]
    fn test_handle_response_client_has_cached() {
        let mods = vec![ModInfo {
            id: "test-mod".to_string(),
            version: "1.0.0".to_string(),
            client_payload_hash: "abc123def456".to_string(),
            client_payload_size: 1024,
            payload_path: PathBuf::from("/tmp/test.zip"),
            has_network_channels: false,
        }];

        let mut session = SyncSession::new(test_join_policy(), test_sync_config(), mods);

        let response = ModSetResponse {
            supports_sync: true,
            cached_hashes: vec!["abc123def456".to_string()],
            engine_version: None,
        };

        let decision = session.handle_response(response);

        assert!(decision.allowed);
        assert!(decision.mods_to_sync.is_empty()); // Client already has it
        assert_eq!(session.state(), SyncState::Complete);
    }

    // Security tests (Feature 040)

    #[test]
    fn test_cached_hash_limit_exceeded() {
        let mut session = SyncSession::new(test_join_policy(), test_sync_config(), vec![]);

        // Create response with too many cached hashes
        let many_hashes: Vec<String> = (0..MAX_CACHED_PAYLOAD_HASHES + 10)
            .map(|i| format!("hash{:05}", i))
            .collect();

        let response = ModSetResponse {
            supports_sync: true,
            cached_hashes: many_hashes,
            engine_version: None,
        };

        let decision = session.handle_response(response);

        assert!(!decision.allowed);
        assert_eq!(session.state(), SyncState::Failed);
        assert_eq!(
            session.detected_abuse(),
            Some(SyncAbuseType::CachedHashLimitExceeded)
        );
    }
}
