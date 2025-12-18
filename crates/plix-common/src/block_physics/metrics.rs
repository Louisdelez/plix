//! Block physics metrics
//!
//! Observable counters for monitoring physics simulation load.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Observable counters for monitoring physics load.
///
/// Uses atomic operations for potential future threading support.
#[derive(Debug, Default)]
pub struct BlockPhysicsMetrics {
    /// Events processed in last tick
    events_processed_last_tick: AtomicU32,
    /// Current queue depth
    queue_depth: AtomicU32,
    /// Total blocks that have fallen (lifetime)
    total_blocks_fallen: AtomicU64,
    /// Total liquid spread updates (lifetime)
    total_liquid_updates: AtomicU64,
}

impl BlockPhysicsMetrics {
    /// Create new metrics instance with all counters at zero
    pub fn new() -> Self {
        Self::default()
    }

    /// Get events processed in last tick
    pub fn events_processed_last_tick(&self) -> u32 {
        self.events_processed_last_tick.load(Ordering::Relaxed)
    }

    /// Get current queue depth
    pub fn queue_depth(&self) -> u32 {
        self.queue_depth.load(Ordering::Relaxed)
    }

    /// Get total blocks that have fallen (lifetime)
    pub fn total_blocks_fallen(&self) -> u64 {
        self.total_blocks_fallen.load(Ordering::Relaxed)
    }

    /// Get total liquid spread updates (lifetime)
    pub fn total_liquid_updates(&self) -> u64 {
        self.total_liquid_updates.load(Ordering::Relaxed)
    }

    /// Set events processed in last tick
    pub fn set_events_processed(&self, count: u32) {
        self.events_processed_last_tick
            .store(count, Ordering::Relaxed);
    }

    /// Set current queue depth
    pub fn set_queue_depth(&self, depth: u32) {
        self.queue_depth.store(depth, Ordering::Relaxed);
    }

    /// Increment total blocks fallen counter
    pub fn increment_blocks_fallen(&self) {
        self.total_blocks_fallen.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment total liquid updates counter
    pub fn increment_liquid_updates(&self) {
        self.total_liquid_updates.fetch_add(1, Ordering::Relaxed);
    }

    /// Reset all counters (useful for testing)
    pub fn reset(&self) {
        self.events_processed_last_tick.store(0, Ordering::Relaxed);
        self.queue_depth.store(0, Ordering::Relaxed);
        self.total_blocks_fallen.store(0, Ordering::Relaxed);
        self.total_liquid_updates.store(0, Ordering::Relaxed);
    }
}

impl Clone for BlockPhysicsMetrics {
    fn clone(&self) -> Self {
        Self {
            events_processed_last_tick: AtomicU32::new(
                self.events_processed_last_tick.load(Ordering::Relaxed),
            ),
            queue_depth: AtomicU32::new(self.queue_depth.load(Ordering::Relaxed)),
            total_blocks_fallen: AtomicU64::new(self.total_blocks_fallen.load(Ordering::Relaxed)),
            total_liquid_updates: AtomicU64::new(self.total_liquid_updates.load(Ordering::Relaxed)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_metrics_zero() {
        let metrics = BlockPhysicsMetrics::new();
        assert_eq!(metrics.events_processed_last_tick(), 0);
        assert_eq!(metrics.queue_depth(), 0);
        assert_eq!(metrics.total_blocks_fallen(), 0);
        assert_eq!(metrics.total_liquid_updates(), 0);
    }

    #[test]
    fn test_set_events_processed() {
        let metrics = BlockPhysicsMetrics::new();
        metrics.set_events_processed(42);
        assert_eq!(metrics.events_processed_last_tick(), 42);
    }

    #[test]
    fn test_set_queue_depth() {
        let metrics = BlockPhysicsMetrics::new();
        metrics.set_queue_depth(100);
        assert_eq!(metrics.queue_depth(), 100);
    }

    #[test]
    fn test_increment_blocks_fallen() {
        let metrics = BlockPhysicsMetrics::new();
        metrics.increment_blocks_fallen();
        metrics.increment_blocks_fallen();
        metrics.increment_blocks_fallen();
        assert_eq!(metrics.total_blocks_fallen(), 3);
    }

    #[test]
    fn test_increment_liquid_updates() {
        let metrics = BlockPhysicsMetrics::new();
        metrics.increment_liquid_updates();
        metrics.increment_liquid_updates();
        assert_eq!(metrics.total_liquid_updates(), 2);
    }

    #[test]
    fn test_reset() {
        let metrics = BlockPhysicsMetrics::new();
        metrics.set_events_processed(10);
        metrics.set_queue_depth(20);
        metrics.increment_blocks_fallen();
        metrics.increment_liquid_updates();

        metrics.reset();

        assert_eq!(metrics.events_processed_last_tick(), 0);
        assert_eq!(metrics.queue_depth(), 0);
        assert_eq!(metrics.total_blocks_fallen(), 0);
        assert_eq!(metrics.total_liquid_updates(), 0);
    }

    #[test]
    fn test_clone() {
        let metrics = BlockPhysicsMetrics::new();
        metrics.set_events_processed(50);
        metrics.increment_blocks_fallen();

        let cloned = metrics.clone();
        assert_eq!(cloned.events_processed_last_tick(), 50);
        assert_eq!(cloned.total_blocks_fallen(), 1);
    }
}
