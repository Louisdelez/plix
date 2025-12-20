//! Performance metrics collection with O(1) insertion
//!
//! Provides histogram-based metrics collection with percentile calculation.

use std::collections::HashMap;

/// Fixed-size ring buffer histogram for O(1) insertion and percentile calculation
#[derive(Debug)]
pub struct Histogram {
    /// Ring buffer of samples
    samples: Vec<f64>,
    /// Current write position
    write_pos: usize,
    /// Number of samples recorded (up to capacity)
    count: usize,
    /// Capacity of the ring buffer
    capacity: usize,
}

impl Histogram {
    /// Create a new histogram with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: vec![0.0; capacity],
            write_pos: 0,
            count: 0,
            capacity,
        }
    }

    /// Create a histogram with default capacity (600 samples = 10 seconds at 60 TPS)
    pub fn with_defaults() -> Self {
        Self::new(600)
    }

    /// Record a sample in milliseconds
    #[inline]
    pub fn record(&mut self, value_ms: f64) {
        self.samples[self.write_pos] = value_ms;
        self.write_pos = (self.write_pos + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    /// Record a duration
    #[inline]
    pub fn record_duration(&mut self, duration: std::time::Duration) {
        self.record(duration.as_secs_f64() * 1000.0);
    }

    /// Get the number of samples recorded
    pub fn count(&self) -> usize {
        self.count
    }

    /// Check if the histogram is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Calculate statistics from the histogram
    pub fn stats(&self) -> HistogramStats {
        if self.count == 0 {
            return HistogramStats::default();
        }

        // Get actual samples and sort for percentile calculation
        let mut sorted: Vec<f64> = self.samples[..self.count].to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let sum: f64 = sorted.iter().sum();
        let avg = sum / self.count as f64;

        let p50 = percentile(&sorted, 50.0);
        let p95 = percentile(&sorted, 95.0);
        let p99 = percentile(&sorted, 99.0);
        let max = sorted.last().copied().unwrap_or(0.0);

        HistogramStats {
            count: self.count as u64,
            avg,
            p50,
            p95,
            p99,
            max,
        }
    }

    /// Clear all samples
    pub fn clear(&mut self) {
        self.write_pos = 0;
        self.count = 0;
    }
}

/// Calculate percentile from sorted slice
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }

    let rank = (p / 100.0) * (sorted.len() - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    let frac = rank - lower as f64;

    if upper >= sorted.len() {
        sorted[sorted.len() - 1]
    } else {
        sorted[lower] * (1.0 - frac) + sorted[upper] * frac
    }
}

/// Statistics computed from a histogram
#[derive(Debug, Clone, Default)]
pub struct HistogramStats {
    /// Number of samples
    pub count: u64,
    /// Average value
    pub avg: f64,
    /// 50th percentile (median)
    pub p50: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
    /// Maximum value
    pub max: f64,
}

/// Per-subsystem metrics tracking
#[derive(Debug)]
pub struct SubsystemMetrics {
    /// Histograms for each subsystem
    histograms: HashMap<String, Histogram>,
    /// Invocation counts
    invocations: HashMap<String, u64>,
}

impl SubsystemMetrics {
    /// Create a new subsystem metrics tracker
    pub fn new() -> Self {
        Self {
            histograms: HashMap::new(),
            invocations: HashMap::new(),
        }
    }

    /// Record timing for a subsystem
    pub fn record(&mut self, name: &str, ms: f64) {
        let histogram = self
            .histograms
            .entry(name.to_string())
            .or_insert_with(Histogram::with_defaults);
        histogram.record(ms);

        *self.invocations.entry(name.to_string()).or_insert(0) += 1;
    }

    /// Record a duration for a subsystem
    pub fn record_duration(&mut self, name: &str, duration: std::time::Duration) {
        self.record(name, duration.as_secs_f64() * 1000.0);
    }

    /// Get stats for a specific subsystem
    pub fn stats(&self, name: &str) -> Option<SubsystemStats> {
        let histogram = self.histograms.get(name)?;
        let stats = histogram.stats();
        let invocations = self.invocations.get(name).copied().unwrap_or(0);

        Some(SubsystemStats {
            name: name.to_string(),
            avg_ms: stats.avg,
            p95_ms: stats.p95,
            p99_ms: stats.p99,
            invocations,
        })
    }

    /// Get all subsystem stats
    pub fn all_stats(&self) -> Vec<SubsystemStats> {
        self.histograms
            .keys()
            .filter_map(|name| self.stats(name))
            .collect()
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        for histogram in self.histograms.values_mut() {
            histogram.clear();
        }
        self.invocations.clear();
    }
}

impl Default for SubsystemMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a single subsystem
#[derive(Debug, Clone)]
pub struct SubsystemStats {
    /// Subsystem name
    pub name: String,
    /// Average time per invocation in milliseconds
    pub avg_ms: f64,
    /// 95th percentile time in milliseconds
    pub p95_ms: f64,
    /// 99th percentile time in milliseconds
    pub p99_ms: f64,
    /// Total invocation count
    pub invocations: u64,
}

/// Network message statistics
#[derive(Debug, Default)]
pub struct NetMessageStats {
    /// Message counts by type
    counts: HashMap<String, u64>,
    /// Total bytes by type
    bytes_total: HashMap<String, u64>,
    /// Size histograms by type
    size_histograms: HashMap<String, Histogram>,
}

impl NetMessageStats {
    /// Create a new network message stats tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a message of the given type and size
    pub fn record(&mut self, msg_type: &str, bytes: usize) {
        *self.counts.entry(msg_type.to_string()).or_insert(0) += 1;
        *self.bytes_total.entry(msg_type.to_string()).or_insert(0) += bytes as u64;

        let histogram = self
            .size_histograms
            .entry(msg_type.to_string())
            .or_insert_with(|| Histogram::new(100));
        histogram.record(bytes as f64);
    }

    /// Get stats for a specific message type
    pub fn stats(&self, msg_type: &str) -> Option<MessageTypeStats> {
        let count = self.counts.get(msg_type)?;
        let total_bytes = self.bytes_total.get(msg_type).copied().unwrap_or(0);
        let histogram = self.size_histograms.get(msg_type)?;
        let size_stats = histogram.stats();

        Some(MessageTypeStats {
            message_type: msg_type.to_string(),
            count: *count,
            total_bytes,
            avg_bytes: size_stats.avg,
            p95_bytes: size_stats.p95 as u64,
            max_bytes: size_stats.max as u64,
        })
    }

    /// Get all message type stats
    pub fn all_stats(&self) -> Vec<MessageTypeStats> {
        self.counts
            .keys()
            .filter_map(|msg_type| self.stats(msg_type))
            .collect()
    }

    /// Get total bytes in/out
    pub fn total_bytes(&self) -> u64 {
        self.bytes_total.values().sum()
    }

    /// Clear all stats
    pub fn clear(&mut self) {
        self.counts.clear();
        self.bytes_total.clear();
        for histogram in self.size_histograms.values_mut() {
            histogram.clear();
        }
    }
}

/// Statistics for a single message type
#[derive(Debug, Clone)]
pub struct MessageTypeStats {
    /// Message type name
    pub message_type: String,
    /// Total message count
    pub count: u64,
    /// Total bytes
    pub total_bytes: u64,
    /// Average message size in bytes
    pub avg_bytes: f64,
    /// 95th percentile size in bytes
    pub p95_bytes: u64,
    /// Maximum message size in bytes
    pub max_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_empty() {
        let histogram = Histogram::with_defaults();
        assert!(histogram.is_empty());
        let stats = histogram.stats();
        assert_eq!(stats.count, 0);
    }

    #[test]
    fn test_histogram_single() {
        let mut histogram = Histogram::with_defaults();
        histogram.record(10.0);
        let stats = histogram.stats();
        assert_eq!(stats.count, 1);
        assert!((stats.avg - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_histogram_percentiles() {
        let mut histogram = Histogram::new(100);
        for i in 1..=100 {
            histogram.record(i as f64);
        }
        let stats = histogram.stats();
        assert_eq!(stats.count, 100);
        assert!((stats.avg - 50.5).abs() < 0.1);
        assert!((stats.p50 - 50.0).abs() < 1.0);
        assert!((stats.p95 - 95.0).abs() < 1.0);
        assert!((stats.p99 - 99.0).abs() < 1.0);
        assert!((stats.max - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_histogram_ring_buffer() {
        let mut histogram = Histogram::new(10);
        // Fill beyond capacity
        for i in 1..=20 {
            histogram.record(i as f64);
        }
        // Should only have last 10 values (11-20)
        assert_eq!(histogram.count(), 10);
        let stats = histogram.stats();
        assert!((stats.avg - 15.5).abs() < 0.1);
    }

    #[test]
    fn test_subsystem_metrics() {
        let mut metrics = SubsystemMetrics::new();
        metrics.record("simulation", 5.0);
        metrics.record("simulation", 6.0);
        metrics.record("net_encode", 2.0);

        let sim_stats = metrics.stats("simulation").unwrap();
        assert_eq!(sim_stats.invocations, 2);
        assert!((sim_stats.avg_ms - 5.5).abs() < 0.1);

        let net_stats = metrics.stats("net_encode").unwrap();
        assert_eq!(net_stats.invocations, 1);
    }

    #[test]
    fn test_net_message_stats() {
        let mut stats = NetMessageStats::new();
        stats.record("PlayerSnapshot", 128);
        stats.record("PlayerSnapshot", 256);
        stats.record("WorldSnapshot", 4096);

        let ps_stats = stats.stats("PlayerSnapshot").unwrap();
        assert_eq!(ps_stats.count, 2);
        assert_eq!(ps_stats.total_bytes, 384);
        assert!((ps_stats.avg_bytes - 192.0).abs() < 0.1);

        let ws_stats = stats.stats("WorldSnapshot").unwrap();
        assert_eq!(ws_stats.count, 1);
        assert_eq!(ws_stats.total_bytes, 4096);
    }
}
