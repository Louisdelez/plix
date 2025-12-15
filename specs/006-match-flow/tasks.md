# Tasks: Match Flow

**Input**: Design documents from `/specs/006-match-flow/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: Included per constitution requirement (Code Quality - Mandatory Testing)

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Workspace structure**: `crates/plix-common/`, `crates/plix-server/`, `crates/plix-client/`

---

## Phase 1: Setup (Protocol & Shared Types)

**Purpose**: Add shared protocol types and messages required by all user stories

- [x] T001 [P] Update MatchPhase enum in crates/plix-common/src/protocol/messages.rs (Lobby, Countdown, Playing, EndScreen, Resetting)
- [x] T002 [P] Add PlayerScore struct in crates/plix-common/src/protocol/messages.rs
- [x] T003 [P] Add MatchEndReason enum in crates/plix-common/src/protocol/messages.rs (ScoreLimit, TimeLimit, Forfeit)
- [x] T004 Add ReadyToggle variant to ClientMessage enum in crates/plix-common/src/protocol/messages.rs
- [x] T005 Add MatchPhaseChanged event to GameEvent enum in crates/plix-common/src/protocol/messages.rs
- [x] T006 [P] Add CountdownTick event to GameEvent enum in crates/plix-common/src/protocol/messages.rs
- [x] T007 [P] Add ScoreUpdate event to GameEvent enum in crates/plix-common/src/protocol/messages.rs
- [x] T008 Update MatchEnded event in GameEvent to include scores and reason in crates/plix-common/src/protocol/messages.rs
- [x] T009 [P] Add ArenaChanged event to GameEvent enum in crates/plix-common/src/protocol/messages.rs
- [x] T010 Update MatchState struct with countdown_remaining, time_remaining, score_limit, player_scores, winner, arena_name in crates/plix-common/src/protocol/messages.rs
- [x] T011 Verify protocol changes compile: cargo build -p plix-common
- [x] T012 Add roundtrip encode/decode tests for new messages in crates/plix-common/src/protocol/messages.rs

**Checkpoint**: Protocol foundation ready - server/client can now use new types

---

## Phase 2: Foundational (Server Match State Machine)

**Purpose**: Core server infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T013 Add is_ready field to ServerPlayer struct in crates/plix-server/src/session.rs
- [x] T014 Add clear_ready() method to ServerPlayer in crates/plix-server/src/session.rs
- [x] T015 Update MatchConfig with score_limit, time_limit_seconds, end_screen_ticks, arena_rotation fields in crates/plix-server/src/match_state.rs
- [x] T016 Refactor MatchStateMachine to use new MatchPhase variants (Lobby, Countdown, Playing, EndScreen, Resetting) in crates/plix-server/src/match_state.rs
- [x] T017 Add phase_start_tick and arena_index fields to MatchStateMachine in crates/plix-server/src/match_state.rs
- [x] T018 Add ready_count() method to count ready players in SessionManager in crates/plix-server/src/session.rs
- [x] T019 Verify server compiles and boots: cargo build -p plix-server && timeout 3 cargo run -p plix-server

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Ready Up and Start Match (Priority: P1) 🎯 MVP

**Goal**: Players can toggle ready and start match via countdown when all ready

**Independent Test**: Two players connect, both press Ready, observe 3-second countdown, then match begins with players at spawn points

### Tests for User Story 1

- [x] T020 [P] [US1] Unit test: all ready triggers Lobby→Countdown transition in crates/plix-server/src/match_state.rs (tests module)
- [x] T021 [P] [US1] Unit test: unready during Countdown cancels back to Lobby in crates/plix-server/src/match_state.rs
- [x] T022 [P] [US1] Unit test: disconnect during Countdown cancels back to Lobby in crates/plix-server/src/match_state.rs
- [x] T023 [P] [US1] Unit test: Countdown timer expires → Playing transition in crates/plix-server/src/match_state.rs

### Implementation for User Story 1

- [x] T024 [US1] Implement Lobby→Countdown transition when all ready AND min_players met in crates/plix-server/src/match_state.rs
- [x] T025 [US1] Implement Countdown→Lobby cancellation on disconnect/unready in crates/plix-server/src/match_state.rs
- [x] T026 [US1] Implement Countdown timer decrement and countdown_remaining tracking in crates/plix-server/src/match_state.rs
- [x] T027 [US1] Implement Countdown→Playing transition when timer expires in crates/plix-server/src/match_state.rs
- [x] T028 [US1] Handle ReadyToggle message in server message handler (toggle is_ready, check transition) in crates/plix-server/src/lib.rs
- [x] T029 [US1] Broadcast MatchPhaseChanged event on phase transitions in crates/plix-server/src/lib.rs
- [x] T030 [US1] Broadcast CountdownTick event each second during Countdown in crates/plix-server/src/lib.rs
- [x] T031 [US1] Spawn all players at arena spawn points when entering Playing phase in crates/plix-server/src/lib.rs
- [x] T032 [US1] Reset player health and match stats when entering Playing phase in crates/plix-server/src/lib.rs
- [x] T033 [US1] Verify tests pass: cargo test -p plix-server -- ready countdown lobby

**Checkpoint**: User Story 1 complete - ready→countdown→playing flow works

---

## Phase 4: User Story 2 - Play Match with Scoring (Priority: P1)

**Goal**: Kills tracked during Playing phase, match ends on score/time limit

**Independent Test**: Start a match, one player eliminates another, verify kill count increases. Continue until score limit triggers match end.

### Tests for User Story 2

- [x] T034 [P] [US2] Unit test: kill increments attacker kills and victim deaths in crates/plix-server/src/match_state.rs
- [x] T035 [P] [US2] Unit test: score_limit reached triggers Playing→EndScreen in crates/plix-server/src/match_state.rs
- [x] T036 [P] [US2] Unit test: time_limit reached triggers Playing→EndScreen in crates/plix-server/src/match_state.rs
- [x] T037 [P] [US2] Unit test: tie at time_limit → winner is None in crates/plix-server/src/match_state.rs

### Implementation for User Story 2

- [x] T038 [US2] Create scoring.rs module in crates/plix-server/src/scoring.rs with kill tracking functions (integrated in match_state.rs)
- [x] T039 [US2] Update kill processing to increment attacker.kills, victim.deaths in crates/plix-server/src/lib.rs (combat handler)
- [x] T040 [US2] Broadcast ScoreUpdate event on each kill in crates/plix-server/src/lib.rs
- [x] T041 [US2] Check score_limit after each kill, trigger Playing→EndScreen if reached in crates/plix-server/src/match_state.rs
- [x] T042 [US2] Implement time_remaining decrement during Playing phase in crates/plix-server/src/match_state.rs
- [x] T043 [US2] Check time_limit in tick update, trigger Playing→EndScreen if reached in crates/plix-server/src/match_state.rs
- [x] T044 [US2] Determine winner: highest score wins, None if tie in crates/plix-server/src/scoring.rs (integrated in match_state.rs)
- [x] T045 [US2] Broadcast MatchEnd event with winner, scores, reason on EndScreen transition in crates/plix-server/src/lib.rs
- [x] T046 [US2] Verify tests pass: cargo test -p plix-server -- scoring kill limit

**Checkpoint**: User Story 2 complete - scoring and match end conditions work

---

## Phase 5: User Story 3 - View End Screen and Restart (Priority: P1)

**Goal**: EndScreen displays final scores, then server resets for next match

**Independent Test**: Complete a match, verify end screen displays for 5 seconds, then server resets to Lobby with all players still connected.

### Tests for User Story 3

- [x] T047 [P] [US3] Unit test: EndScreen timer expires → Resetting transition in crates/plix-server/src/match_state.rs
- [x] T048 [P] [US3] Unit test: Resetting completes → Lobby transition in crates/plix-server/src/match_state.rs
- [x] T049 [P] [US3] Unit test: scores and ready states cleared on reset in crates/plix-server/src/match_state.rs

### Implementation for User Story 3

- [x] T050 [US3] Implement EndScreen timer (end_screen_ticks) decrement in crates/plix-server/src/match_state.rs
- [x] T051 [US3] Implement EndScreen→Resetting transition when timer expires in crates/plix-server/src/match_state.rs
- [x] T052 [US3] Implement world reset during Resetting phase (reset blocks to arena baseline) in crates/plix-server/src/lib.rs
- [x] T053 [US3] Clear all player is_ready flags during Resetting in crates/plix-server/src/session.rs
- [x] T054 [US3] Clear all player kills/deaths during Resetting in crates/plix-server/src/session.rs
- [x] T055 [US3] Implement Resetting→Lobby transition when reset completes in crates/plix-server/src/match_state.rs
- [x] T056 [US3] Broadcast MatchPhaseChanged for EndScreen→Resetting→Lobby transitions in crates/plix-server/src/lib.rs
- [x] T057 [US3] Verify tests pass: cargo test -p plix-server -- endscreen reset

**Checkpoint**: User Story 3 complete - full match cycle Lobby→End→Lobby works

---

## Phase 6: User Story 4 - Arena Rotation (Priority: P2)

**Goal**: Server rotates to next arena after match ends (if configured)

**Independent Test**: Configure server with 2 arenas. Complete match on arena 1, verify next match loads arena 2.

### Tests for User Story 4

- [x] T058 [P] [US4] Unit test: arena_index increments on rotation in crates/plix-server/src/match_state.rs
- [x] T059 [P] [US4] Unit test: arena_index wraps to 0 at end of list in crates/plix-server/src/match_state.rs
- [x] T060 [P] [US4] Unit test: empty rotation list replays same arena in crates/plix-server/src/match_state.rs

### Implementation for User Story 4

- [x] T061 [US4] Create arena_rotation.rs module in crates/plix-server/src/arena_rotation.rs (integrated in match_state.rs)
- [x] T062 [US4] Add arena_rotation config parsing (CLI or config file) in crates/plix-server/src/main.rs (MatchConfig struct)
- [x] T063 [US4] Implement arena index increment with wraparound in crates/plix-server/src/arena_rotation.rs (in complete_reset())
- [x] T064 [US4] Load next arena during Resetting phase in crates/plix-server/src/lib.rs
- [x] T065 [US4] Broadcast ArenaChanged event when arena changes in crates/plix-server/src/lib.rs
- [x] T066 [US4] Verify tests pass: cargo test -p plix-server -- arena rotation

**Checkpoint**: User Story 4 complete - arena rotation works

---

## Phase 7: User Story 5 - Lobby Phase Restrictions (Priority: P2)

**Goal**: Combat and block edits disabled during Lobby phase

**Independent Test**: In Lobby phase, attempt to attack another player. Verify no damage is dealt.

### Tests for User Story 5

- [x] T067 [P] [US5] Unit test: damage rejected in Lobby phase in crates/plix-server/src/lib.rs (tests) (simulate_tick only runs in Playing)
- [x] T068 [P] [US5] Unit test: block edit rejected in Lobby phase in crates/plix-server/src/lib.rs (tests) (test_validate_rejects_invalid_phase_lobby)
- [x] T069 [P] [US5] Unit test: movement allowed in Lobby phase in crates/plix-server/src/lib.rs (tests) (movement not gated by phase)

### Implementation for User Story 5

- [x] T070 [US5] Gate combat processing: skip damage if phase != Playing in crates/plix-server/src/lib.rs (simulate_tick only called in Playing)
- [x] T071 [US5] Gate block edit processing: return InvalidPhase if phase != Playing in crates/plix-server/src/lib.rs (block handler)
- [x] T072 [US5] Ensure movement input processing works in all phases (no gating) in crates/plix-server/src/lib.rs
- [x] T073 [US5] Verify tests pass: cargo test -p plix-server -- phase gate lobby

**Checkpoint**: User Story 5 complete - phase restrictions enforced

---

## Phase 8: User Story 6 - Late Joiner Handling (Priority: P3)

**Goal**: Players joining mid-match wait until next Lobby

**Independent Test**: Start a match, have a new player connect mid-game, verify they cannot participate until next match.

### Tests for User Story 6

- [ ] T074 [P] [US6] Unit test: late joiner during Playing cannot spawn in crates/plix-server/src/lib.rs (tests) - DEFERRED P3
- [x] T075 [P] [US6] Unit test: late joiner receives current match state in crates/plix-server/src/lib.rs (tests) - MatchState sent in snapshots
- [ ] T076 [P] [US6] Unit test: late joiner included in next match after reset in crates/plix-server/src/lib.rs (tests) - DEFERRED P3

### Implementation for User Story 6

- [ ] T077 [US6] Track is_late_joiner flag on ServerPlayer in crates/plix-server/src/session.rs - DEFERRED P3
- [ ] T078 [US6] Set is_late_joiner=true for players connecting during Playing/EndScreen in crates/plix-server/src/lib.rs - DEFERRED P3
- [ ] T079 [US6] Skip spawning late joiners during active match in crates/plix-server/src/lib.rs - DEFERRED P3
- [x] T080 [US6] Send current MatchState to late joiners on connect in crates/plix-server/src/lib.rs - MatchState in WorldSnapshot
- [ ] T081 [US6] Clear is_late_joiner on reset so they join next match in crates/plix-server/src/session.rs - DEFERRED P3
- [ ] T082 [US6] Verify tests pass: cargo test -p plix-server -- late join - DEFERRED P3

**Checkpoint**: User Story 6 complete - late joiners handled correctly

---

## Phase 9: Client UX (Native, No CEF)

**Purpose**: Client-side display and input for match flow

- [ ] T083 [P] Track MatchState on client (phase, countdown, time, scores) in crates/plix-client/src/state.rs
- [ ] T084 [P] Handle MatchState updates from WorldSnapshot in crates/plix-client/src/net.rs
- [ ] T085 Add Ready toggle keybind (R key) in crates/plix-client/src/input.rs
- [ ] T086 Send ReadyToggle message on R key press (only in Lobby) in crates/plix-client/src/main.rs
- [ ] T087 [P] Gate attack/block edit inputs: suppress unless phase==Playing in crates/plix-client/src/main.rs
- [ ] T088 Create match_hud.rs with ready indicator, countdown display, scoreboard in crates/plix-client/src/ui/match_hud.rs
- [ ] T089 Create end_screen.rs with final scores and winner display in crates/plix-client/src/ui/end_screen.rs
- [ ] T090 Integrate match HUD rendering into main render loop in crates/plix-client/src/main.rs
- [ ] T091 Display countdown overlay (3... 2... 1...) during Countdown phase in crates/plix-client/src/ui/match_hud.rs
- [ ] T092 Display EndScreen with winner/draw and scores for end_screen duration in crates/plix-client/src/ui/end_screen.rs

**Checkpoint**: Client UX complete - players can see and interact with match flow

---

## Phase 10: Validation & Regression

**Purpose**: Verify all features work together and existing functionality preserved

### Manual E2E Testing

- [ ] T093 Manual test: ready → countdown → play → end → reset (2 clients, no server restart)
- [ ] T094 Manual test: arena rotation (configure 2 arenas, verify switch)
- [ ] T095 Manual test: late joiner during Playing (verify spectator state)

### Regression Testing

- [ ] T096 Headless client regression: cargo run -p plix-client -- --headless --server 127.0.0.1:7777
- [ ] T097 Load test regression: ./scripts/run_load_test.sh 8 30 127.0.0.1:7777 (if script exists)
- [ ] T098 All tests green: cargo test --workspace
- [ ] T099 Clippy clean: cargo clippy --all-targets
- [ ] T100 Format check: cargo fmt --all -- --check

---

## Phase 11: Polish & Optional

**Purpose**: Quality of life improvements

- [ ] T101 [P] Bot auto-ready for load tests in crates/plix-tools/src/bot.rs (if bot exists)
- [ ] T102 [P] Add match flow logging with tracing in crates/plix-server/src/match_state.rs
- [ ] T103 Update CLAUDE.md with match flow commands/testing info in CLAUDE.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **Phase 3-8 (User Stories)**: All depend on Phase 2 completion
  - US1 (P1), US2 (P1), US3 (P1): Core MVP - complete in order
  - US4 (P2), US5 (P2): Can start after US3 complete
  - US6 (P3): Can start after US3 complete
- **Phase 9 (Client UX)**: Can proceed in parallel with Phase 3-8 after Phase 2
- **Phase 10 (Validation)**: Depends on all implementation phases
- **Phase 11 (Polish)**: After validation passes

### User Story Dependencies

| Story | Priority | Depends On | Can Start After |
|-------|----------|------------|-----------------|
| US1 | P1 | Phase 2 | Phase 2 complete |
| US2 | P1 | US1 | US1 complete (needs Playing phase) |
| US3 | P1 | US2 | US2 complete (needs EndScreen) |
| US4 | P2 | US3 | US3 complete (needs reset flow) |
| US5 | P2 | Phase 2 | Phase 2 complete (parallel with US1) |
| US6 | P3 | US3 | US3 complete (needs match cycle) |

### Parallel Opportunities

**Within Phase 1**:
```bash
# All protocol types can be added in parallel:
T001 T002 T003 T006 T007 T009  # Different structs/enums
```

**Within Each User Story**:
```bash
# Tests can run in parallel:
T020 T021 T022 T023  # US1 tests
T034 T035 T036 T037  # US2 tests
```

**Phase 9 (Client) parallel with Phase 3-8 (Server)**:
```bash
# Server work:
T024-T033  # US1 server implementation

# Client work (parallel):
T083-T092  # Client UX (after Phase 2)
```

---

## Implementation Strategy

### MVP First (User Stories 1-3)

1. Complete Phase 1: Setup (protocol)
2. Complete Phase 2: Foundational (state machine)
3. Complete Phase 3: US1 (ready → countdown → playing)
4. Complete Phase 4: US2 (scoring, match end)
5. Complete Phase 5: US3 (end screen, reset)
6. **STOP and VALIDATE**: Full match cycle works
7. Deploy/demo MVP

### Incremental Delivery

1. MVP (US1+US2+US3) → Functional match cycle
2. Add US4 (Arena Rotation) → Map variety
3. Add US5 (Phase Restrictions) → Fair play
4. Add US6 (Late Joiners) → Server continuity
5. Each story adds value without breaking previous

---

## Summary

| Phase | Tasks | Purpose |
|-------|-------|---------|
| 1. Setup | T001-T012 | Protocol types |
| 2. Foundational | T013-T019 | Server state machine |
| 3. US1 (P1) | T020-T033 | Ready/countdown/start |
| 4. US2 (P1) | T034-T046 | Scoring/match end |
| 5. US3 (P1) | T047-T057 | End screen/reset |
| 6. US4 (P2) | T058-T066 | Arena rotation |
| 7. US5 (P2) | T067-T073 | Phase restrictions |
| 8. US6 (P3) | T074-T082 | Late joiners |
| 9. Client | T083-T092 | Client UX |
| 10. Validation | T093-T100 | Testing/regression |
| 11. Polish | T101-T103 | Optional improvements |

**Total Tasks**: 103
**MVP Tasks**: T001-T057 (57 tasks for US1+US2+US3)
**Parallel Opportunities**: 35+ tasks marked [P]
