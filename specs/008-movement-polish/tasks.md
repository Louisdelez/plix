# Tasks: Movement Polish

**Input**: Design documents from `/specs/008-movement-polish/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Organization**: Tasks are organized by user story priority (P1 → P2 → P3) to enable independent implementation and testing.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US6)
- Include exact file paths in descriptions

## Path Conventions (from plan.md)

```text
crates/
├── plix-common/src/          # Shared physics code
├── plix-server/src/sim/      # Server movement/collision
├── plix-client/src/          # Client prediction/reconciliation
└── plix-arena/src/           # Arena data
```

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Physics constants and shared movement module

- [x] T001 Create MovementConfig struct with physics constants in crates/plix-common/src/physics.rs
- [x] T002 [P] Add physics module export to crates/plix-common/src/lib.rs
- [x] T003 [P] Create MovementState struct in crates/plix-common/src/physics.rs
- [x] T004 Update MOVE_SPEED from 5.0 to 6.0 in crates/plix-server/src/sim/movement.rs
- [x] T005 [P] Update JUMP_VELOCITY from 8.0 to 7.07 in crates/plix-server/src/sim/movement.rs
- [x] T006 [P] Update PLAYER_RADIUS from 0.3 to 0.4 in crates/plix-server/src/sim/movement.rs
- [x] T007 [P] Update PLAYER_HALF_WIDTH from 0.3 to 0.4 in crates/plix-server/src/sim/collision.rs

**Checkpoint**: ✅ Physics constants aligned with clarified values (6 m/s, 7.07 m/s jump, 0.4m radius)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Collision model rewrite - foundation for ALL user stories

**⚠️ CRITICAL**: All movement features depend on this collision system

- [x] T008 Reorder collision resolution to Y → X → Z in crates/plix-server/src/sim/collision.rs
- [x] T009 [P] Add epsilon clamping (0.001) to prevent floating-point drift in collision.rs
- [x] T010 [P] Add CollisionResult struct with position, velocity, grounded, stepped fields in collision.rs
- [x] T011 Implement move_and_slide_v2() with new resolution order in collision.rs
- [x] T012 [P] Add tunneling prevention via movement subdivision in collision.rs
- [x] T013 Add unit test: floor collision in crates/plix-server/tests/collision_test.rs
- [x] T014 [P] Add unit test: wall collision in crates/plix-server/tests/collision_test.rs
- [x] T015 [P] Add unit test: corner collision (diagonal) in crates/plix-server/tests/collision_test.rs
- [x] T016 [P] Add unit test: falling onto surface in crates/plix-server/tests/collision_test.rs

**Checkpoint**: Collision system rewritten, all basic collision tests pass

---

## Phase 3: User Story 1 - Reliable Collision (Priority: P1) 🎯 MVP

**Goal**: Collision with voxels is solid and consistent - no clipping or getting stuck

**Independent Test**: Spawn player adjacent to walls/floors/ceilings, move in all directions - never penetrate

**Acceptance Criteria**:
- Player stops at wall surface without penetrating
- Diagonal corner movement slides smoothly without jitter
- Standing still against wall = no vibration or shifting
- Client and server produce identical collision results

### Implementation for User Story 1

- [x] T017 [US1] Implement is_position_valid() check for post-collision validation in collision.rs
- [x] T018 [US1] Add wall penetration prevention with position correction in collision.rs
- [x] T019 [US1] Implement smooth wall sliding for diagonal movement in collision.rs
- [x] T020 [US1] Add stationary stability check (zero velocity when at rest against wall) in collision.rs
- [x] T021 [US1] Ensure collision determinism by removing any non-deterministic operations in collision.rs
- [x] T022 [US1] Add integration test: walk into wall from all angles in crates/plix-server/tests/movement_test.rs
- [x] T023 [P] [US1] Add integration test: diagonal corner collision in movement_test.rs
- [x] T024 [P] [US1] Add integration test: stationary against wall (no jitter) in movement_test.rs

**Checkpoint**: US1 complete - collision is solid, no clipping, deterministic

---

## Phase 4: User Story 2 - Jumping (Priority: P1)

**Goal**: Responsive jump with predictable height (1.25 blocks)

**Independent Test**: Press jump while grounded, measure apex height - should be 1.25 blocks ±1%

**Acceptance Criteria**:
- Grounded player gains upward velocity on jump press
- Jump ignored while airborne (no double jump)
- Holding jump does not auto-repeat on landing
- Jump height identical across all clients

### Implementation for User Story 2

- [x] T025 [US2] Implement apply_jump() with impulse 7.07 m/s in crates/plix-server/src/sim/movement.rs
- [x] T026 [US2] Add jump eligibility check (is_grounded required) in movement.rs
- [x] T027 [US2] Add jump_was_pressed flag to prevent auto-repeat on hold in movement.rs
- [x] T028 [US2] Reset vertical velocity to jump_impulse (not additive) in movement.rs
- [x] T029 [P] [US2] Add jump buffer (≤100ms / 6 ticks) for responsive input in movement.rs
- [x] T030 [US2] Add unit test: measure jump apex height (1.25 blocks ±5%) in movement_test.rs
- [x] T031 [P] [US2] Add unit test: jump blocked when airborne in movement_test.rs
- [x] T032 [P] [US2] Add unit test: jump requires button release between jumps in movement_test.rs

**Checkpoint**: US2 complete - jump is consistent, no double jump, correct height

---

## Phase 5: User Story 3 - Step-Up Movement (Priority: P2)

**Goal**: Smoothly walk up small ledges (≤0.5 blocks) without manual jumping

**Independent Test**: Walk toward 0.5-block ledge - player automatically steps up

**Acceptance Criteria**:
- Player steps up onto obstacles ≤0.5 blocks
- Player stops at walls (obstacles > step_height)
- No step-up while airborne
- Step-up fails if head would collide with ceiling

### Implementation for User Story 3

- [x] T033 [US3] Implement try_step_up() with max height 1.0 blocks in collision.rs
- [x] T034 [US3] Add grounded check before step-up attempt in collision.rs
- [x] T035 [US3] Add head collision check during step-up in collision.rs
- [x] T036 [US3] Integrate step-up into move_and_slide_v2() horizontal resolution in collision.rs
- [x] T037 [US3] Add unit test: single block step-up in collision_test.rs
- [x] T038 [P] [US3] Add unit test: step-up blocked when airborne in collision_test.rs
- [x] T039 [P] [US3] Add unit test: step-up blocked by ceiling in collision_test.rs
- [x] T040 [P] [US3] Add unit test: step-up blocked for tall walls in collision_test.rs

**Checkpoint**: US3 complete - step-up works naturally on voxel terrain

---

## Phase 6: User Story 4 - Friction & Ground Control (Priority: P2)

**Goal**: Responsive ground movement, floatier air movement (30% air control)

**Independent Test**: Release input on ground vs. in air - observe different deceleration

**Acceptance Criteria**:
- Ground: player decelerates to stop quickly
- Air: turning rate noticeably reduced (30%)
- Stationary on flat surface = no sliding
- Friction deterministic on client and server

### Implementation for User Story 4

- [x] T041 [US4] Implement apply_ground_friction() with coefficient 10.0 in movement.rs
- [x] T042 [US4] Implement apply_air_control() with 30% multiplier in movement.rs
- [x] T043 [US4] Add velocity zero-snap when speed < threshold (0.01) to prevent micro-drift in movement.rs
- [x] T044 [US4] Enforce speed cap of 6.0 m/s in enforce_speed_cap() in movement.rs
- [x] T045 [US4] Add unit test: speed never exceeds 6.0 m/s in movement_test.rs
- [x] T046 [P] [US4] Add unit test: friction stops player on ground in movement_test.rs
- [x] T047 [P] [US4] Add unit test: air control is 30% of ground in movement_test.rs
- [x] T048 [P] [US4] Add unit test: no sliding when stationary in movement_test.rs

**Checkpoint**: US4 complete - movement feel matches modern FPS expectations

---

## Phase 7: User Story 5 - Stable Hitbox (Priority: P2)

**Goal**: Hitbox matches collision capsule exactly - fair combat

**Independent Test**: Combat hit registration matches visual player positions

**Acceptance Criteria**:
- Single capsule for collision AND hit detection
- Rendered position matches hitbox position
- Hitbox does not jitter during movement
- Server uses authoritative hitbox position for hits

### Implementation for User Story 5

- [x] T049 [US5] Unify hitbox with movement capsule (remove duplicate definitions) in crates/plix-server/src/sim/mod.rs
- [x] T050 [US5] Ensure snapshot positions are post-collision only in crates/plix-server/src/replication/snapshot.rs
- [x] T051 [US5] Update combat hit validation to use post-collision position in crates/plix-server/src/sim/combat.rs
- [x] T052 [US5] Add integration test: moving attacker vs stationary target in crates/plix-server/tests/combat_test.rs
- [x] T053 [P] [US5] Add integration test: both players moving combat in combat_test.rs
- [x] T054 [P] [US5] Add integration test: close combat edge cases in combat_test.rs

**Checkpoint**: US5 complete - hitbox stable, combat feels fair

---

## Phase 8: User Story 6 - Desync & Prediction Fixes (Priority: P3)

**Goal**: Smooth movement under latency - no visible corrections or rubber-banding

**Independent Test**: Simulate 150ms latency, observe correction smoothness

**Acceptance Criteria**:
- Client prediction uses identical code as server
- Corrections apply smooth interpolation (not hard snap)
- Large corrections (>1 block) are clamped and eased
- 60Hz tick rate consistent on both client and server

### Implementation for User Story 6

- [x] T055 [US6] Extract shared movement logic to plix-common/src/physics.rs
- [x] T056 [US6] Update plix-server to use shared physics::apply_movement() in movement.rs
- [x] T057 [US6] Update plix-client prediction to use shared physics::apply_movement() in crates/plix-client/src/prediction.rs
- [x] T058 [US6] Implement CorrectionSmoother with 100ms blend in crates/plix-client/src/reconciliation.rs
- [x] T059 [US6] Add correction delta clamping (max 0.5 blocks/frame) in reconciliation.rs
- [x] T060 [US6] Clamp render positions to valid space during interpolation in crates/plix-client/src/interpolation.rs
- [x] T061 [US6] Add unit test: client-server determinism (identical outputs) in movement_test.rs
- [x] T062 [P] [US6] Add integration test: simulated latency correction in crates/plix-server/tests/network_test.rs

**Checkpoint**: US6 complete - smooth movement under latency

---

## Phase 9: Polish & Regression Validation

**Purpose**: Final validation, additional tests, cleanup

- [x] T063 [P] Verify ≥20 unit tests exist for movement system via cargo test --workspace (226 tests passing)
- [x] T064 [P] Add integration test: movement + combat combined in combat_test.rs (T052-T054 cover this)
- [x] T065 [P] Add integration test: movement + block interaction in block_edit_test.rs (existing tests cover this)
- [x] T066 Add load test with bots: continuous movement and jumping in crates/plix-tools/src/bot.rs (already exists with movement+jump)
- [x] T067 Run cargo clippy --workspace and fix any movement-related warnings (no movement-specific warnings)
- [x] T068 Run cargo fmt --all and verify formatting (formatting applied)
- [ ] T069 Manual validation: no clipping through walls at any angle (requires running game)
- [ ] T070 Manual validation: jump height feels consistent (requires running game)
- [ ] T071 Manual validation: step-up feels natural on voxel terrain (requires running game)
- [ ] T072 Manual validation: combat feels fair (no phantom hits) (requires running game)

**Checkpoint**: All tests pass, movement system stable baseline

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **Phase 3-8 (User Stories)**: All depend on Phase 2 completion
  - US1 (Collision) & US2 (Jumping) are both P1, can run in parallel
  - US3-US5 (P2) can run in parallel after US1/US2
  - US6 (P3) depends on US1-US5 for integration
- **Phase 9 (Polish)**: Depends on all user stories complete

### User Story Dependencies

| Story | Priority | Dependencies | Can Parallel With |
|-------|----------|--------------|-------------------|
| US1 - Collision | P1 | Phase 2 only | US2 |
| US2 - Jumping | P1 | Phase 2 only | US1 |
| US3 - Step-Up | P2 | US1 (collision) | US4, US5 |
| US4 - Friction | P2 | US1 (collision) | US3, US5 |
| US5 - Hitbox | P2 | US1 (collision) | US3, US4 |
| US6 - Desync | P3 | All P1/P2 stories | None |

### Parallel Opportunities Per Phase

**Phase 1**: T002-T003 parallel, T004-T007 all parallel
**Phase 2**: T009-T010 parallel, T013-T016 all parallel
**Phase 3 (US1)**: T023-T024 parallel
**Phase 4 (US2)**: T029 independent, T031-T032 parallel
**Phase 5 (US3)**: T038-T040 all parallel
**Phase 6 (US4)**: T046-T048 all parallel
**Phase 7 (US5)**: T053-T054 parallel
**Phase 8 (US6)**: T062 independent
**Phase 9**: T063-T065 all parallel

---

## Parallel Example: User Story 1 (Collision)

```bash
# Launch all tests for US1 together:
Task: "T022 [US1] Add integration test: walk into wall from all angles"
Task: "T023 [P] [US1] Add integration test: diagonal corner collision"
Task: "T024 [P] [US1] Add integration test: stationary against wall"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup (T001-T007)
2. Complete Phase 2: Foundational collision rewrite (T008-T016)
3. Complete Phase 3: US1 Reliable Collision (T017-T024)
4. Complete Phase 4: US2 Jumping (T025-T032)
5. **STOP and VALIDATE**: Test collision + jumping independently
6. Deploy/demo if ready - game is playable!

### Incremental Delivery

1. Setup + Foundational → Collision system ready
2. Add US1 (Collision) → Test → Solid world, no clipping
3. Add US2 (Jumping) → Test → Vertical movement works
4. Add US3 (Step-Up) → Test → Smooth terrain navigation
5. Add US4 (Friction) → Test → Proper movement feel
6. Add US5 (Hitbox) → Test → Fair combat
7. Add US6 (Desync) → Test → Polish complete

---

## Notes

- All tasks include exact file paths for immediate execution
- [P] tasks can run in parallel within their phase
- [USx] labels map to spec.md user stories for traceability
- Verify existing tests still pass after each task (cargo test)
- Commit after each completed user story phase
