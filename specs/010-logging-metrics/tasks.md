# Tasks: Logging & Metrics

**Input**: Design documents from `/specs/010-logging-metrics/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Included per constitution ("All network and simulation logic MUST have automated tests")

**Scope**: Server tick metrics, per-session network metrics, client debug overlay, structured logging.

**Constraints**: No per-tick logging, no allocations in hot path, headless mode compatible, load tests must keep working.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Path Conventions

- **Workspace crates**: `crates/plix-common/src/`, `crates/plix-server/src/`, `crates/plix-client/src/`, `crates/plix-net/src/`
- **Tests**: `crates/plix-server/tests/`, `crates/plix-common/tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create reusable metrics utilities in plix-common

- [x] T001 [P] Create RollingWindow<T> struct in crates/plix-common/src/metrics.rs
  - Fixed-size ring buffer with pre-allocated Vec
  - Store (Instant, T) pairs for time-based expiry
  - Capacity: 600 samples (10s at 60Hz)
- [x] T002 [P] Implement RollingWindow methods: new(), push(), samples_in_window(), len(), is_empty() in crates/plix-common/src/metrics.rs
- [x] T003 [P] Create Stats struct (avg, p95, max, min, count) in crates/plix-common/src/metrics.rs
- [x] T004 Implement stats() and stats_ms() methods on RollingWindow in crates/plix-common/src/metrics.rs
- [x] T005 Export metrics module in crates/plix-common/src/lib.rs
- [x] T006 Verify workspace compiles with `cargo build --workspace`

---

## Phase 2: Foundational (RTT Protocol Plumbing)

**Purpose**: Extend protocol for RTT measurement - MUST be complete before user stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T007 Add `rtt_nonce: u64` field to PlayerInput in crates/plix-common/src/protocol/messages.rs
- [x] T008 Update PlayerInput::empty() to set rtt_nonce = 0 in crates/plix-common/src/protocol/messages.rs
- [x] T009 Add `rtt_nonce_echo: u64` field to WorldSnapshot in crates/plix-common/src/protocol/messages.rs
- [x] T010 Update server to store latest rtt_nonce per client in crates/plix-server/src/session.rs
- [x] T011 Echo rtt_nonce in WorldSnapshot construction in crates/plix-server/src/lib.rs (snapshot.rs updated too)
- [x] T012 Verify workspace compiles and existing tests pass with `cargo test --workspace`

**Checkpoint**: Protocol ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Server Tick-Time Metrics (Priority: P1) 🎯 MVP

**Goal**: Monitor tick processing time to detect performance degradation

**Independent Test**: Start server, run load test with 8 clients, verify tick metrics logged every 5 seconds showing avg/p95/max over 10s window.

### Tests for User Story 1

- [x] T013 [P] [US1] Unit test: RollingWindow push and stats computation in crates/plix-common/src/metrics.rs (inline tests)
- [x] T014 [P] [US1] Unit test: P95 calculation correctness with known values in crates/plix-common/src/metrics.rs (inline tests)
- [x] T015 [P] [US1] Unit test: Time-based window expiry in crates/plix-common/src/metrics.rs (inline tests)

### Implementation for User Story 1

- [x] T016 [P] [US1] Create ServerMetricsCollector struct in crates/plix-server/src/metrics.rs
  - tick_times: RollingWindow<Duration>
  - last_log: Instant
  - log_interval: Duration (default 5s)
- [x] T017 [US1] Implement record_tick(&mut self, duration: Duration) in crates/plix-server/src/metrics.rs
- [x] T018 [US1] Implement maybe_log(&mut self, players: usize, sessions: usize) in crates/plix-server/src/metrics.rs
- [x] T019 [US1] Integrate ServerMetricsCollector into server main loop in crates/plix-server/src/lib.rs
- [x] T020 [US1] Call record_tick() with tick duration from TickLoop::tick() in crates/plix-server/src/lib.rs
- [x] T021 [US1] Call maybe_log() after each tick with player/session counts in crates/plix-server/src/lib.rs
- [x] T022 [US1] Verify no per-tick logging occurs (only every 5s) in crates/plix-server/src/metrics.rs

**Checkpoint**: Tick metrics working - server logs performance summary every 5 seconds

---

## Phase 4: User Story 2 - Per-Connection Network Metrics (Priority: P1)

**Goal**: Monitor per-connection network quality (RTT, jitter, packet loss)

**Independent Test**: Connect 2 clients, verify each connection has its own RTT, jitter, and loss_pct metrics.

### Tests for User Story 2

- [x] T023 [P] [US2] Unit test: SessionNetMetrics RTT recording in crates/plix-server/src/session.rs (inline tests)
- [x] T024 [P] [US2] Unit test: Jitter calculation (std dev of RTT) in crates/plix-server/src/session.rs (inline tests)
- [x] T025 [P] [US2] Unit test: Packet loss detection from sequence gaps in crates/plix-server/src/session.rs (inline tests)

### Implementation for User Story 2

- [x] T026 [P] [US2] Create SessionNetMetrics struct in crates/plix-server/src/session.rs
  - rtt_window: RollingWindow<Duration>
  - last_seq, expected_seq for loss detection
  - packets_received, packets_lost counts
- [x] T027 [US2] Implement record_rtt(), record_packet(), jitter(), loss_pct() methods in crates/plix-server/src/session.rs
- [x] T028 [US2] Add SessionNetMetrics field to ServerPlayer in crates/plix-server/src/session.rs
- [x] T029 [US2] Initialize SessionNetMetrics in ServerPlayer::new() in crates/plix-server/src/session.rs
- [x] T030 [US2] Update metrics on packet receive (extract seq) in crates/plix-server/src/lib.rs
- [x] T031 [US2] Include per-session metrics in periodic log output in crates/plix-server/src/metrics.rs

**Checkpoint**: Per-connection metrics working - each player has RTT/jitter/loss tracked

---

## Phase 5: User Story 3 - Server Aggregate Metrics (Priority: P2)

**Goal**: Aggregate metrics (PPS, player count, session count) for server load monitoring

**Independent Test**: Run server with multiple clients, verify aggregate PPS_in, PPS_out, players_connected, sessions_active are logged.

### Implementation for User Story 3

- [x] T032 [P] [US3] Add pps_in, pps_out counters to ServerMetricsCollector in crates/plix-server/src/metrics.rs
- [x] T033 [US3] Implement record_packet_in(), record_packet_out() methods in crates/plix-server/src/metrics.rs
- [x] T034 [US3] Track bytes_in, bytes_out for bandwidth metrics in crates/plix-server/src/metrics.rs
- [x] T035 [US3] Call packet tracking on each recv/send in crates/plix-server/src/lib.rs
- [x] T036 [US3] Include PPS and bandwidth in periodic log output in crates/plix-server/src/metrics.rs

**Checkpoint**: Aggregate metrics working - server-wide load visible in logs

---

## Phase 6: User Story 4 - Client Network Debug Overlay (Priority: P2)

**Goal**: F3-toggled debug overlay showing network stats at 2Hz refresh

**Independent Test**: Connect client, press F3, verify overlay shows RTT, jitter, loss%, FPS. Press F3 again to hide.

### Implementation for User Story 4

- [x] T037 [P] [US4] Add F3 key variant to Key enum in crates/plix-client/src/config.rs
- [x] T038 [P] [US4] Add ToggleDebugOverlay action to Action enum in crates/plix-client/src/config.rs
- [x] T039 [US4] Add F3 → ToggleDebugOverlay to default keybinds in crates/plix-client/src/config.rs
- [x] T040 [US4] Add fps and player_id fields to NetDebugData in crates/plix-client/src/ui/net_debug.rs
- [x] T041 [US4] Add last_update and cached_lines to NetDebugOverlay in crates/plix-client/src/ui/net_debug.rs
- [x] T042 [US4] Implement should_update() for 2Hz refresh rate in crates/plix-client/src/ui/net_debug.rs
- [x] T043 [US4] Implement build_cached_lines() to format overlay text in crates/plix-client/src/ui/net_debug.rs
- [x] T044 [US4] Implement render() to draw overlay text in crates/plix-client/src/ui/net_debug.rs
- [x] T045 [US4] Add RTT nonce generation in client input in crates/plix-client/src/main.rs
- [x] T046 [US4] Store pending nonces with timestamps in client in crates/plix-client/src/main.rs
- [x] T047 [US4] Compute RTT on snapshot echo, update metrics in crates/plix-client/src/main.rs
- [x] T048 [US4] Handle F3 keypress in main event loop in crates/plix-client/src/main.rs
- [x] T049 [US4] Call overlay.toggle() on F3 press in crates/plix-client/src/main.rs
- [x] T050 [US4] Call overlay.render() in render loop when visible in crates/plix-client/src/main.rs

**Checkpoint**: Client overlay working - F3 toggles debug display at 2Hz

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Validation, cleanup, and non-regression testing

### Automated Validation

- [x] T051 Run cargo test --workspace and verify all tests pass
- [x] T052 Run cargo clippy --all-targets and address warnings
- [x] T053 Run cargo fmt --all -- --check and fix formatting

### Manual Validation

- [ ] T054 Manual test: Start server, verify tick metrics logged every 5s
- [ ] T055 Manual test: Connect 2 clients, verify per-connection metrics
- [ ] T056 Manual test: Press F3, verify overlay appears with correct data
- [ ] T057 Manual test: Press F3 again, verify overlay hides
- [ ] T058 Manual test: Verify overlay updates at ~2Hz
- [ ] T059 Manual test: Run load test, verify no performance regression

### Non-Regression

- [ ] T060 Verify headless server compiles and runs without client code
- [ ] T061 Run quickstart.md verification checklist

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - US1 (Tick Metrics) and US2 (Network Metrics) are P1 - implement first
  - US3 (Aggregate) and US4 (Overlay) are P2 - can proceed in parallel after P1
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1 Tick Metrics)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P1 Network Metrics)**: Can start after Foundational - Uses RollingWindow from US1 but file is separate
- **User Story 3 (P2 Aggregate)**: Can start after Foundational - Extends ServerMetricsCollector from US1
- **User Story 4 (P2 Overlay)**: Can start after Foundational - Uses client RTT computed from protocol changes

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Data model changes before service logic
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- T001, T002, T003 can run in parallel (different aspects of same file)
- T013, T014, T015 can run in parallel (different test functions)
- T023, T024, T025 can run in parallel (different test functions)
- T037, T038 can run in parallel (different enums in same file)

---

## Parallel Example: Phase 1 Setup

```bash
# Launch all setup tasks together:
Task: "Create RollingWindow<T> struct in crates/plix-common/src/metrics.rs"
Task: "Create Stats struct in crates/plix-common/src/metrics.rs"
```

## Parallel Example: User Story 1 Tests

```bash
# Launch all US1 tests together:
Task: "Unit test: RollingWindow push and stats computation"
Task: "Unit test: P95 calculation correctness"
Task: "Unit test: Time-based window expiry"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup (RollingWindow, Stats)
2. Complete Phase 2: Foundational (Protocol changes)
3. Complete Phase 3: User Story 1 (Tick Metrics)
4. Complete Phase 4: User Story 2 (Network Metrics)
5. **STOP and VALIDATE**: Test tick and network metrics independently
6. Deploy/demo if ready - server observability achieved

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. US1 + US2 → Test → Deploy (MVP - server metrics)
3. US3 → Test → Deploy (aggregate metrics)
4. US4 → Test → Deploy (client overlay)
5. Each increment adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Tick Metrics)
   - Developer B: User Story 2 (Network Metrics)
3. After P1 complete:
   - Developer A: User Story 3 (Aggregate)
   - Developer B: User Story 4 (Overlay)

---

## Definition of Done

- [ ] Server logs tick metrics (avg/p95/max) every 5 seconds
- [ ] Per-connection RTT, jitter, loss_pct tracked and logged
- [ ] Aggregate PPS and player/session counts logged
- [ ] F3 toggles client debug overlay
- [ ] Overlay shows RTT, jitter, loss%, FPS at 2Hz
- [ ] No per-tick logging, no allocations in hot path
- [ ] Headless server works without client code
- [ ] All tests pass; manual checks OK
- [ ] cargo clippy and cargo fmt clean

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- All metrics use 10-second rolling window per clarification
- RTT uses echo nonce on existing input messages per clarification
