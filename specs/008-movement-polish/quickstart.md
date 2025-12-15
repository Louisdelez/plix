# Quickstart: Movement Polish Implementation

**Feature**: 008-movement-polish
**Date**: 2025-12-15

## Prerequisites

- Rust 1.75+ (stable)
- Existing plix workspace builds: `cargo build --workspace`
- Tests pass: `cargo test --workspace`

## Key Files to Modify

| File | Changes |
|------|---------|
| `crates/plix-common/src/physics.rs` | **NEW** - Shared movement logic |
| `crates/plix-common/src/lib.rs` | Export physics module |
| `crates/plix-server/src/sim/movement.rs` | Update constants, use shared code |
| `crates/plix-server/src/sim/collision.rs` | Rewrite resolution, add step-up |
| `crates/plix-client/src/prediction.rs` | Use shared movement code |
| `crates/plix-client/src/reconciliation.rs` | Add smooth correction |

## Implementation Order

### Step 1: Create Shared Physics Module

Create `crates/plix-common/src/physics.rs`:

```rust
//! Shared movement physics
//!
//! Used by both client (prediction) and server (authoritative)

use crate::math::Vec3;

/// Movement configuration constants
pub struct MovementConfig {
    pub max_speed: f32,
    pub gravity: f32,
    pub jump_impulse: f32,
    pub ground_friction: f32,
    pub air_control: f32,
    pub step_height: f32,
    pub player_half_width: f32,
    pub player_height: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            max_speed: 6.0,
            gravity: 20.0,
            jump_impulse: 7.07,       // sqrt(2 * 20 * 1.25)
            ground_friction: 10.0,
            air_control: 0.3,
            step_height: 0.5,
            player_half_width: 0.4,
            player_height: 1.8,
        }
    }
}

/// Movement state
pub struct MovementState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub is_grounded: bool,
}
```

### Step 2: Update Server Constants

In `crates/plix-server/src/sim/movement.rs`:

```rust
// Replace hardcoded constants
pub const MOVE_SPEED: f32 = 6.0;        // Was 5.0
pub const JUMP_VELOCITY: f32 = 7.07;    // Was 8.0
pub const PLAYER_RADIUS: f32 = 0.4;     // Was 0.3
// GRAVITY and PLAYER_HEIGHT unchanged
```

### Step 3: Fix Collision Resolution Order

In `crates/plix-server/src/sim/collision.rs`:

```rust
pub fn move_and_slide(&self, position: Vec3, velocity: Vec3, dt: f32) -> CollisionResult {
    let mut new_pos = position;
    let mut new_vel = velocity;
    let mut grounded = false;

    // Y FIRST (gravity/jump resolution)
    let test_y = Vec3::new(new_pos.x, position.y + velocity.y * dt, new_pos.z);
    if !self.check_collision(test_y) {
        new_pos.y = test_y.y;
    } else {
        if velocity.y < 0.0 {
            grounded = true;
        }
        new_vel.y = 0.0;
    }

    // Then X
    let test_x = Vec3::new(position.x + velocity.x * dt, new_pos.y, new_pos.z);
    if !self.check_collision(test_x) {
        new_pos.x = test_x.x;
    } else {
        new_vel.x = 0.0;
    }

    // Then Z
    let test_z = Vec3::new(new_pos.x, new_pos.y, position.z + velocity.z * dt);
    if !self.check_collision(test_z) {
        new_pos.z = test_z.z;
    } else {
        new_vel.z = 0.0;
    }

    CollisionResult { position: new_pos, velocity: new_vel, grounded, stepped: false }
}
```

### Step 4: Add Step-Up

```rust
fn try_step_up(&self, position: Vec3, velocity: Vec3, config: &MovementConfig) -> Option<Vec3> {
    // Only attempt if grounded
    if !self.is_grounded(position) {
        return None;
    }

    // Check positions at increasing heights
    for step in 1..=5 {
        let height = step as f32 * 0.1; // 0.1 to 0.5 blocks
        if height > config.step_height {
            break;
        }

        let elevated = Vec3::new(position.x, position.y + height, position.z);

        // Check no head collision
        if self.check_collision(elevated) {
            continue;
        }

        // Check can move horizontally at this height
        let forward = Vec3::new(
            position.x + velocity.x.signum() * 0.1,
            elevated.y,
            position.z + velocity.z.signum() * 0.1,
        );

        if !self.check_collision(forward) {
            return Some(elevated);
        }
    }

    None
}
```

### Step 5: Add Smooth Correction (Client)

In `crates/plix-client/src/reconciliation.rs`:

```rust
pub struct CorrectionSmoother {
    target: Option<Vec3>,
    blend_rate: f32,
}

impl CorrectionSmoother {
    pub fn new() -> Self {
        Self {
            target: None,
            blend_rate: 10.0, // Complete in 100ms
        }
    }

    pub fn set_correction(&mut self, target: Vec3) {
        self.target = Some(target);
    }

    pub fn update(&mut self, current: &mut Vec3, dt: f32) {
        if let Some(target) = self.target {
            let blend = (self.blend_rate * dt).min(1.0);
            *current = current.lerp(target, blend);

            // Clear when close enough
            if (*current - target).length() < 0.001 {
                self.target = None;
            }
        }
    }
}
```

## Testing

### Unit Tests

```bash
# Run movement tests
cargo test -p plix-server movement

# Run collision tests
cargo test -p plix-server collision
```

### Manual Test Scenarios

1. **Collision**: Walk into walls from all angles
2. **Step-up**: Walk onto 0.5-block ledges
3. **Jump**: Verify consistent height (1.25 blocks)
4. **Ground detection**: Jump and verify landing detection
5. **Air control**: Change direction mid-air (should be limited)

### Integration Test

```rust
#[test]
fn test_movement_determinism() {
    let config = MovementConfig::default();
    let arena = load_test_arena();

    let input = PlayerInput { ... };
    let state = MovementState { ... };

    // Simulate twice with same inputs
    let result1 = apply_movement(&input, &state, &arena, &config, 1.0/60.0);
    let result2 = apply_movement(&input, &state, &arena, &config, 1.0/60.0);

    assert_eq!(result1.position, result2.position);
    assert_eq!(result1.velocity, result2.velocity);
}
```

## Verification Checklist

- [ ] Constants updated (MOVE_SPEED=6.0, JUMP_VELOCITY=7.07, PLAYER_RADIUS=0.4)
- [ ] Collision order is Y → X → Z
- [ ] Step-up works for ≤0.5 block obstacles
- [ ] Jump height is 1.25 blocks (±1%)
- [ ] Air control is 30% of ground control
- [ ] Ground detection works at block edges
- [ ] No clipping through walls at max speed
- [ ] Client and server produce identical results
- [ ] Correction smoothing completes in ≤100ms
- [ ] All existing tests still pass
