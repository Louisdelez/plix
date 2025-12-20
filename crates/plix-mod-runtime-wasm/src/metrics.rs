//! Per-mod metrics collection for the WASM runtime
//!
//! Tracks CPU time, memory usage, trap counts, and host call statistics.

use std::time::Duration;

/// Metrics collected per WASM mod instance
#[derive(Debug, Clone, Default)]
pub struct WasmModMetrics {
    /// Total CPU time consumed (estimated from fuel)
    pub cpu_time: Duration,
    /// Number of WASM traps caught
    pub trap_count: u32,
    /// Total host function calls
    pub host_call_count: u64,
    /// Number of permission denied responses (EMOD002)
    pub permission_denied_count: u32,
    /// Number of rate limited responses (EMOD005)
    pub rate_limited_count: u32,
    /// Fuel used in the last handler call
    pub last_call_fuel: u64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: usize,
    /// Current memory usage in bytes (updated by ResourceLimiter)
    pub memory_bytes: usize,
    /// Number of memory allocation failures (OOM)
    pub memory_error_count: u32,
    /// Total events processed
    pub events_processed: u64,
    /// Events that resulted in cancellation
    pub events_cancelled: u64,
}

impl WasmModMetrics {
    /// Create new metrics with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Record CPU time from fuel consumption
    pub fn record_cpu_time(&mut self, fuel_consumed: u64, fuel_per_ms: u64) {
        if fuel_per_ms > 0 {
            let ms = fuel_consumed / fuel_per_ms;
            let micros = ((fuel_consumed % fuel_per_ms) * 1000) / fuel_per_ms;
            self.cpu_time += Duration::from_millis(ms) + Duration::from_micros(micros);
        }
        self.last_call_fuel = fuel_consumed;
    }

    /// Record a trap
    pub fn record_trap(&mut self) {
        self.trap_count += 1;
    }

    /// Record a host function call
    pub fn record_host_call(&mut self) {
        self.host_call_count += 1;
    }

    /// Record a permission denied error
    pub fn record_permission_denied(&mut self) {
        self.permission_denied_count += 1;
    }

    /// Record a rate limited error
    pub fn record_rate_limited(&mut self) {
        self.rate_limited_count += 1;
    }

    /// Record a memory allocation error (OOM)
    pub fn record_memory_error(&mut self) {
        self.memory_error_count += 1;
    }

    /// Update memory usage
    pub fn update_memory(&mut self, current_bytes: usize) {
        self.memory_bytes = current_bytes;
        if current_bytes > self.peak_memory_bytes {
            self.peak_memory_bytes = current_bytes;
        }
    }

    /// Record an event being processed
    pub fn record_event(&mut self, cancelled: bool) {
        self.events_processed += 1;
        if cancelled {
            self.events_cancelled += 1;
        }
    }

    /// Reset all metrics (for testing or new session)
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Get CPU time as milliseconds (f64 for precision)
    pub fn cpu_time_ms(&self) -> f64 {
        self.cpu_time.as_secs_f64() * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_default() {
        let metrics = WasmModMetrics::new();
        assert_eq!(metrics.trap_count, 0);
        assert_eq!(metrics.host_call_count, 0);
        assert_eq!(metrics.cpu_time, Duration::ZERO);
    }

    #[test]
    fn test_record_cpu_time() {
        let mut metrics = WasmModMetrics::new();
        let fuel_per_ms = 10_000;

        // 50,000 fuel = 5ms
        metrics.record_cpu_time(50_000, fuel_per_ms);
        assert_eq!(metrics.cpu_time, Duration::from_millis(5));
        assert_eq!(metrics.last_call_fuel, 50_000);

        // Add another 25,000 fuel = 2.5ms
        metrics.record_cpu_time(25_000, fuel_per_ms);
        assert_eq!(
            metrics.cpu_time,
            Duration::from_millis(7) + Duration::from_micros(500)
        );
    }

    #[test]
    fn test_record_trap() {
        let mut metrics = WasmModMetrics::new();
        metrics.record_trap();
        metrics.record_trap();
        assert_eq!(metrics.trap_count, 2);
    }

    #[test]
    fn test_record_host_call() {
        let mut metrics = WasmModMetrics::new();
        for _ in 0..100 {
            metrics.record_host_call();
        }
        assert_eq!(metrics.host_call_count, 100);
    }

    #[test]
    fn test_update_memory() {
        let mut metrics = WasmModMetrics::new();

        metrics.update_memory(1000);
        assert_eq!(metrics.memory_bytes, 1000);
        assert_eq!(metrics.peak_memory_bytes, 1000);

        metrics.update_memory(2000);
        assert_eq!(metrics.memory_bytes, 2000);
        assert_eq!(metrics.peak_memory_bytes, 2000);

        metrics.update_memory(1500);
        assert_eq!(metrics.memory_bytes, 1500);
        assert_eq!(metrics.peak_memory_bytes, 2000); // Peak unchanged
    }

    #[test]
    fn test_record_event() {
        let mut metrics = WasmModMetrics::new();

        metrics.record_event(false);
        metrics.record_event(true);
        metrics.record_event(false);

        assert_eq!(metrics.events_processed, 3);
        assert_eq!(metrics.events_cancelled, 1);
    }

    #[test]
    fn test_reset() {
        let mut metrics = WasmModMetrics::new();
        metrics.record_trap();
        metrics.record_host_call();
        metrics.update_memory(1000);

        metrics.reset();

        assert_eq!(metrics.trap_count, 0);
        assert_eq!(metrics.host_call_count, 0);
        assert_eq!(metrics.peak_memory_bytes, 0);
    }

    #[test]
    fn test_cpu_time_ms() {
        let mut metrics = WasmModMetrics::new();
        metrics.cpu_time = Duration::from_millis(5) + Duration::from_micros(500);

        let ms = metrics.cpu_time_ms();
        assert!((ms - 5.5).abs() < 0.001);
    }
}
