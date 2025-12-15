# Implementation Plan: Match Flow

**Branch**: `006-match-flow` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/006-match-flow/spec.md`

## Summary

Implements the full competitive match lifecycle: **Lobby → ReadyCheck → Countdown → Playing → EndScreen → Resetting**. The server owns all match state and phase transitions (server-authoritative). Clients send `ReadyToggle` requests; server broadcasts `MatchState` and events. Scoring is kill-based with unlimited respawn; matches end by score limit or time. Arena rotation enables map variety between matches.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: tokio (async), bincode (serialization), glam (math), wgpu (client rendering)
**Storage**: N/A (in-memory state only, no persistence)
**Testing**: `cargo test` (unit + integration tests)
**Target Platform**: Linux/Windows (cross-platform)
**Project Type**: Workspace with multiple crates (plix-common, plix-server, plix-client, plix-net, plix-arena)
**Performance Goals**: 60 Hz tick rate, <16ms per tick, phase transitions within 1 tick
**Constraints**: Server-authoritative (clients cannot force transitions), no stop-the-world pauses
**Scale/Scope**: 2-16 players per server, single arena per match

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Server Authority | ✅ Pass | All phase transitions server-authoritative per FR-027 |
| II. Performance | ✅ Pass | Tick-driven state machine, no blocking operations |
| III. Architecture | ✅ Pass | Extends existing MatchStateMachine, uses engine primitives |
| IV. Modding | ✅ Pass | No mod system changes, uses existing hooks |
| V. Code Quality | ✅ Pass | All network/simulation logic will have tests |
| VI. Technical Standards | ✅ Pass | Stable Rust, cargo clippy/fmt compliance |
| VII. Player Experience | ✅ Pass | Multiplayer-first, fair match start via ready check |
| VIII. Open Source | ✅ Pass | No proprietary dependencies |
| IX. Scoping & Realism | ✅ Pass | Minimal viable scope, extends existing systems |
| X. Long-Term Vision | ✅ Pass | Phase-based design supports future game modes |

## Project Structure

### Documentation (this feature)

```text
specs/006-match-flow/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── ready-toggle.md
│   ├── match-state.md
│   ├── score-update.md
│   └── phase-transition.md
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
crates/
├── plix-common/
│   └── src/
│       ├── protocol/
│       │   └── messages.rs      # Add ReadyToggle, update MatchPhase enum
│       └── types.rs             # Add PlayerMatchStats type
├── plix-server/
│   └── src/
│       ├── match_state.rs       # Refactor to new phase model
│       ├── session.rs           # Add ready flag, score tracking
│       ├── scoring.rs           # NEW: Kill tracking, score updates
│       ├── arena_rotation.rs    # NEW: Arena rotation logic
│       └── game_loop.rs         # Integrate phase restrictions
└── plix-client/
    └── src/
        ├── ui/
        │   ├── match_hud.rs     # NEW: Ready button, countdown, scoreboard
        │   └── end_screen.rs    # NEW: Final scores display
        └── input.rs             # Handle ReadyToggle input
```

**Structure Decision**: Extends existing workspace structure. New files for scoring and arena rotation to maintain separation of concerns. Protocol changes in plix-common for shared types.

## Architecture Overview

### State Ownership

- **Server** owns `MatchState` (phase, scores, timers, player ready status)
- **Client** is view/controller: sends `ReadyToggle`, receives `MatchState` updates

### Phase Model

```
Lobby → ReadyCheck → Countdown → Playing → EndScreen → Resetting → Lobby
         ↑______________|  (cancel on disconnect/unready)
```

| Phase | Duration | Actions Allowed | Exit Condition |
|-------|----------|-----------------|----------------|
| Lobby | Indefinite | Move, join/leave | min_players reached + any ready toggle |
| ReadyCheck | Indefinite | Ready/unready | All ready AND min_players |
| Countdown | 3 seconds (configurable) | None (inputs ignored) | Timer expires OR player disconnects/unreadies |
| Playing | Until end condition | Combat, block edit, move | Score limit OR time limit |
| EndScreen | 5 seconds (configurable) | View only | Timer expires |
| Resetting | <2 seconds | None | World reset complete |

### Key Mechanics

1. **Tick-driven**: All phase transitions occur on server tick (60 Hz)
2. **Ready aggregation**: Server tracks `is_ready` per player, transitions when all ready
3. **Score = Kills**: Player score equals kill count (simplest competitive metric)
4. **Unlimited respawn**: Players respawn during Playing (no elimination)
5. **Match end**: First to `score_limit` OR highest score at `time_limit`
6. **Ties**: If scores equal at time limit, declare tie (no tiebreaker per spec)

## Implementation Phases

### Phase 1: Protocol & Messages

1. Add `ReadyToggle` to `ClientMessage` enum
2. Update `MatchPhase` enum: `Lobby | ReadyCheck | Countdown | Playing | EndScreen | Resetting`
3. Add `MatchPhaseChanged` event to `GameEvent`
4. Add `CountdownTick { remaining: u8 }` event
5. Add `ScoreUpdate { player: PlayerId, kills: u16, deaths: u16 }` event
6. Add `MatchEnded { winner: Option<PlayerId>, scores: Vec<PlayerScore> }` event
7. Extend `MatchState` with `countdown_remaining`, `time_remaining`, `player_scores`

### Phase 2: Server State Machine

1. Refactor `MatchStateMachine` phases to match new model
2. Add `ready_states: HashMap<PlayerId, bool>` to server
3. Implement phase transition logic:
   - `Lobby → ReadyCheck`: Any player toggles ready
   - `ReadyCheck → Countdown`: All ready AND min_players
   - `Countdown → Playing`: Timer expires (cancel if disconnect/unready)
   - `Playing → EndScreen`: Score/time limit reached
   - `EndScreen → Resetting`: Timer expires
   - `Resetting → Lobby`: Reset complete
4. Add phase restrictions in `game_loop.rs`:
   - Lobby: No damage, no block edit (movement OK)
   - Countdown: All inputs ignored
   - Playing: Full gameplay
   - EndScreen/Resetting: All inputs ignored
5. Handle disconnects per phase

### Phase 3: Scoring & Match End

1. Add `score` field to `ServerPlayer` (= kills count)
2. On kill: increment attacker's kills, broadcast `ScoreUpdate`
3. Check score limit after each kill
4. Implement time limit check in tick update
5. Determine winner:
   - Score limit: player who reached it wins
   - Time limit: highest score wins (tie if equal)
6. Broadcast `MatchEnded` with final standings

### Phase 4: Arena Rotation

1. Add `arena_rotation: Vec<String>` to `MatchConfig`
2. Add `current_arena_index: usize` to server state
3. During `Resetting` phase:
   - Increment arena index (wrap to 0)
   - Load next arena from plix-arena
   - Reset world blocks
4. Broadcast `ArenaChanged { name: String }` if arena changes

### Phase 5: Client UX

1. Add ready button UI (toggle on press, show ready state)
2. Display countdown overlay (3... 2... 1...)
3. Show match timer during Playing phase
4. Display scoreboard (kills/deaths per player)
5. Implement end screen:
   - Final scores table
   - Winner highlight (or "Draw" for tie)
   - Auto-dismiss after server transitions

### Phase 6: Validation & Testing

1. Unit tests for phase transitions
2. Integration tests for full match cycle
3. Test edge cases:
   - Disconnect during each phase
   - Ready/unready spam
   - Late joiner handling
4. Verify existing tests still pass
5. Load test with headless clients

## Milestones

| Milestone | Deliverable | Criteria |
|-----------|-------------|----------|
| M1 | Protocol Ready | `ReadyToggle` message, updated `MatchPhase` enum, compiles |
| M2 | State Machine | Server phases work, ready aggregation functional |
| M3 | Scoring | Kills tracked, score/time limits trigger end |
| M4 | Full Cycle | Complete Lobby→End→Lobby loop works |
| M5 | Polish | Arena rotation, client UI, all tests passing |

## Complexity Tracking

> No constitution violations identified. Feature uses existing patterns.

| Decision | Rationale | Alternative Considered |
|----------|-----------|----------------------|
| Score = Kills | Simplest metric, spec-compliant | Complex scoring (objectives, assists) - deferred |
| In-memory ready state | No persistence needed | Database storage - overkill for session data |
| Single winner or tie | Spec requirement | Tiebreaker rounds - explicitly rejected in clarifications |
