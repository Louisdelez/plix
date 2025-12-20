# How to Profile Plix

This guide shows you how to run performance profiling for the Plix server.

## Prerequisites

1. Build in release mode (required for meaningful measurements):
   ```bash
   cargo build --release --features perf
   ```

2. Verify the perf feature is enabled in `Cargo.toml`:
   ```toml
   [features]
   perf = ["dep:serde_json", "dep:chrono"]
   ```

## Quick Start

```bash
# Run the perf harness with default settings (Idle scenario, 60 seconds)
cargo run --release --features perf --bin plix-perf

# Output: perf_report.json in current directory
```

## Command Line Options

```bash
cargo run --release --features perf --bin plix-perf -- [OPTIONS]

OPTIONS:
    --scenario <SCENARIO>     Scenario to run [default: idle]
                              Values: idle, world_churn, net_stress
    --duration <SECONDS>      Duration in seconds [default: 60]
    --output <PATH>           Output report file [default: perf_report.json]
    --tick-rate <TPS>         Tick rate 20-60 [default: 60]
    --players <COUNT>         Simulated players (for net_stress)
    --threshold-p95 <MS>      Fail if p95 exceeds threshold
    --no-file                 Print report to console only
    --pretty                  Pretty-print JSON output
    --log-level <LEVEL>       Log verbosity [default: info]
```

## Examples

### Run Idle Baseline

```bash
cargo run --release --features perf --bin plix-perf -- \
    --scenario idle \
    --duration 60 \
    --output baseline.json
```

### Run Meshing Stress Test

```bash
cargo run --release --features perf --bin plix-perf -- \
    --scenario world_churn \
    --duration 120 \
    --output meshing_test.json
```

### Run Network Stress Test

```bash
cargo run --release --features perf --bin plix-perf -- \
    --scenario net_stress \
    --duration 60 \
    --players 16 \
    --output network_test.json
```

### CI Threshold Check

```bash
cargo run --release --features perf --bin plix-perf -- \
    --scenario idle \
    --duration 60 \
    --threshold-p95 12.0
# Exits with code 1 if p95 > 12ms
```

## Understanding the Report

The report contains several sections:

### Metadata

```json
{
  "metadata": {
    "timestamp": "2025-12-19T10:30:00Z",
    "git_sha": "abc1234",
    "build_mode": "release",
    "scenario": "idle",
    "duration_secs": 60,
    "tick_rate": 60
  }
}
```

### Tick Statistics

```json
{
  "tick_stats": {
    "count": 3600,
    "avg_ms": 8.5,
    "p50_ms": 7.2,
    "p95_ms": 12.1,
    "p99_ms": 15.8,
    "max_ms": 22.3,
    "overruns": 5
  }
}
```

**Targets:**
- `avg_ms` < 10ms (good), > 12ms (warning)
- `p95_ms` < 12ms (good), > 15ms (critical)
- `p99_ms` < 15ms (good), > 18ms (critical)
- `overruns` = 0 (good), > 10 (warning)

### Subsystem Breakdown

```json
{
  "subsystem_stats": {
    "simulation": { "avg_ms": 2.1, "p95_ms": 3.5 },
    "net_encode": { "avg_ms": 1.2, "p95_ms": 2.0 },
    "meshing": { "avg_ms": 3.2, "p95_ms": 6.0 }
  }
}
```

Use this to identify which subsystem is the bottleneck.

## External Profiling Tools

For deeper analysis:

### CPU Profiling with perf

```bash
perf record -g cargo run --release --features perf --bin plix-perf -- --scenario world_churn
perf report

# Generate flamegraph
perf script | inferno-collapse-perf | inferno-flamegraph > flamegraph.svg
```

### Memory Profiling with heaptrack

```bash
heaptrack cargo run --release --features perf --bin plix-perf -- --scenario idle
heaptrack --analyze heaptrack.plix-perf.*.zst
```

## Comparing Results

```bash
# Run baseline
cargo run --release --features perf --bin plix-perf -- --output baseline.json

# Apply optimization...

# Run again
cargo run --release --features perf --bin plix-perf -- --output optimized.json

# Compare
jq '.tick_stats' baseline.json
jq '.tick_stats' optimized.json
```

## Troubleshooting

### "Tick overrun" warnings

**Cause**: Tick time exceeded budget (16.67ms for 60 TPS).

**Solutions**:
1. Check `subsystem_stats` for the slowest subsystem
2. Ensure running in release mode (`--release`)
3. Close other applications
4. Reduce complexity in the scenario

### Low sample count

**Cause**: Scenario duration too short.

**Solution**: Increase duration (`--duration 120`).

### Inconsistent results

**Cause**: Background processes, thermal throttling.

**Solutions**:
1. Close other applications
2. Run multiple times and average
3. Use `--duration 300` for statistical stability
