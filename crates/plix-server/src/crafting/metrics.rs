//! Crafting metrics for observability.

use std::collections::HashMap;

/// Crafting metrics for monitoring and debugging.
#[derive(Debug, Default)]
pub struct CraftMetrics {
    /// Total craft attempts.
    pub total_attempts: u64,
    /// Successful crafts.
    pub successes: u64,
    /// Failed crafts by reason.
    pub failures_by_reason: HashMap<String, u64>,
    /// Crafts per recipe.
    pub crafts_per_recipe: HashMap<String, u64>,
}

impl CraftMetrics {
    /// Create a new metrics tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful craft.
    pub fn record_success(&mut self, recipe_id: &str) {
        self.total_attempts += 1;
        self.successes += 1;
        *self
            .crafts_per_recipe
            .entry(recipe_id.to_string())
            .or_insert(0) += 1;
    }

    /// Record a failed craft.
    pub fn record_failure(&mut self, recipe_id: &str, reason: &str) {
        self.total_attempts += 1;
        *self
            .failures_by_reason
            .entry(reason.to_string())
            .or_insert(0) += 1;
        let _ = recipe_id; // Could track failed attempts per recipe if needed
    }

    /// Get success rate (0.0 to 1.0).
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.successes as f64 / self.total_attempts as f64
        }
    }

    /// Reset all metrics.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_metrics() {
        let metrics = CraftMetrics::new();
        assert_eq!(metrics.total_attempts, 0);
        assert_eq!(metrics.successes, 0);
        assert_eq!(metrics.success_rate(), 0.0);
    }

    #[test]
    fn test_record_success() {
        let mut metrics = CraftMetrics::new();
        metrics.record_success("health_pack");
        metrics.record_success("health_pack");
        metrics.record_success("sword");

        assert_eq!(metrics.total_attempts, 3);
        assert_eq!(metrics.successes, 3);
        assert_eq!(metrics.crafts_per_recipe.get("health_pack"), Some(&2));
        assert_eq!(metrics.crafts_per_recipe.get("sword"), Some(&1));
    }

    #[test]
    fn test_record_failure() {
        let mut metrics = CraftMetrics::new();
        metrics.record_failure("health_pack", "MissingIngredients");
        metrics.record_failure("sword", "MissingIngredients");
        metrics.record_failure("bow", "HotbarFull");

        assert_eq!(metrics.total_attempts, 3);
        assert_eq!(metrics.successes, 0);
        assert_eq!(
            metrics.failures_by_reason.get("MissingIngredients"),
            Some(&2)
        );
        assert_eq!(metrics.failures_by_reason.get("HotbarFull"), Some(&1));
    }

    #[test]
    fn test_success_rate() {
        let mut metrics = CraftMetrics::new();
        metrics.record_success("health_pack");
        metrics.record_success("health_pack");
        metrics.record_failure("sword", "MissingIngredients");
        metrics.record_failure("bow", "HotbarFull");

        assert_eq!(metrics.total_attempts, 4);
        assert_eq!(metrics.successes, 2);
        assert!((metrics.success_rate() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_reset() {
        let mut metrics = CraftMetrics::new();
        metrics.record_success("health_pack");
        metrics.record_failure("sword", "Error");

        metrics.reset();

        assert_eq!(metrics.total_attempts, 0);
        assert_eq!(metrics.successes, 0);
        assert!(metrics.failures_by_reason.is_empty());
        assert!(metrics.crafts_per_recipe.is_empty());
    }
}
