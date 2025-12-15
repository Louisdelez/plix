# Protocol Contract: Combat Events

**Feature**: 003-combat-visible
**Date**: 2025-12-14
**Protocol Version**: Unchanged (compatible extension)

## Overview

This document specifies the protocol messages added for combat feedback. These are **backward compatible additions** to the existing `GameEvent` enum.

## Message Format

All messages use bincode serialization with serde. Existing `ServerMessage::Event` wrapper carries `GameEvent` variants.

## New Event Types

### HitConfirmed

**Direction**: Server → Client (attacker only)
**Reliability**: Reliable (via event channel)
**Purpose**: Inform attacker their attack landed

```rust
GameEvent::HitConfirmed {
    attacker: PlayerId,  // Should match recipient's ID
    target: PlayerId,    // Who was hit
    damage: u8,          // Damage dealt (1-100)
}
```

**Trigger**: Server validates successful melee attack
**Recipient**: Only the attacking player
**Client Action**: Display hit confirmation feedback (e.g., "HIT" text, crosshair flash)

### DamageTaken

**Direction**: Server → Client (victim only)
**Reliability**: Reliable (via event channel)
**Purpose**: Inform victim they took damage

```rust
GameEvent::DamageTaken {
    victim: PlayerId,    // Should match recipient's ID
    attacker: PlayerId,  // Who dealt damage
    damage: u8,          // Damage amount (1-100)
    new_health: u8,      // Remaining HP (0-100)
}
```

**Trigger**: Server applies damage to player
**Recipient**: Only the damaged player
**Client Action**: Display damage feedback (screen flash, damage number), update HP display

## Existing Events (Unchanged)

### PlayerDied

```rust
GameEvent::PlayerDied {
    victim: PlayerId,
    killer: Option<PlayerId>,  // None for environmental death
}
```

**Recipient**: Broadcast to all clients
**Client Action**: Display kill feed, stop rendering victim

### PlayerRespawned

```rust
GameEvent::PlayerRespawned {
    id: PlayerId,
}
```

**Recipient**: Broadcast to all clients
**Client Action**: Resume rendering player at new position

## Event Ordering

Combat events are emitted in this order within a single tick:

1. `HitConfirmed` (to attacker)
2. `DamageTaken` (to victim)
3. `PlayerDied` (broadcast, if killed)

Respawn events occur on a later tick:

4. `PlayerRespawned` (broadcast, after respawn_delay ticks)

## Bandwidth Considerations

| Event | Size (estimated) | Frequency |
|-------|------------------|-----------|
| HitConfirmed | ~8 bytes | Per successful attack |
| DamageTaken | ~10 bytes | Per successful attack |
| PlayerDied | ~6 bytes | Per death |
| PlayerRespawned | ~4 bytes | Per respawn |

Maximum combat events per tick (assuming 50 players, all attacking):
- 25 HitConfirmed + 25 DamageTaken = ~450 bytes (negligible)

## Compatibility

- **Backward compatible**: New enum variants added, existing variants unchanged
- **Forward compatible**: Clients ignoring unknown variants will miss feedback only
- **Protocol version**: No increment required (additive change)

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Client receives HitConfirmed for non-local player | Log warning, ignore |
| Client receives DamageTaken for non-local player | Log warning, ignore |
| Event arrives before player snapshot | Queue until player known |
| Network drop loses event | Snapshot contains authoritative state; feedback lost is acceptable |
