# Implementation Plan: TDM Arena Mode

**Branch**: `016-tdm-arena` | **Date**: 2025-12-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/016-tdm-arena/spec.md`

## Summary

Add Team Deathmatch (TDM) game mode to the existing match system. Currently, the server tracks individual player kills as the win condition. TDM mode introduces **team-based scoring** where kills award points to the killer's team (Red or Blue), and the first team to reach the score limit wins. This extends the existing `MatchStateMachine` with team scoring, spectate-killer on death, and auto-reset after match end.

**Key Changes**:
1. Add `TdmMatchConfig` alongside existing `MatchConfig` for TDM-specific parameters
2. Extend `MatchState` to track team scores (already has `scores: Vec<TeamScore>`)
3. Add team score increment on kills (enemy kills only, no friendly fire points)
4. Implement spectate-killer during respawn delay (client-side camera follow)
5. Configure auto-reset after post-match scoreboard (already has `end_screen_ticks`)

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: tokio (async), bincode (serialization), glam (math), wgpu (client rendering)
**Storage**: N/A (in-memory state only, no persistence required for match state)
**Testing**: `cargo test` for unit/integration tests
**Target Platform**: Linux server, cross-platform clients
**Project Type**: Rust workspace with crates (plix-common, plix-server, plix-client)
**Performance Goals**: 60Hz tick rate, O(1) per-event operations
**Constraints**: Server-authoritative, no client game logic
**Scale/Scope**: 2-16 players per match, 2 teams (Red/Blue)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | ✅ PASS | All TDM logic server-side; team assignment, scoring, respawn server-authoritative |
| II. Performance (Low Latency) | ✅ PASS | O(1) per kill/death/respawn; no world scans; tick-driven timers |
| III. Architecture (Engine-First) | ✅ PASS | Extends existing match_state module; reuses TeamId, MatchPhase infrastructure |
| IV. Modding (Extensibility) | N/A | TDM is core gameplay, not a mod; follows existing patterns |
| V. Code Quality (Explicit & Tested) | ✅ PASS | Unit tests for scoring, state machine, respawn; no clever tricks |
| VI. Technical Standards (Stable Rust) | ✅ PASS | Stable Rust only; cargo clippy + fmt enforced |
| VII. Player Experience (Multiplayer-First) | ✅ PASS | TDM is inherently multiplayer; team balance on join |
| VIII. Open Source | ✅ PASS | No proprietary dependencies |
| IX. Scoping (Minimal Viable) | ✅ PASS | MVP defined: team scoring, respawn, match end, auto-reset. No K/D stats, spectator mode, or advanced features |
| X. Long-Term Vision | ✅ PASS | Game mode is a layer on existing infrastructure; no breaking changes |

**Gate Result**: PASS - No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/016-tdm-arena/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (internal event contracts)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── plix-common/src/
│   ├── protocol/messages.rs    # MODIFY: Add TdmScoreUpdate, SpectateTarget events
│   └── types.rs                # MODIFY: Add Team enum (Red/Blue) if needed
│
├── plix-server/src/
│   ├── match_state.rs          # MODIFY: Add team scoring logic, TdmMatchConfig
│   ├── session.rs              # MODIFY: Add spectate_target field to player
│   └── lib.rs                  # MODIFY: Hook team scoring on kill, spectate on death
│
└── plix-client/src/
    └── spectate.rs             # ADD: Camera follow logic for killer spectate

tests/
└── integration/
    └── tdm_match_test.rs       # ADD: Full TDM match simulation test
```

**Structure Decision**: Minimal new files. TDM extends existing `match_state.rs` with team scoring. Spectate is a new client module but integrates with existing camera system.

## Architecture Overview

### Game Mode Layer

```text
┌─────────────────────────────────────────────────────────────┐
│                      Server Main Loop                       │
│   tick() -> simulate_tick() -> send_snapshots()            │
├─────────────────────────────────────────────────────────────┤
│                    Match State Machine                      │
│   Lobby → Countdown → Playing → EndScreen → Resetting       │
│                         │                                   │
│   [TDM Extension]       │                                   │
│   - team_scores: {Red: 0, Blue: 0}                         │
│   - on_kill: team_scores[killer.team] += 1                 │
│   - win: team_scores[team] >= score_limit                  │
├─────────────────────────────────────────────────────────────┤
│                     Session Manager                         │
│   - player.team: TeamId (Red/Blue)                         │
│   - player.is_dead: bool                                   │
│   - player.respawn_tick: Option<Tick>                      │
│   - player.spectate_target: Option<PlayerId> [NEW]         │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow: Kill → Team Score

```text
1. Combat System detects hit, kills target
2. Server.simulate_tick() calls:
   - match_state.award_team_kill(killer.team)
   - victim.set_spectate_target(killer.id)
   - victim.respawn_tick = current_tick + respawn_delay
3. Match state broadcasts:
   - GameEvent::TeamScoreUpdate { team, new_score }
   - GameEvent::PlayerDied { victim, killer }
4. Client receives events:
   - Updates HUD team score display
   - Victim camera switches to spectate killer
5. After respawn_delay ticks:
   - victim.respawn()
   - victim.spectate_target = None
   - GameEvent::PlayerRespawned { id }
```

### State Machine Extensions

```text
MatchStateMachine (existing):
  - phase: MatchPhase
  - player_scores: Vec<PlayerScore>  // Individual K/D
  - winner: Option<PlayerId>         // Individual winner

MatchStateMachine (TDM extension):
  + scores: Vec<TeamScore>           // Already exists, now actively used
  + team_winner: Option<TeamId>      // Team that won (replaces winner for TDM)
  + award_team_kill(team: TeamId)    // Increment team score
  + check_team_score_limit()         // Check if team reached limit
```

## Complexity Tracking

No constitution violations requiring justification. Implementation uses existing infrastructure:
- TeamId already defined in types.rs
- TeamScore already in protocol/messages.rs
- MatchPhase state machine already implemented
- Respawn delay already tracked in session

## Key Design Decisions

### D1: Team Scoring vs Individual Scoring

**Decision**: TDM uses team scoring (sum of all team members' kills).
**Rationale**: Standard TDM gameplay. The existing `player_scores` remains for per-player stats display.
**Implementation**: `match_state.scores[team].score += 1` on each valid kill.

### D2: Win Condition Check Timing

**Decision**: Check team score limit immediately after awarding kill point.
**Rationale**: Prevents race conditions with simultaneous kills. First team to reach limit wins.
**Implementation**: Inside `award_team_kill()`, call `check_team_score_limit()` and transition to EndScreen if met.

### D3: Spectate Target Assignment

**Decision**: Spectate killer (victim.spectate_target = killer.id) on death.
**Rationale**: Per clarification Q1, players spectate their killer during respawn delay.
**Implementation**: Set in `simulate_tick()` when processing kill, clear on respawn.

### D4: Auto-Reset vs Manual Reset

**Decision**: Auto-reset after `reset_delay` (15s default) in EndScreen phase.
**Rationale**: Per clarification Q2, matches cycle automatically.
**Implementation**: Existing `end_screen_ticks` already handles this. Just ensure `complete_reset()` clears team scores.

### D5: Friendly Fire Handling

**Decision**: No team score for friendly fire (already covered by combat system not awarding kills for same-team hits).
**Rationale**: Spec FR-006 requires no points for friendly fire.
**Implementation**: Check `killer.team != victim.team` before awarding team point.

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Team imbalance (5v3) | Medium | Low | Auto-balance on join (existing); no mid-match rebalance |
| Score desync client/server | Low | Medium | Server authoritative; client only displays received state |
| Spectate target invalid (disconnected) | Low | Low | Fallback to free camera or black screen if target gone |
| Simultaneous kills race | Low | Low | Process kills sequentially per tick; first to increment wins |

## Integration Points

### Existing Systems Modified

1. **match_state.rs**: Add team scoring methods, team-based win condition
2. **session.rs**: Add spectate_target field
3. **lib.rs (Server)**: Hook team scoring on kill event, set spectate target on death
4. **protocol/messages.rs**: Add TeamScoreUpdate event (or reuse existing GameEvent)

### New Systems Added

1. **spectate.rs (Client)**: Camera follow logic for spectating killer

### Arena Integration

Arena TOML files already support spawn points. For TDM, spawn points need team affiliation:
- Option A: Existing spawn_points with team field (already present)
- Option B: Separate red_spawns / blue_spawns arrays

**Decision**: Use existing spawn point structure with `team` field. Already implemented in plix-arena.
