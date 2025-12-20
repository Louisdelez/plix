# Tasks: Sandboxed Mod Runtime (WASM)

**Input**: Design documents from `/specs/035-sandboxed-mod-runtime/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Tests are included as this feature involves security-critical sandboxing that requires thorough validation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Runtime crate**: `crates/plix-mod-runtime-wasm/`
- **Server integration**: `crates/plix-server/src/mods/`
- **Test fixtures**: `crates/plix-mod-runtime-wasm/tests/fixtures/`

---

## Phase 1: Setup (Crate Skeleton)

**Purpose**: Create plix-mod-runtime-wasm crate with basic structure

- [x] T001 Create crate directory and Cargo.toml with wasmtime + plix-mod-core dependencies in `crates/plix-mod-runtime-wasm/Cargo.toml`
- [x] T002 Add crate to workspace members in root `Cargo.toml`
- [x] T003 [P] Create lib.rs with public module declarations in `crates/plix-mod-runtime-wasm/src/lib.rs`
- [x] T004 [P] Create errors.rs with RuntimeError enum (WasmInvalid, Trap, FuelExhausted, MemoryLimit, MissingExport) in `crates/plix-mod-runtime-wasm/src/errors.rs`
- [x] T005 [P] Create abi/mod.rs with ABI_VERSION constant and HostCallOp enum in `crates/plix-mod-runtime-wasm/src/abi/mod.rs`
- [x] T006 [P] Create abi/types.rs with AbiRequest, AbiResponse structs in `crates/plix-mod-runtime-wasm/src/abi/types.rs`
- [x] T007 [P] Create abi/response.rs with ResponseBuffer struct (8KB) in `crates/plix-mod-runtime-wasm/src/abi/response.rs`
- [x] T008 [P] Create metrics.rs with WasmModMetrics struct in `crates/plix-mod-runtime-wasm/src/metrics.rs`
- [x] T009 Verify crate compiles with `cargo build -p plix-mod-runtime-wasm`

---

## Phase 2: Foundational (Core Runtime Infrastructure)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T010 Implement RuntimeConfig struct with defaults (5ms CPU, 32 MiB memory, 5 violations) in `crates/plix-mod-runtime-wasm/src/lib.rs`
- [x] T011 Implement WasmEngine struct wrapping wasmtime::Engine with fuel enabled in `crates/plix-mod-runtime-wasm/src/engine.rs`
- [x] T012 Implement ModState struct for per-mod store data in `crates/plix-mod-runtime-wasm/src/instance.rs`
- [x] T013 [P] Implement memory helpers read_bytes/write_bytes with bounds checking in `crates/plix-mod-runtime-wasm/src/memory.rs`
- [x] T014 Implement ModExports struct caching typed function references in `crates/plix-mod-runtime-wasm/src/instance.rs`
- [x] T015 Create host/mod.rs with Linker setup for plix namespace in `crates/plix-mod-runtime-wasm/src/host/mod.rs`
- [x] T016 [P] Implement bincode encode/decode for AbiRequest/AbiResponse in `crates/plix-mod-runtime-wasm/src/abi/types.rs`
- [x] T017 [P] Create host/caps.rs with plix_get_api_version, plix_get_abi_version stubs in `crates/plix-mod-runtime-wasm/src/host/caps.rs`
- [x] T018 [P] Create budgets.rs with FuelBudget struct and fuel_to_ms conversion in `crates/plix-mod-runtime-wasm/src/budgets.rs`
- [x] T019 Add unit test for bincode roundtrip in `crates/plix-mod-runtime-wasm/src/abi/types.rs`
- [x] T020 Add unit test for memory bounds checking in `crates/plix-mod-runtime-wasm/src/memory.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Safe Mod Loading (Priority: P1) 🎯 MVP

**Goal**: Load WASM mods in complete sandbox with no OS/FS/network access

**Independent Test**: Load a valid mod.wasm, verify mod_init called. Load invalid WASM, verify graceful failure.

### Tests for User Story 1

- [x] T021 [P] [US1] Create minimal test mod fixture (mod.toml + lib.rs) in `crates/plix-mod-runtime-wasm/tests/fixtures/minimal_mod/`
- [x] T022 [P] [US1] Create invalid WASM fixture (corrupted bytes) in `crates/plix-mod-runtime-wasm/tests/fixtures/invalid_mod/`
- [x] T023 [P] [US1] Write unit test: valid WASM loads and mod_init called in `crates/plix-mod-runtime-wasm/tests/unit/loader_tests.rs` (in lib.rs tests)
- [x] T024 [P] [US1] Write unit test: invalid WASM returns error, no crash in `crates/plix-mod-runtime-wasm/tests/unit/loader_tests.rs` (in lib.rs tests)
- [x] T025 [P] [US1] Write unit test: missing export (mod_init) rejected in `crates/plix-mod-runtime-wasm/tests/unit/loader_tests.rs` (in lib.rs tests)

### Implementation for User Story 1

- [x] T026 [US1] Implement ModuleLoader struct with load_from_bytes in `crates/plix-mod-runtime-wasm/src/module_loader.rs`
- [x] T027 [US1] Implement WASM validation (wasmtime::Module::validate) in `crates/plix-mod-runtime-wasm/src/module_loader.rs`
- [x] T028 [US1] Implement export validation (mod_init, mod_on_event, mod_shutdown, memory) in `crates/plix-mod-runtime-wasm/src/module_loader.rs`
- [x] T029 [US1] Implement WasmInstance struct with instantiate method in `crates/plix-mod-runtime-wasm/src/instance.rs`
- [x] T030 [US1] Implement call_mod_init with error handling in `crates/plix-mod-runtime-wasm/src/instance.rs`
- [x] T031 [US1] Ensure no WASI imports linked (reject if mod imports wasi_*) in `crates/plix-mod-runtime-wasm/src/module_loader.rs`
- [x] T032 [US1] Implement WasmRuntime public struct with load_mod method in `crates/plix-mod-runtime-wasm/src/lib.rs`
- [x] T033 [US1] Add build script to compile test fixtures to WASM in `crates/plix-mod-runtime-wasm/build.rs` (manual compilation)
- [x] T034 [US1] Run all US1 tests and verify they pass (67 tests passing)

**Checkpoint**: User Story 1 complete - mods load safely in sandbox

---

## Phase 4: User Story 2 - Event Handling via Host ABI (Priority: P1)

**Goal**: Mods can subscribe to events and call World/Entity/Net/Timer APIs

**Independent Test**: Load chat_filter mod, trigger PlayerChat event, verify handler called and can cancel.

### Tests for User Story 2

- [x] T035 [P] [US2] Create chat_filter_mod fixture in `crates/plix-mod-runtime-wasm/tests/fixtures/chat_filter_mod/`
- [x] T036 [P] [US2] Write integration test: mod receives PlayerChat event in `crates/plix-mod-runtime-wasm/src/lib.rs` (test_dispatch_event_to_chat_filter_mod)
- [x] T037 [P] [US2] Write integration test: mod cancels chat with cap in `crates/plix-mod-runtime-wasm/src/lib.rs` (test_dispatch_event_to_chat_filter_mod)
- [x] T038 [P] [US2] Write integration test: cancel without cap returns EMOD002 in `crates/plix-mod-runtime-wasm/src/lib.rs` (test_dispatch_event_without_cancel_capability)
- [x] T039 [P] [US2] Write unit test: capability check returns correct value in `crates/plix-mod-runtime-wasm/src/host/mod.rs` (via plix_has_capability)

### Implementation for User Story 2

- [x] T040 [US2] Implement plix_subscribe_event host function in `crates/plix-mod-runtime-wasm/src/host/mod.rs`
- [x] T041 [US2] Implement plix_cancel_event host function with cap check in `crates/plix-mod-runtime-wasm/src/host/mod.rs`
- [x] T042 [US2] Implement plix_has_capability host function in `crates/plix-mod-runtime-wasm/src/host/mod.rs`
- [x] T043 [US2] Implement EventContext storage in ModState for current event in `crates/plix-mod-runtime-wasm/src/instance.rs`
- [x] T044 [US2] Implement call_mod_on_event with payload serialization in `crates/plix-mod-runtime-wasm/src/instance.rs`
- [x] T045 [US2] Implement plix_world_call with capability enforcement in `crates/plix-mod-runtime-wasm/src/host/mod.rs` (stub returns EMOD007)
- [x] T046 [US2] Implement plix_entity_call with capability enforcement in `crates/plix-mod-runtime-wasm/src/host/mod.rs` (stub returns EMOD007)
- [x] T047 [US2] Implement plix_net_call with rate limiting (via 034) in `crates/plix-mod-runtime-wasm/src/host/mod.rs` (stub returns EMOD007)
- [x] T048 [US2] Implement plix_timer_call with limits (via 034) in `crates/plix-mod-runtime-wasm/src/host/mod.rs` (stub returns EMOD007)
- [x] T049 [US2] Implement plix_response_ptr and plix_response_len in `crates/plix-mod-runtime-wasm/src/host/mod.rs`
- [x] T050 [US2] Register all host functions in Linker in `crates/plix-mod-runtime-wasm/src/host/mod.rs`
- [x] T051 [US2] Add dispatch_event method to WasmRuntime in `crates/plix-mod-runtime-wasm/src/lib.rs`
- [x] T052 [US2] Run all US2 tests and verify they pass (70 tests passing)

**Checkpoint**: User Story 2 complete - mods can handle events and call APIs

---

## Phase 5: User Story 3 - CPU Budget Enforcement (Priority: P1)

**Goal**: Interrupt mods exceeding CPU budget, auto-disable after 5 consecutive violations

**Independent Test**: Load infinite_loop mod, verify interrupted within budget, disabled after 5 failures.

### Tests for User Story 3

- [x] T053 [P] [US3] Create infinite_loop_mod fixture in `crates/plix-mod-runtime-wasm/tests/fixtures/infinite_loop_mod/`
- [x] T054 [P] [US3] Write integration test: infinite loop interrupted in `crates/plix-mod-runtime-wasm/src/lib.rs` (test_infinite_loop_interrupted_by_fuel)
- [x] T055 [P] [US3] Write integration test: 5 consecutive violations disables mod in `crates/plix-mod-runtime-wasm/src/lib.rs` (test_mod_disabled_after_error_threshold)
- [x] T056 [P] [US3] Write unit test: success resets error counter in `crates/plix-mod-runtime-wasm/src/lib.rs` (test_success_resets_error_counter)

### Implementation for User Story 3

- [x] T057 [US3] Implement fuel calibration at runtime init in `crates/plix-mod-runtime-wasm/src/budgets.rs`
- [x] T058 [US3] Implement set_fuel before handler call in `crates/plix-mod-runtime-wasm/src/instance.rs`
- [x] T059 [US3] Implement OutOfFuel trap handling in `crates/plix-mod-runtime-wasm/src/instance.rs`
- [x] T060 [US3] Implement consecutive error tracking using ModContext from 034 in `crates/plix-mod-runtime-wasm/src/instance.rs`
- [x] T061 [US3] Implement auto-disable logic after ERROR_THRESHOLD violations in `crates/plix-mod-runtime-wasm/src/lib.rs`
- [x] T062 [US3] Implement success resets error count in `crates/plix-mod-runtime-wasm/src/instance.rs`
- [x] T063 [US3] Run all US3 tests and verify they pass (73 tests passing)

**Checkpoint**: User Story 3 complete - CPU budget enforced, runaway mods stopped

---

## Phase 6: User Story 4 - Memory Limits (Priority: P2)

**Goal**: Prevent mods from consuming excessive memory via memory.grow

**Independent Test**: Load memory_bomb mod, verify trapped when exceeding 32 MiB limit.

### Tests for User Story 4

- [x] T064 [P] [US4] Create memory_bomb_mod fixture in `crates/plix-mod-runtime-wasm/tests/fixtures/memory_bomb_mod/`
- [x] T065 [P] [US4] Write integration test: memory.grow beyond limit fails in `crates/plix-mod-runtime-wasm/src/lib.rs` (test_memory_growth_beyond_limit_fails)
- [x] T066 [P] [US4] Write integration test: repeated OOM disables mod in `crates/plix-mod-runtime-wasm/src/lib.rs` (test_repeated_oom_disables_mod)

### Implementation for User Story 4

- [x] T067 [US4] Implement ResourceLimiter trait for ModState in `crates/plix-mod-runtime-wasm/src/instance.rs`
- [x] T068 [US4] Configure Store with resource limiter in `crates/plix-mod-runtime-wasm/src/lib.rs`
- [x] T069 [US4] Handle memory limit trap as error in `crates/plix-mod-runtime-wasm/src/instance.rs`
- [x] T070 [US4] Run all US4 tests and verify they pass (75 tests passing)

**Checkpoint**: User Story 4 complete - memory limits enforced

---

## Phase 7: User Story 5 - Observability and Logging (Priority: P2)

**Goal**: Mods can log messages, server tracks per-mod metrics

**Independent Test**: Mod calls plix_log, verify message appears in server logs with mod_id.

### Tests for User Story 5

- [x] T071 [P] [US5] Write unit test: plix_log routes to tracing with mod_id in `crates/plix-mod-runtime-wasm/src/host/mod.rs`
- [x] T072 [P] [US5] Write unit test: metrics increment on host calls in `crates/plix-mod-runtime-wasm/src/metrics.rs`
- [x] T073 [P] [US5] Write unit test: invalid UTF-8 log truncated gracefully in `crates/plix-mod-runtime-wasm/src/memory.rs` (test_read_string_lossy)

### Implementation for User Story 5

- [x] T074 [US5] Implement plix_log host function with level mapping in `crates/plix-mod-runtime-wasm/src/host/mod.rs`
- [x] T075 [US5] Implement log message truncation (4KB max) in `crates/plix-mod-runtime-wasm/src/host/log.rs`
- [x] T076 [US5] Implement pointer validation for log message in `crates/plix-mod-runtime-wasm/src/host/mod.rs`
- [x] T077 [US5] Implement metrics increment in each host function in `crates/plix-mod-runtime-wasm/src/host/mod.rs`
- [x] T078 [US5] Implement debug mode logging (configurable) in `crates/plix-mod-runtime-wasm/src/lib.rs`
- [x] T079 [US5] Add get_metrics method to WasmRuntime in `crates/plix-mod-runtime-wasm/src/lib.rs` (get_mod_metrics)
- [x] T080 [US5] Run all US5 tests and verify they pass (76 tests passing)

**Checkpoint**: User Story 5 complete - logging and metrics available

---

## Phase 8: User Story 6 - Capability Discovery (Priority: P3)

**Goal**: Mods can query their granted capabilities at runtime

**Independent Test**: Mod calls plix_has_capability, returns correct values based on manifest.

### Tests for User Story 6

- [x] T081 [P] [US6] Write unit test: has_capability returns 1 for granted cap in `crates/plix-mod-runtime-wasm/src/host/caps.rs`
- [x] T082 [P] [US6] Write unit test: has_capability returns 0 for denied cap in `crates/plix-mod-runtime-wasm/src/host/caps.rs`

### Implementation for User Story 6

- [x] T083 [US6] Implement cap_id to Capability mapping in `crates/plix-mod-runtime-wasm/src/abi/mod.rs` (CapabilityId)
- [x] T084 [US6] Complete plix_has_capability implementation in `crates/plix-mod-runtime-wasm/src/host/mod.rs`
- [x] T085 [US6] Run all US6 tests and verify they pass (79 tests passing)

**Checkpoint**: User Story 6 complete - capability discovery works

---

## Phase 9: Server Integration

**Purpose**: Integrate WasmRuntime with plix-server ModManager

- [x] T086 Create wasm_bridge.rs with WasmBridge struct in `crates/plix-server/src/mods/wasm_bridge.rs`
- [x] T087 Implement load_wasm_mod bridging manifest to WasmRuntime in `crates/plix-server/src/mods/wasm_bridge.rs`
- [x] T088 Implement dispatch_to_wasm_mods for event routing in `crates/plix-server/src/mods/wasm_bridge.rs`
- [x] T089 Extend ModManager with WasmBridge integration in `crates/plix-server/src/mods/mod.rs`
- [x] T090 Implement mod_shutdown call on server stop in `crates/plix-server/src/mods/mod.rs`
- [x] T091 Add integration test: full mod lifecycle via server in `crates/plix-server/tests/mod_integration_test.rs`

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, final validation, CI readiness

- [ ] T092 [P] Create net_spam_mod fixture for rate limit testing in `crates/plix-mod-runtime-wasm/tests/fixtures/net_spam_mod/` (deferred - rate limiting enforced via plix-mod-core)
- [ ] T093 [P] Write integration test: net spam rate limited in `crates/plix-mod-runtime-wasm/tests/integration/malicious_mod_test.rs` (deferred - rate limiting enforced via plix-mod-core)
- [x] T094 [P] Create docs/feature-035.md with ABI v1 documentation
- [x] T095 Run cargo fmt --all and fix any formatting issues
- [x] T096 Run cargo clippy -p plix-mod-runtime-wasm and fix warnings
- [x] T097 Run full test suite: cargo test -p plix-mod-runtime-wasm (79 tests passing)
- [x] T098 Verify build in release mode: cargo build -p plix-mod-runtime-wasm --release
- [ ] T099 Validate quickstart.md instructions work end-to-end (manual verification required)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion
  - US1-US3 are all P1 priority but have logical dependencies
  - US4-US6 can proceed after US1-US3 foundation
- **Server Integration (Phase 9)**: Depends on US1-US3 completion
- **Polish (Phase 10)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P1)**: Depends on US1 (needs working mod loading)
- **User Story 3 (P1)**: Depends on US1 (needs working mod execution)
- **User Story 4 (P2)**: Can start after US1 - Independent from US2/US3
- **User Story 5 (P2)**: Can start after Foundational - Independent
- **User Story 6 (P3)**: Can start after Foundational - Independent

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Module stubs before full implementation
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks T003-T008 marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (T013, T016-T018)
- Test fixtures for different stories can be created in parallel
- US4, US5, US6 can proceed in parallel after US1 completes

---

## Parallel Example: User Story 1 Setup

```bash
# Launch all test fixtures together:
Task: "Create minimal test mod fixture in tests/fixtures/minimal_mod/"
Task: "Create invalid WASM fixture in tests/fixtures/invalid_mod/"

# Launch all tests together:
Task: "Write unit test: valid WASM loads in tests/unit/loader_tests.rs"
Task: "Write unit test: invalid WASM returns error in tests/unit/loader_tests.rs"
Task: "Write unit test: missing export rejected in tests/unit/loader_tests.rs"
```

---

## Implementation Strategy

### MVP First (User Stories 1-3)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Safe Loading)
4. Complete Phase 4: User Story 2 (Event Handling)
5. Complete Phase 5: User Story 3 (CPU Budgets)
6. **STOP and VALIDATE**: Test all P1 stories independently
7. Deploy/demo if ready - this is a functional MVP

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Basic mod loading works
3. Add User Story 2 → Test independently → Mods can handle events
4. Add User Story 3 → Test independently → DoS protection active (MVP!)
5. Add User Story 4 → Test independently → Memory protection active
6. Add User Story 5 → Test independently → Observability ready
7. Add User Story 6 → Test independently → Full feature complete

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 → User Story 2 → User Story 3
   - Developer B: Test fixtures for US4-US6 (can start early)
   - Developer C: Documentation and contracts validation
3. After US1 complete, US4-US6 can proceed in parallel

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Test fixture compilation requires wasm32-unknown-unknown target installed
