//! Auto-retry logic for matchmaking.

use std::collections::HashSet;

/// Maximum connection attempts before giving up.
pub const MAX_ATTEMPTS: u8 = 3;

/// Retry state for quick join attempts.
///
/// Tracks failed servers and attempt count to manage auto-retry behavior.
#[derive(Debug, Clone)]
pub struct RetryState {
    /// Server IDs that have already failed.
    failed_servers: HashSet<String>,
    /// Number of attempts made.
    attempts: u8,
}

impl Default for RetryState {
    fn default() -> Self {
        Self::new()
    }
}

impl RetryState {
    /// Create a new retry state.
    pub fn new() -> Self {
        Self {
            failed_servers: HashSet::new(),
            attempts: 0,
        }
    }

    /// Get the set of failed server IDs.
    pub fn failed_servers(&self) -> &HashSet<String> {
        &self.failed_servers
    }

    /// Get the number of attempts made.
    pub fn attempts(&self) -> u8 {
        self.attempts
    }

    /// Mark a server as failed.
    pub fn mark_failed(&mut self, server_id: impl Into<String>) {
        self.failed_servers.insert(server_id.into());
    }

    /// Check if a server has failed.
    pub fn is_failed(&self, server_id: &str) -> bool {
        self.failed_servers.contains(server_id)
    }

    /// Check if another attempt can be made.
    pub fn can_retry(&self) -> bool {
        self.attempts < MAX_ATTEMPTS
    }

    /// Increment attempt counter.
    pub fn increment_attempt(&mut self) {
        self.attempts = self.attempts.saturating_add(1);
    }

    /// Get remaining attempts.
    pub fn remaining_attempts(&self) -> u8 {
        MAX_ATTEMPTS.saturating_sub(self.attempts)
    }

    /// Reset the retry state for a new quick join session.
    pub fn reset(&mut self) {
        self.failed_servers.clear();
        self.attempts = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let state = RetryState::new();
        assert_eq!(state.attempts(), 0);
        assert!(state.failed_servers().is_empty());
        assert!(state.can_retry());
    }

    #[test]
    fn test_mark_failed() {
        let mut state = RetryState::new();
        state.mark_failed("server1");
        state.mark_failed("server2");

        assert!(state.is_failed("server1"));
        assert!(state.is_failed("server2"));
        assert!(!state.is_failed("server3"));
    }

    #[test]
    fn test_increment_attempt() {
        let mut state = RetryState::new();

        state.increment_attempt();
        assert_eq!(state.attempts(), 1);
        assert!(state.can_retry());

        state.increment_attempt();
        assert_eq!(state.attempts(), 2);
        assert!(state.can_retry());

        state.increment_attempt();
        assert_eq!(state.attempts(), 3);
        assert!(!state.can_retry()); // MAX_ATTEMPTS reached
    }

    #[test]
    fn test_remaining_attempts() {
        let mut state = RetryState::new();
        assert_eq!(state.remaining_attempts(), 3);

        state.increment_attempt();
        assert_eq!(state.remaining_attempts(), 2);

        state.increment_attempt();
        assert_eq!(state.remaining_attempts(), 1);

        state.increment_attempt();
        assert_eq!(state.remaining_attempts(), 0);
    }

    #[test]
    fn test_reset() {
        let mut state = RetryState::new();
        state.mark_failed("server1");
        state.increment_attempt();
        state.increment_attempt();

        state.reset();

        assert_eq!(state.attempts(), 0);
        assert!(state.failed_servers().is_empty());
        assert!(state.can_retry());
    }

    #[test]
    fn test_can_retry_at_boundary() {
        let mut state = RetryState::new();

        // At 2 attempts, can still retry (one more allowed)
        state.increment_attempt();
        state.increment_attempt();
        assert!(state.can_retry());

        // At 3 attempts, cannot retry
        state.increment_attempt();
        assert!(!state.can_retry());
    }
}
