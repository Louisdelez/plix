# Quickstart: Combat Polish

**Feature**: 009-combat-polish
**Date**: 2025-12-15

## Prerequisites

- Rust 1.75+ (stable)
- Feature 008 (movement-polish) merged (collision system dependency)
- All existing tests passing

## Build & Test

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run combat-specific tests
cargo test -p plix-server combat

# Check formatting and lints
cargo fmt --all -- --check
cargo clippy --all-targets
```

## Quick Implementation Guide

### Step 1: Add CombatConfig (plix-common)

Create `crates/plix-common/src/combat.rs`:

```rust
/// Combat system configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CombatConfig {
    pub attack_cooldown_ticks: u32,
    pub attack_range: f32,
    pub attack_range_epsilon: f32,
    pub knockback_strength: f32,
    pub respawn_invuln_ticks: u32,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            attack_cooldown_ticks: 30,
            attack_range: 1.8,
            attack_range_epsilon: 0.15,
            knockback_strength: 4.0,
            respawn_invuln_ticks: 120,
        }
    }
}
```

Export in `crates/plix-common/src/lib.rs`:
```rust
pub mod combat;
pub use combat::CombatConfig;
```

### Step 2: Add Invulnerability Field (plix-server)

In `crates/plix-server/src/session.rs`, add to `ServerPlayer`:

```rust
pub invulnerable_until_tick: Option<Tick>,
```

Initialize in `ServerPlayer::new`:
```rust
invulnerable_until_tick: None,
```

### Step 3: Update Spawn Logic

Modify `ServerPlayer::spawn` to accept config and set invulnerability:

```rust
pub fn spawn(&mut self, position: Vec3, yaw: f32, current_tick: Tick, config: &CombatConfig) {
    // ... existing spawn logic ...
    self.invulnerable_until_tick = Some(Tick(current_tick.0 + config.respawn_invuln_ticks));
}
```

### Step 4: Update Combat Validation

In `crates/plix-server/src/sim/combat.rs`, update `try_attack`:

```rust
pub fn try_attack(
    &self,
    config: &CombatConfig,  // NEW parameter
    // ... existing params ...
) -> Option<(PlayerId, HitResult)> {
    // Use config.attack_cooldown_ticks instead of constant
    if ticks_since_attack < config.attack_cooldown_ticks {
        return None;
    }

    // Use config.effective_range() for distance check
    if distance > config.attack_range + config.attack_range_epsilon {
        continue;
    }

    // ... rest of implementation
}
```

### Step 5: Add Invulnerability Check

Before applying damage, check invulnerability:

```rust
// Check if target is invulnerable
if target_invulnerable_until.map(|t| current_tick.0 < t.0).unwrap_or(false) {
    return None;  // Attack blocked
}
```

### Step 6: Apply Knockback

On successful hit, calculate and apply knockback:

```rust
let knockback_dir = (target_pos - attacker_pos).normalize_or_zero();
// Apply to victim's velocity in the caller
```

### Step 7: Run Tests

```bash
# Verify all tests pass
cargo test --workspace

# Check for regressions
cargo test -p plix-server -- --include-ignored
```

## Verification Checklist

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes (all 226+ tests)
- [ ] `cargo clippy --all-targets` has no warnings
- [ ] `cargo fmt --all -- --check` passes
- [ ] Combat cooldown enforced at 30 ticks
- [ ] Attack range is 1.8 + 0.15 epsilon
- [ ] Knockback applied on valid hits
- [ ] Respawn grants 2s invulnerability
- [ ] Invulnerable players immune to damage/knockback

## Common Issues

### "Cannot find CombatConfig"
Ensure you've added the module to `plix-common/src/lib.rs`:
```rust
pub mod combat;
pub use combat::CombatConfig;
```

### "spawn() expects different arguments"
Update all spawn() call sites to pass `current_tick` and `&config`.

### Tests Failing After Range Change
Update test expectations from 2.0 to 1.8 + 0.15 = 1.95 effective range.
