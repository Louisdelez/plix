# Contract: MatchState

**Feature**: 006-match-flow | **Date**: 2025-12-15

## Overview

Server broadcasts `MatchState` in every `WorldSnapshot` to inform clients of current match status.

## Message Definition

```rust
/// Included in WorldSnapshot
pub struct MatchState {
    pub phase: MatchPhase,
    pub countdown_remaining: u8,
    pub time_remaining: u32,
    pub score_limit: u16,
    pub player_scores: Vec<PlayerScore>,
    pub winner: Option<PlayerId>,
    pub arena_name: String,
}
```

## Field Semantics by Phase

| Phase | countdown_remaining | time_remaining | player_scores | winner |
|-------|---------------------|----------------|---------------|--------|
| Lobby | 0 | `time_limit_seconds` | All zeros | None |
| Countdown | Seconds left (3,2,1) | `time_limit_seconds` | All zeros | None |
| Playing | 0 | Decreasing | Live scores | None |
| EndScreen | 0 | 0 | Final scores | Winner or None (tie) |
| Resetting | 0 | 0 | Cleared | None |

## Broadcast Frequency

- Sent with every `WorldSnapshot` (60 Hz default)
- Fields update immediately when state changes
- No batching or throttling of state updates

## Client Rendering

| Phase | Client Display |
|-------|----------------|
| Lobby | Ready button, player list with ready indicators |
| Countdown | Full-screen countdown (3... 2... 1...) |
| Playing | Match timer, scoreboard |
| EndScreen | Final scores, winner announcement |
| Resetting | "Loading next match..." |

## Invariants

1. `player_scores` contains entry for every connected player
2. `countdown_remaining` only > 0 during `Countdown` phase
3. `time_remaining` only decrements during `Playing` phase
4. `winner` only set during `EndScreen` phase
5. `arena_name` matches currently loaded arena

## Test Cases

1. **Initial state**: Fresh server → `phase=Lobby`, all fields at defaults
2. **Countdown values**: During countdown → `countdown_remaining` decrements each second
3. **Score sync**: Kill event → `player_scores` updated in same tick
4. **Winner set**: Match ends → `winner` populated (or None for tie)
5. **Reset clears**: Enter `Resetting` → scores cleared, countdown/time reset
