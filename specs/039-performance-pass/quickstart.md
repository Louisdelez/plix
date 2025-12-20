# Quickstart: Performance Profiling

**Feature**: 039-performance-pass
**Date**: 2025-12-19

## Overview

This guide shows how to run performance profiling scenarios and interpret the results.

---

## Prerequisites

1. **Build in release mode** (required for meaningful measurements):
   ```bash
   cargo build --release
   ```

2. **Verify tracing is enabled** (check Cargo.toml):
   ```toml
   [features]
   perf = ["tracing/max_level_trace"]
   ```

---

## Running Profiling Scenarios

### Quick Start

```bash
# Run the perf harness with default settings (Idle scenario, 60 seconds)
cargo run --release --bin plix-perf

# Output: perf_report.json in current directory
```

### Available Scenarios

| Scenario | Description | Duration | Use Case |
|----------|-------------|----------|----------|
| `idle` | Empty server, minimal world | 60s | Baseline overhead |
| `world_churn` | Chunk loading + block edits | 60s | Meshing stress |
| `net_stress` | Simulated players, rapid movement | 60s | Network bandwidth |

### Running Specific Scenarios

```bash
# Idle scenario (baseline)
cargo run --release --bin plix-perf -- --scenario idle --duration 60

# World churn (meshing stress)
cargo run --release --bin plix-perf -- --scenario world_churn --duration 120

# Network stress (bandwidth)
cargo run --release --bin plix-perf -- --scenario net_stress --duration 60 --players 16
```

### Output Options

```bash
# Custom output file
cargo run --release --bin plix-perf -- --output my_report.json

# Pretty-printed JSON
cargo run --release --bin plix-perf -- --pretty

# Console summary only (no file)
cargo run --release --bin plix-perf -- --no-file
```

---

## Interpreting Results

### Key Metrics

| Metric | Target | Warning | Critical |
|--------|--------|---------|----------|
| `tick_stats.avg_ms` | < 10ms | > 12ms | > 16ms |
| `tick_stats.p95_ms` | < 12ms | > 14ms | > 16ms |
| `tick_stats.p99_ms` | < 14ms | > 16ms | > 20ms |
| `tick_stats.overruns` | 0 | > 10 | > 100 |

### Reading the Report

```json
{
  "tick_stats": {
    "avg_ms": 8.5,     // ✅ Good: under 10ms
    "p95_ms": 12.1,    // ⚠️ Warning: close to limit
    "p99_ms": 15.8,    // ⚠️ Warning: approaching critical
    "overruns": 5      // ⚠️ Some budget violations
  }
}
```

### Subsystem Analysis

Look at `subsystem_stats` to find bottlenecks:

```json
{
  "subsystem_stats": {
    "simulation": { "avg_ms": 2.1, "p95_ms": 3.5 },   // OK
    "net_encode": { "avg_ms": 1.2, "p95_ms": 2.0 },   // OK
    "meshing": { "avg_ms": 3.2, "p95_ms": 6.0 }       // ⚠️ High variance
  }
}
```

**Interpretation**: Meshing has high p95, indicating spike potential.

### Network Analysis

Check `net_stats.by_message_type` for bandwidth hogs:

```json
{
  "by_message_type": {
    "WorldSnapshot": {
      "count": 120,
      "avg_bytes": 4096,  // Large messages
      "p95_bytes": 8192   // Even larger at p95
    }
  }
}
```

**Interpretation**: WorldSnapshot is a compression candidate (>1KB).

---

## Comparing Results

### Before/After Comparison

```bash
# Run baseline
cargo run --release --bin plix-perf -- --output baseline.json

# Apply optimization...

# Run again
cargo run --release --bin plix-perf -- --output optimized.json

# Compare (manual for now)
jq '.tick_stats' baseline.json
jq '.tick_stats' optimized.json
```

### Key Comparisons

| Metric | Formula | Good Result |
|--------|---------|-------------|
| p99 improvement | `(baseline.p99 - optimized.p99) / baseline.p99` | > 20% |
| Bandwidth reduction | `(baseline.kb_out - optimized.kb_out) / baseline.kb_out` | > 10% |
| Overrun reduction | `baseline.overruns - optimized.overruns` | ≥ 0 |

---

## Advanced Usage

### With Allocation Tracking

Allocation tracking requires external tools:

```bash
# Using heaptrack (Linux)
heaptrack cargo run --release --bin plix-perf -- --scenario idle
heaptrack --analyze heaptrack.plix-perf.*.zst

# Using dhat (requires nightly + feature)
RUSTFLAGS="-C target-cpu=native" cargo +nightly run --release --features dhat-heap --bin plix-perf
```

### With CPU Profiling

```bash
# Using perf (Linux)
perf record -g cargo run --release --bin plix-perf -- --scenario world_churn
perf report

# Generate flamegraph
perf script | inferno-collapse-perf | inferno-flamegraph > flamegraph.svg
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PLIX_PERF_SCENARIO` | Override scenario | `idle` |
| `PLIX_PERF_DURATION` | Override duration (seconds) | `60` |
| `RUST_LOG` | Tracing verbosity | `info` |

---

## Troubleshooting

### "Tick overrun" warnings

**Cause**: Tick time exceeded budget (16.67ms for 60 TPS).

**Solutions**:
1. Check `subsystem_stats` for the slowest subsystem
2. Run in release mode (`--release`)
3. Reduce player count or world complexity

### Low sample count

**Cause**: Scenario duration too short.

**Solution**: Increase duration (`--duration 120`).

### Inconsistent results

**Cause**: Background processes, thermal throttling.

**Solutions**:
1. Close other applications
2. Run multiple times and average
3. Use `--duration 300` for statistical stability

---

## Next Steps

1. **Establish baseline**: Run `idle` scenario and save as reference
2. **Identify bottlenecks**: Look at subsystem p95/p99 values
3. **Apply optimizations**: Focus on highest-impact areas
4. **Verify improvements**: Compare before/after reports
5. **Set up CI**: Add harness to CI pipeline for regression detection

---

## Related Documentation

- [Scenarios](../../docs/perf/scenarios.md) - Detailed scenario descriptions
- [Budgets](../../docs/perf/budgets.md) - Budget configuration guide
- [How to Profile](../../docs/perf/how-to-profile.md) - Full profiling guide
