//! Performance profiling configuration
//!
//! Configuration for enabling profiling, selecting scenarios, and setting budgets.

use std::path::PathBuf;

/// Performance profiling configuration
#[derive(Debug, Clone)]
pub struct PerfConfig {
    /// Whether performance profiling is enabled
    pub enabled: bool,
    /// Scenario to run (idle, world_churn, net_stress)
    pub scenario: String,
    /// Duration of the profiling run in seconds
    pub duration_secs: u32,
    /// Output path for the performance report
    pub report_path: PathBuf,
    /// Tick budget configuration
    pub budgets: BudgetConfig,
}

/// Tick budget configuration for backpressure
#[derive(Debug, Clone)]
pub struct BudgetConfig {
    /// Target tick duration in milliseconds (1000/TPS)
    pub tick_target_ms: f64,
    /// Network subsystem budget in milliseconds
    pub net_budget_ms: f64,
    /// Meshing subsystem budget in milliseconds
    pub meshing_budget_ms: f64,
    /// Mods dispatch budget in milliseconds
    pub mods_budget_ms: f64,
    /// Warning threshold in milliseconds (log if exceeded)
    pub warn_threshold_ms: f64,
    /// Critical threshold in milliseconds (engage backpressure)
    pub critical_threshold_ms: f64,
}

impl Default for PerfConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            scenario: "idle".to_string(),
            duration_secs: 60,
            report_path: PathBuf::from("perf_report.json"),
            budgets: BudgetConfig::default(),
        }
    }
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            // 60 TPS = 16.67ms per tick
            tick_target_ms: 16.67,
            net_budget_ms: 2.0,
            meshing_budget_ms: 3.0,
            mods_budget_ms: 1.5,
            warn_threshold_ms: 14.0,
            critical_threshold_ms: 16.0,
        }
    }
}

impl PerfConfig {
    /// Create a new config for a specific scenario
    pub fn for_scenario(scenario: &str, duration_secs: u32) -> Self {
        Self {
            enabled: true,
            scenario: scenario.to_string(),
            duration_secs,
            report_path: PathBuf::from("perf_report.json"),
            budgets: BudgetConfig::default(),
        }
    }

    /// Set the output report path
    pub fn with_report_path(mut self, path: PathBuf) -> Self {
        self.report_path = path;
        self
    }

    /// Set custom tick budget (for different TPS)
    pub fn with_tick_rate(mut self, tps: u8) -> Self {
        let tps = tps.clamp(20, 60) as f64;
        self.budgets.tick_target_ms = 1000.0 / tps;
        // Scale thresholds proportionally
        let scale = 60.0 / tps;
        self.budgets.warn_threshold_ms = 14.0 * scale;
        self.budgets.critical_threshold_ms = 16.0 * scale;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate scenario name
        let valid_scenarios = ["idle", "world_churn", "net_stress"];
        if !valid_scenarios.contains(&self.scenario.as_str()) {
            return Err(ConfigError::InvalidScenario(self.scenario.clone()));
        }

        // Validate duration
        if self.duration_secs < 10 {
            return Err(ConfigError::DurationTooShort(self.duration_secs));
        }

        // Validate budgets
        let budgets = &self.budgets;
        if budgets.tick_target_ms <= 0.0 {
            return Err(ConfigError::InvalidBudget("tick_target_ms must be > 0"));
        }
        if budgets.warn_threshold_ms >= budgets.tick_target_ms {
            return Err(ConfigError::InvalidBudget(
                "warn_threshold_ms must be < tick_target_ms",
            ));
        }
        if budgets.critical_threshold_ms > budgets.tick_target_ms {
            return Err(ConfigError::InvalidBudget(
                "critical_threshold_ms must be <= tick_target_ms",
            ));
        }

        let subsystem_sum =
            budgets.net_budget_ms + budgets.meshing_budget_ms + budgets.mods_budget_ms;
        if subsystem_sum >= budgets.tick_target_ms {
            return Err(ConfigError::InvalidBudget(
                "subsystem budgets sum must be < tick_target_ms",
            ));
        }

        Ok(())
    }
}

impl BudgetConfig {
    /// Create budget config for a specific TPS
    pub fn for_tps(tps: u8) -> Self {
        let tps = tps.clamp(20, 60) as f64;
        let tick_target_ms = 1000.0 / tps;
        let scale = tick_target_ms / 16.67; // Scale relative to 60 TPS

        Self {
            tick_target_ms,
            net_budget_ms: 2.0 * scale,
            meshing_budget_ms: 3.0 * scale,
            mods_budget_ms: 1.5 * scale,
            warn_threshold_ms: 14.0 * scale,
            critical_threshold_ms: 16.0 * scale,
        }
    }
}

/// Configuration validation errors
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// Invalid scenario name
    InvalidScenario(String),
    /// Duration too short (minimum 10 seconds)
    DurationTooShort(u32),
    /// Invalid budget configuration
    InvalidBudget(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidScenario(s) => {
                write!(f, "Invalid scenario '{}'. Valid: idle, world_churn, net_stress", s)
            }
            ConfigError::DurationTooShort(d) => {
                write!(f, "Duration {} too short (minimum 10 seconds)", d)
            }
            ConfigError::InvalidBudget(msg) => write!(f, "Invalid budget: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PerfConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.scenario, "idle");
        assert_eq!(config.duration_secs, 60);
        assert!((config.budgets.tick_target_ms - 16.67).abs() < 0.01);
    }

    #[test]
    fn test_for_scenario() {
        let config = PerfConfig::for_scenario("world_churn", 120);
        assert!(config.enabled);
        assert_eq!(config.scenario, "world_churn");
        assert_eq!(config.duration_secs, 120);
    }

    #[test]
    fn test_validate_valid_config() {
        let config = PerfConfig::for_scenario("idle", 60);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_scenario() {
        let config = PerfConfig::for_scenario("invalid", 60);
        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidScenario(_))
        ));
    }

    #[test]
    fn test_validate_duration_too_short() {
        let config = PerfConfig::for_scenario("idle", 5);
        assert!(matches!(
            config.validate(),
            Err(ConfigError::DurationTooShort(_))
        ));
    }

    #[test]
    fn test_with_tick_rate() {
        let config = PerfConfig::default().with_tick_rate(30);
        // 30 TPS = 33.33ms per tick
        assert!((config.budgets.tick_target_ms - 33.33).abs() < 0.01);
    }

    #[test]
    fn test_budget_for_tps() {
        let budget = BudgetConfig::for_tps(30);
        // 30 TPS should have 2x the budget of 60 TPS
        assert!((budget.tick_target_ms - 33.33).abs() < 0.01);
    }
}
