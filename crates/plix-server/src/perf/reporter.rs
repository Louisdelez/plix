//! Performance report generation
//!
//! Generates structured JSON reports from profiling runs.

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::perf::metrics::{HistogramStats, MessageTypeStats, SubsystemStats};

/// Complete performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfReport {
    /// Run metadata
    pub metadata: Metadata,
    /// Tick timing statistics
    pub tick_stats: TickStats,
    /// Per-subsystem timing breakdown
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub subsystem_stats: HashMap<String, SubsystemStatsReport>,
    /// Network statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_stats: Option<NetStats>,
    /// Meshing statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meshing_stats: Option<MeshingStats>,
    /// Allocation statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alloc_stats: Option<AllocStats>,
}

/// Report metadata for reproducibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// ISO 8601 timestamp of run start
    pub timestamp: String,
    /// Git commit hash or "unknown"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_sha: Option<String>,
    /// Build mode (debug/release)
    pub build_mode: String,
    /// Scenario name
    pub scenario: String,
    /// Duration in seconds
    pub duration_secs: u32,
    /// Configured tick rate
    pub tick_rate: u8,
    /// Number of players
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_count: Option<u32>,
}

/// Tick timing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickStats {
    /// Total ticks recorded
    pub count: u64,
    /// Average tick time in milliseconds
    pub avg_ms: f64,
    /// Median tick time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p50_ms: Option<f64>,
    /// 95th percentile tick time in milliseconds
    pub p95_ms: f64,
    /// 99th percentile tick time in milliseconds
    pub p99_ms: f64,
    /// Maximum tick time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ms: Option<f64>,
    /// Number of ticks exceeding budget
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overruns: Option<u64>,
}

/// Per-subsystem statistics for report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemStatsReport {
    /// Average time in milliseconds
    pub avg_ms: f64,
    /// 95th percentile time in milliseconds
    pub p95_ms: f64,
    /// 99th percentile time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p99_ms: Option<f64>,
    /// Number of invocations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invocations: Option<u64>,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetStats {
    /// Total inbound bandwidth in KB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_kb_in: Option<f64>,
    /// Total outbound bandwidth in KB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_kb_out: Option<f64>,
    /// Average inbound rate in KB/s
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_kbps_in: Option<f64>,
    /// Average outbound rate in KB/s
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_kbps_out: Option<f64>,
    /// Per-message-type breakdown
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub by_message_type: HashMap<String, MessageTypeStatsReport>,
}

/// Per-message-type statistics for report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTypeStatsReport {
    /// Total message count
    pub count: u64,
    /// Total bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes: Option<u64>,
    /// Average message size in bytes
    pub avg_bytes: f64,
    /// 95th percentile size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p95_bytes: Option<u64>,
    /// Maximum message size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bytes: Option<u64>,
}

/// Meshing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshingStats {
    /// Total chunks meshed
    pub chunks_built: u64,
    /// Average mesh time per chunk in milliseconds
    pub avg_ms_per_chunk: f64,
    /// 95th percentile mesh time per chunk in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p95_ms_per_chunk: Option<f64>,
    /// Maximum mesh time per chunk in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ms_per_chunk: Option<f64>,
    /// Chunks deferred due to budget
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deferred_count: Option<u64>,
    /// Edits coalesced (saved remeshes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coalesced_count: Option<u64>,
}

/// Allocation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocStats {
    /// Whether allocation tracking was enabled
    pub enabled: bool,
    /// Total allocations (if enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_allocs: Option<u64>,
    /// Total bytes allocated (if enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes: Option<u64>,
    /// Allocations per second (if enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocs_per_sec: Option<f64>,
    /// Top allocation hotspots
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub hotspots: Vec<AllocHotspot>,
    /// Note when disabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Allocation hotspot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocHotspot {
    /// Source location
    pub location: String,
    /// Number of allocations
    pub alloc_count: u64,
    /// Total bytes allocated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes: Option<u64>,
    /// Percentage of total allocations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<f64>,
}

impl PerfReport {
    /// Create a new report with the given metadata
    pub fn new(metadata: Metadata, tick_stats: TickStats) -> Self {
        Self {
            metadata,
            tick_stats,
            subsystem_stats: HashMap::new(),
            net_stats: None,
            meshing_stats: None,
            alloc_stats: None,
        }
    }

    /// Add subsystem stats from collected metrics
    pub fn with_subsystem_stats(mut self, stats: Vec<SubsystemStats>) -> Self {
        for stat in stats {
            self.subsystem_stats.insert(
                stat.name.clone(),
                SubsystemStatsReport {
                    avg_ms: stat.avg_ms,
                    p95_ms: stat.p95_ms,
                    p99_ms: Some(stat.p99_ms),
                    invocations: Some(stat.invocations),
                },
            );
        }
        self
    }

    /// Add network stats
    pub fn with_net_stats(mut self, net_stats: NetStats) -> Self {
        self.net_stats = Some(net_stats);
        self
    }

    /// Add meshing stats
    pub fn with_meshing_stats(mut self, meshing_stats: MeshingStats) -> Self {
        self.meshing_stats = Some(meshing_stats);
        self
    }

    /// Add allocation stats (disabled placeholder)
    pub fn with_alloc_stats_disabled(mut self) -> Self {
        self.alloc_stats = Some(AllocStats {
            enabled: false,
            total_allocs: None,
            total_bytes: None,
            allocs_per_sec: None,
            hotspots: Vec::new(),
            note: Some("Use external profiler (heaptrack) for allocation data".to_string()),
        });
        self
    }

    /// Write report to JSON file
    pub fn write_to_file(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl Metadata {
    /// Create metadata for the current run
    pub fn new(scenario: &str, duration_secs: u32, tick_rate: u8) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            git_sha: get_git_sha(),
            build_mode: if cfg!(debug_assertions) {
                "debug".to_string()
            } else {
                "release".to_string()
            },
            scenario: scenario.to_string(),
            duration_secs,
            tick_rate,
            player_count: None,
        }
    }

    /// Set player count
    pub fn with_player_count(mut self, count: u32) -> Self {
        self.player_count = Some(count);
        self
    }
}

impl TickStats {
    /// Create tick stats from histogram stats
    pub fn from_histogram(stats: &HistogramStats, overruns: Option<u64>) -> Self {
        Self {
            count: stats.count,
            avg_ms: stats.avg,
            p50_ms: Some(stats.p50),
            p95_ms: stats.p95,
            p99_ms: stats.p99,
            max_ms: Some(stats.max),
            overruns,
        }
    }
}

impl NetStats {
    /// Create empty net stats
    pub fn new() -> Self {
        Self {
            total_kb_in: None,
            total_kb_out: None,
            avg_kbps_in: None,
            avg_kbps_out: None,
            by_message_type: HashMap::new(),
        }
    }

    /// Add message type stats
    pub fn add_message_type(&mut self, stats: MessageTypeStats) {
        self.by_message_type.insert(
            stats.message_type.clone(),
            MessageTypeStatsReport {
                count: stats.count,
                total_bytes: Some(stats.total_bytes),
                avg_bytes: stats.avg_bytes,
                p95_bytes: Some(stats.p95_bytes),
                max_bytes: Some(stats.max_bytes),
            },
        );
    }
}

impl Default for NetStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the current git SHA, or None if not in a git repo
fn get_git_sha() -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let metadata = Metadata::new("idle", 60, 60);
        assert_eq!(metadata.scenario, "idle");
        assert_eq!(metadata.duration_secs, 60);
        assert_eq!(metadata.tick_rate, 60);
        assert!(metadata.timestamp.len() > 0);
    }

    #[test]
    fn test_tick_stats_from_histogram() {
        let histogram_stats = HistogramStats {
            count: 3600,
            avg: 8.5,
            p50: 7.2,
            p95: 12.1,
            p99: 15.8,
            max: 22.3,
        };
        let tick_stats = TickStats::from_histogram(&histogram_stats, Some(5));
        assert_eq!(tick_stats.count, 3600);
        assert!((tick_stats.avg_ms - 8.5).abs() < 0.001);
        assert_eq!(tick_stats.overruns, Some(5));
    }

    #[test]
    fn test_report_serialization() {
        let metadata = Metadata::new("idle", 60, 60);
        let tick_stats = TickStats {
            count: 3600,
            avg_ms: 8.5,
            p50_ms: Some(7.2),
            p95_ms: 12.1,
            p99_ms: 15.8,
            max_ms: Some(22.3),
            overruns: Some(5),
        };
        let report = PerfReport::new(metadata, tick_stats);
        let json = report.to_json().unwrap();
        assert!(json.contains("\"scenario\": \"idle\""));
        assert!(json.contains("\"count\": 3600"));
    }

    #[test]
    fn test_report_with_subsystems() {
        let metadata = Metadata::new("world_churn", 60, 60);
        let tick_stats = TickStats {
            count: 3600,
            avg_ms: 8.5,
            p50_ms: None,
            p95_ms: 12.1,
            p99_ms: 15.8,
            max_ms: None,
            overruns: None,
        };
        let subsystems = vec![
            SubsystemStats {
                name: "simulation".to_string(),
                avg_ms: 2.1,
                p95_ms: 3.5,
                p99_ms: 4.0,
                invocations: 3600,
            },
            SubsystemStats {
                name: "net_encode".to_string(),
                avg_ms: 1.2,
                p95_ms: 2.0,
                p99_ms: 2.5,
                invocations: 7200,
            },
        ];
        let report = PerfReport::new(metadata, tick_stats).with_subsystem_stats(subsystems);
        assert_eq!(report.subsystem_stats.len(), 2);
        assert!(report.subsystem_stats.contains_key("simulation"));
    }
}
