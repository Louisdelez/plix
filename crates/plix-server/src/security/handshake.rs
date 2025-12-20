//! Handshake Tracking - Pending connection management and timeout detection
//!
//! Tracks connections during the handshake phase and enforces timeouts
//! and per-source limits.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Instant;

use plix_common::limits::{HANDSHAKE_TIMEOUT_SECS, MAX_CACHED_PAYLOAD_HASHES, MAX_PENDING_PER_SOURCE};

/// Handshake state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeState {
    /// Waiting for Connect message
    AwaitingConnect,
    /// Waiting for ModSetResponse
    AwaitingModSet,
    /// Waiting for payload sync to complete
    AwaitingPayloadSync,
    /// Handshake complete
    Complete,
}

/// Errors that can occur during handshake tracking
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeError {
    /// Too many pending connections from this source
    TooManyPending,
    /// Invalid state transition attempted
    InvalidTransition {
        from: HandshakeState,
        to: HandshakeState,
    },
    /// Handshake timed out
    Timeout,
    /// Too many cached payload hashes
    CacheHashLimitExceeded { count: usize },
    /// Connection not found
    NotFound,
}

impl std::fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandshakeError::TooManyPending => {
                write!(f, "too many pending connections from this source")
            }
            HandshakeError::InvalidTransition { from, to } => {
                write!(f, "invalid state transition from {:?} to {:?}", from, to)
            }
            HandshakeError::Timeout => {
                write!(f, "handshake timeout")
            }
            HandshakeError::CacheHashLimitExceeded { count } => {
                write!(
                    f,
                    "cached hash count {} exceeds limit {}",
                    count, MAX_CACHED_PAYLOAD_HASHES
                )
            }
            HandshakeError::NotFound => {
                write!(f, "connection not found in handshake tracker")
            }
        }
    }
}

impl std::error::Error for HandshakeError {}

/// Pending handshake information
#[derive(Debug)]
struct PendingHandshake {
    connection_id: u64,
    source_addr: SocketAddr,
    start_time: Instant,
    state: HandshakeState,
    cached_hashes_count: usize,
}

/// Tracks pending handshakes for timeout and abuse detection
#[derive(Debug)]
pub struct HandshakeTracker {
    pending: HashMap<u64, PendingHandshake>,
    per_source_count: HashMap<SocketAddr, usize>,
}

impl Default for HandshakeTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl HandshakeTracker {
    /// Create a new handshake tracker
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
            per_source_count: HashMap::new(),
        }
    }

    /// Register a new pending handshake.
    ///
    /// Returns error if per-source limit is exceeded.
    pub fn register(
        &mut self,
        connection_id: u64,
        source_addr: SocketAddr,
    ) -> Result<(), HandshakeError> {
        // Check per-source limit
        let current = self.per_source_count.get(&source_addr).copied().unwrap_or(0);
        if current >= MAX_PENDING_PER_SOURCE {
            return Err(HandshakeError::TooManyPending);
        }

        // Add to tracking
        let pending = PendingHandshake {
            connection_id,
            source_addr,
            start_time: Instant::now(),
            state: HandshakeState::AwaitingConnect,
            cached_hashes_count: 0,
        };

        self.pending.insert(connection_id, pending);
        *self.per_source_count.entry(source_addr).or_insert(0) += 1;

        Ok(())
    }

    /// Update handshake state.
    ///
    /// Returns error if transition is invalid.
    pub fn update_state(
        &mut self,
        connection_id: u64,
        new_state: HandshakeState,
    ) -> Result<(), HandshakeError> {
        let pending = self
            .pending
            .get_mut(&connection_id)
            .ok_or(HandshakeError::NotFound)?;

        // Validate state transition
        let valid = match (pending.state, new_state) {
            (HandshakeState::AwaitingConnect, HandshakeState::AwaitingModSet) => true,
            (HandshakeState::AwaitingModSet, HandshakeState::AwaitingPayloadSync) => true,
            (HandshakeState::AwaitingModSet, HandshakeState::Complete) => true, // No sync needed
            (HandshakeState::AwaitingPayloadSync, HandshakeState::Complete) => true,
            _ => false,
        };

        if !valid {
            return Err(HandshakeError::InvalidTransition {
                from: pending.state,
                to: new_state,
            });
        }

        pending.state = new_state;
        Ok(())
    }

    /// Record cached hashes count from client.
    ///
    /// Returns error if limit is exceeded.
    pub fn set_cached_hashes_count(
        &mut self,
        connection_id: u64,
        count: usize,
    ) -> Result<(), HandshakeError> {
        if count > MAX_CACHED_PAYLOAD_HASHES {
            return Err(HandshakeError::CacheHashLimitExceeded { count });
        }

        let pending = self
            .pending
            .get_mut(&connection_id)
            .ok_or(HandshakeError::NotFound)?;

        pending.cached_hashes_count = count;
        Ok(())
    }

    /// Check all pending handshakes for timeout.
    ///
    /// Returns list of timed-out connection IDs.
    pub fn check_timeouts(&mut self) -> Vec<u64> {
        let timeout = std::time::Duration::from_secs(HANDSHAKE_TIMEOUT_SECS as u64);
        let now = Instant::now();

        let timed_out: Vec<u64> = self
            .pending
            .iter()
            .filter(|(_, p)| now.duration_since(p.start_time) >= timeout)
            .map(|(id, _)| *id)
            .collect();

        // Remove timed-out connections
        for id in &timed_out {
            self.abort(*id);
        }

        timed_out
    }

    /// Mark handshake as complete and remove from tracking.
    pub fn complete(&mut self, connection_id: u64) {
        self.remove(connection_id);
    }

    /// Abort handshake and remove from tracking.
    pub fn abort(&mut self, connection_id: u64) {
        self.remove(connection_id);
    }

    /// Get current pending count for a source.
    pub fn pending_count(&self, source_addr: &SocketAddr) -> usize {
        self.per_source_count.get(source_addr).copied().unwrap_or(0)
    }

    /// Get total pending handshakes.
    pub fn total_pending(&self) -> usize {
        self.pending.len()
    }

    /// Get the state of a connection
    pub fn get_state(&self, connection_id: u64) -> Option<HandshakeState> {
        self.pending.get(&connection_id).map(|p| p.state)
    }

    /// Remove a connection from tracking
    fn remove(&mut self, connection_id: u64) {
        if let Some(pending) = self.pending.remove(&connection_id) {
            if let Some(count) = self.per_source_count.get_mut(&pending.source_addr) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    self.per_source_count.remove(&pending.source_addr);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn test_addr() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12345)
    }

    #[test]
    fn test_register_success() {
        let mut tracker = HandshakeTracker::new();
        assert!(tracker.register(1, test_addr()).is_ok());
        assert_eq!(tracker.total_pending(), 1);
    }

    #[test]
    fn test_register_per_source_limit() {
        let mut tracker = HandshakeTracker::new();
        let addr = test_addr();

        // Fill up to limit
        for i in 0..MAX_PENDING_PER_SOURCE {
            assert!(tracker.register(i as u64, addr).is_ok());
        }

        // Next should fail
        let result = tracker.register(100, addr);
        assert!(matches!(result, Err(HandshakeError::TooManyPending)));
    }

    #[test]
    fn test_register_different_sources() {
        let mut tracker = HandshakeTracker::new();

        let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12345);
        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)), 12345);

        // Both sources can register up to limit
        for i in 0..MAX_PENDING_PER_SOURCE {
            assert!(tracker.register(i as u64, addr1).is_ok());
            assert!(tracker.register((i + 100) as u64, addr2).is_ok());
        }
    }

    #[test]
    fn test_update_state_valid() {
        let mut tracker = HandshakeTracker::new();
        tracker.register(1, test_addr()).unwrap();

        assert!(tracker
            .update_state(1, HandshakeState::AwaitingModSet)
            .is_ok());
        assert_eq!(
            tracker.get_state(1),
            Some(HandshakeState::AwaitingModSet)
        );
    }

    #[test]
    fn test_update_state_invalid() {
        let mut tracker = HandshakeTracker::new();
        tracker.register(1, test_addr()).unwrap();

        // Can't skip from AwaitingConnect to Complete
        let result = tracker.update_state(1, HandshakeState::Complete);
        assert!(matches!(
            result,
            Err(HandshakeError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn test_cached_hashes_within_limit() {
        let mut tracker = HandshakeTracker::new();
        tracker.register(1, test_addr()).unwrap();

        assert!(tracker.set_cached_hashes_count(1, 100).is_ok());
    }

    #[test]
    fn test_cached_hashes_exceeds_limit() {
        let mut tracker = HandshakeTracker::new();
        tracker.register(1, test_addr()).unwrap();

        let result = tracker.set_cached_hashes_count(1, MAX_CACHED_PAYLOAD_HASHES + 1);
        assert!(matches!(
            result,
            Err(HandshakeError::CacheHashLimitExceeded { .. })
        ));
    }

    #[test]
    fn test_complete_removes_from_tracking() {
        let mut tracker = HandshakeTracker::new();
        let addr = test_addr();
        tracker.register(1, addr).unwrap();

        tracker.complete(1);

        assert_eq!(tracker.total_pending(), 0);
        assert_eq!(tracker.pending_count(&addr), 0);
    }

    #[test]
    fn test_abort_removes_from_tracking() {
        let mut tracker = HandshakeTracker::new();
        let addr = test_addr();
        tracker.register(1, addr).unwrap();

        tracker.abort(1);

        assert_eq!(tracker.total_pending(), 0);
        assert_eq!(tracker.pending_count(&addr), 0);
    }

    #[test]
    fn test_pending_count_updates() {
        let mut tracker = HandshakeTracker::new();
        let addr = test_addr();

        tracker.register(1, addr).unwrap();
        assert_eq!(tracker.pending_count(&addr), 1);

        tracker.register(2, addr).unwrap();
        assert_eq!(tracker.pending_count(&addr), 2);

        tracker.complete(1);
        assert_eq!(tracker.pending_count(&addr), 1);

        tracker.complete(2);
        assert_eq!(tracker.pending_count(&addr), 0);
    }
}
