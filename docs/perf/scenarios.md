# Performance Scenarios

This document describes the reproducible performance scenarios used by the Plix profiling harness.

## Overview

The perf harness provides three predefined scenarios that stress different subsystems. Each scenario is deterministic given the same random seed, allowing for reproducible comparisons.

## Idle Scenario

**Purpose**: Measure baseline server overhead with no player activity.

**What it does**:
- Server runs empty tick loop
- No chunk loads/unloads
- No network messages (beyond keepalives)
- No block updates

**When to use**:
- Establishing baseline performance
- Measuring framework overhead
- Comparing optimizations to tick loop itself

**Expected metrics**:
- `avg_ms`: < 1ms
- `p95_ms`: < 2ms
- `overruns`: 0

**Command**:
```bash
cargo run --release --bin plix-perf --features perf -- \
  --scenario idle \
  --duration 60 \
  --output baseline.json
```

## World Churn Scenario

**Purpose**: Stress test chunk meshing and world updates.

**What it does**:
- Simulates players moving through the world
- Loads and unloads chunks continuously
- Triggers meshing operations at high frequency
- Generates block update events

**Parameters**:
- Chunk load rate: ~10 chunks/second
- Player movement: Random walk pattern
- Block updates: Sparse random modifications

**When to use**:
- Testing meshing performance
- Validating chunk load/unload efficiency
- Measuring world generation overhead

**Expected metrics**:
- `avg_ms`: < 8ms
- `p95_ms`: < 12ms
- `overruns`: < 10

**Subsystem focus**:
- `meshing`: Should be primary consumer
- `simulation`: Moderate usage
- `net_encode`: Low usage

**Command**:
```bash
cargo run --release --bin plix-perf --features perf -- \
  --scenario world_churn \
  --duration 120 \
  --output meshing_test.json
```

## Net Stress Scenario

**Purpose**: Stress test network encoding and bandwidth.

**What it does**:
- Simulates many concurrent players
- Generates high message throughput
- Tests serialization performance
- Measures bandwidth under load

**Parameters**:
- `--players <COUNT>`: Number of simulated players (default: 8)
- Message rate: ~100 messages/second per player
- Message types: Position updates, block changes, chat

**When to use**:
- Testing network subsystem capacity
- Validating serialization efficiency
- Measuring bandwidth per player

**Expected metrics** (8 players):
- `avg_ms`: < 6ms
- `p95_ms`: < 10ms
- `net_stats.bytes_per_sec`: < 100KB/s per player

**Subsystem focus**:
- `net_encode`: Should be primary consumer
- `simulation`: Moderate usage
- `meshing`: Low usage

**Command**:
```bash
cargo run --release --bin plix-perf --features perf -- \
  --scenario net_stress \
  --duration 60 \
  --players 16 \
  --output network_test.json
```

## Scenario Comparison

| Scenario | Primary Stress | Secondary Stress | Network Load |
|----------|---------------|------------------|--------------|
| Idle | None | None | Minimal |
| World Churn | Meshing | Simulation | Low |
| Net Stress | Net Encode | Simulation | High |

## Custom Scenarios

For custom scenarios, you can modify the scenario configuration programmatically:

```rust
use plix_server::perf::{PerfScenario, ScenarioRunner};

let scenario = PerfScenario::NetStress { player_count: 32 };
let runner = ScenarioRunner::new(scenario, seed);
```

## Determinism

All scenarios use a seeded random number generator for reproducibility:

```bash
# Same seed = same results
cargo run --release --bin plix-perf --features perf -- \
  --scenario world_churn \
  --seed 12345

# Different runs with same seed should produce identical tick sequences
```

## Combining with External Tools

For deeper analysis, combine scenarios with external profilers:

```bash
# CPU profiling
perf record -g cargo run --release --bin plix-perf --features perf -- --scenario world_churn
perf report

# Memory profiling
heaptrack cargo run --release --bin plix-perf --features perf -- --scenario idle
```

See `how-to-profile.md` for detailed external profiling instructions.
