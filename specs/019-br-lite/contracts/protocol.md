# Protocol Contract: BR Lite Mode

**Feature**: 019-br-lite
**Date**: 2025-12-16
**Protocol Version**: 1.0

## Overview

This document defines the network protocol messages for BR Lite mode. All messages are server-to-client (server-authoritative). Clients send standard input messages; no BR-specific client messages are required.

## Message Format

All messages are serialized with `bincode` and prefixed with a message type discriminator (u8).

### Message Types

| Type ID | Message | Description |
|---------|---------|-------------|
| 0x20 | `BrZoneUpdate` | Zone state update |
| 0x21 | `BrLootSpawn` | Loot item spawned |
| 0x22 | `BrLootPickup` | Loot collected by player |
| 0x23 | `BrElimination` | Player eliminated |
| 0x24 | `BrVictory` | Match won |

## Message Definitions

### BrZoneUpdate (0x20)

Sent when zone phase changes and periodically every 5 seconds during match.

```rust
pub struct BrZoneUpdate {
    /// Zone center (XZ coordinates)
    pub center: [f32; 2],
    /// Current zone radius
    pub current_radius: f32,
    /// Target radius for current phase
    pub target_radius: f32,
    /// Current phase index (0-indexed)
    pub phase_index: u8,
    /// Phase mode: 0 = Stable, 1 = Shrinking
    pub phase_mode: u8,
    /// Time remaining in current phase (seconds)
    pub phase_time_remaining_secs: u16,
    /// Damage per second outside zone
    pub damage_per_tick: u16,
}
```

**Wire Format** (20 bytes):
```
[0-3]   f32 center_x
[4-7]   f32 center_z
[8-11]  f32 current_radius
[12-15] f32 target_radius
[16]    u8  phase_index
[17]    u8  phase_mode
[18-19] u16 phase_time_remaining_secs
[20-21] u16 damage_per_tick
```

**Trigger Conditions**:
- Match transitions to Playing phase
- Phase mode changes (Stable → Shrinking or vice versa)
- Every 300 ticks (5 seconds) during Playing phase

---

### BrLootSpawn (0x21)

Sent at match start for each loot item in the arena.

```rust
pub struct BrLootSpawn {
    /// Unique loot identifier
    pub loot_id: u16,
    /// World position
    pub position: [f32; 3],
    /// Loot type: 0 = HealthPack, 1 = SpeedBoost
    pub loot_type: u8,
    /// Type-specific parameter (heal amount or speed multiplier * 100)
    pub param: u16,
}
```

**Wire Format** (17 bytes):
```
[0-1]   u16 loot_id
[2-5]   f32 position_x
[6-9]   f32 position_y
[10-13] f32 position_z
[14]    u8  loot_type
[15-16] u16 param
```

**Loot Type Encoding**:
| loot_type | param meaning |
|-----------|---------------|
| 0 (HealthPack) | heal_amount (HP) |
| 1 (SpeedBoost) | multiplier * 100 (e.g., 150 = 1.5x) |

**Trigger Conditions**:
- Match transitions to Playing phase (sent for all loot items)
- Player joins mid-match (sent for uncollected loot only)

---

### BrLootPickup (0x22)

Sent when a player collects a loot item.

```rust
pub struct BrLootPickup {
    /// Loot identifier (matches BrLootSpawn.loot_id)
    pub loot_id: u16,
    /// Player who collected the loot
    pub player_id: u16,
}
```

**Wire Format** (4 bytes):
```
[0-1] u16 loot_id
[2-3] u16 player_id
```

**Trigger Conditions**:
- Server validates player pickup (position overlap, loot not collected)

**Client Behavior**:
- Remove loot item from world render
- Play pickup effect at loot position
- If player_id is local player, show effect UI (heal flash, speed indicator)

---

### BrElimination (0x23)

Sent when a player is eliminated (permanent death).

```rust
pub struct BrElimination {
    /// Eliminated player
    pub player_id: u16,
    /// Remaining alive players
    pub alive_count: u8,
}
```

**Wire Format** (3 bytes):
```
[0-1] u16 player_id
[2]   u8  alive_count
```

**Trigger Conditions**:
- Player dies (combat or zone damage)
- Player disconnects

**Client Behavior**:
- Update player list UI (mark eliminated)
- If local player, show elimination screen / offer spectate option
- Display "X players remaining" UI

---

### BrVictory (0x24)

Sent when a player wins the match.

```rust
pub struct BrVictory {
    /// Winning player
    pub winner_id: u16,
}
```

**Wire Format** (2 bytes):
```
[0-1] u16 winner_id
```

**Trigger Conditions**:
- Only one player remains alive
- All players eliminated simultaneously (lowest ID wins)

**Client Behavior**:
- Display victory screen (winner name)
- If local player is winner, show "WINNER" UI
- Otherwise show "GAME OVER" with winner info

---

## Message Flow

### Match Start

```
Server                              Client
   │                                   │
   │──── BrZoneUpdate (initial) ──────>│
   │──── BrLootSpawn (item 1) ────────>│
   │──── BrLootSpawn (item 2) ────────>│
   │──── BrLootSpawn (item N) ────────>│
   │                                   │
```

### During Match

```
Server                              Client
   │                                   │
   │ (every 5s or phase change)        │
   │──── BrZoneUpdate ────────────────>│
   │                                   │
   │ (on loot pickup)                  │
   │──── BrLootPickup ────────────────>│
   │                                   │
   │ (on player death)                 │
   │──── BrElimination ───────────────>│
   │                                   │
```

### Match End

```
Server                              Client
   │                                   │
   │──── BrElimination (2nd last) ────>│
   │──── BrVictory ───────────────────>│
   │──── MatchPhase(EndScreen) ───────>│
   │                                   │
```

## Compatibility

### Version Negotiation

BR Lite messages use type IDs 0x20-0x24, reserved in the existing protocol range. Clients that don't support BR Lite should ignore unknown message types gracefully.

### Backward Compatibility

- New fields can be appended to messages (bincode handles this)
- Message type IDs are stable within protocol version
- New message types can be added with new IDs

### Forward Compatibility

Clients should:
1. Ignore unknown message type IDs
2. Handle missing optional fields with defaults
3. Skip extra bytes at end of known messages

## Error Handling

| Condition | Server Behavior | Client Behavior |
|-----------|-----------------|-----------------|
| Invalid loot_id in pickup | Ignore request | Ignore message |
| Elimination for unknown player | Skip broadcast | Ignore message |
| Zone update with invalid phase | Use last valid | Display last valid |

## Testing

### Protocol Tests

1. **Serialization Round-Trip**: All messages serialize/deserialize correctly
2. **Size Bounds**: Messages fit within expected byte sizes
3. **Field Encoding**: Specific values encode as expected (e.g., multiplier * 100)
4. **Unknown Message Handling**: Client ignores unknown type IDs
