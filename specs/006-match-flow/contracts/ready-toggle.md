# Contract: ReadyToggle

**Feature**: 006-match-flow | **Date**: 2025-12-15

## Overview

Client message to toggle ready state for match start.

## Message Definition

```rust
ClientMessage::ReadyToggle
```

No payload - server maintains state and toggles it.

## Preconditions

| Condition | Validation |
|-----------|------------|
| Player connected | Player must have valid session |
| Phase is Lobby | `match_state.phase == MatchPhase::Lobby` |

## Behavior

### Happy Path

1. Client sends `ReadyToggle`
2. Server toggles `player.is_ready = !player.is_ready`
3. Server broadcasts updated `MatchState` to all clients
4. If all players ready AND `player_count >= min_players`:
   - Server transitions to `Countdown` phase
   - Server broadcasts `MatchPhaseChanged { from: Lobby, to: Countdown }`

### Edge Cases

| Scenario | Server Behavior |
|----------|-----------------|
| Phase is not Lobby | Ignore message silently (no error) |
| Player disconnects while ready | Remove from ready count, recheck transition |
| Only player in Lobby toggles ready | Mark ready but don't start countdown (min_players not met) |
| Player spams toggle | Each toggle processes normally (rate limiting at network layer) |

## State Changes

| Field | Before | After |
|-------|--------|-------|
| `player.is_ready` | `X` | `!X` |
| `match_state.phase` | `Lobby` | `Lobby` or `Countdown` |

## Events Triggered

| Condition | Event |
|-----------|-------|
| Always | Updated `MatchState` in next snapshot |
| If transition | `MatchPhaseChanged { from: Lobby, to: Countdown }` |

## Test Cases

1. **Toggle on**: Player with `is_ready=false` toggles → `is_ready=true`
2. **Toggle off**: Player with `is_ready=true` toggles → `is_ready=false`
3. **Ignored in Playing**: Toggle during `Playing` phase → no state change
4. **Triggers countdown**: 2 players, both toggle ready → transition to `Countdown`
5. **Insufficient players**: 1 player toggles ready, `min_players=2` → stays in `Lobby`
