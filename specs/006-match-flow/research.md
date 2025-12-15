# Research: Match Flow

**Feature**: 006-match-flow | **Date**: 2025-12-15

## Existing Codebase Analysis

### Current Match State Machine

**Location**: `crates/plix-server/src/match_state.rs`

The existing `MatchStateMachine` provides a foundation with these phases:
- `WaitingForPlayers` - Server idle until min_players reached
- `Countdown` - Timer before round starts
- `Playing` - Active gameplay
- `RoundEnd` - Brief pause between rounds
- `MatchEnd` - Match complete

**Key observations**:
1. Phase transitions are tick-driven (good - matches our design)
2. Uses `MatchConfig` for configurable parameters
3. Team-based scoring via `TeamScore` struct
4. Round-based model (rounds_to_win) differs from our kill-based score limit

**Gap analysis**:
- Missing `Lobby` phase (currently jumps straight to countdown)
- Missing `ReadyCheck` phase (no explicit ready system)
- Missing `Resetting` phase (no world reset flow)
- No per-player ready tracking
- No arena rotation support

### Current Protocol Messages

**Location**: `crates/plix-common/src/protocol/messages.rs`

**ClientMessage enum**:
- `Connect`, `Disconnect`, `Input`, `SnapshotAck`, `BlockEdit`
- **Missing**: `ReadyToggle` message

**ServerMessage enum**:
- `Connected`, `Rejected`, `Kicked`, `Snapshot`, `Event`
- Structure supports adding new events easily

**GameEvent enum**:
- Already has: `RoundStart`, `RoundEnd`, `MatchEnd`, `PlayerDied`, `PlayerRespawned`
- **Missing**: `MatchPhaseChanged`, `CountdownTick`, `ScoreUpdate`

**MatchState struct**:
- Contains: `phase`, `round_number`, `round_start_tick`, `round_time_limit`, `scores`
- **Missing**: `countdown_remaining`, `player_scores`, `time_remaining`

### Current Session Management

**Location**: `crates/plix-server/src/session.rs`

**ServerPlayer struct**:
- Has: `kills`, `deaths` fields (already tracked!)
- Has: `respawn_tick` for respawn timing
- **Missing**: `is_ready` flag, `score` (can derive from kills)

**SessionManager**:
- Handles add/remove players, max_players enforcement
- **Missing**: Ready state aggregation methods

### Arena System

**Location**: `crates/plix-arena/`

Need to investigate arena loading mechanism for rotation support.

## Technical Decisions

### Phase Model Mapping

| Spec Phase | Implementation |
|------------|----------------|
| Lobby | New phase - movement allowed, no combat |
| ReadyCheck | Combined into Lobby (ready UI shows in Lobby) |
| Countdown | Existing - refactor to cancel on disconnect/unready |
| Playing | Existing - add phase restrictions |
| EndScreen | New phase - replaces RoundEnd for final display |
| Resetting | New phase - world reset + arena rotation |

**Decision**: Merge Lobby and ReadyCheck into single `Lobby` phase with ready state tracking. Simpler implementation, same user experience.

### Score Model

**Current**: Team-based rounds (first to N rounds wins)
**New**: Player-based kills (first to score_limit OR time_limit)

**Decision**: Keep both models configurable. Default to kill-based scoring for this feature. Team scoring can be layered on top (player kills contribute to team score).

### Ready State Storage

**Options**:
1. Add `is_ready: bool` to `ServerPlayer`
2. Separate `ready_states: HashMap<PlayerId, bool>` in server

**Decision**: Option 1 - Add to `ServerPlayer`. Simpler, natural fit, cleared during reset anyway.

### Phase Restrictions

**Implementation approach**:
- Check phase in `game_loop.rs` before processing damage/block edits
- Movement always allowed (except during Countdown? - spec unclear, allow for now)
- Block edit already checks `InvalidPhase` - extend to check Lobby

### Arena Rotation

**Implementation approach**:
1. Add `arena_rotation: Vec<String>` to server config
2. Track `current_arena_index` in server state
3. On `Resetting` phase: increment index, load arena, reset world
4. Empty list = replay same arena

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking existing tests | High | Run test suite after each phase implementation |
| Protocol version mismatch | Medium | Bump protocol version, document changes |
| Performance regression | Medium | Profile tick time, ensure <16ms |
| Late joiner edge cases | Low | Explicit spectator state, include in tests |

## Dependencies

### Internal Dependencies
- `plix-common`: Protocol types (must update first)
- `plix-arena`: Arena loading (for rotation)
- `plix-net`: No changes expected
- `plix-client`: UI updates (parallel to server work)

### External Dependencies
- None added. Uses existing crate ecosystem.

## Implementation Order

1. **Protocol changes** (plix-common) - Foundation for everything
2. **Server state machine** (plix-server) - Core phase logic
3. **Phase restrictions** (plix-server) - Enforce rules per phase
4. **Scoring system** (plix-server) - Kill tracking, end conditions
5. **Arena rotation** (plix-server + plix-arena) - Match variety
6. **Client UI** (plix-client) - User-facing elements
7. **Testing** - Unit + integration + edge cases

## Open Questions (Resolved)

1. ~~Respawn during Playing phase?~~ → **Unlimited respawn** (clarified in spec)
2. ~~Tie handling?~~ → **Declare tie** (clarified in spec)
3. ~~Movement during Countdown?~~ → **Yes**, inputs processed but actions blocked
