# Data Model: Movement Polish

**Feature**: 008-movement-polish
**Date**: 2025-12-15

## Entities

### MovementConfig

Physics configuration constants. Shared between client and server.

```rust
/// Movement physics configuration
pub struct MovementConfig {
    /// Maximum horizontal movement speed (m/s)
    pub max_speed: f32,           // Default: 6.0

    /// Gravity acceleration (m/s²)
    pub gravity: f32,             // Default: 20.0

    /// Jump impulse velocity (m/s)
    pub jump_impulse: f32,        // Default: 7.07 (for 1.25 block height)

    /// Ground friction coefficient
    pub ground_friction: f32,     // Default: 10.0

    /// Air control multiplier (0.0-1.0)
    pub air_control: f32,         // Default: 0.3

    /// Maximum step-up height (blocks)
    pub step_height: f32,         // Default: 0.5

    /// Player collision half-width (m)
    pub player_half_width: f32,   // Default: 0.4

    /// Player collision height (m)
    pub player_height: f32,       // Default: 1.8
}
```

**Validation Rules**:
- `max_speed` > 0.0
- `gravity` > 0.0
- `jump_impulse` > 0.0
- `air_control` ∈ [0.0, 1.0]
- `step_height` ∈ [0.0, 1.0]
- `player_half_width` > 0.0
- `player_height` > 0.0

---

### MovementState

Per-player movement state. Server-authoritative.

```rust
/// Player movement state
pub struct MovementState {
    /// Current position (feet position)
    pub position: Vec3,

    /// Current velocity (m/s)
    pub velocity: Vec3,

    /// Whether player is on ground
    pub is_grounded: bool,

    /// Whether jump is pending (buffer)
    pub jump_buffered: bool,

    /// Ticks since last ground contact
    pub air_time: u32,
}
```

**State Transitions**:
- `is_grounded`: true → false when `position.y` increases or no ground detected
- `is_grounded`: false → true when collision with ground during Y resolution
- `jump_buffered`: true when jump pressed, cleared after jump executes or 6 ticks

---

### CorrectionData

Server-to-client position correction.

```rust
/// Position correction from server
pub struct CorrectionData {
    /// Server tick this correction applies to
    pub tick: Tick,

    /// Authoritative position
    pub position: Vec3,

    /// Authoritative velocity
    pub velocity: Vec3,

    /// Whether player is grounded
    pub is_grounded: bool,

    /// Input sequence this acknowledges
    pub ack_seq: InputSeq,
}
```

**Relationships**:
- Links to `InputSeq` for reconciliation
- Maps to `Tick` for temporal ordering

---

### PlayerInput (Existing - Reference)

```rust
/// Player input for a single tick
pub struct PlayerInput {
    /// Input sequence number
    pub seq: InputSeq,

    /// Server tick this input targets
    pub tick: Tick,

    /// Forward/backward movement (-1.0 to 1.0)
    pub move_forward: f32,

    /// Left/right strafe (-1.0 to 1.0)
    pub move_right: f32,

    /// Jump requested
    pub jump: bool,

    /// Crouch held (out of scope but field exists)
    pub crouch: bool,

    /// Attack requested
    pub attack: bool,

    /// View yaw (radians)
    pub yaw: f32,

    /// View pitch (radians)
    pub pitch: f32,
}
```

---

### CollisionResult

Output of collision resolution.

```rust
/// Result of collision resolution
pub struct CollisionResult {
    /// Final resolved position
    pub position: Vec3,

    /// Final velocity (may be zeroed on collision)
    pub velocity: Vec3,

    /// Whether player is on ground after resolution
    pub grounded: bool,

    /// Whether step-up occurred
    pub stepped: bool,

    /// Collision normal (if any)
    pub hit_normal: Option<Vec3>,
}
```

---

## Relationships

```
┌─────────────────┐      ┌─────────────────┐
│  MovementConfig │◄─────│  MovementSystem │
└─────────────────┘      └────────┬────────┘
                                  │ uses
                                  ▼
┌─────────────────┐      ┌─────────────────┐
│   PlayerInput   │─────►│  MovementState  │
└─────────────────┘      └────────┬────────┘
        │                         │
        │                         │ produces
        ▼                         ▼
┌─────────────────┐      ┌─────────────────┐
│  CorrectionData │◄─────│CollisionResult  │
└─────────────────┘      └─────────────────┘
```

---

## State Machine: Grounded State

```
                    ┌──────────────┐
                    │              │
         ┌──────────│   GROUNDED   │◄─────────┐
         │          │              │          │
         │          └──────────────┘          │
         │                                    │
         │ jump || fall off edge              │ land (Y collision while falling)
         ▼                                    │
┌──────────────┐                              │
│              │                              │
│   AIRBORNE   │──────────────────────────────┘
│              │
└──────────────┘
```

**Transitions**:
- GROUNDED → AIRBORNE: `jump` input accepted OR position.y increases without support
- AIRBORNE → GROUNDED: Y-axis collision detected while velocity.y < 0

---

## Index/Query Patterns

### Block Lookup (Existing)

```rust
// Arena block query
fn get_block_at(arena: &LoadedArena, pos: BlockPos) -> BlockType
```

### Collision Sweep

```rust
// Check AABB intersection with voxels
fn check_collision(arena: &LoadedArena, player_aabb: AABB) -> bool

// Resolve collision with sliding
fn resolve_collision(
    arena: &LoadedArena,
    position: Vec3,
    velocity: Vec3,
    config: &MovementConfig,
    dt: f32,
) -> CollisionResult
```

### Ground Check

```rust
// Check if position has ground support
fn is_grounded(arena: &LoadedArena, position: Vec3, half_width: f32) -> bool
```

---

## Migration Notes

### From Current Implementation

1. **Constants Migration**:
   - `MOVE_SPEED: 5.0` → `MovementConfig::max_speed: 6.0`
   - `JUMP_VELOCITY: 8.0` → `MovementConfig::jump_impulse: 7.07`
   - `PLAYER_RADIUS: 0.3` → `MovementConfig::player_half_width: 0.4`

2. **Collision Migration**:
   - `PLAYER_HALF_WIDTH: 0.3` → `0.4`
   - Add step-up logic (not present in current)
   - Fix resolution order (currently X → Y → Z, should be Y → X → Z)

3. **New Fields**:
   - `MovementState::jump_buffered` (new)
   - `MovementState::air_time` (new)
   - `CollisionResult::stepped` (new)

4. **Shared Code**:
   - Move physics functions from `plix-server` to `plix-common`
   - Client imports from `plix-common` instead of duplicating
