# Movement System Contracts

**Feature**: 008-movement-polish
**Date**: 2025-12-15

## Overview

This document defines the internal API contracts for the movement system. Since this is a game engine (not a web API), contracts are defined as Rust function signatures and message formats.

---

## Core Movement API

### apply_movement

Applies player input to produce new movement state. Must be deterministic.

```rust
/// Apply player input to movement state
///
/// # Arguments
/// * `input` - Player input for this tick
/// * `state` - Current movement state
/// * `arena` - Arena for collision detection
/// * `config` - Physics configuration
/// * `dt` - Delta time (1/60 for 60Hz)
///
/// # Returns
/// Updated movement state after physics simulation
///
/// # Determinism
/// Given identical inputs, this function MUST produce identical outputs
/// on both client and server.
pub fn apply_movement(
    input: &PlayerInput,
    state: &MovementState,
    arena: &LoadedArena,
    config: &MovementConfig,
    dt: f32,
) -> MovementState;
```

**Preconditions**:
- `input.move_forward` ∈ [-1.0, 1.0]
- `input.move_right` ∈ [-1.0, 1.0]
- `dt` > 0.0

**Postconditions**:
- `result.position` does not intersect solid blocks
- `result.velocity.length()` ≤ `config.max_speed` (horizontal)
- `result.is_grounded` is accurate for the new position

---

### resolve_collision

Resolves position against voxel geometry with sliding.

```rust
/// Resolve collision for a movement delta
///
/// # Arguments
/// * `position` - Starting position (feet)
/// * `velocity` - Movement velocity
/// * `arena` - Arena for collision queries
/// * `config` - Physics configuration
/// * `dt` - Delta time
///
/// # Returns
/// Collision result with final position and state
pub fn resolve_collision(
    position: Vec3,
    velocity: Vec3,
    arena: &LoadedArena,
    config: &MovementConfig,
    dt: f32,
) -> CollisionResult;
```

**Resolution Order**: Y → X → Z

**Step-Up Logic**:
1. If horizontal collision detected AND grounded
2. AND obstacle height ≤ `config.step_height`
3. AND no head collision at elevated position
4. THEN teleport up and retry horizontal movement

---

### check_grounded

Determines if player has ground support.

```rust
/// Check if position is grounded
///
/// # Arguments
/// * `position` - Feet position
/// * `arena` - Arena for collision queries
/// * `half_width` - Player collision half-width
///
/// # Returns
/// true if solid block exists within 0.02 units below feet
pub fn check_grounded(
    position: Vec3,
    arena: &LoadedArena,
    half_width: f32,
) -> bool;
```

**Probe Distance**: 0.02 units (2cm)

---

## Network Messages

### Snapshot (Server → Client)

Existing message extended with movement state.

```rust
/// Game state snapshot
pub struct Snapshot {
    pub tick: Tick,
    pub players: Vec<PlayerSnapshot>,
    // ... other fields
}

pub struct PlayerSnapshot {
    pub id: PlayerId,
    pub position: Vec3,
    pub velocity: Vec3,      // NEW: include velocity
    pub rotation: Rotation,
    pub is_grounded: bool,   // NEW: include grounded state
    pub health: u8,
    // ... other fields
}
```

**Rate**: Every tick (60Hz)

---

### Correction (Server → Client)

Position correction when prediction diverges.

```rust
/// Position correction message
pub struct PositionCorrection {
    /// Server tick this correction applies to
    pub tick: Tick,

    /// Authoritative position
    pub position: Vec3,

    /// Authoritative velocity
    pub velocity: Vec3,

    /// Grounded state
    pub is_grounded: bool,

    /// Last acknowledged input sequence
    pub ack_seq: InputSeq,
}
```

**Trigger**: Sent when server detects client position differs by > 0.1 blocks

**Client Handling**:
1. Receive correction
2. Discard inputs with seq < ack_seq
3. Set authoritative state
4. Re-simulate buffered inputs
5. Smooth visual position toward result (100ms)

---

### PlayerInput (Client → Server)

Existing message, no changes needed.

```rust
pub struct PlayerInput {
    pub seq: InputSeq,
    pub tick: Tick,
    pub move_forward: f32,
    pub move_right: f32,
    pub jump: bool,
    pub crouch: bool,
    pub attack: bool,
    pub yaw: f32,
    pub pitch: f32,
}
```

**Rate**: Every tick (60Hz)
**Reliability**: Unreliable (UDP), latest-wins on server

---

## Validation Contracts

### Input Validation

```rust
/// Validate player input is within acceptable bounds
///
/// # Returns
/// None if valid, Some(violation) if invalid
pub fn validate_input(input: &PlayerInput) -> Option<InputViolation>;

pub enum InputViolation {
    MovementOutOfRange,      // |move_forward| > 1.0 or |move_right| > 1.0
    InvalidRotation,         // NaN or infinite
    // Rate limiting handled separately by anti-cheat
}
```

---

### Movement Validation (Anti-Cheat Integration)

```rust
/// Validate movement delta is physically possible
///
/// Used by anti-cheat system (feature 007)
///
/// # Arguments
/// * `old_pos` - Previous tick position
/// * `new_pos` - Current tick position
/// * `config` - Physics configuration
/// * `dt` - Delta time
///
/// # Returns
/// true if movement is within possible bounds
pub fn validate_movement_delta(
    old_pos: Vec3,
    new_pos: Vec3,
    config: &MovementConfig,
    dt: f32,
) -> bool;
```

**Maximum Delta**: `config.max_speed * dt * 1.5` (50% tolerance for rounding)

---

## Error Handling

### Collision Errors

| Error | Handling |
|-------|----------|
| Position inside solid block | Resolve to nearest valid position |
| Arena bounds exceeded | Clamp to arena bounds |
| Invalid block query | Treat as air (fail-safe) |

### Network Errors

| Error | Handling |
|-------|----------|
| Correction for past tick | Apply to current state, re-simulate |
| Input sequence gap | Interpolate missing input from neighbors |
| Stale input (> 500ms) | Drop input, wait for fresh |

---

## Performance Contracts

| Operation | Target | Measurement |
|-----------|--------|-------------|
| apply_movement | < 10µs | Per player per tick |
| resolve_collision | < 50µs | Per player per tick |
| check_grounded | < 5µs | Per call |
| Full tick (8 players) | < 1ms | Total movement processing |

---

## Determinism Contract

The following must be bit-identical between client and server:

1. `apply_movement` output for identical inputs
2. `resolve_collision` output for identical inputs
3. `check_grounded` output for identical inputs

**Requirements**:
- No floating-point non-determinism (no fast-math, no SIMD divergence)
- Same code path on client and server (shared crate)
- Same configuration values
- Same arena data
