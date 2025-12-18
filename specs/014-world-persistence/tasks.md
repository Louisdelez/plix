# Tasks: World Persistence

**Input**: Design documents from `/specs/014-world-persistence/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Tests are included as they are essential for persistence reliability (SC-001, SC-006).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US6)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md structure:
- **Core types**: `crates/plix-common/src/persist/`
- **Server integration**: `crates/plix-server/src/persist/`
- **Client integration**: `crates/plix-client/src/persist/`
- **Existing files to modify**: `crates/plix-common/src/chunk.rs`, `crates/plix-common/src/world.rs`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create persist module structure and define foundational types

- [ ] T001 Create persist module directory structure in crates/plix-common/src/persist/
- [ ] T002 [P] Create PersistError enum in crates/plix-common/src/persist/error.rs
- [ ] T003 [P] Define CURRENT_VERSION, MIN_SUPPORTED_VERSION constants and VersionCheck enum in crates/plix-common/src/persist/version.rs
- [ ] T004 [P] Define WorldMetadata and WorldKind structs in crates/plix-common/src/persist/world_meta.rs
- [ ] T005 [P] Define ChunkData struct for serialization in crates/plix-common/src/persist/chunk_codec.rs
- [ ] T006 Create persist module exports in crates/plix-common/src/persist/mod.rs
- [ ] T007 Add persist module to crates/plix-common/src/lib.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T008 Implement version compatibility check function in crates/plix-common/src/persist/version.rs
- [ ] T009 [P] Implement WorldMetadata validation in crates/plix-common/src/persist/world_meta.rs
- [ ] T010 [P] Implement ChunkCodec::encode() in crates/plix-common/src/persist/chunk_codec.rs
- [ ] T011 [P] Implement ChunkCodec::decode() with validation in crates/plix-common/src/persist/chunk_codec.rs
- [ ] T012 [P] Implement chunk_filename() and parse_chunk_filename() utilities in crates/plix-common/src/persist/chunk_codec.rs
- [ ] T013 Add persistence_dirty HashSet to ChunkedWorld in crates/plix-common/src/world.rs
- [ ] T014 Add mark_persistence_dirty() and clear_persistence_dirty() methods to ChunkedWorld in crates/plix-common/src/world.rs
- [ ] T015 Add persistence_dirty_chunks() iterator to ChunkedWorld in crates/plix-common/src/world.rs
- [ ] T016 Modify ChunkedWorld::set_block() to mark persistence_dirty on block change in crates/plix-common/src/world.rs
- [ ] T017 [P] Add unit test for ChunkCodec round-trip in crates/plix-common/src/persist/chunk_codec.rs
- [ ] T018 [P] Add unit test for version compatibility check in crates/plix-common/src/persist/version.rs
- [ ] T019 [P] Add unit test for WorldMetadata validation in crates/plix-common/src/persist/world_meta.rs
- [ ] T020 [P] Add unit test for chunk filename generation/parsing in crates/plix-common/src/persist/chunk_codec.rs

**Checkpoint**: Core persistence types ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Solo World Save and Reload (Priority: P1) 🎯 MVP

**Goal**: Solo player can save world, quit, and reload with 100% fidelity

**Independent Test**: Create solo world → modify blocks → save → restart → load → verify identical state

### Tests for User Story 1

- [ ] T021 [P] [US1] Integration test: create world, save, load, verify metadata in crates/plix-server/src/persist/tests/
- [ ] T022 [P] [US1] Integration test: save chunk, load chunk, verify blocks identical in crates/plix-server/src/persist/tests/
- [ ] T023 [P] [US1] Integration test: multiple chunks save/load cycle in crates/plix-server/src/persist/tests/

### Implementation for User Story 1

- [ ] T024 Create persist module structure in crates/plix-server/src/persist/
- [ ] T025 [P] [US1] Implement atomic_write() utility (temp file + rename) in crates/plix-server/src/persist/atomic.rs
- [ ] T026 [P] [US1] Implement default_worlds_dir() platform detection in crates/plix-server/src/persist/world_store.rs
- [ ] T027 [US1] Implement WorldStore::create_world() in crates/plix-server/src/persist/world_store.rs
- [ ] T028 [US1] Implement WorldStore::open_world() in crates/plix-server/src/persist/world_store.rs
- [ ] T029 [US1] Implement WorldStore::save_meta() with atomic write in crates/plix-server/src/persist/world_store.rs
- [ ] T030 [US1] Implement WorldStore::load_meta() in crates/plix-server/src/persist/world_store.rs
- [ ] T031 [US1] Implement WorldStore::save_chunk() with atomic write in crates/plix-server/src/persist/world_store.rs
- [ ] T032 [US1] Implement WorldStore::load_chunk() in crates/plix-server/src/persist/world_store.rs
- [ ] T033 [US1] Implement WorldStore::list_saved_chunks() in crates/plix-server/src/persist/world_store.rs
- [ ] T034 [US1] Create persist module structure in crates/plix-client/src/persist/
- [ ] T035 [US1] Implement LocalStore wrapper for solo mode in crates/plix-client/src/persist/local_store.rs
- [ ] T036 [US1] Add save_world() trigger on quit/menu in crates/plix-client/src/
- [ ] T037 [US1] Add load_world() on world selection in crates/plix-client/src/
- [ ] T038 [US1] Add INFO logging for world open/create/save in crates/plix-server/src/persist/world_store.rs

**Checkpoint**: Solo save/load fully functional - verify SC-001 (100% fidelity)

---

## Phase 4: User Story 2 - Server World Persistence (Priority: P1)

**Goal**: Server persists world across restarts with auto-save every 5 minutes

**Independent Test**: Run server → clients modify blocks → stop server → restart → verify state preserved

### Tests for User Story 2

- [ ] T039 [P] [US2] Integration test: server save on shutdown preserves state in crates/plix-server/src/persist/tests/
- [ ] T040 [P] [US2] Integration test: auto-save scheduler triggers at interval in crates/plix-server/src/persist/tests/

### Implementation for User Story 2

- [ ] T041 [P] [US2] Define SaveSchedulerConfig struct in crates/plix-server/src/persist/scheduler.rs
- [ ] T042 [P] [US2] Define SaveMetrics struct in crates/plix-server/src/persist/scheduler.rs
- [ ] T043 [US2] Implement SaveScheduler::new() and SaveScheduler::with_defaults() in crates/plix-server/src/persist/scheduler.rs
- [ ] T044 [US2] Implement SaveScheduler::mark_dirty() and dirty tracking in crates/plix-server/src/persist/scheduler.rs
- [ ] T045 [US2] Implement SaveScheduler::tick() with interval check and bounded chunk save in crates/plix-server/src/persist/scheduler.rs
- [ ] T046 [US2] Implement SaveScheduler::flush() for shutdown in crates/plix-server/src/persist/scheduler.rs
- [ ] T047 [US2] Hook block edits to mark_dirty() in server game loop in crates/plix-server/src/lib.rs
- [ ] T048 [US2] Add scheduler.tick() call to server main loop in crates/plix-server/src/lib.rs
- [ ] T049 [US2] Add scheduler.flush() on graceful shutdown in crates/plix-server/src/lib.rs
- [ ] T050 [US2] Add world loading on server start in crates/plix-server/src/lib.rs
- [ ] T051 [US2] Add INFO logging for auto-save start/complete in crates/plix-server/src/persist/scheduler.rs

**Checkpoint**: Server persistence with auto-save functional - verify SC-002 (<500ms save) and SC-007

---

## Phase 5: User Story 3 - Procedural World with Modifications (Priority: P2)

**Goal**: Procedural world saves only modified chunks, regenerates unmodified from seed

**Independent Test**: Generate world with seed → modify some chunks → save → reload → verify generated + modified parts correct

### Tests for User Story 3

- [ ] T052 [P] [US3] Integration test: generated world saves only dirty chunks in crates/plix-server/src/persist/tests/
- [ ] T053 [P] [US3] Integration test: reload regenerates unmodified chunks from seed in crates/plix-server/src/persist/tests/

### Implementation for User Story 3

- [ ] T054 [US3] Implement WorldStore::chunk_exists() for checking saved chunk presence in crates/plix-server/src/persist/world_store.rs
- [ ] T055 [US3] Implement get_or_load_chunk() logic: check file first, fallback to generate in crates/plix-server/src/persist/world_store.rs
- [ ] T056 [US3] Integrate chunk loading with ChunkGenerator for WorldKind::Generated in crates/plix-server/src/lib.rs
- [ ] T057 [US3] Ensure only persistence_dirty_chunks are saved (delta approach) in crates/plix-server/src/persist/scheduler.rs
- [ ] T058 [US3] Add test: verify save file size proportional to modified chunks only (SC-004) in crates/plix-server/src/persist/tests/

**Checkpoint**: Delta save/load working - verify SC-004 (proportional file size)

---

## Phase 6: User Story 4 - World Metadata Access (Priority: P2)

**Goal**: List available worlds with metadata without loading chunk data

**Independent Test**: Create multiple worlds → list worlds → verify metadata displayed without loading chunks

### Tests for User Story 4

- [ ] T059 [P] [US4] Integration test: list_worlds returns all worlds with metadata in crates/plix-server/src/persist/tests/
- [ ] T060 [P] [US4] Integration test: corrupted world appears in list with error indicator in crates/plix-server/src/persist/tests/

### Implementation for User Story 4

- [ ] T061 [US4] Implement WorldStore::list_worlds() to enumerate world directories in crates/plix-server/src/persist/world_store.rs
- [ ] T062 [US4] Load only meta.bin for each world (not chunks) in list_worlds() in crates/plix-server/src/persist/world_store.rs
- [ ] T063 [US4] Return (world_id, Result<WorldMetadata, PersistError>) for each world in crates/plix-server/src/persist/world_store.rs
- [ ] T064 [US4] Add metadata load benchmark test (<50ms per SC-003) in crates/plix-server/src/persist/tests/

**Checkpoint**: World listing functional - verify SC-003 (<50ms metadata load)

---

## Phase 7: User Story 5 - Version Compatibility Handling (Priority: P2)

**Goal**: Clear feedback on version incompatibility, automatic migration when possible

**Independent Test**: Create worlds with different versions → attempt load → verify correct behavior per version

### Tests for User Story 5

- [ ] T065 [P] [US5] Unit test: version TooNew returns clear error message in crates/plix-common/src/persist/version.rs
- [ ] T066 [P] [US5] Unit test: version TooOld returns clear error message in crates/plix-common/src/persist/version.rs
- [ ] T067 [P] [US5] Integration test: load world with current version succeeds in crates/plix-server/src/persist/tests/
- [ ] T068 [P] [US5] Integration test: load world with future version fails with clear message in crates/plix-server/src/persist/tests/

### Implementation for User Story 5

- [ ] T069 [US5] Add migration registry structure (empty for v1) in crates/plix-common/src/persist/version.rs
- [ ] T070 [US5] Implement migrate() function framework in crates/plix-common/src/persist/version.rs
- [ ] T071 [US5] Integrate version check in WorldStore::open_world() in crates/plix-server/src/persist/world_store.rs
- [ ] T072 [US5] Return VersionMismatch error with clear user-facing message in crates/plix-server/src/persist/world_store.rs
- [ ] T073 [US5] Add WARN logging for version mismatch in crates/plix-server/src/persist/world_store.rs

**Checkpoint**: Version handling complete - verify SC-005 (100% correct version handling)

---

## Phase 8: User Story 6 - Crash-Safe Saving (Priority: P3)

**Goal**: World remains valid even if crash during save

**Independent Test**: Simulate crash during save → verify world loads from previous valid state

### Tests for User Story 6

- [ ] T074 [P] [US6] Unit test: partial temp file left after crash is cleaned up on next load in crates/plix-server/src/persist/tests/
- [ ] T075 [P] [US6] Integration test: kill during chunk save, world still loadable in crates/plix-server/src/persist/tests/
- [ ] T076 [P] [US6] Integration test: kill during meta save, previous meta still valid in crates/plix-server/src/persist/tests/

### Implementation for User Story 6

- [ ] T077 [US6] Ensure atomic_write uses fsync before rename in crates/plix-server/src/persist/atomic.rs
- [ ] T078 [US6] Add parent directory fsync after rename for durability in crates/plix-server/src/persist/atomic.rs
- [ ] T079 [US6] Implement temp file cleanup on world open in crates/plix-server/src/persist/world_store.rs
- [ ] T080 [US6] Add chunk corruption detection in load_chunk() in crates/plix-server/src/persist/world_store.rs
- [ ] T081 [US6] Return ChunkCorrupted error with coord and reason in crates/plix-server/src/persist/world_store.rs
- [ ] T082 [US6] Add ERROR logging for I/O failures in crates/plix-server/src/persist/atomic.rs

**Checkpoint**: Crash safety verified - verify SC-006 (100% crash recovery)

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T083 [P] Add persistence metrics to SaveMetrics (chunks_dirty_pending gauge) in crates/plix-server/src/persist/scheduler.rs
- [ ] T084 [P] Verify all existing arena tests still pass (non-regression) in crates/plix-arena/
- [ ] T085 [P] Verify ChunkedWorld streaming compatibility in crates/plix-common/
- [ ] T086 Run cargo clippy and fix any warnings in persist modules
- [ ] T087 Run cargo fmt on all modified files
- [ ] T088 Update quickstart.md with actual implementation paths in specs/014-world-persistence/quickstart.md
- [ ] T089 Verify all success criteria (SC-001 through SC-007) pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - can start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **Phase 3-8 (User Stories)**: All depend on Phase 2 completion
- **Phase 9 (Polish)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (Solo Save/Load)**: Can start after Phase 2 - No dependencies on other stories
- **US2 (Server Persistence)**: Depends on US1 (reuses WorldStore) - Can run in parallel with US3-US6
- **US3 (Procedural Delta)**: Depends on US1 (reuses WorldStore) - Independent of US2
- **US4 (Metadata Access)**: Depends on US1 (reuses WorldStore) - Independent of US2, US3
- **US5 (Version Handling)**: Can start after Phase 2 - Independent of US1-US4
- **US6 (Crash Safety)**: Depends on US1 (tests atomic writes) - Independent of US2-US5

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Core components before integration
- WorldStore before Scheduler
- Story complete before moving to next priority

### Parallel Opportunities

All tasks marked [P] can run in parallel within their phase:
- Phase 1: T002, T003, T004, T005 (different files)
- Phase 2: T009, T010, T011, T012, T017, T018, T019, T020 (different files/tests)
- Phase 3: T021, T022, T023 tests; T025, T026 utilities
- Phase 4: T039, T040 tests; T041, T042 structs
- Phase 5-8: Test tasks within each story

---

## Parallel Example: Phase 2 Foundational

```bash
# Launch all independent implementations together:
Task: "Implement WorldMetadata validation in crates/plix-common/src/persist/world_meta.rs"
Task: "Implement ChunkCodec::encode() in crates/plix-common/src/persist/chunk_codec.rs"
Task: "Implement ChunkCodec::decode() in crates/plix-common/src/persist/chunk_codec.rs"
Task: "Implement chunk_filename() utilities in crates/plix-common/src/persist/chunk_codec.rs"

# Launch all unit tests together:
Task: "Add unit test for ChunkCodec round-trip in crates/plix-common/src/persist/chunk_codec.rs"
Task: "Add unit test for version compatibility in crates/plix-common/src/persist/version.rs"
Task: "Add unit test for WorldMetadata validation in crates/plix-common/src/persist/world_meta.rs"
Task: "Add unit test for chunk filename parsing in crates/plix-common/src/persist/chunk_codec.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL)
3. Complete Phase 3: User Story 1 (Solo Save/Load)
4. **STOP and VALIDATE**: Test SC-001 (100% fidelity)
5. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Core types ready
2. US1 (Solo) → Basic save/load works (MVP!)
3. US2 (Server) → Auto-save for multiplayer
4. US3 (Procedural) → Delta saves for large worlds
5. US4 (Metadata) → World selection UX
6. US5 (Versioning) → Future compatibility
7. US6 (Crash Safety) → Production reliability

### Success Criteria Verification Order

| Criterion | Verified After |
|-----------|----------------|
| SC-001 (100% fidelity) | US1 |
| SC-002 (<500ms save) | US2 |
| SC-003 (<50ms metadata) | US4 |
| SC-004 (proportional size) | US3 |
| SC-005 (version handling) | US5 |
| SC-006 (crash recovery) | US6 |
| SC-007 (server persistence) | US2 |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
