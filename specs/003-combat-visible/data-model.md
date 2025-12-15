# Data Model: Server-Authoritative Combat System

**Feature**: 003-combat-visible
**Date**: 2025-12-14

## Overview

This document describes the data entities involved in the combat system. Most entities already exist in the codebase; new additions are marked with **(NEW)**.

## Entities

### Player (ServerPlayer)

**Location**: `crates/plix-server/src/session.rs`
**Status**: Exists (no changes needed)

| Field | Type | Description |
|-------|------|-------------|
| id | PlayerId | Unique player identifier |
| name | String | Display name |
| team | TeamId | Team assignment (0, 1, or NONE) |
| addr | SocketAddr | Network address |
| position | Vec3 | World position |
| rotation | Rotation | Yaw/pitch facing |
| velocity | Vec3 | Movement velocity |
| health | u8 | Current HP (0-100) |
| is_dead | bool | Death state |
| respawn_tick | Option\<Tick\> | When to respawn (None if alive) |
| last_attack_tick | Tick | Last attack time (for cooldown) |
| last_input_seq | InputSeq | Last processed input sequence |
| pending_inputs | Vec\<PlayerInput\> | Queued inputs |
| kills | u16 | Kill count |
| deaths | u16 | Death count |

**State Transitions**:
```
Alive (health > 0, is_dead = false)
  ↓ [takes fatal damage]
Dead (health = 0, is_dead = true, respawn_tick = Some(current + delay))
  ↓ [respawn_tick reached]
Alive (health = 100, is_dead = false, position = spawn point)
```

### PlayerInput

**Location**: `crates/plix-common/src/protocol/messages.rs`
**Status**: Exists (no changes needed)

| Field | Type | Description |
|-------|------|-------------|
| seq | u16 | Input sequence number |
| tick | Tick | Client's estimated server tick |
| move_forward | f32 | Forward/backward axis (-1.0 to 1.0) |
| move_right | f32 | Left/right axis (-1.0 to 1.0) |
| jump | bool | Jump action |
| crouch | bool | Crouch action |
| attack | bool | Attack action |
| yaw | f32 | Horizontal look (radians) |
| pitch | f32 | Vertical look (radians) |

### HitResult

**Location**: `crates/plix-server/src/sim/combat.rs`
**Status**: Exists (no changes needed)

| Field | Type | Description |
|-------|------|-------------|
| attacker | PlayerId | Who attacked |
| target | PlayerId | Who was hit |
| damage | u8 | Damage dealt |
| killed | bool | Whether target died |

### GameEvent (Enum)

**Location**: `crates/plix-server/src/replication/events.rs`
**Status**: Extend with new variants

Existing variants (no changes):
- `PlayerJoined { id, name, team }`
- `PlayerLeft { id }`
- `PlayerDied { victim, killer: Option<PlayerId> }`
- `PlayerRespawned { id }`
- `RoundStart { round }`
- `RoundEnd { winner: Option<TeamId> }`
- `MatchEnd { winner: Option<TeamId> }`

**New variants (NEW)**:
```rust
HitConfirmed {
    attacker: PlayerId,  // Who landed the hit
    target: PlayerId,    // Who was hit
    damage: u8,          // Damage dealt
}

DamageTaken {
    victim: PlayerId,    // Who took damage
    attacker: PlayerId,  // Who dealt it
    damage: u8,          // Damage amount
    new_health: u8,      // Victim's remaining HP
}
```

### Combat Constants

**Location**: `crates/plix-server/src/sim/combat.rs`
**Status**: Exists (no changes needed)

| Constant | Value | Description |
|----------|-------|-------------|
| MELEE_DAMAGE | 20 | Damage per hit |
| ATTACK_COOLDOWN_TICKS | 30 | Cooldown in ticks (500ms at 60 Hz) |
| ATTACK_RANGE | 2.0 | Attack reach in blocks |

### Match Config (Respawn)

**Location**: `crates/plix-server/src/match_state.rs`
**Status**: Exists (no changes needed)

| Field | Value | Description |
|-------|-------|-------------|
| respawn_delay | 180 | Respawn wait in ticks (3 seconds at 60 Hz) |

## Relationships

```
Player 1 --attacks--> Player 2
    |                    |
    | emits              | emits
    ↓                    ↓
HitConfirmed         DamageTaken
(to attacker)        (to victim)
                         |
                         | if killed
                         ↓
                     PlayerDied
                     (broadcast)
                         |
                         | after delay
                         ↓
                   PlayerRespawned
                     (broadcast)
```

## Validation Rules

### Attack Validation (Server)
1. Attacker must be alive (`is_dead == false`)
2. Attacker must have cooldown ready (`current_tick - last_attack_tick >= ATTACK_COOLDOWN_TICKS`)
3. Target must be in range (`distance <= ATTACK_RANGE`)
4. Target must be in facing cone (dot product check)
5. Target must be alive (`is_dead == false`)
6. Match phase must be `Playing`

### Damage Application (Server)
1. Damage capped at remaining health (`min(damage, health)`)
2. Death triggered when `health - damage <= 0`
3. On death: `is_dead = true`, `respawn_tick = current_tick + respawn_delay`

### Respawn Execution (Server)
1. Triggered when `current_tick >= respawn_tick`
2. Reset: `health = 100`, `is_dead = false`, `respawn_tick = None`
3. Position set to team spawn point

## Client State (Read-Only)

Client receives player state via snapshots. Relevant fields for combat:

| Field | Source | Usage |
|-------|--------|-------|
| health | Snapshot | Display in HUD |
| is_dead | Snapshot | Skip rendering if true |

Client receives events for feedback:

| Event | Action |
|-------|--------|
| HitConfirmed | Show hit indicator (attacker only) |
| DamageTaken | Show damage flash (victim only) |
| PlayerDied | Show kill feed message |
| PlayerRespawned | Resume rendering player |
