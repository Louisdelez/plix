# Tasks: BR Lite Mode (Mini Battle Royale)

**Input**: Design documents from `/specs/019-br-lite/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Organization**: Tasks are grouped by user story to enable independent implementation and testing. Tests are included as per spec Test Strategy section.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US6)
- Include exact file paths in descriptions

## Path Conventions

```text
crates/
├── plix-common/src/           # Shared types, protocol
├── plix-server/src/           # Server logic, br_lite module
├── plix-server/tests/         # Integration tests
└── plix-arena/src/            # Arena parsing
```

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, module structure, and GameMode extension

- [ ] T001 Add `GameMode::BrLite` variant to `crates/plix-common/src/types.rs`
- [ ] T002 [P] Create BR Lite module structure in `crates/plix-server/src/br_lite/mod.rs`
- [ ] T003 [P] Add BR protocol messages (BrZoneUpdate, BrLootSpawn, BrLootPickup, BrElimination, BrVictory) to `crates/plix-common/src/protocol/messages.rs`
- [ ] T004 Export br_lite module in `crates/plix-server/src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core BR Lite types and configuration that MUST be complete before ANY user story

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Configuration & Types

- [ ] T005 Create `BrLiteConfig` struct with defaults (min_players=4, bonus_duration=10s, post_match_delay) in `crates/plix-server/src/br_lite/config.rs`
- [ ] T006 [P] Add ZonePhase struct (stable_duration, shrink_duration, end_radius, damage_per_tick) in `crates/plix-server/src/br_lite/config.rs`
- [ ] T007 [P] Add default 5-phase schedule function `default_phases()` in `crates/plix-server/src/br_lite/config.rs`
- [ ] T008 [P] Add validation for config bounds (durations > 0, radii coherent, damage >= 0) in `crates/plix-server/src/br_lite/config.rs`

### Arena Parsing

- [ ] T009 Add `BrLiteArenaConfig` parsing struct in `crates/plix-arena/src/format.rs`
- [ ] T010 [P] Add `LootSpawnConfig` parsing for arena TOML in `crates/plix-arena/src/format.rs`
- [ ] T011 Add validation for loot spawn positions (within arena bounds) in `crates/plix-arena/src/format.rs`
- [ ] T012 Add unit tests for BR arena config parsing in `crates/plix-arena/src/format.rs`

### Core State Types

- [ ] T013 Create `ZoneState` struct (center, current_radius, target_radius, phase_index, phase_mode, phase_timer, damage_per_tick) in `crates/plix-server/src/br_lite/state.rs`
- [ ] T014 [P] Create `PhaseMode` enum (Stable, Shrinking) in `crates/plix-server/src/br_lite/state.rs`
- [ ] T015 [P] Create `PlayerBrState` enum (Alive, Eliminated, Spectating) in `crates/plix-server/src/br_lite/state.rs`
- [ ] T016 [P] Create `LootItem` struct (id, position, loot_type, collected) in `crates/plix-server/src/br_lite/loot.rs`
- [ ] T017 [P] Create `LootType` enum (HealthPack, SpeedBoost) in `crates/plix-server/src/br_lite/loot.rs`
- [ ] T018 [P] Create `ActiveEffect` struct (effect_type, expires_at) in `crates/plix-server/src/br_lite/loot.rs`
- [ ] T019 Ensure all state types derive Serialize/Deserialize for replication in `crates/plix-server/src/br_lite/state.rs`
- [ ] T020 Add unit tests for state type construction in `crates/plix-server/src/br_lite/state.rs`

### Match Config Extension

- [ ] T021 Add `br_lite_default()` function to MatchConfig in `crates/plix-server/src/match_state.rs`
- [ ] T022 Verify server initializes BR Lite mode when arena config has `game_mode = "br_lite"` in `crates/plix-server/src/session.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Last Player Standing Victory (Priority: P1) 🎯 MVP

**Goal**: Core BR elimination mechanics - players eliminated on death, last player wins

**Independent Test**: 2+ players join match, combat until one remains, winner declared

### Tests for User Story 1

- [ ] T023 [P] [US1] Create test file `crates/plix-server/tests/br_elimination_test.rs`
- [ ] T024 [P] [US1] Add test: elimination marks player correctly in `crates/plix-server/tests/br_elimination_test.rs`
- [ ] T025 [P] [US1] Add test: victory at alive_count == 1 in `crates/plix-server/tests/br_elimination_test.rs`
- [ ] T026 [P] [US1] Add test: simultaneous death edge case (lowest ID wins) in `crates/plix-server/tests/br_elimination_test.rs`
- [ ] T027 [P] [US1] Add test: disconnect = elimination in `crates/plix-server/tests/br_elimination_test.rs`

### Implementation for User Story 1

- [ ] T028 [US1] Create `BrLiteState` struct (alive_players, eliminated_players, winner, zone, loot, effects) in `crates/plix-server/src/br_lite/state.rs`
- [ ] T029 [US1] Implement `AliveTracker` with alive player HashSet in `crates/plix-server/src/br_lite/state.rs`
- [ ] T030 [US1] Implement `eliminate(player_id)` - mark Eliminated, decrement alive_count in `crates/plix-server/src/br_lite/state.rs`
- [ ] T031 [US1] Implement victory detection (alive_count == 1 → winner, alive_count == 0 → lowest ID wins) in `crates/plix-server/src/br_lite/state.rs`
- [ ] T032 [US1] Create `BrLiteCoordinator` struct in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T033 [US1] Implement `on_player_death()` hook - eliminate + cleanup effects + check victory in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T034 [US1] Implement `on_player_disconnect()` hook - eliminate if Alive + re-check victory in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T035 [US1] Integrate coordinator death/disconnect hooks in `crates/plix-server/src/session.rs`

**Checkpoint**: US1 complete - elimination and victory mechanics functional

---

## Phase 4: User Story 2 - Shrinking Safe Zone (Priority: P1)

**Goal**: Dynamic zone that shrinks over time, players outside take damage

**Independent Test**: Start match, observe zone shrink through phases, players outside zone take damage

### Tests for User Story 2

- [ ] T036 [P] [US2] Create test file `crates/plix-server/tests/br_zone_test.rs`
- [ ] T037 [P] [US2] Add test: linear interpolation during shrink in `crates/plix-server/tests/br_zone_test.rs`
- [ ] T038 [P] [US2] Add test: phase transitions (stable → shrink → stable) in `crates/plix-server/tests/br_zone_test.rs`
- [ ] T039 [P] [US2] Add test: determinism (same ticks = same radius) in `crates/plix-server/tests/br_zone_test.rs`
- [ ] T040 [P] [US2] Create test file `crates/plix-server/tests/br_damage_test.rs`
- [ ] T041 [P] [US2] Add test: player inside zone = 0 damage in `crates/plix-server/tests/br_damage_test.rs`
- [ ] T042 [P] [US2] Add test: player outside zone = damage applied in `crates/plix-server/tests/br_damage_test.rs`

### Implementation for User Story 2

- [ ] T043 [US2] Create `ZoneController` struct in `crates/plix-server/src/br_lite/zone.rs`
- [ ] T044 [US2] Implement `ZoneController::new()` with initial radius and phase config in `crates/plix-server/src/br_lite/zone.rs`
- [ ] T045 [US2] Implement `ZoneController::tick()` - update phase_timer, mode transitions in `crates/plix-server/src/br_lite/zone.rs`
- [ ] T046 [US2] Implement linear interpolation of radius during shrink phase in `crates/plix-server/src/br_lite/zone.rs`
- [ ] T047 [US2] Implement `is_in_zone(player_pos, zone_state)` helper function in `crates/plix-server/src/br_lite/zone.rs`
- [ ] T048 [US2] Create `DamageController` struct in `crates/plix-server/src/br_lite/damage.rs`
- [ ] T049 [US2] Implement damage tick interval (every 60 ticks = 1 second) in `crates/plix-server/src/br_lite/damage.rs`
- [ ] T050 [US2] Implement `DamageController::tick()` - check each alive player, apply damage if outside zone in `crates/plix-server/src/br_lite/damage.rs`
- [ ] T051 [US2] Integrate ZoneController and DamageController in `BrLiteCoordinator::tick()` in `crates/plix-server/src/br_lite/coordinator.rs`

**Checkpoint**: US2 complete - zone shrinking and damage functional

---

## Phase 5: User Story 3 - Zone Phase Progression (Priority: P2)

**Goal**: Clear stable/shrinking phases with increasing damage per phase

**Independent Test**: Configure multiple phases, observe transitions between stable and shrinking phases, damage increases

### Tests for User Story 3

- [ ] T052 [P] [US3] Add test: damage evolves per phase (phase 1 vs phase N) in `crates/plix-server/tests/br_damage_test.rs`
- [ ] T053 [P] [US3] Add test: phases follow configured durations in `crates/plix-server/tests/br_zone_test.rs`

### Implementation for User Story 3

- [ ] T054 [US3] Implement phase advancement in `ZoneController::tick()` - stable → shrink → next phase in `crates/plix-server/src/br_lite/zone.rs`
- [ ] T055 [US3] Associate `damage_per_tick` per phase in ZoneState in `crates/plix-server/src/br_lite/state.rs`
- [ ] T056 [US3] Update DamageController to use current phase damage in `crates/plix-server/src/br_lite/damage.rs`
- [ ] T057 [US3] Implement final phase handling (stay in final phase indefinitely) in `crates/plix-server/src/br_lite/zone.rs`

**Checkpoint**: US3 complete - phase progression with escalating damage functional

---

## Phase 6: User Story 4 - Minimal Loot Collection (Priority: P2)

**Goal**: Players can pick up health packs and speed boosts from the arena

**Independent Test**: Place loot items, player walks over them, effect applied instantly

### Tests for User Story 4

- [ ] T058 [P] [US4] Create test file `crates/plix-server/tests/br_loot_test.rs`
- [ ] T059 [P] [US4] Add test: pickup applies correct effect (health instant, speed buff) in `crates/plix-server/tests/br_loot_test.rs`
- [ ] T060 [P] [US4] Add test: loot removed after pickup in `crates/plix-server/tests/br_loot_test.rs`
- [ ] T061 [P] [US4] Add test: double pickup impossible in `crates/plix-server/tests/br_loot_test.rs`
- [ ] T062 [P] [US4] Add test: speed boost expires after 10s in `crates/plix-server/tests/br_loot_test.rs`

### Implementation for User Story 4

- [ ] T063 [US4] Create `LootManager` struct in `crates/plix-server/src/br_lite/loot.rs`
- [ ] T064 [US4] Implement `LootManager::spawn_loot()` - instantiate loot at defined positions at match start in `crates/plix-server/src/br_lite/loot.rs`
- [ ] T065 [US4] Implement `LootManager::check_pickup()` - position overlap detection (PICKUP_RADIUS = 1.0) in `crates/plix-server/src/br_lite/loot.rs`
- [ ] T066 [US4] Implement `LootManager::try_pickup()` - validate + apply effect + mark collected in `crates/plix-server/src/br_lite/loot.rs`
- [ ] T067 [US4] Implement health pack effect - instant heal in `crates/plix-server/src/br_lite/loot.rs`
- [ ] T068 [US4] Implement speed boost effect - store ActiveEffect with expires_at in `crates/plix-server/src/br_lite/loot.rs`
- [ ] T069 [US4] Implement `LootManager::tick()` - expire active effects in `crates/plix-server/src/br_lite/loot.rs`
- [ ] T070 [US4] Implement `LootManager::clear_effects(player_id)` - cleanup on elimination in `crates/plix-server/src/br_lite/loot.rs`
- [ ] T071 [US4] Integrate LootManager in BrLiteCoordinator (spawn at start, check pickup on movement, tick for expiry) in `crates/plix-server/src/br_lite/coordinator.rs`

**Checkpoint**: US4 complete - loot pickup and temporary effects functional

---

## Phase 7: User Story 5 - Match Lifecycle Management (Priority: P2)

**Goal**: Full match lifecycle with min_players gate, PostMatch, and auto-reset

**Independent Test**: Run match through all states: Lobby → InProgress → PostMatch → Reset

### Tests for User Story 5

- [ ] T072 [P] [US5] Add test: match doesn't start below min_players in `crates/plix-server/tests/br_elimination_test.rs`
- [ ] T073 [P] [US5] Add test: match starts when min_players reached in `crates/plix-server/tests/br_elimination_test.rs`
- [ ] T074 [P] [US5] Add test: lobby rollback if players leave before start in `crates/plix-server/tests/br_elimination_test.rs`
- [ ] T075 [P] [US5] Add test: reset clears all state (zone, loot, effects, players) in `crates/plix-server/tests/br_elimination_test.rs`

### Implementation for User Story 5

- [ ] T076 [US5] Implement min_players gating in lobby - stay in Warmup until min reached in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T077 [US5] Implement lobby rollback if player leaves and count drops below min in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T078 [US5] Implement `on_player_join()` hook - update alive roster, check start condition in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T079 [US5] Implement `BrLiteCoordinator::start()` - initialize zone, spawn loot at match start in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T080 [US5] Implement victory → PostMatch transition with winner broadcast in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T081 [US5] Implement PostMatch timer (post_match_delay config) in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T082 [US5] Implement `BrLiteCoordinator::reset()` - clear zone, loot, effects, player states in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T083 [US5] Implement auto-restart after reset (return to Lobby) in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T084 [US5] Integrate coordinator tick in server game loop in `crates/plix-server/src/session.rs`

**Checkpoint**: US5 complete - full match lifecycle functional

---

## Phase 8: User Story 6 - Server Observability (Priority: P3)

**Goal**: Logs and metrics for monitoring BR matches

**Independent Test**: Run match, verify logs expose phase changes, eliminations, match end

### Implementation for User Story 6

- [ ] T085 [US6] Add tracing::info! for phase change events in `crates/plix-server/src/br_lite/zone.rs`
- [ ] T086 [US6] Add tracing::info! for player elimination events in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T087 [US6] Add tracing::info! for match end with winner in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T088 [US6] Create `BrLiteDebugInfo` struct (alive_count, phase_index, radius, mode, loot_remaining, eliminations) in `crates/plix-server/src/br_lite/state.rs`
- [ ] T089 [US6] Implement `BrLiteState::debug_info()` method for server queries in `crates/plix-server/src/br_lite/state.rs`
- [ ] T090 [US6] Verify no per-tick logging (only event-driven logs) in `crates/plix-server/src/br_lite/`

**Checkpoint**: US6 complete - observability functional

---

## Phase 9: Network Replication

**Goal**: Sync BR state to clients for zone visualization and UI

### Implementation for Network Replication

- [ ] T091 Extend MatchState snapshots with BR fields (game_mode, alive_count, winner, zone_state) in `crates/plix-server/src/replication/snapshot.rs`
- [ ] T092 [P] Implement BrZoneUpdate message sending (every phase change + every 5s) in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T093 [P] Implement BrLootSpawn messages at match start in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T094 [P] Implement BrLootPickup message on pickup in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T095 [P] Implement BrElimination message on player death in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T096 Implement BrVictory message on match end in `crates/plix-server/src/br_lite/coordinator.rs`
- [ ] T097 Add unit tests for snapshot serialization in `crates/plix-server/src/replication/snapshot.rs`

---

## Phase 10: Integration Testing

**Goal**: Full match integration tests and regression tests

### Integration Tests

- [ ] T098 Create full match integration test file `crates/plix-server/tests/br_match_test.rs`
- [ ] T099 Add integration test: complete match cycle (lobby → zone shrink → eliminations → winner → reset) in `crates/plix-server/tests/br_match_test.rs`
- [ ] T100 Add integration test: loot pickup + effect expiration during match in `crates/plix-server/tests/br_match_test.rs`
- [ ] T101 Add integration test: zone damage kills player → elimination in `crates/plix-server/tests/br_match_test.rs`
- [ ] T102 Add integration test: all players disconnect → no winner edge case in `crates/plix-server/tests/br_match_test.rs`
- [ ] T103 Add integration test: validate clean state after reset in `crates/plix-server/tests/br_match_test.rs`

### Regression Tests

- [ ] T104 [P] Add regression test: TDM mode still works unchanged in `crates/plix-server/tests/`
- [ ] T105 [P] Add regression test: FFA mode still works unchanged in `crates/plix-server/tests/`
- [ ] T106 [P] Add regression test: CTF mode still works unchanged in `crates/plix-server/tests/`
- [ ] T107 Verify BR Lite doesn't impact common state machine in `crates/plix-server/src/match_state.rs`

---

## Phase 11: Polish & Documentation

**Purpose**: Final cleanup and documentation

- [ ] T108 Create BR test arena `assets/arenas/br_arena.toml` with sample phases and loot spawns
- [ ] T109 Run cargo clippy --all-targets and fix warnings in `crates/plix-server/src/br_lite/`
- [ ] T110 Run cargo fmt --all in workspace
- [ ] T111 Validate quickstart.md steps work end-to-end
- [ ] T112 Update CLAUDE.md with BR Lite feature documentation

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **US1-US6 (Phases 3-8)**: All depend on Foundational phase completion
- **Network Replication (Phase 9)**: Depends on US1, US2 completion
- **Integration Testing (Phase 10)**: Depends on all US phases complete
- **Polish (Phase 11)**: Depends on all previous phases

### User Story Dependencies

| Story | Priority | Dependencies | Can Parallelize With |
|-------|----------|--------------|---------------------|
| US1 - Elimination | P1 | Foundational only | US2 (different files) |
| US2 - Shrinking Zone | P1 | Foundational only | US1 (different files) |
| US3 - Phase Progression | P2 | US2 (zone controller) | US4 (different files) |
| US4 - Loot Collection | P2 | Foundational only | US1, US2, US3 |
| US5 - Match Lifecycle | P2 | US1 (victory), US2 (zone start) | Partial |
| US6 - Observability | P3 | US1-US5 (needs full system) | None |

### Within Each User Story

1. Tests written first and FAIL
2. State types before controllers
3. Controllers before coordinator integration
4. Core implementation before session integration

### Parallel Opportunities

- All tasks marked [P] can run in parallel within their phase
- US1 and US2 can be developed in parallel (different controllers)
- US4 (Loot) can be developed in parallel with US1-US3 (different module)
- All regression tests (T104-T106) can run in parallel

---

## Parallel Execution Examples

### Foundational Phase Parallelization

```bash
# Launch in parallel (different files):
T006: ZonePhase struct in config.rs
T007: default_phases() in config.rs
T010: LootSpawnConfig in format.rs
T013-T018: All state types in state.rs and loot.rs
```

### US1 + US2 Parallel Development

```bash
# Developer A: User Story 1 (elimination)
T023-T027: Elimination tests
T028-T035: Elimination implementation

# Developer B: User Story 2 (zone)
T036-T042: Zone and damage tests
T043-T051: Zone and damage implementation
```

---

## Implementation Strategy

### MVP First (US1 + US2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL)
3. Complete Phase 3: US1 - Elimination (parallel with US2)
4. Complete Phase 4: US2 - Shrinking Zone
5. **STOP and VALIDATE**: Core BR mechanics work
6. Deploy/demo with elimination + zone damage

### Incremental Delivery

1. MVP: US1 + US2 → Core BR loop works
2. Add US3 → Phase progression adds depth
3. Add US4 → Loot adds variety
4. Add US5 → Full lifecycle for production
5. Add US6 → Observability for operations
6. Network + Polish → Client visualization

---

## Summary

| Phase | Task Count | Parallel Tasks |
|-------|------------|----------------|
| Setup | 4 | 2 |
| Foundational | 18 | 11 |
| US1 - Elimination | 13 | 5 |
| US2 - Shrinking Zone | 16 | 7 |
| US3 - Phase Progression | 6 | 2 |
| US4 - Loot Collection | 15 | 5 |
| US5 - Match Lifecycle | 13 | 4 |
| US6 - Observability | 6 | 0 |
| Network Replication | 7 | 4 |
| Integration Testing | 10 | 3 |
| Polish | 5 | 0 |
| **Total** | **113** | **43** |

**MVP Scope**: Phases 1-4 (US1 + US2) = 51 tasks
**Full Feature**: All 113 tasks
