# Research: Movement Polish

**Feature**: 008-movement-polish
**Date**: 2025-12-15
**Status**: Complete

## Research Questions

### 1. Capsule vs AABB Collision for Player

**Question**: Should we use capsule or AABB collision for the player?

**Decision**: Use AABB with capsule-like dimensions for simplicity

**Rationale**:
- Current implementation uses AABB (0.6m wide × 1.8m tall)
- True capsule-AABB intersection is complex and unnecessary for 1m³ voxels
- An "inflated AABB" (capsule bounding box) with proper sliding provides equivalent gameplay
- Capsule only matters for smooth corner sliding, achievable with axis-separated resolution

**Alternatives Considered**:
- True capsule collision: More complex math, minimal benefit for voxel worlds
- Cylinder collision: Simpler than capsule but still requires special intersection code

**Implementation**: Keep AABB player shape but update dimensions to 0.8m × 1.8m (radius 0.4m → half-width 0.4m)

---

### 2. Collision Resolution Order

**Question**: What order should we resolve collision axes?

**Decision**: Y → X → Z (vertical first)

**Rationale**:
- Resolving Y first ensures ground detection works correctly
- Player lands on floor before horizontal sliding
- Prevents "bouncing" off corners into walls
- Standard in Source Engine, Quake, and most FPS games

**Alternatives Considered**:
- X → Y → Z: Causes issues with landing detection
- Simultaneous resolution: Complex, can cause corner sticking

---

### 3. Step-Up Implementation

**Question**: How should step-up be implemented?

**Decision**: Probe-and-teleport with height check

**Rationale**:
- When horizontal collision detected while grounded:
  1. Check if obstacle height ≤ 0.5 blocks
  2. Check if space above current position allows upward movement
  3. Check if landing position after step is clear
  4. If all pass, teleport up and continue horizontal movement
- Simple, deterministic, no edge cases

**Alternatives Considered**:
- Gradual ramp-up: Visual smoothing but creates prediction complexity
- Physics-based impulse: Non-deterministic, can cause exploits

---

### 4. Jump Impulse Calculation

**Question**: How to calculate jump impulse from desired height?

**Decision**: Use kinematic formula: v = sqrt(2 * g * h)

**Rationale**:
- Target height: 1.25 blocks = 1.25m
- Gravity: 20 m/s²
- Jump impulse: sqrt(2 × 20 × 1.25) = sqrt(50) ≈ 7.07 m/s
- Produces exactly 1.25 blocks at apex in ideal conditions

**Verification**:
```
Time to apex: t = v/g = 7.07/20 = 0.354s
Height at apex: h = v²/(2g) = 50/40 = 1.25m ✓
```

---

### 5. Ground Detection Threshold

**Question**: How far below feet to check for ground?

**Decision**: 0.02 units (2cm) downward probe

**Rationale**:
- Too small (0.001): Floating point errors cause false negatives
- Too large (0.1): Player considered grounded while falling
- 0.02 is standard in Source Engine and works reliably

---

### 6. Air Control Implementation

**Question**: How should 30% air control be implemented?

**Decision**: Scale acceleration, not velocity

**Rationale**:
- Air control affects how fast player can change direction, not max speed
- Implementation: `acceleration *= is_grounded ? 1.0 : 0.3`
- Preserves momentum while limiting steering

**Alternatives Considered**:
- Velocity cap in air: Feels unnatural, momentum not preserved
- Separate air friction: More complex, similar result

---

### 7. Tunneling Prevention

**Question**: How to prevent high-speed tunneling through thin walls?

**Decision**: Subdivide movement into steps if velocity > 6 m/s

**Rationale**:
- Max speed: 6 m/s
- At 60 Hz: 0.1 units per tick maximum
- If velocity exceeds threshold, subdivide into smaller steps
- Each step performs full collision check
- Worst case: 2 subdivisions needed (falling at terminal velocity)

**Implementation**:
```rust
const MAX_MOVE_PER_STEP: f32 = 0.5; // Half a block
let steps = (velocity.length() * dt / MAX_MOVE_PER_STEP).ceil() as u32;
let sub_dt = dt / steps as f32;
for _ in 0..steps {
    position = resolve_collision(position, velocity * sub_dt);
}
```

---

### 8. Network Correction Smoothing

**Question**: How to smooth server corrections within 100ms?

**Decision**: Exponential interpolation with clamped delta

**Rationale**:
- Store correction target (server position)
- Each frame: blend toward target with factor based on elapsed time
- Clamp maximum correction per frame to prevent jarring teleports
- Complete within 100ms (6 ticks at 60Hz)

**Implementation**:
```rust
const CORRECTION_RATE: f32 = 10.0; // 1/0.1s = complete in 100ms
let blend = (CORRECTION_RATE * dt).min(1.0);
visual_position = visual_position.lerp(authoritative_position, blend);
```

---

### 9. Determinism Requirements

**Question**: What must be identical between client and server?

**Decision**: Movement code must be extracted to shared crate

**Rationale**:
- Currently movement logic is in `plix-server`
- Client prediction duplicates logic in `plix-client`
- Create shared movement functions in `plix-common`
- Both client and server call same code path
- Eliminates divergence bugs

**Implementation**:
- Move `MovementSystem::apply_input()` to `plix-common::physics`
- Server and client both use this function
- Only server writes authoritative state

---

### 10. Existing Code Analysis

**Question**: What exists that needs modification?

**Current State**:
| File | Status | Changes Needed |
|------|--------|----------------|
| `plix-server/src/sim/movement.rs` | Prototype | Update constants, extract to common |
| `plix-server/src/sim/collision.rs` | Basic AABB | Add step-up, fix resolution order |
| `plix-client/src/prediction.rs` | Exists | Use shared movement code |
| `plix-client/src/reconciliation.rs` | Exists | Add smooth correction |
| `plix-common/src/math.rs` | Exists | No changes needed (AABB already present) |

**Constants to Update**:
| Current | New | Location |
|---------|-----|----------|
| MOVE_SPEED = 5.0 | 6.0 | movement.rs |
| JUMP_VELOCITY = 8.0 | 7.07 | movement.rs |
| GRAVITY = 20.0 | 20.0 | No change |
| PLAYER_RADIUS = 0.3 | 0.4 | movement.rs, collision.rs |
| PLAYER_HEIGHT = 1.8 | 1.8 | No change |

---

## Summary

All research questions resolved. No NEEDS CLARIFICATION remaining.

**Key Decisions**:
1. Keep AABB collision with capsule-equivalent dimensions
2. Y → X → Z collision resolution order
3. Probe-and-teleport step-up
4. Jump impulse: 7.07 m/s for 1.25 block height
5. 0.02 unit ground detection threshold
6. Air control scales acceleration (×0.3)
7. Movement subdivision for tunneling prevention
8. Exponential correction smoothing (100ms)
9. Shared movement code in plix-common

**Ready for Phase 1**: Data model and contracts generation
