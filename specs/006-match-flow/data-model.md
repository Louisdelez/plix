# Data Model: Match Flow

**Feature**: 006-match-flow | **Date**: 2025-12-15

## Protocol Types

### MatchPhase (Updated)

```rust
/// Match phase - server-authoritative state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchPhase {
    /// Waiting for players, movement allowed, no combat
    Lobby,
    /// Countdown before match start (3 seconds default)
    Countdown,
    /// Match in progress - full gameplay
    Playing,
    /// Match ended - showing final scores
    EndScreen,
    /// Resetting world for next match
    Resetting,
}
```

**Changes from current**:
- Renamed `WaitingForPlayers` → `Lobby`
- Removed `RoundEnd` (merged into `EndScreen`)
- Removed `MatchEnd` (use `EndScreen` instead)
- Added `Resetting`

### PlayerScore (New)

```rust
/// Per-player score for match results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerScore {
    /// Player ID
    pub player_id: PlayerId,
    /// Player display name
    pub name: String,
    /// Kill count (= score)
    pub kills: u16,
    /// Death count
    pub deaths: u16,
}
```

### MatchState (Updated)

```rust
/// Match state broadcast to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchState {
    /// Current phase
    pub phase: MatchPhase,
    /// Countdown remaining (seconds, only valid in Countdown phase)
    pub countdown_remaining: u8,
    /// Match time remaining (seconds, only valid in Playing phase)
    pub time_remaining: u32,
    /// Score limit to win
    pub score_limit: u16,
    /// Per-player scores
    pub player_scores: Vec<PlayerScore>,
    /// Winner player ID (only set in EndScreen phase, None for tie)
    pub winner: Option<PlayerId>,
    /// Current arena name
    pub arena_name: String,
}
```

### MatchConfig (Updated)

```rust
/// Server-side match configuration
#[derive(Debug, Clone)]
pub struct MatchConfig {
    /// Minimum players to start countdown
    pub min_players: u8,
    /// Countdown duration in ticks (default: 180 = 3s at 60Hz)
    pub countdown_ticks: u32,
    /// Match time limit in seconds (default: 300 = 5 minutes)
    pub time_limit_seconds: u32,
    /// Score limit to win (default: 5 kills)
    pub score_limit: u16,
    /// End screen duration in ticks (default: 300 = 5s at 60Hz)
    pub end_screen_ticks: u32,
    /// Respawn delay in ticks (default: 180 = 3s at 60Hz)
    pub respawn_delay_ticks: u32,
    /// Arena rotation list (empty = replay same arena)
    pub arena_rotation: Vec<String>,
}
```

## Client Messages

### ReadyToggle (New)

```rust
/// Client → Server: Toggle ready state
ClientMessage::ReadyToggle
```

No payload needed - server tracks current state and toggles it.

**Handling rules**:
- Valid only in `Lobby` phase
- Ignored in other phases (no error response)
- Server broadcasts updated `MatchState` after toggle

## Game Events

### MatchPhaseChanged (New)

```rust
/// Phase transition notification
GameEvent::MatchPhaseChanged {
    /// Previous phase
    from: MatchPhase,
    /// New phase
    to: MatchPhase,
}
```

### CountdownTick (New)

```rust
/// Countdown tick (broadcast each second)
GameEvent::CountdownTick {
    /// Seconds remaining (3, 2, 1, 0)
    remaining: u8,
}
```

### ScoreUpdate (New)

```rust
/// Player score changed
GameEvent::ScoreUpdate {
    /// Player whose score changed
    player_id: PlayerId,
    /// New kill count
    kills: u16,
    /// New death count
    deaths: u16,
}
```

### MatchEnded (Updated)

```rust
/// Match ended with final results
GameEvent::MatchEnded {
    /// Winner (None for tie)
    winner: Option<PlayerId>,
    /// Final scores for all players
    scores: Vec<PlayerScore>,
    /// Reason match ended
    reason: MatchEndReason,
}

/// Why the match ended
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MatchEndReason {
    /// A player reached the score limit
    ScoreLimit,
    /// Time limit expired
    TimeLimit,
    /// All other players disconnected
    Forfeit,
}
```

### ArenaChanged (New)

```rust
/// Arena changed (during Resetting phase)
GameEvent::ArenaChanged {
    /// New arena name
    name: String,
}
```

## Server State

### ServerPlayer (Updated)

```rust
pub struct ServerPlayer {
    // ... existing fields ...

    /// Ready state for match start
    pub is_ready: bool,

    // Note: kills/deaths already exist, no changes needed
}
```

### MatchStateMachine (Updated)

```rust
pub struct MatchStateMachine {
    /// Current state (broadcast to clients)
    state: MatchState,
    /// Server-only configuration
    config: MatchConfig,
    /// Tick when current phase started
    phase_start_tick: Tick,
    /// Current arena index in rotation
    arena_index: usize,
}
```

## State Transitions

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  ┌───────┐    ready &&    ┌───────────┐    timer    ┌─────────┐│
│  │ Lobby │───min_players──▶│ Countdown │────────────▶│ Playing ││
│  └───────┘                └───────────┘              └─────────┘│
│      ▲                         │                          │     │
│      │                         │ disconnect/              │     │
│      │                         │ unready                  │     │
│      │                         ▼                          │     │
│      │                    ┌───────┐                       │     │
│      │                    │ Lobby │                       │     │
│      │                    └───────┘                       │     │
│      │                                                    │     │
│      │         ┌────────────┐    timer    ┌───────────┐  │     │
│      └─────────│ Resetting  │◀────────────│ EndScreen │◀─┘     │
│                └────────────┘             └───────────┘        │
│                                           score/time limit     │
└─────────────────────────────────────────────────────────────────┘
```

## Invariants

1. **Phase ownership**: Only server can change `MatchPhase`
2. **Ready state reset**: All `is_ready` flags cleared when entering `Lobby`
3. **Score reset**: All kills/deaths cleared when entering `Playing`
4. **Countdown cancel**: If any player disconnects or toggles unready during `Countdown`, return to `Lobby`
5. **Time tracking**: `time_remaining` only decrements during `Playing` phase
6. **Arena rotation**: Index wraps to 0 when reaching end of list
7. **Winner determination**:
   - Score limit: first player to reach it
   - Time limit: highest score wins
   - Tie: `winner = None` if scores equal

## Validation Rules

| Field | Rule |
|-------|------|
| `min_players` | 1-16, must be ≤ max_players |
| `countdown_ticks` | > 0 (minimum 1 tick) |
| `time_limit_seconds` | > 0 (minimum 1 second) |
| `score_limit` | > 0 (minimum 1 kill to win) |
| `end_screen_ticks` | > 0 (minimum 1 tick) |
| `respawn_delay_ticks` | ≥ 0 (0 = instant respawn) |
| `arena_rotation` | Each name must exist in arena registry |
