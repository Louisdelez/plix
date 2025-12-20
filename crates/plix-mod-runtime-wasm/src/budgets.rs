//! CPU and memory budget tracking
//!
//! Manages fuel budgets for CPU limiting and memory allocation limits.
//! Fuel is Wasmtime's mechanism for metering WASM instruction execution.

use std::time::{Duration, Instant};

/// Default fuel units per millisecond (calibrated at startup)
pub const DEFAULT_FUEL_PER_MS: u64 = 10_000;

/// Minimum fuel budget (prevents edge cases with very small budgets)
pub const MIN_FUEL_BUDGET: u64 = 1_000;

/// Fuel budget manager for CPU limiting
#[derive(Debug, Clone)]
pub struct FuelBudget {
    /// Fuel units per millisecond (calibrated)
    pub fuel_per_ms: u64,
    /// Default budget for handler calls
    pub handler_budget: u64,
    /// Default budget for tick processing
    pub tick_budget: u64,
}

impl FuelBudget {
    /// Create a new fuel budget with given ms budgets
    pub fn new(handler_ms: u64, tick_ms: u64, fuel_per_ms: u64) -> Self {
        let fuel_per_ms = fuel_per_ms.max(1); // Prevent division by zero

        Self {
            fuel_per_ms,
            handler_budget: (handler_ms * fuel_per_ms).max(MIN_FUEL_BUDGET),
            tick_budget: (tick_ms * fuel_per_ms).max(MIN_FUEL_BUDGET),
        }
    }

    /// Create with default calibration (use calibrate() for accurate values)
    pub fn with_defaults(handler_ms: u64, tick_ms: u64) -> Self {
        Self::new(handler_ms, tick_ms, DEFAULT_FUEL_PER_MS)
    }

    /// Convert fuel units to milliseconds
    pub fn fuel_to_ms(&self, fuel: u64) -> f64 {
        (fuel as f64) / (self.fuel_per_ms as f64)
    }

    /// Convert milliseconds to fuel units
    pub fn ms_to_fuel(&self, ms: u64) -> u64 {
        ms * self.fuel_per_ms
    }

    /// Get fuel budget for handler call
    pub fn handler_fuel(&self) -> u64 {
        self.handler_budget
    }

    /// Get fuel budget for tick
    pub fn tick_fuel(&self) -> u64 {
        self.tick_budget
    }

    /// Calibrate fuel consumption by running a test loop
    ///
    /// This measures how many fuel units are consumed in 1ms on the current CPU.
    /// Should be called once at runtime initialization.
    pub fn calibrate(handler_ms: u64, tick_ms: u64) -> Self {
        let fuel_per_ms = calibrate_fuel();
        Self::new(handler_ms, tick_ms, fuel_per_ms)
    }
}

impl Default for FuelBudget {
    fn default() -> Self {
        Self::with_defaults(5, 10)
    }
}

/// Calibrate fuel consumption on the current CPU
///
/// Returns estimated fuel units per millisecond.
fn calibrate_fuel() -> u64 {
    // Run a simple calibration loop
    // In Wasmtime, fuel is consumed per instruction, so we measure time
    // for a known operation count

    let iterations = 100_000u64;
    let start = Instant::now();

    // Simulate work - volatile to prevent optimization
    let mut sum: u64 = 0;
    for i in 0..iterations {
        sum = sum.wrapping_add(i);
    }

    // Prevent optimization
    std::hint::black_box(sum);

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;

    if elapsed_ms < 0.001 {
        // Too fast to measure, use default
        return DEFAULT_FUEL_PER_MS;
    }

    // Estimate: each iteration ~= 1 fuel unit (rough approximation)
    // Real calibration would run actual WASM code
    let fuel_per_ms = (iterations as f64 / elapsed_ms) as u64;

    // Clamp to reasonable range
    fuel_per_ms.clamp(1_000, 1_000_000)
}

/// Convert a Duration to estimated fuel consumption
pub fn duration_to_fuel(duration: Duration, fuel_per_ms: u64) -> u64 {
    let ms = duration.as_secs_f64() * 1000.0;
    (ms * fuel_per_ms as f64) as u64
}

/// Convert fuel consumption to estimated Duration
pub fn fuel_to_duration(fuel: u64, fuel_per_ms: u64) -> Duration {
    let ms = fuel as f64 / fuel_per_ms as f64;
    Duration::from_secs_f64(ms / 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuel_budget_new() {
        let budget = FuelBudget::new(5, 10, 10_000);
        assert_eq!(budget.handler_budget, 50_000);
        assert_eq!(budget.tick_budget, 100_000);
        assert_eq!(budget.fuel_per_ms, 10_000);
    }

    #[test]
    fn test_fuel_budget_min() {
        // Very small ms values should still get minimum fuel
        let budget = FuelBudget::new(0, 0, 10_000);
        assert_eq!(budget.handler_budget, MIN_FUEL_BUDGET);
        assert_eq!(budget.tick_budget, MIN_FUEL_BUDGET);
    }

    #[test]
    fn test_fuel_to_ms() {
        let budget = FuelBudget::new(5, 10, 10_000);

        assert!((budget.fuel_to_ms(10_000) - 1.0).abs() < 0.01);
        assert!((budget.fuel_to_ms(50_000) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_ms_to_fuel() {
        let budget = FuelBudget::new(5, 10, 10_000);

        assert_eq!(budget.ms_to_fuel(1), 10_000);
        assert_eq!(budget.ms_to_fuel(5), 50_000);
    }

    #[test]
    fn test_calibration() {
        // Just verify it returns a reasonable value
        let budget = FuelBudget::calibrate(5, 10);
        assert!(budget.fuel_per_ms >= 1_000);
        assert!(budget.fuel_per_ms <= 1_000_000);
    }

    #[test]
    fn test_duration_conversion() {
        let fuel_per_ms = 10_000;

        let fuel = duration_to_fuel(Duration::from_millis(5), fuel_per_ms);
        assert_eq!(fuel, 50_000);

        let duration = fuel_to_duration(50_000, fuel_per_ms);
        assert!((duration.as_secs_f64() - 0.005).abs() < 0.0001);
    }

    #[test]
    fn test_zero_fuel_per_ms_safety() {
        // Should not panic with zero fuel_per_ms
        let budget = FuelBudget::new(5, 10, 0);
        assert!(budget.fuel_per_ms >= 1);
    }
}
