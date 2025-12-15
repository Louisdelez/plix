# Tasks: Server-Authoritative Block Interaction

**Input**: Design documents from `/specs/004-block-interaction/`
**Prerequisites**: plan.md (required), spec.md (required), data-model.md, contracts/block-protocol.md

**Tests**: Unit tests included for server validation logic (per constitution V. Code Quality).

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4, US5)
- Include exact file paths in descriptions

## Path Conventions

Multi-crate workspace structure:
- **plix-common**: `crates/plix-common/src/` - Shared types and protocol
- **plix-server**: `crates/plix-server/src/` - Server logic
- **plix-client**: `crates/plix-client/src/` - Client logic
- **plix-arena**: `crates/plix-arena/src/` - Arena/world data
- **Tests**: `crates/plix-server/tests/`

---

## Phase 1: Protocol & Types (Shared Infrastructure)

**Purpose**: Define block edit message types used by all user stories

- [x] T001 [P] Add BlockEditKind enum (Place/Remove) in crates/plix-common/src/protocol/messages.rs
- [x] T002 [P] Add BlockEditRejectReason enum with all reason codes in crates/plix-common/src/protocol/messages.rs
- [x] T003 [P] Add BlockEditRequest struct in crates/plix-common/src/protocol/messages.rs
- [x] T004 [P] Add BlockEditApplied struct in crates/plix-common/src/protocol/messages.rs
- [x] T005 [P] Add BlockEditRejected struct in crates/plix-common/src/protocol/messages.rs
- [x] T006 Add BlockEdit variant to ClientMessage enum in crates/plix-common/src/protocol/messages.rs
- [x] T007 Add BlockEditApplied and BlockEditRejected variants to GameEvent enum in crates/plix-common/src/protocol/messages.rs
- [x] T008 Verify bincode serialization roundtrip for new message types with `cargo test -p plix-common`

**Checkpoint**: Protocol types compile and serialize correctly

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before user story work

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T009 Add set_block(pos, block_type) method to LoadedArena in crates/plix-arena/src/lib.rs
- [x] T010 Add is_in_bounds(pos) method to LoadedArena in crates/plix-arena/src/lib.rs
- [x] T011 Add last_edit_tick field to ServerPlayer struct in crates/plix-server/src/session.rs
- [x] T012 Create crates/plix-server/src/sim/block_edit.rs with BlockEditSystem struct and constants (MAX_EDIT_RANGE=5.0, EDIT_COOLDOWN_TICKS=15)
- [x] T013 Add mod block_edit to crates/plix-server/src/sim/mod.rs
- [x] T014 Verify server builds with `cargo build -p plix-server`

**Checkpoint**: Foundation ready - user story implementation can begin

---

## Phase 3: User Story 1 - Remove Block (Priority: P1) 🎯 MVP

**Goal**: Players can remove blocks and see them disappear for all connected clients in real-time.

**Independent Test**: Start server + 2 clients, player A removes a block, both see it disappear.

### Tests for User Story 1

- [x] T015 [P] [US1] Unit test: validate_remove rejects out-of-bounds in crates/plix-server/tests/block_edit_test.rs
- [x] T016 [P] [US1] Unit test: validate_remove rejects out-of-range in crates/plix-server/tests/block_edit_test.rs
- [x] T017 [P] [US1] Unit test: validate_remove rejects air cell (CellEmpty) in crates/plix-server/tests/block_edit_test.rs
- [x] T018 [P] [US1] Unit test: validate_remove rejects dead player in crates/plix-server/tests/block_edit_test.rs
- [x] T019 [P] [US1] Unit test: validate_remove rejects rate-limited player in crates/plix-server/tests/block_edit_test.rs
- [x] T020 [P] [US1] Unit test: validate_remove accepts valid request in crates/plix-server/tests/block_edit_test.rs

### Server Implementation for User Story 1

- [x] T021 [US1] Implement is_in_bounds validation in BlockEditSystem in crates/plix-server/src/sim/block_edit.rs
- [x] T022 [US1] Implement is_in_range validation (distance check) in BlockEditSystem in crates/plix-server/src/sim/block_edit.rs
- [x] T023 [US1] Implement is_rate_limited validation in BlockEditSystem in crates/plix-server/src/sim/block_edit.rs
- [x] T024 [US1] Implement validate_remove method combining all checks in BlockEditSystem in crates/plix-server/src/sim/block_edit.rs
- [x] T025 [US1] Handle ClientMessage::BlockEdit in server packet handler in crates/plix-server/src/lib.rs
- [x] T026 [US1] Process remove edits in simulate_tick: validate, apply, update last_edit_tick in crates/plix-server/src/lib.rs
- [x] T027 [US1] Broadcast BlockEditApplied event to all clients after successful remove in crates/plix-server/src/lib.rs
- [x] T028 [US1] Send BlockEditRejected event to requester on validation failure in crates/plix-server/src/lib.rs

### Client Implementation for User Story 1

- [x] T029 [P] [US1] Create crates/plix-client/src/raycast.rs with RaycastHit struct
- [x] T030 [US1] Implement raycast_blocks DDA algorithm in crates/plix-client/src/raycast.rs
- [x] T031 [US1] Add remove_block input flag to InputManager in crates/plix-client/src/input.rs
- [x] T032 [US1] Map LMB to remove_block action in handle_mouse_button in crates/plix-client/src/input.rs
- [x] T033 [P] [US1] Create crates/plix-client/src/world.rs with ClientWorld struct wrapping LoadedArena
- [x] T034 [US1] Implement apply_edit and dirty flag methods in ClientWorld in crates/plix-client/src/world.rs
- [x] T035 [US1] On remove_block input: raycast, send BlockEditRequest(Remove) in crates/plix-client/src/lib.rs
- [x] T036 [US1] Handle BlockEditApplied event: update ClientWorld, mark dirty in crates/plix-client/src/lib.rs
- [x] T037 [US1] Trigger mesh rebuild when ClientWorld is dirty in crates/plix-client/src/render/voxels.rs

**Checkpoint**: Remove block works end-to-end. Verify with 2 clients.

---

## Phase 4: User Story 2 - Place Block (Priority: P1)

**Goal**: Players can place blocks and see them appear for all connected clients in real-time.

**Independent Test**: Start server + 2 clients, player A places a block, both see it appear.

### Tests for User Story 2

- [x] T038 [P] [US2] Unit test: validate_place rejects non-air cell (CellNotEmpty) in crates/plix-server/tests/block_edit_test.rs
- [x] T039 [P] [US2] Unit test: validate_place rejects player collision in crates/plix-server/tests/block_edit_test.rs
- [x] T040 [P] [US2] Unit test: validate_place accepts valid request in crates/plix-server/tests/block_edit_test.rs

### Server Implementation for User Story 2

- [x] T041 [US2] Implement would_collide_with_player AABB check in BlockEditSystem in crates/plix-server/src/sim/block_edit.rs
- [x] T042 [US2] Implement validate_place method combining all checks in BlockEditSystem in crates/plix-server/src/sim/block_edit.rs
- [x] T043 [US2] Process place edits in simulate_tick: validate, apply, update last_edit_tick in crates/plix-server/src/lib.rs
- [x] T044 [US2] Broadcast BlockEditApplied event to all clients after successful place in crates/plix-server/src/lib.rs

### Client Implementation for User Story 2

- [x] T045 [US2] Add place_block input flag to InputManager in crates/plix-client/src/input.rs
- [x] T046 [US2] Map RMB to place_block action in handle_mouse_button in crates/plix-client/src/input.rs
- [x] T047 [US2] On place_block input: raycast, calculate adjacent cell (pos + face_normal), send BlockEditRequest(Place) in crates/plix-client/src/lib.rs

**Checkpoint**: Place block works end-to-end. Verify with 2 clients removing AND placing blocks.

---

## Phase 5: User Story 3 - Server Validation (Priority: P2)

**Goal**: All block edits are validated server-side, preventing cheating.

**Independent Test**: Send malformed edit requests directly, verify all rejected with correct reasons.

### Tests for User Story 3

- [x] T048 [P] [US3] Unit test: validate rejects InvalidPhase (not Playing) in crates/plix-server/tests/block_edit_test.rs
- [x] T049 [P] [US3] Unit test: all BlockEditRejectReason codes are reachable in crates/plix-server/tests/block_edit_test.rs

### Server Implementation for User Story 3

- [x] T050 [US3] Add match phase check (must be Playing) to validation in crates/plix-server/src/sim/block_edit.rs
- [x] T051 [US3] Log rejected edits with reason for debugging in crates/plix-server/src/lib.rs
- [x] T052 [US3] Run `cargo test -p plix-server` and verify all validation tests pass

**Checkpoint**: Server rejects all invalid edit types with correct reason codes.

---

## Phase 6: User Story 4 - Late Joiner World Sync (Priority: P2)

**Goal**: Players joining mid-match see the correct current world state including prior edits.

**Independent Test**: Client A edits world, Client B joins later, B sees same state as A.

**⚠️ LIMITATION**: Full arena state (131KB) exceeds UDP packet size (1389 bytes). Current workaround:
- Server sends arena metadata only (not full block data)
- Client loads arena locally from file
- Block edits replicate as deltas (BlockEditApplied/Rejected)
- **Late joiners see stale arena** (do not receive prior edits)
- **TODO**: Implement fragmented/segmented transfer for large payloads

### Implementation for User Story 4

- [x] T053 [US4] ~~Verify Connected message sends current arena state (with edits)~~ **REVERTED** - sends metadata only
- [x] T054 [US4] ~~Initialize ClientWorld from Connected.arena_data~~ **REVERTED** - client loads locally
- [x] T055 [US4] ~~Build initial mesh from ClientWorld on connect~~ **REVERTED** - uses local arena
- [ ] T053b [US4] **TODO**: Implement fragmented arena transfer over reliable channel
- [ ] T054b [US4] **TODO**: Reassemble fragmented arena on client
- [ ] T055b [US4] **TODO**: Build mesh from reassembled server arena

### Manual Validation for User Story 4

- [ ] T056 [US4] Manual test: Start server, Client A places/removes blocks, Client B joins later, verify B sees edited world
  - **NOTE**: Currently expected to FAIL - late joiners get stale arena

**Checkpoint**: ~~Late joiners see consistent world state.~~ **BLOCKED** - needs fragmentation

---

## Phase 7: User Story 5 - Debug Feedback (Priority: P3)

**Goal**: Debug HUD shows feedback for block actions (placed/removed/rejected).

**Independent Test**: Perform valid and invalid actions, verify debug messages appear.

### Implementation for User Story 5

- [x] T057 [US5] Add debug message display infrastructure in crates/plix-client/src/lib.rs or ui module
- [x] T058 [US5] Show "Block removed" on successful remove in crates/plix-client/src/lib.rs
- [x] T059 [US5] Show "Block placed" on successful place in crates/plix-client/src/lib.rs
- [x] T060 [US5] Handle BlockEditRejected event: show "Edit rejected: {reason}" in crates/plix-client/src/lib.rs

**Checkpoint**: Debug feedback visible for all action types.

---

## Phase 8: Polish & Non-Regression

**Purpose**: Final validation and cleanup

### Non-Regression Tests

- [x] T061 Run `cargo test --workspace` and verify all tests pass
- [x] T062 Run `cargo clippy --workspace` and fix any warnings
- [x] T063 Run `cargo fmt --check` and fix any formatting issues
- [ ] T064 Run load test: `./scripts/run_load_test.sh` and verify stable (bots ignore edits)
- [ ] T065 Verify headless server mode still works

### Manual E2E Validation

- [ ] T066 Manual test: 2 windowed clients place/remove blocks, verify both see same edits in real-time
- [ ] T067 Manual test: Verify invalid edits rejected (try out-of-range, rapid fire, etc.)

### Optional Cleanup

- [ ] T068 [P] Run `cargo fix --workspace` to reduce warnings
- [x] T069 [P] Add doc comments to public API in BlockEditSystem

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Protocol)**: No dependencies - can start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Phase 2 completion
  - US1 and US2 are both P1 but share foundation; implement sequentially (US1 first, then US2 builds on it)
  - US3, US4, US5 can proceed after US1+US2 complete
- **Phase 8 (Polish)**: Depends on all user stories complete

### User Story Dependencies

- **US1 (Remove Block)**: Depends on Phase 2 - Foundation for all editing
- **US2 (Place Block)**: Depends on US1 - Shares raycast, input handling, world state
- **US3 (Server Validation)**: Can start after Phase 2 but integrates with US1/US2
- **US4 (Late Join)**: Depends on US1/US2 working - Tests replication
- **US5 (Debug Feedback)**: Depends on US1/US2 - Shows feedback for their actions

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Server validation before server application
- Server implementation before client
- Core functionality before integration
- Manual verification at each checkpoint

### Parallel Opportunities

**Phase 1** (all independent protocol types):
```
T001 || T002 || T003 || T004 || T005
```

**US1 Tests** (independent test files):
```
T015 || T016 || T017 || T018 || T019 || T020
```

**US1 Client** (independent modules):
```
T029 (raycast.rs) || T033 (world.rs)
```

**US2 Tests**:
```
T038 || T039 || T040
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Protocol types
2. Complete Phase 2: Foundational infrastructure
3. Complete Phase 3: US1 - Remove Block
4. Complete Phase 4: US2 - Place Block
5. **STOP and VALIDATE**: Test both operations with 2 clients
6. This is a deployable MVP - world is now interactive

### Full Feature

1. Complete MVP (above)
2. Add Phase 5: US3 - Server Validation (hardening)
3. Add Phase 6: US4 - Late Joiner Sync
4. Add Phase 7: US5 - Debug Feedback
5. Complete Phase 8: Non-regression validation

### Task Count by Phase

| Phase | Story | Task Count |
|-------|-------|------------|
| Phase 1 | Protocol | 8 |
| Phase 2 | Foundational | 6 |
| Phase 3 | US1 - Remove Block | 23 |
| Phase 4 | US2 - Place Block | 10 |
| Phase 5 | US3 - Server Validation | 5 |
| Phase 6 | US4 - Late Join | 4 |
| Phase 7 | US5 - Debug Feedback | 4 |
| Phase 8 | Polish | 9 |
| **Total** | | **69** |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Each user story can be tested independently at its checkpoint
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate independently
- MVP = US1 + US2 (minimal interactive world)
