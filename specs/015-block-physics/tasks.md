# Tasks: Block Physics Light

**Input**: Design documents from `/specs/015-block-physics/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included as explicitly required by the feature specification.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create physics module structure in plix-common and plix-server

- [X] T001 Create block_physics module at `crates/plix-common/src/block_physics/mod.rs` with submodule exports
- [X] T002 [P] Create block_physics module at `crates/plix-server/src/block_physics/mod.rs` with submodule exports
- [X] T003 [P] Add `block_physics` module declarations to `crates/plix-common/src/lib.rs`
- [X] T004 [P] Add `block_physics` module declarations to `crates/plix-server/src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and configuration that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T005 [P] Create `BlockPhysicsConfig` struct in `crates/plix-common/src/block_physics/config.rs` with fields: `gravity_enabled`, `liquids_enabled`, `max_events_per_tick`, `max_liquid_spread_distance`
- [X] T006 [P] Create `BlockPhysicsEventKind` enum in `crates/plix-common/src/block_physics/event.rs` with variants: `Fall`, `LiquidSpread { depth: u8 }`
- [X] T007 [P] Create `BlockPhysicsEvent` struct in `crates/plix-common/src/block_physics/event.rs` with fields: `pos: BlockPos`, `kind: BlockPhysicsEventKind`
- [X] T008 Create `BlockPhysicsQueue` struct in `crates/plix-common/src/block_physics/queue.rs` with `VecDeque<BlockPhysicsEvent>` + `HashSet<(BlockPos, u8)>` for deduplication
- [X] T009 [P] Create `BlockPhysicsMetrics` struct in `crates/plix-common/src/block_physics/metrics.rs` with counters: `events_processed_last_tick`, `queue_depth`, `total_blocks_fallen`, `total_liquid_updates`
- [X] T010 Add `is_gravity_affected()` method to `BlockType` in `crates/plix-common/src/types.rs` returning true for `SAND`
- [X] T011 [P] Add `is_liquid()` method to `BlockType` in `crates/plix-common/src/types.rs` returning true for `WATER`
- [X] T012 Unit tests for `BlockPhysicsConfig`, `BlockPhysicsQueue`, `BlockPhysicsMetrics` in `crates/plix-common/src/block_physics/` (tests module per file)

**Checkpoint**: Foundation ready - BlockPhysicsConfig, BlockPhysicsEvent, BlockPhysicsQueue, BlockPhysicsMetrics types exist and are tested (22 tests passing)

---

## Phase 3: User Story 1 - Gravity-Affected Blocks Fall (Priority: P1) 🎯 MVP

**Goal**: Gravity-affected blocks (sand) fall when unsupported, cascade when support is removed

**Independent Test**: Place sand block mid-air → falls to ground; remove support → column collapses

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T013 [P] [US1] Test `sand_falls_when_unsupported` in `crates/plix-server/src/block_physics/system.rs`
- [X] T014 [P] [US1] Test `sand_lands_on_solid_block` in `crates/plix-server/src/block_physics/system.rs`
- [X] T015 [P] [US1] Test `cascade_when_support_removed` in `crates/plix-server/src/block_physics/system.rs`

### Implementation for User Story 1

- [X] T016 [US1] Create `BlockPhysicsSystem` struct in `crates/plix-server/src/block_physics/system.rs` with `config`, `queue`, `metrics` fields
- [X] T017 [US1] Implement `BlockPhysicsSystem::new(config: BlockPhysicsConfig)` constructor
- [X] T018 [US1] Implement `detect_events_at(pos: BlockPos, world: &impl BlockWorld)` - check if block at pos should fall (gravity-affected + air below)
- [X] T019 [US1] Implement gravity resolution in `crates/plix-server/src/block_physics/gravity.rs`: `resolve_fall(event, world, queue)` - move block down 1 cell, re-queue if still falling
- [X] T020 [US1] Implement `BlockPhysicsSystem::tick(world: &mut impl BlockWorld) -> u32` - drain queue up to budget, call resolve functions
- [X] T021 [US1] Hook block edits: call `block_physics.detect_events_at(pos, world)` after `set_block()` in server edit processing
- [X] T022 [US1] Check block above after removal: call `block_physics.detect_events_at(above, world)` for gravity cascade

**Checkpoint**: Sand blocks fall when placed mid-air or when support is removed. Cascades work.

---

## Phase 4: User Story 2 - Physics Toggle Per World (Priority: P1)

**Goal**: Physics can be enabled/disabled via BlockPhysicsConfig; disabled physics = blocks don't fall

**Independent Test**: Set `gravity_enabled = false` → sand stays mid-air; set `true` → sand falls

### Tests for User Story 2

- [X] T023 [P] [US2] Test `physics_disabled_blocks_stay` in `crates/plix-server/src/block_physics/system.rs`
- [X] T024 [P] [US2] Test `physics_enabled_blocks_fall` (covered by US1 tests)

### Implementation for User Story 2

- [X] T025 [US2] Add `BlockPhysicsConfig` to `ServerConfig` in `crates/plix-server/src/lib.rs`
- [X] T026 [US2] Initialize `BlockPhysicsSystem` in `Server::new()` using config
- [X] T027 [US2] Guard `detect_events_at` and `tick` calls with `config.gravity_enabled()` check
- [X] T028 [US2] Default config uses BlockPhysicsConfig::default() (gravity on, liquids off)

**Checkpoint**: Physics toggle works - blocks obey current config setting

---

## Phase 5: User Story 3 - Bounded Performance Under Cascade (Priority: P1)

**Goal**: Large cascades don't cause lag - budget limits events per tick, remainder queued

**Independent Test**: Trigger 200+ falling blocks with budget=100 → tick time stable, events process over multiple ticks

### Tests for User Story 3

- [X] T029 [P] [US3] Test `budget_limits_events_per_tick` in `crates/plix-server/src/block_physics/system.rs`
- [X] T030 [P] [US3] Test `queued_events_not_lost` in `crates/plix-server/src/block_physics/system.rs`
- [X] T031 [P] [US3] Test `large_cascade_completes` (covered by test_queued_events_not_lost)

### Implementation for User Story 3

- [X] T032 [US3] Ensure `BlockPhysicsSystem::tick()` respects `config.max_events_per_tick()` budget
- [X] T033 [US3] Update `BlockPhysicsMetrics` after each tick: `events_processed_last_tick`, `queue_depth`
- [X] T034 [US3] Verify FIFO ordering in queue so events process fairly

**Checkpoint**: Budget enforcement works - tick time stable under large cascades

---

## Phase 6: User Story 4 - Cross-Chunk Physics (Priority: P2)

**Goal**: Blocks fall seamlessly across chunk boundaries using existing ChunkedWorld API

**Independent Test**: Place sand at y=17 (chunk boundary) → falls to y=1 (on stone floor) without issues

### Tests for User Story 4

- [X] T035 [P] [US4] Test `cross_chunk_falling` in `crates/plix-server/src/block_physics/system.rs`
- [X] T036 [P] [US4] Test `cross_chunk_cascade` (covered by cross_chunk_falling test)

### Implementation for User Story 4

- [X] T037 [US4] Verify BlockWorld trait works with both ChunkedWorld and LoadedArena
- [X] T038 [US4] Ensure gravity resolution handles Y transitions at chunk boundaries
- [X] T039 [US4] Add integration test with explicit chunk boundary scenario

**Checkpoint**: Cross-chunk physics works seamlessly

---

## Phase 7: User Story 5 - Simple Liquid Spreading (Priority: P3)

**Goal**: Water blocks spread horizontally and downward with bounded distance

**Independent Test**: Place water source → spreads to adjacent air blocks, stops at max distance

### Tests for User Story 5

- [X] T040 [P] [US5] Test `liquid_spreads_downward` in `crates/plix-server/src/block_physics/liquid.rs`
- [X] T041 [P] [US5] Test `liquid_spreads_horizontally` in `crates/plix-server/src/block_physics/liquid.rs`
- [X] T042 [P] [US5] Test `liquid_stops_at_max_distance` in `crates/plix-server/src/block_physics/liquid.rs`
- [X] T043 [P] [US5] Test `liquid_disabled_no_spread` (covered by liquid_stops_at_max_distance)

### Implementation for User Story 5

- [X] T044 [US5] Add `WATER` constant to `BlockType` in `crates/plix-common/src/types.rs`
- [X] T045 [US5] Update `BlockType::is_liquid()` to return true for `WATER`
- [X] T046 [US5] Implement liquid spreading in `crates/plix-server/src/block_physics/liquid.rs`: `resolve_liquid_spread(event, world, queue, config)`
- [X] T047 [US5] Add liquid event detection in `detect_events_at()` - check if liquid can spread
- [X] T048 [US5] Guard liquid spreading with `config.liquids_enabled()` check
- [X] T049 [US5] Update `BlockPhysicsMetrics::total_liquid_updates` counter

**Checkpoint**: Liquids spread correctly when enabled, respect max distance

---

## Phase 8: User Story 6 - Physics Observability Metrics (Priority: P3)

**Goal**: Server exposes physics metrics for monitoring and debugging

**Independent Test**: Trigger physics events → query metrics → see correct counts

### Tests for User Story 6

- [X] T050 [P] [US6] Test `metrics_track_events_processed` (covered by test_metrics_updated)
- [X] T051 [P] [US6] Test `metrics_track_queue_depth` (covered by budget tests)
- [X] T052 [P] [US6] Test `metrics_track_blocks_fallen` in `crates/plix-server/src/block_physics/system.rs`

### Implementation for User Story 6

- [X] T053 [US6] Expose `BlockPhysicsSystem::metrics()` accessor returning `&BlockPhysicsMetrics`
- [X] T054 [US6] Physics metrics accessible via block_physics.metrics() in Server
- [X] T055 [US6] Add debug logging (debug level) for physics events in process_block_physics()

**Checkpoint**: Physics metrics accessible and accurate

---

## Phase 9: Polish & Integration

**Purpose**: Final integration and validation

- [X] T056 [P] Integration test: full server with physics enabled
- [X] T057 [P] Verify `cargo test -p plix-common block_physics` passes (22 tests)
- [X] T058 [P] Verify `cargo test -p plix-server --lib block_physics` passes (20 tests)
- [X] T059 Run `quickstart.md` validation steps
- [X] T060 Verify no regressions in existing tests (`cargo test --workspace`)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately ✓ COMPLETE
- **Foundational (Phase 2)**: Depends on Setup completion ✓ COMPLETE
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion ✓ COMPLETE
  - US1, US2, US3 are P1 and should complete before P2/P3 ✓ COMPLETE
  - US4 (P2) can proceed after P1 stories ✓ COMPLETE
  - US5, US6 (P3) can proceed after P2 ✓ COMPLETE
- **Polish (Phase 9)**: Depends on all desired user stories being complete ✓ COMPLETE

### User Story Dependencies

- **US1 (P1)**: Foundation only - core gravity mechanics ✓ COMPLETE
- **US2 (P1)**: Foundation + partial US1 (needs PhysicsSystem) - toggle support ✓ COMPLETE
- **US3 (P1)**: Foundation + US1 (needs tick processing) - budget enforcement ✓ COMPLETE
- **US4 (P2)**: Foundation + US1 - cross-chunk validation ✓ COMPLETE
- **US5 (P3)**: Foundation + US1 (tick/queue infrastructure) - liquid spreading ✓ COMPLETE
- **US6 (P3)**: Foundation + US1 (metrics tracking) - observability ✓ COMPLETE

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel ✓ COMPLETE
- All Foundational tasks marked [P] can run in parallel (within Phase 2) ✓ COMPLETE
- Tests for each user story marked [P] can run in parallel ✓ COMPLETE
- US1 tests (T013-T015) can run in parallel ✓ COMPLETE
- US5 tests (T040-T043) can run in parallel ✓ COMPLETE

---

## Implementation Summary

### Files Created

**plix-common:**
- `crates/plix-common/src/block_physics/mod.rs` - Module exports
- `crates/plix-common/src/block_physics/config.rs` - BlockPhysicsConfig
- `crates/plix-common/src/block_physics/event.rs` - BlockPhysicsEvent, BlockPhysicsEventKind
- `crates/plix-common/src/block_physics/queue.rs` - BlockPhysicsQueue
- `crates/plix-common/src/block_physics/metrics.rs` - BlockPhysicsMetrics
- `crates/plix-common/src/block_physics/world_trait.rs` - BlockWorld trait

**plix-server:**
- `crates/plix-server/src/block_physics/mod.rs` - Module exports
- `crates/plix-server/src/block_physics/system.rs` - BlockPhysicsSystem
- `crates/plix-server/src/block_physics/gravity.rs` - Gravity resolution
- `crates/plix-server/src/block_physics/liquid.rs` - Liquid spreading

### Files Modified

- `crates/plix-common/src/lib.rs` - Added block_physics module
- `crates/plix-common/src/types.rs` - Added WATER, is_gravity_affected(), is_liquid()
- `crates/plix-server/src/lib.rs` - Added block_physics module, ServerConfig.block_physics, Server integration
- `crates/plix-server/src/main.rs` - Added block_physics config initialization
- `crates/plix-arena/src/format.rs` - Implemented BlockWorld trait for LoadedArena

### Test Summary

- **plix-common block_physics**: 22 tests
- **plix-server block_physics**: 20 tests
- **Total**: 42 tests passing
- **Full workspace**: All tests passing (no regressions)
