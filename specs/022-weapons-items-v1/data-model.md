# Data Model: Weapons & Items v1

**Feature**: 022-weapons-items-v1
**Date**: 2025-12-17

## Overview

This document defines the data entities for the weapons system, their fields, relationships, and state transitions.

---

## Entities

### 1. WeaponType (Enum)

Distinguishes melee from ranged weapons.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponType {
    /// Instant hit detection (sword, fist)
    Melee,
    /// Spawns projectile entities (bow)
    Ranged,
}
```

**Location**: `plix-common/src/inventory/item.rs` (extend existing)

---

### 2. WeaponDef (Static Data)

Immutable weapon definition. Stored as static constants.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `ItemId` | Links to item system |
| `name` | `&'static str` | Display name |
| `weapon_type` | `WeaponType` | Melee or Ranged |
| `damage` | `u16` | Damage per hit |
| `cooldown_ticks` | `u32` | Ticks between uses (60 TPS) |
| `range` | `f32` | Melee: cone radius. Ranged: unused |
| `cone_angle_deg` | `f32` | Melee: cone half-angle in degrees |
| `projectile_speed` | `f32` | Ranged: blocks per second |
| `projectile_lifetime_ticks` | `u32` | Ranged: max ticks alive |
| `base_spread_deg` | `f32` | Ranged: base accuracy spread |
| `recoil_per_shot_deg` | `f32` | Spread penalty per shot |
| `recoil_recovery_ticks` | `u32` | Ticks to fully recover |
| `recoil_max_deg` | `f32` | Maximum accumulated spread |

**V1 Constants**:

| Weapon | damage | cooldown | range | cone | speed | lifetime | spread | recoil |
|--------|--------|----------|-------|------|-------|----------|--------|--------|
| Sword | 25 | 36 (0.6s) | 2.5 | 30° (60° full) | - | - | - | - |
| Bow | 15 | 48 (0.8s) | - | - | 30.0 | 180 (3s) | 2.0° | 1.0° |
| Fist | 10 | 36 (0.6s) | 2.0 | 30° | - | - | - | - |

**Location**: `plix-server/src/weapons/defs.rs`

---

### 3. ProjectileId (Identifier)

Unique identifier for active projectiles.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectileId {
    /// Slot index in projectile array (0-127)
    pub index: u16,
    /// Generation counter for slot reuse detection
    pub generation: u16,
}
```

**Validation**: ID is valid if `generation` matches current slot generation.

**Location**: `plix-common/src/types.rs`

---

### 4. Projectile (Server Entity)

Active projectile tracked by server.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `ProjectileId` | Unique identifier |
| `owner` | `PlayerId` | Player who fired it |
| `position` | `Vec3` | Current world position |
| `velocity` | `Vec3` | Movement per tick |
| `damage` | `u16` | Damage on hit |
| `spawn_tick` | `Tick` | Tick when spawned |
| `ttl_remaining` | `u32` | Ticks until expiry |

**State Transitions**:
```
[Spawned] → tick → [Moving] → collision → [Impacted] → despawn
                  ↓
                  ttl=0 → [Expired] → despawn
```

**Location**: `plix-server/src/weapons/projectiles.rs`

---

### 5. ProjectileManager (Server State)

Manages all active projectiles.

| Field | Type | Description |
|-------|------|-------------|
| `slots` | `Vec<Option<Projectile>>` | Projectile storage (128 capacity) |
| `generations` | `Vec<u16>` | Generation counter per slot |
| `count` | `usize` | Current active count |

**Invariants**:
- `slots.len() == 128`
- `count <= 128`
- Slot contains `Some(p)` iff projectile is active

**Location**: `plix-server/src/weapons/projectiles.rs`

---

### 6. CooldownState (Per-Player)

Tracks weapon cooldowns for a single player.

| Field | Type | Description |
|-------|------|-------------|
| `sword_ready_tick` | `Tick` | Tick when sword can be used again |
| `bow_ready_tick` | `Tick` | Tick when bow can be used again |
| `fist_ready_tick` | `Tick` | Tick when fist can be used again |

**Alternative design**: `HashMap<ItemId, Tick>` for extensibility.

**Location**: `plix-server/src/weapons/cooldown.rs`

---

### 7. RecoilState (Per-Player)

Tracks accuracy penalty from rapid firing.

| Field | Type | Description |
|-------|------|-------------|
| `current_spread` | `f32` | Accumulated spread penalty (degrees) |
| `last_shot_tick` | `Tick` | Tick of last ranged attack |

**Behavior**:
- On shot: `current_spread += recoil_per_shot` (capped at `recoil_max`)
- On query: Decay linearly to 0 over `recoil_recovery_ticks`

**Location**: `plix-server/src/weapons/recoil.rs`

---

### 8. PlayerWeaponState (Per-Player Aggregate)

Combined weapon state for a player session.

| Field | Type | Description |
|-------|------|-------------|
| `cooldowns` | `CooldownState` | Per-weapon cooldowns |
| `recoil` | `RecoilState` | Accuracy penalty state |

**Location**: `plix-server/src/weapons/mod.rs`

---

## Relationships

```
┌─────────────────────────────────────────────────────────┐
│                     plix-common                         │
├─────────────────────────────────────────────────────────┤
│  ItemId ────────────────┐                              │
│  WeaponType (enum)      │                              │
│  ProjectileId           │                              │
└─────────────────────────│──────────────────────────────┘
                          │
┌─────────────────────────│──────────────────────────────┐
│                     plix-server                         │
├─────────────────────────│──────────────────────────────┤
│                         ▼                              │
│  WeaponDef ◄──── ItemId                               │
│      │                                                 │
│      │ defines                                         │
│      ▼                                                 │
│  CooldownState ◄──── PlayerSession                    │
│  RecoilState ◄────┘                                   │
│                                                        │
│  Projectile ─────────────────────►ProjectileManager   │
│      │                                   │             │
│      └──────── PlayerId (owner) ◄────────┘             │
└────────────────────────────────────────────────────────┘
```

---

## Validation Rules

### WeaponDef
- `damage > 0`
- `cooldown_ticks >= 1`
- `range > 0` for melee
- `projectile_speed > 0` for ranged
- `base_spread_deg >= 0`

### Projectile
- `owner.is_valid()`
- `ttl_remaining > 0` when active
- `velocity.length() > 0`

### CooldownState
- Ready tick can be in the future (cooldown active) or past (ready)

### RecoilState
- `current_spread >= 0`
- `current_spread <= recoil_max_deg`

---

## Database Schema

N/A - All state is in-memory only. Projectiles, cooldowns, and recoil reset on server restart or match end.
