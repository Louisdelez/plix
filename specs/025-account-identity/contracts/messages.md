# Protocol Contracts: Account Identity

**Feature**: 025-account-identity
**Date**: 2025-12-17

## Message Extensions

### Client → Server

#### 1. Connect (Extended)

Existing message with new optional fields:

```rust
ClientMessage::Connect {
    /// Protocol version
    protocol_version: u8,
    /// Preferred display name (validated/sanitized by server)
    name: String,
    /// Account ID (v2 placeholder - always None in v1)
    #[serde(default)]
    account_id: Option<u64>,
    /// Auth token (v2 placeholder - always None in v1)
    #[serde(default)]
    auth_token: Option<String>,
}
```

**Backward Compatibility**: `account_id` and `auth_token` use `#[serde(default)]`, so existing clients without these fields work unchanged.

---

#### 2. RenameRequest (New)

Request to change display name during session.

```rust
ClientMessage::RenameRequest {
    /// New preferred display name
    new_name: String,
}
```

**Constraints**:
- Rate limited to 1 request per 60 seconds
- Server validates and disambiguates

---

### Server → Client

#### 1. Connected (Extended)

Existing message with new field:

```rust
ServerMessage::Connected {
    /// Assigned player ID
    player_id: PlayerId,
    /// Current server tick
    tick: Tick,
    /// Server tick rate
    tick_rate: u8,
    /// Compressed arena data
    arena_data: Vec<u8>,
    /// Assigned display name (may differ from requested if disambiguated)
    #[serde(default)]
    display_name: String,
    /// Assigned session ID (for client logging/debugging)
    #[serde(default)]
    session_id: u64,
}
```

---

#### 2. RenameResult (New via GameEvent)

Result of a rename request.

```rust
GameEvent::RenameResult {
    /// Whether rename succeeded
    success: bool,
    /// New display name (if successful)
    new_name: Option<String>,
    /// Failure reason (if unsuccessful)
    reason: Option<RenameRejectReason>,
}

/// Reason for rejecting a rename request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenameRejectReason {
    /// Still on cooldown from last rename
    RateLimited,
    /// Name validation failed (empty, too long, invalid chars)
    InvalidName,
    /// All suffix variants taken for this name
    NameUnavailable,
}
```

---

#### 3. PlayerRenamed (New via GameEvent)

Broadcast when any player's name changes.

```rust
GameEvent::PlayerRenamed {
    /// Player whose name changed
    player_id: PlayerId,
    /// Old display name
    old_name: String,
    /// New display name
    new_name: String,
}
```

---

#### 4. PlayerSnapshot (Extended)

Existing struct with new field:

```rust
pub struct PlayerSnapshot {
    /// Player ID
    pub id: PlayerId,
    /// World position
    pub position: Vec3,
    /// Look rotation
    pub rotation: Rotation,
    /// Health (0-100)
    pub health: u8,
    /// Is player dead
    pub is_dead: bool,
    /// Current animation state
    pub animation: AnimationState,
    /// Spectate target
    #[serde(default)]
    pub spectate_target: Option<PlayerId>,
    /// Display name (new field)
    #[serde(default)]
    pub display_name: String,
    /// Team ID
    #[serde(default)]
    pub team: TeamId,
}
```

---

#### 5. PlayerJoined (Extended)

Existing event, already has name field:

```rust
GameEvent::PlayerJoined {
    id: PlayerId,
    name: String,  // Now contains validated/disambiguated display name
    team: TeamId,
}
```

---

## Logging Contracts

All player-related log events MUST include SessionId when available:

```rust
// Connection
tracing::info!(
    player_id = %player.id,
    session_id = %session_id,
    display_name = %display_name,
    addr = %addr,
    "Player connected"
);

// Rename
tracing::info!(
    player_id = %player.id,
    session_id = %session_id,
    old_name = %old_name,
    new_name = %new_name,
    "Player renamed"
);

// Disconnect
tracing::info!(
    player_id = %player.id,
    session_id = %session_id,
    display_name = %display_name,
    duration_secs = %duration,
    "Player disconnected"
);
```

---

## Console Commands

### /name <new_name>

**Client-side parsing** (in `console.rs`):

```rust
CommandResult::SendMessage(ClientMessage::RenameRequest {
    new_name: args.to_string(),
})
```

**Help text update**:
```
/name <new_name> - Change your display name (60s cooldown)
```

---

## Wire Format

All messages use bincode serialization with serde, consistent with existing protocol.

### Bandwidth Impact

| Change | Additional Bytes | Frequency |
|--------|-----------------|-----------|
| `display_name` in PlayerSnapshot | ~16 avg | Every snapshot (20-60 Hz) |
| `session_id` in Connected | 8 | Once per connection |
| RenameRequest | ~20 avg | Rare (rate limited) |
| RenameResult | ~30 avg | Rare (rate limited) |
| PlayerRenamed | ~50 avg | Rare (rate limited) |

**Total Impact**: ~16 bytes per snapshot per player. For 10 players at 60 Hz = ~9.6 KB/s additional. Acceptable for LAN/internet play.

---

## Compatibility Matrix

| Client Version | Server Version | Behavior |
|----------------|----------------|----------|
| v1 (no identity) | v1 (no identity) | Current behavior |
| v1 (no identity) | v2 (identity) | Works - server assigns "Player", no display_name in snapshot |
| v2 (identity) | v1 (no identity) | Works - display_name field ignored, uses existing name field |
| v2 (identity) | v2 (identity) | Full identity support |
