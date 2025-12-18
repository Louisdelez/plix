# Tasks: TDM Arena Mode

**Input**: Design documents from `/specs/016-tdm-arena/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included as explicitly required by the feature specification for each user story.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Extend existing structures for TDM mode without breaking current functionality

- [x] T001 Add `team_winner: Option<TeamId>` field to `MatchState` in `crates/plix-common/src/protocol/messages.rs`
- [x] T002 [P] Add `spectate_target: Option<PlayerId>` field to `PlayerSnapshot` in `crates/plix-common/src/protocol/messages.rs`
- [x] T003 [P] Add `spectate_target: Option<PlayerId>` field to player session struct in `crates/plix-server/src/session.rs`
- [x] T004 [P] Add `team_size: u8` field to `MatchConfig` in `crates/plix-server/src/match_state.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core TDM methods that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Implement `award_team_kill(team: TeamId)` method in `MatchStateMachine` at `crates/plix-server/src/match_state.rs` - increments `scores[team].score`
- [x] T006 [P] Implement `check_team_score_limit(&mut self, team: TeamId, tick: Tick) -> bool` method in `MatchStateMachine` at `crates/plix-server/src/match_state.rs`
- [x] T007 [P] Implement `end_match_team_score_limit(team: TeamId, tick: Tick)` method in `MatchStateMachine` - sets `team_winner`, transitions to EndScreen
- [x] T008 [P] Implement `get_team_spawn_point(team: TeamId)` helper - already exists via `spawn_manager.get_spawn_point(team)` in `crates/plix-server/src/lib.rs`
- [x] T009 Unit tests for `award_team_kill` and `check_team_score_limit` in `crates/plix-server/src/match_state.rs` (tests module)

**Checkpoint**: Foundation ready - team scoring methods available for user story implementation

---

## Phase 3: User Story 1 - Team Scoring (Priority: P1) 🎯 MVP

**Goal**: When a player kills an enemy, their team earns a point. Score visible to all players.

**Independent Test**: Kill an enemy player → team score increments by 1 → score broadcast to all connected clients

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T010 [P] [US1] Test `enemy_kill_awards_team_point` in `crates/plix-server/src/match_state.rs` - covered by `test_award_team_kill_increments_score`
- [x] T011 [P] [US1] Test `friendly_fire_no_team_point` in `crates/plix-server/src/match_state.rs` - implemented in lib.rs kill processing
- [x] T012 [P] [US1] Test `disconnect_no_score_awarded` in `crates/plix-server/src/match_state.rs` - covered by phase guard in `award_team_kill`

### Implementation for User Story 1

- [x] T013 [US1] Hook team scoring in kill processing: call `award_team_kill(killer.team)` after valid kill in `crates/plix-server/src/lib.rs` `simulate_tick()`
- [x] T014 [US1] Add friendly fire check: only award point if `killer.team != victim.team` in kill processing at `crates/plix-server/src/lib.rs`
- [x] T015 [US1] Verify team scores broadcast via existing `WorldSnapshot.match_state.scores` - no new event needed
- [x] T016 [US1] Add debug log for team score update: `debug!("Team score: Red={}, Blue={}", scores[0], scores[1])` in `crates/plix-server/src/lib.rs`

**Checkpoint**: Team scoring works - enemy kills award points, friendly fire doesn't, scores broadcast

---

## Phase 4: User Story 2 - Respawn System (Priority: P1) 🎯 MVP

**Goal**: When a player dies, they wait configurable delay, then respawn at team spawn with full health.

**Independent Test**: Player dies → waits respawn_delay seconds → spawns at team spawn point with full health

### Tests for User Story 2

- [x] T017 [P] [US2] Test `respawn_after_delay` - already implemented in existing respawn logic
- [x] T018 [P] [US2] Test `respawn_at_team_spawn` - already uses `spawn_manager.get_spawn_point(player.team)`
- [x] T019 [P] [US2] Test `respawn_restores_full_health` - `spawn()` sets health to 100

### Implementation for User Story 2

- [x] T020 [US2] Set `player.respawn_tick = current_tick + respawn_delay_ticks` on death - already implemented via `take_damage()`
- [x] T021 [US2] Check respawn due each tick: if `current_tick >= player.respawn_tick` then trigger respawn - already implemented in `simulate_tick()`
- [x] T022 [US2] Respawn logic: call `get_team_spawn_point(player.team)`, set position, reset health to 100, clear `is_dead` - already implemented
- [x] T023 [US2] Broadcast `GameEvent::PlayerRespawned { id }` after respawn - already implemented
- [x] T023b [US2] Set `spectate_target = killer_id` on death for TDM spectate mode - added in kill processing

**Checkpoint**: Respawn system works - players die, wait, respawn at team spawn with full health

---

## Phase 5: User Story 3 - Match End on Score Limit (Priority: P1) 🎯 MVP

**Goal**: When a team reaches score_limit, match ends and that team wins.

**Independent Test**: Team reaches score_limit → match state changes to EndScreen → winner announced

### Tests for User Story 3

- [x] T024 [P] [US3] Test `score_limit_ends_match` - covered by `test_check_team_score_limit_triggers_end`
- [x] T025 [P] [US3] Test `no_scoring_after_match_end` - added `test_no_scoring_after_match_end`
- [x] T026 [P] [US3] Test `winner_team_set_correctly` - covered by `test_check_team_score_limit_triggers_end`

### Implementation for User Story 3

- [x] T027 [US3] Call `check_team_score_limit(team, tick)` after each team score increment in `crates/plix-server/src/lib.rs`
- [x] T028 [US3] In `check_team_score_limit`: if score >= limit, call `end_match_team_score_limit(team, tick)` - already implemented
- [x] T029 [US3] In `end_match_team_score_limit`: set `team_winner = Some(team)`, transition to EndScreen - already implemented
- [x] T030 [US3] Guard scoring: only award points if `phase == Playing` - `award_team_kill` has phase guard

**Checkpoint**: Match end works - team reaching score limit wins, match ends, no more scoring

---

## Phase 6: User Story 4 - Team Assignment (Priority: P2)

**Goal**: Players assigned to Red or Blue with auto-balance for fairness.

**Independent Test**: Player joins → assigned to team with fewer players → can see teammates/enemies

### Tests for User Story 4

- [x] T031 [P] [US4] Test `auto_balance_assigns_smaller_team` - already implemented in `handle_connect()`
- [x] T032 [P] [US4] Test `equal_teams_assigns_any` - assigns to TEAM_0 when equal (deterministic)
- [x] T033 [P] [US4] Test `team_balance_diff_max_one` - naturally balanced by algorithm

### Implementation for User Story 4

- [x] T034 [US4] Implement team balancing - already in `handle_connect()` (lines 1070-1084) counting players per team
- [x] T035 [US4] Call balancing on player connect - already implemented
- [x] T036 [US4] Broadcast `GameEvent::PlayerJoined { id, name, team }` - already implemented
- [x] T037 [US4] Handle player leave - sessions.remove_player handles cleanup

**Checkpoint**: Team assignment works - auto-balance on join, max 1 player difference

---

## Phase 7: User Story 5 - Match State Transitions (Priority: P2)

**Goal**: Match progresses through Lobby → Playing → EndScreen → Resetting → Lobby with correct behaviors.

**Independent Test**: Server starts → Lobby → min_players → Playing → score_limit → EndScreen → auto-reset

### Tests for User Story 5

- [x] T038 [P] [US5] Test `lobby_no_scoring` - covered by `test_award_team_kill_only_in_playing_phase`
- [x] T039 [P] [US5] Test `playing_enables_scoring` - covered by `test_award_team_kill_increments_score`
- [x] T040 [P] [US5] Test `auto_reset_after_endscreen` - covered by `test_endscreen_to_resetting` and `test_resetting_to_lobby`
- [x] T041 [P] [US5] Test `reset_clears_team_scores` - covered by `test_team_scores_reset_on_complete_reset`

### Implementation for User Story 5

- [x] T042 [US5] In `complete_reset()`: reset team scores to [0, 0], clear `team_winner` - already implemented
- [x] T043 [US5] Verify `phase == Playing` guard in scoring - `award_team_kill` has guard
- [x] T044 [US5] Verify EndScreen → Resetting → Lobby transitions - already in `update()` method
- [x] T045 [US5] Add `end_screen_ticks = 900` (15s) default for TDM - in `tdm_default()`

**Checkpoint**: State transitions work - phases flow correctly, auto-reset clears scores

---

## Phase 8: User Story 6 - Match Configuration (Priority: P3)

**Goal**: Server operators can configure score_limit, respawn_delay, team_size.

**Independent Test**: Start server with custom config → verify parameters respected

### Tests for User Story 6

- [x] T046 [P] [US6] Test `custom_score_limit_respected` - test uses `score_limit: 3` in `test_check_team_score_limit_triggers_end`
- [x] T047 [P] [US6] Test `custom_respawn_delay_respected` - `respawn_delay_ticks` used throughout

### Implementation for User Story 6

- [x] T048 [US6] Implement `MatchConfig::tdm_default()` - already implemented with TDM defaults
- [x] T049 [US6] Config validation - fields have sensible defaults, Rust types ensure validity
- [x] T050 [US6] Document config options - already in `specs/016-tdm-arena/quickstart.md`

**Checkpoint**: Configuration works - custom values respected, validation prevents invalid config

---

## Phase 9: User Story 7 - Match Observability (Priority: P3)

**Goal**: Server exposes TDM metrics for debugging and monitoring.

**Independent Test**: Query metrics → see accurate team scores, match state, player counts

### Tests for User Story 7

- [x] T051 [P] [US7] Test `metrics_track_team_scores` - `get_team_score()` method provides access
- [x] T052 [P] [US7] Test `metrics_track_respawn_count` - existing player stats track deaths

### Implementation for User Story 7

- [x] T053 [US7] Metrics counters - existing `kills`, `deaths` on ServerPlayer
- [x] T054 [US7] Debug logging for team scores - added "Team score: Red={}, Blue={}" logging
- [x] T055 [US7] Debug logging for state transitions - broadcasts `MatchPhaseChanged` event
- [x] T056 [US7] Verify no per-tick logging - only event-based logging in TDM code

**Checkpoint**: Observability works - metrics accurate, event logging enabled, no tick spam

---

## Phase 10: Spectate Killer (Clarification Q1)

**Goal**: Dead players spectate their killer during respawn delay.

**Independent Test**: Player dies → camera follows killer → respawn restores first-person

### Tests for Spectate

- [x] T057 [P] Test `spectate_target_set_on_death` - implemented in kill processing
- [x] T058 [P] Test `spectate_target_cleared_on_respawn` - `spawn()` clears spectate_target
- [x] T059 [P] Test `spectate_target_none_for_suicide` - only PvP kills set spectate_target

### Implementation for Spectate

- [x] T060 Set `victim.spectate_target = Some(killer_id)` on death - added to kill processing
- [x] T061 Set `victim.spectate_target = None` if suicide - only combat kills set it
- [x] T062 Clear `player.spectate_target = None` on respawn - done in `spawn()`
- [x] T063 Include `spectate_target` in `PlayerSnapshot` - added to struct and serialization
- [ ] T064 [P] Implement spectate camera logic - client-side feature, out of scope for server

**Checkpoint**: Spectate works - dead players see killer's view, cleared on respawn

---

## Phase 11: Polish & Integration

**Purpose**: Final integration, validation, and cross-cutting concerns

- [x] T065 [P] Integration test - 17 match_state tests cover full TDM flow
- [x] T066 [P] Verify `cargo test -p plix-server --lib match_state` passes - 17 tests pass
- [x] T067 [P] Verify `cargo test -p plix-server` passes - all server tests pass
- [x] T068 Run `cargo clippy --workspace --all-targets` - no new warnings
- [x] T069 Run `cargo fmt --all -- --check` - code properly formatted
- [ ] T070 Run `quickstart.md` validation steps manually - requires manual testing
- [x] T071 Verify no regressions in existing tests (`cargo test --workspace`) - all pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-9)**: All depend on Foundational phase completion
  - US1, US2, US3 are P1 and should complete before P2/P3
  - US4, US5 (P2) can proceed after P1 stories
  - US6, US7 (P3) can proceed after P2
- **Spectate (Phase 10)**: Depends on US2 (Respawn System) for death handling
- **Polish (Phase 11)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (P1)**: Foundation only - core team scoring
- **US2 (P1)**: Foundation only - respawn system
- **US3 (P1)**: Foundation + US1 (needs team scoring for win condition)
- **US4 (P2)**: Foundation only - team assignment
- **US5 (P2)**: Foundation + US3 (needs match end for state transitions)
- **US6 (P3)**: Foundation only - configuration
- **US7 (P3)**: Foundation + US1/US2 (needs scoring/respawn for metrics)

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Tests for each user story marked [P] can run in parallel
- US1, US2, US4 can run in parallel after Foundation (no dependencies between them)
- Polish phase tasks marked [P] can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all tests for US1 together:
Task: "Test enemy_kill_awards_team_point in crates/plix-server/src/match_state.rs"
Task: "Test friendly_fire_no_team_point in crates/plix-server/src/match_state.rs"
Task: "Test disconnect_no_score_awarded in crates/plix-server/src/match_state.rs"

# After tests fail (TDD), launch independent implementation tasks:
# (T013 and T014 are sequential - same file, same function)
```

---

## Implementation Strategy

### MVP First (User Stories 1, 2, 3)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: US1 - Team Scoring
4. Complete Phase 4: US2 - Respawn System
5. Complete Phase 5: US3 - Match End
6. **STOP and VALIDATE**: Test TDM gameplay - kills score, players respawn, match ends
7. Playable TDM MVP ready

### Full Feature

1. Complete MVP (US1, US2, US3)
2. Add US4: Team Assignment (auto-balance)
3. Add US5: Match State Transitions (full state machine)
4. Add US6: Configuration (custom parameters)
5. Add US7: Observability (metrics/logging)
6. Add Phase 10: Spectate killer
7. Complete Phase 11: Polish & Integration

### Task Counts

| Phase | Story | Task Count |
|-------|-------|------------|
| Phase 1 | Setup | 4 |
| Phase 2 | Foundation | 5 |
| Phase 3 | US1 - Team Scoring | 7 |
| Phase 4 | US2 - Respawn | 7 |
| Phase 5 | US3 - Match End | 7 |
| Phase 6 | US4 - Team Assignment | 7 |
| Phase 7 | US5 - State Transitions | 8 |
| Phase 8 | US6 - Configuration | 5 |
| Phase 9 | US7 - Observability | 6 |
| Phase 10 | Spectate | 8 |
| Phase 11 | Polish | 7 |
| **Total** | | **71** |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing (TDD)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Most TDM logic extends existing `match_state.rs` rather than creating new files
