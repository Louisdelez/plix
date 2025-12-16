# Tasks: Procedural Generation v1

**Input**: Design documents from `/specs/013-procedural-generation/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

**Tests**: Unit tests are INCLUDED as this feature explicitly requires determinism validation per success criteria (SC-001 to SC-008).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md - Rust workspace structure:
- `crates/plix-common/src/` - shared types and worldgen module
- `crates/plix-common/src/worldgen/` - NEW module for generation
- `crates/plix-client/src/` - client integration
- `crates/plix-server/src/` - server integration

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add noise dependency and create module structure

- [ ] T001 Add `noise = "0.9"` dependency to crates/plix-common/Cargo.toml
- [ ] T002 Create worldgen module directory at crates/plix-common/src/worldgen/
- [ ] T003 Create mod.rs with module exports at crates/plix-common/src/worldgen/mod.rs
- [ ] T004 Add `pub mod worldgen;` to crates/plix-common/src/lib.rs
- [ ] T005 [P] Extend BlockType with GRASS, DIRT, SAND, SANDSTONE, BEDROCK in crates/plix-common/src/types.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core configuration and noise infrastructure that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T006 Create WorldGenConfig struct with Default impl in crates/plix-common/src/worldgen/config.rs
- [ ] T007 Implement seed derivation function `derive_seed(u64, u32) -> u32` in crates/plix-common/src/worldgen/config.rs
- [ ] T008 Create NoiseSource struct wrapping noise-rs in crates/plix-common/src/worldgen/noise.rs
- [ ] T009 Implement NoiseSource::new(seed, octaves) with height/biome/temperature noise in crates/plix-common/src/worldgen/noise.rs
- [ ] T010 Implement sample_height(), sample_biome_elevation(), sample_temperature() in crates/plix-common/src/worldgen/noise.rs
- [ ] T011 [P] Add test `test_noise_determinism` verifying same seed produces same values in crates/plix-common/src/worldgen/noise.rs

**Checkpoint**: Foundation ready - noise sampling works deterministically

---

## Phase 3: User Story 1 - Deterministic World Generation (Priority: P1) 🎯 MVP

**Goal**: Same seed + chunk coord always produces identical chunk output

**Independent Test**: Generate same chunk twice with same seed, verify block-for-block equality

### Tests for User Story 1

- [ ] T012 [P] [US1] Add test `test_chunk_determinism_same_seed` in crates/plix-common/src/worldgen/generator.rs
- [ ] T013 [P] [US1] Add test `test_chunk_determinism_order_independent` in crates/plix-common/src/worldgen/generator.rs
- [ ] T014 [P] [US1] Add test `test_seed_edge_cases` (seed=0, seed=u64::MAX) in crates/plix-common/src/worldgen/generator.rs

### Implementation for User Story 1

- [ ] T015 [US1] Create ChunkGenerator struct skeleton in crates/plix-common/src/worldgen/generator.rs
- [ ] T016 [US1] Implement ChunkGenerator::new(config) in crates/plix-common/src/worldgen/generator.rs
- [ ] T017 [US1] Implement generate_chunk(coord) -> Chunk as pure function in crates/plix-common/src/worldgen/generator.rs
- [ ] T018 [US1] Implement block_at(y, surface_y, biome) layer logic in crates/plix-common/src/worldgen/generator.rs

**Checkpoint**: Deterministic generation works - US1 complete

---

## Phase 4: User Story 2 - Heightmap-Based Terrain (Priority: P1)

**Goal**: Terrain follows 2D noise heightmap with smooth chunk boundaries

**Independent Test**: Generate adjacent chunks and verify height continuity at boundaries

### Tests for User Story 2

- [ ] T019 [P] [US2] Add test `test_heightmap_range` verifying heights in [32, 96] in crates/plix-common/src/worldgen/height.rs
- [ ] T020 [P] [US2] Add test `test_heightmap_continuity` at chunk boundaries in crates/plix-common/src/worldgen/height.rs
- [ ] T021 [P] [US2] Add test `test_solid_below_surface_air_above` in crates/plix-common/src/worldgen/generator.rs

### Implementation for User Story 2

- [ ] T022 [US2] Create HeightModel struct in crates/plix-common/src/worldgen/height.rs
- [ ] T023 [US2] Implement HeightModel::new(config) in crates/plix-common/src/worldgen/height.rs
- [ ] T024 [US2] Implement surface_height(x, z, biome) with fBm noise in crates/plix-common/src/worldgen/height.rs
- [ ] T025 [US2] Wire HeightModel into ChunkGenerator in crates/plix-common/src/worldgen/generator.rs

**Checkpoint**: Heightmap terrain works - US2 complete

---

## Phase 5: User Story 5 - Per-Chunk Independent Generation (Priority: P1)

**Goal**: Each chunk generates independently without neighbor data

**Independent Test**: Generate chunk (5,0,3) with no other chunks loaded, verify valid output

### Tests for User Story 5

- [ ] T026 [P] [US5] Add test `test_chunk_independence_no_neighbors` in crates/plix-common/src/worldgen/generator.rs
- [ ] T027 [P] [US5] Add test `test_chunk_no_side_effects` verifying generation doesn't modify other state in crates/plix-common/src/worldgen/generator.rs
- [ ] T028 [P] [US5] Add test `test_parallel_generation_safe` generating multiple chunks concurrently in crates/plix-common/src/worldgen/generator.rs

### Implementation for User Story 5

- [ ] T029 [US5] Verify ChunkGenerator uses only (seed, coord) inputs - no external state in crates/plix-common/src/worldgen/generator.rs
- [ ] T030 [US5] Add `seed(&self) -> u64` accessor to ChunkGenerator in crates/plix-common/src/worldgen/generator.rs
- [ ] T031 [US5] Document thread-safety guarantees in ChunkGenerator rustdoc in crates/plix-common/src/worldgen/generator.rs

**Checkpoint**: Independent generation works - US5 complete, P1 stories done

---

## Phase 6: User Story 3 - Basic Biome System (Priority: P2)

**Goal**: Three biomes (plains, mountains, desert) with per-block selection

**Independent Test**: Generate chunks at varied positions, verify all 3 biomes appear with correct blocks

### Tests for User Story 3

- [ ] T032 [P] [US3] Add test `test_biome_plains_blocks` in crates/plix-common/src/worldgen/biome.rs
- [ ] T033 [P] [US3] Add test `test_biome_mountains_blocks` in crates/plix-common/src/worldgen/biome.rs
- [ ] T034 [P] [US3] Add test `test_biome_desert_blocks` in crates/plix-common/src/worldgen/biome.rs
- [ ] T035 [P] [US3] Add test `test_biome_continuity_at_boundaries` in crates/plix-common/src/worldgen/biome.rs

### Implementation for User Story 3

- [ ] T036 [US3] Create Biome enum (Plains, Mountains, Desert) in crates/plix-common/src/worldgen/biome.rs
- [ ] T037 [US3] Implement Biome::surface_block() returning biome-specific BlockType in crates/plix-common/src/worldgen/biome.rs
- [ ] T038 [US3] Implement Biome::subsurface_block() returning biome-specific BlockType in crates/plix-common/src/worldgen/biome.rs
- [ ] T039 [US3] Implement Biome::height_amplitude() returning multiplier in crates/plix-common/src/worldgen/biome.rs
- [ ] T040 [US3] Create BiomeModel struct in crates/plix-common/src/worldgen/biome.rs
- [ ] T041 [US3] Implement BiomeModel::biome_at(x, z) with dual-noise selection in crates/plix-common/src/worldgen/biome.rs
- [ ] T042 [US3] Wire BiomeModel into ChunkGenerator in crates/plix-common/src/worldgen/generator.rs

**Checkpoint**: Biome system works - US3 complete

---

## Phase 7: User Story 4 - Layer-Based Block Placement (Priority: P2)

**Goal**: Correct layers: bedrock at y=0, stone fill, subsurface, surface, air above

**Independent Test**: Examine column at any position, verify block types at each depth

### Tests for User Story 4

- [ ] T043 [P] [US4] Add test `test_bedrock_at_y0` in crates/plix-common/src/worldgen/generator.rs
- [ ] T044 [P] [US4] Add test `test_surface_block_correct` in crates/plix-common/src/worldgen/generator.rs
- [ ] T045 [P] [US4] Add test `test_subsurface_3_layers` in crates/plix-common/src/worldgen/generator.rs
- [ ] T046 [P] [US4] Add test `test_stone_fill_below_subsurface` in crates/plix-common/src/worldgen/generator.rs
- [ ] T047 [P] [US4] Add test `test_air_above_surface` in crates/plix-common/src/worldgen/generator.rs
- [ ] T048 [P] [US4] Add test `test_negative_y_chunks_all_air` in crates/plix-common/src/worldgen/generator.rs

### Implementation for User Story 4

- [ ] T049 [US4] Verify block_at() implements correct layer logic in crates/plix-common/src/worldgen/generator.rs
- [ ] T050 [US4] Add negative Y chunk handling (return all AIR) in crates/plix-common/src/worldgen/generator.rs
- [ ] T051 [US4] Add config.subsurface_depth usage in block_at() in crates/plix-common/src/worldgen/generator.rs

**Checkpoint**: Layer system works - US4 complete

---

## Phase 8: User Story 6 - Generation Performance (Priority: P3)

**Goal**: <10ms per chunk, 512 chunks in <5s

**Independent Test**: Time 100 chunk generations, verify average <10ms

### Tests for User Story 6

- [ ] T052 [P] [US6] Add benchmark test `test_chunk_generation_under_10ms` in crates/plix-common/src/worldgen/generator.rs

### Implementation for User Story 6

- [ ] T053 [US6] Add GenerationMetrics struct in crates/plix-common/src/worldgen/config.rs
- [ ] T054 [US6] Add metrics tracking to ChunkGenerator (chunks_generated_total, etc.) in crates/plix-common/src/worldgen/generator.rs
- [ ] T055 [US6] Add metrics() accessor to ChunkGenerator in crates/plix-common/src/worldgen/generator.rs

**Checkpoint**: Performance validated - US6 complete

---

## Phase 9: Integration (Client & Server)

**Purpose**: Connect generation to existing chunk streaming systems

### Client Integration

- [ ] T056 Add get_or_generate_chunk() method to ChunkedWorld in crates/plix-common/src/world.rs
- [ ] T057 Add ChunkGenerator field or parameter to client chunk loading in crates/plix-client/src/chunk_manager.rs
- [ ] T058 Call mark_dirty() for newly generated chunks in crates/plix-client/src/chunk_manager.rs
- [ ] T059 Add integration test `test_generated_world_renders` in crates/plix-client/src/world.rs

### Server Integration

- [ ] T060 Add ChunkGenerator to server world state in crates/plix-server/src/world.rs (or equivalent)
- [ ] T061 Generate chunks on demand when requested by clients in crates/plix-server/src/world.rs
- [ ] T062 Add test `test_server_client_same_chunks` verifying identical generation in crates/plix-common/src/worldgen/generator.rs

**Checkpoint**: Full pipeline connected - generation works in game

---

## Phase 10: Polish & Validation

**Purpose**: Final validation, non-regression, documentation

- [ ] T063 Run all worldgen tests with `cargo test -p plix-common worldgen`
- [ ] T064 [P] Run `cargo clippy --all-targets` and fix any warnings
- [ ] T065 [P] Run `cargo fmt --all` to ensure formatting
- [ ] T066 Add inline rustdoc for all public types and methods in crates/plix-common/src/worldgen/
- [ ] T067 Verify success criteria SC-001 through SC-008 via test suite
- [ ] T068 [P] Verify arena loading still works (non-regression) with `cargo test -p plix-arena`
- [ ] T069 Run quickstart.md validation scenarios manually

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - verify baseline
- **Foundational (Phase 2)**: Depends on Setup - adds noise infrastructure
- **User Stories (Phase 3-8)**: All depend on Foundational completion
  - US1, US2, US5 are all P1 priority - complete in order
  - US3, US4 are P2 priority - complete after P1 stories
  - US6 is P3 priority - complete last
- **Integration (Phase 9)**: Depends on all US complete
- **Polish (Phase 10)**: Depends on all phases complete

### User Story Dependencies

- **US1 (Determinism)**: Foundation only - creates ChunkGenerator
- **US2 (Heightmap)**: Foundation + US1 skeleton - adds HeightModel
- **US5 (Independence)**: Foundation + US1 - verifies pure function design
- **US3 (Biomes)**: Foundation - adds BiomeModel, wires to generator
- **US4 (Layers)**: Foundation + US3 (needs biome blocks) - refines block_at()
- **US6 (Performance)**: All previous - benchmarks complete system

### Within Each User Story

- Tests FIRST, verify they FAIL before implementation
- Implementation follows test definitions
- Verify tests PASS after implementation

### Parallel Opportunities

**Phase 1 (Setup)**: T005 can run in parallel with T001-T004

**Phase 2 (Foundational)**: T011 can run in parallel once T008-T010 done

**Phase 3-8 (User Stories)**:
- All tests within a story marked [P] can run in parallel
- P1 stories (US1, US2, US5) should complete before P2 (US3, US4)
- Within P1: US1 → US2 → US5 (US2 uses US1 generator, US5 validates design)

**Phase 10 (Polish)**: T064, T065, T068 can run in parallel

---

## Parallel Example: User Story 3 Tests

```bash
# Launch all US3 tests together:
Task: "Add test test_biome_plains_blocks in crates/plix-common/src/worldgen/biome.rs"
Task: "Add test test_biome_mountains_blocks in crates/plix-common/src/worldgen/biome.rs"
Task: "Add test test_biome_desert_blocks in crates/plix-common/src/worldgen/biome.rs"
Task: "Add test test_biome_continuity_at_boundaries in crates/plix-common/src/worldgen/biome.rs"
```

---

## Implementation Strategy

### MVP First (User Stories 1, 2, 5 Only)

1. Complete Phase 1: Setup (add noise dependency)
2. Complete Phase 2: Foundational (NoiseSource, WorldGenConfig)
3. Complete Phase 3: US1 - Deterministic Generation
4. Complete Phase 4: US2 - Heightmap Terrain
5. Complete Phase 5: US5 - Independent Generation
6. **STOP and VALIDATE**: Test P1 stories work together
7. Deploy/demo if ready - basic terrain generation works

### Full Implementation

1. Complete MVP (P1 stories)
2. Add US3 - Biome system
3. Add US4 - Layer refinement
4. Add US6 - Performance metrics
5. Complete Integration (Phase 9)
6. Polish and validate (Phase 10)

---

## Notes

- [P] tasks = different files or independent tests, no dependencies
- [Story] label maps task to specific user story for traceability
- This feature creates NEW files - no conflicts with existing code
- Foundational work is sequential (noise depends on config)
- User input task list (T001-T015) mapped to user stories and expanded with test tasks
- Research.md decisions embedded in implementation tasks
- data-model.md entities mapped to specific file paths
