# Protocol Contracts: Training Mode

**Feature**: 020-training-mode | **Date**: 2025-12-17

## Overview

Training mode extends the existing plix protocol with new message types for:
1. Session reset (client → server → broadcast)
2. Stats request/response (client ↔ server)
3. Bot state replication (server → client via snapshots)

All messages use bincode serialization (existing pattern).

---

## New Client Messages

### TrainingReset

**Direction**: Client → Server
**Trigger**: Player presses reset keybinding

```rust
// Add to ClientMessage enum in crates/plix-common/src/protocol/messages.rs

/// Request to reset training session (training mode only)
ClientMessage::TrainingReset
```

**Validation**:
- Server MUST verify `game_mode == Training`
- Server MUST verify player exists in session
- Server SHOULD rate-limit (once per second)

**Server Response**: Processes reset, broadcasts `GameEvent::TrainingReset`

---

### TrainingStatsRequest

**Direction**: Client → Server
**Trigger**: Player presses stats keybinding

```rust
/// Request current training statistics (training mode only)
ClientMessage::TrainingStatsRequest
```

**Validation**:
- Server MUST verify `game_mode == Training`
- Server MUST verify player exists in session

**Server Response**:
1. Logs stats to server console
2. Optionally sends `ServerMessage::TrainingStats` to requester

---

## New Server Messages

### TrainingStats

**Direction**: Server → Client (unicast to requester)
**Trigger**: Response to `TrainingStatsRequest`

```rust
// Add to ServerMessage enum

/// Training statistics response
ServerMessage::TrainingStats {
    hits: u32,
    kills: u32,
    attacks: u32,
    accuracy_pct: f32,      // 0.0 - 100.0
    session_duration_secs: f32,
}
```

---

## New Game Events

### TrainingReset

**Direction**: Server → All Clients (broadcast)
**Trigger**: After processing `ClientMessage::TrainingReset`

```rust
// Add to GameEvent enum

/// Training session was reset
GameEvent::TrainingReset {
    /// Player who triggered the reset
    player_id: PlayerId,
}
```

**Client Handling**:
- Clear local prediction state
- Reset any client-side stats display
- Visual/audio feedback for reset

---

### BotHit

**Direction**: Server → Client (unicast to attacker)
**Trigger**: When attack hits a bot

```rust
/// Bot was hit by player attack
GameEvent::BotHit {
    bot_id: BotId,
    damage: u8,
    killed: bool,
}
```

**Client Handling**:
- Play hit marker effect
- Update hit counter (if displaying)
- If killed: play kill effect

---

### BotRespawned

**Direction**: Server → All Clients (broadcast)
**Trigger**: When a bot respawns after death

```rust
/// Bot respawned after elimination
GameEvent::BotRespawned {
    bot_id: BotId,
    position: Vec3,
}
```

**Client Handling**:
- Spawn bot entity at position
- Play respawn visual effect

---

## Extended WorldSnapshot

Bot state is included in world snapshots when in training mode.

```rust
// Extend WorldSnapshot in crates/plix-common/src/protocol/messages.rs

pub struct WorldSnapshot {
    pub tick: Tick,
    pub last_input_seq: InputSeq,
    pub players: Vec<PlayerSnapshot>,
    pub match_state: MatchState,
    pub rtt_nonce_echo: u64,

    // NEW: Bot snapshots (only present in training mode, empty otherwise)
    #[serde(default)]
    pub bots: Vec<BotSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotSnapshot {
    pub id: BotId,
    pub position: Vec3,
    pub health: u8,
    pub is_dead: bool,
}
```

**Bandwidth Consideration**:
- Per bot: ~13 bytes (1 id + 12 position + 1 health + 1 dead)
- 20 bots: ~260 bytes additional per snapshot
- At 60Hz: ~15.6 KB/s additional (acceptable for training mode)

---

## Message Flow Diagrams

### Session Reset Flow

```text
Client                          Server                         All Clients
   │                               │                                │
   │ TrainingReset                 │                                │
   │──────────────────────────────►│                                │
   │                               │                                │
   │                               │ validate(game_mode, player)    │
   │                               │ coordinator.reset()            │
   │                               │ player.spawn(spawn_point)      │
   │                               │                                │
   │                               │ GameEvent::TrainingReset       │
   │◄──────────────────────────────│───────────────────────────────►│
   │                               │                                │
```

### Stats Request Flow

```text
Client                          Server
   │                               │
   │ TrainingStatsRequest          │
   │──────────────────────────────►│
   │                               │
   │                               │ validate(game_mode, player)
   │                               │ stats = coordinator.stats
   │                               │ info!("Training stats: ...")
   │                               │
   │ ServerMessage::TrainingStats  │
   │◄──────────────────────────────│
   │                               │
```

### Combat Flow (Bot Hit)

```text
Client                          Server                         Other Clients
   │                               │                                │
   │ Input(attack=true)            │                                │
   │──────────────────────────────►│                                │
   │                               │                                │
   │                               │ combat.try_attack()            │
   │                               │ (targets include bots)         │
   │                               │                                │
   │                               │ if hit_bot:                    │
   │                               │   coordinator.process_hit()    │
   │                               │   stats.record_hit()           │
   │                               │                                │
   │ GameEvent::BotHit             │                                │
   │◄──────────────────────────────│                                │
   │                               │                                │
   │                               │ if killed:                     │
   │                               │   stats.record_kill()          │
   │                               │   (bot respawns later)         │
   │                               │                                │
```

---

## Existing Protocol Extensions

### GameMode Enum (Extend)

```rust
// crates/plix-common/src/types.rs

pub enum GameMode {
    Tdm,
    Ffa,
    Ctf,
    BrLite,
    Training,  // NEW - must serialize as "training"
}
```

**Serialization Test**:
```rust
#[test]
fn test_game_mode_training_serde() {
    let mode = GameMode::Training;
    let json = serde_json::to_string(&mode).unwrap();
    assert_eq!(json, "\"training\"");

    let parsed: GameMode = serde_json::from_str("\"training\"").unwrap();
    assert_eq!(parsed, GameMode::Training);
}
```

---

## BotId Type

New identifier type for bots (not using PlayerId to avoid confusion).

```rust
// crates/plix-common/src/types.rs (or training module)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BotId(pub u8);

impl BotId {
    pub const NONE: Self = Self(0xFF);
}
```

---

## Wire Format Summary

| Message | Size (bytes) | Frequency |
|---------|--------------|-----------|
| ClientMessage::TrainingReset | ~1 | On keypress |
| ClientMessage::TrainingStatsRequest | ~1 | On keypress |
| ServerMessage::TrainingStats | ~21 | On request |
| GameEvent::TrainingReset | ~3 | On reset |
| GameEvent::BotHit | ~4 | Per hit |
| GameEvent::BotRespawned | ~14 | Per respawn |
| BotSnapshot (in WorldSnapshot) | ~17 per bot | 60Hz |

---

## Error Handling

| Scenario | Response |
|----------|----------|
| TrainingReset when not training mode | Ignore silently (no error message) |
| TrainingStatsRequest when not training mode | Ignore silently |
| TrainingReset rate limited | Ignore (no error to client) |
| Invalid BotId in hit detection | Skip (defensive, should not happen) |
