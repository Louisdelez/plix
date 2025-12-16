# Tasks: World Edit Optimization

**Input**: Design documents from `/specs/012-world-edit-optimization/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md

**Tests**: Unit tests are INCLUDED as this feature explicitly requires validation per success criteria (SC-001 to SC-008).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md - Rust workspace structure:
- `crates/plix-common/src/` - shared types (chunk.rs)
- `crates/plix-client/src/` - client code (chunk_manager.rs, world.rs)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verify Feature 011 baseline and prepare for extensions

- [x] T001 Verify Feature 011 tests pass with `cargo test -p plix-client chunk_manager`
- [x] T002 Verify Feature 011 chunk.rs tests pass with `cargo test -p plix-common chunk`
- [x] T003 [P] Run `cargo clippy` and fix any existing warnings in modified files

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: These extend existing structures - all user stories depend on them

- [x] T004 Add `MeshMetrics` struct to crates/plix-client/src/chunk_manager.rs (per data-model.md)
- [x] T005 Add `max_retries: u8` field to `ChunkManagerConfig` in crates/plix-client/src/chunk_manager.rs
- [x] T006 Add `retry_counts: HashMap<ChunkCoord, u8>` field to `ChunkManager` in crates/plix-client/src/chunk_manager.rs
- [x] T007 Add `skipped_chunks: HashSet<ChunkCoord>` field to `ChunkManager` in crates/plix-client/src/chunk_manager.rs
- [x] T008 Add `metrics: MeshMetrics` field to `ChunkManager` in crates/plix-client/src/chunk_manager.rs
- [x] T009 Update `ChunkManager::new()` and `with_config()` to initialize new fields in crates/plix-client/src/chunk_manager.rs
- [x] T010 Add `metrics` field to `ChunkManagerUpdate` struct in crates/plix-client/src/chunk_manager.rs

**Checkpoint**: Foundation ready - ChunkManager has all new fields initialized

---

## Phase 3: User Story 1 - Localized Chunk Updates (Priority: P1) 🎯 MVP

**Goal**: Block edits mark only the containing chunk as dirty, with loaded-check

**Independent Test**: Place a block in chunk interior, verify only 1 chunk marked dirty via metrics

### Tests for User Story 1

- [x] T011 [P] [US1] Add test `test_mark_dirty_ignores_unloaded` in crates/plix-client/src/chunk_manager.rs
- [x] T012 [P] [US1] Add test `test_mark_dirty_clears_skipped_status` in crates/plix-client/src/chunk_manager.rs

### Implementation for User Story 1

- [x] T013 [US1] Modify `mark_dirty()` to early-return if chunk not loaded in crates/plix-client/src/chunk_manager.rs (FR-024)
- [x] T014 [US1] Modify `mark_dirty()` to clear skipped status on new edit in crates/plix-client/src/chunk_manager.rs
- [x] T015 [US1] Add `mark_dirty_for_block(pos: BlockPos)` method to ChunkManager in crates/plix-client/src/chunk_manager.rs
- [x] T016 [US1] Implement mark_dirty_for_block: compute chunk coord, check loaded, mark dirty in crates/plix-client/src/chunk_manager.rs

**Checkpoint**: Localized marking works - edits to unloaded chunks are ignored, single chunk marked

---

## Phase 4: User Story 2 - Boundary Block Handling (Priority: P1)

**Goal**: Boundary edits automatically mark affected neighbor chunks dirty

**Independent Test**: Place block at (15, y, z), verify 2 chunks marked dirty (origin + neighbor)

### Tests for User Story 2

- [x] T017 [P] [US2] Add test `test_boundary_neighbors_face` for single-axis boundary in crates/plix-common/src/chunk.rs
- [x] T018 [P] [US2] Add test `test_boundary_neighbors_edge` for two-axis boundary in crates/plix-common/src/chunk.rs
- [x] T019 [P] [US2] Add test `test_boundary_neighbors_corner` for three-axis boundary in crates/plix-common/src/chunk.rs
- [x] T020 [P] [US2] Add test `test_mark_dirty_for_block_boundary` in crates/plix-client/src/chunk_manager.rs

### Implementation for User Story 2

- [x] T021 [US2] Verify existing `boundary_neighbors()` handles face boundaries correctly in crates/plix-common/src/chunk.rs
- [x] T022 [US2] Extend `mark_dirty_for_block()` to call `is_boundary_local()` and mark neighbors in crates/plix-client/src/chunk_manager.rs
- [x] T023 [US2] Filter boundary neighbors through `is_loaded()` before marking in crates/plix-client/src/chunk_manager.rs

**Checkpoint**: Boundary handling works - face/edge/corner edits mark correct neighbors

---

## Phase 5: User Story 3 - Dirty Queue Deduplication (Priority: P1)

**Goal**: Duplicate dirty requests collapsed (already implemented in Feature 011, verify)

**Independent Test**: Mark same chunk dirty 100 times, verify queue has 1 entry

### Tests for User Story 3

- [x] T024 [P] [US3] Add test `test_deduplication_100_marks` marking same chunk 100 times in crates/plix-client/src/chunk_manager.rs
- [x] T025 [P] [US3] Add test `test_deduplication_multiple_chunks` marking 5 chunks twice each in crates/plix-client/src/chunk_manager.rs

### Implementation for User Story 3

- [x] T026 [US3] Verify existing deduplication in `mark_dirty()` uses `dirty_set.insert()` in crates/plix-client/src/chunk_manager.rs
- [x] T027 [US3] Verify `dirty_set` and `dirty_queue` invariant (same length) holds in crates/plix-client/src/chunk_manager.rs

**Checkpoint**: Deduplication verified - rapid edits don't explode queue size

---

## Phase 6: User Story 4 - Mesh Budget Enforcement (Priority: P2)

**Goal**: Per-frame rebuild limit prevents frame time spikes (already implemented, verify + metrics)

**Independent Test**: Queue 100 dirty chunks with budget=2, verify 2 processed per frame

### Tests for User Story 4

- [x] T028 [P] [US4] Add test `test_budget_50_chunks_25_frames` verifying exact drain time in crates/plix-client/src/chunk_manager.rs

### Implementation for User Story 4

- [x] T029 [US4] Verify existing `pop_dirty_batch()` respects `mesh_budget_per_frame` in crates/plix-client/src/chunk_manager.rs
- [x] T030 [US4] Update `update()` to set `metrics.rebuilds_this_frame` from batch size in crates/plix-client/src/chunk_manager.rs
- [x] T031 [US4] Update `update()` to set `metrics.dirty_queue_depth` after pop in crates/plix-client/src/chunk_manager.rs

**Checkpoint**: Budget enforcement verified with accurate metrics

---

## Phase 7: User Story 5 - Retry Policy (Priority: P2)

**Goal**: Failed mesh rebuilds retry up to 3 times then skip

**Independent Test**: Simulate 3 failures for a chunk, verify it's skipped on 4th

### Tests for User Story 5

- [x] T032 [P] [US5] Add test `test_retry_success_clears_count` in crates/plix-client/src/chunk_manager.rs
- [x] T033 [P] [US5] Add test `test_retry_failure_increments_count` in crates/plix-client/src/chunk_manager.rs
- [x] T034 [P] [US5] Add test `test_retry_exceeded_skips_chunk` in crates/plix-client/src/chunk_manager.rs
- [x] T035 [P] [US5] Add test `test_skipped_chunk_requeues_on_new_edit` in crates/plix-client/src/chunk_manager.rs

### Implementation for User Story 5

- [x] T036 [US5] Add `report_rebuild_result(coord: ChunkCoord, success: bool)` method in crates/plix-client/src/chunk_manager.rs
- [x] T037 [US5] Implement success path: clear retry count, increment metrics.successful_rebuilds
- [x] T038 [US5] Implement failure path: increment retry count, check against max_retries
- [x] T039 [US5] Implement skip path: add to skipped_chunks, remove from retry_counts, increment metrics
- [x] T040 [US5] Implement re-queue path: call mark_dirty() for retry when below max

**Checkpoint**: Retry logic complete - failures tracked, retried, then skipped

---

## Phase 8: User Story 6 - Observability Metrics (Priority: P2)

**Goal**: Counter metrics exposed for debugging

**Independent Test**: Process some chunks, verify metrics reflect actual counts

### Tests for User Story 6

- [x] T041 [P] [US6] Add test `test_metrics_accuracy` verifying all counter fields in crates/plix-client/src/chunk_manager.rs

### Implementation for User Story 6

- [x] T042 [US6] Add `metrics(&self) -> &MeshMetrics` accessor method in crates/plix-client/src/chunk_manager.rs
- [x] T043 [US6] Add `is_skipped(coord: ChunkCoord) -> bool` method in crates/plix-client/src/chunk_manager.rs
- [x] T044 [US6] Add `clear_skipped(coord: ChunkCoord)` method in crates/plix-client/src/chunk_manager.rs
- [x] T045 [US6] Update `update()` to reset per-frame metrics at start in crates/plix-client/src/chunk_manager.rs
- [x] T046 [US6] Include metrics snapshot in `ChunkManagerUpdate` return value in crates/plix-client/src/chunk_manager.rs

**Checkpoint**: Metrics exposed - debug tools can read counters

---

## Phase 9: Integration (Block Edit Pipeline)

**Goal**: Connect block edits to dirty marking automatically

**Independent Test**: Edit block via ClientWorld, verify chunk marked dirty

### Tests for Integration

- [x] T047 [P] Add integration test `test_block_edit_marks_dirty` in crates/plix-client/src/world.rs

### Implementation for Integration

- [x] T048 Integrate `mark_dirty_for_block()` call into ClientWorld block edit path in crates/plix-client/src/world.rs
- [x] T049 Integrate dirty marking into server-sent block update handler in crates/plix-client/src/world.rs

**Note**: T048-T049 are implemented via the existing `ChunkedWorld::set_block()` which already returns affected chunks. The new `ChunkManager::mark_dirty_for_block()` provides an alternative API for callers who have a ChunkManager reference and want automatic boundary handling.

**Checkpoint**: Full pipeline connected - block edits auto-trigger mesh updates

---

## Phase 10: Polish & Validation

**Purpose**: Final validation, non-regression, documentation

- [x] T050 Run all Feature 011 tests to verify non-regression with `cargo test -p plix-client`
- [x] T051 Run all Feature 011 common tests with `cargo test -p plix-common`
- [x] T052 [P] Run `cargo clippy --all-targets` and fix any new warnings
- [x] T053 [P] Run `cargo fmt --all` to ensure formatting
- [x] T054 Add inline documentation for new public methods in crates/plix-client/src/chunk_manager.rs
- [x] T055 Verify success criteria SC-001 through SC-006 via test suite
- [ ] T056 Run quickstart.md validation scenarios manually (deferred - requires game running)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - verify baseline
- **Foundational (Phase 2)**: Depends on Setup - adds new struct fields
- **User Stories (Phase 3-8)**: All depend on Foundational completion
  - US1, US2, US3 are all P1 priority - complete in order
  - US4, US5, US6 are P2 priority - complete after P1 stories
- **Integration (Phase 9)**: Depends on US1 (mark_dirty_for_block exists)
- **Polish (Phase 10)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (Localized Updates)**: Foundation only - creates mark_dirty_for_block()
- **US2 (Boundary Handling)**: Foundation only, extends mark_dirty_for_block()
- **US3 (Deduplication)**: Foundation only - verification of existing code
- **US4 (Budget)**: Foundation + US6 metrics (for visibility)
- **US5 (Retry Policy)**: Foundation + MeshMetrics
- **US6 (Observability)**: Foundation only - adds accessors

### Within Each User Story

- Tests FIRST, verify they FAIL before implementation
- Implementation follows test definitions
- Verify tests PASS after implementation

### Parallel Opportunities

**Phase 2 (Foundational)**: T004-T010 modify same file - execute sequentially

**Phase 3-8 (User Stories)**:
- All tests within a story marked [P] can run in parallel
- US1, US2, US3 (P1) should complete before US4, US5, US6 (P2)
- Within P1: US1 → US2 (US2 extends mark_dirty_for_block)

**Phase 10 (Polish)**: T052, T053 can run in parallel

---

## Parallel Example: User Story 2 Tests

```bash
# Launch all US2 tests together:
Task: "Add test test_boundary_neighbors_face in crates/plix-common/src/chunk.rs"
Task: "Add test test_boundary_neighbors_edge in crates/plix-common/src/chunk.rs"
Task: "Add test test_boundary_neighbors_corner in crates/plix-common/src/chunk.rs"
Task: "Add test test_mark_dirty_for_block_boundary in crates/plix-client/src/chunk_manager.rs"
```

---

## Implementation Strategy

### MVP First (User Stories 1-3 Only)

1. Complete Phase 1: Setup (verify baseline)
2. Complete Phase 2: Foundational (add fields)
3. Complete Phase 3: US1 - Localized Updates
4. Complete Phase 4: US2 - Boundary Handling
5. Complete Phase 5: US3 - Deduplication
6. **STOP and VALIDATE**: Test P1 stories work together
7. Deploy/demo if ready - core optimization complete

### Full Implementation

1. Complete MVP (P1 stories)
2. Add US4 - Budget verification
3. Add US5 - Retry policy
4. Add US6 - Observability
5. Complete Integration (Phase 9)
6. Polish and validate (Phase 10)

---

## Notes

- [P] tasks = different files or independent tests, no dependencies
- [Story] label maps task to specific user story for traceability
- This feature EXTENDS existing Feature 011 code - verify non-regression
- Most foundational work is in single file (chunk_manager.rs) - sequential execution
- Existing dirty queue/deduplication/budget from F011 only needs verification, not reimplementation
- Research.md clarified: boundary_neighbors() is CORRECT for mesh visibility (axis-aligned only)
