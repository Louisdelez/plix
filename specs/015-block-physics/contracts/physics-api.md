# Physics API Contract

**Feature**: 015-block-physics
**Date**: 2025-12-16
**Type**: Internal Rust API (not network protocol)

## Overview

This document defines the internal API contracts for the block physics system. These are Rust trait and struct interfaces, not REST/GraphQL endpoints.

---

## PhysicsConfig API

### Construction

```rust
impl PhysicsConfig {
    /// Create default physics config (gravity on, liquids off)
    pub fn default() -> Self;

    /// Create with all physics disabled
    pub fn disabled() -> Self;

    /// Create with custom settings
    pub fn new(
        gravity_enabled: bool,
        liquids_enabled: bool,
        max_events_per_tick: u32,
        max_liquid_spread_distance: u8,
    ) -> Self;
}
```

### Accessors

```rust
impl PhysicsConfig {
    pub fn gravity_enabled(&self) -> bool;
    pub fn liquids_enabled(&self) -> bool;
    pub fn max_events_per_tick(&self) -> u32;
    pub fn max_liquid_spread_distance(&self) -> u8;
}
```

---

## PhysicsQueue API

### Construction

```rust
impl PhysicsQueue {
    /// Create empty queue
    pub fn new() -> Self;
}
```

### Operations

```rust
impl PhysicsQueue {
    /// Add event to queue if not duplicate
    /// Returns true if added, false if duplicate
    pub fn push(&mut self, event: PhysicsEvent) -> bool;

    /// Remove and return front event, or None if empty
    pub fn pop(&mut self) -> Option<PhysicsEvent>;

    /// Current number of pending events
    pub fn len(&self) -> usize;

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool;

    /// Remove all events
    pub fn clear(&mut self);
}
```

---

## PhysicsSystem API

### Construction

```rust
impl PhysicsSystem {
    /// Create physics system with config
    pub fn new(config: PhysicsConfig) -> Self;
}
```

### Core Operations

```rust
impl PhysicsSystem {
    /// Process physics for one tick
    /// Drains queue up to budget, resolves events, may add new events
    /// Returns number of events processed
    pub fn tick(&mut self, world: &mut ChunkedWorld) -> u32;

    /// Queue a physics event for processing
    /// Called when blocks are modified
    pub fn queue_event(&mut self, event: PhysicsEvent);

    /// Detect and queue physics events for a position
    /// Called after block edits
    pub fn detect_events_at(&mut self, pos: BlockPos, world: &ChunkedWorld);
}
```

### Accessors

```rust
impl PhysicsSystem {
    /// Get current configuration
    pub fn config(&self) -> &PhysicsConfig;

    /// Get mutable configuration (for runtime toggle)
    pub fn config_mut(&mut self) -> &mut PhysicsConfig;

    /// Get current metrics
    pub fn metrics(&self) -> &PhysicsMetrics;

    /// Get current queue depth
    pub fn queue_depth(&self) -> usize;
}
```

---

## PhysicsMetrics API

### Accessors

```rust
impl PhysicsMetrics {
    /// Events processed in last tick
    pub fn events_processed_last_tick(&self) -> u32;

    /// Current queue depth
    pub fn queue_depth(&self) -> u32;

    /// Total blocks that have fallen (lifetime)
    pub fn total_blocks_fallen(&self) -> u64;

    /// Total liquid spread updates (lifetime)
    pub fn total_liquid_updates(&self) -> u64;
}
```

---

## BlockType Extensions API

```rust
impl BlockType {
    /// Check if this block type is affected by gravity
    pub fn is_gravity_affected(&self) -> bool;

    /// Check if this block type is a liquid
    pub fn is_liquid(&self) -> bool;
}
```

---

## Event Detection Contract

When a block is modified, the physics system must be notified:

```rust
// After any block edit in server
let affected = world.set_block(pos, new_block);

// Detect physics events
physics_system.detect_events_at(pos, world);

// Also check block above (for gravity cascades)
let above = pos.with_y(pos.y + 1);
physics_system.detect_events_at(above, world);
```

---

## Server Integration Contract

Physics must be called in this order within the server tick:

```rust
impl Server {
    async fn tick(&mut self) {
        // 1. Process player inputs
        self.process_inputs().await;

        // 2. Process block edits (triggers physics detection)
        self.process_block_edits().await;

        // 3. Run physics simulation
        let processed = self.physics.tick(&mut self.world);

        // 4. Update metrics
        self.metrics.physics_events = processed;

        // 5. Send snapshots (includes physics changes)
        self.send_snapshots().await;
    }
}
```

---

## Error Handling

Physics operations do not return errors - they are infallible:

- Invalid positions: Silently ignored (out-of-bounds checks in ChunkedWorld)
- Budget exceeded: Remaining events stay queued
- Disabled physics: `tick()` returns 0, no events processed

---

## Thread Safety

- `PhysicsSystem` is `!Sync` - only used from main server thread
- `PhysicsMetrics` uses atomic operations for potential future threading
- `PhysicsConfig` is `Clone + Send` for passing to new servers

---

## Determinism Guarantee

Given:
- Same initial `ChunkedWorld` state
- Same sequence of block edits
- Same `PhysicsConfig`

The physics system MUST produce:
- Identical final world state
- Identical sequence of block changes
- Identical metrics values

This is achieved via:
- FIFO event ordering
- Deterministic event detection order
- No use of HashMap iteration (uses IndexMap or Vec)
- No floating-point operations
