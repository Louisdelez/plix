//! Network metrics collection for Plix
//!
//! Tracks:
//! - Round-trip time (RTT)
//! - Jitter
//! - Packet loss

use std::collections::VecDeque;
use std::time::Duration;

/// Maximum samples to keep for averaging
const MAX_SAMPLES: usize = 64;

/// Network quality metrics
#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    /// RTT samples in microseconds
    rtt_samples: VecDeque<u32>,
    /// Packets sent
    packets_sent: u64,
    /// Packets received
    packets_received: u64,
    /// Packets lost (estimated from sequence gaps)
    packets_lost: u64,
    /// Bytes sent
    bytes_sent: u64,
    /// Bytes received
    bytes_received: u64,
}

impl NetworkMetrics {
    /// Create new metrics tracker
    pub fn new() -> Self {
        Self {
            rtt_samples: VecDeque::with_capacity(MAX_SAMPLES),
            packets_sent: 0,
            packets_received: 0,
            packets_lost: 0,
            bytes_sent: 0,
            bytes_received: 0,
        }
    }

    /// Record an RTT sample
    pub fn record_rtt(&mut self, rtt: Duration) {
        let micros = rtt.as_micros().min(u32::MAX as u128) as u32;

        if self.rtt_samples.len() >= MAX_SAMPLES {
            self.rtt_samples.pop_front();
        }
        self.rtt_samples.push_back(micros);
    }

    /// Record a sent packet
    pub fn record_send(&mut self, bytes: usize) {
        self.packets_sent += 1;
        self.bytes_sent += bytes as u64;
    }

    /// Record a received packet
    pub fn record_recv(&mut self, bytes: usize) {
        self.packets_received += 1;
        self.bytes_received += bytes as u64;
    }

    /// Record packet loss
    pub fn record_loss(&mut self, count: u64) {
        self.packets_lost += count;
    }

    /// Get average RTT
    pub fn rtt_avg(&self) -> Duration {
        if self.rtt_samples.is_empty() {
            return Duration::from_millis(100); // Default estimate
        }

        let sum: u64 = self.rtt_samples.iter().map(|&x| x as u64).sum();
        let avg = sum / self.rtt_samples.len() as u64;
        Duration::from_micros(avg)
    }

    /// Get RTT variance (for jitter calculation)
    pub fn rtt_variance(&self) -> Duration {
        if self.rtt_samples.len() < 2 {
            return Duration::ZERO;
        }

        let avg = self.rtt_avg().as_micros() as f64;
        let variance: f64 = self
            .rtt_samples
            .iter()
            .map(|&x| {
                let diff = x as f64 - avg;
                diff * diff
            })
            .sum::<f64>()
            / (self.rtt_samples.len() - 1) as f64;

        Duration::from_micros(variance.sqrt() as u64)
    }

    /// Get jitter (standard deviation of RTT)
    pub fn jitter(&self) -> Duration {
        self.rtt_variance()
    }

    /// Get estimated packet loss percentage (0-100)
    pub fn packet_loss_pct(&self) -> f32 {
        let total = self.packets_received + self.packets_lost;
        if total == 0 {
            return 0.0;
        }
        (self.packets_lost as f32 / total as f32) * 100.0
    }

    /// Get total packets sent
    pub fn packets_sent(&self) -> u64 {
        self.packets_sent
    }

    /// Get total packets received
    pub fn packets_received(&self) -> u64 {
        self.packets_received
    }

    /// Get total bytes sent
    pub fn bytes_sent(&self) -> u64 {
        self.bytes_sent
    }

    /// Get total bytes received
    pub fn bytes_received(&self) -> u64 {
        self.bytes_received
    }

    /// Get RTT min (best case)
    pub fn rtt_min(&self) -> Duration {
        self.rtt_samples
            .iter()
            .min()
            .map(|&x| Duration::from_micros(x as u64))
            .unwrap_or(Duration::from_millis(100))
    }

    /// Get RTT max (worst case)
    pub fn rtt_max(&self) -> Duration {
        self.rtt_samples
            .iter()
            .max()
            .map(|&x| Duration::from_micros(x as u64))
            .unwrap_or(Duration::from_millis(100))
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.rtt_samples.clear();
        self.packets_sent = 0;
        self.packets_received = 0;
        self.packets_lost = 0;
        self.bytes_sent = 0;
        self.bytes_received = 0;
    }
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtt_tracking() {
        let mut metrics = NetworkMetrics::new();

        metrics.record_rtt(Duration::from_millis(50));
        metrics.record_rtt(Duration::from_millis(60));
        metrics.record_rtt(Duration::from_millis(70));

        let avg = metrics.rtt_avg();
        assert!(avg >= Duration::from_millis(59) && avg <= Duration::from_millis(61));
    }

    #[test]
    fn test_packet_counting() {
        let mut metrics = NetworkMetrics::new();

        metrics.record_send(100);
        metrics.record_send(200);
        metrics.record_recv(150);

        assert_eq!(metrics.packets_sent(), 2);
        assert_eq!(metrics.packets_received(), 1);
        assert_eq!(metrics.bytes_sent(), 300);
        assert_eq!(metrics.bytes_received(), 150);
    }

    #[test]
    fn test_packet_loss() {
        let mut metrics = NetworkMetrics::new();

        for _ in 0..90 {
            metrics.record_recv(100);
        }
        metrics.record_loss(10);

        let loss = metrics.packet_loss_pct();
        assert!((loss - 10.0).abs() < 0.1);
    }
}
