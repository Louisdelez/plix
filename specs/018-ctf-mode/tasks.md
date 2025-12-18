# Tasks: CTF Mode (Capture The Flag)

**Input**: Design documents from `/specs/018-ctf-mode/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included per constitution requirement (V. Code Quality: Mandatory tests for flag state transitions and zone collisions)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Rust workspace**: `crates/plix-{common,server,arena,client}/src/`
- **Tests**: `crates/plix-server/tests/ctf/`
- **Assets**: `assets/arenas/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Extend existing types and project structure for CTF mode

- [ ] T001 [P] Add `GameMode::Ctf` variant to enum in `crates/plix-common/src/types.rs`
- [ ] T002 [P] Add `FlagState` enum (AtBase, Carried, Dropped) in `crates/plix-common/src/types.rs`
- [ ] T003 [P] Add `Flag` struct with team, state, base_position in `crates/plix-common/src/types.rs`
- [ ] T004 [P] Add `FlagZoneType` enum and `FlagZone` struct with AABB collision in `crates/plix-common/src/types.rs`
- [ ] T005 Add serde serialization tests for GameMode::Ctf and FlagState in `crates/plix-common/src/types.rs`
- [ ] T006 Create CTF module structure: `crates/plix-server/src/ctf/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Arena loading, zone validation, and CTF configuration that MUST be complete before ANY user story

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Arena CTF Zone Support

- [ ] T007 [P] Add `CtfArenaConfig` struct in `crates/plix-arena/src/format.rs`
- [ ] T008 [P] Add `CtfZoneDef` struct for zone parsing in `crates/plix-arena/src/format.rs`
- [ ] T009 Add `ctf` field to `Arena` struct in `crates/plix-arena/src/format.rs`
- [ ] T010 Implement CTF zone loading from TOML in `crates/plix-arena/src/loader.rs`
- [ ] T011 Add CTF zone validation (2 flag_bases, 2 capture_zones required) in `crates/plix-arena/src/validate.rs`
- [ ] T012 Add validation error types for missing/duplicate zones in `crates/plix-arena/src/validate.rs`

### CTF State Management

- [ ] T013 [P] Add `CtfConfig` struct with defaults in `crates/plix-server/src/ctf/mod.rs`
- [ ] T014 [P] Create `CtfState` struct in `crates/plix-server/src/ctf/state.rs`
- [ ] T015 Implement `CtfState::new()` to initialize flags from zones in `crates/plix-server/src/ctf/state.rs`
- [ ] T016 Implement `CtfState::reset()` for match reset in `crates/plix-server/src/ctf/state.rs`

### Match Configuration

- [ ] T017 Add `MatchConfig::ctf_default()` with CTF-specific values in `crates/plix-server/src/match_state.rs`

### Example Arena

- [ ] T018 Create example `assets/arenas/ctf_arena.toml` with valid zones and spawn points

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Flag Capture (Priority: P1) 🎯 MVP

**Goal**: Player picks up enemy flag and returns it to base to score a point

**Independent Test**: Player enters enemy flag zone, picks up flag, returns to own capture zone, flag is captured, team scores 1 point, flags reset to bases.

### Tests for User Story 1

- [ ] T019 [P] [US1] Unit test: flag pickup valid (enemy player in flag zone) in `crates/plix-server/tests/ctf/pickup_test.rs`
- [ ] T020 [P] [US1] Unit test: flag pickup invalid (same team) in `crates/plix-server/tests/ctf/pickup_test.rs`
- [ ] T021 [P] [US1] Unit test: capture succeeds (own flag at base) in `crates/plix-server/tests/ctf/capture_test.rs`
- [ ] T022 [P] [US1] Unit test: capture blocked (own flag not at base) in `crates/plix-server/tests/ctf/capture_test.rs`
- [ ] T023 [P] [US1] Unit test: both flags reset after capture in `crates/plix-server/tests/ctf/capture_test.rs`

### Implementation for User Story 1

- [ ] T024 [P] [US1] Implement `CtfRules::can_pickup()` in `crates/plix-server/src/ctf/rules.rs`
- [ ] T025 [P] [US1] Implement `CtfRules::pickup()` state transition in `crates/plix-server/src/ctf/rules.rs`
- [ ] T026 [US1] Implement `CtfRules::can_capture()` with classic rule check in `crates/plix-server/src/ctf/rules.rs`
- [ ] T027 [US1] Implement `CtfRules::capture()` scoring and flag reset in `crates/plix-server/src/ctf/rules.rs`
- [ ] T028 [US1] Create `CtfCoordinator` with `on_player_position()` in `crates/plix-server/src/ctf/coordinator.rs`
- [ ] T029 [US1] Implement `CtfEvent` enum (FlagPickup, FlagCapture) in `crates/plix-server/src/ctf/coordinator.rs`
- [ ] T030 [US1] Integrate coordinator position checks in game loop in `crates/plix-server/src/lib.rs`

**Checkpoint**: At this point, User Story 1 should be fully functional - players can pickup and capture flags

---

## Phase 4: User Story 2 - Flag Drop on Death (Priority: P1)

**Goal**: Flag carrier death drops flag; teammates can return it, enemies can pick it up, auto-return on timer

**Independent Test**: Flag carrier is killed, flag drops at carrier's position, flag enters "dropped" state with return timer.

### Tests for User Story 2

- [ ] T031 [P] [US2] Unit test: death drops flag at carrier position in `crates/plix-server/tests/ctf/drop_test.rs`
- [ ] T032 [P] [US2] Unit test: enemy can pickup dropped flag in `crates/plix-server/tests/ctf/drop_test.rs`
- [ ] T033 [P] [US2] Unit test: teammate touch returns dropped flag in `crates/plix-server/tests/ctf/return_test.rs`
- [ ] T034 [P] [US2] Unit test: auto-return on timer expiry in `crates/plix-server/tests/ctf/return_test.rs`
- [ ] T035 [P] [US2] Unit test: disconnect drops flag (same as death) in `crates/plix-server/tests/ctf/drop_test.rs`

### Implementation for User Story 2

- [ ] T036 [P] [US2] Implement `CtfRules::drop()` with return timer in `crates/plix-server/src/ctf/rules.rs`
- [ ] T037 [P] [US2] Implement `CtfRules::can_return()` for teammate touch in `crates/plix-server/src/ctf/rules.rs`
- [ ] T038 [US2] Implement `CtfRules::return_flag()` immediate return in `crates/plix-server/src/ctf/rules.rs`
- [ ] T039 [US2] Implement `CtfRules::update_return_timers()` tick processing in `crates/plix-server/src/ctf/rules.rs`
- [ ] T040 [US2] Implement `CtfCoordinator::on_player_death()` flag drop in `crates/plix-server/src/ctf/coordinator.rs`
- [ ] T041 [US2] Implement `CtfCoordinator::on_player_disconnect()` flag drop in `crates/plix-server/src/ctf/coordinator.rs`
- [ ] T042 [US2] Implement `CtfCoordinator::tick()` for timer updates in `crates/plix-server/src/ctf/coordinator.rs`
- [ ] T043 [US2] Add `CtfEvent::FlagDrop` and `CtfEvent::FlagReturn` variants in `crates/plix-server/src/ctf/coordinator.rs`
- [ ] T044 [US2] Integrate death handler with coordinator in `crates/plix-server/src/lib.rs`
- [ ] T045 [US2] Integrate disconnect handler with coordinator in `crates/plix-server/src/lib.rs`

**Checkpoint**: At this point, flag drop, pickup of dropped flags, and return mechanics work

---

## Phase 5: User Story 3 - Match Victory (Priority: P1)

**Goal**: Match ends when team reaches capture limit; time limit produces winner or tie

**Independent Test**: Team captures flag, score increments, when capture limit is reached match ends with team declared winner.

### Tests for User Story 3

- [ ] T046 [P] [US3] Unit test: victory when capture_limit reached in `crates/plix-server/tests/ctf/victory_test.rs`
- [ ] T047 [P] [US3] Unit test: time limit with leader wins in `crates/plix-server/tests/ctf/victory_test.rs`
- [ ] T048 [P] [US3] Unit test: time limit with tie in `crates/plix-server/tests/ctf/victory_test.rs`
- [ ] T049 [P] [US3] Unit test: post-match disables capture in `crates/plix-server/tests/ctf/victory_test.rs`

### Implementation for User Story 3

- [ ] T050 [US3] Implement `CtfCoordinator::is_victory()` capture limit check in `crates/plix-server/src/ctf/coordinator.rs`
- [ ] T051 [US3] Add `CtfEvent::Victory` variant with scores in `crates/plix-server/src/ctf/coordinator.rs`
- [ ] T052 [US3] Integrate CTF victory with `MatchStateMachine` in `crates/plix-server/src/match_state.rs`
- [ ] T053 [US3] Add time limit CTF victory logic (most captures wins) in `crates/plix-server/src/match_state.rs`
- [ ] T054 [US3] Implement match reset with `CtfState::reset()` in `crates/plix-server/src/lib.rs`

**Checkpoint**: At this point, full CTF match cycle works (P1 complete)

---

## Phase 6: User Story 4 - CTF Configuration (Priority: P2)

**Goal**: Server operators can customize CTF parameters via arena TOML

**Independent Test**: Load arena with custom CTF config values, verify match uses those values instead of defaults.

### Tests for User Story 4

- [ ] T055 [P] [US4] Unit test: arena with custom capture_limit loads correctly in `crates/plix-arena/tests/ctf_config_test.rs`
- [ ] T056 [P] [US4] Unit test: arena without config uses defaults in `crates/plix-arena/tests/ctf_config_test.rs`
- [ ] T057 [P] [US4] Unit test: invalid zone config produces clear error in `crates/plix-arena/tests/ctf_config_test.rs`

### Implementation for User Story 4

- [ ] T058 [US4] Parse CTF config overrides from arena TOML in `crates/plix-arena/src/loader.rs`
- [ ] T059 [US4] Merge arena config with defaults in `CtfConfig::from_arena()` in `crates/plix-server/src/ctf/mod.rs`
- [ ] T060 [US4] Pass arena config to `CtfState::new()` in `crates/plix-server/src/lib.rs`

**Checkpoint**: At this point, arena-based CTF configuration works

---

## Phase 7: User Story 5 - Flag State Visibility (Priority: P2)

**Goal**: Clients receive flag state broadcasts for strategic decisions

**Independent Test**: Flag state changes are broadcast to all clients, clients receive flag position and carrier information.

### Tests for User Story 5

- [ ] T061 [P] [US5] Unit test: FlagUpdate serialization in `crates/plix-common/tests/protocol_test.rs`
- [ ] T062 [P] [US5] Unit test: CaptureEvent serialization in `crates/plix-common/tests/protocol_test.rs`
- [ ] T063 [P] [US5] Unit test: CtfMatchInfo in MatchState in `crates/plix-common/tests/protocol_test.rs`

### Implementation for User Story 5

- [ ] T064 [P] [US5] Add `CtfFlagUpdate` message in `crates/plix-common/src/protocol/messages.rs`
- [ ] T065 [P] [US5] Add `CtfCaptureEvent` message in `crates/plix-common/src/protocol/messages.rs`
- [ ] T066 [US5] Add `CtfMatchInfo` struct in `crates/plix-common/src/protocol/messages.rs`
- [ ] T067 [US5] Add optional `ctf: Option<CtfMatchInfo>` to `MatchState` in `crates/plix-common/src/protocol/messages.rs`
- [ ] T068 [US5] Broadcast `CtfFlagUpdate` on state changes in `crates/plix-server/src/lib.rs`
- [ ] T069 [US5] Broadcast `CtfCaptureEvent` on captures in `crates/plix-server/src/lib.rs`
- [ ] T070 [US5] Include `CtfMatchInfo` in `MatchState` broadcast in `crates/plix-server/src/lib.rs`

**Checkpoint**: At this point, clients receive complete flag state information

---

## Phase 8: User Story 6 - CTF Observability (Priority: P3)

**Goal**: Server operators see structured logs for CTF events

**Independent Test**: CTF events generate structured logs, match state is queryable.

### Implementation for User Story 6

- [ ] T071 [P] [US6] Add tracing events for flag pickup in `crates/plix-server/src/ctf/coordinator.rs`
- [ ] T072 [P] [US6] Add tracing events for flag drop in `crates/plix-server/src/ctf/coordinator.rs`
- [ ] T073 [P] [US6] Add tracing events for flag return in `crates/plix-server/src/ctf/coordinator.rs`
- [ ] T074 [P] [US6] Add tracing events for flag capture in `crates/plix-server/src/ctf/coordinator.rs`
- [ ] T075 [US6] Add CTF metrics counters (flags_picked, captures_total, etc.) in `crates/plix-server/src/ctf/mod.rs`

**Checkpoint**: At this point, all user stories are complete

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Integration tests, non-regression, and validation

### Integration Tests

- [ ] T076 [P] Integration test: full capture flow (pickup → carry → capture) in `crates/plix-server/tests/ctf/integration_test.rs`
- [ ] T077 [P] Integration test: flag state transitions (pickup → drop → return → pickup → capture) in `crates/plix-server/tests/ctf/integration_test.rs`
- [ ] T078 [P] Integration test: complete match cycle (lobby → playing → victory → reset) in `crates/plix-server/tests/ctf/integration_test.rs`

### Non-Regression Tests

- [ ] T079 [P] Non-regression: TDM mode unchanged in `crates/plix-server/tests/tdm_test.rs`
- [ ] T080 [P] Non-regression: FFA mode unchanged in `crates/plix-server/tests/ffa_test.rs`

### Edge Cases

- [ ] T081 [P] Edge case test: player cannot pickup own team's flag in `crates/plix-server/tests/ctf/edge_case_test.rs`
- [ ] T082 [P] Edge case test: player cannot carry two flags in `crates/plix-server/tests/ctf/edge_case_test.rs`
- [ ] T083 [P] Edge case test: flag out of bounds returns immediately in `crates/plix-server/tests/ctf/edge_case_test.rs`

### Final Validation

- [ ] T084 Run `cargo clippy --all-targets` and fix warnings
- [ ] T085 Run `cargo fmt --all -- --check` and fix formatting
- [ ] T086 Run `cargo test` and ensure all tests pass
- [ ] T087 Validate quickstart.md checklist items complete

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion
  - US1, US2, US3 are P1 priority and build on each other (some sequential)
  - US4, US5 are P2 priority and can start after Phase 3-5
  - US6 is P3 priority and can start after Phase 5
- **Polish (Phase 9)**: Depends on all user stories being complete

### User Story Dependencies

```
Phase 2 (Foundational)
    │
    ├──► US1 (Flag Capture) ──► US2 (Flag Drop) ──► US3 (Victory)
    │                               │                     │
    │                               └─────────────────────┤
    │                                                     │
    ├──► US4 (Configuration) ─────────────────────────────┤ (can start after Phase 2)
    │                                                     │
    ├──► US5 (Visibility) ────────────────────────────────┤ (can start after US1)
    │                                                     │
    └──► US6 (Observability) ─────────────────────────────┘ (can start after US3)
```

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Rules before coordinator integration
- Server integration last

### Parallel Opportunities

- All Setup tasks T001-T005 marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (T007-T008, T013-T014)
- Tests for each story marked [P] can run in parallel
- Protocol messages (T064-T065) can run in parallel
- Observability tasks (T071-T074) can run in parallel
- All non-regression and edge case tests can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: T019 "Unit test: flag pickup valid"
Task: T020 "Unit test: flag pickup invalid"
Task: T021 "Unit test: capture succeeds"
Task: T022 "Unit test: capture blocked"
Task: T023 "Unit test: both flags reset"

# Launch parallel implementation tasks:
Task: T024 "Implement CtfRules::can_pickup()"
Task: T025 "Implement CtfRules::pickup()"
```

---

## Implementation Strategy

### MVP First (User Stories 1-3 = P1)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 - Flag Capture
4. Complete Phase 4: User Story 2 - Flag Drop
5. Complete Phase 5: User Story 3 - Match Victory
6. **STOP and VALIDATE**: Run full match cycle test
7. Deploy/demo if ready - CTF is playable!

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add US1 (Flag Capture) → Test independently → Partial CTF works
3. Add US2 (Flag Drop) → Test independently → Full flag mechanics
4. Add US3 (Victory) → Test independently → **Complete playable CTF (MVP)**
5. Add US4 (Config) → Customizable matches
6. Add US5 (Visibility) → Clients see flag state
7. Add US6 (Observability) → Server monitoring
8. Polish phase → Production ready

---

## Task Summary

| Phase | Tasks | Parallel |
|-------|-------|----------|
| Phase 1: Setup | T001-T006 (6) | 5 [P] |
| Phase 2: Foundational | T007-T018 (12) | 4 [P] |
| Phase 3: US1 Flag Capture | T019-T030 (12) | 7 [P] |
| Phase 4: US2 Flag Drop | T031-T045 (15) | 7 [P] |
| Phase 5: US3 Victory | T046-T054 (9) | 4 [P] |
| Phase 6: US4 Config | T055-T060 (6) | 3 [P] |
| Phase 7: US5 Visibility | T061-T070 (10) | 5 [P] |
| Phase 8: US6 Observability | T071-T075 (5) | 4 [P] |
| Phase 9: Polish | T076-T087 (12) | 7 [P] |
| **Total** | **87 tasks** | **46 parallel** |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- User stories US1-US3 (P1) form the MVP
- Tests are included per constitution requirement V (Code Quality)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
