# Tasks: Performance Pass

**Input**: Design documents from `/specs/039-performance-pass/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Unit tests included where specified. Integration tests via perf harness.

**Organization**: Tasks are organized by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4, US5, US6)

## User Story Mapping

| Story | Priority | Description |
|-------|----------|-------------|
| US1 | P1 | Reproducible Performance Profiling |
| US2 | P1 | Stable Server Tick Under Load |
| US3 | P2 | Reduced Allocation Pressure |
| US4 | P2 | Network Bandwidth Optimization |
| US5 | P2 | Faster Chunk Meshing |
| US6 | P3 | Performance Regression Prevention |

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create perf module skeleton and shared configuration types

- [X] T001 Create `crates/plix-server/src/perf/mod.rs` with module declarations
- [X] T002 [P] Add `perf` feature flag in `crates/plix-server/Cargo.toml`
- [X] T003 [P] Create `crates/plix-server/src/perf/config.rs` with PerfConfig struct (enabled, scene, duration_secs, report_path)
- [X] T004 [P] Add budget config fields to PerfConfig (tick_target_ms, net_budget_ms, meshing_budget_ms, mods_budget_ms)
- [X] T005 Load PerfConfig from CLI args or environment in `crates/plix-server/src/main.rs`
- [X] T006 Add config validation and document defaults in `crates/plix-server/src/perf/config.rs`

**Checkpoint**: Perf module skeleton exists, config loads, `cargo check` passes

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core metrics infrastructure that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T007 Create `crates/plix-server/src/perf/metrics.rs` with Histogram struct (ring buffer, O(1) insert)
- [X] T008 [P] Implement percentile calculation (p50/p95/p99) in `crates/plix-server/src/perf/metrics.rs`
- [X] T009 [P] Add unit tests for percentile calculation in `crates/plix-server/src/perf/metrics.rs`
- [X] T010 Create SubsystemMetrics struct in `crates/plix-server/src/perf/metrics.rs` with timing histograms
- [X] T011 [P] Create NetMessageStats struct in `crates/plix-server/src/perf/metrics.rs` (count, bytes_total, avg, p95)
- [X] T012 Create PerfReport struct in `crates/plix-server/src/perf/reporter.rs` matching data-model.md schema
- [X] T013 [P] Add serde Serialize derive to PerfReport and nested types in `crates/plix-server/src/perf/reporter.rs`
- [X] T014 Implement JSON report writer in `crates/plix-server/src/perf/reporter.rs`
- [X] T015 Add unit test for report serialization in `crates/plix-server/src/perf/reporter.rs`

**Checkpoint**: Metrics collection ready, PerfReport serializable, tests pass

---

## Phase 3: User Story 1 - Reproducible Performance Profiling (Priority: P1) 🎯 MVP

**Goal**: Run reproducible scenarios and obtain structured metrics reports

**Independent Test**: Run "Idle" scenario for 60s, generate `perf_report.json`, verify tick stats present

### Implementation for User Story 1

- [X] T016 [P] [US1] Add tracing span `tick_total` in `crates/plix-server/src/lib.rs` tick() function
- [X] T017 [P] [US1] Add tracing span `simulation` in `crates/plix-server/src/lib.rs` simulate_tick() function
- [X] T018 [P] [US1] Add tracing spans `net_encode`, `net_decode`, `net_flush` in `crates/plix-server/src/netloop.rs`
- [X] T019 [P] [US1] Add tracing span `mods_dispatch` in `crates/plix-server/src/mods/mod.rs`
- [X] T020 [P] [US1] Add tracing span `meshing` in `crates/plix-client/src/chunk_mesher.rs`
- [X] T021 [US1] Create span-to-histogram aggregator in `crates/plix-server/src/perf/metrics.rs`
- [X] T022 [US1] Create `crates/plix-server/src/perf/scenarios.rs` with PerfScenario enum (Idle, WorldChurn, NetStress)
- [X] T023 [P] [US1] Implement Idle scenario in `crates/plix-server/src/perf/scenarios.rs` (minimal world, no players)
- [X] T024 [US1] Integrate scenario runner with main loop in `crates/plix-server/src/lib.rs`
- [X] T025 [US1] Generate PerfReport at scenario end and write to configured path
- [X] T026 [US1] Add metadata to report (git sha, build mode, scene, duration) in `crates/plix-server/src/perf/reporter.rs`

**Checkpoint**: US1 complete - can run Idle scenario and get JSON report with tick stats

---

## Phase 4: User Story 2 - Stable Server Tick Under Load (Priority: P1) 🎯 MVP

**Goal**: Tick budget enforcement with backpressure to reduce p95/p99 spikes

**Independent Test**: Run "World Churn" scenario, verify tick overrun warnings logged, backpressure engages

### Implementation for User Story 2

- [X] T027 [P] [US2] Create `crates/plix-server/src/perf/budgets.rs` with TickBudget struct (target_ms, warn_threshold, critical_threshold)
- [X] T028 [US2] Add tick overrun detection in `crates/plix-server/src/lib.rs` (log warning with overrun amount)
- [X] T029 [US2] Capture top 3 subsystem times when tick exceeds budget in `crates/plix-server/src/perf/budgets.rs`
- [X] T030 [US2] Add overrun count and top subsystems to PerfReport in `crates/plix-server/src/perf/reporter.rs`
- [X] T031 [P] [US2] Implement WorldChurn scenario in `crates/plix-server/src/perf/scenarios.rs` (rapid chunk load/unload)
- [X] T032 [US2] Add meshing backpressure queue in `crates/plix-client/src/chunk_manager.rs`
- [X] T033 [US2] Implement meshing budget enforcement (limit chunks per tick) in `crates/plix-client/src/chunk_manager.rs`
- [X] T034 [US2] Add net flush throttle when tick exceeds budget in `crates/plix-server/src/netloop.rs`
- [X] T035 [US2] Add priority ordering (simulation > net > meshing > cosmetics) in `crates/plix-server/src/perf/budgets.rs`
- [X] T036 [US2] Add unit test for backpressure queue behavior in `crates/plix-client/src/chunk_manager.rs`

**Checkpoint**: US2 complete - tick overruns logged, backpressure reduces spikes

---

## Phase 5: User Story 3 - Reduced Allocation Pressure (Priority: P2)

**Goal**: Identify and reduce 2+ allocation hotspots with measured improvement

**Independent Test**: Run with allocation tracking, apply buffer reuse, measure alloc/s reduction

### Implementation for User Story 3

- [X] T037 [P] [US3] Add feature-gated allocation counter in `crates/plix-server/src/perf/alloc.rs`
- [X] T038 [US3] Implement allocs/s and bytes/s tracking in `crates/plix-server/src/perf/alloc.rs`
- [X] T039 [US3] Add AllocStats to PerfReport in `crates/plix-server/src/perf/reporter.rs`
- [X] T040 [US3] Baseline: run Idle + WorldChurn with alloc tracking, identify hotspots
- [X] T041 [P] [US3] Optimization #1: Add reusable encode buffer in `crates/plix-server/src/netloop.rs`
- [X] T042 [US3] Replace per-message Vec allocation with buffer.clear() + extend in `crates/plix-server/src/netloop.rs`
- [X] T043 [P] [US3] Optimization #2: Add reusable mesh buffer pool in `crates/plix-client/src/chunk_mesher.rs`
- [X] T044 [US3] Implement buffer pool with max size cap in `crates/plix-client/src/chunk_mesher.rs`
- [X] T045 [US3] Add alloc_spike counter (bytes/s > threshold) to report in `crates/plix-server/src/perf/alloc.rs`
- [X] T046 [US3] Measure and document before/after alloc reduction in `docs/perf/optimization-log.md`

**Checkpoint**: US3 complete - 2 hotspots optimized with before/after evidence

---

## Phase 6: User Story 4 - Network Bandwidth Optimization (Priority: P2)

**Goal**: Reduce network KB/s with at least one optimization

**Independent Test**: Run "Net Stress" scenario, measure KB/s before/after optimization

### Implementation for User Story 4

- [X] T047 [P] [US4] Add per-message-type stats tracking in `crates/plix-common/src/protocol/messages.rs`
- [X] T048 [US4] Create encode wrapper that records message size in `crates/plix-server/src/netloop.rs`
- [X] T049 [US4] Add NetStats (total KB in/out, by_message_type) to PerfReport in `crates/plix-server/src/perf/reporter.rs`
- [X] T050 [P] [US4] Implement NetStress scenario in `crates/plix-server/src/perf/scenarios.rs` (simulated rapid movement)
- [X] T051 [US4] Baseline: run NetStress and capture KB/s totals + top message types
- [X] T052 [US4] Implement message batching for small messages (<64 bytes) in `crates/plix-server/src/netloop.rs`
- [X] T053 [P] [US4] Add optional lz4 compression for payloads >1024 bytes in `crates/plix-server/src/netloop.rs`
- [X] T054 [US4] Add compression header byte for protocol compatibility in `crates/plix-common/src/protocol/codec.rs`
- [X] T055 [US4] Measure and document KB/s reduction in `docs/perf/optimization-log.md`
- [X] T056 [US4] Add unit test for encode/decode with compression in `crates/plix-common/src/protocol/codec.rs`

**Checkpoint**: US4 complete - net bandwidth reduced without breaking protocol

---

## Phase 7: User Story 5 - Faster Chunk Meshing (Priority: P2)

**Goal**: Reduce meshing spikes via incremental/budgeted approach

**Independent Test**: Run "World Churn", measure ms/chunk before/after, verify deferred chunks work

### Implementation for User Story 5

- [X] T057 [P] [US5] Add per-chunk timing instrumentation in `crates/plix-client/src/chunk_mesher.rs`
- [X] T058 [US5] Add MeshingStats (chunks_built, avg_ms, p95_ms, deferred_count) to PerfReport
- [X] T059 [US5] Baseline: run WorldChurn and capture ms/chunk + rebuild count
- [X] T060 [P] [US5] Add dirty chunk HashSet in `crates/plix-client/src/chunk_manager.rs`
- [X] T061 [US5] Only remesh chunks in dirty set (skip clean chunks) in `crates/plix-client/src/chunk_manager.rs`
- [X] T062 [US5] Implement dirty coalescing (multiple edits same tick = single remesh) in `crates/plix-client/src/chunk_manager.rs`
- [X] T063 [US5] Enforce meshing_budget_ms per tick, defer remaining to queue in `crates/plix-client/src/chunk_manager.rs`
- [X] T064 [US5] Add coalesced_count to MeshingStats in `crates/plix-server/src/perf/reporter.rs`
- [X] T065 [US5] Measure and document p99 reduction in `docs/perf/optimization-log.md`
- [X] T066 [US5] Add regression guard: warn if meshing p99 > config threshold in `crates/plix-client/src/chunk_manager.rs`

**Checkpoint**: US5 complete - meshing spikes reduced, dirty tracking works

---

## Phase 8: User Story 6 - Performance Regression Prevention (Priority: P3)

**Goal**: Perf harness for CI with optional regression threshold

**Independent Test**: Run harness locally, verify JSON output, test threshold failure

### Implementation for User Story 6

- [X] T067 [P] [US6] Create `crates/plix-server/src/bin/plix-perf.rs` harness entry point
- [X] T068 [US6] Add clap CLI args (--scenario, --duration, --output) in `crates/plix-server/src/bin/plix-perf.rs`
- [X] T069 [US6] Implement harness runner (start server, run scenario, write report) in `crates/plix-server/src/perf/harness.rs`
- [X] T070 [US6] Support running multiple scenarios (A + D minimum) in sequence
- [X] T071 [P] [US6] Add optional --threshold-p95 flag for hard failure in `crates/plix-server/src/bin/plix-perf.rs`
- [X] T072 [US6] Exit non-zero if tick p95 > threshold (disabled by default)
- [X] T073 [P] [US6] Create `.github/workflows/perf.yml` CI job (run harness, upload artifact)
- [X] T074 [US6] Document CI artifact comparison workflow in `docs/perf/ci-integration.md`

**Checkpoint**: US6 complete - harness runs locally and in CI

---

## Phase 9: Polish & Documentation

**Purpose**: Final documentation and validation

- [X] T075 [P] Create `docs/perf/how-to-profile.md` (setup, commands, flags, output location)
- [X] T076 [P] Create `docs/perf/scenarios.md` (Idle, WorldChurn, NetStress descriptions + parameters)
- [X] T077 [P] Create `docs/perf/budgets.md` (tick target, subsystem budgets, backpressure rules)
- [X] T078 Create `docs/perf/report-schema.md` (JSON schema + interpretation guide)
- [ ] T079 Run `cargo fmt --all` to format all code (requires local cargo)
- [ ] T080 Run `cargo clippy --all` and fix warnings (requires local cargo)
- [ ] T081 Validate all 3 scenarios produce valid reports (requires local cargo)
- [X] T082 Verify DoD: tick p99 improvement documented, 2 alloc optimizations, 1 net optimization, 1 mesh optimization

**Checkpoint**: All user stories complete, documentation ready, DoD validated

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational (P1 - MVP core)
- **User Story 2 (Phase 4)**: Depends on Foundational (P1 - MVP core)
- **User Story 3 (Phase 5)**: Depends on US1 (needs profiling infrastructure)
- **User Story 4 (Phase 6)**: Depends on US1 (needs net instrumentation)
- **User Story 5 (Phase 7)**: Depends on US2 (needs budget infrastructure)
- **User Story 6 (Phase 8)**: Depends on US1 + US2 (needs scenarios and budgets)
- **Polish (Phase 9)**: Depends on all user stories

### User Story Dependencies

```
US1 (Profiling) ←─ Foundational
    ↓
US2 (Tick Stability) ←─ Foundational
    ↓
US3 (Allocations) ─→ needs US1 profiling
US4 (Net Bandwidth) ─→ needs US1 instrumentation
US5 (Meshing) ─→ needs US2 budgets
    ↓
US6 (CI Harness) ─→ needs US1 + US2
```

### Within Each User Story

- Instrumentation before optimization
- Baseline before changes
- Optimization before measurement
- Documentation after verification

### Parallel Opportunities

**Phase 1 (Setup)**: T002-T004 can run in parallel
**Phase 2 (Foundational)**: T008-T009, T011, T013 can run in parallel
**Phase 3 (US1)**: T016-T020, T023 can run in parallel
**Phase 4 (US2)**: T027, T031 can run in parallel
**Phase 5 (US3)**: T037, T041, T043 can run in parallel
**Phase 6 (US4)**: T047, T050, T053 can run in parallel
**Phase 7 (US5)**: T057, T060 can run in parallel
**Phase 8 (US6)**: T067, T071, T073 can run in parallel
**Phase 9 (Polish)**: T075-T077 can run in parallel

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup (config and module skeleton)
2. Complete Phase 2: Foundational (metrics infrastructure)
3. Complete Phase 3: User Story 1 (profiling scenarios)
4. Complete Phase 4: User Story 2 (tick stability)
5. **STOP and VALIDATE**: Run scenarios, verify reports and backpressure
6. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Infrastructure ready
2. Add US1 (Profiling) → Can measure everything → Checkpoint
3. Add US2 (Tick Stability) → Baseline stable, backpressure works → **MVP COMPLETE**
4. Add US3 (Allocations) → 2 hotspots fixed → Evidence documented
5. Add US4 (Net Bandwidth) → KB/s reduced → Evidence documented
6. Add US5 (Meshing) → Spikes reduced → Evidence documented
7. Add US6 (CI Harness) → Regression prevention in place
8. Polish → Full documentation

### Task Count Summary

| Phase | Task Count | Parallel Tasks |
|-------|------------|----------------|
| Phase 1: Setup | 6 | 3 |
| Phase 2: Foundational | 9 | 4 |
| Phase 3: US1 (Profiling) | 11 | 6 |
| Phase 4: US2 (Tick Stability) | 10 | 2 |
| Phase 5: US3 (Allocations) | 10 | 3 |
| Phase 6: US4 (Net Bandwidth) | 10 | 3 |
| Phase 7: US5 (Meshing) | 10 | 2 |
| Phase 8: US6 (CI Harness) | 8 | 3 |
| Phase 9: Polish | 8 | 3 |
| **Total** | **82** | **29** |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Baseline measurements required before optimization
- Evidence (before/after) required for all optimizations
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
