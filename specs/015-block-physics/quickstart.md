# Quickstart: Block Physics Light

**Feature**: 015-block-physics
**Date**: 2025-12-16

## Overview

This guide provides quick instructions for implementing and using the block physics system.

---

## Implementation Order

### Phase 1: Core Types (plix-common)

1. **Create physics module** at `crates/plix-common/src/physics/mod.rs`
2. **Add PhysicsConfig** - configuration struct
3. **Add PhysicsEventKind** - event type enum
4. **Add PhysicsEvent** - event struct
5. **Add PhysicsQueue** - bounded FIFO queue with dedup
6. **Add PhysicsMetrics** - counter struct
7. **Extend BlockType** - add `is_gravity_affected()` method

### Phase 2: Physics System (plix-server)

1. **Create physics module** at `crates/plix-server/src/physics/mod.rs`
2. **Add PhysicsSystem** - main simulation struct
3. **Implement gravity resolution** - step-based falling
4. **Implement event detection** - check neighbors on block edit
5. **Add tick() method** - process queue with budget

### Phase 3: Server Integration

1. **Add PhysicsConfig to ServerConfig**
2. **Initialize PhysicsSystem in Server::new()**
3. **Hook block edits** - call detect_events_at() after set_block()
4. **Call tick()** in game loop (after movement, before snapshots)
5. **Expose metrics** in server metrics system

### Phase 4: Testing

1. **Unit tests** for queue, config, block type methods
2. **Gravity tests** - fall, cascade, cross-chunk
3. **Budget tests** - verify event limiting
4. **Integration tests** - full server with physics

### Phase 5 (Optional): Liquids

1. **Add WATER block type**
2. **Implement liquid spreading**
3. **Add liquid tests**

---

## Quick Test

After implementation, verify with:

```bash
# Run physics tests
cargo test -p plix-common physics
cargo test -p plix-server physics

# Run server with physics enabled
cargo run -p plix-server -- --physics

# In another terminal, run client
cargo run -p plix-client -- --connect 127.0.0.1:7777
```

Test physics by:
1. Place sand block mid-air → should fall
2. Place stack of sand, remove bottom → cascade should occur
3. Physics should complete without server lag

---

## Code Snippets

### Adding Physics to Server

```rust
// In ServerConfig
pub struct ServerConfig {
    // ... existing fields
    pub physics: PhysicsConfig,
}

// In Server
pub struct Server {
    // ... existing fields
    physics: PhysicsSystem,
}

// In Server::new()
let physics = PhysicsSystem::new(config.physics.clone());

// In Server::tick()
self.physics.tick(&mut self.world);
```

### Detecting Events After Block Edit

```rust
// In process_single_edit()
let affected = self.world.set_block(pos, new_block);

// Detect physics events
if self.physics.config().gravity_enabled() {
    self.physics.detect_events_at(pos, &self.world);
    // Check block above for gravity cascade
    let above = BlockPos::new(pos.x, pos.y + 1, pos.z);
    self.physics.detect_events_at(above, &self.world);
}
```

### Basic Physics Test

```rust
#[test]
fn test_sand_falls() {
    let mut world = ChunkedWorld::new();
    let mut physics = PhysicsSystem::new(PhysicsConfig::default());

    // Place support
    world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
    // Place sand above
    world.set_block(BlockPos::new(0, 2, 0), BlockType::SAND);
    physics.detect_events_at(BlockPos::new(0, 2, 0), &world);

    // Tick until sand lands
    for _ in 0..10 {
        physics.tick(&mut world);
    }

    // Sand should have fallen to y=1 (on top of stone)
    assert_eq!(world.get_block(BlockPos::new(0, 1, 0)), BlockType::SAND);
    assert_eq!(world.get_block(BlockPos::new(0, 2, 0)), BlockType::AIR);
}
```

---

## Configuration

### Enable Physics (Server CLI)

```bash
# Default (gravity on, liquids off)
plix-server --physics

# Custom budget
plix-server --physics --physics-budget 200

# Disable physics
plix-server --no-physics
```

### Config File

```toml
# server.toml
[physics]
gravity_enabled = true
liquids_enabled = false
max_events_per_tick = 100
max_liquid_spread_distance = 7
```

---

## Debugging

### Check Metrics

```rust
let metrics = server.physics.metrics();
println!("Queue depth: {}", metrics.queue_depth());
println!("Last tick processed: {}", metrics.events_processed_last_tick());
println!("Total fallen: {}", metrics.total_blocks_fallen());
```

### Verbose Logging (Debug Only)

```rust
// In PhysicsSystem::tick()
#[cfg(debug_assertions)]
tracing::trace!(
    pos = ?event.pos,
    kind = ?event.kind,
    "Processing physics event"
);
```

---

## Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Blocks don't fall | Physics disabled | Check `config.gravity_enabled()` |
| Cascade too slow | Low budget | Increase `max_events_per_tick` |
| Events lost | Bug in queue | Verify `pending` HashSet sync |
| Non-deterministic | HashMap iteration | Use IndexMap or Vec for ordered iteration |
