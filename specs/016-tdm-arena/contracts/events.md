# Event Contracts: TDM Arena Mode

**Feature**: 016-tdm-arena
**Date**: 2025-12-16
**Purpose**: Define internal and network events for TDM mode

## Overview

TDM mode primarily uses existing events with minor extensions. This document defines event contracts for team scoring, spectate mode, and match flow.

## Network Events (Server → Client)

### Existing Events (No Changes)

These events work unchanged for TDM:

| Event | Trigger | Data |
|-------|---------|------|
| `PlayerJoined` | Player connects | id, name, team |
| `PlayerLeft` | Player disconnects | id |
| `PlayerDied` | Player killed | victim, killer |
| `PlayerRespawned` | Respawn complete | id |
| `MatchPhaseChanged` | State transition | from, to |
| `CountdownTick` | Each countdown second | remaining |
| `ScoreUpdate` | Player K/D changed | player_id, kills, deaths |

### Extended Events

#### WorldSnapshot (Modified)

The `WorldSnapshot.match_state` already contains `scores: Vec<TeamScore>`. TDM uses this actively.

```rust
pub struct WorldSnapshot {
    pub tick: Tick,
    pub last_input_seq: InputSeq,
    pub players: Vec<PlayerSnapshot>,
    pub match_state: MatchState,  // Contains team scores
    pub rtt_nonce_echo: u64,
}

pub struct MatchState {
    pub phase: MatchPhase,
    pub countdown_remaining: u8,
    pub time_remaining: u32,
    pub score_limit: u16,
    pub player_scores: Vec<PlayerScore>,
    pub winner: Option<PlayerId>,      // Legacy individual
    pub arena_name: String,
    pub scores: Vec<TeamScore>,        // Team scores - USED FOR TDM
    // NEW
    pub team_winner: Option<TeamId>,   // Winning team for TDM
}
```

**Contract**:
- `scores[0]` = Red team (TeamId::TEAM_0) score
- `scores[1]` = Blue team (TeamId::TEAM_1) score
- `team_winner` = None during Lobby/Countdown/Playing, Some(TeamId) in EndScreen/Resetting

#### PlayerSnapshot (Extended)

Add spectate target for death camera.

```rust
pub struct PlayerSnapshot {
    pub id: PlayerId,
    pub position: Vec3,
    pub rotation: Rotation,
    pub health: u8,
    pub is_dead: bool,
    pub animation: AnimationState,
    // NEW for TDM spectate
    pub spectate_target: Option<PlayerId>,  // Who this player spectates when dead
}
```

**Contract**:
- `spectate_target` = None when alive
- `spectate_target` = Some(killer_id) when dead
- `spectate_target` = None if killer disconnected (client shows black screen)

### Optional New Event (Deferred)

If immediate score feedback is needed (beyond 60Hz snapshots):

```rust
pub enum GameEvent {
    // ... existing ...

    /// Team score changed (optional - snapshots already include this)
    TeamScoreUpdate {
        team: TeamId,
        new_score: u32,
        scorer_id: PlayerId,  // Who got the kill
    },
}
```

**Decision**: Deferred. Snapshot-based updates are sufficient at 60Hz.

## Internal Events (Server-Side Only)

These are conceptual events processed within the server, not sent over network.

### KillProcessed

Triggered when combat system confirms a kill.

```text
KillProcessed {
    killer: PlayerId,
    killer_team: TeamId,
    victim: PlayerId,
    victim_team: TeamId,
    tick: Tick,
}
```

**Handler Contract**:
1. IF killer_team != victim_team AND phase == Playing:
   - Increment `match_state.scores[killer_team]`
   - Check team score limit
2. Set `victim.spectate_target = killer`
3. Set `victim.respawn_tick = tick + respawn_delay`
4. Broadcast `PlayerDied` event
5. Update individual scores for both players

### TeamScoreLimitReached

Triggered when a team reaches score_limit.

```text
TeamScoreLimitReached {
    winning_team: TeamId,
    final_scores: [u32; 2],
    tick: Tick,
}
```

**Handler Contract**:
1. Set `match_state.team_winner = winning_team`
2. Transition phase to EndScreen
3. Broadcast `MatchPhaseChanged { Playing → EndScreen }`
4. Broadcast `MatchEnd { winner: None, team_winner: winning_team, ... }`

### PlayerRespawnDue

Triggered when `current_tick >= player.respawn_tick`.

```text
PlayerRespawnDue {
    player_id: PlayerId,
    team: TeamId,
}
```

**Handler Contract**:
1. Get team spawn point from SpawnManager
2. Set `player.position = spawn.position`
3. Set `player.health = 100`
4. Set `player.is_dead = false`
5. Set `player.spectate_target = None`
6. Set `player.respawn_tick = None`
7. Apply invulnerability if configured
8. Broadcast `PlayerRespawned { id }`

## Event Sequencing

### Kill → Team Score → Match End Sequence

```text
Time    Server                              Client
─────────────────────────────────────────────────────────────
T       Combat detects kill
T       KillProcessed handler:
        - scores[Red] += 1
        - check_team_score_limit() = true
        - TeamScoreLimitReached handler:
          - team_winner = Red
          - phase = EndScreen

T+1     Send PlayerDied event           → Receive PlayerDied
T+1     Send MatchPhaseChanged          → Receive MatchPhaseChanged
T+1     Send Snapshot (scores updated)  → Receive Snapshot
                                          - Display team scores
                                          - Show EndScreen UI
```

### Death → Spectate → Respawn Sequence

```text
Time    Server                              Client
─────────────────────────────────────────────────────────────
T       Player killed by Killer
        - is_dead = true
        - spectate_target = Killer.id
        - respawn_tick = T + 180 (3s)

T+1     Send PlayerDied                 → Receive PlayerDied
T+1     Send Snapshot                   → Receive Snapshot
          (spectate_target = Killer.id)   - Camera follows Killer

T+180   respawn_tick reached
        - is_dead = false
        - spectate_target = None
        - position = spawn point

T+181   Send PlayerRespawned            → Receive PlayerRespawned
T+181   Send Snapshot                   → Receive Snapshot
          (spectate_target = None)        - Camera back to self
                                          - Control restored
```

## Error Handling

### Spectate Target Disconnected

If killer disconnects while victim is dead:
1. `spectate_target` remains set to disconnected player ID
2. Next snapshot: `spectate_target` player not in `players` list
3. Client detects missing target: show black screen with respawn timer
4. On respawn: normal camera restored

### Match End During Respawn

If match ends while player is dead:
1. Respawn continues normally in EndScreen phase
2. Combat disabled, but position/spawn works
3. Player can observe EndScreen scoreboard

## Backward Compatibility

All changes are additive:
- `MatchState.team_winner` is new field (clients ignore if not implemented)
- `PlayerSnapshot.spectate_target` is new field (clients ignore if not implemented)
- Existing `scores: Vec<TeamScore>` now actively used but format unchanged
