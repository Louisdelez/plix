# Tasks: Chunked World

**Input**: Design documents from `/specs/011-chunked-world/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4, US5)
- Exact file paths included in descriptions

## Path Conventions

- **Workspace crates**: `crates/plix-common/src/`, `crates/plix-client/src/`, `crates/plix-arena/src/`
- **Tests**: `crates/*/tests/` or inline `#[cfg(test)]` modules

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Core types and coordinate math that ALL user stories depend on

- [x] T001 Add ChunkCoord type alias in crates/plix-common/src/types.rs (leverage existing ChunkPos)
- [x] T002 [P] Add CHUNK_SIZE constant (16) in crates/plix-common/src/chunk.rs
- [x] T003 [P] Implement local_to_index(lx, ly, lz) -> usize helper in crates/plix-common/src/chunk.rs
- [x] T004 [P] Implement index_to_local(i) -> (usize, usize, usize) helper in crates/plix-common/src/chunk.rs
- [x] T005 Implement world_to_chunk(x, y, z) -> (ChunkCoord, (usize, usize, usize)) conversion (floor division for negatives) in crates/plix-common/src/chunk.rs
- [x] T006 Implement chunk_to_world(ChunkCoord, local) -> (i32, i32, i32) conversion in crates/plix-common/src/chunk.rs
- [x] T007 [P] Implement is_boundary_local(local) -> bool in crates/plix-common/src/chunk.rs
- [x] T008 [P] Implement boundary_neighbor(coord, local) -> Option<ChunkCoord> in crates/plix-common/src/chunk.rs
- [x] T009 Add unit tests for coordinate conversions: positive, negative, boundary cases in crates/plix-common/src/chunk.rs (#[cfg(test)] module)
- [x] T010 Export chunk module from crates/plix-common/src/lib.rs

**Checkpoint**: Coordinate math foundation ready for all user stories

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data structures that MUST be complete before user story implementation

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T011 Create AABB struct (min: Vec3, max: Vec3) in crates/plix-common/src/chunk.rs (NOTE: AABB already exists in math.rs, reused)
- [x] T012 Implement AABB::from_chunk_coord(coord) -> Self in crates/plix-common/src/chunk.rs (implemented as Chunk::compute_aabb)
- [x] T013 [P] Implement AABB::center() -> Vec3 in crates/plix-common/src/chunk.rs (AABB already has center() in math.rs)
- [x] T014 Create Chunk struct with blocks: [BlockType; 4096], dirty: bool, aabb: AABB in crates/plix-common/src/chunk.rs
- [x] T015 Implement Chunk::new(coord) -> Self (fills with AIR) in crates/plix-common/src/chunk.rs
- [x] T016 Implement Chunk::get_block(local) and Chunk::set_block(local, block) in crates/plix-common/src/chunk.rs
- [x] T017 [P] Implement Chunk::mark_dirty(), clear_dirty(), is_dirty() in crates/plix-common/src/chunk.rs
- [x] T018 [P] Implement Chunk::is_empty() (all AIR) in crates/plix-common/src/chunk.rs
- [x] T019 Create ChunkedWorld struct with chunks: HashMap<ChunkCoord, Chunk> in crates/plix-common/src/world.rs
- [x] T020 Implement ChunkedWorld::new(), get_block(BlockPos), set_block(BlockPos, BlockType) in crates/plix-common/src/world.rs
- [x] T021 [P] Implement ChunkedWorld::get_chunk(), get_chunk_mut(), ensure_chunk() in crates/plix-common/src/world.rs
- [x] T022 [P] Implement ChunkedWorld::remove_chunk(), iter_chunks(), chunk_count() in crates/plix-common/src/world.rs
- [x] T023 Add unit tests for Chunk set/get correctness in crates/plix-common/src/chunk.rs
- [x] T024 Add unit tests for ChunkedWorld cross-chunk operations in crates/plix-common/src/world.rs
- [x] T025 Export world module from crates/plix-common/src/lib.rs

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Smooth World Rendering (Priority: P1) 🎯 MVP

**Goal**: Render voxel world using per-chunk meshes with visual parity to existing system

**Independent Test**: Load test_arena, verify all blocks render correctly as chunk meshes matching previous output

### Implementation for User Story 1

- [x] T026 [US1] Implement LoadedArena::to_chunked_world() in crates/plix-arena/src/format.rs
- [x] T027 [US1] Define WorldView trait for cross-chunk block lookup in crates/plix-client/src/chunk_mesher.rs
- [x] T028 [US1] Implement WorldView for ChunkedWorld in crates/plix-client/src/chunk_mesher.rs
- [x] T029 [US1] Create ChunkMesh struct (vertex_buffer, index_buffer, num_indices) in crates/plix-client/src/chunk_mesher.rs
- [x] T030 [US1] Implement ChunkMesher::mesh_chunk() - iterate blocks, emit visible faces only in crates/plix-client/src/chunk_mesher.rs
- [x] T031 [US1] Handle cross-chunk neighbor lookups via WorldView in ChunkMesher in crates/plix-client/src/chunk_mesher.rs
- [x] T032 [US1] Implement ChunkMesher::create_mesh() - upload vertex/index buffers to GPU in crates/plix-client/src/chunk_mesher.rs
- [x] T033 [US1] Update ClientWorld to use ChunkedWorld instead of flat arena storage in crates/plix-client/src/world.rs
- [x] T034 [US1] Modify arena loading path to call to_chunked_world() on connect in crates/plix-client/src/main.rs
- [x] T035 [US1] Implement render loop to iterate loaded chunks and draw meshes in crates/plix-client/src/render/engine.rs
- [x] T036 [US1] Replace/remove monolithic arena mesh path in crates/plix-client/src/render/voxels.rs (deprecated, chunked path used)
- [x] T037 [US1] Add meshing sanity tests: single block=6 faces, 2 adjacent=10 faces in crates/plix-client/src/chunk_mesher.rs
- [x] T038 [US1] Export chunk_mesher module from crates/plix-client/src/lib.rs

**Checkpoint**: User Story 1 complete - chunked rendering works, visual parity achieved

---

## Phase 4: User Story 2 - Chunk Streaming (Priority: P1)

**Goal**: Load nearby chunks and unload distant ones based on player position

**Independent Test**: Move player through world, verify chunks load/unload without hitches >30ms

### Implementation for User Story 2

- [x] T039 [US2] Create ChunkManagerConfig struct (view_distance: u8, mesh_budget: u32) in crates/plix-client/src/chunk_manager.rs
- [x] T040 [US2] Create ChunkManager struct (config, loaded: HashSet, dirty_queue: VecDeque, meshes: HashMap) in crates/plix-client/src/chunk_manager.rs
- [x] T041 [US2] Implement ChunkManager::new() and with_defaults() in crates/plix-client/src/chunk_manager.rs
- [x] T042 [US2] Implement compute_desired_chunks(player_pos, radius) -> HashSet<ChunkCoord> in crates/plix-client/src/chunk_manager.rs
- [x] T043 [US2] Implement load_missing_chunks() - pull from ChunkedWorld, mark dirty for initial mesh in crates/plix-client/src/chunk_manager.rs
- [x] T044 [US2] Implement unload_far_chunks() - remove from loaded, free mesh resources in crates/plix-client/src/chunk_manager.rs
- [x] T045 [US2] Implement ChunkManager::update() - orchestrates load/unload/rebuild per frame in crates/plix-client/src/chunk_manager.rs
- [x] T046 [US2] Add mesh rebuild budget enforcement (max mesh_budget_per_frame rebuilds) in crates/plix-client/src/chunk_manager.rs
- [ ] T047 [US2] Integrate ChunkManager into client game loop in crates/plix-client/src/main.rs
- [x] T048 [US2] Add unit tests: streaming set correctness, stable when standing still in crates/plix-client/src/chunk_manager.rs
- [x] T049 [US2] Export chunk_manager module from crates/plix-client/src/lib.rs

**Checkpoint**: User Story 2 complete - streaming works, no hitches

---

## Phase 5: User Story 3 - Efficient Block Edit Updates (Priority: P2)

**Goal**: Block edits update only affected chunk meshes within 2 frames

**Independent Test**: Place/remove blocks, verify only affected chunks rebuild, visible in ≤2 frames

### Implementation for User Story 3

- [ ] T050 [US3] Implement ChunkManager::mark_dirty(coord) with deduplication (HashSet check) in crates/plix-client/src/chunk_manager.rs
- [ ] T051 [US3] Hook BlockEditApplied event to call mark_dirty for owning chunk in crates/plix-client/src/world.rs
- [ ] T052 [US3] Implement boundary neighbor invalidation: if local is 0 or 15 on any axis, mark neighbor dirty in crates/plix-client/src/world.rs
- [ ] T053 [US3] Implement dirty queue processing: rebuild up to mesh_budget dirty chunks per frame in crates/plix-client/src/chunk_manager.rs
- [ ] T054 [US3] Ensure mesh update replaces existing GPU buffers (no leaks) in crates/plix-client/src/chunk_manager.rs
- [ ] T055 [US3] Add unit tests: boundary edit dirties both chunks, interior edit dirties one in crates/plix-client/src/chunk_manager.rs

**Checkpoint**: User Story 3 complete - partial rebuild works

---

## Phase 6: User Story 4 - View Culling for Performance (Priority: P2)

**Goal**: Skip rendering chunks outside view frustum to reduce draw calls

**Independent Test**: Load many chunks, verify draw calls decrease when camera looks away

### Implementation for User Story 4

- [ ] T056 [US4] Create Plane struct (normal: Vec3, distance: f32) in crates/plix-client/src/render/frustum.rs
- [ ] T057 [US4] Create Frustum struct with 6 planes (left, right, bottom, top, near, far) in crates/plix-client/src/render/frustum.rs
- [ ] T058 [US4] Implement Frustum::from_view_proj(Mat4) -> Self (extract planes from matrix) in crates/plix-client/src/render/frustum.rs
- [ ] T059 [US4] Implement AABB::intersects_frustum(planes) -> bool in crates/plix-common/src/chunk.rs
- [ ] T060 [US4] Implement ChunkManager::visible_chunks(frustum) -> iterator of visible (coord, mesh) in crates/plix-client/src/chunk_manager.rs
- [ ] T061 [US4] Update render loop to use visible_chunks() instead of all loaded chunks in crates/plix-client/src/render/engine.rs
- [ ] T062 [P] [US4] Add culling_enabled config toggle in crates/plix-client/src/chunk_manager.rs
- [ ] T063 [P] [US4] Add show_chunk_bounds debug config (optional wireframe rendering) in crates/plix-client/src/chunk_manager.rs
- [ ] T064 [US4] Add unit tests: chunk outside frustum culled, chunk inside drawn in crates/plix-client/src/render/frustum.rs
- [ ] T065 [US4] Export frustum module from crates/plix-client/src/render/mod.rs

**Checkpoint**: User Story 4 complete - culling reduces draw calls

---

## Phase 7: User Story 5 - Late Joiner Compatibility (Priority: P3)

**Goal**: Late joiners see correct world state including prior block modifications

**Independent Test**: Join after block edits, verify world state matches server

### Implementation for User Story 5

- [ ] T066 [US5] Verify to_chunked_world() handles modified arena state correctly in crates/plix-arena/src/format.rs
- [ ] T067 [US5] Ensure BlockEditApplied events replay correctly after initial load in crates/plix-client/src/world.rs
- [ ] T068 [US5] Verify ChunkManager initializes from pre-modified ChunkedWorld in crates/plix-client/src/chunk_manager.rs
- [ ] T069 [US5] Add integration test: modify arena server-side, late join, verify visual match in crates/plix-client/tests/

**Checkpoint**: User Story 5 complete - late joiners work correctly

---

## Phase 8: Integration & Non-Regression

**Purpose**: Ensure all systems integrate without breaking existing functionality

- [ ] T070 Ensure headless server mode bypasses wgpu/rendering paths cleanly in crates/plix-server/src/lib.rs
- [ ] T071 Verify match phase restrictions unchanged (block edits only in Playing) in crates/plix-server/src/sim/block_edit.rs
- [ ] T072 Run cargo test --workspace and fix any failures
- [ ] T073 Run cargo clippy --all-targets and fix any warnings
- [ ] T074 Run cargo fmt --all -- --check and fix any formatting issues
- [ ] T075 [P] Verify collision system works with chunked block access in crates/plix-server/src/sim/collision.rs

**Checkpoint**: All automated tests pass, lint clean

---

## Phase 9: Polish & Validation

**Purpose**: Manual testing and documentation

- [ ] T076 Manual test M001: Fly around arena, verify chunks load/unload correctly
- [ ] T077 Manual test M002: Place/remove blocks rapidly, verify only nearby chunks rebuild
- [ ] T078 Manual test M003: Edit blocks at chunk boundaries, verify neighbor updates
- [ ] T079 Manual test M004: Look away from chunks, verify culling reduces draw calls
- [ ] T080 [P] Update quickstart.md with final testing instructions in specs/011-chunked-world/quickstart.md
- [ ] T081 Verify visual parity: chunked rendering matches previous arena rendering exactly

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **Phase 3 (US1 Rendering)**: Depends on Phase 2
- **Phase 4 (US2 Streaming)**: Depends on Phase 2, can run in parallel with US1
- **Phase 5 (US3 Block Edits)**: Depends on US1 + US2 being complete
- **Phase 6 (US4 Culling)**: Depends on US1 + US2, can run in parallel with US3
- **Phase 7 (US5 Late Joiner)**: Depends on all prior user stories
- **Phase 8 (Integration)**: Depends on all user stories complete
- **Phase 9 (Polish)**: Depends on Phase 8

### User Story Dependencies

```
     Phase 1 (Setup)
           │
           ▼
     Phase 2 (Foundation)
           │
     ┌─────┴─────┐
     │           │
     ▼           ▼
   US1 ←──────► US2    (can run in parallel)
     │           │
     └─────┬─────┘
           │
     ┌─────┴─────┐
     │           │
     ▼           ▼
   US3 ←──────► US4    (can run in parallel)
     │           │
     └─────┬─────┘
           │
           ▼
         US5
           │
           ▼
     Phase 8 (Integration)
           │
           ▼
     Phase 9 (Polish)
```

### Parallel Opportunities

**Within Phase 1 (Setup)**:
- T002, T003, T004 can run in parallel
- T007, T008 can run in parallel

**Within Phase 2 (Foundational)**:
- T013, T017, T018 can run in parallel
- T021, T022 can run in parallel

**User Stories US1 + US2**: Can be developed in parallel after Foundation
**User Stories US3 + US4**: Can be developed in parallel after US1/US2

---

## Parallel Example: Setup Phase

```bash
# Launch parallel tasks for Phase 1:
Task: "Add CHUNK_SIZE constant in crates/plix-common/src/chunk.rs"
Task: "Implement local_to_index helper in crates/plix-common/src/chunk.rs"
Task: "Implement index_to_local helper in crates/plix-common/src/chunk.rs"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (coordinate math)
2. Complete Phase 2: Foundational (Chunk, ChunkedWorld)
3. Complete Phase 3: US1 (rendering)
4. Complete Phase 4: US2 (streaming)
5. **STOP and VALIDATE**: Test visual parity and streaming
6. Deploy/demo if ready - MVP achieved!

### Incremental Delivery

1. Setup + Foundation → Core ready
2. Add US1 (Rendering) → Verify visual parity → Checkpoint
3. Add US2 (Streaming) → Verify no hitches → MVP Checkpoint!
4. Add US3 (Block Edits) → Verify partial rebuild → Checkpoint
5. Add US4 (Culling) → Verify draw call reduction → Checkpoint
6. Add US5 (Late Joiner) → Verify multiplayer → Checkpoint
7. Integration + Polish → Release ready

---

## Definition of Done

- [ ] ChunkedWorld storage and coordinate conversions work correctly (tests pass)
- [ ] Chunk meshes render with visual parity to previous approach
- [ ] Streaming loads/unloads chunks around player without hitches
- [ ] Block edits trigger partial rebuild (including boundary neighbors)
- [ ] Frustum culling reduces draw calls when looking away
- [ ] Late joiners see correct world state
- [ ] All workspace tests pass: cargo test --workspace
- [ ] Lint clean: cargo clippy --all-targets
- [ ] Format clean: cargo fmt --all -- --check
- [ ] Headless mode unaffected
- [ ] Manual validation checklist complete

---

## Notes

- [P] = Can run in parallel (different files, no dependencies)
- [US#] = Maps to user story for traceability
- CHUNK_SIZE = 16, view_distance = 8, mesh_budget = 2 (configurable)
- Verify tests pass after each checkpoint
- Commit after each task or logical group
