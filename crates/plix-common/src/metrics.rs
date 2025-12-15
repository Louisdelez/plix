//! Metrics utilities for Plix
//!
//! Provides:
//! - RollingWindow<T>: Time-based fixed-size ring buffer for metric samples
//! - Stats: Statistical summary (avg, p95, max, min, count)

use std::time::{Duration, Instant};

/// Default window duration (10 seconds)
pub const DEFAULT_WINDOW_DURATION: Duration = Duration::from_secs(10);

/// Default capacity (600 samples = 10s at 60Hz)
pub const DEFAULT_CAPACITY: usize = 600;

/// Statistical summary of samples in a rolling window
#[derive(Debug, Clone, Copy, Default)]
pub struct Stats {
    /// Average value
    pub avg: f64,
    /// 95th percentile value
    pub p95: f64,
    /// Maximum value
    pub max: f64,
    /// Minimum value
    pub min: f64,
    /// Number of samples
    pub count: usize,
}

impl Stats {
    /// Create stats with no samples
    pub fn empty() -> Self {
        Self::default()
    }
}

/// Time-based rolling window for metric samples.
///
/// Uses a fixed-size ring buffer with pre-allocated storage.
/// Samples older than `window_duration` are excluded from statistics.
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// use plix_common::metrics::RollingWindow;
///
/// let mut window: RollingWindow<f64> = RollingWindow::new(100, Duration::from_secs(10));
/// window.push(1.0);
/// window.push(2.0);
/// window.push(3.0);
///
/// let stats = window.stats();
/// assert_eq!(stats.count, 3);
/// ```
#[derive(Debug)]
pub struct RollingWindow<T> {
    /// Storage for samples with timestamps
    samples: Vec<Option<(Instant, T)>>,
    /// Current write position (ring buffer index)
    head: usize,
    /// Number of valid samples written (up to capacity)
    written: usize,
    /// Window duration (samples older than this are excluded from stats)
    window_duration: Duration,
    /// Cached statistics (invalidated on push)
    cached_stats: Option<Stats>,
}

impl<T: Copy> RollingWindow<T> {
    /// Create a new rolling window with the given capacity and duration.
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of samples to store
    /// * `window_duration` - Time window for statistics (samples older than this are excluded)
    pub fn new(capacity: usize, window_duration: Duration) -> Self {
        assert!(capacity > 0, "RollingWindow capacity must be at least 1");
        Self {
            samples: vec![None; capacity],
            head: 0,
            written: 0,
            window_duration,
            cached_stats: None,
        }
    }

    /// Create a rolling window with default settings (600 samples, 10s window).
    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_CAPACITY, DEFAULT_WINDOW_DURATION)
    }

    /// Push a new sample into the window.
    ///
    /// This operation is O(1) and does not allocate.
    pub fn push(&mut self, value: T) {
        self.samples[self.head] = Some((Instant::now(), value));
        self.head = (self.head + 1) % self.samples.len();
        if self.written < self.samples.len() {
            self.written += 1;
        }
        // Invalidate cached stats
        self.cached_stats = None;
    }

    /// Push a sample with a specific timestamp (for testing).
    pub fn push_at(&mut self, timestamp: Instant, value: T) {
        self.samples[self.head] = Some((timestamp, value));
        self.head = (self.head + 1) % self.samples.len();
        if self.written < self.samples.len() {
            self.written += 1;
        }
        self.cached_stats = None;
    }

    /// Get an iterator over samples within the time window.
    ///
    /// Returns references to values where the timestamp is within `window_duration` of now.
    pub fn samples_in_window(&self) -> impl Iterator<Item = &T> {
        let now = Instant::now();
        let cutoff = now.checked_sub(self.window_duration).unwrap_or(now);
        self.samples
            .iter()
            .filter_map(|opt| opt.as_ref())
            .filter(move |(ts, _)| *ts >= cutoff)
            .map(|(_, v)| v)
    }

    /// Get samples in window with a specific reference time (for testing).
    pub fn samples_in_window_at(&self, now: Instant) -> impl Iterator<Item = &T> {
        let cutoff = now.checked_sub(self.window_duration).unwrap_or(now);
        self.samples
            .iter()
            .filter_map(|opt| opt.as_ref())
            .filter(move |(ts, _)| *ts >= cutoff)
            .map(|(_, v)| v)
    }

    /// Get the number of samples currently stored (up to capacity).
    pub fn len(&self) -> usize {
        self.written
    }

    /// Check if the window is empty.
    pub fn is_empty(&self) -> bool {
        self.written == 0
    }

    /// Get the capacity of the window.
    pub fn capacity(&self) -> usize {
        self.samples.len()
    }

    /// Clear all samples.
    pub fn clear(&mut self) {
        for slot in &mut self.samples {
            *slot = None;
        }
        self.head = 0;
        self.written = 0;
        self.cached_stats = None;
    }
}

impl RollingWindow<f64> {
    /// Compute statistics for f64 samples in the current window.
    ///
    /// Results are cached until the next push.
    pub fn stats(&mut self) -> Stats {
        if let Some(cached) = self.cached_stats {
            return cached;
        }

        let samples: Vec<f64> = self.samples_in_window().copied().collect();
        let stats = compute_stats(&samples);
        self.cached_stats = Some(stats);
        stats
    }

    /// Compute statistics without caching (for use with custom time).
    pub fn stats_at(&self, now: Instant) -> Stats {
        let samples: Vec<f64> = self.samples_in_window_at(now).copied().collect();
        compute_stats(&samples)
    }
}

impl RollingWindow<Duration> {
    /// Compute statistics for Duration samples, converting to milliseconds.
    ///
    /// Results are cached until the next push.
    pub fn stats_ms(&mut self) -> Stats {
        if let Some(cached) = self.cached_stats {
            return cached;
        }

        let samples: Vec<f64> = self
            .samples_in_window()
            .map(|d| d.as_secs_f64() * 1000.0)
            .collect();
        let stats = compute_stats(&samples);
        self.cached_stats = Some(stats);
        stats
    }

    /// Compute statistics without caching (for use with custom time).
    pub fn stats_ms_at(&self, now: Instant) -> Stats {
        let samples: Vec<f64> = self
            .samples_in_window_at(now)
            .map(|d| d.as_secs_f64() * 1000.0)
            .collect();
        compute_stats(&samples)
    }
}

/// Compute statistics for a slice of f64 values.
fn compute_stats(samples: &[f64]) -> Stats {
    if samples.is_empty() {
        return Stats::empty();
    }

    let count = samples.len();
    let sum: f64 = samples.iter().sum();
    let avg = sum / count as f64;

    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min = sorted[0];
    let max = sorted[count - 1];

    // P95: 95th percentile
    let p95_index = ((count as f64) * 0.95).ceil() as usize;
    let p95 = sorted[p95_index.saturating_sub(1).min(count - 1)];

    Stats {
        avg,
        p95,
        max,
        min,
        count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_window_basic() {
        let mut window: RollingWindow<f64> = RollingWindow::new(10, Duration::from_secs(10));
        assert!(window.is_empty());
        assert_eq!(window.len(), 0);

        window.push(1.0);
        window.push(2.0);
        window.push(3.0);

        assert!(!window.is_empty());
        assert_eq!(window.len(), 3);
    }

    #[test]
    fn test_rolling_window_ring_buffer() {
        let mut window: RollingWindow<f64> = RollingWindow::new(3, Duration::from_secs(10));

        window.push(1.0);
        window.push(2.0);
        window.push(3.0);
        assert_eq!(window.len(), 3);

        // Push beyond capacity - oldest should be overwritten
        window.push(4.0);
        assert_eq!(window.len(), 3); // Still 3, ring buffer wraps
    }

    #[test]
    fn test_stats_basic() {
        let mut window: RollingWindow<f64> = RollingWindow::new(100, Duration::from_secs(60));

        for i in 1..=10 {
            window.push(i as f64);
        }

        let stats = window.stats();
        assert_eq!(stats.count, 10);
        assert!((stats.avg - 5.5).abs() < 0.001);
        assert!((stats.min - 1.0).abs() < 0.001);
        assert!((stats.max - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_stats_empty() {
        let mut window: RollingWindow<f64> = RollingWindow::new(10, Duration::from_secs(10));
        let stats = window.stats();
        assert_eq!(stats.count, 0);
        assert!((stats.avg - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_p95_calculation() {
        let mut window: RollingWindow<f64> = RollingWindow::new(100, Duration::from_secs(60));

        // Push 100 values: 1 to 100
        for i in 1..=100 {
            window.push(i as f64);
        }

        let stats = window.stats();
        assert_eq!(stats.count, 100);
        // P95 of 1-100 should be 95
        assert!((stats.p95 - 95.0).abs() < 1.0);
    }

    #[test]
    fn test_duration_stats() {
        let mut window: RollingWindow<Duration> = RollingWindow::new(100, Duration::from_secs(60));

        window.push(Duration::from_millis(10));
        window.push(Duration::from_millis(20));
        window.push(Duration::from_millis(30));

        let stats = window.stats_ms();
        assert_eq!(stats.count, 3);
        assert!((stats.avg - 20.0).abs() < 0.001);
        assert!((stats.min - 10.0).abs() < 0.001);
        assert!((stats.max - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_with_defaults() {
        let window: RollingWindow<f64> = RollingWindow::with_defaults();
        assert_eq!(window.capacity(), DEFAULT_CAPACITY);
    }
}
