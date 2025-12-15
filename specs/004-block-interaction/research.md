# Research: Server-Authoritative Block Interaction

**Feature**: 004-block-interaction
**Date**: 2025-12-15

## Overview

This document captures research findings for implementing server-authoritative block interactions. All technical context items were resolved through codebase exploration - no external clarifications were needed.

## Research Findings

### 1. Existing Protocol Pattern (Combat Events)

**Source**: `crates/plix-server/src/sim/combat.rs`, `crates/plix-common/src/protocol/messages.rs`

**Decision**: Follow the combat event pattern for block edits.

**Rationale**:
- Combat uses `GameEvent` enum for reliable delivery (HitConfirmed, DamageTaken, PlayerDied)
- Events are broadcast via `broadcast_event()` in server tick loop
- Per-player events (HitConfirmed) sent only to requester
- This pattern already proven stable and server-authoritative

**Alternatives Considered**:
- Embedding block edits in `PlayerInput`: Rejected - input is for continuous state, edits are discrete events
- Separate reliable channel: Rejected - unnecessary complexity, existing event system sufficient

### 2. World State Representation

**Source**: `crates/plix-arena/src/format.rs`, `crates/plix-arena/src/lib.rs`

**Decision**: Extend `LoadedArena` with mutation capability.

**Rationale**:
- Arena already stores blocks as `Vec<BlockType>` with `get_block(x, y, z)`
- Adding `set_block(x, y, z, block_type)` is straightforward
- Arena data is already serialized and sent on connect (`Connected::arena_data`)
- No separate edit log needed - current state is truth

**Alternatives Considered**:
- Copy-on-write arena: Rejected - unnecessary for small arenas
- Delta log with baseline: Rejected - adds complexity, full state sufficient for MVP

### 3. Client Raycast Algorithm

**Source**: Standard voxel raycasting literature

**Decision**: Use DDA (Digital Differential Analyzer) algorithm.

**Rationale**:
- Industry standard for grid-based raycasting (Minecraft, etc.)
- O(n) where n = distance in blocks
- Provides exact hit position and face normal
- No floating-point precision issues

**Implementation Notes**:
```rust
// DDA raycast pseudocode
fn raycast(origin: Vec3, dir: Vec3, max_dist: f32, arena: &Arena) -> Option<Hit> {
    let step = dir.signum();
    let t_delta = (1.0 / dir).abs();
    let mut t_max = initial_t_max(origin, dir, step);
    let mut pos = origin.floor().as_ivec3();

    while distance < max_dist {
        if arena.is_solid(pos) {
            return Some(Hit { pos, face: last_axis });
        }
        // Step along smallest t_max axis
        if t_max.x < t_max.y && t_max.x < t_max.z {
            pos.x += step.x;
            t_max.x += t_delta.x;
        } else if t_max.y < t_max.z {
            pos.y += step.y;
            t_max.y += t_delta.y;
        } else {
            pos.z += step.z;
            t_max.z += t_delta.z;
        }
    }
    None
}
```

**Alternatives Considered**:
- Bresenham line algorithm: Works but doesn't give face normals naturally
- Ray-AABB intersection per block: Too slow for long rays

### 4. Late Join Synchronization

**Source**: `crates/plix-server/src/lib.rs`, `Connected` message handling

**Decision**: Use existing arena data path - send full current world state on connect.

**Rationale**:
- `Connected` message already includes `arena_data: Vec<u8>`
- Server can serialize current (edited) arena state
- Client initializes world from this data
- No replay mechanism needed
- Simple and correct

**Alternatives Considered**:
- Baseline + edit log replay: Rejected - more complex, no benefit for small arenas
- Snapshot delta from last known state: Rejected - client is new, has no prior state

### 5. Mesh Update Strategy

**Source**: `crates/plix-client/src/render/voxels.rs`

**Decision**: Full mesh rebuild for MVP; chunk-based optimization deferred.

**Rationale**:
- Current renderer is placeholder (TODO comments throughout)
- Arena is small (bounded competitive arena, not infinite world)
- Full rebuild is simpler to implement correctly
- Performance concern mitigated by arena size
- Can optimize to per-chunk rebuild later if needed

**Alternatives Considered**:
- Per-block mesh delta: Complex, premature optimization
- Deferred/batched rebuild: May introduce visual lag

### 6. Rate Limiting Approach

**Source**: Spec requirement FR-010, existing combat cooldown pattern

**Decision**: Server-side per-player cooldown of 15 ticks (4 edits/sec at 60Hz).

**Rationale**:
- Prevents edit spam/DoS
- 4 edits/sec allows reasonable building pace
- Follows combat system's cooldown pattern (`last_attack_tick`)
- Client can optionally mirror for UX feedback

**Implementation**:
```rust
// In ServerPlayer
last_edit_tick: Option<Tick>,

// In validation
fn is_rate_limited(&self, player: &ServerPlayer, current_tick: Tick) -> bool {
    match player.last_edit_tick {
        Some(last) => current_tick.diff(last) < EDIT_COOLDOWN_TICKS,
        None => false,
    }
}
```

### 7. Player Collision Check for Placement

**Source**: Spec requirement FR-011

**Decision**: Check all player AABBs before allowing block placement.

**Rationale**:
- Prevents griefing (trapping players inside blocks)
- Simple AABB-vs-block check
- Only check alive players (dead players are spectating)

**Implementation**:
```rust
fn would_collide_with_player(pos: BlockPos, players: &[&ServerPlayer]) -> bool {
    let block_aabb = Aabb::from_block(pos);
    players.iter()
        .filter(|p| !p.is_dead)
        .any(|p| p.aabb().intersects(&block_aabb))
}
```

### 8. Edit Request Message Design

**Decision**: Separate `BlockEditRequest` message (not part of `PlayerInput`).

**Rationale**:
- Block edits are discrete events, not continuous input
- `PlayerInput` is sent every frame; edits are occasional
- Separate message allows reliable channel if needed
- Cleaner API separation

**Message Structure**:
```rust
pub struct BlockEditRequest {
    pub kind: BlockEditKind,
    pub target_pos: BlockPos,
    pub block_type: Option<BlockType>, // For Place, None for Remove
}

pub enum BlockEditKind {
    Place,
    Remove,
}
```

## Resolved Items Summary

| Item | Resolution |
|------|------------|
| Protocol extension pattern | Follow existing GameEvent pattern |
| World state storage | Extend LoadedArena with set_block() |
| Late join mechanism | Send full current arena state |
| Raycast algorithm | DDA (Digital Differential Analyzer) |
| Mesh update strategy | Full rebuild for MVP |
| Rate limiting | 15 tick cooldown per player |
| Player collision | AABB intersection check |
| Message design | Separate BlockEditRequest |

## Dependencies

All dependencies already present in workspace:
- `glam`: Vec3, IVec3 for positions
- `bincode`: Serialization of messages
- `tokio`: Async networking
- `wgpu`: Rendering (client)

No new dependencies required.

## Next Steps

With all research resolved, proceed to Phase 1:
1. Generate `data-model.md` with entity definitions
2. Generate `contracts/block-protocol.md` with message schemas
3. Generate `quickstart.md` with implementation guide
