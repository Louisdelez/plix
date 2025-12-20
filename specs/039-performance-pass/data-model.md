# Data Model: Performance Pass

**Feature**: 039-performance-pass
**Date**: 2025-12-19

## Overview

Data structures for performance profiling, reporting, and budget management.

---

## Core Entities

### PerfReport

The structured output of a profiling run.

| Field | Type | Description |
|-------|------|-------------|
| metadata | Metadata | Run context (timestamp, git sha, build, scenario) |
| tick_stats | TickStats | Aggregate tick timing statistics |
| subsystem_stats | Map<String, SubsystemStats> | Per-subsystem timing breakdown |
| net_stats | NetStats | Network bandwidth and message statistics |
| meshing_stats | MeshingStats | Chunk meshing performance data |
| alloc_stats | AllocStats | Allocation tracking (if enabled) |

**Lifecycle**: Created at scenario end, serialized to JSON, immutable after creation.

---

### Metadata

Run context for reproducibility.

| Field | Type | Description |
|-------|------|-------------|
| timestamp | DateTime | ISO 8601 timestamp of run start |
| git_sha | String | Git commit hash (or "unknown") |
| build_mode | String | "debug" or "release" |
| scenario | String | Scenario name (e.g., "idle", "world_churn") |
| duration_secs | u32 | Total scenario duration |
| tick_rate | u8 | Configured TPS (20-60) |
| player_count | u32 | Number of connected/simulated players |

---

### TickStats

Aggregate tick timing statistics.

| Field | Type | Description |
|-------|------|-------------|
| count | u64 | Total ticks recorded |
| avg_ms | f64 | Mean tick time |
| p50_ms | f64 | Median tick time |
| p95_ms | f64 | 95th percentile |
| p99_ms | f64 | 99th percentile |
| max_ms | f64 | Maximum tick time |
| overruns | u64 | Ticks exceeding budget |

**Derived from**: Rolling window samples in `ServerMetricsCollector`.

---

### SubsystemStats

Per-subsystem timing breakdown.

| Field | Type | Description |
|-------|------|-------------|
| name | String | Subsystem identifier |
| avg_ms | f64 | Mean time per tick |
| p95_ms | f64 | 95th percentile time |
| p99_ms | f64 | 99th percentile time |
| invocations | u64 | Number of times invoked |

**Tracked Subsystems**:
- `simulation` - Player movement, combat, physics
- `net_encode` - Message serialization
- `net_decode` - Message deserialization
- `net_flush` - Packet transmission
- `mods_dispatch` - Mod event handling
- `meshing` - Chunk mesh generation (client)
- `streaming` - Chunk loading/unloading

---

### NetStats

Network bandwidth and message statistics.

| Field | Type | Description |
|-------|------|-------------|
| total_kb_in | f64 | Total inbound bandwidth (KB) |
| total_kb_out | f64 | Total outbound bandwidth (KB) |
| avg_kbps_in | f64 | Average inbound rate (KB/s) |
| avg_kbps_out | f64 | Average outbound rate (KB/s) |
| by_message_type | Map<String, MessageTypeStats> | Per-type breakdown |

---

### MessageTypeStats

Statistics for a single message type.

| Field | Type | Description |
|-------|------|-------------|
| message_type | String | Message type name (e.g., "PlayerSnapshot") |
| count | u64 | Total messages sent/received |
| total_bytes | u64 | Total bytes |
| avg_bytes | f64 | Average message size |
| p95_bytes | u64 | 95th percentile size |
| max_bytes | u64 | Maximum size |

---

### MeshingStats

Chunk meshing performance (client-side).

| Field | Type | Description |
|-------|------|-------------|
| chunks_built | u64 | Total chunks meshed |
| avg_ms_per_chunk | f64 | Mean mesh time per chunk |
| p95_ms_per_chunk | f64 | 95th percentile mesh time |
| max_ms_per_chunk | f64 | Maximum mesh time |
| deferred_count | u64 | Chunks deferred due to budget |
| coalesced_count | u64 | Edits coalesced (saved remeshes) |

---

### AllocStats

Allocation tracking (optional, feature-gated).

| Field | Type | Description |
|-------|------|-------------|
| enabled | bool | Whether tracking was active |
| total_allocs | u64 | Total allocations (if enabled) |
| total_bytes | u64 | Total bytes allocated (if enabled) |
| allocs_per_sec | f64 | Allocation rate (if enabled) |
| hotspots | Vec<AllocHotspot> | Top allocation sites (if enabled) |
| note | String | Instructions if disabled |

---

### AllocHotspot

Allocation hotspot information.

| Field | Type | Description |
|-------|------|-------------|
| location | String | Source location (file:line or function) |
| alloc_count | u64 | Number of allocations |
| total_bytes | u64 | Total bytes allocated |
| percentage | f64 | Percentage of total allocations |

---

## Configuration Entities

### TickBudget

Configuration for tick budget and backpressure.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| target_ms | f64 | 16.67 | Target tick duration (1000/TPS) |
| net_budget_ms | f64 | 2.0 | Network subsystem budget |
| meshing_budget_ms | f64 | 3.0 | Meshing subsystem budget |
| mods_budget_ms | f64 | 1.5 | Mods dispatch budget |
| warn_threshold_ms | f64 | 14.0 | Log warning if exceeded |
| critical_threshold_ms | f64 | 16.0 | Engage backpressure if exceeded |

---

### PerfScenario

Definition of a reproducible performance scenario.

| Field | Type | Description |
|-------|------|-------------|
| name | String | Scenario identifier |
| description | String | Human-readable description |
| duration_secs | u32 | How long to run |
| player_count | u32 | Simulated/required players |
| world_config | WorldConfig | World generation parameters |
| actions | Vec<ScenarioAction> | Actions to simulate |

**Predefined Scenarios**:
- `idle` - Empty server, minimal world
- `world_churn` - Rapid chunk loading, block edits
- `net_stress` - Multiple players, rapid movement

---

### ScenarioAction

An action to perform during a scenario.

| Variant | Fields | Description |
|---------|--------|-------------|
| Wait | duration_secs | Pause execution |
| SpawnPlayers | count | Add simulated players |
| MovePlayer | player_id, direction | Trigger movement |
| EditBlock | pos, block_id | Trigger block edit |
| LoadChunks | count | Force chunk loading |

---

## Relationships

```
PerfReport
├── 1:1 Metadata
├── 1:1 TickStats
├── 1:N SubsystemStats (keyed by name)
├── 1:1 NetStats
│   └── 1:N MessageTypeStats (keyed by type)
├── 1:1 MeshingStats
└── 1:1 AllocStats
    └── 1:N AllocHotspot

PerfScenario
├── 1:1 WorldConfig
└── 1:N ScenarioAction
```

---

## Validation Rules

### PerfReport
- `tick_stats.count > 0` (at least one tick recorded)
- `tick_stats.avg_ms >= 0`
- All percentiles: `p50 <= p95 <= p99 <= max`

### TickBudget
- `target_ms > 0`
- `warn_threshold_ms < target_ms`
- All subsystem budgets positive and sum < target_ms

### PerfScenario
- `name` matches `^[a-z][a-z0-9_]*$`
- `duration_secs >= 10` (minimum meaningful duration)

---

## State Transitions

### Scenario Execution States

```
[Idle] → Start → [Running] → Complete → [Finished]
                     ↓
                  Cancel → [Aborted]
```

### Metric Collection States

```
[Disabled] ←→ [Enabled]
                 ↓
            [Recording] → [Aggregating] → [Exported]
```

---

## Notes

- All timing values in milliseconds (f64) for JSON consistency
- All byte counts as u64 to handle large scenarios
- Feature-gated fields marked with `enabled: bool` pattern
- Schema versioned in metadata for forward compatibility
