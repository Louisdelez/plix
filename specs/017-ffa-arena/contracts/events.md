# Event Contracts: FFA Arena Mode

**Feature**: 017-ffa-arena | **Date**: 2025-12-16

## Overview

FFA mode reuses existing event infrastructure. No new event types are required. This document specifies how existing events are used in FFA context.

## Events Used

### KillEvent (Existing)

**Trigger**: Player eliminates another player
**Handler**: Kill processing in `plix-server/src/lib.rs`

```rust
/// Triggered when a player eliminates another player
struct KillEvent {
    /// Player who made the kill
    attacker_id: PlayerId,
    /// Player who was killed
    victim_id: PlayerId,
    /// Current server tick
    tick: Tick,
}
```

**FFA Behavior**:
- If `attacker_id == victim_id`: Suicide, no score change
- If `game_mode == Ffa`: Award +1 to attacker's individual score
- If `game_mode == Tdm`: Award +1 to attacker's team score

### MatchEnd (Existing)

**Trigger**: Player reaches score limit OR time expires
**Broadcast**: To all connected clients

```rust
/// Broadcast when match ends
struct MatchEnd {
    /// Winning player (FFA) or None (tie/time limit)
    winner: Option<PlayerId>,
    /// Winning team (TDM only)
    team_winner: Option<TeamId>,
    /// End reason
    reason: MatchEndReason,
    /// Final player scores
    final_scores: Vec<PlayerScore>,
}
```

**FFA Behavior**:
- `winner`: Set to PlayerId who reached score_limit
- `team_winner`: Always `None` for FFA
- `reason`: `ScoreLimit` or `TimeLimit`

### RespawnEvent (Existing)

**Trigger**: Respawn timer expires for dead player
**Handler**: Respawn processing in server tick

```rust
/// Triggered when player respawns
struct RespawnEvent {
    player_id: PlayerId,
    spawn_position: Vec3,
    spawn_rotation: f32,
    tick: Tick,
}
```

**FFA Behavior**:
- Spawn position selected from any spawn point (team ignored)
- Uses round-robin or random selection among all spawns
- Player health reset to 100

### PhaseTransition (Existing - Logging Only)

**Trigger**: Match phase changes
**Handler**: Logged via tracing

```rust
/// Logged when match phase changes
// tracing::info! format
"Match phase transition: {:?} -> {:?}", old_phase, new_phase
```

**FFA Behavior**: Identical to TDM. All phases apply.

## WorldSnapshot Integration

The `MatchState` within `WorldSnapshot` carries FFA state to clients:

```rust
/// Part of WorldSnapshot broadcast every tick
struct MatchState {
    phase: MatchPhase,
    countdown_remaining: u8,
    time_remaining: u32,
    score_limit: u16,
    player_scores: Vec<PlayerScore>,  // Individual scores (FFA uses this)
    winner: Option<PlayerId>,          // FFA winner
    arena_name: String,
    scores: Vec<TeamScore>,            // Team scores (empty/unused in FFA)
    team_winner: Option<TeamId>,       // Always None in FFA
    game_mode: GameMode,               // NEW: "ffa" or "tdm"
}
```

## Event Sequence: FFA Kill

```
Time    Event                           Handler
────────────────────────────────────────────────────────────────
t       Combat hit detected             physics system
t       Damage applied (health=0)       damage system
t       KillEvent emitted               server
t       [FFA] update_player_score(+1)   match_state
t       check_score_limit()             match_state
t       [if limit] MatchEnd broadcast   networking
t+1     WorldSnapshot (updated scores)  networking
```

## Event Sequence: FFA Respawn

```
Time            Event                   Handler
────────────────────────────────────────────────────────────────
t               Player killed           damage system
t               Dead state set          player state
t               respawn_tick set        player state
t               spectate_target set     player state
...
t+180           respawn_tick reached    server tick
t+180           RespawnEvent emitted    respawn system
t+180           Select spawn point      spawn manager
t+180           Reset health to 100     player state
t+180           Clear dead state        player state
t+181           WorldSnapshot           networking
```

## Event Sequence: FFA Match End

```
Time    Event                           Handler
────────────────────────────────────────────────────────────────
t       Kill brings player to limit     kill processing
t       check_score_limit() returns true match_state
t       end_match_score_limit()         match_state
t       MatchEnd broadcast              networking
t       phase = EndScreen               match_state
...
t+600   end_screen_ticks expires        server tick
t+600   phase = Resetting               match_state
t+600   complete_reset()                match_state
t+600   phase = Lobby                   match_state
t+600   Scores cleared                  match_state
```

## Contract Guarantees

### Timing Guarantees

| Event | Guarantee | Spec Reference |
|-------|-----------|----------------|
| Score update visible | Within 1 tick (16ms) | SC-002 |
| Respawn after delay | Within 1 tick of expiry | SC-003 |
| Match end on limit | Within 1 tick of reaching | SC-004 |

### Ordering Guarantees

1. Kill scoring happens before score limit check
2. Score limit check happens before match end transition
3. Match end broadcast happens before next tick starts
4. Respawn happens exactly at respawn_tick, not before

### State Guarantees

1. No scoring outside Playing phase (FR-014)
2. Winner is always set when match ends by score limit (FR-011)
3. Winner may be None if match ends by time limit with tie
4. All player scores reset to 0 on match reset (FR-019)
