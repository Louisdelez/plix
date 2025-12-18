# Research: Block Physics Light

**Feature**: 015-block-physics
**Date**: 2025-12-16

## Overview

This document consolidates research findings for implementing block physics in the plix voxel engine.

---

## R1: Event Queue Design for Voxel Physics

### Decision
Use a `VecDeque<PhysicsEvent>` with a `HashSet<(BlockPos, PhysicsEventKind)>` for O(1) duplicate detection.

### Rationale
- VecDeque provides O(1) push_back/pop_front for FIFO semantics
- HashSet prevents same block being queued multiple times for same event type
- Bounded by `max_events_per_tick` during processing, not queue size (allows cascade backlog)
- Memory efficient: events are small structs (BlockPos + enum = ~16 bytes)

### Alternatives Considered
| Alternative | Why Rejected |
|-------------|--------------|
| Priority queue | Adds complexity, no clear priority benefit for simple physics |
| Channel (mpsc) | Overkill for single-threaded game loop |
| Global scan each tick | Violates constitution II.4 (event-driven) |

---

## R2: Gravity Block Identification

### Decision
Add `is_gravity_affected()` method to `BlockType` that returns true for SAND.

### Rationale
- Minimal change to existing type
- Fast O(1) check
- Extensible: add more block types to match arm as needed
- No runtime configuration needed for v1 (hardcoded list is sufficient)

### Implementation
```rust
impl BlockType {
    pub fn is_gravity_affected(&self) -> bool {
        matches!(*self, Self::SAND)
    }
}
```

### Alternatives Considered
| Alternative | Why Rejected |
|-------------|--------------|
| Separate BlockPhysicsType enum | Over-engineering for v1 with only gravity |
| Config file with block IDs | Adds complexity, premature for initial feature |
| Bitflags in BlockType | Breaks current repr(u8) simplicity |

---

## R3: Physics Event Detection Strategy

### Decision
Detect physics events at the point of block modification (removal/placement).

### Rationale
- Precise: only affected blocks are checked
- Efficient: no global iteration
- Deterministic: same edit always triggers same events

### Detection Rules
1. **Block removed (→ AIR)**:
   - Check block above: if gravity-affected, queue Fall event
   - Check horizontal neighbors: if liquid, queue LiquidSpread event
2. **Block placed**:
   - If gravity-affected and air below: queue Fall event for self
   - If liquid: queue LiquidSpread event for self

---

## R4: Gravity Resolution Algorithm

### Decision
Step-based falling: move block down 1 cell per physics tick.

### Rationale
- Visual smoothness: players see blocks falling
- Predictable: 1 block per tick is easy to reason about
- Budget-friendly: each step is one event processed
- Cross-chunk safe: ChunkedWorld.set_block handles boundaries

### Algorithm
```
resolve_fall(pos):
    block = world.get_block(pos)
    if not block.is_gravity_affected(): return

    below = pos.with_y(pos.y - 1)
    if world.get_block(below) == AIR:
        world.set_block(pos, AIR)
        world.set_block(below, block)
        queue.push(Fall(below))  # Continue falling next tick
    # else: landed, stop
```

### Alternatives Considered
| Alternative | Why Rejected |
|-------------|--------------|
| Instant drop to ground | Less visual feedback, harder to budget cascades |
| Entity-based falling | Violates spec (blocks only), adds complexity |
| Async resolution | Non-deterministic, breaks multiplayer |

---

## R5: Liquid Spreading Algorithm

### Decision
Breadth-first flood-fill with depth tracking, vertical priority.

### Rationale
- BFS ensures even spreading (not biased by iteration order)
- Depth tracking limits spread range (spec: 7 blocks default)
- Vertical priority: water flows down before spreading horizontally
- Budget-bounded: each spread step is one event

### Algorithm
```
resolve_liquid_spread(pos, depth):
    if depth > max_spread_distance: return

    # Try down first (vertical priority)
    below = pos.with_y(pos.y - 1)
    if world.get_block(below) == AIR:
        world.set_block(below, WATER)
        queue.push(LiquidSpread(below, 0))  # Reset depth for vertical
        return

    # Spread horizontally
    for neighbor in [pos + X, pos - X, pos + Z, pos - Z]:
        if world.get_block(neighbor) == AIR:
            world.set_block(neighbor, WATER)
            queue.push(LiquidSpread(neighbor, depth + 1))
```

### Alternatives Considered
| Alternative | Why Rejected |
|-------------|--------------|
| Cellular automaton | Requires global iteration per tick |
| Pressure-based flow | Out of scope (spec: no pressure) |
| Source/flow distinction | Added complexity for v1 |

---

## R6: Budget Enforcement Strategy

### Decision
Process up to `budget` events per tick, leave remainder in queue.

### Rationale
- Simple: just count processed events
- Fair: FIFO ensures all events eventually processed
- Predictable: tick time stays bounded
- No event loss: queue persists between ticks

### Implementation
```rust
fn tick(&mut self, world: &mut ChunkedWorld) {
    let mut processed = 0;
    while processed < self.config.max_events_per_tick {
        let Some(event) = self.queue.pop() else { break };
        self.resolve(event, world);
        processed += 1;
    }
    self.metrics.events_processed = processed;
    self.metrics.queue_depth = self.queue.len();
}
```

---

## R7: Cross-Chunk Physics Handling

### Decision
Rely on existing `ChunkedWorld.get_block()` and `set_block()` APIs.

### Rationale
- ChunkedWorld already handles cross-chunk block access
- set_block returns affected chunks for dirty marking
- No special physics code needed for boundaries
- Tested in Feature 011 (chunked world)

### Verification
- ChunkedWorld.get_block(BlockPos) handles any coordinates
- ChunkedWorld.set_block(BlockPos) creates chunks if needed
- Block at chunk boundary (e.g., y=15) falling to y=14 in different chunk: works automatically

---

## R8: Server Integration Point

### Decision
Call `physics_system.tick()` in server game loop after movement, before snapshots.

### Rationale
- After movement: player edits are processed before physics
- Before snapshots: physics changes included in next client update
- Single tick per frame: deterministic

### Integration Location
```rust
// In Server::tick()
self.process_block_edits().await;  // Existing: detect physics events here
self.physics_system.tick(&mut self.world);  // NEW
self.send_snapshots().await;
```

---

## R9: Metrics Exposure

### Decision
Add PhysicsMetrics struct with atomic counters, expose via existing metrics system.

### Rationale
- Consistent with existing ServerMetricsCollector pattern
- Atomic for potential future threading
- Simple counters: processed, queue_depth, blocks_fallen, liquid_updates

### Counters
| Counter | Description |
|---------|-------------|
| `physics_events_processed` | Events resolved this tick |
| `physics_queue_depth` | Events waiting in queue |
| `physics_blocks_fallen` | Total gravity moves |
| `physics_liquid_updates` | Total liquid spreads |

---

## Summary

All technical decisions have been made. No NEEDS CLARIFICATION markers remain.

| Research Area | Decision |
|---------------|----------|
| Queue design | VecDeque + HashSet dedup |
| Block identification | `is_gravity_affected()` method |
| Event detection | At point of block modification |
| Gravity algorithm | Step-based, 1 cell per tick |
| Liquid algorithm | BFS flood-fill with depth limit |
| Budget enforcement | Count-based per tick |
| Cross-chunk | Use existing ChunkedWorld API |
| Server integration | After movement, before snapshots |
| Metrics | Atomic counters in PhysicsMetrics |
