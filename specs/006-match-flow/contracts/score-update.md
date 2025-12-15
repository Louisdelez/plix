# Contract: ScoreUpdate Event

**Feature**: 006-match-flow | **Date**: 2025-12-15

## Overview

Server broadcasts `ScoreUpdate` event when a player's score changes (kill or death).

## Message Definition

```rust
GameEvent::ScoreUpdate {
    player_id: PlayerId,
    kills: u16,
    deaths: u16,
}
```

## Trigger Conditions

| Event | ScoreUpdate Sent |
|-------|------------------|
| Player kills another | Yes (attacker's kills +1) |
| Player dies | Yes (victim's deaths +1) |
| Round/match reset | No (scores are in MatchState) |

## Timing

- Sent immediately after kill is processed
- Same tick as `PlayerDied` event
- Order: `DamageTaken` → `PlayerDied` → `ScoreUpdate` (attacker) → `ScoreUpdate` (victim)

## Recipients

- **Broadcast**: All connected clients receive all score updates
- No per-player filtering (scoreboard is public information)

## Match End Check

After each `ScoreUpdate`:

```rust
if player.kills >= config.score_limit {
    trigger_match_end(ScoreLimit, Some(player_id));
}
```

## State Changes

| Field | Before | After |
|-------|--------|-------|
| `player.kills` | N | N+1 (for attacker) |
| `player.deaths` | M | M+1 (for victim) |
| `match_state.player_scores` | Old values | Updated values |

## Related Events

| Event | Relationship |
|-------|-------------|
| `PlayerDied` | Precedes ScoreUpdate, same tick |
| `MatchEnded` | May follow if score limit reached |
| `MatchState` | Contains cumulative scores |

## Test Cases

1. **Kill increments attacker**: Player A kills B → ScoreUpdate for A with kills+1
2. **Death increments victim**: Player A kills B → ScoreUpdate for B with deaths+1
3. **Triggers match end**: Player reaches score_limit → MatchEnded follows
4. **Self-kill**: Player dies to environment → ScoreUpdate for deaths only (no killer)
5. **Multiple kills same tick**: Multiple deaths → Multiple ScoreUpdates in order
