//! Time utilities for Plix

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{Duration, Instant};

/// Tick number (wraps at u32::MAX)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct Tick(pub u32);

impl Tick {
    /// The zero tick
    pub const ZERO: Self = Self(0);

    /// Get the next tick (wrapping)
    pub fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    /// Add a number of ticks
    pub fn add(self, ticks: u32) -> Self {
        Self(self.0.wrapping_add(ticks))
    }

    /// Subtract ticks (saturating at 0)
    pub fn saturating_sub(self, ticks: u32) -> Self {
        Self(self.0.saturating_sub(ticks))
    }

    /// Calculate difference between ticks (handles wrapping)
    pub fn diff(self, other: Self) -> i32 {
        self.0.wrapping_sub(other.0) as i32
    }

    /// Check if self is more recent than other (handles wrapping)
    pub fn is_newer_than(self, other: Self) -> bool {
        let diff = self.0.wrapping_sub(other.0);
        diff > 0 && diff < 0x80000000
    }

    /// Convert tick to duration given tick rate
    pub fn to_duration(self, tick_rate: u8) -> Duration {
        Duration::from_secs_f64(self.0 as f64 / tick_rate as f64)
    }

    /// Convert duration to ticks given tick rate
    pub fn from_duration(duration: Duration, tick_rate: u8) -> Self {
        Self((duration.as_secs_f64() * tick_rate as f64) as u32)
    }
}

impl fmt::Display for Tick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T{}", self.0)
    }
}

impl From<u32> for Tick {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Tick> for u32 {
    fn from(tick: Tick) -> Self {
        tick.0
    }
}

/// Tick rate configuration
#[derive(Debug, Clone, Copy)]
pub struct TickRate {
    pub rate: u8,
    pub duration: Duration,
}

impl TickRate {
    /// Create a new tick rate
    pub fn new(rate: u8) -> Self {
        let rate = rate.clamp(20, 60);
        Self {
            rate,
            duration: Duration::from_secs_f64(1.0 / rate as f64),
        }
    }

    /// Default tick rate (60 Hz)
    pub const DEFAULT: Self = Self {
        rate: 60,
        duration: Duration::from_nanos(16_666_667),
    };
}

impl Default for TickRate {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Timing helper for fixed tick loops
#[derive(Debug)]
pub struct TickTimer {
    tick_rate: TickRate,
    last_tick: Instant,
    accumulator: Duration,
}

impl TickTimer {
    /// Create a new tick timer
    pub fn new(tick_rate: TickRate) -> Self {
        Self {
            tick_rate,
            last_tick: Instant::now(),
            accumulator: Duration::ZERO,
        }
    }

    /// Update and return how many ticks should be processed
    pub fn update(&mut self) -> u32 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.last_tick = now;
        self.accumulator += elapsed;

        let mut ticks = 0;
        while self.accumulator >= self.tick_rate.duration {
            self.accumulator -= self.tick_rate.duration;
            ticks += 1;
        }

        // Cap at 10 ticks per update to prevent spiral of death
        ticks.min(10)
    }

    /// Get interpolation factor (0.0 to 1.0) for rendering between ticks
    pub fn alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / self.tick_rate.duration.as_secs_f32()
    }

    /// Get time until next tick
    pub fn time_until_next(&self) -> Duration {
        self.tick_rate.duration.saturating_sub(self.accumulator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_wrapping() {
        let tick = Tick(u32::MAX);
        assert_eq!(tick.next(), Tick(0));
    }

    #[test]
    fn test_tick_newer() {
        assert!(Tick(100).is_newer_than(Tick(50)));
        assert!(!Tick(50).is_newer_than(Tick(100)));
        // Wrap-around
        assert!(Tick(1).is_newer_than(Tick(u32::MAX - 10)));
    }

    #[test]
    fn test_tick_rate_clamping() {
        let rate = TickRate::new(5);
        assert_eq!(rate.rate, 20);

        let rate = TickRate::new(120);
        assert_eq!(rate.rate, 60);
    }
}
