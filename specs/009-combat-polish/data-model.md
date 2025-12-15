# Data Model: Combat Polish

**Feature**: 009-combat-polish
**Date**: 2025-12-15

## New Structures

### CombatConfig

**Location**: `crates/plix-common/src/combat.rs`

**Purpose**: Centralized combat configuration shared between client and server.

```rust
/// Combat system configuration.
/// Shared between client (for animation prediction) and server (for validation).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CombatConfig {
    /// Attack cooldown in ticks (default: 30 = 0.5s at 60Hz)
    pub attack_cooldown_ticks: u32,

    /// Base attack range in blocks (default: 1.8)
    pub attack_range: f32,

    /// Latency tolerance added to attack range (default: 0.15)
    pub attack_range_epsilon: f32,

    /// Knockback velocity impulse in m/s (default: 4.0)
    pub knockback_strength: f32,

    /// Respawn invulnerability duration in ticks (default: 120 = 2.0s at 60Hz)
    pub respawn_invuln_ticks: u32,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            attack_cooldown_ticks: 30,      // 0.5 seconds at 60Hz
            attack_range: 1.8,               // blocks
            attack_range_epsilon: 0.15,      // blocks
            knockback_strength: 4.0,         // m/s
            respawn_invuln_ticks: 120,       // 2.0 seconds at 60Hz
        }
    }
}

impl CombatConfig {
    /// Get effective attack range including latency tolerance
    pub fn effective_range(&self) -> f32 {
        self.attack_range + self.attack_range_epsilon
    }
}
```

**Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `attack_cooldown_ticks` | `u32` | 30 | Minimum ticks between attacks |
| `attack_range` | `f32` | 1.8 | Base melee range in blocks |
| `attack_range_epsilon` | `f32` | 0.15 | Latency forgiveness radius |
| `knockback_strength` | `f32` | 4.0 | Knockback impulse velocity (m/s) |
| `respawn_invuln_ticks` | `u32` | 120 | Invulnerability duration after spawn |

---

## Modified Structures

### ServerPlayer

**Location**: `crates/plix-server/src/session.rs`

**Changes**: Add invulnerability tracking field.

```rust
pub struct ServerPlayer {
    // ... existing fields ...

    /// Tick when invulnerability expires (None if vulnerable)
    pub invulnerable_until_tick: Option<Tick>,
}
```

**Initialization** (in `ServerPlayer::new`):
```rust
invulnerable_until_tick: None,
```

**Spawn Update** (in `ServerPlayer::spawn`):
```rust
pub fn spawn(&mut self, position: Vec3, yaw: f32, current_tick: Tick, config: &CombatConfig) {
    self.position = position;
    self.rotation = Rotation::new(yaw, 0.0);
    self.velocity = Vec3::ZERO;
    self.health = 100;
    self.is_dead = false;
    self.respawn_tick = None;
    // NEW: Grant invulnerability
    self.invulnerable_until_tick = Some(Tick(current_tick.0 + config.respawn_invuln_ticks));
    // Reset anti-cheat
    self.anti_cheat.update_position(position);
}
```

**New Method**:
```rust
/// Check if player is currently invulnerable
pub fn is_invulnerable(&self, current_tick: Tick) -> bool {
    self.invulnerable_until_tick
        .map(|until| current_tick.0 < until.0)
        .unwrap_or(false)
}
```

---

### HitResult

**Location**: `crates/plix-server/src/sim/combat.rs`

**Changes**: Add knockback direction for applying impulse.

```rust
pub struct HitResult {
    pub attacker: PlayerId,
    pub target: PlayerId,
    pub damage: u8,
    pub killed: bool,
    /// Knockback direction (normalized, attacker → victim)
    pub knockback_dir: Vec3,
}
```

---

## Removed/Deprecated Constants

### validation.rs Changes

**Remove hardcoded constants** (move to CombatConfig):
```rust
// DEPRECATED - use CombatConfig instead
// pub const ATTACK_COOLDOWN_TICKS: u32 = 30;
// pub const ATTACK_RANGE: f32 = 2.0;
```

**Keep for backwards compatibility during migration**:
```rust
/// Attack cooldown in ticks - use CombatConfig.attack_cooldown_ticks instead
#[deprecated(note = "Use CombatConfig.attack_cooldown_ticks")]
pub const ATTACK_COOLDOWN_TICKS: u32 = 30;

/// Attack range - use CombatConfig.attack_range instead
#[deprecated(note = "Use CombatConfig.attack_range")]
pub const ATTACK_RANGE: f32 = 1.8;  // Updated from 2.0

/// Latency tolerance for range check
pub const ATTACK_RANGE_EPSILON: f32 = 0.15;
```

---

## State Transitions

### Player Combat State

```
┌─────────────────────────────────────────────────────────┐
│                    Player State                          │
├─────────────────────────────────────────────────────────┤
│                                                          │
│   ┌─────────┐    spawn()     ┌──────────────┐            │
│   │  Dead   │ ───────────────▶│ Invulnerable │            │
│   └─────────┘                 └──────────────┘            │
│        ▲                            │                     │
│        │                            │ tick >= invuln_until│
│        │ take_damage()              │                     │
│        │ (killed=true)              ▼                     │
│        │                      ┌──────────────┐            │
│        └──────────────────────│  Vulnerable  │            │
│                               └──────────────┘            │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Attack Validation Flow

```
┌─────────────────────────────────────────────────────────┐
│                  Attack Validation                       │
├─────────────────────────────────────────────────────────┤
│                                                          │
│   attack_input                                           │
│        │                                                 │
│        ▼                                                 │
│   ┌─────────────┐                                        │
│   │ On Cooldown?│──── Yes ────▶ Reject (silent)          │
│   └─────────────┘                                        │
│        │ No                                              │
│        ▼                                                 │
│   ┌─────────────┐                                        │
│   │ In Range?   │──── No ─────▶ Miss (no target)         │
│   │ (+ epsilon) │                                        │
│   └─────────────┘                                        │
│        │ Yes                                             │
│        ▼                                                 │
│   ┌─────────────┐                                        │
│   │ Target      │──── Yes ────▶ Blocked (no effect)      │
│   │ Invuln?     │                                        │
│   └─────────────┘                                        │
│        │ No                                              │
│        ▼                                                 │
│   ┌─────────────┐                                        │
│   │ Apply Hit   │──── damage + knockback                 │
│   └─────────────┘                                        │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

## Validation Rules

### CombatConfig

| Field | Validation | Error |
|-------|------------|-------|
| `attack_cooldown_ticks` | > 0 | "Cooldown must be positive" |
| `attack_range` | > 0.0 | "Range must be positive" |
| `attack_range_epsilon` | >= 0.0 | "Epsilon must be non-negative" |
| `knockback_strength` | >= 0.0 | "Knockback must be non-negative" |
| `respawn_invuln_ticks` | >= 0 | "Invuln duration must be non-negative" |

### Attack Validation

| Check | Condition | Result |
|-------|-----------|--------|
| Cooldown | `current_tick - last_attack_tick >= cooldown` | Pass/Fail |
| Range | `distance <= attack_range + epsilon` | Pass/Fail |
| Invulnerability | `current_tick >= invulnerable_until_tick` | Pass/Blocked |

---

## Module Exports

### plix-common/src/lib.rs

```rust
pub mod combat;
pub use combat::CombatConfig;
```

### plix-server/src/sim/mod.rs

No changes - CombatSystem already exported.
