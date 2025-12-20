# CI Integration: Performance Profiling

This document describes how to use the performance profiling harness in CI.

## Workflow Overview

The performance workflow (`.github/workflows/perf.yml`) runs automatically on:
- Push to main/master branches
- Pull requests targeting main/master
- Manual dispatch via GitHub Actions UI

## Artifacts

Each run produces `perf_report_*.json` files as artifacts:
- `perf_report_idle.json` - Baseline overhead scenario
- `perf_report_world_churn.json` - Meshing stress scenario
- `perf_report_net_stress.json` - Network bandwidth scenario

Artifacts are retained for 30 days.

## Comparing Results

### Manual Comparison

1. Download artifacts from both runs (baseline and current)
2. Use `jq` to compare key metrics:

```bash
# Compare p95 tick times
echo "Baseline:"
jq '.tick_stats.p95_ms' baseline/perf_report_idle.json

echo "Current:"
jq '.tick_stats.p95_ms' current/perf_report_idle.json

# Calculate improvement
baseline=$(jq '.tick_stats.p95_ms' baseline/perf_report_idle.json)
current=$(jq '.tick_stats.p95_ms' current/perf_report_idle.json)
improvement=$(echo "scale=2; ($baseline - $current) / $baseline * 100" | bc)
echo "Improvement: ${improvement}%"
```

### Key Metrics to Compare

| Metric | Good | Warning | Critical |
|--------|------|---------|----------|
| `tick_stats.p95_ms` | < 12ms | > 12ms | > 15ms |
| `tick_stats.p99_ms` | < 15ms | > 15ms | > 18ms |
| `tick_stats.overruns` | 0 | > 10 | > 100 |

### Subsystem Breakdown

Check `subsystem_stats` for bottleneck identification:

```bash
jq '.subsystem_stats | to_entries | sort_by(.value.p95_ms) | reverse | .[0:3]' perf_report.json
```

## Regression Prevention

### Threshold Checks

The `regression-check` job warns if p95 exceeds 15ms on any scenario.

To enforce hard failure, add `--threshold-p95` flag:

```yaml
- name: Run with threshold
  run: |
    cargo run --release --bin plix-perf --features perf -- \
      --scenario idle \
      --duration 60 \
      --threshold-p95 12.0 \
      --output perf_report_idle.json
```

### Manual Dispatch

You can manually trigger the workflow with custom parameters:
1. Go to Actions > Performance Profiling
2. Click "Run workflow"
3. Set scenario (idle, world_churn, net_stress, or all)
4. Set duration in seconds

## Local Development

Run the harness locally before pushing:

```bash
# Quick sanity check
cargo run --release --bin plix-perf --features perf -- \
  --scenario idle --duration 30

# Full test with threshold
cargo run --release --bin plix-perf --features perf -- \
  --scenario idle --duration 60 --threshold-p95 12.0
```

## Troubleshooting

### High Variance Results

If results vary significantly between runs:
1. Increase duration (`--duration 120`)
2. Run multiple times and average
3. Check for background processes (CI runners can be noisy)

### Build Failures

Ensure the `perf` feature is enabled:
```bash
cargo build --release --bin plix-perf --features perf
```

### Missing Artifacts

Artifacts expire after 30 days. For long-term tracking:
1. Download artifacts before expiration
2. Store in a separate performance tracking repository
3. Consider using a time-series database for historical data
