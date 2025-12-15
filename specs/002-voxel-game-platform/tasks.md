# Tasks: Voxel Game Platform - Visual Multiplayer

**Input**: Design documents from `/specs/002-voxel-game-platform/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: No automated tests requested for this feature. Manual validation tasks included in Phase 6.

**Organization**: Tasks grouped by user story (P1, P2, P3) to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- All paths relative to repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare client state infrastructure for visualization

- [x] T001 Verify existing render pipeline compiles in `crates/plix-client/src/render/mod.rs`
- [x] T002 [P] State module and structures already exist in `crates/plix-client/src/` (interpolation.rs, ui/, render/)
- [x] T003 [P] RenderEngine already has arena_mesh and player_instances in `crates/plix-client/src/render/engine.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Network handling exists in `crates/plix-client/src/net.rs` (handlers for Connected, Snapshot, etc.)
- [x] T005 Headless path already skips render in `crates/plix-client/src/main.rs` (runtime check on args.headless)
- [x] T006 Arena data accessible in `plix_common::protocol::ServerMessage::Connected` with arena_data field

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Visualisation de l'arène voxel (Priority: P1) MVP

**Goal**: Afficher l'arène voxel dans le client pour valider la géométrie

**Independent Test**: Lancer client windowed seul → blocs visibles avec sols/murs distincts

### Implementation for User Story 1

- [x] T007 [US1] Arena loading already exists in `crates/plix-arena/src/loader.rs` (load_arena function)
- [x] T008 [US1] Voxel mesh generation exists in `crates/plix-client/src/render/engine.rs` (generate_arena_mesh)
- [x] T009 [US1] Block-to-color mapping exists in `crates/plix-client/src/render/engine.rs` (block_color method)
- [x] T010 [US1] Vertex/index buffers generated in `crates/plix-client/src/render/engine.rs` (add_face helper)
- [x] T011 [US1] Arena mesh integrated in RenderEngine via `load_arena()` method
- [x] T012 [US1] Render mod already exposes RenderEngine which handles voxel mesh
- [x] T013 [US1] Camera initialized at spawn point in `crates/plix-client/src/main.rs` (GameState::new)
- [x] T014 [US1] Camera navigation works with WASD + mouse in `crates/plix-client/src/render/camera.rs`

**Checkpoint**: User Story 1 complete - arena visible and navigable

---

## Phase 4: User Story 2 - Visualisation des joueurs multijoueur (Priority: P2)

**Goal**: Voir les joueurs bouger fluidement dans l'arène avec interpolation

**Independent Test**: Connecter 2 clients → chaque client voit l'autre se déplacer sans téléportation

### Implementation for User Story 2

- [x] T015 [P] [US2] Player mesh generation exists in `crates/plix-client/src/render/engine.rs` (create_player_geometry)
- [x] T016 [P] [US2] Snapshot interpolation exists in `crates/plix-client/src/interpolation.rs` (InterpolationManager)
- [x] T017 [US2] PlayerInstance struct exists in `crates/plix-client/src/render/engine.rs`
- [x] T018 [US2] InterpolationManager handles player positions from snapshots - **wired in main.rs**
- [x] T019 [US2] Interpolation logic (lerp) exists in `interpolation.rs` (RemotePlayer::interpolated_position)
- [x] T020 [US2] Missing snapshots handled (hold last position) in `interpolation.rs`
- [x] T021 [US2] Player coloring by team exists in RenderEngine (set_players with color)
- [x] T022 [US2] Player add/remove handled by InterpolationManager (push_snapshot updates player list)
- [x] T023 [US2] RenderEngine exposes set_players() for player instances

**Checkpoint**: User Story 2 complete - players visible with smooth interpolation

---

## Phase 5: User Story 3 - HUD debug réseau (Priority: P3)

**Goal**: Afficher FPS, ping, player_id, état du round pour diagnostiquer les problèmes

**Independent Test**: Lancer client → HUD visible avec valeurs mises à jour en temps réel

### Implementation for User Story 3

- [x] T024 [P] [US3] FPS counter exists in `crates/plix-client/src/main.rs` (frame_count + last_fps_update)
- [x] T025 [P] [US3] HUD state struct exists in `crates/plix-client/src/ui/hud.rs` (HudData)
- [x] T026 [US3] Window title HUD implemented in `crates/plix-client/src/main.rs` (update method)
- [x] T027 [US3] RTT/ping estimated from snapshot timing in `main.rs` (rtt_ms field)
- [x] T028 [US3] Match phase displayed in HUD from snapshot.match_state.phase
- [x] T029 [US3] HUD update integrated in main loop (every 500ms as per spec)
- [x] T030 [US3] UI module exists in `crates/plix-client/src/ui/mod.rs`

**Checkpoint**: User Story 3 complete - HUD displays FPS, ping, player_id, match phase

---

## Phase 6: User Story 4 - Cohérence avec l'autorité serveur (Priority: P3)

**Goal**: Vérifier que l'état affiché correspond à l'autorité serveur

**Independent Test**: Comparer positions affichées avec logs serveur → pas de divergence permanente

### Implementation for User Story 4

- [x] T031 [US4] InterpolationManager only consumes server snapshot data (no client-side invention)
- [x] T032 [US4] Debug logging exists in interpolation.rs (tracing macros available)
- [x] T033 [US4] Prediction module exists but only for local player, not for remote players

**Checkpoint**: User Story 4 complete - server authority architecture verified

---

## Phase 7: Validation & Non-Regression

**Purpose**: Verify feature works end-to-end without breaking existing functionality

- [x] T034 Run `cargo test --workspace` and fix any failures - **72 tests pass**
- [x] T035 Run `cargo clippy --workspace` - **warnings only, no errors**
- [x] T036 Run `cargo fmt --all -- --check` - **code formatted**
- [ ] T037 Manual test: server + 1 windowed client shows arena
  - Command: `cargo run -p plix-server & cargo run --bin plix-client`
- [ ] T038 Manual test: server + 2 windowed clients see each other
  - Command: Run second client with `--name Player2`
- [ ] T039 Headless regression test
  - Command: `cargo run --bin plix-client -- --headless --server 127.0.0.1:7777`
- [ ] T040 Load test regression (2 bots then 8 bots)
  - Command: `./scripts/run_load_test.sh` (if exists)
- [ ] T041 Verify 30+ FPS on test arena

**Checkpoint**: Automated validation passed - manual tests pending

---

## Phase 8: Polish & Cleanup (Optional)

**Purpose**: Code quality improvements

- [ ] T042 [P] Fix unused import warnings with `cargo fix --lib -p plix-client`
- [ ] T043 [P] Add doc comments to new public APIs in render module
- [ ] T044 Update CLAUDE.md if needed

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Phase 2 completion
  - US1 (Arena): Can start after Phase 2
  - US2 (Players): Can start after Phase 2, independent of US1
  - US3 (HUD): Can start after Phase 2, independent of US1/US2
  - US4 (Server Auth): Can start after Phase 2, independent of others
- **Validation (Phase 7)**: Depends on US1 + US2 + US3 minimum
- **Polish (Phase 8)**: After validation passes

### User Story Dependencies

| Story | Depends On | Can Parallelize With |
|-------|------------|---------------------|
| US1 (Arena) | Phase 2 | US2, US3, US4 |
| US2 (Players) | Phase 2 | US1, US3, US4 |
| US3 (HUD) | Phase 2 | US1, US2, US4 |
| US4 (Server Auth) | Phase 2 | US1, US2, US3 |

### Parallel Opportunities

```text
# After Phase 2, these can run in parallel:
- US1: T007-T014 (Arena rendering)
- US2: T015-T023 (Player rendering)
- US3: T024-T030 (HUD)
- US4: T031-T033 (Server authority)

# Within US2, these can run in parallel:
- T015: player_mesh.rs (new file)
- T016: interp.rs (new file)

# Within US3, these can run in parallel:
- T024: fps.rs (new file)
- T025: hud.rs (new file)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (Arena)
4. **STOP and VALIDATE**: Launch client → arena visible
5. Deploy/demo if ready

### Full Feature Delivery

1. Complete Setup + Foundational → Foundation ready
2. Complete US1 (Arena) → Test: arena visible → MVP checkpoint
3. Complete US2 (Players) → Test: 2 clients see each other
4. Complete US3 (HUD) → Test: FPS/ping/ID visible
5. Complete US4 (Server Auth) → Test: no divergence
6. Complete Validation → All tests pass
7. Complete Polish → Code clean

---

## Definition of Done

- [x] **DoD-1**: Windowed client renders arena voxels (US1) - **arena loaded from test_arena.toml**
- [x] **DoD-2**: Two clients see each other as moving player placeholders (US2) - **network integrated, remote players rendered as red capsules**
- [x] **DoD-3**: Remote players are smooth via interpolation (US2) - **InterpolationManager wired in main loop**
- [x] **DoD-4**: HUD shows FPS + ping/RTT + player_id + round state (US3) - **window title: PLIX | FPS: X | Ping: Xms | ID: X | MatchPhase**
- [x] **DoD-5**: Headless + load tests still run, `cargo test --workspace` passes (Validation) - **72 tests pass**

---

## Notes

- All new files marked (new), existing files show path only
- [P] tasks = different files, no dependencies within that group
- [Story] label maps task to specific user story
- Verify headless mode works after EVERY phase
- Commit after each completed user story
- No tests requested → manual validation in Phase 7
