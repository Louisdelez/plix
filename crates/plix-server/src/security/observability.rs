//! Security Observability - Metrics and rate-limited logging
//!
//! Provides security monitoring through atomic counters and throttled logging.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use super::strikes::StrikeCategory;

/// Snapshot of security metrics
#[derive(Debug, Clone, Default)]
pub struct MetricsSnapshot {
    pub invalid_messages_total: u64,
    pub disconnects_strikes_total: u64,
    pub payload_sync_aborts_total: u64,
    pub registry_parse_failures_total: u64,
    pub rate_limited_total: u64,
    pub handshake_timeouts_total: u64,
    pub zip_safety_rejections_total: u64,
    pub strikes_by_category: HashMap<StrikeCategory, u64>,
}

/// Thread-safe security metrics with atomic counters
#[derive(Debug)]
pub struct SecurityMetrics {
    invalid_messages_total: AtomicU64,
    disconnects_strikes_total: AtomicU64,
    payload_sync_aborts_total: AtomicU64,
    registry_parse_failures_total: AtomicU64,
    rate_limited_total: AtomicU64,
    handshake_timeouts_total: AtomicU64,
    zip_safety_rejections_total: AtomicU64,
    // Category breakdown stored separately for simplicity
    strikes_decode_error: AtomicU64,
    strikes_size_violation: AtomicU64,
    strikes_rate_exceeded: AtomicU64,
    strikes_handshake_abuse: AtomicU64,
    strikes_payload_sync_abuse: AtomicU64,
    strikes_hash_mismatch: AtomicU64,
    strikes_parser_abuse: AtomicU64,
}

impl Default for SecurityMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityMetrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self {
            invalid_messages_total: AtomicU64::new(0),
            disconnects_strikes_total: AtomicU64::new(0),
            payload_sync_aborts_total: AtomicU64::new(0),
            registry_parse_failures_total: AtomicU64::new(0),
            rate_limited_total: AtomicU64::new(0),
            handshake_timeouts_total: AtomicU64::new(0),
            zip_safety_rejections_total: AtomicU64::new(0),
            strikes_decode_error: AtomicU64::new(0),
            strikes_size_violation: AtomicU64::new(0),
            strikes_rate_exceeded: AtomicU64::new(0),
            strikes_handshake_abuse: AtomicU64::new(0),
            strikes_payload_sync_abuse: AtomicU64::new(0),
            strikes_hash_mismatch: AtomicU64::new(0),
            strikes_parser_abuse: AtomicU64::new(0),
        }
    }

    /// Record an invalid message
    pub fn record_invalid_message(&self) {
        self.invalid_messages_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a disconnect due to strikes
    pub fn record_disconnect(&self, category: StrikeCategory) {
        self.disconnects_strikes_total.fetch_add(1, Ordering::Relaxed);
        self.increment_category(category);
    }

    /// Record a payload sync abort
    pub fn record_payload_sync_abort(&self) {
        self.payload_sync_aborts_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a registry parse failure
    pub fn record_registry_parse_failure(&self) {
        self.registry_parse_failures_total
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Record a rate-limited message
    pub fn record_rate_limited(&self) {
        self.rate_limited_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a handshake timeout
    pub fn record_handshake_timeout(&self) {
        self.handshake_timeouts_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a zip safety rejection
    pub fn record_zip_safety_rejection(&self) {
        self.zip_safety_rejections_total
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Get a snapshot of all metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let mut strikes_by_category = HashMap::new();
        strikes_by_category.insert(
            StrikeCategory::DecodeError,
            self.strikes_decode_error.load(Ordering::Relaxed),
        );
        strikes_by_category.insert(
            StrikeCategory::SizeViolation,
            self.strikes_size_violation.load(Ordering::Relaxed),
        );
        strikes_by_category.insert(
            StrikeCategory::RateExceeded,
            self.strikes_rate_exceeded.load(Ordering::Relaxed),
        );
        strikes_by_category.insert(
            StrikeCategory::HandshakeAbuse,
            self.strikes_handshake_abuse.load(Ordering::Relaxed),
        );
        strikes_by_category.insert(
            StrikeCategory::PayloadSyncAbuse,
            self.strikes_payload_sync_abuse.load(Ordering::Relaxed),
        );
        strikes_by_category.insert(
            StrikeCategory::HashMismatch,
            self.strikes_hash_mismatch.load(Ordering::Relaxed),
        );
        strikes_by_category.insert(
            StrikeCategory::ParserAbuse,
            self.strikes_parser_abuse.load(Ordering::Relaxed),
        );

        MetricsSnapshot {
            invalid_messages_total: self.invalid_messages_total.load(Ordering::Relaxed),
            disconnects_strikes_total: self.disconnects_strikes_total.load(Ordering::Relaxed),
            payload_sync_aborts_total: self.payload_sync_aborts_total.load(Ordering::Relaxed),
            registry_parse_failures_total: self
                .registry_parse_failures_total
                .load(Ordering::Relaxed),
            rate_limited_total: self.rate_limited_total.load(Ordering::Relaxed),
            handshake_timeouts_total: self.handshake_timeouts_total.load(Ordering::Relaxed),
            zip_safety_rejections_total: self.zip_safety_rejections_total.load(Ordering::Relaxed),
            strikes_by_category,
        }
    }

    /// Reset all counters (for testing)
    #[cfg(test)]
    pub fn reset(&self) {
        self.invalid_messages_total.store(0, Ordering::Relaxed);
        self.disconnects_strikes_total.store(0, Ordering::Relaxed);
        self.payload_sync_aborts_total.store(0, Ordering::Relaxed);
        self.registry_parse_failures_total.store(0, Ordering::Relaxed);
        self.rate_limited_total.store(0, Ordering::Relaxed);
        self.handshake_timeouts_total.store(0, Ordering::Relaxed);
        self.zip_safety_rejections_total.store(0, Ordering::Relaxed);
        self.strikes_decode_error.store(0, Ordering::Relaxed);
        self.strikes_size_violation.store(0, Ordering::Relaxed);
        self.strikes_rate_exceeded.store(0, Ordering::Relaxed);
        self.strikes_handshake_abuse.store(0, Ordering::Relaxed);
        self.strikes_payload_sync_abuse.store(0, Ordering::Relaxed);
        self.strikes_hash_mismatch.store(0, Ordering::Relaxed);
        self.strikes_parser_abuse.store(0, Ordering::Relaxed);
    }

    fn increment_category(&self, category: StrikeCategory) {
        match category {
            StrikeCategory::DecodeError => {
                self.strikes_decode_error.fetch_add(1, Ordering::Relaxed);
            }
            StrikeCategory::SizeViolation => {
                self.strikes_size_violation.fetch_add(1, Ordering::Relaxed);
            }
            StrikeCategory::RateExceeded => {
                self.strikes_rate_exceeded.fetch_add(1, Ordering::Relaxed);
            }
            StrikeCategory::HandshakeAbuse => {
                self.strikes_handshake_abuse.fetch_add(1, Ordering::Relaxed);
            }
            StrikeCategory::PayloadSyncAbuse => {
                self.strikes_payload_sync_abuse
                    .fetch_add(1, Ordering::Relaxed);
            }
            StrikeCategory::HashMismatch => {
                self.strikes_hash_mismatch.fetch_add(1, Ordering::Relaxed);
            }
            StrikeCategory::ParserAbuse => {
                self.strikes_parser_abuse.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// Rate-limited logger for security events
///
/// Prevents log spam by throttling similar messages.
#[derive(Debug)]
pub struct RateLimitedLogger {
    last_log: HashMap<String, Instant>,
    suppressed: HashMap<String, u64>,
    min_interval: Duration,
    debug_mode: bool,
}

impl Default for RateLimitedLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimitedLogger {
    /// Create with default interval (1 second)
    pub fn new() -> Self {
        Self {
            last_log: HashMap::new(),
            suppressed: HashMap::new(),
            min_interval: Duration::from_secs(1),
            debug_mode: false,
        }
    }

    /// Create with custom interval
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            last_log: HashMap::new(),
            suppressed: HashMap::new(),
            min_interval: interval,
            debug_mode: false,
        }
    }

    /// Enable debug mode for verbose logging
    pub fn with_debug_mode(mut self, enabled: bool) -> Self {
        self.debug_mode = enabled;
        self
    }

    /// Log a warning with rate limiting.
    ///
    /// Category is used for per-category throttling.
    pub fn warn(&mut self, category: &str, message: &str) {
        let now = Instant::now();

        if let Some(last) = self.last_log.get(category) {
            if now.duration_since(*last) < self.min_interval {
                *self.suppressed.entry(category.to_string()).or_insert(0) += 1;
                return;
            }
        }

        let suppressed = self.suppressed.remove(category).unwrap_or(0);
        if suppressed > 0 {
            tracing::warn!(
                "[security:{}] {} (suppressed {} similar)",
                category,
                message,
                suppressed
            );
        } else {
            tracing::warn!("[security:{}] {}", category, message);
        }

        self.last_log.insert(category.to_string(), now);
    }

    /// Log debug message (only if debug_mode enabled)
    pub fn debug(&mut self, category: &str, message: &str) {
        if self.debug_mode {
            tracing::debug!("[security:{}] {}", category, message);
        }
    }

    /// Log error (always logged, no rate limiting)
    pub fn error(&mut self, category: &str, message: &str) {
        tracing::error!("[security:{}] {}", category, message);
    }

    /// Get suppressed message count for a category
    pub fn suppressed_count(&self, category: &str) -> u64 {
        self.suppressed.get(category).copied().unwrap_or(0)
    }

    /// Reset all suppression state
    pub fn reset(&mut self) {
        self.last_log.clear();
        self.suppressed.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_new() {
        let metrics = SecurityMetrics::new();
        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.invalid_messages_total, 0);
        assert_eq!(snapshot.disconnects_strikes_total, 0);
    }

    #[test]
    fn test_metrics_record_invalid_message() {
        let metrics = SecurityMetrics::new();

        metrics.record_invalid_message();
        metrics.record_invalid_message();

        assert_eq!(metrics.snapshot().invalid_messages_total, 2);
    }

    #[test]
    fn test_metrics_record_disconnect() {
        let metrics = SecurityMetrics::new();

        metrics.record_disconnect(StrikeCategory::DecodeError);
        metrics.record_disconnect(StrikeCategory::RateExceeded);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.disconnects_strikes_total, 2);
        assert_eq!(
            snapshot.strikes_by_category.get(&StrikeCategory::DecodeError),
            Some(&1)
        );
        assert_eq!(
            snapshot.strikes_by_category.get(&StrikeCategory::RateExceeded),
            Some(&1)
        );
    }

    #[test]
    fn test_metrics_all_counters() {
        let metrics = SecurityMetrics::new();

        metrics.record_invalid_message();
        metrics.record_disconnect(StrikeCategory::DecodeError);
        metrics.record_payload_sync_abort();
        metrics.record_registry_parse_failure();
        metrics.record_rate_limited();
        metrics.record_handshake_timeout();
        metrics.record_zip_safety_rejection();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.invalid_messages_total, 1);
        assert_eq!(snapshot.disconnects_strikes_total, 1);
        assert_eq!(snapshot.payload_sync_aborts_total, 1);
        assert_eq!(snapshot.registry_parse_failures_total, 1);
        assert_eq!(snapshot.rate_limited_total, 1);
        assert_eq!(snapshot.handshake_timeouts_total, 1);
        assert_eq!(snapshot.zip_safety_rejections_total, 1);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = SecurityMetrics::new();

        metrics.record_invalid_message();
        metrics.reset();

        assert_eq!(metrics.snapshot().invalid_messages_total, 0);
    }

    #[test]
    fn test_logger_new() {
        let logger = RateLimitedLogger::new();
        assert_eq!(logger.suppressed_count("test"), 0);
    }

    #[test]
    fn test_logger_debug_mode_disabled() {
        let mut logger = RateLimitedLogger::new();
        // Debug should do nothing when debug_mode is false
        logger.debug("test", "message");
        // No assertion needed - just verify no panic
    }

    #[test]
    fn test_logger_debug_mode_enabled() {
        let mut logger = RateLimitedLogger::new().with_debug_mode(true);
        // Debug should work when debug_mode is true
        logger.debug("test", "message");
        // No assertion needed - just verify no panic
    }

    #[test]
    fn test_logger_reset() {
        let mut logger = RateLimitedLogger::new();
        logger.warn("test", "message");
        logger.reset();
        assert!(logger.last_log.is_empty());
        assert!(logger.suppressed.is_empty());
    }
}
