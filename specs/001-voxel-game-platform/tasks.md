# Tasks: Plix MVP v0.1 - Authoritative Server Network Architecture

**Input**: Design documents from `/specs/001-voxel-game-platform/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: Unit and integration tests are INCLUDED as this is a network-critical MVP requiring validation.

**Organization**: Tasks are organized by technical phase due to MVP focus on network architecture validation. User Stories US1 (Server Join) and US2 (Fair PvP Combat) are the primary MVP targets; US3-US8 are deferred to post-MVP.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2 for MVP)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Project Initialization)

**Purpose**: Initialize Cargo workspace and project infrastructure

- [ ] T001 Create Cargo workspace manifest in Cargo.toml
- [ ] T002 [P] Create crate plix-common with Cargo.toml in crates/plix-common/
- [ ] T003 [P] Create crate plix-net with Cargo.toml in crates/plix-net/
- [ ] T004 [P] Create crate plix-server with Cargo.toml in crates/plix-server/
- [ ] T005 [P] Create crate plix-client with Cargo.toml in crates/plix-client/
- [ ] T006 [P] Create crate plix-arena with Cargo.toml in crates/plix-arena/
- [ ] T007 [P] Create crate plix-tools with Cargo.toml in crates/plix-tools/
- [ ] T008 [P] Create rustfmt.toml with project formatting rules
- [ ] T009 [P] Create scripts/run_server.sh and scripts/run_client.sh
- [ ] T010 Create .github/workflows/ci.yml with lint + format + test + build matrix

**Checkpoint**: `cargo build` succeeds for all crates, CI pipeline runs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

### 2.1 Common Types (plix-common)

- [ ] T011 Create math types (Vec3, Rotation, AABB) in crates/plix-common/src/math.rs
- [ ] T012 [P] Create identifier types (PlayerId, EntityId, Tick, InputSeq) in crates/plix-common/src/types.rs
- [ ] T013 [P] Create time utilities (Tick math, timestamps) in crates/plix-common/src/time.rs
- [ ] T014 Create lib.rs exporting all modules in crates/plix-common/src/lib.rs

### 2.2 Protocol Definition (plix-common)

- [ ] T015 Create protocol module with version constant in crates/plix-common/src/protocol/mod.rs
- [ ] T016 Define ClientMessage enum (Connect, Disconnect, Input, SnapshotAck) in crates/plix-common/src/protocol/messages.rs
- [ ] T017 Define ServerMessage enum (Connected, Rejected, Kicked, Snapshot, Event) in crates/plix-common/src/protocol/messages.rs
- [ ] T018 Define PlayerSnapshot, WorldSnapshot, MatchState structs in crates/plix-common/src/protocol/messages.rs
- [ ] T019 Define GameEvent enum (PlayerJoined, PlayerLeft, PlayerDied, etc.) in crates/plix-common/src/protocol/messages.rs
- [ ] T020 Implement binary codec (encode/decode) in crates/plix-common/src/protocol/codec.rs
- [ ] T021 Create protocol codec tests (roundtrip, limits) in crates/plix-common/tests/protocol_codec_tests.rs

### 2.3 Network Transport (plix-net)

- [ ] T022 Create UDP socket wrapper in crates/plix-net/src/transport.rs
- [ ] T023 Implement packet header (version, channel, sequence, ack) in crates/plix-net/src/packet.rs
- [ ] T024 Create unreliable channel in crates/plix-net/src/channel/unreliable.rs
- [ ] T025 [P] Create reliable channel with ACK/resend in crates/plix-net/src/channel/reliable.rs
- [ ] T026 [P] Create ordered reliable channel in crates/plix-net/src/channel/ordered.rs
- [ ] T027 Create channel module exporting all channels in crates/plix-net/src/channel/mod.rs
- [ ] T028 Implement connection state machine (handshake, keepalive, timeout) in crates/plix-net/src/connection.rs
- [ ] T029 Implement RTT, jitter, packet loss measurement in crates/plix-net/src/metrics.rs
- [ ] T030 Create lib.rs with public API in crates/plix-net/src/lib.rs
- [ ] T031 Create network reliability tests (reorder, loss, resend) in crates/plix-net/tests/reliability_tests.rs

### 2.4 Arena System (plix-arena)

- [ ] T032 Define arena format structs (ArenaMetadata, SpawnPoint, BlockType) in crates/plix-arena/src/format.rs
- [ ] T033 Implement arena loader from TOML in crates/plix-arena/src/loader.rs
- [ ] T034 Implement arena validation (bounds, spawn points) in crates/plix-arena/src/validate.rs
- [ ] T035 Create spawn point logic in crates/plix-arena/src/spawn.rs
- [ ] T036 Create lib.rs exporting arena API in crates/plix-arena/src/lib.rs
- [ ] T037 Create test arena file in assets/arenas/test_arena.toml

**Checkpoint**: Foundation ready - `cargo test` passes on plix-common, plix-net, plix-arena

---

## Phase 3: User Story 1 - Quick Server Join (Priority: P1)

**Goal**: Player can connect to server via IP:PORT and join a match

**Independent Test**: Launch server, launch client, enter IP:PORT, verify connection and spawn

**Note**: MVP uses direct IP connection. Server browser (spec acceptance scenarios 1-5) deferred to post-MVP.

### Server Implementation for US1

- [ ] T038 [US1] Create server CLI entry point in crates/plix-server/src/main.rs
- [ ] T039 [US1] Implement fixed tick loop in crates/plix-server/src/tick.rs
- [ ] T040 [US1] Implement player session management (join/leave/spawn) in crates/plix-server/src/session.rs
- [ ] T041 [US1] Implement server network loop (accept connections) in crates/plix-server/src/netloop.rs
- [ ] T042 [US1] Create lib.rs exporting server modules in crates/plix-server/src/lib.rs

### Client Implementation for US1

- [ ] T043 [US1] Create client entry point with window in crates/plix-client/src/main.rs
- [ ] T044 [US1] Implement connect screen (IP:PORT input) in crates/plix-client/src/ui/connect.rs
- [ ] T045 [US1] Implement client network loop (connect, send, receive) in crates/plix-client/src/net.rs
- [ ] T046 [US1] Create lib.rs exporting client modules in crates/plix-client/src/lib.rs

### Integration for US1

- [ ] T047 [US1] Integration test: client connects to server in crates/plix-server/tests/connection_test.rs

**Checkpoint**: At this point, a client can connect to server via IP - User Story 1 MVP complete

---

## Phase 4: User Story 2 - Fair PvP Combat (Priority: P1)

**Goal**: Players experience fair, responsive PvP with server-authoritative validation

**Independent Test**: Two clients connect, move, attack, verify smooth interpolation and server-validated hits

### Input & Prediction (Client)

- [ ] T048 [US2] Implement FPS input capture in crates/plix-client/src/input.rs
- [ ] T049 [US2] Implement command buffer (sequence, timestamp) in crates/plix-client/src/commands.rs
- [ ] T050 [US2] Implement client-side prediction for local player in crates/plix-client/src/prediction.rs
- [ ] T051 [US2] Implement server reconciliation (correction handling) in crates/plix-client/src/reconciliation.rs
- [ ] T052 [US2] Implement remote player interpolation in crates/plix-client/src/interpolation.rs

### Simulation (Server)

- [ ] T053 [US2] Implement player movement simulation in crates/plix-server/src/sim/movement.rs
- [ ] T054 [US2] Implement collision detection (player vs arena) in crates/plix-server/src/sim/collision.rs
- [ ] T055 [US2] Implement melee combat (attack, damage, respawn) in crates/plix-server/src/sim/combat.rs
- [ ] T056 [US2] Create simulation module in crates/plix-server/src/sim/mod.rs
- [ ] T057 [US2] Implement input validation (anti-speedhack) in crates/plix-server/src/validation.rs

### Replication (Server)

- [ ] T058 [US2] Define replicated state struct in crates/plix-server/src/replication/state.rs
- [ ] T059 [US2] Implement snapshot generation in crates/plix-server/src/replication/snapshot.rs
- [ ] T060 [US2] Implement game events (hit, death, respawn) in crates/plix-server/src/replication/events.rs
- [ ] T061 [US2] Create replication module in crates/plix-server/src/replication/mod.rs

### Match Management

- [ ] T062 [US2] Implement round state machine (waiting, countdown, playing, end) in crates/plix-server/src/match_state.rs
- [ ] T063 [US2] Implement scoring and round reset in crates/plix-server/src/match_state.rs

### Rendering (Client)

- [ ] T064 [US2] Implement voxel arena rendering in crates/plix-client/src/render/voxels.rs
- [ ] T065 [US2] Implement FPS camera in crates/plix-client/src/render/camera.rs
- [ ] T066 [US2] Implement player capsule rendering in crates/plix-client/src/render/players.rs
- [ ] T067 [US2] Create render module in crates/plix-client/src/render/mod.rs

### HUD (Client)

- [ ] T068 [US2] Implement minimal HUD (FPS, ping, HP, score) in crates/plix-client/src/ui/hud.rs
- [ ] T069 [US2] Implement network debug overlay in crates/plix-client/src/ui/net_debug.rs

### Integration Tests for US2

- [ ] T070 [US2] Test: two clients can see each other moving in crates/plix-server/tests/movement_test.rs
- [ ] T071 [US2] Test: combat hits are server-validated in crates/plix-server/tests/combat_test.rs

**Checkpoint**: At this point, two players can connect, move, fight - User Story 2 complete

---

## Phase 5: Tools & Load Testing

**Purpose**: Validate stability with automated testing

### Bot Client (plix-tools)

- [ ] T072 Implement network simulator (latency/loss injection) in crates/plix-tools/src/net_sim.rs
- [ ] T073 Implement headless bot client in crates/plix-tools/src/bot.rs
- [ ] T074 Create tools CLI entry point in crates/plix-tools/src/main.rs
- [ ] T075 Create lib.rs exporting tools modules in crates/plix-tools/src/lib.rs

### Load Tests

- [ ] T076 Create load test: 8-16 bots for 60 seconds in crates/plix-tools/tests/load_test.rs
- [ ] T077 Create stability assertions (no crash, stable tick) in crates/plix-tools/tests/stability_test.rs
- [ ] T078 Create scripts/run_load_test.sh

**Checkpoint**: `cargo test -p plix-tools` passes, load test completes without crashes

---

## Phase 6: Polish & Documentation

**Purpose**: Final documentation and cleanup

- [ ] T079 [P] Write README.md with quickstart instructions
- [ ] T080 [P] Write docs/architecture.md with module diagram
- [ ] T081 [P] Write docs/protocol.md with message specifications
- [ ] T082 [P] Write docs/testing.md with test procedures
- [ ] T083 Verify CI passes on all platforms

**Checkpoint**: All documentation complete, CI green

---

## MVP Checkpoints

- [ ] **CP1**: Server headless launches, loads arena, accepts 8-16 connections
- [ ] **CP2**: Client connects via IP, movement is fluid (prediction/reconciliation)
- [ ] **CP3**: Remote players render smoothly (interpolation), scores work
- [ ] **CP4**: Combat PvP functions with server-authoritative hit detection
- [ ] **CP5**: Load test 8-16 bots for 10 minutes, no crash, stable tick rate

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational phase 2
- **User Story 2 (Phase 4)**: Depends on User Story 1 (connection must work first)
- **Tools (Phase 5)**: Depends on User Story 2 (needs working game to test)
- **Polish (Phase 6)**: Depends on all previous phases

### User Story Dependencies

- **US1 (Server Join)**: Foundational only - can start after Phase 2
- **US2 (Fair PvP)**: Requires US1 (players must be able to connect first)

### Within Phases

- Tasks marked [P] can run in parallel
- Tests must be written with implementation (not TDD for MVP)
- Models/types before simulation logic
- Server logic before client consumption

### Parallel Opportunities

```text
# Phase 1 - All crate creations can run in parallel:
T002, T003, T004, T005, T006, T007, T008, T009

# Phase 2.1 - Type definitions can run in parallel:
T012, T013

# Phase 2.3 - Channel implementations can run in parallel:
T025, T026

# Phase 4 - Client and server work can partially parallelize:
Server: T053-T063
Client: T048-T052, T064-T069 (after T053 for contract knowledge)

# Phase 6 - All docs can run in parallel:
T079, T080, T081, T082
```

---

## Implementation Strategy

### MVP First (Minimum to Validate Network)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (basic connection)
4. **STOP and VALIDATE**: Can a client connect?
5. Complete Phase 4: User Story 2 (combat and rendering)
6. **STOP and VALIDATE**: Can two players fight fairly?
7. Complete Phase 5: Tools and load testing
8. **VALIDATE**: 10-minute bot test passes
9. Complete Phase 6: Documentation

### Incremental Delivery

1. Setup + Foundational → Crates compile, tests pass
2. Add US1 → Client can connect to server
3. Add US2 → Full gameplay loop works
4. Add Tools → Automated validation
5. Docs → Ready for external testing

---

## Deferred User Stories (Post-MVP)

| Story | Reason Deferred |
|-------|-----------------|
| US3: Custom Game Modes | Requires mod system |
| US4: Performant Mods | Requires mod system |
| US5: Offline Solo | Requires local server (same code, lower priority) |
| US6: Server Admin | Requires permission system |
| US7: Server Discovery | Requires server browser (direct IP for MVP) |
| US8: Customizable UI | Polish feature |

---

## Task Summary

| Phase | Task Count | Parallel Tasks |
|-------|------------|----------------|
| Phase 1: Setup | 10 | 8 |
| Phase 2: Foundational | 27 | 6 |
| Phase 3: US1 (Join) | 10 | 0 |
| Phase 4: US2 (Combat) | 24 | 4 |
| Phase 5: Tools | 7 | 0 |
| Phase 6: Polish | 5 | 4 |
| **Total** | **83** | **22** |

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] labels map to spec.md user stories for traceability
- Commit after each task or logical group
- Stop at any checkpoint to validate progress
- MVP validates US1 + US2 only; US3-US8 are post-MVP
