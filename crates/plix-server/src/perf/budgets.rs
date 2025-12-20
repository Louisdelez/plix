//! Tick budget tracking and backpressure mechanisms
//!
//! Provides budget enforcement for subsystems and backpressure when ticks exceed limits.

use std::time::Duration;

use crate::perf::config::BudgetConfig;

/// Tick budget tracker with backpressure support
#[derive(Debug)]
pub struct TickBudget {
    config: BudgetConfig,
    /// Accumulated overruns this session
    overrun_count: u64,
    /// Last tick duration in milliseconds
    last_tick_ms: f64,
    /// Top 3 subsystems by time when tick exceeds budget
    top_subsystems: Vec<(String, f64)>,
}

impl TickBudget {
    /// Create a new tick budget tracker with the given configuration
    pub fn new(config: BudgetConfig) -> Self {
        Self {
            config,
            overrun_count: 0,
            last_tick_ms: 0.0,
            top_subsystems: Vec::with_capacity(3),
        }
    }

    /// Record a tick duration and check for overruns
    ///
    /// Returns `true` if tick exceeded the warning threshold
    pub fn record_tick(&mut self, duration: Duration) -> bool {
        let ms = duration.as_secs_f64() * 1000.0;
        self.last_tick_ms = ms;

        if ms > self.config.warn_threshold_ms {
            self.overrun_count += 1;
            true
        } else {
            false
        }
    }

    /// Check if backpressure should be engaged
    pub fn should_engage_backpressure(&self) -> bool {
        self.last_tick_ms > self.config.critical_threshold_ms
    }

    /// Get remaining budget for a subsystem
    pub fn remaining_budget(&self, subsystem: &str) -> f64 {
        let budget = match subsystem {
            "net" | "net_encode" | "net_decode" | "net_flush" => self.config.net_budget_ms,
            "meshing" => self.config.meshing_budget_ms,
            "mods" | "mods_dispatch" => self.config.mods_budget_ms,
            _ => self.config.tick_target_ms / 4.0, // Default 25% for unknown
        };
        budget
    }

    /// Record subsystem timing for budget analysis
    pub fn record_subsystem(&mut self, name: &str, ms: f64) {
        // Keep top 3 by time
        self.top_subsystems.push((name.to_string(), ms));
        self.top_subsystems.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        self.top_subsystems.truncate(3);
    }

    /// Get top subsystems from the last overrun tick
    pub fn top_subsystems(&self) -> &[(String, f64)] {
        &self.top_subsystems
    }

    /// Clear subsystem tracking for new tick
    pub fn clear_subsystems(&mut self) {
        self.top_subsystems.clear();
    }

    /// Get total overrun count
    pub fn overrun_count(&self) -> u64 {
        self.overrun_count
    }

    /// Get the target tick duration in milliseconds
    pub fn target_ms(&self) -> f64 {
        self.config.tick_target_ms
    }

    /// Get the warning threshold in milliseconds
    pub fn warn_threshold_ms(&self) -> f64 {
        self.config.warn_threshold_ms
    }

    /// Get the critical threshold in milliseconds
    pub fn critical_threshold_ms(&self) -> f64 {
        self.config.critical_threshold_ms
    }
}

impl Default for TickBudget {
    fn default() -> Self {
        Self::new(BudgetConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_tick_normal() {
        let mut budget = TickBudget::default();
        // 10ms tick should be fine (under 14ms warn threshold)
        let exceeded = budget.record_tick(Duration::from_millis(10));
        assert!(!exceeded);
        assert_eq!(budget.overrun_count(), 0);
    }

    #[test]
    fn test_record_tick_overrun() {
        let mut budget = TickBudget::default();
        // 15ms tick exceeds 14ms warn threshold
        let exceeded = budget.record_tick(Duration::from_millis(15));
        assert!(exceeded);
        assert_eq!(budget.overrun_count(), 1);
    }

    #[test]
    fn test_backpressure_threshold() {
        let mut budget = TickBudget::default();
        budget.record_tick(Duration::from_millis(17));
        assert!(budget.should_engage_backpressure());
    }

    #[test]
    fn test_no_backpressure() {
        let mut budget = TickBudget::default();
        budget.record_tick(Duration::from_millis(15));
        assert!(!budget.should_engage_backpressure());
    }

    #[test]
    fn test_top_subsystems() {
        let mut budget = TickBudget::default();
        budget.record_subsystem("net", 2.5);
        budget.record_subsystem("meshing", 4.0);
        budget.record_subsystem("simulation", 3.0);
        budget.record_subsystem("mods", 1.0);

        let top = budget.top_subsystems();
        assert_eq!(top.len(), 3);
        assert_eq!(top[0].0, "meshing");
        assert_eq!(top[1].0, "simulation");
        assert_eq!(top[2].0, "net");
    }
}
