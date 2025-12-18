# Tasks: FFA Arena Mode

**Input**: Design documents from `/specs/017-ffa-arena/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US6)
- Include exact file paths in descriptions

## Path Conventions

```text
crates/
├── plix-common/src/          # Shared types, protocol
├── plix-arena/src/           # Arena loading, spawns
├── plix-server/src/          # Server logic, match state
└── plix-client/src/          # Client (minimal changes)

assets/arenas/                # Arena TOML files
```

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add GameMode type and arena config field that all user stories depend on

- [x] T001 [P] Add GameMode enum (Tdm, Ffa) with serde support in crates/plix-common/src/types.rs
- [x] T002 [P] Add game_mode field to ArenaMetadata struct in crates/plix-arena/src/format.rs
- [x] T003 Add game_mode field to MatchState in crates/plix-common/src/protocol/messages.rs
- [x] T004 Add MatchConfig::ffa_default() constructor in crates/plix-server/src/match_state.rs
- [x] T005 Update MatchStateMachine::new() to accept and store GameMode in crates/plix-server/src/match_state.rs
- [x] T006 Run cargo build and cargo clippy to verify compilation

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Game mode detection and server branching that MUST complete before user stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T007 Implement game mode detection on arena load in crates/plix-server/src/lib.rs
- [x] T008 Pass game_mode from loaded arena to MatchStateMachine initialization in crates/plix-server/src/lib.rs
- [x] T009 Add game_mode logging on match initialization in crates/plix-server/src/lib.rs
- [x] T010 [P] Create FFA example arena at assets/arenas/ffa_arena.toml with game_mode = "ffa"
- [x] T011 [P] Update existing test_arena.toml to explicitly set game_mode = "tdm" in assets/arenas/test_arena.toml
- [x] T012 Add unit test: arena with game_mode="ffa" loads correctly in crates/plix-arena/src/loader.rs
- [x] T013 Add unit test: arena without game_mode defaults to Tdm in crates/plix-arena/src/loader.rs
- [x] T014 Run cargo test to verify foundational setup

**Checkpoint**: Foundation ready - FFA/TDM mode is detectable from arena config

---

## Phase 3: User Story 1 - Individual Kill Scoring (Priority: P1) 🎯 MVP

**Goal**: Players earn points by eliminating others - core FFA gameplay loop

**Independent Test**: Kill another player → verify killer's score increases by 1 → score broadcast to all clients

### Implementation for User Story 1

- [ ] T015 [US1] Add FFA scoring branch in kill processing - if game_mode==Ffa, call update_player_score instead of award_team_kill in crates/plix-server/src/lib.rs
- [ ] T016 [US1] Ensure update_player_score increments kills by 1 for attacker in crates/plix-server/src/match_state.rs
- [ ] T017 [US1] Add suicide check - if attacker_id == victim_id, skip scoring in crates/plix-server/src/lib.rs
- [ ] T018 [US1] Log FFA kill events with killer and victim IDs in crates/plix-server/src/lib.rs
- [ ] T019 [US1] Add unit test: FFA kill increments attacker score by 1 in crates/plix-server/src/match_state.rs
- [ ] T020 [US1] Add unit test: FFA suicide does not award points in crates/plix-server/src/match_state.rs
- [ ] T021 [US1] Add unit test: FFA scoring only occurs in Playing phase in crates/plix-server/src/match_state.rs

**Checkpoint**: FFA individual scoring works - kill = +1 point to attacker

---

## Phase 4: User Story 2 - FFA Respawn System (Priority: P1)

**Goal**: Eliminated players respawn after delay at neutral spawn points

**Independent Test**: Player dies → waits respawn_delay → spawns at neutral spawn point with full health

### Implementation for User Story 2

- [ ] T022 [US2] Implement FFA spawn selection - select from all spawns ignoring team field in crates/plix-arena/src/spawn.rs
- [ ] T023 [US2] Add get_ffa_spawn() method to SpawnManager that returns any available spawn in crates/plix-arena/src/spawn.rs
- [ ] T024 [US2] Update respawn logic to use get_ffa_spawn() when game_mode==Ffa in crates/plix-server/src/lib.rs
- [ ] T025 [US2] Ensure dead state is set on player death (reuse existing) in crates/plix-server/src/lib.rs
- [ ] T026 [US2] Ensure respawn clears dead state and spectate_target (reuse existing) in crates/plix-server/src/lib.rs
- [ ] T027 [US2] Add unit test: FFA spawn selection returns valid spawn in crates/plix-arena/src/spawn.rs
- [ ] T028 [US2] Add unit test: respawn resets health to 100 in crates/plix-server/src/lib.rs

**Checkpoint**: FFA respawn works - players respawn at neutral spawns after delay

---

## Phase 5: User Story 3 - Match End on Score Limit (Priority: P1)

**Goal**: Match ends when a player reaches score_limit, declaring individual winner

**Independent Test**: Player reaches score_limit → match transitions to EndScreen → winner declared

### Implementation for User Story 3

- [ ] T029 [US3] Call check_score_limit() after FFA kill scoring in crates/plix-server/src/lib.rs
- [ ] T030 [US3] Ensure check_score_limit() sets winner to PlayerId when limit reached in crates/plix-server/src/match_state.rs
- [ ] T031 [US3] Ensure team_winner is None for FFA matches in crates/plix-server/src/match_state.rs
- [ ] T032 [US3] Log match end with winner PlayerId in crates/plix-server/src/lib.rs
- [ ] T033 [US3] Add unit test: FFA score_limit reached ends match with correct winner in crates/plix-server/src/match_state.rs
- [ ] T034 [US3] Add unit test: FFA match end sets phase to EndScreen in crates/plix-server/src/match_state.rs

**Checkpoint**: FFA victory works - first to score_limit wins

---

## Phase 6: User Story 4 - Match State Transitions (Priority: P2)

**Goal**: Proper phase flow: Lobby → Countdown → Playing → EndScreen → Resetting → Lobby

**Independent Test**: Server starts → Lobby → countdown → Playing → score_limit → EndScreen → auto-reset to Lobby

### Implementation for User Story 4

- [ ] T035 [US4] Verify FFA match initializes in Lobby phase (reuse existing) in crates/plix-server/src/match_state.rs
- [ ] T036 [US4] Verify Countdown → Playing transition works for FFA (reuse existing) in crates/plix-server/src/match_state.rs
- [ ] T037 [US4] Implement EndScreen → Resetting → Lobby auto-reset for FFA in crates/plix-server/src/match_state.rs
- [ ] T038 [US4] Reset all player scores to 0 on match reset in crates/plix-server/src/match_state.rs
- [ ] T039 [US4] Clear winner on match reset in crates/plix-server/src/match_state.rs
- [ ] T040 [US4] Log phase transitions in crates/plix-server/src/match_state.rs
- [ ] T041 [US4] Add unit test: FFA match resets scores after EndScreen in crates/plix-server/src/match_state.rs
- [ ] T042 [US4] Add unit test: FFA match returns to Lobby after reset in crates/plix-server/src/match_state.rs

**Checkpoint**: Complete FFA match cycle works with auto-reset

---

## Phase 7: User Story 5 - FFA Configuration (Priority: P2)

**Goal**: Configurable score_limit, respawn_delay, end_screen_delay with FFA defaults

**Independent Test**: Start server with custom config values → verify match uses those values

### Implementation for User Story 5

- [ ] T043 [US5] Apply FFA defaults: score_limit=15, respawn_delay=180 ticks, end_screen=600 ticks in crates/plix-server/src/match_state.rs
- [ ] T044 [US5] Support config overrides from arena file in crates/plix-arena/src/format.rs
- [ ] T045 [US5] Add optional score_limit, respawn_delay fields to ArenaMetadata in crates/plix-arena/src/format.rs
- [ ] T046 [US5] Merge arena overrides into MatchConfig on initialization in crates/plix-server/src/lib.rs
- [ ] T047 [US5] Add unit test: FFA defaults are applied when no override in crates/plix-server/src/match_state.rs
- [ ] T048 [US5] Add unit test: arena config overrides FFA defaults in crates/plix-server/src/match_state.rs

**Checkpoint**: FFA configuration is customizable with sensible defaults

---

## Phase 8: User Story 6 - FFA Observability (Priority: P3)

**Goal**: Server operators can monitor match state, scores, and winner

**Independent Test**: Query server state → see match phase, leader scores, winner if applicable

### Implementation for User Story 6

- [ ] T049 [US6] Ensure game_mode is included in MatchState broadcast in crates/plix-common/src/protocol/messages.rs
- [ ] T050 [US6] Add leader score tracking (highest kills player) in crates/plix-server/src/match_state.rs
- [ ] T051 [US6] Add get_leader() method returning PlayerId with highest score in crates/plix-server/src/match_state.rs
- [ ] T052 [US6] Log match start event with game_mode and score_limit in crates/plix-server/src/lib.rs
- [ ] T053 [US6] Log match reset event in crates/plix-server/src/lib.rs
- [ ] T054 [US6] Verify no per-tick logging (only event-based) in crates/plix-server/src/lib.rs
- [ ] T055 [US6] Add unit test: get_leader() returns correct player in crates/plix-server/src/match_state.rs

**Checkpoint**: FFA match state is observable and debuggable

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Non-regression, documentation, and validation

### TDM Non-Regression

- [ ] T056 [P] Add unit test: TDM scoring still uses team scoring (not FFA individual) in crates/plix-server/src/match_state.rs
- [ ] T057 [P] Add unit test: TDM arena loads with game_mode=Tdm or default in crates/plix-arena/src/loader.rs
- [ ] T058 [P] Add unit test: TDM match uses team_winner not winner in crates/plix-server/src/match_state.rs

### Integration Tests

- [ ] T059 Add integration test: complete FFA match flow (lobby → playing → endscreen → reset) in crates/plix-server/tests/
- [ ] T060 Add integration test: multiple FFA kills until score_limit in crates/plix-server/tests/
- [ ] T061 Add integration test: FFA respawn after death in crates/plix-server/tests/

### Final Validation

- [ ] T062 Run cargo test --workspace to verify all tests pass
- [ ] T063 Run cargo clippy --workspace to verify no warnings
- [ ] T064 Run cargo fmt --check to verify formatting
- [ ] T065 Validate quickstart.md scenarios work end-to-end

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup)
    ↓
Phase 2 (Foundational) ─── BLOCKS ALL USER STORIES
    ↓
Phase 3-8 (User Stories) ─── Can run in priority order or parallel
    ↓
Phase 9 (Polish)
```

### User Story Dependencies

| Story | Depends On | Can Parallel With |
|-------|------------|-------------------|
| US1 (Scoring) | Phase 2 | US2, US4, US5, US6 |
| US2 (Respawn) | Phase 2 | US1, US4, US5, US6 |
| US3 (Match End) | US1 | US4, US5, US6 |
| US4 (Transitions) | US3 | US5, US6 |
| US5 (Config) | Phase 2 | US1, US2, US6 |
| US6 (Observability) | Phase 2 | US1, US2, US5 |

### Within Each User Story

1. Implementation tasks first (T0xx)
2. Unit tests after implementation
3. Story complete before marking checkpoint

### Parallel Opportunities

**Setup Phase (T001-T006)**:
```bash
# Run in parallel:
T001: Add GameMode enum in plix-common/src/types.rs
T002: Add game_mode to ArenaMetadata in plix-arena/src/format.rs
```

**User Story 1 (T015-T021)**:
```bash
# After T015-T018, tests can run in parallel:
T019: FFA kill test
T020: FFA suicide test
T021: FFA scoring phase test
```

---

## Implementation Strategy

### MVP First (User Stories 1-3 Only)

1. Complete Phase 1: Setup (T001-T006)
2. Complete Phase 2: Foundational (T007-T014)
3. Complete Phase 3: US1 - Individual Scoring (T015-T021)
4. Complete Phase 4: US2 - Respawn (T022-T028)
5. Complete Phase 5: US3 - Match End (T029-T034)
6. **STOP and VALIDATE**: Test complete FFA match flow
7. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Mode detection works
2. Add US1 → Scoring works → Test
3. Add US2 → Respawn works → Test
4. Add US3 → Victory works → **MVP Complete** 🎯
5. Add US4 → Full cycle works → Test
6. Add US5 → Configurable → Test
7. Add US6 → Observable → Test
8. Polish → Non-regression, docs → Ship

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- MVP = US1 + US2 + US3 (scoring + respawn + victory)
- FFA reuses 95% of TDM infrastructure
- Key branch point: kill processing in plix-server/src/lib.rs
- Verify tests fail before implementing
- Commit after each task or logical group
