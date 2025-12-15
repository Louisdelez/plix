//! Fixed tick loop for the authoritative server

use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Server tick rate configuration
#[derive(Debug, Clone, Copy)]
pub struct TickConfig {
    /// Tick rate in Hz (ticks per second)
    pub rate: u8,
    /// Duration per tick
    pub duration: Duration,
}

impl TickConfig {
    /// Create a new tick config with the given rate
    pub fn new(rate: u8) -> Self {
        let rate = rate.clamp(20, 60);
        Self {
            rate,
            duration: Duration::from_secs_f64(1.0 / rate as f64),
        }
    }
}

impl Default for TickConfig {
    fn default() -> Self {
        Self::new(60)
    }
}

/// Fixed tick loop runner
pub struct TickLoop {
    config: TickConfig,
    tick_count: u32,
    last_tick: Instant,
    overrun_count: u64,
}

impl TickLoop {
    /// Create a new tick loop with the given configuration
    pub fn new(config: TickConfig) -> Self {
        info!(rate = config.rate, "Initializing tick loop");
        Self {
            config,
            tick_count: 0,
            last_tick: Instant::now(),
            overrun_count: 0,
        }
    }

    /// Get current tick count
    pub fn tick_count(&self) -> u32 {
        self.tick_count
    }

    /// Get tick rate
    pub fn tick_rate(&self) -> u8 {
        self.config.rate
    }

    /// Run a single tick, returning time spent
    pub fn tick<F>(&mut self, mut tick_fn: F) -> Duration
    where
        F: FnMut(u32),
    {
        let tick_start = Instant::now();

        tick_fn(self.tick_count);
        self.tick_count = self.tick_count.wrapping_add(1);

        let elapsed = tick_start.elapsed();

        if elapsed > self.config.duration {
            self.overrun_count += 1;
            warn!(
                tick = self.tick_count,
                elapsed_ms = elapsed.as_millis(),
                target_ms = self.config.duration.as_millis(),
                overruns = self.overrun_count,
                "Tick overrun"
            );
        }

        self.last_tick = tick_start;
        elapsed
    }

    /// Sleep until next tick
    pub fn sleep_until_next(&self) {
        let now = Instant::now();
        let elapsed_since_tick = now.duration_since(self.last_tick);

        if let Some(sleep_time) = self.config.duration.checked_sub(elapsed_since_tick) {
            std::thread::sleep(sleep_time);
        }
    }

    /// Get time until next tick
    pub fn time_until_next(&self) -> Duration {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.config.duration.saturating_sub(elapsed)
    }

    /// Get total overrun count
    pub fn overrun_count(&self) -> u64 {
        self.overrun_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_config_clamping() {
        let config = TickConfig::new(10);
        assert_eq!(config.rate, 20);

        let config = TickConfig::new(100);
        assert_eq!(config.rate, 60);
    }

    #[test]
    fn test_tick_counting() {
        let config = TickConfig::new(60);
        let mut loop_runner = TickLoop::new(config);

        loop_runner.tick(|_| {});
        loop_runner.tick(|_| {});
        loop_runner.tick(|_| {});

        assert_eq!(loop_runner.tick_count(), 3);
    }
}
