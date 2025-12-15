# Data Model: Server-Authoritative Block Interaction

**Feature**: 004-block-interaction
**Date**: 2025-12-15

## Overview

This document defines the data entities, their relationships, and validation rules for the block interaction feature.

## Entities

### BlockPos (existing, extended usage)

Integer coordinates identifying a cell in the voxel grid.

**Location**: `crates/plix-common/src/types.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
```

**Validation Rules**:
- All coordinates must be within arena bounds: `0 <= x < arena.size.x`, etc.
- Used as key for block lookups and edit targets

**Relationships**:
- Maps to index in `LoadedArena.blocks` via `arena.block_index(pos)`
- Adjacent positions calculated via `pos + face_normal`

---

### BlockType (existing)

Type identifier for blocks in the voxel world.

**Location**: `crates/plix-common/src/types.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum BlockType {
    Air = 0,
    Stone = 1,
    Brick = 2,
    Metal = 3,
}
```

**Validation Rules**:
- `Air` (0) represents empty space
- Non-air types are solid and collidable
- MVP uses single default type `Stone` for placement

**Relationships**:
- Stored in `LoadedArena.blocks[index]`
- Sent in `BlockEditApplied` events

---

### BlockEditKind (new)

Discriminator for the type of block edit operation.

**Location**: `crates/plix-common/src/protocol/messages.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockEditKind {
    Place,
    Remove,
}
```

**Validation Rules**:
- `Place`: Target cell must be `Air`, new block must be non-`Air`
- `Remove`: Target cell must be non-`Air`, result is `Air`

---

### BlockEditRequest (new)

Client request to modify a block in the world.

**Location**: `crates/plix-common/src/protocol/messages.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEditRequest {
    pub kind: BlockEditKind,
    pub target_pos: BlockPos,
    pub block_type: BlockType,  // For Place; ignored for Remove
}
```

**Validation Rules**:
- `target_pos` must be within arena bounds
- `target_pos` must be within `MAX_EDIT_RANGE` (5.0 blocks) of player position
- For `Remove`: `arena.get_block(target_pos) != Air`
- For `Place`: `arena.get_block(target_pos) == Air`
- For `Place`: No player AABB intersects the target block
- Player must not be dead
- Match phase must be `Playing`
- Player must not be rate-limited (cooldown expired)

**Relationships**:
- Sent as `ClientMessage::BlockEdit(BlockEditRequest)`
- Produces `BlockEditApplied` or `BlockEditRejected` response

---

### BlockEditRejectReason (new)

Reason code for rejected block edit requests.

**Location**: `crates/plix-common/src/protocol/messages.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockEditRejectReason {
    OutOfBounds,
    OutOfRange,
    CellNotEmpty,      // Place into non-air
    CellEmpty,         // Remove air
    PlayerCollision,   // Would trap player
    RateLimited,
    PlayerDead,
    InvalidPhase,      // Not in Playing phase
}
```

**Relationships**:
- Returned in `BlockEditRejected` event
- Maps to FR-006 through FR-013 validation rules

---

### BlockEditApplied (new)

Server event confirming a successful block edit.

**Location**: `crates/plix-common/src/protocol/messages.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEditApplied {
    pub pos: BlockPos,
    pub new_block: BlockType,  // Air for Remove, block type for Place
    pub tick: Tick,
}
```

**Validation Rules**:
- Broadcast to all connected clients (including requester)
- `tick` is the server tick when edit was applied

**Relationships**:
- Part of `GameEvent` enum
- Triggers client world update and mesh rebuild

---

### BlockEditRejected (new)

Server event indicating a rejected block edit request.

**Location**: `crates/plix-common/src/protocol/messages.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEditRejected {
    pub reason: BlockEditRejectReason,
    pub pos: BlockPos,  // Echo back for client correlation
}
```

**Validation Rules**:
- Sent only to the requesting client (unicast)
- Contains reason for rejection

**Relationships**:
- Part of `GameEvent` enum
- Triggers client debug HUD feedback

---

### RaycastHit (new, client-side)

Result of a client-side raycast for targeting.

**Location**: `crates/plix-client/src/raycast.rs`

```rust
#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    pub block_pos: BlockPos,
    pub face_normal: IVec3,  // Which face was hit (-1/0/1 per axis)
    pub distance: f32,
}
```

**Usage**:
- `block_pos`: Target for Remove action
- `block_pos + face_normal`: Target for Place action
- `distance`: Used for range validation (client-side UX only)

**Relationships**:
- Computed by `raycast_blocks()` function
- Used to populate `BlockEditRequest.target_pos`

---

### ServerPlayer (extended)

Per-player server-side state.

**Location**: `crates/plix-server/src/session.rs`

**New Field**:
```rust
pub struct ServerPlayer {
    // ... existing fields ...

    /// Last tick when this player successfully edited a block
    pub last_edit_tick: Option<Tick>,
}
```

**Validation Rules**:
- Updated on successful block edit
- Checked for rate limiting: `current_tick.diff(last_edit_tick) >= EDIT_COOLDOWN_TICKS`

---

### ClientWorld (new)

Client-side mutable world representation.

**Location**: `crates/plix-client/src/world.rs`

```rust
pub struct ClientWorld {
    arena: LoadedArena,  // Mutable copy
    dirty: bool,         // Mesh needs rebuild
}
```

**Methods**:
```rust
impl ClientWorld {
    pub fn from_arena_data(data: &[u8]) -> Result<Self, Error>;
    pub fn get_block(&self, pos: BlockPos) -> BlockType;
    pub fn apply_edit(&mut self, pos: BlockPos, new_block: BlockType);
    pub fn is_dirty(&self) -> bool;
    pub fn clear_dirty(&mut self);
}
```

**Relationships**:
- Initialized from `Connected.arena_data`
- Updated by `BlockEditApplied` events
- Drives `VoxelRenderer` mesh rebuilds

---

## Entity Relationships Diagram

```
                    Client                                  Server
                    ------                                  ------

InputManager                                            SessionManager
    │                                                        │
    ├─ remove_block: bool                                    ├─ players: HashMap<PlayerId, ServerPlayer>
    ├─ place_block: bool                                     │       │
    │                                                        │       └─ last_edit_tick: Option<Tick>
    v                                                        │
raycast_blocks()                                             │
    │                                                        │
    └─► RaycastHit                                           │
            │                                                │
            v                                                │
    BlockEditRequest ─────────────────────────────────────►  │
            │                                                │
            │                                    BlockEditSystem.validate()
            │                                                │
            │                                                v
            │                                         LoadedArena
            │                                           (mutated)
            │                                                │
            │                                                v
            │◄────────────────────────────────────  BlockEditApplied
            │                                       (broadcast)
            │
            v
    ClientWorld.apply_edit()
            │
            v
    VoxelRenderer (mesh rebuild)
```

## State Transitions

### Block State

```
    [Air]  ◄───── Remove ─────  [Solid]
      │                            ▲
      │                            │
      └────── Place (Stone) ───────┘
```

### Player Edit Cooldown

```
    [Ready]  ──── edit request ────►  [Cooldown]
       ▲                                  │
       │                                  │
       └───── 15 ticks elapsed ───────────┘
```

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `MAX_EDIT_RANGE` | 5.0 | Maximum distance (blocks) from player to target |
| `EDIT_COOLDOWN_TICKS` | 15 | Ticks between allowed edits (4/sec at 60Hz) |
| `DEFAULT_BLOCK_TYPE` | `BlockType::Stone` | Default block type for placement |

## Serialization Notes

All new types use `bincode` serialization (workspace standard):
- Compact binary format
- Serde derive macros
- Compatible with existing protocol codec
- Max payload size: 1389 bytes (existing constraint)

Block edit messages are small (~20 bytes) and fit comfortably within limits.
