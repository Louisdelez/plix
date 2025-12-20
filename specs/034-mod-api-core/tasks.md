# Tasks: Mod API Core

**Input**: Design documents from `/specs/034-mod-api-core/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Unit and integration tests are included as requested in the feature specification (Definition of Done requires passing tests).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

```text
crates/plix-mod-core/src/       # New mod API crate
crates/plix-mod-core/src/api/   # API modules (world, entities, net, timers)
crates/plix-server/src/mods/    # Server integration
crates/plix-common/src/         # Shared types
```

---

## Phase 1: Setup (Crate Skeleton)

**Purpose**: Create plix-mod-core crate structure and dependencies

- [ ] T001 Create crate directory `crates/plix-mod-core/` with Cargo.toml
- [ ] T002 [P] Create module skeleton `crates/plix-mod-core/src/lib.rs` with public exports
- [ ] T003 [P] Create empty module files:
  - `crates/plix-mod-core/src/errors.rs`
  - `crates/plix-mod-core/src/capabilities.rs`
  - `crates/plix-mod-core/src/manifest.rs`
  - `crates/plix-mod-core/src/registry.rs`
  - `crates/plix-mod-core/src/events.rs`
  - `crates/plix-mod-core/src/observability.rs`
- [ ] T004 [P] Create API submodule skeleton:
  - `crates/plix-mod-core/src/api/mod.rs`
  - `crates/plix-mod-core/src/api/world.rs`
  - `crates/plix-mod-core/src/api/entities.rs`
  - `crates/plix-mod-core/src/api/net.rs`
  - `crates/plix-mod-core/src/api/timers.rs`
- [ ] T005 Add plix-mod-core to workspace Cargo.toml

**Checkpoint**: Crate compiles with empty modules

---

## Phase 2: Foundational (Error Model & Capabilities)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T006 Implement ModApiError struct with code, message, context in `crates/plix-mod-core/src/errors.rs`
- [ ] T007 [P] Define ErrorCode enum (EMOD001-EMOD007) in `crates/plix-mod-core/src/errors.rs`
- [ ] T008 [P] Implement error helper functions (err_invalid, err_perm, err_not_found, err_bounds, err_rate, err_world_not_ready, err_unsupported) in `crates/plix-mod-core/src/errors.rs`
- [ ] T009 [P] Add unit tests for error creation and formatting in `crates/plix-mod-core/src/errors.rs`
- [ ] T010 Define Capability bitflags enum (world.read, world.write, entity.read, entity.write, net.send, event.cancel.chat, event.cancel.blocks) in `crates/plix-mod-core/src/capabilities.rs`
- [ ] T011 [P] Implement `require(cap)` helper returning EMOD002 in `crates/plix-mod-core/src/capabilities.rs`
- [ ] T012 [P] Add server policy override struct (allow/deny per mod/capability) in `crates/plix-mod-core/src/capabilities.rs`
- [ ] T013 [P] Add unit tests for capability checks and overrides in `crates/plix-mod-core/src/capabilities.rs`
- [ ] T014 Define ModManifest struct with serde derives in `crates/plix-mod-core/src/manifest.rs`
- [ ] T015 [P] Implement TOML parser for mod.toml format in `crates/plix-mod-core/src/manifest.rs`
- [ ] T016 [P] Implement manifest validation (id format, api_version check, capability validation) in `crates/plix-mod-core/src/manifest.rs`
- [ ] T017 [P] Add unit tests for manifest parsing (valid/invalid cases) in `crates/plix-mod-core/src/manifest.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Subscribe to Gameplay Events (Priority: P1) 🎯 MVP

**Goal**: Mods can subscribe to stable gameplay events (join, chat, block placed, etc.) without polling

**Independent Test**: Load a test mod that subscribes to `on_player_join` event and verify it receives the event with player_id and name

### Tests for User Story 1

- [ ] T018 [P] [US1] Unit test for event subscription/unsubscription in `crates/plix-mod-core/src/events.rs`
- [ ] T019 [P] [US1] Unit test for FIFO dispatch order in `crates/plix-mod-core/src/events.rs`
- [ ] T020 [P] [US1] Unit test for error isolation (handler error doesn't crash) in `crates/plix-mod-core/src/events.rs`
- [ ] T021 [P] [US1] Unit test for auto-disable after 5 consecutive errors in `crates/plix-mod-core/src/events.rs`

### Implementation for User Story 1

- [ ] T022 [US1] Define EventType enum with MVP events (server_start, server_stop, player_join, player_leave, player_chat, block_placed, block_broken, entity_damaged, mod_message) in `crates/plix-mod-core/src/events.rs`
- [ ] T023 [P] [US1] Define event payload structs (ServerStartPayload, PlayerJoinPayload, PlayerChatPayload, BlockPlacedPayload, etc.) in `crates/plix-mod-core/src/events.rs`
- [ ] T024 [P] [US1] Define GameEvent struct with event_type, payload, timestamp, cancellable flag in `crates/plix-mod-core/src/events.rs`
- [ ] T025 [US1] Implement ModRegistry struct with ModContext (caps, state, error_count, subscriptions) in `crates/plix-mod-core/src/registry.rs`
- [ ] T026 [P] [US1] Implement disable_mod(mod_id, reason) and is_enabled(mod_id) in `crates/plix-mod-core/src/registry.rs`
- [ ] T027 [P] [US1] Add unit tests for registry load/disable/enable in `crates/plix-mod-core/src/registry.rs`
- [ ] T028 [US1] Implement EventBus struct with subscription management (subscribe, unsubscribe) in `crates/plix-mod-core/src/events.rs`
- [ ] T029 [US1] Implement phase-based event queue (collect during tick, dispatch at end-of-tick) in `crates/plix-mod-core/src/events.rs`
- [ ] T030 [US1] Implement FIFO dispatch with re-entrancy prevention in `crates/plix-mod-core/src/events.rs`
- [ ] T031 [US1] Implement handler error isolation with consecutive error counter in `crates/plix-mod-core/src/events.rs`
- [ ] T032 [US1] Implement auto-disable after 5 consecutive errors with logging in `crates/plix-mod-core/src/events.rs`
- [ ] T033 [US1] Add cancellable flag to PlayerChat, BlockPlaced, BlockBroken events in `crates/plix-mod-core/src/events.rs`
- [ ] T034 [US1] Implement cancel_event() API with capability check (event.cancel.chat, event.cancel.blocks) in `crates/plix-mod-core/src/events.rs`
- [ ] T035 [P] [US1] Add unit tests for cancellation (allowed with cap, denied without cap) in `crates/plix-mod-core/src/events.rs`

**Checkpoint**: Event subscription and dispatch fully functional, mods can subscribe/receive/cancel events

---

## Phase 4: User Story 2 - Read and Write World Data (Priority: P1)

**Goal**: Mods can read and modify world data (blocks, entities) through safe, bounded API

**Independent Test**: Load a mod that calls `get_block(pos)` and `set_block(pos, block_id)` with proper permissions

### Tests for User Story 2

- [ ] T036 [P] [US2] Unit test for get_block with permission check in `crates/plix-mod-core/src/api/world.rs`
- [ ] T037 [P] [US2] Unit test for raycast bounds enforcement (256 max) in `crates/plix-mod-core/src/api/world.rs`
- [ ] T038 [P] [US2] Unit test for query_aabb limit enforcement (128 max) in `crates/plix-mod-core/src/api/world.rs`
- [ ] T039 [P] [US2] Unit test for set_block permission denied in `crates/plix-mod-core/src/api/world.rs`
- [ ] T040 [P] [US2] Unit test for chunk not loaded error (EMOD006) in `crates/plix-mod-core/src/api/world.rs`

### Implementation for User Story 2

- [ ] T041 [US2] Define WorldApi trait with get_block, set_block, raycast, query_aabb signatures in `crates/plix-mod-core/src/api/world.rs`
- [ ] T042 [US2] Implement get_block(pos) with world.read capability check in `crates/plix-mod-core/src/api/world.rs`
- [ ] T043 [US2] Implement raycast(origin, dir, max_dist) with 256 block clamp in `crates/plix-mod-core/src/api/world.rs`
- [ ] T044 [US2] Implement query_aabb(bounds, limit) with 128 result clamp in `crates/plix-mod-core/src/api/world.rs`
- [ ] T045 [US2] Implement set_block(pos, block_id) with world.write capability check in `crates/plix-mod-core/src/api/world.rs`
- [ ] T046 [US2] Add chunk loaded validation returning EMOD006 in `crates/plix-mod-core/src/api/world.rs`
- [ ] T047 [US2] Add position bounds validation returning EMOD004 in `crates/plix-mod-core/src/api/world.rs`
- [ ] T048 [US2] Define EntityHandle struct with index and generation in `crates/plix-mod-core/src/api/entities.rs`
- [ ] T049 [P] [US2] Add unit tests for entity handle stale detection in `crates/plix-mod-core/src/api/entities.rs`
- [ ] T050 [US2] Define EntityApi trait with read functions (get_transform, get_velocity, get_health, get_owner, get_team) in `crates/plix-mod-core/src/api/entities.rs`
- [ ] T051 [US2] Implement entity read functions with entity.read capability check in `crates/plix-mod-core/src/api/entities.rs`
- [ ] T052 [US2] Implement apply_damage(id, amount, source) with entity.write capability check in `crates/plix-mod-core/src/api/entities.rs`
- [ ] T053 [US2] Implement apply_impulse(id, vec3) with entity.write capability check in `crates/plix-mod-core/src/api/entities.rs`
- [ ] T054 [US2] Implement spawn_entity(type, transform) with entity.write capability check in `crates/plix-mod-core/src/api/entities.rs`
- [ ] T055 [US2] Implement despawn_entity(id) with entity.write capability check in `crates/plix-mod-core/src/api/entities.rs`
- [ ] T056 [P] [US2] Add unit tests for entity API permission denied in `crates/plix-mod-core/src/api/entities.rs`

**Checkpoint**: World and Entity APIs fully functional with bounds and permissions

---

## Phase 5: User Story 3 - Control Mod Permissions (Priority: P1)

**Goal**: Server administrators can control what mods are allowed to do via capabilities

**Independent Test**: Configure server to deny world.write for a mod, verify set_block returns EMOD002

### Tests for User Story 3

- [ ] T057 [P] [US3] Unit test for server policy override in `crates/plix-mod-core/src/capabilities.rs`
- [ ] T058 [P] [US3] Unit test for manifest validation rejection in `crates/plix-mod-core/src/manifest.rs`

### Implementation for User Story 3

- [ ] T059 [US3] Implement CapabilityPolicy struct with per-mod overrides in `crates/plix-mod-core/src/capabilities.rs`
- [ ] T060 [US3] Implement effective_capabilities(mod_id, manifest_caps, policy) function in `crates/plix-mod-core/src/capabilities.rs`
- [ ] T061 [US3] Integrate capability policy into ModRegistry.load() in `crates/plix-mod-core/src/registry.rs`
- [ ] T062 [US3] Add policy configuration parsing (from server config) in `crates/plix-mod-core/src/capabilities.rs`
- [ ] T063 [P] [US3] Add integration test for policy override in `crates/plix-mod-core/src/capabilities.rs`

**Checkpoint**: Server can grant/deny capabilities per mod

---

## Phase 6: User Story 4 - Send/Receive Mod Network Messages (Priority: P2)

**Goal**: Mods can send and receive typed network messages between server and clients

**Independent Test**: Load a mod that sends message on channel "mod:testmod:ping", verify server receives it

### Tests for User Story 4

- [ ] T064 [P] [US4] Unit test for channel name validation (mod:id:name format) in `crates/plix-mod-core/src/api/net.rs`
- [ ] T065 [P] [US4] Unit test for payload size limit (8KB) in `crates/plix-mod-core/src/api/net.rs`
- [ ] T066 [P] [US4] Unit test for rate limiting (20 msg/s) in `crates/plix-mod-core/src/api/net.rs`
- [ ] T067 [P] [US4] Unit test for permission denied without net.send in `crates/plix-mod-core/src/api/net.rs`

### Implementation for User Story 4

- [ ] T068 [US4] Define ModChannel struct with mod_id, name, direction in `crates/plix-mod-core/src/api/net.rs`
- [ ] T069 [US4] Implement channel name parsing and validation in `crates/plix-mod-core/src/api/net.rs`
- [ ] T070 [US4] Define NetApi trait with send_message signature in `crates/plix-mod-core/src/api/net.rs`
- [ ] T071 [US4] Implement send_message with net.send capability check in `crates/plix-mod-core/src/api/net.rs`
- [ ] T072 [US4] Implement payload size validation (8KB max, EMOD001) in `crates/plix-mod-core/src/api/net.rs`
- [ ] T073 [US4] Implement token bucket rate limiter (20 msg/s) in `crates/plix-mod-core/src/api/net.rs`
- [ ] T074 [US4] Add rate limit violation returning EMOD005 in `crates/plix-mod-core/src/api/net.rs`
- [ ] T075 [US4] Wire on_mod_message event emission for received messages in `crates/plix-mod-core/src/api/net.rs`
- [ ] T076 [US4] Implement channel isolation (only owner mod receives) in `crates/plix-mod-core/src/api/net.rs`

**Checkpoint**: Mod networking fully functional with rate limiting

---

## Phase 7: User Story 5 - Validate API Version Compatibility (Priority: P2)

**Goal**: Engine validates mod's API version requirements at load time

**Independent Test**: Create mod with api_version=99, verify EMOD007 on load

### Tests for User Story 5

- [ ] T077 [P] [US5] Unit test for api_version check in manifest validation in `crates/plix-mod-core/src/manifest.rs`
- [ ] T078 [P] [US5] Unit test for min/max_api_version range check in `crates/plix-mod-core/src/manifest.rs`

### Implementation for User Story 5

- [ ] T079 [US5] Define ENGINE_API_VERSION constant (MVP = 1) in `crates/plix-mod-core/src/lib.rs`
- [ ] T080 [US5] Implement get_api_version() function in `crates/plix-mod-core/src/lib.rs`
- [ ] T081 [US5] Implement get_engine_version() function returning SemVer in `crates/plix-mod-core/src/lib.rs`
- [ ] T082 [US5] Add api_version compatibility check in manifest validation in `crates/plix-mod-core/src/manifest.rs`
- [ ] T083 [US5] Add min_api_version/max_api_version optional fields and validation in `crates/plix-mod-core/src/manifest.rs`

**Checkpoint**: API version compatibility enforced at load time

---

## Phase 8: User Story 6 - Use Bounded Timers (Priority: P3)

**Goal**: Mods can schedule timed callbacks with set_timeout/set_interval

**Independent Test**: Call set_timeout(500ms), verify callback executes after ~500ms

### Tests for User Story 6

- [ ] T084 [P] [US6] Unit test for min interval clamping (50ms) in `crates/plix-mod-core/src/api/timers.rs`
- [ ] T085 [P] [US6] Unit test for max timers enforcement (32) in `crates/plix-mod-core/src/api/timers.rs`
- [ ] T086 [P] [US6] Unit test for clear_timer with invalid handle in `crates/plix-mod-core/src/api/timers.rs`

### Implementation for User Story 6

- [ ] T087 [US6] Define TimerHandle struct in `crates/plix-mod-core/src/api/timers.rs`
- [ ] T088 [US6] Define TimerApi trait with set_timeout, set_interval, clear_timer in `crates/plix-mod-core/src/api/timers.rs`
- [ ] T089 [US6] Implement timer storage per mod in `crates/plix-mod-core/src/api/timers.rs`
- [ ] T090 [US6] Implement set_timeout with 50ms min clamp in `crates/plix-mod-core/src/api/timers.rs`
- [ ] T091 [US6] Implement set_interval with 50ms min clamp in `crates/plix-mod-core/src/api/timers.rs`
- [ ] T092 [US6] Implement max timer enforcement (32 per mod, EMOD005) in `crates/plix-mod-core/src/api/timers.rs`
- [ ] T093 [US6] Implement clear_timer with handle validation (EMOD003) in `crates/plix-mod-core/src/api/timers.rs`
- [ ] T094 [US6] Implement timer tick processing (fire expired timers) in `crates/plix-mod-core/src/api/timers.rs`

**Checkpoint**: Timer API fully functional with bounds

---

## Phase 9: Observability & Server Integration

**Purpose**: Logging, metrics, and server integration

- [ ] T095 Implement structured logging with tracing (mod_id, event, error_code, context) in `crates/plix-mod-core/src/observability.rs`
- [ ] T096 [P] Implement metrics counters (events_dispatched, handler_errors, net_sent, net_dropped, api_denied, timers_active) in `crates/plix-mod-core/src/observability.rs`
- [ ] T097 [P] Add unit tests for observability metrics in `crates/plix-mod-core/src/observability.rs`
- [ ] T098 Create mod integration module in `crates/plix-server/src/mods/mod.rs`
- [ ] T099 Implement mod loading from filesystem in `crates/plix-server/src/mods/mod.rs`
- [ ] T100 Wire event emission points in server game loop in `crates/plix-server/src/mods/mod.rs`
- [ ] T101 Add server dependency on plix-mod-core in `crates/plix-server/Cargo.toml`

**Checkpoint**: Mods load and integrate with server

---

## Phase 10: Integration Testing & Documentation

**Purpose**: End-to-end validation and documentation

- [ ] T102 Create dummy mod manifest for integration tests in `crates/plix-mod-core/tests/fixtures/dummy_mod/mod.toml`
- [ ] T103 Implement dummy mod integration test - event subscription in `crates/plix-mod-core/tests/integration_test.rs`
- [ ] T104 [P] Implement dummy mod integration test - event cancellation in `crates/plix-mod-core/tests/integration_test.rs`
- [ ] T105 [P] Implement dummy mod integration test - permission denied scenarios in `crates/plix-mod-core/tests/integration_test.rs`
- [ ] T106 [P] Implement dummy mod integration test - rate limiting in `crates/plix-mod-core/tests/integration_test.rs`
- [ ] T107 [P] Implement dummy mod integration test - timer limits in `crates/plix-mod-core/tests/integration_test.rs`
- [ ] T108 Implement dummy mod integration test - auto-disable after 5 errors in `crates/plix-mod-core/tests/integration_test.rs`
- [ ] T109 Create feature documentation in `docs/feature-034.md` (manifest format, capabilities, events, APIs, error codes)

---

## Phase 11: Polish & Validation (DoD)

**Purpose**: Final validation and cleanup

- [ ] T110 Run cargo fmt on plix-mod-core
- [ ] T111 Run cargo clippy on plix-mod-core and fix warnings
- [ ] T112 Verify all unit tests pass with cargo test -p plix-mod-core
- [ ] T113 Verify integration tests pass
- [ ] T114 Verify no panics in any API path (review all unwrap/expect)
- [ ] T115 Validate manifest: valid loads, invalid fails with clear error
- [ ] T116 Validate events: dispatch stable, cancellation conforme, auto-disable works
- [ ] T117 Validate World/Entities: bounds enforced, permissions applied
- [ ] T118 Validate networking: 8KB limit, 20 msg/s rate limit, channel isolation
- [ ] T119 Validate timers: 50ms min, 32 max, errors cohérentes
- [ ] T120 Run quickstart.md validation checklist

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion
  - US1 (Events) - Foundation for other stories
  - US2 (World/Entities) - Can start after US1 basics
  - US3 (Permissions) - Can run parallel to US2
  - US4 (Networking) - Can start after Foundational
  - US5 (Versioning) - Can run parallel to US4
  - US6 (Timers) - Can run parallel to US4/US5
- **Integration (Phase 9-10)**: Depends on US1-US6 completion
- **Polish (Phase 11)**: Depends on all phases complete

### User Story Dependencies

- **User Story 1 (Events)**: Depends on Foundational only - MVP core
- **User Story 2 (World/Entities)**: Depends on US1 for event integration
- **User Story 3 (Permissions)**: Depends on Foundational - can parallel with US2
- **User Story 4 (Networking)**: Depends on Foundational - independent of US2/US3
- **User Story 5 (Versioning)**: Depends on Foundational - independent
- **User Story 6 (Timers)**: Depends on Foundational - independent

### Parallel Opportunities

**Within Phase 2 (Foundational)**:
- T006, T007, T008 (errors) can parallel
- T010, T011, T012 (capabilities) can parallel
- T014, T015, T016 (manifest) can parallel

**Within User Story 1**:
- T018, T019, T020, T021 (tests) can parallel
- T022, T023, T024 (event types/payloads) can parallel
- T025, T026, T027 (registry) can parallel

**Across User Stories (after Foundational)**:
- US3, US4, US5, US6 can all run in parallel once US1 basics are done

---

## Parallel Example: Foundational Phase

```bash
# Launch error module tasks together:
Task: "Implement ModApiError struct in crates/plix-mod-core/src/errors.rs"
Task: "Define ErrorCode enum in crates/plix-mod-core/src/errors.rs"
Task: "Implement error helpers in crates/plix-mod-core/src/errors.rs"

# Launch capability module tasks together:
Task: "Define Capability bitflags in crates/plix-mod-core/src/capabilities.rs"
Task: "Implement require(cap) helper in crates/plix-mod-core/src/capabilities.rs"
Task: "Add server policy override in crates/plix-mod-core/src/capabilities.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (errors, capabilities, manifest)
3. Complete Phase 3: User Story 1 (events, registry, dispatch)
4. **STOP and VALIDATE**: Test event subscription independently
5. This provides a working mod system with event subscription

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 (Events) → Test independently → Working event bus
3. Add US2 (World/Entities) → Test independently → Full world access
4. Add US3 (Permissions) → Test independently → Admin control
5. Add US4 (Networking) → Test independently → Mod messaging
6. Add US5 (Versioning) → Test independently → Version safety
7. Add US6 (Timers) → Test independently → Scheduling
8. Integration + Polish → Production ready

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Events) + User Story 2 (World)
   - Developer B: User Story 3 (Permissions) + User Story 4 (Networking)
   - Developer C: User Story 5 (Versioning) + User Story 6 (Timers)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- All API calls MUST return Result<T, ModApiError> - no panics allowed
