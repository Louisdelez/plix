# Protocol Contracts: Weapons & Items v1

**Feature**: 022-weapons-items-v1
**Date**: 2025-12-17

## Overview

This document defines the network protocol messages for the weapons system. All messages use bincode serialization over UDP (existing plix-net transport).

---

## Client → Server Messages

### UseActiveItem (Existing - Enhanced)

Already exists in `ClientMessage::UseActiveItem`. No changes needed. The server routes this to the appropriate weapon system based on the active hotbar item.

**Behavior**:
- If active item is `ItemId::SWORD` → `MeleeSystem::try_attack()`
- If active item is `ItemId::BOW` → `RangedSystem::try_shoot()`
- If active slot is empty → `MeleeSystem::try_attack()` with fist (default melee)
- Other items → existing item use behavior

---

## Server → Client Messages

### New GameEvent Variants

Add to `GameEvent` enum in `plix-common/src/protocol/messages.rs`:

```rust
/// Projectile spawned (broadcast to all clients)
ProjectileSpawn {
    /// Unique projectile identifier
    id: ProjectileId,
    /// Player who fired it
    owner: PlayerId,
    /// Initial world position
    position: Vec3,
    /// Velocity vector (blocks per tick)
    velocity: Vec3,
    /// Server tick when spawned (for client interpolation)
    spawn_tick: Tick,
},

/// Projectile hit something (broadcast to all clients)
ProjectileImpact {
    /// Projectile identifier
    id: ProjectileId,
    /// Impact world position
    position: Vec3,
    /// What was hit
    impact_type: ProjectileImpactType,
    /// Target player/bot if applicable
    target: Option<PlayerId>,
},

/// Projectile removed without impact (broadcast to all clients)
ProjectileDespawn {
    /// Projectile identifier
    id: ProjectileId,
    /// Reason for despawn
    reason: ProjectileDespawnReason,
},

/// Weapon attack rejected (sent to attacker only)
WeaponCooldown {
    /// Item that was on cooldown
    item_id: ItemId,
    /// Ticks remaining until ready
    remaining_ticks: u32,
},

/// Projectile spawn rejected due to limit (sent to shooter only)
ProjectileLimitReached {
    /// Current projectile count
    current_count: u8,
    /// Maximum allowed
    max_count: u8,
},
```

### Supporting Types

```rust
/// What the projectile hit
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ProjectileImpactType {
    /// Hit a player
    Player,
    /// Hit a training bot
    Bot,
    /// Hit a solid block
    Block,
}

/// Why projectile was despawned without impact
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ProjectileDespawnReason {
    /// Lifetime expired
    Timeout,
    /// Server projectile limit exceeded (shouldn't happen often)
    LimitPurge,
    /// Owner disconnected
    OwnerLeft,
}
```

---

## Message Flow Diagrams

### Successful Melee Attack

```
Client                          Server
  │                               │
  │── UseActiveItem ──────────────►│
  │                               │ validate cooldown
  │                               │ cone hit detection
  │                               │ apply damage
  │◄─ HitConfirmed ───────────────│ (to attacker)
  │◄─ DamageTaken ────────────────│ (to victim)
  │                               │
```

### Successful Ranged Attack

```
Client                          Server
  │                               │
  │── UseActiveItem ──────────────►│
  │                               │ validate cooldown
  │                               │ check projectile limit
  │                               │ calculate spread
  │                               │ spawn projectile
  │◄─ ProjectileSpawn ────────────│ (broadcast)
  │                               │
  │                               │ ... projectile moves each tick ...
  │                               │
  │                               │ collision detected
  │                               │ apply damage
  │◄─ ProjectileImpact ───────────│ (broadcast)
  │◄─ HitConfirmed ───────────────│ (to attacker)
  │◄─ DamageTaken ────────────────│ (to victim)
  │                               │
```

### Attack Rejected (Cooldown)

```
Client                          Server
  │                               │
  │── UseActiveItem ──────────────►│
  │                               │ cooldown still active
  │◄─ WeaponCooldown ─────────────│
  │                               │
```

### Projectile Spawn Rejected (Limit)

```
Client                          Server
  │                               │
  │── UseActiveItem ──────────────►│
  │                               │ cooldown OK
  │                               │ projectile count = 128
  │◄─ ProjectileLimitReached ─────│
  │                               │
  │                               │ (cooldown still triggered)
```

### Projectile Timeout

```
                                Server
                                  │
                                  │ projectile ttl = 0
                                  │
Clients ◄─ ProjectileDespawn ─────│ (broadcast)
                                  │
```

---

## Serialization Format

All messages use bincode with default configuration:
- Little-endian byte order
- Variable-length integers
- No length prefix (UDP packets are sized)

### Estimated Sizes

| Message | Approx Size (bytes) |
|---------|---------------------|
| ProjectileSpawn | 34-38 |
| ProjectileImpact | 18-22 |
| ProjectileDespawn | 6-8 |
| WeaponCooldown | 8-10 |
| ProjectileLimitReached | 4-6 |

---

## Compatibility

### Existing Messages (Unchanged)
- `ClientMessage::UseActiveItem` - Behavior extended, format unchanged
- `GameEvent::HitConfirmed` - Used for both melee and ranged hits
- `GameEvent::DamageTaken` - Used for both melee and ranged damage
- `GameEvent::PlayerDied` - Used when weapon kills target

### Protocol Version
Increment `PROTOCOL_VERSION` in plix-common when these messages are added.

---

## Anti-Cheat Considerations

- All validation is server-side
- Client cannot specify damage, spread, or projectile parameters
- Cooldown enforcement prevents rapid-fire exploits
- Projectile limit prevents memory exhaustion attacks
- Projectile owner field enables kill attribution
