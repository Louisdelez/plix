//! Strike Tracking - Violation-based disconnect system
//!
//! Tracks accumulated violations per connection and triggers disconnect
//! when the threshold is reached.

use std::collections::HashMap;
use std::time::Instant;

use plix_common::limits::{STRIKES_BEFORE_DISCONNECT, STRIKES_DECAY_SECS};

/// Categories of security violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrikeCategory {
    /// Invalid message format (decode failure)
    DecodeError,
    /// Message exceeds size limits
    SizeViolation,
    /// Rate limit exceeded
    RateExceeded,
    /// Handshake protocol violation
    HandshakeAbuse,
    /// Payload sync protocol violation
    PayloadSyncAbuse,
    /// Hash verification failed
    HashMismatch,
    /// Parser limit exceeded
    ParserAbuse,
}

impl StrikeCategory {
    /// Get a human-readable description of this category
    pub fn description(&self) -> &'static str {
        match self {
            StrikeCategory::DecodeError => "invalid message format",
            StrikeCategory::SizeViolation => "message too large",
            StrikeCategory::RateExceeded => "rate limit exceeded",
            StrikeCategory::HandshakeAbuse => "handshake protocol violation",
            StrikeCategory::PayloadSyncAbuse => "payload sync abuse",
            StrikeCategory::HashMismatch => "hash verification failed",
            StrikeCategory::ParserAbuse => "parser limit exceeded",
        }
    }
}

/// Per-connection strike tracker
///
/// Accumulates violations and triggers disconnect when threshold is reached.
/// Strikes decay over time to allow recovery from transient issues.
#[derive(Debug)]
pub struct StrikeTracker {
    connection_id: u64,
    strikes: u32,
    last_strike_time: Option<Instant>,
    categories: HashMap<StrikeCategory, u32>,
    disconnect_triggered: bool,
    disconnect_reason: Option<String>,
}

impl StrikeTracker {
    /// Create a new strike tracker for a connection
    pub fn new(connection_id: u64) -> Self {
        Self {
            connection_id,
            strikes: 0,
            last_strike_time: None,
            categories: HashMap::new(),
            disconnect_triggered: false,
            disconnect_reason: None,
        }
    }

    /// Record a strike in the given category.
    ///
    /// Returns `true` if the connection should be disconnected (threshold reached).
    pub fn record_strike(&mut self, category: StrikeCategory) -> bool {
        // Once disconnect is triggered, it stays triggered
        if self.disconnect_triggered {
            return true;
        }

        self.strikes += 1;
        self.last_strike_time = Some(Instant::now());
        *self.categories.entry(category).or_insert(0) += 1;

        if self.strikes >= STRIKES_BEFORE_DISCONNECT {
            self.disconnect_triggered = true;
            self.disconnect_reason = Some(format!(
                "Too many violations: {} ({})",
                category.description(),
                self.strikes
            ));
            true
        } else {
            false
        }
    }

    /// Apply time-based strike decay.
    ///
    /// Call this periodically (e.g., once per second) to allow recovery.
    pub fn decay(&mut self) {
        if self.disconnect_triggered {
            return;
        }

        if let Some(last_time) = self.last_strike_time {
            let elapsed = last_time.elapsed().as_secs();
            if elapsed >= STRIKES_DECAY_SECS as u64 && self.strikes > 0 {
                self.strikes = self.strikes.saturating_sub(1);
                self.last_strike_time = Some(Instant::now());
            }
        }
    }

    /// Get current strike count
    pub fn get_strikes(&self) -> u32 {
        self.strikes
    }

    /// Get strikes broken down by category
    pub fn get_strikes_by_category(&self) -> &HashMap<StrikeCategory, u32> {
        &self.categories
    }

    /// Check if connection should be disconnected
    pub fn should_disconnect(&self) -> bool {
        self.disconnect_triggered
    }

    /// Get disconnect reason if should_disconnect is true
    pub fn disconnect_reason(&self) -> Option<&str> {
        self.disconnect_reason.as_deref()
    }

    /// Get the connection ID this tracker is associated with
    pub fn connection_id(&self) -> u64 {
        self.connection_id
    }

    /// Reset the tracker (for testing)
    #[cfg(test)]
    pub fn reset(&mut self) {
        self.strikes = 0;
        self.last_strike_time = None;
        self.categories.clear();
        self.disconnect_triggered = false;
        self.disconnect_reason = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tracker_has_zero_strikes() {
        let tracker = StrikeTracker::new(1);
        assert_eq!(tracker.get_strikes(), 0);
        assert!(!tracker.should_disconnect());
    }

    #[test]
    fn test_record_strike_increments_count() {
        let mut tracker = StrikeTracker::new(1);
        tracker.record_strike(StrikeCategory::DecodeError);
        assert_eq!(tracker.get_strikes(), 1);
    }

    #[test]
    fn test_record_strike_tracks_category() {
        let mut tracker = StrikeTracker::new(1);
        tracker.record_strike(StrikeCategory::DecodeError);
        tracker.record_strike(StrikeCategory::DecodeError);
        tracker.record_strike(StrikeCategory::RateExceeded);

        let by_category = tracker.get_strikes_by_category();
        assert_eq!(by_category.get(&StrikeCategory::DecodeError), Some(&2));
        assert_eq!(by_category.get(&StrikeCategory::RateExceeded), Some(&1));
    }

    #[test]
    fn test_threshold_triggers_disconnect() {
        let mut tracker = StrikeTracker::new(1);

        // Record strikes up to threshold - 1
        for _ in 0..STRIKES_BEFORE_DISCONNECT - 1 {
            assert!(!tracker.record_strike(StrikeCategory::DecodeError));
        }
        assert!(!tracker.should_disconnect());

        // Next strike should trigger disconnect
        assert!(tracker.record_strike(StrikeCategory::DecodeError));
        assert!(tracker.should_disconnect());
    }

    #[test]
    fn test_disconnect_reason_set() {
        let mut tracker = StrikeTracker::new(1);

        for _ in 0..STRIKES_BEFORE_DISCONNECT {
            tracker.record_strike(StrikeCategory::SizeViolation);
        }

        let reason = tracker.disconnect_reason().unwrap();
        assert!(reason.contains("message too large"));
    }

    #[test]
    fn test_disconnect_stays_triggered() {
        let mut tracker = StrikeTracker::new(1);

        for _ in 0..STRIKES_BEFORE_DISCONNECT {
            tracker.record_strike(StrikeCategory::DecodeError);
        }

        // Additional strikes still return true
        assert!(tracker.record_strike(StrikeCategory::DecodeError));
        assert!(tracker.should_disconnect());
    }

    #[test]
    fn test_decay_does_not_increase_strikes() {
        let mut tracker = StrikeTracker::new(1);
        tracker.record_strike(StrikeCategory::DecodeError);
        let initial = tracker.get_strikes();

        tracker.decay();

        assert!(tracker.get_strikes() <= initial);
    }

    #[test]
    fn test_decay_does_not_affect_disconnected() {
        let mut tracker = StrikeTracker::new(1);

        for _ in 0..STRIKES_BEFORE_DISCONNECT {
            tracker.record_strike(StrikeCategory::DecodeError);
        }

        let strikes_before = tracker.get_strikes();
        tracker.decay();

        // Strikes unchanged after disconnect
        assert_eq!(tracker.get_strikes(), strikes_before);
        assert!(tracker.should_disconnect());
    }

    #[test]
    fn test_category_description() {
        assert_eq!(StrikeCategory::DecodeError.description(), "invalid message format");
        assert_eq!(StrikeCategory::SizeViolation.description(), "message too large");
        assert_eq!(StrikeCategory::RateExceeded.description(), "rate limit exceeded");
    }
}
