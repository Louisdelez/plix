//! Observability: structured logging and metrics for mod execution

use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{debug, error, info, warn};

/// Metrics counters for mod system observability
#[derive(Debug, Default)]
pub struct ModMetrics {
    /// Total events dispatched
    pub events_dispatched: AtomicU64,
    /// Total handler errors
    pub handler_errors: AtomicU64,
    /// Total network messages sent
    pub net_messages_sent: AtomicU64,
    /// Total network messages dropped (rate limited)
    pub net_messages_dropped: AtomicU64,
    /// Total API permission denials
    pub api_permission_denied: AtomicU64,
    /// Current active timers across all mods
    pub timers_active: AtomicU64,
    /// Total mods loaded
    pub mods_loaded: AtomicU64,
    /// Total mods disabled
    pub mods_disabled: AtomicU64,
}

impl ModMetrics {
    /// Create new metrics counters
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an event dispatch
    pub fn record_event_dispatch(&self) {
        self.events_dispatched.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a handler error
    pub fn record_handler_error(&self) {
        self.handler_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a network message sent
    pub fn record_net_sent(&self) {
        self.net_messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a network message dropped
    pub fn record_net_dropped(&self) {
        self.net_messages_dropped.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an API permission denial
    pub fn record_permission_denied(&self) {
        self.api_permission_denied.fetch_add(1, Ordering::Relaxed);
    }

    /// Update active timer count
    pub fn set_timers_active(&self, count: u64) {
        self.timers_active.store(count, Ordering::Relaxed);
    }

    /// Increment timers active
    pub fn inc_timers_active(&self) {
        self.timers_active.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement timers active
    pub fn dec_timers_active(&self) {
        self.timers_active.fetch_sub(1, Ordering::Relaxed);
    }

    /// Record a mod load
    pub fn record_mod_loaded(&self) {
        self.mods_loaded.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a mod disable
    pub fn record_mod_disabled(&self) {
        self.mods_disabled.fetch_add(1, Ordering::Relaxed);
    }

    /// Get snapshot of all metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            events_dispatched: self.events_dispatched.load(Ordering::Relaxed),
            handler_errors: self.handler_errors.load(Ordering::Relaxed),
            net_messages_sent: self.net_messages_sent.load(Ordering::Relaxed),
            net_messages_dropped: self.net_messages_dropped.load(Ordering::Relaxed),
            api_permission_denied: self.api_permission_denied.load(Ordering::Relaxed),
            timers_active: self.timers_active.load(Ordering::Relaxed),
            mods_loaded: self.mods_loaded.load(Ordering::Relaxed),
            mods_disabled: self.mods_disabled.load(Ordering::Relaxed),
        }
    }
}

/// Point-in-time snapshot of metrics
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub events_dispatched: u64,
    pub handler_errors: u64,
    pub net_messages_sent: u64,
    pub net_messages_dropped: u64,
    pub api_permission_denied: u64,
    pub timers_active: u64,
    pub mods_loaded: u64,
    pub mods_disabled: u64,
}

// === Structured logging helpers ===

/// Log a mod load event
pub fn log_mod_loaded(mod_id: &str, version: &str, capabilities: &str) {
    info!(
        target: "plix_mod",
        mod_id = %mod_id,
        version = %version,
        capabilities = %capabilities,
        "Mod loaded"
    );
}

/// Log a mod unload event
pub fn log_mod_unloaded(mod_id: &str) {
    info!(
        target: "plix_mod",
        mod_id = %mod_id,
        "Mod unloaded"
    );
}

/// Log a mod disable event
pub fn log_mod_disabled(mod_id: &str, reason: &str) {
    warn!(
        target: "plix_mod",
        mod_id = %mod_id,
        reason = %reason,
        "Mod disabled"
    );
}

/// Log an event dispatch
pub fn log_event_dispatch(event_type: &str, subscriber_count: usize) {
    debug!(
        target: "plix_mod",
        event_type = %event_type,
        subscribers = subscriber_count,
        "Event dispatched"
    );
}

/// Log a handler error
pub fn log_handler_error(mod_id: &str, event_type: &str, error: &str, error_count: u32) {
    error!(
        target: "plix_mod",
        mod_id = %mod_id,
        event_type = %event_type,
        error = %error,
        consecutive_errors = error_count,
        "Handler error"
    );
}

/// Log an API permission denial
pub fn log_permission_denied(mod_id: &str, api_call: &str, capability: &str) {
    warn!(
        target: "plix_mod",
        mod_id = %mod_id,
        api_call = %api_call,
        required_capability = %capability,
        "API permission denied"
    );
}

/// Log a rate limit violation
pub fn log_rate_limited(mod_id: &str, operation: &str) {
    warn!(
        target: "plix_mod",
        mod_id = %mod_id,
        operation = %operation,
        "Rate limited"
    );
}

/// Log a timer creation
pub fn log_timer_created(mod_id: &str, timer_id: u32, interval_ms: u32, is_interval: bool) {
    debug!(
        target: "plix_mod",
        mod_id = %mod_id,
        timer_id = timer_id,
        interval_ms = interval_ms,
        is_interval = is_interval,
        "Timer created"
    );
}

/// Log a timer cancellation
pub fn log_timer_cancelled(mod_id: &str, timer_id: u32) {
    debug!(
        target: "plix_mod",
        mod_id = %mod_id,
        timer_id = timer_id,
        "Timer cancelled"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_counters() {
        let metrics = ModMetrics::new();

        metrics.record_event_dispatch();
        metrics.record_event_dispatch();
        metrics.record_handler_error();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.events_dispatched, 2);
        assert_eq!(snapshot.handler_errors, 1);
    }

    #[test]
    fn test_metrics_timers() {
        let metrics = ModMetrics::new();

        metrics.inc_timers_active();
        metrics.inc_timers_active();
        assert_eq!(metrics.timers_active.load(Ordering::Relaxed), 2);

        metrics.dec_timers_active();
        assert_eq!(metrics.timers_active.load(Ordering::Relaxed), 1);

        metrics.set_timers_active(10);
        assert_eq!(metrics.timers_active.load(Ordering::Relaxed), 10);
    }

    #[test]
    fn test_metrics_snapshot() {
        let metrics = ModMetrics::new();

        metrics.record_net_sent();
        metrics.record_net_sent();
        metrics.record_net_dropped();
        metrics.record_permission_denied();
        metrics.record_mod_loaded();
        metrics.record_mod_loaded();
        metrics.record_mod_disabled();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.net_messages_sent, 2);
        assert_eq!(snapshot.net_messages_dropped, 1);
        assert_eq!(snapshot.api_permission_denied, 1);
        assert_eq!(snapshot.mods_loaded, 2);
        assert_eq!(snapshot.mods_disabled, 1);
    }
}
