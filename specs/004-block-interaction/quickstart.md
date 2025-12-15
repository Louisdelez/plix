# Quickstart: Implementing Block Interaction

**Feature**: 004-block-interaction
**Date**: 2025-12-15

## Prerequisites

- Rust 1.75+ (stable)
- Existing plix workspace builds: `cargo build --workspace`
- Existing tests pass: `cargo test --workspace`

## Implementation Order

Follow this order to maintain a working build throughout:

1. Protocol types (plix-common)
2. Arena mutation (plix-arena)
3. Server validation + events (plix-server)
4. Client raycast (plix-client)
5. Client world + rendering (plix-client)
6. Tests

---

## Step 1: Protocol Types

**File**: `crates/plix-common/src/protocol/messages.rs`

### Add Block Edit Types

```rust
// Near other type definitions

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockEditKind {
    Place,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockEditRejectReason {
    OutOfBounds,
    OutOfRange,
    CellNotEmpty,
    CellEmpty,
    PlayerCollision,
    RateLimited,
    PlayerDead,
    InvalidPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEditRequest {
    pub kind: BlockEditKind,
    pub target_pos: BlockPos,
    pub block_type: BlockType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEditApplied {
    pub pos: BlockPos,
    pub new_block: BlockType,
    pub tick: Tick,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEditRejected {
    pub reason: BlockEditRejectReason,
    pub pos: BlockPos,
}
```

### Extend ClientMessage

```rust
pub enum ClientMessage {
    Connect(Connect),
    Disconnect,
    Input(PlayerInput),
    SnapshotAck(SnapshotAck),
    BlockEdit(BlockEditRequest),  // ADD THIS
}
```

### Extend GameEvent

```rust
pub enum GameEvent {
    PlayerJoined { ... },
    PlayerLeft { ... },
    // ... existing variants ...
    BlockEditApplied(BlockEditApplied),    // ADD THIS
    BlockEditRejected(BlockEditRejected),  // ADD THIS
}
```

**Verify**: `cargo build -p plix-common`

---

## Step 2: Arena Mutation

**File**: `crates/plix-arena/src/lib.rs` (or `loaded.rs`)

### Add set_block Method

```rust
impl LoadedArena {
    // Existing: get_block(&self, x, y, z) -> BlockType

    pub fn set_block(&mut self, pos: BlockPos, block_type: BlockType) {
        if self.is_in_bounds(pos) {
            let index = self.block_index(pos);
            self.blocks[index] = block_type;
        }
    }

    pub fn is_in_bounds(&self, pos: BlockPos) -> bool {
        pos.x >= 0 && pos.x < self.size.x as i32 &&
        pos.y >= 0 && pos.y < self.size.y as i32 &&
        pos.z >= 0 && pos.z < self.size.z as i32
    }

    fn block_index(&self, pos: BlockPos) -> usize {
        let (x, y, z) = (pos.x as usize, pos.y as usize, pos.z as usize);
        x + y * self.size.x as usize + z * self.size.x as usize * self.size.y as usize
    }
}
```

**Verify**: `cargo build -p plix-arena`

---

## Step 3: Server Block Edit System

### 3a. Player Cooldown Tracking

**File**: `crates/plix-server/src/session.rs`

```rust
pub struct ServerPlayer {
    // ... existing fields ...
    pub last_edit_tick: Option<Tick>,
}

impl ServerPlayer {
    pub fn new(...) -> Self {
        Self {
            // ... existing init ...
            last_edit_tick: None,
        }
    }
}
```

### 3b. Block Edit Validation

**File**: `crates/plix-server/src/sim/block_edit.rs` (NEW FILE)

```rust
use plix_common::{BlockPos, BlockType, Tick};
use plix_arena::LoadedArena;
use crate::session::ServerPlayer;

pub const MAX_EDIT_RANGE: f32 = 5.0;
pub const EDIT_COOLDOWN_TICKS: u32 = 15;

pub struct BlockEditSystem;

impl BlockEditSystem {
    pub fn validate_remove(
        pos: BlockPos,
        player: &ServerPlayer,
        arena: &LoadedArena,
        current_tick: Tick,
        is_playing: bool,
    ) -> Result<(), BlockEditRejectReason> {
        Self::validate_common(pos, player, arena, current_tick, is_playing)?;

        if arena.get_block(pos) == BlockType::Air {
            return Err(BlockEditRejectReason::CellEmpty);
        }

        Ok(())
    }

    pub fn validate_place(
        pos: BlockPos,
        player: &ServerPlayer,
        arena: &LoadedArena,
        all_players: &[&ServerPlayer],
        current_tick: Tick,
        is_playing: bool,
    ) -> Result<(), BlockEditRejectReason> {
        Self::validate_common(pos, player, arena, current_tick, is_playing)?;

        if arena.get_block(pos) != BlockType::Air {
            return Err(BlockEditRejectReason::CellNotEmpty);
        }

        if Self::would_collide_with_player(pos, all_players) {
            return Err(BlockEditRejectReason::PlayerCollision);
        }

        Ok(())
    }

    fn validate_common(
        pos: BlockPos,
        player: &ServerPlayer,
        arena: &LoadedArena,
        current_tick: Tick,
        is_playing: bool,
    ) -> Result<(), BlockEditRejectReason> {
        if !is_playing {
            return Err(BlockEditRejectReason::InvalidPhase);
        }

        if player.is_dead {
            return Err(BlockEditRejectReason::PlayerDead);
        }

        if !arena.is_in_bounds(pos) {
            return Err(BlockEditRejectReason::OutOfBounds);
        }

        let distance = Self::distance_to_block(player.position, pos);
        if distance > MAX_EDIT_RANGE {
            return Err(BlockEditRejectReason::OutOfRange);
        }

        if let Some(last_tick) = player.last_edit_tick {
            if current_tick.diff(last_tick) < EDIT_COOLDOWN_TICKS {
                return Err(BlockEditRejectReason::RateLimited);
            }
        }

        Ok(())
    }

    fn distance_to_block(player_pos: Vec3, block_pos: BlockPos) -> f32 {
        let block_center = Vec3::new(
            block_pos.x as f32 + 0.5,
            block_pos.y as f32 + 0.5,
            block_pos.z as f32 + 0.5,
        );
        player_pos.distance(block_center)
    }

    fn would_collide_with_player(pos: BlockPos, players: &[&ServerPlayer]) -> bool {
        // Simple AABB check: block occupies pos..(pos+1)
        // Player AABB is approximately pos ± 0.4 in x/z, 0..1.8 in y
        for player in players {
            if player.is_dead {
                continue;
            }
            let p = player.position;
            let bx = pos.x as f32;
            let by = pos.y as f32;
            let bz = pos.z as f32;

            // Check overlap
            if p.x + 0.4 > bx && p.x - 0.4 < bx + 1.0 &&
               p.y + 1.8 > by && p.y < by + 1.0 &&
               p.z + 0.4 > bz && p.z - 0.4 < bz + 1.0 {
                return true;
            }
        }
        false
    }
}
```

### 3c. Add mod declaration

**File**: `crates/plix-server/src/sim/mod.rs`

```rust
pub mod combat;
pub mod block_edit;  // ADD THIS
```

### 3d. Process in Tick Loop

**File**: `crates/plix-server/src/lib.rs`

In `simulate_tick()` or equivalent, after combat processing:

```rust
// Process block edits
for (player_id, request) in block_edit_requests {
    let player = self.sessions.get(&player_id).unwrap();
    let is_playing = matches!(self.match_state.phase(), MatchPhase::Playing);

    let result = match request.kind {
        BlockEditKind::Remove => {
            BlockEditSystem::validate_remove(
                request.target_pos,
                player,
                &self.arena,
                self.current_tick,
                is_playing,
            )
        }
        BlockEditKind::Place => {
            let all_players: Vec<_> = self.sessions.values().collect();
            BlockEditSystem::validate_place(
                request.target_pos,
                player,
                &self.arena,
                &all_players,
                self.current_tick,
                is_playing,
            )
        }
    };

    match result {
        Ok(()) => {
            // Apply edit
            let new_block = match request.kind {
                BlockEditKind::Remove => BlockType::Air,
                BlockEditKind::Place => request.block_type,
            };
            self.arena.set_block(request.target_pos, new_block);

            // Update player cooldown
            if let Some(player) = self.sessions.get_mut(&player_id) {
                player.last_edit_tick = Some(self.current_tick);
            }

            // Broadcast to all
            let event = GameEvent::BlockEditApplied(BlockEditApplied {
                pos: request.target_pos,
                new_block,
                tick: self.current_tick,
            });
            self.broadcast_event(event).await;
        }
        Err(reason) => {
            // Send rejection to requester only
            let event = GameEvent::BlockEditRejected(BlockEditRejected {
                reason,
                pos: request.target_pos,
            });
            self.send_event_to(player_id, event).await;
        }
    }
}
```

**Verify**: `cargo build -p plix-server`

---

## Step 4: Client Raycast

**File**: `crates/plix-client/src/raycast.rs` (NEW FILE)

```rust
use glam::{Vec3, IVec3};
use plix_common::BlockPos;
use plix_arena::LoadedArena;

#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    pub block_pos: BlockPos,
    pub face_normal: IVec3,
    pub distance: f32,
}

/// DDA raycast through voxel grid
pub fn raycast_blocks(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    arena: &LoadedArena,
) -> Option<RaycastHit> {
    let dir = direction.normalize();
    if dir.length_squared() < 0.001 {
        return None;
    }

    let step = IVec3::new(
        if dir.x > 0.0 { 1 } else { -1 },
        if dir.y > 0.0 { 1 } else { -1 },
        if dir.z > 0.0 { 1 } else { -1 },
    );

    let mut pos = IVec3::new(
        origin.x.floor() as i32,
        origin.y.floor() as i32,
        origin.z.floor() as i32,
    );

    let t_delta = Vec3::new(
        (1.0 / dir.x).abs(),
        (1.0 / dir.y).abs(),
        (1.0 / dir.z).abs(),
    );

    let mut t_max = Vec3::new(
        if dir.x > 0.0 { (pos.x as f32 + 1.0 - origin.x) / dir.x }
        else { (origin.x - pos.x as f32) / dir.x.abs() },
        if dir.y > 0.0 { (pos.y as f32 + 1.0 - origin.y) / dir.y }
        else { (origin.y - pos.y as f32) / dir.y.abs() },
        if dir.z > 0.0 { (pos.z as f32 + 1.0 - origin.z) / dir.z }
        else { (origin.z - pos.z as f32) / dir.z.abs() },
    );

    let mut distance = 0.0;
    let mut last_face = IVec3::ZERO;

    while distance < max_distance {
        let block_pos = BlockPos { x: pos.x, y: pos.y, z: pos.z };

        if arena.is_in_bounds(block_pos) && arena.get_block(block_pos).is_solid() {
            return Some(RaycastHit {
                block_pos,
                face_normal: last_face,
                distance,
            });
        }

        // Step to next voxel boundary
        if t_max.x < t_max.y && t_max.x < t_max.z {
            pos.x += step.x;
            distance = t_max.x;
            t_max.x += t_delta.x;
            last_face = IVec3::new(-step.x, 0, 0);
        } else if t_max.y < t_max.z {
            pos.y += step.y;
            distance = t_max.y;
            t_max.y += t_delta.y;
            last_face = IVec3::new(0, -step.y, 0);
        } else {
            pos.z += step.z;
            distance = t_max.z;
            t_max.z += t_delta.z;
            last_face = IVec3::new(0, 0, -step.z);
        }
    }

    None
}
```

**File**: `crates/plix-client/src/lib.rs`

```rust
pub mod raycast;  // ADD THIS
```

**Verify**: `cargo build -p plix-client`

---

## Step 5: Client Input + World Update

### 5a. Add Input Actions

**File**: `crates/plix-client/src/input.rs`

```rust
pub struct InputManager {
    // ... existing fields ...
    pub remove_block: bool,
    pub place_block: bool,
}

impl InputManager {
    pub fn handle_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        match button {
            MouseButton::Left => {
                if pressed { self.remove_block = true; }
                // Also handle existing attack logic
            }
            MouseButton::Right => {
                if pressed { self.place_block = true; }
            }
            _ => {}
        }
    }

    pub fn clear_block_actions(&mut self) {
        self.remove_block = false;
        self.place_block = false;
    }
}
```

### 5b. Client World State

**File**: `crates/plix-client/src/world.rs` (NEW FILE)

```rust
use plix_arena::LoadedArena;
use plix_common::{BlockPos, BlockType};

pub struct ClientWorld {
    pub arena: LoadedArena,
    dirty: bool,
}

impl ClientWorld {
    pub fn new(arena: LoadedArena) -> Self {
        Self { arena, dirty: true }
    }

    pub fn apply_edit(&mut self, pos: BlockPos, new_block: BlockType) {
        self.arena.set_block(pos, new_block);
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}
```

### 5c. Handle Events + Send Requests

In client main loop:

```rust
// On block input
if input.remove_block {
    if let Some(hit) = raycast_blocks(camera_pos, camera_forward, 5.0, &world.arena) {
        let request = BlockEditRequest {
            kind: BlockEditKind::Remove,
            target_pos: hit.block_pos,
            block_type: BlockType::Air,
        };
        send_message(ClientMessage::BlockEdit(request));
    }
    input.clear_block_actions();
}

if input.place_block {
    if let Some(hit) = raycast_blocks(camera_pos, camera_forward, 5.0, &world.arena) {
        let place_pos = BlockPos {
            x: hit.block_pos.x + hit.face_normal.x,
            y: hit.block_pos.y + hit.face_normal.y,
            z: hit.block_pos.z + hit.face_normal.z,
        };
        let request = BlockEditRequest {
            kind: BlockEditKind::Place,
            target_pos: place_pos,
            block_type: BlockType::Stone,
        };
        send_message(ClientMessage::BlockEdit(request));
    }
    input.clear_block_actions();
}

// On receiving events
match event {
    GameEvent::BlockEditApplied(edit) => {
        world.apply_edit(edit.pos, edit.new_block);
        show_debug("Block placed" or "Block removed");
    }
    GameEvent::BlockEditRejected(reject) => {
        show_debug(format!("Edit rejected: {:?}", reject.reason));
    }
    // ... other events
}

// In render loop
if world.is_dirty() {
    voxel_renderer.rebuild_mesh(&world.arena);
    world.clear_dirty();
}
```

---

## Step 6: Testing

### Unit Tests

**File**: `crates/plix-server/tests/block_edit_test.rs`

```rust
use plix_server::sim::block_edit::*;
// ... setup helpers ...

#[test]
fn test_remove_air_rejected() {
    let arena = create_empty_arena();
    let player = create_player_at(Vec3::ZERO);
    let result = BlockEditSystem::validate_remove(
        BlockPos { x: 0, y: 0, z: 0 },
        &player, &arena, Tick(0), true
    );
    assert_eq!(result, Err(BlockEditRejectReason::CellEmpty));
}

#[test]
fn test_place_into_solid_rejected() { ... }

#[test]
fn test_out_of_range_rejected() { ... }

#[test]
fn test_rate_limit_enforced() { ... }

#[test]
fn test_valid_remove_succeeds() { ... }

#[test]
fn test_valid_place_succeeds() { ... }
```

### Run All Tests

```bash
cargo test --workspace
```

### Manual Validation

1. Start server: `./scripts/run_server.sh`
2. Start client 1: `./scripts/run_client.sh`
3. Start client 2: `./scripts/run_client.sh`
4. Client 1: Remove a block (left click)
5. Verify: Both clients see block disappear
6. Client 2: Place a block (right click)
7. Verify: Both clients see block appear
8. Test rejection: Try to place out of range

### Load Test

```bash
./scripts/run_load_test.sh
```

Bots don't send block edits - test should pass unchanged.

---

## Troubleshooting

### "Unknown message type" errors

- Ensure protocol version is incremented
- Rebuild both client and server

### Mesh not updating

- Check `ClientWorld.dirty` flag is being set
- Check `VoxelRenderer.rebuild_mesh()` is called

### Edits not replicated

- Check `broadcast_event()` is called after apply
- Check event handling in client receive loop

### Rate limiting too aggressive

- Adjust `EDIT_COOLDOWN_TICKS` (15 = 4/sec at 60Hz)
- Consider 10 for faster building (6/sec)
