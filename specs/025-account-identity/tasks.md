# Tasks: Account Identity

**Input**: Design documents from `/specs/025-account-identity/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Tests are included as user stories reference specific test scenarios.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4, US5)
- Include exact file paths in descriptions

## Path Conventions

Existing Rust workspace structure:
```
crates/
├── plix-common/src/     # Shared types, protocol
├── plix-client/src/     # Client application
├── plix-server/src/     # Server application
└── plix-server/tests/   # Integration tests
```

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create identity module structure in all crates

- [x] T001 Create identity module structure in `crates/plix-common/src/identity/mod.rs`
- [x] T002 [P] Create identity module structure in `crates/plix-server/src/identity/mod.rs`
- [x] T003 [P] Create profile module structure in `crates/plix-client/src/profile/mod.rs`
- [x] T004 Add module declarations to `crates/plix-common/src/lib.rs`, `crates/plix-server/src/lib.rs`, `crates/plix-client/src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Create DisplayName type with validation in `crates/plix-common/src/identity/display_name.rs` (newtype struct, MIN_LEN=1, MAX_LEN=32, allowed chars: a-zA-Z0-9_- )
- [x] T006 [P] Implement DisplayName validation: trim whitespace, validate length/charset, return Result<DisplayName, DisplayNameError> in `crates/plix-common/src/identity/display_name.rs`
- [x] T007 [P] Implement DisplayName::sanitize() fallback to "Player" when invalid in `crates/plix-common/src/identity/display_name.rs`
- [x] T008 [P] Implement base_name() extraction (strip #N suffix) in `crates/plix-common/src/identity/display_name.rs`
- [x] T009 [P] Create SessionId type (u64 newtype, NONE=0, Display impl) in `crates/plix-common/src/identity/session.rs`
- [x] T010 [P] Create AccountId placeholder type (v2, unused) in `crates/plix-common/src/identity/session.rs`
- [x] T011 Add unit tests for DisplayName validation (valid/invalid/sanitize) in `crates/plix-common/src/identity/display_name.rs`
- [x] T012 [P] Add unit tests for SessionId/AccountId in `crates/plix-common/src/identity/session.rs`

**Checkpoint**: Core identity types ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Player Display Name Setup (Priority: P1) 🎯 MVP

**Goal**: Players can choose a display name that is validated and applied by the server

**Independent Test**: Connect a client with a custom name, verify server logs show the name and other clients see it in PlayerSnapshot

### Implementation for User Story 1

- [ ] T013 [US1] Extend ClientMessage::Connect with optional account_id and auth_token fields (v2 placeholders) in `crates/plix-common/src/protocol/messages.rs`
- [ ] T014 [P] [US1] Extend ServerMessage::Connected with display_name and session_id fields in `crates/plix-common/src/protocol/messages.rs`
- [ ] T015 [P] [US1] Add display_name field to PlayerSnapshot with #[serde(default)] in `crates/plix-common/src/protocol/messages.rs`
- [ ] T016 [US1] Add session_id: SessionId field to ServerPlayer in `crates/plix-server/src/session.rs`
- [ ] T017 [P] [US1] Create NameRegistry struct in `crates/plix-server/src/identity/name_registry.rs` (HashSet<String> active_names, HashMap<String, HashSet<u32>> suffix_map, HashMap<PlayerId, String> player_names)
- [ ] T018 [US1] Implement NameRegistry::register() with disambiguation logic (#2, #3..., max 99) in `crates/plix-server/src/identity/name_registry.rs`
- [ ] T019 [P] [US1] Implement NameRegistry::unregister() to free name on disconnect in `crates/plix-server/src/identity/name_registry.rs`
- [ ] T020 [P] [US1] Implement NameRegistry::get_name() lookup in `crates/plix-server/src/identity/name_registry.rs`
- [ ] T021 [US1] Implement fallback name generation ("Player") when input invalid/empty in `crates/plix-server/src/identity/name_registry.rs`
- [ ] T022 [US1] Generate unique SessionId on connection (monotonic counter) in `crates/plix-server/src/netloop.rs`
- [ ] T023 [US1] Integrate NameRegistry into connection handler: validate name, register, assign in `crates/plix-server/src/netloop.rs`
- [ ] T024 [US1] Send assigned display_name and session_id in Connected response in `crates/plix-server/src/netloop.rs`
- [ ] T025 [US1] Add display_name to ReplicatedPlayer struct in `crates/plix-server/src/replication/state.rs`
- [ ] T026 [US1] Include display_name in PlayerSnapshot generation in `crates/plix-server/src/replication/snapshot.rs`
- [ ] T027 [US1] Call NameRegistry::unregister on player disconnect in `crates/plix-server/src/netloop.rs`
- [ ] T028 [US1] Add structured logging: connect with player_id, session_id, display_name in `crates/plix-server/src/netloop.rs`
- [ ] T029 [US1] Unit tests for NameRegistry: register unique, register duplicate (#2), unregister frees name in `crates/plix-server/src/identity/name_registry.rs`

**Checkpoint**: Player can connect with custom name, server validates/disambiguates, name appears in snapshots

---

## Phase 4: User Story 2 - Local Profile Persistence (Priority: P1)

**Goal**: Player's display name is saved locally and loaded on startup

**Independent Test**: Set a display name, close client, reopen, verify saved name is loaded automatically

### Implementation for User Story 2

- [ ] T030 [P] [US2] Create PlayerProfile struct (version: u32, display_name: String, optional account_id/auth_token) in `crates/plix-client/src/profile/player_profile.rs`
- [ ] T031 [P] [US2] Implement profile_path() function returning ~/.config/plix/profile.toml in `crates/plix-client/src/profile/player_profile.rs`
- [ ] T032 [US2] Implement load_profile() with TOML parsing and Default fallback in `crates/plix-client/src/profile/player_profile.rs`
- [ ] T033 [P] [US2] Implement save_profile() with atomic write (temp file + rename) in `crates/plix-client/src/profile/player_profile.rs`
- [ ] T034 [US2] Handle corrupted profile file: log warning, recreate defaults in `crates/plix-client/src/profile/player_profile.rs`
- [ ] T035 [US2] Load profile on client startup and use display_name in Connect message in `crates/plix-client/src/main.rs`
- [ ] T036 [US2] Unit tests: load missing -> defaults, save/load roundtrip, corrupted -> defaults in `crates/plix-client/src/profile/player_profile.rs`

**Checkpoint**: Client persists display name across restarts

---

## Phase 5: User Story 3 - Display Name Uniqueness on Server (Priority: P2)

**Goal**: Server prevents duplicate display names by adding #N suffixes

**Independent Test**: Connect two clients with same name, verify second gets #2 suffix

### Implementation for User Story 3

- [ ] T037 [US3] Add suffix_map tracking to NameRegistry (HashMap<String, HashSet<u32>>) in `crates/plix-server/src/identity/name_registry.rs`
- [ ] T038 [US3] Implement find_available_name() to get lowest available suffix in `crates/plix-server/src/identity/name_registry.rs`
- [ ] T039 [US3] Handle max suffix exhaustion (99): reject with "server full for this name" in `crates/plix-server/src/identity/name_registry.rs`
- [ ] T040 [US3] Reuse freed suffixes when player disconnects in `crates/plix-server/src/identity/name_registry.rs`
- [ ] T041 [US3] Unit tests: Alex#2 on duplicate, suffix reuse on disconnect, max suffix rejection in `crates/plix-server/src/identity/name_registry.rs`
- [ ] T042 [US3] Integration test: two clients same name -> second gets suffix in `crates/plix-server/tests/identity_test.rs`

**Checkpoint**: Duplicate names are automatically disambiguated

---

## Phase 6: User Story 4 - In-Game Name Change (Priority: P2)

**Goal**: Players can change display name during session via /name command

**Independent Test**: Connect with one name, use /name command, verify new name is applied and broadcast

### Implementation for User Story 4

- [ ] T043 [P] [US4] Add ClientMessage::RenameRequest { new_name: String } to protocol in `crates/plix-common/src/protocol/messages.rs`
- [ ] T044 [P] [US4] Add RenameRejectReason enum (RateLimited, InvalidName, NameUnavailable) in `crates/plix-common/src/protocol/messages.rs`
- [ ] T045 [P] [US4] Add GameEvent::RenameResult { success, new_name, reason } in `crates/plix-common/src/protocol/messages.rs`
- [ ] T046 [P] [US4] Add GameEvent::PlayerRenamed { player_id, old_name, new_name } in `crates/plix-common/src/protocol/messages.rs`
- [ ] T047 [US4] Add last_rename_tick: Option<Tick> field to ServerPlayer in `crates/plix-server/src/session.rs`
- [ ] T048 [P] [US4] Implement can_rename() and record_rename() helpers on ServerPlayer in `crates/plix-server/src/session.rs`
- [ ] T049 [US4] Define RENAME_COOLDOWN_TICKS = 3600 (60s at 60 TPS) in `crates/plix-server/src/identity/mod.rs`
- [ ] T050 [US4] Implement NameRegistry::rename() to change player's name in `crates/plix-server/src/identity/name_registry.rs`
- [ ] T051 [US4] Handle RenameRequest in netloop: validate cooldown, validate name, call registry.rename in `crates/plix-server/src/netloop.rs`
- [ ] T052 [US4] Send RenameResult to requester in `crates/plix-server/src/netloop.rs`
- [ ] T053 [US4] Broadcast PlayerRenamed event to all clients on success in `crates/plix-server/src/netloop.rs`
- [ ] T054 [US4] Add /name <new_name> command parsing in `crates/plix-client/src/console.rs`
- [ ] T055 [P] [US4] Update help text to include /name command in `crates/plix-client/src/console.rs`
- [ ] T056 [US4] Save updated display_name to profile.toml on successful rename in `crates/plix-client/src/main.rs` or net handler
- [ ] T057 [US4] Add structured logging: rename with session_id, old_name, new_name in `crates/plix-server/src/netloop.rs`
- [ ] T058 [US4] Unit tests: rename cooldown enforcement in `crates/plix-server/src/session.rs`
- [ ] T059 [P] [US4] Unit tests: /name command parsing in `crates/plix-client/src/console.rs`
- [ ] T060 [US4] Integration test: rename success, rename rate limited in `crates/plix-server/tests/identity_test.rs`

**Checkpoint**: Players can change names with rate limiting

---

## Phase 7: User Story 5 - Session Identity Tracking (Priority: P3)

**Goal**: Server assigns unique SessionId for logging and correlation

**Independent Test**: Connect player, verify server logs show PlayerId, SessionId, and display name

### Implementation for User Story 5

- [ ] T061 [US5] Add session_id_counter to server state for monotonic generation in `crates/plix-server/src/lib.rs` or main
- [ ] T062 [US5] Log player disconnect with session_id, display_name, duration in `crates/plix-server/src/netloop.rs`
- [ ] T063 [US5] Include session_id in rename logs for correlation in `crates/plix-server/src/netloop.rs`
- [ ] T064 [US5] Unit tests: SessionId uniqueness across connections in `crates/plix-server/tests/identity_test.rs`

**Checkpoint**: All player events are traceable via SessionId

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Integration validation and documentation

- [ ] T065 [P] Integration test: full multi-player scenario (A=Alex, B=Alex#2, rename, disconnect, reuse) in `crates/plix-server/tests/identity_test.rs`
- [ ] T066 [P] Non-regression test: verify TDM/FFA/CTF/BR/Training still work with identity changes in `crates/plix-server/tests/`
- [ ] T067 [P] Non-regression test: client without profile.toml can connect (uses defaults) in `crates/plix-server/tests/identity_test.rs`
- [ ] T068 [P] Verify display_name appears in PlayerJoined event with correct value in `crates/plix-server/src/netloop.rs`
- [ ] T069 Run cargo clippy and cargo fmt on all modified files
- [ ] T070 Run full test suite: cargo test --workspace

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - US1 and US2 can proceed in parallel (different crates)
  - US3 depends on US1 (builds on NameRegistry)
  - US4 depends on US1 and US3 (uses NameRegistry)
  - US5 can proceed in parallel with US3/US4
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - Core display name flow
- **User Story 2 (P1)**: Can start after Foundational - Client profile (parallel with US1)
- **User Story 3 (P2)**: Depends on US1 - Builds on NameRegistry
- **User Story 4 (P2)**: Depends on US1 + US3 - Uses NameRegistry + adds rename
- **User Story 5 (P3)**: Can start after Foundational - SessionId logging (parallel)

### Parallel Opportunities

**Within Phase 2 (Foundational)**:
- T006, T007, T008 can run in parallel (different functions)
- T009, T010 can run in parallel (different types)

**Within Phase 3 (US1)**:
- T014, T015 can run in parallel (different message structs)
- T017, T019, T020 can run in parallel (different NameRegistry methods)

**Within Phase 4 (US2)**:
- T030, T031, T033 can run in parallel (different functions)

**Within Phase 6 (US4)**:
- T043, T044, T045, T046 can run in parallel (different protocol types)

---

## Parallel Example: Phase 2 (Foundational)

```bash
# Launch all type creation tasks together:
Task: "Create DisplayName type in crates/plix-common/src/identity/display_name.rs"
Task: "Create SessionId type in crates/plix-common/src/identity/session.rs"
Task: "Create AccountId placeholder in crates/plix-common/src/identity/session.rs"
```

## Parallel Example: Phase 3 (US1)

```bash
# Launch model creation tasks together:
Task: "Extend ServerMessage::Connected in crates/plix-common/src/protocol/messages.rs"
Task: "Add display_name to PlayerSnapshot in crates/plix-common/src/protocol/messages.rs"

# Then launch NameRegistry implementation:
Task: "Create NameRegistry struct in crates/plix-server/src/identity/name_registry.rs"
Task: "Implement NameRegistry::unregister() in crates/plix-server/src/identity/name_registry.rs"
Task: "Implement NameRegistry::get_name() in crates/plix-server/src/identity/name_registry.rs"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: Foundational (T005-T012) - CRITICAL
3. Complete Phase 3: User Story 1 (T013-T029)
4. Complete Phase 4: User Story 2 (T030-T036)
5. **STOP and VALIDATE**: Players can connect with custom names, names persist
6. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Types ready
2. Add US1 + US2 → Test independently → Deploy (MVP with persistent names!)
3. Add US3 → Test disambiguation → Deploy
4. Add US4 → Test rename command → Deploy
5. Add US5 → Test logging → Deploy
6. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (server-side)
   - Developer B: User Story 2 (client-side)
3. After US1 complete:
   - Developer A: User Story 3 + 4
   - Developer B: User Story 5
4. Final: Polish phase together

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Rate limit: 60 seconds (3600 ticks at 60 TPS) per spec
- Name validation: 1-32 chars, alphanumeric + underscore/hyphen/space
- Profile location: ~/.config/plix/profile.toml (XDG compliant)
