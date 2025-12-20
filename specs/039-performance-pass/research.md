# Research: Performance Pass

**Feature**: 039-performance-pass
**Date**: 2025-12-19

## Overview

Research findings for implementing production-grade performance optimization in Plix.

---

## 1. Current Performance Infrastructure

### 1.1 Tick System

**Location**: `crates/plix-server/src/tick.rs`

**Decision**: Use existing 60 TPS (16.67ms) as baseline
**Rationale**: Already configured with 20-60 Hz range, stable implementation
**Alternatives Considered**:
- Fixed 30 TPS: Rejected - reduces responsiveness for competitive gameplay
- Variable TPS: Rejected - adds complexity, constitution requires stability

**Current Implementation**:
```rust
pub struct TickConfig {
    pub rate: u8,        // 20-60, default 60
    pub duration: Duration,
}
```

### 1.2 Existing Metrics

**Location**: `crates/plix-server/src/metrics.rs`

**Decision**: Extend `ServerMetricsCollector` rather than replace
**Rationale**:
- Already has rolling window (10s), O(1) recording, periodic logging (5s)
- Tracks tick avg/p95/max, packets/bandwidth per session
- Well-integrated with main loop

**Current Capabilities**:
- Tick processing time (avg, p95, max)
- Network: pps_in/out, kbps_in/out
- Session metrics: RTT, jitter, packet loss

**Gaps to Fill**:
- Per-subsystem timing breakdown
- Allocation tracking
- Per-message-type network stats
- Meshing time tracking

### 1.3 Tracing Infrastructure

**Decision**: Use existing `tracing` crate (v0.1) with additional spans
**Rationale**: Already integrated, minimal overhead when disabled
**Alternatives Considered**:
- pprof-rs: Requires nightly, violates constitution
- flamegraph: Good for manual profiling, not automated CI
- custom: More work, less ecosystem support

**Approach**:
- Add `#[instrument]` spans on critical paths (feature-gated)
- Use `tracing-subscriber` for export to JSON/flame

---

## 2. Instrumentation Strategy

### 2.1 Subsystem Spans

**Decision**: Instrument 6 core subsystems
**Rationale**: Covers all major tick contributors identified in codebase

| Subsystem | Location | Span Name |
|-----------|----------|-----------|
| Tick Total | `lib.rs:tick()` | `tick_total` |
| Simulation | `lib.rs:simulate_tick()` | `simulation` |
| Network Encode | `netloop.rs` | `net_encode` |
| Network Decode | `netloop.rs` | `net_decode` |
| Mods Dispatch | `mods/` | `mods_dispatch` |
| Meshing | `chunk_mesher.rs` | `meshing` |

### 2.2 Allocation Tracking

**Decision**: Feature-gated, manual profiling for MVP
**Rationale**:
- jemalloc stats require linking changes
- dhat requires nightly
- heaptrack is external tool
- For MVP: identify hotspots via code review + targeted profiling

**Approach**:
1. Use `tracing` to mark allocation-heavy functions
2. Run with heaptrack/valgrind externally for baseline
3. Implement buffer reuse based on findings
4. Verify reduction with same external tool

**Known Hotspots** (from code review):
- `bincode::serialize` in snapshot encoding (per-tick, full state)
- `ChunkMesher::build()` creates new Vec per chunk
- String formatting in debug logs

### 2.3 Network Instrumentation

**Decision**: Instrument at encode/decode boundary per message type
**Rationale**: Bincode serialization is synchronous, easy to wrap

**Implementation**:
```rust
// In protocol/messages.rs or netloop.rs
fn encode_with_stats<T: Serialize>(msg: &T, stats: &mut NetStats) -> Vec<u8> {
    let bytes = bincode::serialize(msg).unwrap();
    stats.record(msg.type_id(), bytes.len());
    bytes
}
```

**Metrics per Message Type**:
- count/s
- avg_bytes
- p95_bytes
- total_kb/s

---

## 3. Optimization Techniques

### 3.1 Allocation Reduction

**Decision**: Focus on buffer reuse patterns
**Rationale**: Structural fix, no micro-optimization

**Technique 1: Reusable Encode Buffer**
```rust
struct EncoderState {
    buffer: Vec<u8>,  // Reused across ticks
}

impl EncoderState {
    fn encode<T: Serialize>(&mut self, msg: &T) -> &[u8] {
        self.buffer.clear();
        bincode::serialize_into(&mut self.buffer, msg).unwrap();
        &self.buffer
    }
}
```

**Technique 2: Mesh Buffer Pool**
- Pre-allocate vertex/index buffers
- Clear and reuse rather than reallocate
- Cap pool size to prevent memory bloat

### 3.2 Network Bandwidth

**Decision**: Message batching + optional compression
**Rationale**: Simple, measurable, protocol-compatible

**Technique 1: Message Batching**
- Collect small messages (<64 bytes) within tick
- Send as single batched packet
- Reduces per-packet overhead

**Technique 2: Compression (>1KB)**
- Use lz4 (fast) for payloads >1024 bytes
- Header byte indicates compression
- Backward compatible (version bump if needed)

**Technique 3: Quantization** (optional)
- Positions: 3 * f32 (12 bytes) → 3 * i16 (6 bytes) with mm precision
- Angles: f32 (4 bytes) → u16 (2 bytes) with 0.01° precision
- Requires protocol version bump

### 3.3 Meshing Optimization

**Decision**: Dirty chunk tracking + per-tick budget
**Rationale**: Current implementation re-meshes entire chunk on any edit

**Technique 1: Dirty Chunk Set**
```rust
struct ChunkManager {
    dirty_chunks: HashSet<ChunkPos>,
    mesh_budget_ms: f32,  // 2-4ms default
}
```

**Technique 2: Budgeted Remeshing**
- Process dirty chunks in priority order (distance to player)
- Stop when budget exceeded
- Defer remaining to next tick

**Technique 3: Coalescing**
- Multiple edits to same chunk in one tick → single remesh
- Already implicit with dirty set

---

## 4. Report Format

### 4.1 Schema Design

**Decision**: Flat JSON with nested sections
**Rationale**: Easy to parse, diff, and extend

```json
{
  "metadata": {
    "timestamp": "2025-12-19T10:30:00Z",
    "git_sha": "abc123",
    "build_mode": "release",
    "scenario": "world_churn",
    "duration_secs": 60
  },
  "tick_stats": {
    "count": 3600,
    "avg_ms": 8.5,
    "p50_ms": 7.2,
    "p95_ms": 12.1,
    "p99_ms": 15.8,
    "max_ms": 22.3,
    "overruns": 5
  },
  "subsystem_stats": {
    "simulation": { "avg_ms": 2.1, "p95_ms": 3.5 },
    "net_encode": { "avg_ms": 1.2, "p95_ms": 2.0 },
    "net_decode": { "avg_ms": 0.8, "p95_ms": 1.5 },
    "mods_dispatch": { "avg_ms": 0.5, "p95_ms": 1.0 },
    "meshing": { "avg_ms": 3.2, "p95_ms": 6.0 }
  },
  "net_stats": {
    "total_kb_in": 1024.5,
    "total_kb_out": 2048.3,
    "by_message_type": {
      "PlayerSnapshot": { "count": 3600, "avg_bytes": 128, "p95_bytes": 256 },
      "WorldSnapshot": { "count": 120, "avg_bytes": 4096, "p95_bytes": 8192 }
    }
  },
  "meshing_stats": {
    "chunks_built": 150,
    "avg_ms_per_chunk": 0.8,
    "p95_ms_per_chunk": 1.5,
    "deferred_count": 12
  },
  "alloc_stats": {
    "enabled": false,
    "note": "Use external profiler (heaptrack) for allocation data"
  }
}
```

---

## 5. Harness Design

### 5.1 Scenario Definitions

**Scenario A: Idle**
- Server started, no players
- Minimal world loaded (1 chunk)
- Duration: 60 seconds
- Purpose: Baseline overhead

**Scenario D: World Churn**
- Simulated chunk loading/unloading
- Block edits triggering remesh
- Duration: 60 seconds
- Purpose: Meshing stress

**Scenario E: Net Stress**
- Multiple simulated players
- Rapid movement updates
- Duration: 60 seconds
- Purpose: Network bandwidth

### 5.2 Harness Binary

**Decision**: Standalone binary in `benches/`
**Rationale**: Simpler than test framework, explicit execution

```bash
# Usage
cargo run --release --bin plix-perf -- --scenario idle --duration 60 --output perf_report.json
```

---

## 6. CI Integration

**Decision**: Artifact generation only for MVP
**Rationale**: Automated comparison adds complexity, defer to later

**Workflow**:
1. CI runs perf harness on merge to main
2. Uploads `perf_report.json` as artifact
3. Manual comparison to baseline
4. (Future) Automated threshold checks

---

## 7. Dependencies

### Required (already in workspace)
- `tracing` 0.1 - instrumentation
- `serde` 1.0 - JSON serialization
- `bincode` 1.3 - binary encoding

### Optional (feature-gated)
- `lz4_flex` - compression (if implementing net optimization)
- `tracing-flame` - flame graph export (dev only)

### External Tools (not dependencies)
- heaptrack - allocation profiling
- perf/flamegraph - CPU profiling

---

## Summary

| Topic | Decision | Confidence |
|-------|----------|------------|
| TPS Baseline | 60 TPS (16.67ms) | High |
| Instrumentation | Extend ServerMetricsCollector | High |
| Tracing | Use existing `tracing` crate | High |
| Allocation Tracking | Feature-gated, external tools for MVP | Medium |
| Buffer Reuse | Encode buffer + mesh pool | High |
| Network Opt | Batching + compression (>1KB) | High |
| Meshing Opt | Dirty set + budget | High |
| Report Format | JSON with stable schema | High |
| CI Integration | Artifact only, no auto-compare | High |
