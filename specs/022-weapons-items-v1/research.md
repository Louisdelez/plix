# Research: Weapons & Items v1

**Feature**: 022-weapons-items-v1
**Date**: 2025-12-17

## Overview

This document captures research decisions for implementing the weapons system, resolving technical unknowns identified during planning.

---

## Research Item 1: Projectile Entity Storage

### Context
Need to store up to 128 concurrent projectile entities with fast iteration, insertion, and removal.

### Decision
Use `Vec<Option<Projectile>>` with slot reuse and generation IDs for validity checking.

### Rationale
- **Cache-friendly**: Contiguous memory layout for O(n) tick updates
- **Bounded**: Fixed capacity of 128 prevents unbounded allocation
- **Simple**: No external dependencies, standard Rust patterns
- **Fast removal**: Swap with None, no shifting required

### Alternatives Considered
| Alternative | Rejected Because |
|------------|------------------|
| `HashMap<ProjectileId, Projectile>` | Slower iteration, poor cache locality |
| `slab` crate | External dependency for simple use case |
| Linked list | Poor cache locality, complex lifetime management |

### Implementation Notes
```rust
pub struct ProjectileId {
    pub index: u16,      // Slot index (0-127)
    pub generation: u16, // Incremented on reuse
}

pub struct ProjectileManager {
    projectiles: Vec<Option<Projectile>>,
    generations: Vec<u16>,
    count: usize,
}
```

---

## Research Item 2: Collision Detection Strategy

### Context
Projectiles need to detect collisions with players (capsule hitboxes) and blocks (voxel grid).

### Decision
- **Players**: Sphere-vs-capsule intersection using existing `PLAYER_HALF_WIDTH` and `PLAYER_HALF_HEIGHT` constants
- **Blocks**: Discrete stepping along velocity vector, check block at each step

### Rationale
- **Consistency**: Uses same hitbox dimensions as existing melee combat system
- **Simplicity**: No need for continuous collision detection at projectile speeds
- **Performance**: O(players + steps_per_projectile) per tick

### Alternatives Considered
| Alternative | Rejected Because |
|------------|------------------|
| Continuous raycast | Overcomplicated for arrow speeds, blocks are discrete |
| Spatial partitioning | Overkill for 128 projectiles + 16 players |
| Sphere-sphere only | Would miss tall players, inconsistent with melee |

### Implementation Notes
```rust
// Step size for discrete collision (half projectile radius)
const COLLISION_STEP_SIZE: f32 = 0.25;

fn check_block_collision(from: Vec3, to: Vec3, world: &World) -> Option<BlockPos> {
    let dir = (to - from).normalize_or_zero();
    let dist = (to - from).length();
    let steps = (dist / COLLISION_STEP_SIZE).ceil() as usize;

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let pos = from + dir * (dist * t);
        let block_pos = BlockPos::from_vec3(pos);
        if world.get_block(block_pos).is_solid() {
            return Some(block_pos);
        }
    }
    None
}
```

---

## Research Item 3: Spread/Accuracy System

### Context
Ranged weapons need accuracy mechanics: base spread, movement penalty, server-authoritative calculation.

### Decision
Server calculates final shot direction by applying random offset within spread cone.

### Rationale
- **Anti-cheat**: Client cannot manipulate spread to achieve perfect accuracy
- **Determinism**: Seed RNG with tick + player_id for reproducible results
- **Simplicity**: Single random rotation, no complex ballistics

### Alternatives Considered
| Alternative | Rejected Because |
|------------|------------------|
| Client-side spread | Cheat-vulnerable (client could send perfect aim) |
| Per-frame spread | Overcomplicated, tick-based is sufficient |
| Projectile deviation over time | Complex, out of scope for v1 |

### Implementation Notes
```rust
// Base spread for bow: ±2 degrees
const BOW_BASE_SPREAD_DEG: f32 = 2.0;
// Movement penalty: +50% spread when moving
const MOVEMENT_SPREAD_MULTIPLIER: f32 = 1.5;

fn calculate_spread(base_spread_deg: f32, is_moving: bool, recoil_spread: f32) -> f32 {
    let mut spread = base_spread_deg + recoil_spread;
    if is_moving {
        spread *= MOVEMENT_SPREAD_MULTIPLIER;
    }
    spread.min(MAX_SPREAD_DEG)
}

fn apply_spread(direction: Vec3, spread_deg: f32, rng: &mut impl Rng) -> Vec3 {
    let spread_rad = spread_deg.to_radians();
    let offset_angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let offset_magnitude = rng.gen_range(0.0..spread_rad);
    // Apply rotation offset to direction vector
    rotate_around_axis(direction, offset_angle, offset_magnitude)
}
```

---

## Research Item 4: Recoil Model

### Context
Rapid firing should accumulate spread penalty (recoil). Need decay mechanism.

### Decision
Additive spread penalty with linear decay over ticks.

### Rationale
- **Simple**: No complex curves or animations
- **Server-only**: No client state synchronization needed
- **Tunable**: Easy to adjust per-weapon values

### Parameters (from spec assumptions)
- Recoil per shot: +1 degree spread
- Recoil recovery window: 0.5s (30 ticks at 60 TPS)
- Recoil maximum cap: +5 degrees

### Alternatives Considered
| Alternative | Rejected Because |
|------------|------------------|
| Visual camera recoil | Client-side, out of scope for v1 |
| Multiplicative recoil | More complex, harder to balance |
| Per-weapon recoil patterns | Overcomplicated for 2 weapons |

### Implementation Notes
```rust
pub struct RecoilState {
    /// Current accumulated spread penalty (degrees)
    pub current_spread: f32,
    /// Tick when last shot was fired
    pub last_shot_tick: Tick,
}

impl RecoilState {
    pub fn add_recoil(&mut self, amount: f32, current_tick: Tick) {
        self.current_spread = (self.current_spread + amount).min(MAX_RECOIL_SPREAD);
        self.last_shot_tick = current_tick;
    }

    pub fn get_spread(&self, current_tick: Tick) -> f32 {
        let ticks_since_shot = current_tick.0.wrapping_sub(self.last_shot_tick.0);
        if ticks_since_shot >= RECOIL_RECOVERY_TICKS {
            0.0 // Fully recovered
        } else {
            // Linear decay
            let recovery_progress = ticks_since_shot as f32 / RECOIL_RECOVERY_TICKS as f32;
            self.current_spread * (1.0 - recovery_progress)
        }
    }
}
```

---

## Research Item 5: Projectile Speed and Lifetime

### Context
Need to determine arrow speed and lifetime values for v1.

### Decision
- **Speed**: 30 blocks/second (0.5 blocks/tick at 60 TPS)
- **Lifetime**: 3 seconds (180 ticks)
- **Max range**: ~90 blocks (speed × lifetime)

### Rationale
- **Speed**: Fast enough to be responsive, slow enough to dodge at range
- **Lifetime**: Long enough for cross-arena shots, bounded for cleanup
- **Range**: Covers typical arena sizes (64x64 to 128x128 blocks)

### Alternatives Considered
| Alternative | Rejected Because |
|------------|------------------|
| Instant hitscan | Not satisfying gameplay, trivializes aiming |
| Very slow (10 b/s) | Frustrating at medium range |
| Very fast (50 b/s) | Hard to dodge, reduces skill expression |

---

## Research Item 6: Protocol Events

### Context
Need efficient network replication for projectiles without per-tick updates.

### Decision
Event-based replication with three message types:
1. `ProjectileSpawn` - Initial state for client interpolation
2. `ProjectileImpact` - Hit feedback
3. `ProjectileDespawn` - Cleanup (timeout/limit)

### Rationale
- **Bandwidth efficient**: 3 events per projectile vs 180 position updates
- **Client prediction**: Spawn data sufficient for client-side interpolation
- **Feedback**: Impact events enable hit effects without waiting for snapshot

### Message Sizes (estimated)
| Message | Fields | Size |
|---------|--------|------|
| ProjectileSpawn | id(4) + owner(2) + pos(12) + vel(12) + tick(4) | ~34 bytes |
| ProjectileImpact | id(4) + pos(12) + target(2) + type(1) | ~19 bytes |
| ProjectileDespawn | id(4) + reason(1) | ~5 bytes |

---

## Summary

All technical unknowns resolved. Ready to proceed to Phase 1 design artifacts.
