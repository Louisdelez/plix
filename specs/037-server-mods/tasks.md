# Tasks: Server Mods + Client Sync

**Input**: Design documents from `/specs/037-server-mods/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Server**: `crates/plix-server/src/`
- **Client**: `crates/plix-client/src/`
- **Common**: `crates/plix-common/src/`
- **Mod Distribution**: `crates/plix-mod-distribution/src/`
- **Tests**: `tests/integration/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and mod distribution extensions

- [X] T001 Create mods module directory structure in `crates/plix-client/src/mods/`
- [X] T002 [P] Add new module files: `crates/plix-client/src/mods/mod.rs`
- [X] T003 [P] Register mods module in `crates/plix-client/src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and configuration that MUST be complete before ANY user story

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Manifest Extension (T-001)

- [X] T004 [P] Add `RuntimeMode` enum to `crates/plix-mod-distribution/src/lib.rs` with values: `Server`, `Client`, `Both`
- [X] T005 [P] Add `client_payload` and `client_payload_files` fields to mod manifest struct in `crates/plix-mod-distribution/src/bundle.rs`
- [X] T006 Implement manifest validation: reject invalid runtime, require files when client_payload=true in `crates/plix-mod-distribution/src/bundle.rs`
- [X] T007 [P] Add unit tests for manifest parsing in `crates/plix-mod-distribution/src/bundle.rs` (valid/invalid combinations)

### Server Configuration (T-002)

- [X] T008 [P] Add `JoinPolicy` struct with fields (allow_server_only, allow_payload_sync, require_payload_sync) to `crates/plix-mod-distribution/src/config.rs`
- [X] T009 [P] Add `SyncConfig` struct with fields (max_payload_mb, chunk_size_kb, max_inflight_chunks, transfer_timeout_secs) to `crates/plix-mod-distribution/src/config.rs`
- [X] T010 Extend `DistributionConfig` to include `join_policy` and `sync` sections in `crates/plix-mod-distribution/src/config.rs`
- [X] T011 Add config validation (bounds checking: payload <= max, chunk_size > 0, inflight >= 1) in `crates/plix-mod-distribution/src/config.rs`
- [X] T012 [P] Add unit tests for config parsing/validation in `crates/plix-mod-distribution/src/config.rs`

### Core Protocol Types (T-003, T-004)

- [X] T013 [P] Create `ModSetDescriptor` struct in `crates/plix-common/src/protocol/messages.rs` (moved from modset.rs)
- [X] T014 [P] Create `ModDescriptor` struct in `crates/plix-common/src/protocol/messages.rs` (id, version, payload_hash, payload_size)
- [X] T015 Create `SyncSession` to manage mod sync handshake in `crates/plix-server/src/mods/sync_session.rs`
- [X] T016 [P] Add unit tests for SyncSession in `crates/plix-server/src/mods/sync_session.rs`

### Protocol Messages (T-004)

- [X] T017 Add `ServerMessage::ModSet` variant to `crates/plix-common/src/protocol/messages.rs`
- [X] T018 Add `ClientMessage::ModSetResponse` variant (supports_sync, cached_payload_hashes) to `crates/plix-common/src/protocol/messages.rs`
- [X] T019 Add `ServerMessage::JoinDecision` variant to `crates/plix-common/src/protocol/messages.rs`
- [X] T020 Add `ModSyncRejectReason` enum to `crates/plix-common/src/protocol/messages.rs`
- [X] T021 [P] Add payload transfer messages (PayloadBegin, PayloadChunk, PayloadAck, PayloadEnd) to `crates/plix-common/src/protocol/messages.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Join Server-Only Modded Server (Priority: P1) 🎯 MVP

**Goal**: Players can join servers with server-only mods without installing anything client-side

**Independent Test**: Connect vanilla client to server with 2-3 server-only mods. Join succeeds, mod effects visible.

### Implementation for User Story 1

- [X] T022 [US1] Implement server handshake: send ModSetDescriptor after Connect in `crates/plix-server/src/lib.rs`
- [X] T023 [US1] Create `ClientCapabilities` struct in `crates/plix-client/src/mods/handshake.rs`
- [X] T024 [US1] Implement client handshake: receive ModSetDescriptor, build response in `crates/plix-client/src/mods/handshake.rs`
- [X] T025 [US1] Implement server join decision for server-only mods in `crates/plix-server/src/mods/sync_session.rs`
- [X] T026 [US1] Send JoinDecision::Ok and finalize connection in `crates/plix-server/src/lib.rs`
- [X] T027 [US1] Handle JoinDecision on client side (Ok → continue, Refused → show message) in `crates/plix-client/src/lib.rs`
- [X] T028 [P] [US1] Add handshake event logging (modset sent, decision) in `crates/plix-server/src/mods/sync_session.rs`
- [X] T029 [US1] Integration test: server-only mods, vanilla client joins OK in `crates/plix-server/tests/mod_integration_test.rs`

**Checkpoint**: User Story 1 complete - vanilla clients can join server-only modded servers

---

## Phase 4: User Story 2 - Client-Required Mod Enforcement (Priority: P2)

**Goal**: Server can block clients missing required mods with clear error messages

**Independent Test**: Configure client-required mod. Client without it → refused with clear message.

### Implementation for User Story 2

- [ ] T030 [US2] Extend join decision logic for client-required mods in `crates/plix-server/src/mods/join_policy.rs`
- [ ] T031 [US2] Implement refusal with reason code EMJOIN001 (client_mod_missing) in `crates/plix-server/src/mods/join_policy.rs`
- [ ] T032 [US2] Format user-friendly refusal messages ("Server requires mod '{id}' version {ver}") in `crates/plix-server/src/mods/join_policy.rs`
- [ ] T033 [US2] Display refusal message on client disconnect in `crates/plix-client/src/net.rs`
- [ ] T034 [P] [US2] Add metric: joins_refused_mod_mismatch in `crates/plix-server/src/mods/join_policy.rs`
- [ ] T035 [US2] Integration test: client-required missing → REFUSED in `tests/integration/mod_sync_test.rs`

**Checkpoint**: User Story 2 complete - client-required enforcement works

---

## Phase 5: User Story 3 - Client Data Payload Synchronization (Priority: P2)

**Goal**: Secure, chunked transfer of data-only payloads with caching

**Independent Test**: 5MB payload mod, new client → transfer completes, SHA-256 verified, cached, reconnect skips download.

### Payload Packaging (T-007)

- [ ] T036 [US3] Create `ClientPayloadBuilder` in `crates/plix-server/src/mods/client_payload.rs`
- [ ] T037 [US3] Build deterministic ZIP archive from client_payload_files in `crates/plix-server/src/mods/client_payload.rs`
- [ ] T038 [US3] Calculate SHA-256 hash and size, validate against max_payload_mb in `crates/plix-server/src/mods/client_payload.rs`
- [ ] T039 [P] [US3] Add unit tests: hash stability, size cap enforcement in `crates/plix-server/src/mods/client_payload.rs`

### Payload Transfer Protocol (T-008)

- [ ] T040 [P] [US3] Add `ServerMessage::PayloadBegin` (hash, total_size, chunk_size, num_chunks) to `crates/plix-common/src/protocol/messages.rs`
- [ ] T041 [P] [US3] Add `ServerMessage::PayloadChunk` (hash, index, data) to `crates/plix-common/src/protocol/messages.rs`
- [ ] T042 [P] [US3] Add `ServerMessage::PayloadEnd` (hash) to `crates/plix-common/src/protocol/messages.rs`
- [ ] T043 [P] [US3] Add `ClientMessage::PayloadAck` (hash) to `crates/plix-common/src/protocol/messages.rs`
- [ ] T044 [P] [US3] Add `ClientMessage::PayloadResendRequest` (hash, missing_indices) to `crates/plix-common/src/protocol/messages.rs`
- [ ] T045 [P] [US3] Add unit tests for payload message encode/decode in `crates/plix-common/src/protocol/messages.rs`

### Server Payload Sender (T-009)

- [ ] T046 [US3] Create `PayloadSender` struct in `crates/plix-server/src/mods/payload_transfer.rs`
- [ ] T047 [US3] Implement chunked streaming with max_inflight_chunks in `crates/plix-server/src/mods/payload_transfer.rs`
- [ ] T048 [US3] Handle PayloadAck and PayloadResendRequest in `crates/plix-server/src/mods/payload_transfer.rs`
- [ ] T049 [US3] Add transfer timeout handling (abort + refuse join) in `crates/plix-server/src/mods/payload_transfer.rs`
- [ ] T050 [P] [US3] Add debug logging for transfer progress in `crates/plix-server/src/mods/payload_transfer.rs`
- [ ] T051 [P] [US3] Add unit tests: chunker, inflight window in `crates/plix-server/src/mods/payload_transfer.rs`

### Client Payload Receiver (T-010)

- [ ] T052 [US3] Create `PayloadReceiver` struct in `crates/plix-client/src/mods/payload_receiver.rs`
- [ ] T053 [US3] Implement chunk reassembly in `crates/plix-client/src/mods/payload_receiver.rs`
- [ ] T054 [US3] Verify num_chunks, detect missing indices in `crates/plix-client/src/mods/payload_receiver.rs`
- [ ] T055 [US3] Compute SHA-256 on complete payload, compare with expected in `crates/plix-client/src/mods/payload_receiver.rs`
- [ ] T056 [US3] Handle mismatch: purge transfer, notify disconnect in `crates/plix-client/src/mods/payload_receiver.rs`
- [ ] T057 [P] [US3] Add unit tests: reassembly OK, missing chunks, hash mismatch in `crates/plix-client/src/mods/payload_receiver.rs`

### Client Payload Cache (T-011)

- [ ] T058 [US3] Create `PayloadCache` struct in `crates/plix-client/src/mods/payload_cache.rs`
- [ ] T059 [US3] Implement store/load by SHA-256 hash in `crates/plix-client/src/mods/payload_cache.rs`
- [ ] T060 [US3] Store metadata (mod_id, version, timestamp) alongside payload in `crates/plix-client/src/mods/payload_cache.rs`
- [ ] T061 [US3] Update handshake response to include cached_payload_hashes in `crates/plix-client/src/mods/handshake.rs`
- [ ] T062 [P] [US3] Add unit tests: store/load, cache hit detection in `crates/plix-client/src/mods/payload_cache.rs`

### Client Data Loader (T-012)

- [ ] T063 [P] [US3] Create `ClientModDataRegistry` struct in `crates/plix-client/src/mods/client_data_loader.rs`
- [ ] T064 [US3] Implement extract/parse from payload archive in `crates/plix-client/src/mods/client_data_loader.rs`
- [ ] T065 [P] [US3] Add unit tests: JSON/TOML loading, parse errors in `crates/plix-client/src/mods/client_data_loader.rs`

### Join Decision Integration (T-005, T-006, T-013)

- [ ] T066 [US3] Extend join decision for payload sync (cache hit → OK, cache miss → SYNC_REQUIRED) in `crates/plix-server/src/mods/join_policy.rs`
- [ ] T067 [US3] Integrate PayloadSender with handshake flow in `crates/plix-server/src/netloop.rs`
- [ ] T068 [US3] Finalize join after PayloadAck received in `crates/plix-server/src/netloop.rs`
- [ ] T069 [P] [US3] Add metrics: payload_sync_bytes, payload_sync_failures, payload_cache_hits in `crates/plix-server/src/mods/payload_transfer.rs`

### Integration Tests (T-017 subset)

- [ ] T070 [US3] Integration test: payload required, sync → transfer OK → join OK in `tests/integration/mod_sync_test.rs`
- [ ] T071 [US3] Integration test: payload mismatch → REFUSED + purge in `tests/integration/mod_sync_test.rs`
- [ ] T072 [US3] Integration test: cache hit → skip download → join OK in `tests/integration/mod_sync_test.rs`
- [ ] T073 [US3] Integration test: payload > max_payload_mb → REFUSED in `tests/integration/mod_sync_test.rs`

**Checkpoint**: User Story 3 complete - payload sync works end-to-end

---

## Phase 6: User Story 4 - Mod Network Channels (Priority: P3)

**Goal**: Server-side mods can send messages to clients via dedicated channels

**Independent Test**: Server mod sends message, client receives on `mod:<id>:*` channel.

### Server-to-Client Channels (T-014)

- [ ] T074 [US4] Add `ServerMessage::ModMessage` (channel, data) to `crates/plix-common/src/protocol/messages.rs`
- [ ] T075 [US4] Implement channel validation: allow `mod:<id>:*` only for loaded mods in `crates/plix-server/src/mods/net_policy.rs`
- [ ] T076 [P] [US4] Add unit tests: valid/invalid channel in `crates/plix-server/src/mods/net_policy.rs`

### Client-to-Server Channels (T-015)

- [ ] T077 [US4] Add `ClientMessage::ModMessage` (channel, data) to `crates/plix-common/src/protocol/messages.rs`
- [ ] T078 [US4] Add `allowed_client_channels` field to mod manifest network config in `crates/plix-mod-distribution/src/bundle.rs`
- [ ] T079 [US4] Implement client-to-server channel gating (reject non-allowlisted) in `crates/plix-server/src/mods/net_policy.rs`
- [ ] T080 [US4] Implement spoof protection: reject `mod:<other_id>:*` from wrong mod in `crates/plix-server/src/mods/net_policy.rs`
- [ ] T081 [US4] Verify rate limits via Feature 034 infrastructure in `crates/plix-server/src/mods/net_policy.rs`
- [ ] T082 [P] [US4] Add unit tests: allow/deny/spoof scenarios in `crates/plix-server/src/mods/net_policy.rs`

**Checkpoint**: User Story 4 complete - mod network channels work securely

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, observability, final validation

### Observability (T-016)

- [ ] T083 [P] Ensure handshake logging is complete (modset sent, decision, reasons) in `crates/plix-server/src/mods/join_policy.rs`
- [ ] T084 [P] Ensure sync logging is complete (begin/end, mismatch, retries at debug level) in `crates/plix-server/src/mods/payload_transfer.rs`
- [ ] T085 Add metrics test: verify counter increments in `tests/integration/mod_sync_test.rs`

### Documentation (T-018)

- [ ] T086 Create `docs/feature-037.md` with: manifest fields, ModSetDescriptor schema, handshake messages
- [ ] T087 Add sync protocol documentation (chunks, limits) to `docs/feature-037.md`
- [ ] T088 Add join policy matrix and net policy channels documentation to `docs/feature-037.md`
- [ ] T089 Add server configuration defaults documentation to `docs/feature-037.md`

### Final Validation (T-019)

- [ ] T090 Run all integration tests and verify CI pass
- [ ] T091 Verify: vanilla client joins server-only modded server
- [ ] T092 Verify: payload sync works (chunks, SHA-256, cache hit)
- [ ] T093 Verify: client-required missing blocks join properly
- [ ] T094 Verify: mod channels safe (anti-spoof, allowlist, rate limit)
- [ ] T095 Run quickstart.md validation scenarios

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - US1 (P1) and US2/US3 (P2) and US4 (P3) can proceed in priority order
  - US2 and US3 share same priority, can run in parallel
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P2)**: Builds on US1 handshake but independently testable
- **User Story 3 (P2)**: Builds on US1 handshake but independently testable
- **User Story 4 (P3)**: Independent of US2/US3, can run in parallel

### Within Each User Story

- Protocol messages before implementation
- Server-side before client-side (for handshake flow)
- Core implementation before integration tests
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Protocol message tasks (T040-T045) can run in parallel
- Cache and data loader tasks can run in parallel
- Unit tests can run in parallel with other file implementations

---

## Parallel Example: Foundational Phase

```bash
# Launch all parallel foundational tasks together:
Task: "Add RuntimeMode enum in crates/plix-mod-distribution/src/lib.rs" (T004)
Task: "Add client_payload fields in crates/plix-mod-distribution/src/bundle.rs" (T005)
Task: "Add JoinPolicy struct in crates/plix-mod-distribution/src/config.rs" (T008)
Task: "Add SyncConfig struct in crates/plix-mod-distribution/src/config.rs" (T009)
Task: "Create ModSetDescriptor in crates/plix-server/src/mods/modset.rs" (T013)
Task: "Create ModEntry in crates/plix-server/src/mods/modset.rs" (T014)
```

## Parallel Example: User Story 3 Payload Protocol

```bash
# Launch all payload message tasks together:
Task: "Add PayloadBegin message" (T040)
Task: "Add PayloadChunk message" (T041)
Task: "Add PayloadEnd message" (T042)
Task: "Add PayloadAck message" (T043)
Task: "Add PayloadResendRequest message" (T044)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Vanilla client can join server-only modded server
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → **MVP deployed!**
3. Add User Story 2 → Client-required enforcement works
4. Add User Story 3 → Payload sync works
5. Add User Story 4 → Mod channels work
6. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (P1 - critical path)
   - Developer B: User Story 2 (P2)
   - Developer C: User Story 3 (P2)
3. After US1-3 complete, anyone can take US4 (P3)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
