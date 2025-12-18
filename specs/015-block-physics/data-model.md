# Data Model: Block Physics Light

**Feature**: 015-block-physics
**Date**: 2025-12-16

## Overview

This document defines the data structures and their relationships for the block physics system.

---

## Entities

### PhysicsConfig

Configuration for the physics system, stored per world/server.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `gravity_enabled` | bool | true | Enable gravity physics for affected blocks |
| `liquids_enabled` | bool | false | Enable liquid spreading (optional feature) |
| `max_events_per_tick` | u32 | 100 | Maximum physics events processed per tick |
| `max_liquid_spread_distance` | u8 | 7 | Maximum horizontal spread for liquids |

**Validation Rules**:
- `max_events_per_tick >= 1` (0 effectively disables physics)
- `max_liquid_spread_distance >= 1 && <= 15`

**Serialization**: TOML for config files, bincode for network

---

### PhysicsEventKind

Enumeration of physics event types.

| Variant | Description |
|---------|-------------|
| `Fall` | Block should check if it can fall |
| `LiquidSpread { depth: u8 }` | Liquid should try to spread, with current depth |

**Discriminant Values** (for determinism):
- Fall = 0
- LiquidSpread = 1

---

### PhysicsEvent

A single physics update to be processed.

| Field | Type | Description |
|-------|------|-------------|
| `pos` | BlockPos | World position of the block |
| `kind` | PhysicsEventKind | Type of physics update |

**Size**: ~16 bytes (BlockPos: 12 bytes + kind: 4 bytes with discriminant)

**Identity**: Two events are considered duplicates if `(pos, kind.discriminant())` match

---

### PhysicsQueue

Bounded FIFO queue for pending physics events.

| Field | Type | Description |
|-------|------|-------------|
| `events` | VecDeque<PhysicsEvent> | Ordered event queue |
| `pending` | HashSet<(BlockPos, u8)> | Deduplication set (pos + kind discriminant) |

**Operations**:
| Method | Complexity | Description |
|--------|------------|-------------|
| `push(event)` | O(1) | Add event if not duplicate |
| `pop()` | O(1) | Remove and return front event |
| `len()` | O(1) | Current queue size |
| `clear()` | O(n) | Remove all events |

**Invariants**:
- `events.len() == pending.len()` (always in sync)
- No duplicate `(pos, kind)` pairs in queue

---

### PhysicsMetrics

Observable counters for monitoring physics load.

| Field | Type | Description |
|-------|------|-------------|
| `events_processed_last_tick` | u32 | Events resolved in most recent tick |
| `queue_depth` | u32 | Current events waiting |
| `total_blocks_fallen` | u64 | Cumulative gravity moves (lifetime) |
| `total_liquid_updates` | u64 | Cumulative liquid spreads (lifetime) |

**Thread Safety**: Interior mutability via atomic operations (future-proofing)

---

### PhysicsSystem

Main physics simulation system (server-side only).

| Field | Type | Description |
|-------|------|-------------|
| `config` | PhysicsConfig | Current physics configuration |
| `queue` | PhysicsQueue | Pending events |
| `metrics` | PhysicsMetrics | Observable counters |

**Lifecycle**:
1. Created at server startup with config
2. Events pushed when blocks modified
3. `tick()` called each server frame
4. Destroyed at server shutdown

---

## Relationships

```
PhysicsSystem
    ├── owns PhysicsConfig (1:1)
    ├── owns PhysicsQueue (1:1)
    │       └── contains PhysicsEvent (1:N)
    │               └── has PhysicsEventKind
    └── owns PhysicsMetrics (1:1)

PhysicsEvent
    └── references BlockPos (from plix-common)
            └── computed from ChunkedWorld position

ChunkedWorld (existing)
    └── modified by PhysicsSystem.tick()
            └── triggers new PhysicsEvents
```

---

## State Transitions

### PhysicsEvent Lifecycle

```
Created
    │
    v
Queued (in PhysicsQueue)
    │
    v (budget allows)
Processing
    │
    ├── Resolved (block moved/spread)
    │       └── May create new events
    │
    └── No-op (conditions not met)
            └── Event discarded
```

### Block State Under Gravity

```
Placed (gravity-affected)
    │
    ├── Air below? → Fall event queued
    │                    │
    │                    v
    │               Falling (each tick)
    │                    │
    │                    ├── Still air below? → Continue falling
    │                    │
    │                    └── Solid below? → Landed (static)
    │
    └── Solid below? → Static (no event)
```

### Liquid Spreading States

```
Placed (liquid source)
    │
    v
Spreading (depth=0)
    │
    ├── Air below? → Flow down (reset depth)
    │
    └── No air below?
            │
            ├── depth < max? → Spread horizontally (depth+1)
            │
            └── depth >= max? → Stop spreading
```

---

## Integration with Existing Types

### BlockType Extension

Add method to existing `BlockType` in `plix-common/src/types.rs`:

```rust
impl BlockType {
    /// Check if this block is affected by gravity
    pub fn is_gravity_affected(&self) -> bool {
        matches!(*self, Self::SAND)
    }

    /// Check if this block is a liquid
    pub fn is_liquid(&self) -> bool {
        false // No liquid block types yet; extend when adding WATER
    }
}
```

### Future: WATER BlockType

When liquids are implemented:

```rust
pub const WATER: Self = Self(9);

pub fn is_liquid(&self) -> bool {
    matches!(*self, Self::WATER)
}
```

---

## Serialization Formats

### PhysicsConfig (TOML)

```toml
[physics]
gravity_enabled = true
liquids_enabled = false
max_events_per_tick = 100
max_liquid_spread_distance = 7
```

### PhysicsEvent (bincode for internal use)

Binary format, not exposed externally. Used only within server process.

---

## Memory Estimates

| Structure | Size | Max Count | Max Memory |
|-----------|------|-----------|------------|
| PhysicsEvent | 16 bytes | 10,000 (queue cap) | 160 KB |
| PhysicsQueue | ~40 bytes + events | 1 | 160 KB |
| PhysicsMetrics | 32 bytes | 1 | 32 bytes |
| PhysicsSystem | ~100 bytes + queue | 1 | ~161 KB |

**Total overhead**: <200 KB worst case (large cascade queued)
