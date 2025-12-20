# Tick Budgets and Backpressure

This document explains the tick budget system and backpressure mechanisms in Plix.

## Tick Budget Overview

Plix targets 60 ticks per second (TPS), giving each tick a budget of **16.67ms**. The budget system ensures the server maintains stable performance by:

1. Tracking time spent in each subsystem
2. Warning when budgets are exceeded
3. Triggering backpressure to shed load when necessary

## Budget Configuration

Default budgets are defined in `PerfConfig`:

```rust
BudgetConfig {
    tick_budget_ms: 16.67,      // Total budget per tick
    net_budget_ms: 2.0,         // Network encoding
    meshing_budget_ms: 3.0,     // Chunk meshing
    mods_budget_ms: 1.5,        // Mod execution
    simulation_budget_ms: 8.0,  // Game simulation
}
```

### Budget Allocation

| Subsystem | Budget | Purpose |
|-----------|--------|---------|
| Simulation | 8.0ms | Physics, AI, game logic |
| Meshing | 3.0ms | Chunk mesh generation |
| Net Encode | 2.0ms | Message serialization |
| Mods | 1.5ms | WASM mod execution |
| Overhead | 2.17ms | Frame overhead, scheduling |

## Thresholds

The tick budget system uses two threshold levels:

### Warning Threshold (80%)

- Triggered when tick time > 13.3ms
- Logs warning message
- No automatic action taken
- Useful for identifying trending issues

### Critical Threshold (100%)

- Triggered when tick time > 16.67ms
- Logs error with subsystem breakdown
- Triggers backpressure mechanisms
- Counts as a "tick overrun"

## Backpressure Mechanisms

When critical threshold is exceeded, the following mechanisms activate:

### 1. Meshing Backpressure

The chunk manager reduces meshing work when under pressure:

```rust
// In ChunkManager::update()
if tick_budget.is_critical() {
    // Skip non-essential mesh updates
    // Prioritize visible chunks only
    // Defer distant chunk meshes
}
```

**Effect**: Reduces mesh quality temporarily to maintain tick rate.

### 2. Network Backpressure

High-frequency messages are rate-limited:

```rust
// In NetLoop::send_updates()
if tick_budget.is_critical() {
    // Coalesce position updates
    // Skip redundant state syncs
    // Prioritize critical messages (damage, events)
}
```

**Effect**: Reduces bandwidth but may increase client-side interpolation.

### 3. Mod Execution Limits

WASM mod execution is capped:

```rust
// In ModRuntime::tick()
if tick_budget.remaining_ms() < mods_budget_ms {
    // Defer non-critical mod callbacks
    // Execute only high-priority events
}
```

**Effect**: Mods may miss some events during heavy load.

## Monitoring Budget Usage

### Per-Tick Tracking

```rust
let budget = TickBudget::new(16.67);
budget.start();

// ... simulation work ...
budget.record_subsystem("simulation", elapsed);

// ... meshing work ...
budget.record_subsystem("meshing", elapsed);

if budget.is_overrun() {
    let breakdown = budget.get_breakdown();
    tracing::warn!("Tick overrun: {:?}", breakdown);
}
```

### Report Output

The performance report includes budget violations:

```json
{
  "tick_stats": {
    "overruns": 5,
    "avg_ms": 12.3,
    "p95_ms": 15.8
  },
  "subsystem_stats": {
    "simulation": { "avg_ms": 6.2, "p95_ms": 9.1 },
    "meshing": { "avg_ms": 4.1, "p95_ms": 8.2 },
    "net_encode": { "avg_ms": 1.5, "p95_ms": 2.3 }
  }
}
```

## Tuning Budgets

### Increasing a Subsystem Budget

If a subsystem consistently exceeds its budget:

1. Check if it's a real bottleneck (use `--scenario` targeting that subsystem)
2. Increase its budget in config
3. Reduce another subsystem's budget to compensate
4. Verify total still fits in 16.67ms

### Adjusting Thresholds

For servers with lower TPS targets:

```rust
let config = PerfConfig {
    tick_rate: 30,  // 30 TPS = 33.33ms budget
    budget: BudgetConfig {
        tick_budget_ms: 33.33,
        // ... other budgets can be increased proportionally
    },
    ..Default::default()
};
```

## Best Practices

1. **Monitor in production**: Enable periodic budget logging
2. **Test under load**: Use `net_stress` scenario with realistic player counts
3. **Profile bottlenecks**: Use subsystem breakdown to identify hot paths
4. **Gradual degradation**: Backpressure should be graceful, not abrupt
5. **Alert on overruns**: Set up monitoring for `overruns > 0`

## Troubleshooting

### High Overrun Count

**Symptoms**: Many ticks exceed budget, poor player experience.

**Diagnosis**:
```bash
cargo run --release --bin plix-perf --features perf -- \
  --scenario world_churn \
  --duration 60 \
  --output diagnosis.json

jq '.subsystem_stats | to_entries | sort_by(.value.p95_ms) | reverse' diagnosis.json
```

**Solutions**:
1. Optimize the slowest subsystem
2. Reduce load (fewer players, smaller view distance)
3. Increase tick budget (lower TPS target)

### Inconsistent Performance

**Symptoms**: p99 much higher than p95, sporadic spikes.

**Diagnosis**: Check for garbage collection, memory pressure, or background tasks.

**Solutions**:
1. Pre-allocate buffers (use `BufferPool`)
2. Reduce allocation in hot paths
3. Profile with heaptrack for allocation hotspots

### Backpressure Too Aggressive

**Symptoms**: Quality degrades too quickly under light load.

**Solutions**:
1. Increase warning threshold
2. Tune backpressure sensitivity
3. Add hysteresis (delay before engaging backpressure)
