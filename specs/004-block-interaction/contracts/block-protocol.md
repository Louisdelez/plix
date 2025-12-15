# Block Edit Protocol Contract

**Feature**: 004-block-interaction
**Protocol Version**: 0 (increment to 1 when implemented)
**Date**: 2025-12-15

## Overview

This document defines the wire protocol for block edit operations between client and server. The protocol extends the existing plix network protocol with new message types.

## Message Types

### Client → Server

#### BlockEdit (ClientMessage variant)

Request to modify a block in the world.

**Message ID**: Add to `ClientMessage` enum

```rust
pub enum ClientMessage {
    // ... existing variants ...
    BlockEdit(BlockEditRequest),
}
```

**Payload Structure**:

| Field | Type | Size | Description |
|-------|------|------|-------------|
| kind | u8 | 1 | 0 = Place, 1 = Remove |
| target_pos.x | i32 | 4 | Target X coordinate |
| target_pos.y | i32 | 4 | Target Y coordinate |
| target_pos.z | i32 | 4 | Target Z coordinate |
| block_type | u8 | 1 | Block type (for Place; 0 for Remove) |

**Total Size**: 14 bytes (+ bincode overhead)

**Constraints**:
- Sent over reliable channel (must not be lost)
- Rate: Max 4/sec per client (server enforces)
- Player must be alive and in Playing phase

---

### Server → Client (Broadcast)

#### BlockEditApplied (GameEvent variant)

Confirms a block was successfully modified.

**Message ID**: Add to `GameEvent` enum

```rust
pub enum GameEvent {
    // ... existing variants ...
    BlockEditApplied(BlockEditApplied),
    BlockEditRejected(BlockEditRejected),
}
```

**Payload Structure**:

| Field | Type | Size | Description |
|-------|------|------|-------------|
| pos.x | i32 | 4 | Block X coordinate |
| pos.y | i32 | 4 | Block Y coordinate |
| pos.z | i32 | 4 | Block Z coordinate |
| new_block | u8 | 1 | New block type (0 = Air for remove) |
| tick | u32 | 4 | Server tick when applied |

**Total Size**: 17 bytes (+ bincode overhead)

**Delivery**:
- Broadcast to ALL connected clients
- Reliable channel (must not be lost)
- Order preserved within tick

---

### Server → Client (Unicast)

#### BlockEditRejected (GameEvent variant)

Informs requester that their edit was rejected.

**Payload Structure**:

| Field | Type | Size | Description |
|-------|------|------|-------------|
| reason | u8 | 1 | Rejection reason code |
| pos.x | i32 | 4 | Requested X coordinate |
| pos.y | i32 | 4 | Requested Y coordinate |
| pos.z | i32 | 4 | Requested Z coordinate |

**Total Size**: 13 bytes (+ bincode overhead)

**Reason Codes**:

| Code | Name | Description |
|------|------|-------------|
| 0 | OutOfBounds | Target position outside arena |
| 1 | OutOfRange | Target too far from player |
| 2 | CellNotEmpty | Cannot place - cell occupied |
| 3 | CellEmpty | Cannot remove - cell is air |
| 4 | PlayerCollision | Would trap a player |
| 5 | RateLimited | Edit cooldown not expired |
| 6 | PlayerDead | Player is dead |
| 7 | InvalidPhase | Not in Playing phase |

**Delivery**:
- Unicast to requesting client ONLY
- Reliable channel

---

## Protocol Flow

### Successful Edit

```
Client                          Server                          Other Clients
   │                               │                                  │
   │ BlockEdit(Place, pos, Stone) ─►│                                  │
   │                               │ [Validate: PASS]                 │
   │                               │ [Apply: arena[pos] = Stone]      │
   │                               │                                  │
   │◄── Event(BlockEditApplied) ───┼── Event(BlockEditApplied) ───────►│
   │    {pos, Stone, tick}         │    {pos, Stone, tick}            │
```

### Rejected Edit

```
Client                          Server
   │                               │
   │ BlockEdit(Place, pos, Stone) ─►│
   │                               │ [Validate: FAIL - OutOfRange]
   │                               │
   │◄── Event(BlockEditRejected) ──┤
   │    {OutOfRange, pos}          │
```

### Late Join (World Sync)

```
New Client                      Server
    │                             │
    │──── Connect ───────────────►│
    │                             │ [Serialize current arena state]
    │◄─── Connected ──────────────┤
    │     {arena_data: [current]} │
    │                             │
    │ [Deserialize arena]         │
    │ [Build mesh]                │
```

**Note**: `arena_data` in `Connected` message contains the CURRENT world state including all prior edits. No separate edit replay needed.

---

## Validation Rules (Server-Side)

All validation is performed server-side. Client may mirror for UX but server is authoritative.

### Pre-Check (before any edit)

1. **Player State**:
   - `player.is_dead == false` → else `PlayerDead`
   - `match_state.phase == Playing` → else `InvalidPhase`
   - `current_tick - player.last_edit_tick >= 15` → else `RateLimited`

2. **Position Check**:
   - `0 <= pos.x < arena.size.x` (and y, z) → else `OutOfBounds`
   - `distance(pos.center(), player.pos) <= 5.0` → else `OutOfRange`

### Kind-Specific

3. **For Remove**:
   - `arena.get_block(pos) != Air` → else `CellEmpty`

4. **For Place**:
   - `arena.get_block(pos) == Air` → else `CellNotEmpty`
   - `!any_player_intersects(pos)` → else `PlayerCollision`

---

## Bincode Serialization

All messages use bincode with default configuration:
- Little-endian byte order
- Variable-length integer encoding
- Struct fields serialized in declaration order

**Example** (BlockEditRequest):
```
kind=Place(0), pos=(10,5,20), block_type=Stone(1)

Bytes: 00 0A 00 00 00 05 00 00 00 14 00 00 00 01
       ── ─────────── ─────────── ─────────── ──
       │      x           y           z      type
       kind
```

---

## Error Handling

### Client Behavior on Rejection

1. Display brief debug message: "Edit rejected: {reason}"
2. Do NOT retry automatically (would hit rate limit)
3. Allow player to retry manually after cooldown

### Server Behavior on Invalid Message

1. Malformed message: Log warning, ignore
2. Unknown message type: Log warning, ignore
3. Do NOT disconnect client for invalid edits (may be race condition)

---

## Compatibility Notes

### Protocol Version

- Current version: 0
- After implementation: Increment to 1
- Clients/servers must match version (existing check)

### Backward Compatibility

- Older clients connecting to new server: Will receive unknown `GameEvent` variants → should ignore gracefully
- New clients connecting to older server: Will send `BlockEdit` that server doesn't recognize → server ignores

**Recommendation**: Ensure both client and server are updated together for this feature.

---

## Testing Contracts

### Unit Test Cases

1. **Serialization round-trip**: Encode/decode all message types
2. **Size constraints**: Verify messages fit in MTU (< 1389 bytes)
3. **Reason code coverage**: All rejection reasons reachable

### Integration Test Cases

1. **Happy path**: Client sends Place, receives BlockEditApplied, sees block
2. **Rejection path**: Client sends invalid edit, receives BlockEditRejected
3. **Broadcast**: Two clients, one edits, both receive BlockEditApplied
4. **Late join**: Client joins after edits, sees correct world
