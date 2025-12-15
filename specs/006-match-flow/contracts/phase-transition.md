# Contract: Phase Transitions

**Feature**: 006-match-flow | **Date**: 2025-12-15

## Overview

Server-authoritative phase transitions with defined entry/exit conditions.

## Phase Transition Event

```rust
GameEvent::MatchPhaseChanged {
    from: MatchPhase,
    to: MatchPhase,
}
```

Broadcast to all clients on every phase change.

## Transition Table

| From | To | Trigger | Actions |
|------|-----|---------|---------|
| Lobby | Countdown | All ready AND min_players met | Clear all `is_ready`, record phase_start_tick |
| Countdown | Lobby | Player disconnects OR unreadies | Cancel countdown |
| Countdown | Playing | countdown_ticks elapsed | Spawn all players, reset scores, start match timer |
| Playing | EndScreen | Score limit OR time limit OR all disconnect | Calculate winner, freeze gameplay |
| EndScreen | Resetting | end_screen_ticks elapsed | Begin world reset |
| Resetting | Lobby | Reset complete | Load next arena (if rotation), clear all state |

## Lobby → Countdown

**Trigger**:
```rust
let all_ready = players.iter().all(|p| p.is_ready);
let enough_players = players.len() >= config.min_players as usize;
if all_ready && enough_players {
    transition_to(Countdown);
}
```

**Actions**:
1. Record `phase_start_tick = current_tick`
2. Broadcast `MatchPhaseChanged { from: Lobby, to: Countdown }`
3. Broadcast `CountdownTick { remaining: 3 }` (or configured value)

## Countdown → Lobby (Cancel)

**Trigger**:
```rust
// On player disconnect during countdown
if players.len() < config.min_players as usize {
    transition_to(Lobby);
}

// On ready toggle during countdown
if any_player_toggled_unready {
    transition_to(Lobby);
}
```

**Actions**:
1. Broadcast `MatchPhaseChanged { from: Countdown, to: Lobby }`
2. Clear all `is_ready` flags
3. No penalty for unready player

## Countdown → Playing

**Trigger**:
```rust
let elapsed = current_tick - phase_start_tick;
if elapsed >= config.countdown_ticks {
    transition_to(Playing);
}
```

**Actions**:
1. Reset all player kills/deaths to 0
2. Spawn all players at arena spawn points
3. Set `time_remaining = config.time_limit_seconds`
4. Record `phase_start_tick = current_tick`
5. Broadcast `MatchPhaseChanged { from: Countdown, to: Playing }`

## Playing → EndScreen

**Triggers** (any):
```rust
// Score limit
if let Some(player) = players.iter().find(|p| p.kills >= config.score_limit) {
    end_match(ScoreLimit, Some(player.id));
}

// Time limit
let elapsed_secs = (current_tick - phase_start_tick) / 60;
if elapsed_secs >= config.time_limit_seconds {
    let winner = determine_winner_by_score();
    end_match(TimeLimit, winner);
}

// All players disconnected
if players.is_empty() {
    end_match(Forfeit, None);
}
```

**Winner determination**:
```rust
fn determine_winner_by_score() -> Option<PlayerId> {
    let max_kills = players.iter().map(|p| p.kills).max()?;
    let leaders: Vec<_> = players.iter().filter(|p| p.kills == max_kills).collect();
    if leaders.len() == 1 {
        Some(leaders[0].id)
    } else {
        None // Tie
    }
}
```

**Actions**:
1. Set `match_state.winner` (or None for tie)
2. Disable all gameplay inputs
3. Record `phase_start_tick = current_tick`
4. Broadcast `MatchPhaseChanged { from: Playing, to: EndScreen }`
5. Broadcast `MatchEnded { winner, scores, reason }`

## EndScreen → Resetting

**Trigger**:
```rust
let elapsed = current_tick - phase_start_tick;
if elapsed >= config.end_screen_ticks {
    transition_to(Resetting);
}
```

**Actions**:
1. Broadcast `MatchPhaseChanged { from: EndScreen, to: Resetting }`
2. Begin world reset (async operation)

## Resetting → Lobby

**Trigger**:
```rust
if world_reset_complete {
    transition_to(Lobby);
}
```

**Actions**:
1. Clear all player `is_ready` flags
2. Clear all player kills/deaths
3. Advance arena rotation (if configured):
   ```rust
   if !config.arena_rotation.is_empty() {
       arena_index = (arena_index + 1) % config.arena_rotation.len();
       load_arena(&config.arena_rotation[arena_index]);
       broadcast(ArenaChanged { name });
   }
   ```
4. Broadcast `MatchPhaseChanged { from: Resetting, to: Lobby }`

## Invalid Transitions

All other transitions are invalid and must not occur:
- Lobby → Playing (must go through Countdown)
- Playing → Lobby (must go through EndScreen → Resetting)
- EndScreen → Lobby (must go through Resetting)
- etc.

## Test Cases

1. **Normal flow**: Lobby → Countdown → Playing → EndScreen → Resetting → Lobby
2. **Countdown cancel**: Countdown → Lobby (player disconnects)
3. **Score limit end**: Playing → EndScreen (player reaches limit)
4. **Time limit end**: Playing → EndScreen (time expires)
5. **Tie game**: Time expires with equal scores → winner=None
6. **Arena rotation**: Resetting with rotation → next arena loaded
7. **Single arena**: Resetting without rotation → same arena reloaded
