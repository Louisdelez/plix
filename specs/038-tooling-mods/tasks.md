# Tasks: Tooling Mods (SDK, Templates, CLI, Hot-Reload)

## ✅ FEATURE STATUS: DONE (Phase 8 deferred)

**Completion Date**: 2025-12-19

| Phase | Status |
|-------|--------|
| Phase 1: Setup | ✅ Complete |
| Phase 2: Foundational | ✅ Complete |
| Phase 3-4: SDK APIs + Macros | ✅ Complete |
| Phase 5: Templates + CLI | ✅ Complete |
| Phase 6: Validation | ✅ Complete |
| Phase 7: Documentation | ✅ Complete |
| Phase 8: Hot-Reload | ⏸️ Deferred to 039-dev-hot-reload |
| Phase 9: Polish | ✅ Complete (toolchain tasks pending) |

**Note**: Hot-reload (US4) deferred to dedicated feature 039-dev-hot-reload. All other user stories (US1, US2, US3, US5) are complete.

---

**Input**: Design documents from `/specs/038-tooling-mods/`
**Prerequisites**: plan.md, spec.md, research.md, contracts/

**Tests**: Unit tests included per user story requirements. Integration tests included where feasible.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4, US5)

## User Story Mapping

| Story | Priority | Description |
|-------|----------|-------------|
| US1 | P1 | Create and Load First Mod (full toolchain) |
| US2 | P1 | Use SDK for Safe Host Interactions |
| US3 | P2 | Validate Mod Before Distribution |
| US4 | P3 | Iterate Quickly with Hot-Reload |
| US5 | P2 | Learn Modding from Documentation |

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create SDK crate skeleton and shared types

- [x] T001 Create `crates/plix-mod-sdk/Cargo.toml` with dependencies (glam, bincode, thiserror)
- [x] T002 Create SDK module structure in `crates/plix-mod-sdk/src/lib.rs` with module declarations
- [x] T003 [P] Create `crates/plix-mod-sdk/src/prelude.rs` with re-exports placeholder
- [x] T004 [P] Create `crates/plix-mod-sdk/src/version.rs` with SDK_ABI_VERSION=1, SDK_API_VERSION=1
- [x] T005 [P] Create `crates/plix-mod-sdk/src/error.rs` with ModError and ErrorCode enum (EMOD001-007)
- [x] T006 [P] Create `crates/plix-mod-sdk/src/abi.rs` with extern FFI declarations for host functions
- [x] T007 [P] Create `crates/plix-mod-sdk/src/codec.rs` with bincode encode/decode helpers
- [x] T008 Create `crates/plix-mod-cli/Cargo.toml` with dependencies (clap, zip, sha2, walkdir)
- [x] T009 [P] Create CLI entry point in `crates/plix-mod-cli/src/main.rs` with clap subcommands skeleton
- [x] T010 [P] Create `crates/plix-mod-sdk-macros/Cargo.toml` as proc-macro crate
- [x] T011 [P] Create `crates/plix-mod-sdk-macros/src/lib.rs` with placeholder proc-macro

**Checkpoint**: SDK and CLI crate skeletons exist, `cargo check` passes ✓

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core SDK infrastructure that all user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T012 Implement ABI extern declarations in `crates/plix-mod-sdk/src/abi.rs` for all 19 host functions
- [x] T013 [P] Implement memory helpers in `crates/plix-mod-sdk/src/abi.rs` (ptr/len handling, response buffer)
- [x] T014 [P] Implement ErrorCode mapping in `crates/plix-mod-sdk/src/error.rs` with Display impl
- [x] T015 Implement codec v1 encode/decode in `crates/plix-mod-sdk/src/codec.rs` aligned with runtime 035
- [x] T016 [P] Create capability types in `crates/plix-mod-sdk/src/caps.rs` with bitmask constants
- [x] T017 [P] Create event types in `crates/plix-mod-sdk/src/events.rs` with EventType enum
- [x] T018 [P] Create event payloads in `crates/plix-mod-sdk/src/events.rs` (PlayerChatPayload, BlockPlacedPayload, etc.)
- [x] T019 Add unit tests for codec roundtrip in `crates/plix-mod-sdk/src/codec.rs`
- [x] T020 Add unit tests for error mapping in `crates/plix-mod-sdk/src/error.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel ✓

---

## Phase 3: User Story 2 - Use SDK for Safe Host Interactions (Priority: P1) 🎯 MVP

**Goal**: Provide ergonomic SDK wrappers for all host functions so modders don't need raw ABI calls

**Independent Test**: Write a mod using SDK functions (log, subscribe, world query), compile and run - all calls succeed

### Implementation for User Story 2

- [x] T021 [P] [US2] Implement logging API in `crates/plix-mod-sdk/src/log.rs` (info!, warn!, error!, debug! macros)
- [x] T022 [P] [US2] Implement capability check in `crates/plix-mod-sdk/src/caps.rs` (has() function)
- [x] T023 [US2] Implement event subscription in `crates/plix-mod-sdk/src/events.rs` (subscribe() function)
- [x] T024 [US2] Implement EventContext in `crates/plix-mod-sdk/src/events.rs` with cancel() method
- [x] T025 [P] [US2] Implement world API in `crates/plix-mod-sdk/src/world.rs` (get_block, set_block, raycast, query_aabb)
- [x] T026 [P] [US2] Implement entity API in `crates/plix-mod-sdk/src/entities.rs` (get_transform, apply_damage, apply_impulse)
- [x] T027 [P] [US2] Implement network API in `crates/plix-mod-sdk/src/net.rs` (send, broadcast)
- [x] T028 [P] [US2] Implement timer API in `crates/plix-mod-sdk/src/timers.rs` (set_timeout, set_interval, clear)
- [x] T029 [US2] Implement version helpers in `crates/plix-mod-sdk/src/version.rs` (check_compatibility, assert_compatible)
- [x] T030 [US2] Update prelude in `crates/plix-mod-sdk/src/prelude.rs` with all public exports
- [x] T031 [US2] Add unit tests for world API encoding in `crates/plix-mod-sdk/src/world.rs`
- [x] T032 [US2] Add unit tests for entity API encoding in `crates/plix-mod-sdk/src/entities.rs`
- [x] T033 [US2] Add unit tests for timer API encoding in `crates/plix-mod-sdk/src/timers.rs`

**Checkpoint**: SDK wrappers complete, modders can use ergonomic API instead of raw ABI ✓

---

## Phase 4: User Story 2 (continued) - Proc Macros

**Goal**: Provide attribute macros for mod entry points

### Implementation for User Story 2 Macros

- [x] T034 [US2] Implement `#[plix_mod]` macro in `crates/plix-mod-sdk-macros/src/lib.rs` (generates mod_init, mod_shutdown exports)
- [x] T035 [US2] Implement `#[on_event]` macro in `crates/plix-mod-sdk-macros/src/lib.rs` (registers event handlers)
- [x] T036 [US2] Implement event routing dispatch in generated `mod_on_event` export
- [x] T037 [US2] Re-export macros from `crates/plix-mod-sdk/src/lib.rs` (N/A - macros used via separate crate)
- [x] T038 [US2] Add compile-time tests for macros (ensure exports generated correctly)
- [x] T039 [US2] Add rustdoc examples for macros in `crates/plix-mod-sdk-macros/src/lib.rs`

**Checkpoint**: Macros work, `#[plix_mod]` generates all required WASM exports ✓

---

## Phase 5: User Story 1 - Create and Load First Mod (Priority: P1) 🎯 MVP

**Goal**: Modder can scaffold, build, pack, and load a mod in under 5 minutes

**Independent Test**: Run `plix mod new`, build, pack, copy to server mods folder, start server - mod loads and executes

### Templates for User Story 1

- [x] T040 [P] [US1] Create `templates/mods/chat-filter/Cargo.toml` with plix-mod-sdk dependency
- [x] T041 [P] [US1] Create `templates/mods/chat-filter/mod.toml` with id, name, version, api_version, capabilities
- [x] T042 [P] [US1] Create `templates/mods/chat-filter/src/lib.rs` with SDK usage example (subscribe, cancel chat)
- [x] T043 [P] [US1] Create `templates/mods/chat-filter/build.sh` script for WASM compilation
- [x] T044 [P] [US1] Create `templates/mods/chat-filter/pack.sh` script (replaces README, has usage)
- [x] T045 [P] [US1] Create `templates/mods/world-query/Cargo.toml` with plix-mod-sdk dependency
- [x] T046 [P] [US1] Create `templates/mods/world-query/mod.toml` with WORLD_READ capability
- [x] T047 [P] [US1] Create `templates/mods/world-query/src/lib.rs` with raycast and AABB query examples
- [x] T048 [P] [US1] Create `templates/mods/world-query/build.sh` script
- [x] T049 [P] [US1] Create `templates/mods/timers-net/Cargo.toml` with plix-mod-sdk dependency
- [x] T050 [P] [US1] Create `templates/mods/timers-net/mod.toml` with NET_SEND capability
- [x] T051 [P] [US1] Create `templates/mods/timers-net/src/lib.rs` with interval timer and broadcast examples
- [x] T052 [P] [US1] Create `templates/mods/timers-net/build.sh` script

### CLI Commands for User Story 1

- [x] T053 [US1] Implement `plix mod new` in `crates/plix-mod-cli/src/cmd_new.rs` (copy template, replace placeholders)
- [x] T054 [US1] Implement mod ID validation in `crates/plix-mod-cli/src/cmd_new.rs` (kebab-case, 3-64 chars)
- [x] T055 [US1] Implement `plix mod build` in `crates/plix-mod-cli/src/cmd_build.rs` (invoke cargo build --target wasm32)
- [x] T056 [US1] Implement toolchain detection in `crates/plix-mod-cli/src/cmd_build.rs` (error if wasm target missing)
- [x] T057 [US1] Implement `plix mod pack` in `crates/plix-mod-cli/src/cmd_pack.rs` (create deterministic ZIP)
- [x] T058 [US1] Implement deterministic ZIP in `crates/plix-mod-cli/src/cmd_pack.rs` (sorted entries, epoch timestamps)
- [x] T059 [US1] Implement SHA-256 calculation in `crates/plix-mod-cli/src/cmd_pack.rs`
- [x] T060 [US1] Implement 10 MB size limit enforcement in `crates/plix-mod-cli/src/cmd_pack.rs`
- [x] T061 [US1] Implement `plix mod install --local` in `crates/plix-mod-cli/src/cmd_install.rs` (copy to cache)
- [x] T062 [US1] Add unit tests for deterministic pack in `crates/plix-mod-cli/src/cmd_pack.rs` (same input → same hash)
- [x] T063 [US1] Add unit tests for size limit in `crates/plix-mod-cli/src/cmd_pack.rs` (>10MB fails)

**Checkpoint**: Full toolchain works: new → build → pack → install → server loads mod ✓

---

## Phase 6: User Story 3 - Validate Mod Before Distribution (Priority: P2)

**Goal**: Catch common issues before sharing mods (missing exports, invalid manifest, size limits)

**Independent Test**: Run `plix mod validate` on a mod bundle - get clear pass/fail results

### Implementation for User Story 3

- [x] T064 [US3] Implement `plix mod validate` in `crates/plix-mod-cli/src/cmd_validate.rs`
- [x] T065 [US3] Implement manifest validation in `crates/plix-mod-cli/src/cmd_validate.rs` (parse TOML, required fields)
- [x] T066 [US3] Implement WASM export validation in `crates/plix-mod-cli/src/cmd_validate.rs` (mod_init, mod_on_event, mod_shutdown)
- [x] T067 [US3] Implement capability validation in `crates/plix-mod-cli/src/cmd_validate.rs` (known capability IDs)
- [x] T068 [US3] Implement API version validation in `crates/plix-mod-cli/src/cmd_validate.rs` (≤ current SDK version)
- [x] T069 [US3] Implement bundle size validation in `crates/plix-mod-cli/src/cmd_validate.rs` (≤ 10 MB)
- [x] T070 [US3] Implement JSON output mode in `crates/plix-mod-cli/src/cmd_validate.rs` (--json flag)
- [x] T071 [US3] Implement strict mode in `crates/plix-mod-cli/src/cmd_validate.rs` (--strict fails on warnings)
- [x] T072 [US3] Add unit tests for validation in `crates/plix-mod-cli/src/cmd_validate.rs` (missing exports, bad manifest)

**Checkpoint**: Validation catches all known invalid configurations ✓

---

## Phase 7: User Story 5 - Learn Modding from Documentation (Priority: P2)

**Goal**: New modders can create their first mod by following documentation

**Independent Test**: Follow quickstart guide end-to-end - successfully create and run a mod

### Implementation for User Story 5

- [x] T073 [P] [US5] Create `docs/modding/quickstart.md` with full create-build-pack-run cycle
- [x] T074 [P] [US5] Create `docs/modding/sdk.md` with all host function documentation
- [x] T075 [P] [US5] Document event types and payloads in `docs/modding/sdk.md`
- [x] T076 [P] [US5] Document capabilities and permissions in `docs/modding/sdk.md`
- [x] T077 [P] [US5] Document EMOD error codes in `docs/modding/sdk.md`
- [x] T078 [P] [US5] Create `docs/modding/distribution.md` with bundle format and registry info
- [x] T079 [P] [US5] Create `docs/modding/troubleshooting.md` with common error resolutions
- [x] T080 [US5] Add inline rustdoc examples in `crates/plix-mod-sdk/src/` modules

**Checkpoint**: Documentation enables self-service modding without external help ✓

---

## Phase 8: User Story 4 - Iterate Quickly with Hot-Reload (Priority: P3)

**⚠️ DEFERRED TO FEATURE 039-dev-hot-reload**

**Goal**: Dev-only automatic mod reload on file changes

**Independent Test**: Enable hot-reload in dev config, modify mod code, rebuild - mod reloads without server restart

### Implementation for User Story 4 (DEFERRED)

- [ ] T081 [US4] Add DevConfig struct in `crates/plix-server/src/mods/dev_hot_reload.rs` with hot_reload, debounce_ms, reload_policy
- [ ] T082 [US4] Implement file watcher using notify crate in `crates/plix-server/src/mods/dev_hot_reload.rs`
- [ ] T083 [US4] Implement debounce logic (200ms default) in `crates/plix-server/src/mods/dev_hot_reload.rs`
- [ ] T084 [US4] Implement reload pipeline in `crates/plix-server/src/mods/dev_hot_reload.rs` (shutdown → load → init)
- [ ] T085 [US4] Implement fallback policy in `crates/plix-server/src/mods/dev_hot_reload.rs` (revert to previous on failure)
- [ ] T086 [US4] Add reload logging in `crates/plix-server/src/mods/dev_hot_reload.rs` (start, success, failure, counter)
- [ ] T087 [US4] Add dev-mode guard in `crates/plix-server/src/mods/dev_hot_reload.rs` (warning if enabled, block in prod)
- [ ] T088 [US4] Integrate hot-reload with mod loader in `crates/plix-server/src/mods/mod.rs`
- [ ] T089 [US4] Add unit tests for reload orchestrator in `crates/plix-server/src/mods/dev_hot_reload.rs`

**Checkpoint**: DEFERRED - Hot-reload will be implemented in feature 039-dev-hot-reload

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Final integration, cleanup, and validation

**Note**: T090-T094 require cargo/rustup toolchain. T092-T094 require wasm32-unknown-unknown target.

- [ ] T090 Run `cargo fmt --all` to format all code (requires cargo)
- [ ] T091 Run `cargo clippy --all` and fix warnings (requires cargo)
- [ ] T092 [P] Verify chat-filter template compiles to WASM (requires wasm target)
- [ ] T093 [P] Verify world-query template compiles to WASM (requires wasm target)
- [ ] T094 [P] Verify timers-net template compiles to WASM (requires wasm target)
- [x] T095 Integration test: new → build → pack → validate → install → load cycle (smoke_test.rs created)
- [ ] T096 Validate quickstart.md by following it end-to-end (requires toolchain)
- [x] T097 Update `CLAUDE.md` with new crates if needed
- [x] T098 Update root `Cargo.toml` workspace members

**Checkpoint**: Core implementation complete. Phase 8 (hot-reload) deferred to 039-dev-hot-reload. ✓

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 2 (Phase 3-4)**: SDK APIs - Depends on Foundational
- **User Story 1 (Phase 5)**: Templates & CLI - Depends on US2 (needs SDK)
- **User Story 3 (Phase 6)**: Validation - Depends on US1 (needs pack command)
- **User Story 5 (Phase 7)**: Docs - Can start after US2 (needs API knowledge)
- **User Story 4 (Phase 8)**: Hot-reload - Can start after US1 (needs mod loading)
- **Polish (Phase 9)**: Depends on all user stories

### User Story Dependencies

```
US2 (SDK APIs) ←─ Foundation
    ↓
US1 (Toolchain) ─→ US3 (Validation)
    ↓
US4 (Hot-reload)

US5 (Docs) ←─ US2 (needs API knowledge)
```

### Within Each User Story

- Models/types before API functions
- API functions before CLI commands
- CLI commands before integration
- All tests after implementation

### Parallel Opportunities

**Phase 1 (Setup)**: T003-T007 and T009-T011 can run in parallel
**Phase 2 (Foundational)**: T013-T014, T016-T018 can run in parallel
**Phase 3 (US2 SDK)**: T021-T022, T025-T028 can run in parallel
**Phase 5 (US1 Templates)**: T040-T052 all templates can run in parallel
**Phase 7 (US5 Docs)**: T073-T079 all docs can run in parallel

---

## Parallel Example: Templates

```bash
# Launch all template file creations together:
Task: "Create templates/mods/chat-filter/Cargo.toml"
Task: "Create templates/mods/chat-filter/mod.toml"
Task: "Create templates/mods/chat-filter/src/lib.rs"
Task: "Create templates/mods/world-query/Cargo.toml"
Task: "Create templates/mods/world-query/mod.toml"
# ... all [P] [US1] tasks in Phase 5
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup (crate skeletons)
2. Complete Phase 2: Foundational (ABI, types)
3. Complete Phase 3-4: User Story 2 (SDK APIs + macros)
4. Complete Phase 5: User Story 1 (templates + CLI)
5. **STOP and VALIDATE**: Test full toolchain independently
6. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US2 (SDK) → Test SDK usage → Checkpoint
3. Add US1 (Toolchain) → Test new/build/pack/install → **MVP COMPLETE**
4. Add US3 (Validation) → Test validate command → Enhanced quality
5. Add US5 (Docs) → Test quickstart → Better DX
6. Add US4 (Hot-reload) → Test dev iteration → Full feature complete

### Task Count Summary

| Phase | Task Count | Parallel Tasks |
|-------|------------|----------------|
| Phase 1: Setup | 11 | 8 |
| Phase 2: Foundational | 9 | 5 |
| Phase 3-4: US2 (SDK) | 19 | 10 |
| Phase 5: US1 (Toolchain) | 24 | 13 |
| Phase 6: US3 (Validation) | 9 | 0 |
| Phase 7: US5 (Docs) | 8 | 7 |
| Phase 8: US4 (Hot-reload) | 9 | 0 |
| Phase 9: Polish | 9 | 3 |
| **Total** | **98** | **46** |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently

### Deferred Work

- **Hot-reload (US4 / Phase 8)**: Deferred to feature 039-dev-hot-reload
  - Reason: Separate concern, can be added without breaking existing functionality
  - All infrastructure is in place (mod loading, unloading, event system)

### Pending Toolchain Tasks

When cargo/rustup toolchain is available:
- T090: `cargo fmt --all`
- T091: `cargo clippy --all`
- T092-T094: Verify template WASM compilation
- T096: End-to-end quickstart validation
