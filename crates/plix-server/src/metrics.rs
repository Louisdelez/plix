//! Server metrics collection and logging
//!
//! Provides tick-time metrics with periodic logging (default: every 5 seconds).
//! Uses RollingWindow from plix-common for time-based statistics.

use std::time::{Duration, Instant};
use tracing::info;

use plix_common::metrics::RollingWindow;

/// Default logging interval (5 seconds)
pub const DEFAULT_LOG_INTERVAL: Duration = Duration::from_secs(5);

/// Server-side metrics collector for tick performance monitoring.
///
/// Records tick processing times and periodically logs statistics.
/// Uses a 10-second rolling window for metrics calculation.
#[derive(Debug)]
pub struct ServerMetricsCollector {
    /// Rolling window of tick processing times
    tick_times: RollingWindow<Duration>,
    /// Last time metrics were logged
    last_log: Instant,
    /// Interval between log outputs
    log_interval: Duration,
    /// Packets received since last log
    packets_in: u64,
    /// Packets sent since last log
    packets_out: u64,
    /// Bytes received since last log
    bytes_in: u64,
    /// Bytes sent since last log
    bytes_out: u64,
}

impl ServerMetricsCollector {
    /// Create a new metrics collector with default settings.
    ///
    /// The first call to `maybe_log` will log immediately.
    pub fn new() -> Self {
        // Set last_log in the past so first maybe_log will trigger
        let last_log = Instant::now()
            .checked_sub(DEFAULT_LOG_INTERVAL)
            .unwrap_or_else(Instant::now);
        Self {
            tick_times: RollingWindow::with_defaults(),
            last_log,
            log_interval: DEFAULT_LOG_INTERVAL,
            packets_in: 0,
            packets_out: 0,
            bytes_in: 0,
            bytes_out: 0,
        }
    }

    /// Create a metrics collector with custom log interval.
    pub fn with_log_interval(log_interval: Duration) -> Self {
        // Set last_log in the past so first maybe_log will trigger
        let last_log = Instant::now()
            .checked_sub(log_interval)
            .unwrap_or_else(Instant::now);
        Self {
            tick_times: RollingWindow::with_defaults(),
            last_log,
            log_interval,
            packets_in: 0,
            packets_out: 0,
            bytes_in: 0,
            bytes_out: 0,
        }
    }

    /// Record a tick's processing duration.
    ///
    /// This is O(1) and does not allocate.
    #[inline]
    pub fn record_tick(&mut self, duration: Duration) {
        self.tick_times.push(duration);
    }

    /// Record an incoming packet.
    ///
    /// # Arguments
    /// * `bytes` - Size of the packet in bytes
    #[inline]
    pub fn record_packet_in(&mut self, bytes: usize) {
        self.packets_in += 1;
        self.bytes_in += bytes as u64;
    }

    /// Record an outgoing packet.
    ///
    /// # Arguments
    /// * `bytes` - Size of the packet in bytes
    #[inline]
    pub fn record_packet_out(&mut self, bytes: usize) {
        self.packets_out += 1;
        self.bytes_out += bytes as u64;
    }

    /// Check if it's time to log metrics and do so if needed.
    ///
    /// Logs tick-time statistics (avg, p95, max) along with player/session counts
    /// and packet/bandwidth metrics.
    /// Only logs if `log_interval` has elapsed since last log.
    ///
    /// Returns `true` if metrics were logged.
    pub fn maybe_log(&mut self, players: usize, sessions: usize) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_log);
        if elapsed < self.log_interval {
            return false;
        }

        let elapsed_secs = elapsed.as_secs_f64();
        self.last_log = now;

        let stats = self.tick_times.stats_ms();

        // Calculate PPS (packets per second)
        let pps_in = if elapsed_secs > 0.0 {
            self.packets_in as f64 / elapsed_secs
        } else {
            0.0
        };
        let pps_out = if elapsed_secs > 0.0 {
            self.packets_out as f64 / elapsed_secs
        } else {
            0.0
        };

        // Calculate bandwidth in KB/s
        let kbps_in = if elapsed_secs > 0.0 {
            (self.bytes_in as f64 / 1024.0) / elapsed_secs
        } else {
            0.0
        };
        let kbps_out = if elapsed_secs > 0.0 {
            (self.bytes_out as f64 / 1024.0) / elapsed_secs
        } else {
            0.0
        };

        if stats.count > 0 {
            info!(
                tick_avg_ms = format!("{:.2}", stats.avg),
                tick_p95_ms = format!("{:.2}", stats.p95),
                tick_max_ms = format!("{:.2}", stats.max),
                tick_count = stats.count,
                players = players,
                sessions = sessions,
                pps_in = format!("{:.1}", pps_in),
                pps_out = format!("{:.1}", pps_out),
                kbps_in = format!("{:.1}", kbps_in),
                kbps_out = format!("{:.1}", kbps_out),
                "Server metrics"
            );
        } else {
            info!(
                players = players,
                sessions = sessions,
                pps_in = format!("{:.1}", pps_in),
                pps_out = format!("{:.1}", pps_out),
                kbps_in = format!("{:.1}", kbps_in),
                kbps_out = format!("{:.1}", kbps_out),
                "Server metrics (no tick samples)"
            );
        }

        // Reset counters for next interval
        self.packets_in = 0;
        self.packets_out = 0;
        self.bytes_in = 0;
        self.bytes_out = 0;

        true
    }

    /// Get the current tick time statistics (for testing).
    pub fn tick_stats_ms(&mut self) -> plix_common::metrics::Stats {
        self.tick_times.stats_ms()
    }

    /// Check if it's time to log and return true if so.
    ///
    /// Use this when you want to conditionally log additional session metrics.
    pub fn should_log_now(&self) -> bool {
        Instant::now().duration_since(self.last_log) >= self.log_interval
    }

    /// Log per-session network metrics summary.
    ///
    /// Call this after `maybe_log` returns true to log per-player network stats.
    pub fn log_session_metrics<'a, I>(sessions: I)
    where
        I: Iterator<Item = (u16, &'a str, f64, f64, f64)>, // (id, name, rtt_ms, jitter_ms, loss_pct)
    {
        for (player_id, name, rtt_ms, jitter_ms, loss_pct) in sessions {
            info!(
                player_id = player_id,
                name = name,
                rtt_ms = format!("{:.1}", rtt_ms),
                jitter_ms = format!("{:.1}", jitter_ms),
                loss_pct = format!("{:.1}", loss_pct),
                "Session network metrics"
            );
        }
    }
}

impl Default for ServerMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_tick() {
        let mut collector = ServerMetricsCollector::new();
        collector.record_tick(Duration::from_micros(500));
        collector.record_tick(Duration::from_micros(600));
        collector.record_tick(Duration::from_micros(700));

        let stats = collector.tick_stats_ms();
        assert_eq!(stats.count, 3);
        assert!((stats.avg - 0.6).abs() < 0.01); // ~0.6ms average
    }

    #[test]
    fn test_maybe_log_respects_interval() {
        // Use a very short interval for testing
        let mut collector = ServerMetricsCollector::with_log_interval(Duration::from_millis(10));
        collector.record_tick(Duration::from_micros(500));

        // Should log immediately (first call)
        assert!(collector.maybe_log(2, 2));

        // Should not log again immediately
        assert!(!collector.maybe_log(2, 2));

        // Wait for interval
        std::thread::sleep(Duration::from_millis(15));

        // Should log now
        assert!(collector.maybe_log(2, 2));
    }

    #[test]
    fn test_empty_stats() {
        let mut collector = ServerMetricsCollector::new();
        let stats = collector.tick_stats_ms();
        assert_eq!(stats.count, 0);
    }
}
