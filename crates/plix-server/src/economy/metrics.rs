//! Economy metrics for observability.

use std::collections::HashMap;

use plix_common::economy::PurchaseRejectReason;

/// Economy system metrics collector.
#[derive(Debug, Clone, Default)]
pub struct EconomyMetrics {
    /// Total coins earned across all players
    pub coins_earned_total: u64,
    /// Total coins spent across all players
    pub coins_spent_total: u64,
    /// Total successful purchases
    pub purchases_total: u64,
    /// Failed purchases by rejection reason
    pub purchases_failed: HashMap<PurchaseRejectReason, u64>,
    /// Total rate-limited requests
    pub rate_limited_total: u64,
}

impl EconomyMetrics {
    /// Create new empty metrics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record coins earned.
    pub fn record_earning(&mut self, amount: u32) {
        self.coins_earned_total += amount as u64;
    }

    /// Record a successful purchase.
    pub fn record_purchase(&mut self, price: u32) {
        self.purchases_total += 1;
        self.coins_spent_total += price as u64;
    }

    /// Record a failed purchase.
    pub fn record_failure(&mut self, reason: PurchaseRejectReason) {
        *self.purchases_failed.entry(reason).or_insert(0) += 1;
        if reason == PurchaseRejectReason::RateLimited {
            self.rate_limited_total += 1;
        }
    }

    /// Get total failed purchases.
    pub fn purchases_failed_total(&self) -> u64 {
        self.purchases_failed.values().sum()
    }

    /// Reset all metrics.
    pub fn reset(&mut self) {
        self.coins_earned_total = 0;
        self.coins_spent_total = 0;
        self.purchases_total = 0;
        self.purchases_failed.clear();
        self.rate_limited_total = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_earning() {
        let mut metrics = EconomyMetrics::new();
        metrics.record_earning(100);
        metrics.record_earning(50);
        assert_eq!(metrics.coins_earned_total, 150);
    }

    #[test]
    fn test_record_purchase() {
        let mut metrics = EconomyMetrics::new();
        metrics.record_purchase(20);
        metrics.record_purchase(50);
        assert_eq!(metrics.purchases_total, 2);
        assert_eq!(metrics.coins_spent_total, 70);
    }

    #[test]
    fn test_record_failure() {
        let mut metrics = EconomyMetrics::new();
        metrics.record_failure(PurchaseRejectReason::InsufficientBalance);
        metrics.record_failure(PurchaseRejectReason::InsufficientBalance);
        metrics.record_failure(PurchaseRejectReason::HotbarFull);

        assert_eq!(metrics.purchases_failed_total(), 3);
        assert_eq!(
            metrics
                .purchases_failed
                .get(&PurchaseRejectReason::InsufficientBalance),
            Some(&2)
        );
    }

    #[test]
    fn test_rate_limited_tracking() {
        let mut metrics = EconomyMetrics::new();
        metrics.record_failure(PurchaseRejectReason::RateLimited);
        metrics.record_failure(PurchaseRejectReason::RateLimited);
        metrics.record_failure(PurchaseRejectReason::InsufficientBalance);

        assert_eq!(metrics.rate_limited_total, 2);
    }

    #[test]
    fn test_reset() {
        let mut metrics = EconomyMetrics::new();
        metrics.record_earning(100);
        metrics.record_purchase(20);
        metrics.record_failure(PurchaseRejectReason::InsufficientBalance);

        metrics.reset();

        assert_eq!(metrics.coins_earned_total, 0);
        assert_eq!(metrics.coins_spent_total, 0);
        assert_eq!(metrics.purchases_total, 0);
        assert_eq!(metrics.purchases_failed_total(), 0);
    }
}
