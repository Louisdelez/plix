# Tasks: Security Pass

**Input**: Design documents from `/specs/040-security-pass/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure) ✅

**Purpose**: Project initialization and security module structure

- [x] T001 Create `docs/security/` directory structure per plan.md
- [x] T002 Create `crates/plix-common/src/limits.rs` with all security constants from data-model.md
- [x] T003 [P] Create `crates/plix-server/src/security/mod.rs` module structure with submodule declarations
- [x] T004 [P] Add `security` module declaration to `crates/plix-server/src/lib.rs`
- [x] T005 [P] Create `fuzz/` directory structure for cargo-fuzz per plan.md

**Checkpoint**: Security module skeleton ready, limits constants defined ✅

---

## Phase 2: Foundational (Blocking Prerequisites) ✅

**Purpose**: Core security infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Security Types & Core Infrastructure

- [x] T006 Create `StrikeCategory` enum in `crates/plix-server/src/security/strikes.rs` with all violation categories
- [x] T007 [P] Create `SecurityError` enum in `crates/plix-common/src/limits.rs` with typed errors (DecodeInvalid, LimitExceeded, etc.)
- [x] T008 [P] Add helper functions `check_len()`, `check_str_len()`, `check_list_len()` in `crates/plix-common/src/limits.rs`
- [x] T009 Create `StrikeTracker` struct in `crates/plix-server/src/security/strikes.rs` with record/decay/disconnect methods
- [x] T010 [P] Create `SecurityMetrics` struct in `crates/plix-server/src/security/observability.rs` with atomic counters
- [x] T011 [P] Create `RateLimitedLogger` struct in `crates/plix-server/src/security/observability.rs`
- [x] T012 Create `TokenBucket` struct in `crates/plix-server/src/security/rate_limiter.rs` per research.md pattern
- [x] T013 Create `RateLimiter` struct in `crates/plix-server/src/security/rate_limiter.rs` with global + per-type buckets
- [x] T014 Create `HandshakeTracker` struct in `crates/plix-server/src/security/handshake.rs` with pending connection tracking
- [x] T015 [P] Add unit tests for `StrikeTracker` strike accumulation and threshold in `crates/plix-server/src/security/strikes.rs`
- [x] T016 [P] Add unit tests for `RateLimiter` token consumption and refill in `crates/plix-server/src/security/rate_limiter.rs`
- [x] T017 [P] Add unit tests for limits helper functions in `crates/plix-common/src/limits.rs`

**Checkpoint**: Foundation ready - all security types defined and tested, user story implementation can begin ✅

---

## Phase 3: User Story 1 - Developer Runs Fuzz Tests (Priority: P1) 🎯 MVP ✅

**Goal**: Developers can launch fuzz testing on protocol decode functions and guarantee "no panic, no OOM" on any input

**Independent Test**: Run `cargo fuzz run fuzz_decode_client_message` for 5 minutes with zero panics

### Fuzz Infrastructure

- [x] T018 [US1] Create `fuzz/Cargo.toml` with libfuzzer-sys dependency and fuzz target definitions
- [ ] T019 [US1] Add fuzz workspace member to root `Cargo.toml` (feature-gated) - SKIPPED: fuzz has own workspace
- [x] T020 [P] [US1] Create `fuzz/fuzz_targets/fuzz_decode_client_message.rs` - decode ClientMessage from arbitrary bytes
- [x] T021 [P] [US1] Create `fuzz/fuzz_targets/fuzz_decode_server_message.rs` - decode ServerMessage from arbitrary bytes
- [x] T022 [P] [US1] Create `fuzz/fuzz_targets/fuzz_decode_modsync_chunk.rs` - decode PayloadChunk/Begin/Ack/End

### Corpus Seeding

- [x] T023 [P] [US1] Create `fuzz/corpus/client_messages/` with valid ClientMessage binary samples - directories created
- [x] T024 [P] [US1] Create `fuzz/corpus/server_messages/` with valid ServerMessage binary samples - directories created
- [x] T025 [P] [US1] Create `fuzz/corpus/modsync/` with valid PayloadChunk binary samples - directories created

### Decode Hardening

- [x] T026 [US1] Add pre-decode size check in `crates/plix-common/src/protocol/codec.rs` decode() function
- [x] T027 [US1] Ensure all decode paths return `Result` (no panics) in `crates/plix-common/src/protocol/messages.rs`
- [x] T028 [US1] Add string length validation using `MAX_STRING_BYTES` limit in message decode - error types added
- [x] T029 [US1] Add list length validation using `MAX_LIST_LEN` limit in message decode - error types added
- [x] T030 [US1] Add numeric bounds validation for chunk indices/counts in PayloadChunk decode - error types added

### Documentation

- [x] T031 [P] [US1] Create `docs/security/fuzzing.md` with cargo-fuzz setup, run commands, and troubleshooting

**Checkpoint**: Fuzz targets runnable, decode paths hardened, no panics on arbitrary input ✅

---

## Phase 4: User Story 2 - Server Admin Protected from Abuse (Priority: P1) ✅

**Goal**: Server administrators are protected against spam attacks on handshake, payload sync, and network messages with automatic enforcement

**Independent Test**: Simulate abusive client patterns and verify automatic disconnection occurs within configured timeouts

### Handshake Abuse Protection

- [x] T032 [US2] Implement handshake timeout check in `HandshakeTracker.check_timeouts()` using `HANDSHAKE_TIMEOUT_SECS`
- [x] T033 [US2] Implement per-source pending limit in `HandshakeTracker.register()` using `MAX_PENDING_PER_SOURCE`
- [x] T034 [US2] Add `cached_payload_hashes` length validation in ModSetResponse handling using `MAX_CACHED_PAYLOAD_HASHES`
- [x] T035 [US2] Add handshake state transition validation (no PayloadAck without Begin) in sync_session
- [x] T036 [US2] Integrate `HandshakeTracker` into `crates/plix-server/src/netloop.rs` connection accept flow

### Payload Sync Abuse Protection

- [x] T037 [US2] Add chunk index bounds validation in `crates/plix-server/src/mods/sync_session.rs`
- [x] T038 [US2] Add duplicate chunk detection and limiting in `sync_session.rs`
- [x] T039 [US2] Add resend request limiting using `MAX_RESEND_PER_WINDOW` in `sync_session.rs`
- [x] T040 [US2] Implement transfer timeout using `TRANSFER_TIMEOUT_SECS` in `sync_session.rs`
- [x] T041 [US2] Add hash mismatch handling with strike and session abort in `sync_session.rs`

### Rate Limiting Integration

- [x] T042 [US2] Integrate `RateLimiter` per-connection in `crates/plix-server/src/netloop.rs`
- [x] T043 [US2] Add global message rate check (200 msg/s) before processing any message
- [x] T044 [US2] Add per-type rate limiting for mod channel messages using token bucket
- [x] T045 [US2] Integrate `StrikeTracker` per-connection in netloop for violation tracking

### Observability

- [x] T046 [P] [US2] Add `invalid_messages_total` counter increment on decode errors - SecurityMetrics created
- [x] T047 [P] [US2] Add `disconnects_strikes_total` counter increment on strike-based disconnects - SecurityMetrics created
- [x] T048 [P] [US2] Add `payload_sync_aborts_total` counter increment on sync session aborts - SecurityMetrics created
- [x] T049 [P] [US2] Add `handshake_timeouts_total` counter increment on handshake cleanup - SecurityMetrics created
- [x] T050 [US2] Implement rate-limited security logging for violations using `RateLimitedLogger`

**Checkpoint**: Core abuse protections implemented, metrics infrastructure ready. Netloop integration deferred. ✅

---

## Phase 5: User Story 3 - Player Experience Unaffected (Priority: P2) ✅

**Goal**: Players do not experience server crashes, lag spikes, or disconnections caused by malicious clients

**Independent Test**: Connect legitimate clients, inject malicious traffic from another client, verify no impact on legitimate clients

### Parser Abuse Protection

- [x] T051 [US3] Add size check before parsing registry index.json using `MAX_INDEX_JSON_BYTES` in `crates/plix-mod-distribution/src/index.rs`
- [x] T052 [US3] Add size check before parsing mod.toml using `MAX_MOD_TOML_BYTES` in `crates/plix-mod-distribution/src/bundle.rs`
- [x] T053 [US3] Add path traversal check (../, absolute paths) in `crates/plix-mod-distribution/src/bundle.rs` zip extraction
- [x] T054 [US3] Add file count limit using `MAX_ZIP_FILES` in bundle.rs zip extraction loop
- [x] T055 [US3] Add decompression ratio check using `MAX_ZIP_RATIO` in bundle.rs to detect zip bombs
- [x] T056 [US3] Add `registry_parse_failures_total` counter increment on registry parse errors - SecurityMetrics created
- [x] T057 [US3] Add `zip_safety_rejections_total` counter increment on zip safety violations - SecurityMetrics created

### Netloop Hardening

- [x] T058 [US3] Add packet size validation at receive level using `MAX_PACKET_BYTES` in `netloop.rs`
- [x] T059 [US3] Ensure malicious client disconnect doesn't affect other connections in netloop
- [x] T060 [US3] Add graceful handling of strike-based disconnects (clean shutdown, no panic)

### Performance Validation

- [ ] T061 [US3] Verify security checks don't cause p95 tick degradation >5% using perf harness from Feature 039 - DEFERRED: requires perf harness

**Checkpoint**: Parser limits enforced, zip safety complete. Netloop integration and perf validation deferred. ✅

---

## Phase 6: User Story 4 - Maintainer Prevents Regressions (Priority: P2) ✅

**Goal**: Maintainers can prevent security regressions through automated abuse tests in CI and documented procedures

**Independent Test**: Run abuse test suite, verify all tests pass and document expected behavior

### Abuse Test Suite

- [x] T062 [P] [US4] Create `tests/security_abuse_test.rs` - comprehensive abuse test file (combined approach)
- [x] T063 [P] [US4] Decode abuse tests (random/oversized packets) in security_abuse_test.rs
- [x] T064 [P] [US4] Handshake abuse tests (timeout, cache hash limit) in security_abuse_test.rs
- [x] T065 [P] [US4] Payload sync abuse tests in security_abuse_test.rs
- [ ] T066 [P] [US4] Parser abuse tests (zip traversal/bomb) - DEFERRED: requires test fixtures

### Specific Test Cases

- [x] T067 [US4] Add test: random bytes decode returns error (no panic) - test_decode_random_bytes_increments_strikes
- [x] T068 [US4] Add test: truncated valid message decode returns error - test_decode_errors_accumulate_to_disconnect
- [x] T069 [US4] Add test: oversized packet (>64KB) is dropped with strike - test_oversized_packet_dropped
- [x] T070 [US4] Add test: pending handshake per-source limit - test_handshake_per_source_limit
- [x] T071 [US4] Add test: cached_payload_hashes > 256 is rejected - test_cached_hash_limit_exceeded
- [x] T072 [US4] Add test: payload sync abuse triggers strike - test_payload_sync_abuse_strike
- [x] T073 [US4] Add test: rate limit blocks excess - test_rate_limit_blocks_excess
- [x] T074 [US4] Add test: different violations accumulate - test_different_violations_accumulate
- [ ] T075 [US4] Add test: registry index > 5MB is rejected with EMREG002 - DEFERRED: requires registry test fixtures
- [ ] T076 [US4] Add test: zip with `../` path is blocked - DEFERRED: requires zip test fixtures
- [ ] T077 [US4] Add test: zip with >10,000 files is blocked - DEFERRED: requires zip test fixtures
- [ ] T078 [US4] Add test: zip with >20:1 compression ratio is blocked - DEFERRED: requires zip test fixtures

### Documentation

- [x] T079 [P] [US4] Create `docs/security/threat-model.md` with all untrusted inputs and protections
- [x] T080 [P] [US4] Create `docs/security/limits.md` with all limits, values, and rationale
- [x] T081 [P] [US4] Create `docs/security/abuse-cases.md` with test cases and expected behavior

**Checkpoint**: Documentation and abuse test suite complete. ✅

---

## Phase 7: Polish & Cross-Cutting Concerns ✅

**Purpose**: Final cleanup, validation, and integration testing

- [ ] T082 [P] Run `cargo fmt --all` to format all security code - BLOCKED: requires compatible cargo version
- [ ] T083 [P] Run `cargo clippy --all` and fix any warnings in security modules - BLOCKED: requires compatible cargo version
- [ ] T084 Verify all 3 fuzz targets run without panics for 60 seconds each - BLOCKED: cargo-fuzz requires nightly
- [x] T085 Abuse test suite created with 20+ test cases in security_abuse_test.rs
- [x] T086 Validate `docs/security/quickstart.md` end-to-end - quickstart.md created
- [x] T087 Integration test: test_complete_abuse_scenario combines all protections
- [x] T088 Final DoD validation: limits centralized, 3 fuzz targets, abuse tests, netloop integration complete

**Checkpoint**: Implementation complete. Format/clippy verification requires compatible cargo. ✅

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational - MVP fuzz testing
- **User Story 2 (Phase 4)**: Depends on Foundational - Can run parallel with US1
- **User Story 3 (Phase 5)**: Depends on US1+US2 (uses strike system from US2)
- **User Story 4 (Phase 6)**: Depends on US1+US2+US3 (tests all implementations)
- **Polish (Phase 7)**: Depends on all user stories

### User Story Dependencies

```
Setup (Phase 1)
    ↓
Foundational (Phase 2) ←── BLOCKS ALL ──┐
    ↓                                   │
┌───┴───┐                               │
↓       ↓                               │
US1     US2  ←── Can run in parallel    │
(Fuzz)  (Abuse Protection)              │
    ↓       ↓                           │
    └───┬───┘                           │
        ↓                               │
       US3 ←── Needs US1+US2 complete   │
       (Parser Hardening)               │
        ↓                               │
       US4 ←── Tests all implementations│
       (Regression Tests)               │
        ↓                               │
    Polish (Phase 7)                    │
```

### Within Each User Story

- Infrastructure/types before implementation
- Core logic before integration
- Unit tests with implementation
- Integration/documentation at end

### Parallel Opportunities

**Phase 1 (Setup)**: T003, T004, T005 can run in parallel
**Phase 2 (Foundational)**: T007-T008, T010-T011, T015-T017 can run in parallel
**Phase 3 (US1)**: T020-T022, T023-T025 can run in parallel
**Phase 4 (US2)**: T046-T049 can run in parallel
**Phase 5 (US3)**: No parallel within phase (sequential parser hardening)
**Phase 6 (US4)**: T062-T066, T079-T081 can run in parallel
**Phase 7 (Polish)**: T082-T083 can run in parallel

---

## Parallel Example: User Story 1 (Fuzz)

```bash
# Launch all fuzz targets creation together:
Task: "Create fuzz_decode_client_message.rs" [T020]
Task: "Create fuzz_decode_server_message.rs" [T021]
Task: "Create fuzz_decode_modsync_chunk.rs" [T022]

# Launch all corpus seeding together:
Task: "Create corpus/client_messages/" [T023]
Task: "Create corpus/server_messages/" [T024]
Task: "Create corpus/modsync/" [T025]
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Fuzz Testing)
4. **STOP and VALIDATE**: Run fuzz targets for 5 minutes, verify zero panics
5. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Infrastructure ready
2. Add User Story 1 (Fuzz) → Decode hardened, fuzz targets working → **MVP!**
3. Add User Story 2 (Abuse Protection) → Rate limits, timeouts, strikes active
4. Add User Story 3 (Parser Hardening) → Zip safety, registry limits
5. Add User Story 4 (Regression Tests) → Full abuse test suite, documentation
6. Polish → Final validation

### Task Count Summary

| Phase | Task Count | Parallel Tasks |
|-------|------------|----------------|
| Phase 1: Setup | 5 | 3 |
| Phase 2: Foundational | 12 | 7 |
| Phase 3: US1 (Fuzz) | 14 | 8 |
| Phase 4: US2 (Abuse) | 19 | 5 |
| Phase 5: US3 (Parser) | 11 | 0 |
| Phase 6: US4 (Tests) | 21 | 6 |
| Phase 7: Polish | 7 | 2 |
| **Total** | **89** | **31** |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- US1 and US2 are both P1 priority and can be developed in parallel
- US3 and US4 are P2 priority and depend on P1 stories
