# Contract: CTF Protocol Messages

**Module**: `plix-common/src/protocol/messages.rs`
**Date**: 2025-12-16

## Purpose

Defines network message types for CTF state synchronization between server and clients.

## Interface

### Server → Client Messages

```rust
/// CTF flag state update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtfFlagUpdate {
    /// Team whose flag state changed
    pub team: TeamId,
    /// New flag state
    pub state: FlagState,
    /// Base position (for reference when returning)
    pub base_position: Vec3,
}

/// CTF capture event notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtfCaptureEvent {
    /// Team that captured the flag
    pub capturing_team: TeamId,
    /// Player who made the capture
    pub capturing_player: PlayerId,
    /// Current scores after capture [team0, team1]
    pub scores: [u32; 2],
}

/// CTF match info (included in MatchState)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtfMatchInfo {
    /// Current flag states
    pub flags: [CtfFlagUpdate; 2],
    /// Current capture scores
    pub scores: [u32; 2],
    /// Capture limit to win
    pub capture_limit: u16,
}
```

### Message Integration

```rust
/// Extended ServerMessage enum
pub enum ServerMessage {
    // ... existing variants ...

    /// CTF flag state changed
    CtfFlagUpdate(CtfFlagUpdate),

    /// CTF capture occurred
    CtfCaptureEvent(CtfCaptureEvent),
}

/// Extended MatchState (add field)
pub struct MatchState {
    // ... existing fields ...

    /// CTF-specific state (None if not CTF mode)
    pub ctf: Option<CtfMatchInfo>,
}
```

## Behavior Contracts

### BC-401: Flag Update Broadcast

**When**: Flag state changes (pickup, drop, return, capture)

**Postconditions**:
- `CtfFlagUpdate` sent to all connected clients
- Message contains new state and base position
- Sent immediately after state change (same tick)

### BC-402: Capture Event Broadcast

**When**: Successful flag capture

**Postconditions**:
- `CtfCaptureEvent` sent to all connected clients
- Contains capturing team, player, and updated scores
- Sent after both flags reset to base

### BC-403: Match State Inclusion

**When**: `MatchState` broadcast (every tick or on change)

**Postconditions**:
- If `game_mode == GameMode::Ctf`: `ctf` field is `Some(CtfMatchInfo)`
- If `game_mode != GameMode::Ctf`: `ctf` field is `None`

### BC-404: Join Sync

**When**: Player joins mid-match

**Postconditions**:
- Player receives current `MatchState` including `CtfMatchInfo`
- Player has accurate flag states and scores

## Serialization

All messages use bincode serialization (existing protocol).

Message size estimates:
- `CtfFlagUpdate`: ~32 bytes (team + state enum + Vec3)
- `CtfCaptureEvent`: ~20 bytes (team + player + scores)
- `CtfMatchInfo`: ~80 bytes (2 flags + scores + limit)

## Message Frequency

| Event | Frequency | Notes |
|-------|-----------|-------|
| FlagUpdate | On state change | ~1-5 per minute during active play |
| CaptureEvent | On capture | ~1 every 1-3 minutes |
| MatchState (with ctf) | Every tick / on change | Included in existing broadcast |

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Client receives CTF message in non-CTF mode | Ignore message, log warning |
| Malformed message | Drop message, connection stays open |
| Client behind on state | Next MatchState will resync |

## Test Scenarios

### T-PROTO-001: Flag Update Serializes Correctly
```
Given: CtfFlagUpdate with Carried state
When: serialize and deserialize
Then: all fields match original
```

### T-PROTO-002: Capture Event Serializes Correctly
```
Given: CtfCaptureEvent with scores [2, 1]
When: serialize and deserialize
Then: scores == [2, 1]
```

### T-PROTO-003: MatchState CTF Field Present
```
Given: MatchState in CTF game
When: broadcast
Then: ctf field is Some with current state
```

### T-PROTO-004: MatchState CTF Field Absent for TDM
```
Given: MatchState in TDM game
When: broadcast
Then: ctf field is None
```

### T-PROTO-005: Join Sync Includes CTF State
```
Given: CTF match in progress, flags in various states
When: new player joins
Then: receives MatchState with accurate CtfMatchInfo
```
