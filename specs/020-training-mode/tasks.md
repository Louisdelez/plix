# Tasks: Training Mode

**Input**: Design documents from `/specs/020-training-mode/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Unit tests are required per constitution (Code Quality principle)

**Organization**: Tasks grouped by user story for independent implementation and testing

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US6)
- All paths relative to repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Protocol extensions and module skeleton required by all user stories

- [ ] T001 [P] Add `GameMode::Training` variant to enum in `crates/plix-common/src/types.rs`
- [ ] T002 [P] Add `BotId` type to `crates/plix-common/src/types.rs`
- [ ] T003 [P] Add `ClientMessage::TrainingReset` and `ClientMessage::TrainingStatsRequest` to `crates/plix-common/src/protocol/messages.rs`
- [ ] T004 [P] Add `ServerMessage::TrainingStats` to `crates/plix-common/src/protocol/messages.rs`
- [ ] T005 [P] Add `GameEvent::TrainingReset`, `GameEvent::BotHit`, `GameEvent::BotRespawned` to `crates/plix-common/src/protocol/messages.rs`
- [ ] T006 [P] Add `BotSnapshot` struct and `bots: Vec<BotSnapshot>` field to `WorldSnapshot` in `crates/plix-common/src/protocol/messages.rs`
- [ ] T007 Create training module skeleton in `crates/plix-server/src/training/mod.rs` with submodule declarations
- [ ] T008 Add `pub mod training;` to `crates/plix-server/src/lib.rs`
- [ ] T009 [P] Add serde test for `GameMode::Training` parsing in `crates/plix-common/src/types.rs` (test_game_mode_training_serde)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core training infrastructure that MUST be complete before user stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T010 Implement `TrainingConfig` struct with defaults and validation in `crates/plix-server/src/training/config.rs`
- [ ] T011 [P] Implement `BotBehaviorType` enum (Dummy/Roam/Strafe) in `crates/plix-server/src/training/config.rs`
- [ ] T012 Implement `TrainingBot` struct with spawn/respawn methods in `crates/plix-server/src/training/bot.rs`
- [ ] T013 Implement `BotBehavior` enum with update() method in `crates/plix-server/src/training/bot.rs`
- [ ] T014 Implement `TrainingStats` struct with accuracy/duration methods in `crates/plix-server/src/training/stats.rs`
- [ ] T015 Implement `TrainingCoordinator` struct skeleton with new/tick/reset in `crates/plix-server/src/training/coordinator.rs`
- [ ] T016 Add `MatchConfig::training_default()` factory method in `crates/plix-server/src/match_state.rs`
- [ ] T017 [P] Add `TrainingArenaConfig` struct to `crates/plix-arena/src/format.rs`
- [ ] T018 Create `assets/arenas/training_arena.toml` with game_mode = "training"
- [ ] T019 [P] Unit test: TrainingConfig defaults and validation in `crates/plix-server/src/training/config.rs`
- [ ] T020 [P] Unit test: BotBehavior tick-safety (no panic) in `crates/plix-server/src/training/bot.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Practice Aiming on Target Bots (Priority: P1) 🎯 MVP

**Goal**: Players can spawn into training arena with bots, hit bots, and see bots respawn

**Independent Test**: Start server with training_arena, connect client, attack bots, verify hits register and bots respawn

### Tests for User Story 1

- [ ] T021 [P] [US1] Unit test: bot spawn count matches config in `crates/plix-server/tests/training_bot_test.rs`
- [ ] T022 [P] [US1] Unit test: bot respawn after delay in `crates/plix-server/tests/training_bot_test.rs`
- [ ] T023 [P] [US1] Unit test: bot hit registers damage in `crates/plix-server/tests/training_bot_test.rs`
- [ ] T024 [P] [US1] Unit test: invincible bot takes no damage but hit counted in `crates/plix-server/tests/training_bot_test.rs`

### Implementation for User Story 1

- [ ] T025 [US1] Implement TrainingCoordinator.spawn_all_bots() in `crates/plix-server/src/training/coordinator.rs`
- [ ] T026 [US1] Implement TrainingCoordinator.tick() with bot behavior updates in `crates/plix-server/src/training/coordinator.rs`
- [ ] T027 [US1] Implement TrainingCoordinator.process_hit() with damage/kill logic in `crates/plix-server/src/training/coordinator.rs`
- [ ] T028 [US1] Implement bot respawn timer checking in tick() in `crates/plix-server/src/training/coordinator.rs`
- [ ] T029 [US1] Implement TrainingCoordinator.bot_snapshots() for replication in `crates/plix-server/src/training/coordinator.rs`
- [ ] T030 [US1] Add training_coordinator field to Server struct in `crates/plix-server/src/lib.rs`
- [ ] T031 [US1] Initialize TrainingCoordinator when game_mode == Training in Server::new() in `crates/plix-server/src/lib.rs`
- [ ] T032 [US1] Call training_coordinator.tick() in Server::tick() when training mode active in `crates/plix-server/src/lib.rs`
- [ ] T033 [US1] Include bots in combat target list for hit detection in `crates/plix-server/src/lib.rs`
- [ ] T034 [US1] Include bot_snapshots in WorldSnapshot when training mode in `crates/plix-server/src/lib.rs`
- [ ] T035 [US1] Send GameEvent::BotHit to attacker on bot hit in `crates/plix-server/src/lib.rs`
- [ ] T036 [US1] Broadcast GameEvent::BotRespawned when bot respawns in `crates/plix-server/src/lib.rs`
- [ ] T037 [US1] Add tracing logs for bot spawn/death events in `crates/plix-server/src/training/coordinator.rs`

**Checkpoint**: User Story 1 complete - bots spawn, take hits, respawn

---

## Phase 4: User Story 2 - Quick Warmup Session (Priority: P1)

**Goal**: Players spawn immediately, respawn quickly after death, no victory condition

**Independent Test**: Join training, die, verify respawn in ~1 second, verify no match end

### Tests for User Story 2

- [ ] T038 [P] [US2] Unit test: player spawns immediately on join in `crates/plix-server/tests/training_reset_test.rs`
- [ ] T039 [P] [US2] Unit test: player respawn delay matches config in `crates/plix-server/tests/training_reset_test.rs`
- [ ] T040 [P] [US2] Unit test: no victory condition triggers in training mode in `crates/plix-server/tests/training_reset_test.rs`

### Implementation for User Story 2

- [ ] T041 [US2] Skip countdown phase and go directly to Playing in training mode in `crates/plix-server/src/match_state.rs`
- [ ] T042 [US2] Skip score limit and time limit checks when game_mode == Training in `crates/plix-server/src/lib.rs`
- [ ] T043 [US2] Use TrainingConfig.player_respawn_delay_ticks for player respawn in `crates/plix-server/src/lib.rs`
- [ ] T044 [US2] Implement player invincibility check when invincibility_player = true in `crates/plix-server/src/lib.rs`

**Checkpoint**: User Story 2 complete - quick warmup with fast respawn, no match end

---

## Phase 5: User Story 3 - Reset Training Session (Priority: P2)

**Goal**: Player can press key to reset session (position, stats, bots)

**Independent Test**: Play training, accumulate stats/bot positions, press reset key, verify all reset

### Tests for User Story 3

- [ ] T045 [P] [US3] Unit test: reset repositions player to spawn in `crates/plix-server/tests/training_reset_test.rs`
- [ ] T046 [P] [US3] Unit test: reset clears stats in `crates/plix-server/tests/training_reset_test.rs`
- [ ] T047 [P] [US3] Unit test: reset respawns all bots at initial positions in `crates/plix-server/tests/training_reset_test.rs`

### Implementation for User Story 3

- [ ] T048 [US3] Implement TrainingCoordinator.reset() method in `crates/plix-server/src/training/coordinator.rs`
- [ ] T049 [US3] Handle ClientMessage::TrainingReset in Server::handle_message() in `crates/plix-server/src/lib.rs`
- [ ] T050 [US3] Reposition player to spawn point on reset in `crates/plix-server/src/lib.rs`
- [ ] T051 [US3] Broadcast GameEvent::TrainingReset after reset completes in `crates/plix-server/src/lib.rs`
- [ ] T052 [US3] Add rate limiting for reset requests (once per second) in `crates/plix-server/src/lib.rs`

**Checkpoint**: User Story 3 complete - session reset works via keyboard

---

## Phase 6: User Story 4 - View Training Statistics (Priority: P2)

**Goal**: Player can press key to see hits, kills, accuracy, duration in console

**Independent Test**: Play training, hit/kill bots, press stats key, verify console output

### Tests for User Story 4

- [ ] T053 [P] [US4] Unit test: stats track hits correctly in `crates/plix-server/tests/training_stats_test.rs`
- [ ] T054 [P] [US4] Unit test: stats track kills correctly in `crates/plix-server/tests/training_stats_test.rs`
- [ ] T055 [P] [US4] Unit test: accuracy calculation correct in `crates/plix-server/tests/training_stats_test.rs`
- [ ] T056 [P] [US4] Unit test: stats request does not modify state in `crates/plix-server/tests/training_stats_test.rs`

### Implementation for User Story 4

- [ ] T057 [US4] Call stats.record_hit() on bot hit in TrainingCoordinator.process_hit() in `crates/plix-server/src/training/coordinator.rs`
- [ ] T058 [US4] Call stats.record_kill() on bot elimination in TrainingCoordinator.process_hit() in `crates/plix-server/src/training/coordinator.rs`
- [ ] T059 [US4] Call stats.record_attack() on player attack input in `crates/plix-server/src/lib.rs`
- [ ] T060 [US4] Handle ClientMessage::TrainingStatsRequest in Server::handle_message() in `crates/plix-server/src/lib.rs`
- [ ] T061 [US4] Log stats with tracing::info!() on stats request in `crates/plix-server/src/lib.rs`
- [ ] T062 [US4] Send ServerMessage::TrainingStats response to requester in `crates/plix-server/src/lib.rs`

**Checkpoint**: User Story 4 complete - stats tracking and display works

---

## Phase 7: User Story 5 - Configure Bot Behavior (Priority: P3)

**Goal**: Server admin can configure bots as dummy/roam/strafe via arena config

**Independent Test**: Set bot_behavior in arena TOML, verify bots move (or don't) accordingly

### Tests for User Story 5

- [ ] T063 [P] [US5] Unit test: dummy behavior keeps bot stationary in `crates/plix-server/tests/training_bot_test.rs`
- [ ] T064 [P] [US5] Unit test: roam behavior moves bot within radius in `crates/plix-server/tests/training_bot_test.rs`
- [ ] T065 [P] [US5] Unit test: strafe behavior oscillates bot position in `crates/plix-server/tests/training_bot_test.rs`

### Implementation for User Story 5

- [ ] T066 [US5] Implement BotBehavior::Dummy update (return same position) in `crates/plix-server/src/training/bot.rs`
- [ ] T067 [US5] Implement BotBehavior::Roam update with direction changes in `crates/plix-server/src/training/bot.rs`
- [ ] T068 [US5] Implement BotBehavior::Strafe update with sin oscillation in `crates/plix-server/src/training/bot.rs`
- [ ] T069 [US5] Load bot_behavior from TrainingArenaConfig during coordinator init in `crates/plix-server/src/lib.rs`

**Checkpoint**: User Story 5 complete - bot behaviors work as configured

---

## Phase 8: User Story 6 - Invincibility Options (Priority: P3)

**Goal**: Player can enable invincibility for themselves to practice without dying

**Independent Test**: Enable invincibility_player, take damage, verify health unchanged

### Tests for User Story 6

- [ ] T070 [P] [US6] Unit test: invincible player takes no damage in `crates/plix-server/tests/training_reset_test.rs`
- [ ] T071 [P] [US6] Unit test: non-invincible player takes normal damage in `crates/plix-server/tests/training_reset_test.rs`

### Implementation for User Story 6

- [ ] T072 [US6] Check invincibility_player before applying damage to player in `crates/plix-server/src/lib.rs`
- [ ] T073 [US6] Load invincibility_player from TrainingArenaConfig in `crates/plix-server/src/lib.rs`
- [ ] T074 [US6] Load invincibility_bots from TrainingArenaConfig in `crates/plix-server/src/lib.rs`

**Checkpoint**: User Story 6 complete - invincibility options work

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Integration testing, observability, documentation, non-regression

- [ ] T075 Integration test: complete training session flow in `crates/plix-server/tests/training_integration_test.rs`
- [ ] T076 [P] Add training metrics to ServerMetricsCollector (bots_active, hits_total, kills_total) in `crates/plix-server/src/metrics.rs`
- [ ] T077 [P] Non-regression test: TDM mode still works unchanged in `crates/plix-server/tests/match_state_test.rs`
- [ ] T078 [P] Non-regression test: FFA mode still works unchanged in `crates/plix-server/tests/match_state_test.rs`
- [ ] T079 [P] Non-regression test: CTF mode still works unchanged in `crates/plix-server/tests/ctf_capture_test.rs`
- [ ] T080 [P] Non-regression test: BR Lite mode still works unchanged in `crates/plix-server/tests/br_zone_test.rs`
- [ ] T081 Handle player disconnect: clear training session state in `crates/plix-server/src/lib.rs`
- [ ] T082 Run cargo fmt --all && cargo clippy --all-targets and fix warnings
- [ ] T083 Run full test suite: cargo test -p plix-server

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion
  - US1 and US2 can proceed in parallel after Foundational
  - US3 depends on US1 (stats collection)
  - US4 depends on US1 (stats tracking)
  - US5 can proceed independently after Foundational
  - US6 can proceed independently after Foundational
- **Polish (Phase 9)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Practice Aiming - No dependencies, core MVP
- **US2 (P1)**: Quick Warmup - No dependencies on other stories, pairs with US1 for MVP
- **US3 (P2)**: Reset Session - Depends on US1 for stats reset functionality
- **US4 (P2)**: View Statistics - Depends on US1 for stats collection
- **US5 (P3)**: Configure Bot Behavior - Independent, enhances US1
- **US6 (P3)**: Invincibility Options - Independent, enhances US2

### Within Each User Story

- Tests written first, verify they FAIL before implementation
- Entity structs before methods
- Core logic before server integration
- Integration before events/replication

### Parallel Opportunities

- All Setup tasks T001-T009 marked [P] can run in parallel
- T010, T011, T017 can run in parallel (different files)
- T019, T020 tests can run in parallel
- All US1 tests (T021-T024) can run in parallel
- US1 and US2 can be developed in parallel after Foundational
- US5 and US6 can be developed in parallel
- All Polish tests (T077-T080) can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Unit test: bot spawn count matches config in crates/plix-server/tests/training_bot_test.rs"
Task: "Unit test: bot respawn after delay in crates/plix-server/tests/training_bot_test.rs"
Task: "Unit test: bot hit registers damage in crates/plix-server/tests/training_bot_test.rs"
Task: "Unit test: invincible bot takes no damage but hit counted in crates/plix-server/tests/training_bot_test.rs"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (protocol extensions)
2. Complete Phase 2: Foundational (core structs + coordinator skeleton)
3. Complete Phase 3: User Story 1 (bots spawn, hit, respawn)
4. Complete Phase 4: User Story 2 (fast player respawn, no victory)
5. **STOP and VALIDATE**: Test MVP independently with training_arena
6. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add US1 + US2 → Test independently → **MVP Complete!**
3. Add US3 → Test reset → Deploy/Demo
4. Add US4 → Test stats → Deploy/Demo
5. Add US5 → Test behaviors → Deploy/Demo
6. Add US6 → Test invincibility → Deploy/Demo
7. Each story adds value without breaking previous stories

### Suggested MVP Scope

**US1 + US2** provide core training functionality:
- Bots spawn in arena
- Player can attack bots
- Bots respawn after elimination
- Player respawns quickly
- No match end condition

This is sufficient for basic aim practice - the primary value proposition.

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Constitution requires tests: Code Quality principle (V)
