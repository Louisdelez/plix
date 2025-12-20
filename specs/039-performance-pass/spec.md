# Feature Specification: Performance Pass

**Feature Branch**: `039-performance-pass`
**Created**: 2025-12-19
**Status**: Draft
**Input**: Production-grade performance optimization with profiling, allocations, net bandwidth, meshing, and tick stability

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Reproducible Performance Profiling (Priority: P1)

As a developer, I can run reproducible performance scenarios and obtain a structured metrics report, so I can identify performance bottlenecks and track improvements over time.

**Why this priority**: Without measurement infrastructure, all optimizations are guesswork. This enables evidence-based performance work and regression detection.

**Independent Test**: Run a single profiling scenario (e.g., "Idle" server), generate `perf_report.json`, and verify it contains tick time stats, allocation counts, and network bandwidth metrics.

**Acceptance Scenarios**:

1. **Given** an idle server with no connected clients, **When** I run the "Idle" profiling scenario for 60 seconds, **Then** the system produces a JSON report with avg/p50/p95/p99 tick times
2. **Given** a profiling scenario running, **When** I complete the scenario, **Then** the report includes allocation stats (alloc/s, bytes/s) and network bandwidth (KB/s in/out)
3. **Given** the same scenario configuration, **When** I run the scenario twice, **Then** the results are comparable within acceptable variance (< 10% deviation on p50)
4. **Given** a generated perf report, **When** I inspect it, **Then** I can identify which subsystem consumed the most tick time

---

### User Story 2 - Stable Server Tick Under Load (Priority: P1)

As a server administrator, I benefit from stable server tick times (reduced p95/p99 spikes) even under heavy load, ensuring a smooth experience for all connected players.

**Why this priority**: Tick stability directly impacts player experience. Spikes cause visible lag, desync, and frustration. This is core to game quality.

**Independent Test**: Run the "World Churn" scenario (chunk generation + meshing), measure tick time p99, and verify it stays within budget.

**Acceptance Scenarios**:

1. **Given** a server running with defined tick budget, **When** tick time exceeds budget, **Then** the system logs a warning with the overrun amount and responsible subsystem
2. **Given** subsystems instrumented for tick time, **When** profiling completes, **Then** I can see breakdown by: simulation, networking, mods dispatch, meshing/streaming
3. **Given** a heavy load scenario ("World Churn"), **When** I run before and after optimizations, **Then** p99 tick time improves by a measurable amount
4. **Given** tick budget is exceeded, **When** backpressure engages, **Then** non-critical systems (meshing, cosmetics) reduce frequency while simulation and essential networking maintain priority

---

### User Story 3 - Reduced Allocation Pressure (Priority: P2)

As a developer, I can identify and reduce the highest-impact allocation hotspots, resulting in fewer GC/allocator pauses and smoother tick times.

**Why this priority**: Allocation pressure causes unpredictable pauses. Addressing this after instrumentation ensures we target real hotspots, not guessed ones.

**Independent Test**: Run profiling, identify top allocation sites, apply buffer reuse optimization, and verify reduced alloc/s.

**Acceptance Scenarios**:

1. **Given** profiling with allocation tracking enabled, **When** the scenario completes, **Then** the report lists allocation hotspots (bytes/s per subsystem)
2. **Given** an identified hotspot (e.g., network encoding), **When** I apply buffer reuse, **Then** allocation count for that subsystem decreases measurably
3. **Given** profiling data, **When** I compare before/after optimization, **Then** I have concrete evidence of improvement (specific numbers)
4. **Given** a high-churn scenario, **When** optimizations are applied, **Then** at least 2 hotspots show reduced allocation pressure

---

### User Story 4 - Network Bandwidth Optimization (Priority: P2)

As a player, I experience less bandwidth usage and smoother gameplay because the server sends optimized network updates.

**Why this priority**: Bandwidth affects both server costs and player experience (especially on constrained connections). Builds on instrumentation from P1.

**Independent Test**: Run "Net Stress" scenario, measure KB/s, apply compression/batching, and verify reduced bandwidth without protocol breakage.

**Acceptance Scenarios**:

1. **Given** network instrumentation active, **When** profiling completes, **Then** the report shows per-message-type stats: avg size, p95 size, frequency/s, KB/s
2. **Given** large payloads (> threshold), **When** compression is enabled, **Then** payload size decreases without breaking protocol compatibility
3. **Given** the "Net Stress" scenario, **When** I compare before/after bandwidth optimization, **Then** total KB/s decreases while gameplay remains smooth
4. **Given** multiple small messages, **When** batching is applied, **Then** overall message count decreases and overhead is reduced

---

### User Story 5 - Faster Chunk Meshing (Priority: P2)

As a player, I experience fewer visual stutters during chunk loading and world modifications because meshing is faster and more predictable.

**Why this priority**: Meshing spikes cause visible hitches. Optimization here directly improves perceived smoothness during exploration and building.

**Independent Test**: Run "World Churn" scenario, measure meshing time per chunk, apply incremental/budgeted meshing, and verify reduced p99.

**Acceptance Scenarios**:

1. **Given** meshing instrumentation active, **When** chunks are meshed, **Then** per-chunk meshing time is recorded
2. **Given** multiple block changes in one tick, **When** meshing runs, **Then** only dirty chunks are re-meshed (incremental rebuild)
3. **Given** meshing budget per tick (e.g., 2-4ms), **When** budget is exceeded, **Then** remaining chunks defer to next tick
4. **Given** the "World Churn" scenario, **When** optimizations are applied, **Then** meshing p99 time decreases measurably

---

### User Story 6 - Performance Regression Prevention (Priority: P3)

As a developer, I can run performance benchmarks in CI to detect regressions before they reach production.

**Why this priority**: Ensures performance gains persist. Lower priority because it requires the optimization infrastructure first.

**Independent Test**: Run perf harness locally, compare to baseline, and verify alerts on significant regression.

**Acceptance Scenarios**:

1. **Given** a perf harness binary, **When** I run it locally, **Then** it executes at least "Idle" and "World Churn" scenarios and outputs JSON
2. **Given** a baseline performance report, **When** current results exceed regression threshold (e.g., >15% worse on p95), **Then** the comparison flags a warning
3. **Given** CI pipeline, **When** perf harness runs, **Then** artifacts are produced for comparison (automated comparison can be added later)

---

### Edge Cases

- What happens when profiling is enabled in production? System must have minimal overhead when profiling is disabled, and profiling tools must be feature-gated.
- How does the system handle extreme load (e.g., 100+ chunk generations in one tick)? Backpressure must engage, deferring non-critical work without dropping essential updates.
- What if compression increases latency? Compression must be optional and only apply to payloads above threshold; latency-sensitive messages bypass compression.
- How does meshing budget interact with player proximity? Chunks near players get priority; distant chunks can be deferred longer.
- What happens when all subsystems exceed budget simultaneously? Clear priority order must be defined: simulation > essential networking > streaming > cosmetics.

## Requirements *(mandatory)*

### Functional Requirements

**Profiling & Instrumentation**

- **FR-001**: System MUST provide reproducible performance scenarios: "Idle" (empty server), "World Churn" (chunk generation + meshing), and "Net Stress" (rapid movement + updates)
- **FR-002**: System MUST generate a machine-readable report (`perf_report.json`) containing tick time stats (avg/p50/p95/p99), allocation stats (alloc/s, bytes/s), and network bandwidth (KB/s in/out, per-message-type breakdown)
- **FR-003**: System MUST instrument tick time by subsystem: simulation, networking encode/decode, mods dispatch, meshing/streaming
- **FR-004**: System MUST feature-gate heavy profiling tools (flame graphs, allocation tracking) to avoid production impact

**Tick Stability**

- **FR-005**: System MUST define a tick budget target (based on current TPS: 16.6ms for 60 TPS or 33.3ms for 30 TPS)
- **FR-006**: System MUST log warnings when tick time exceeds budget, including overrun duration and responsible subsystem
- **FR-007**: System MUST implement backpressure: when budget exceeded, reduce frequency of non-critical systems (meshing async, cosmetics) while maintaining simulation and essential networking
- **FR-008**: System MUST define and enforce priority order: simulation > essential networking > streaming > cosmetics

**Allocations**

- **FR-009**: System MUST track allocation rates (alloc/s, bytes/s) per instrumented subsystem
- **FR-010**: System MUST support buffer reuse patterns for high-frequency allocations (network encoding, message construction)
- **FR-011**: System MUST document at least 2 allocation hotspot reductions with before/after measurements

**Network Bandwidth**

- **FR-012**: System MUST instrument network messages: per-type size (avg, p95), frequency, total KB/s in/out
- **FR-013**: System MUST support optional compression for payloads exceeding threshold (default: 1024 bytes)
- **FR-014**: System MUST support message batching for small frequent messages
- **FR-015**: Network optimizations MUST NOT break protocol compatibility (or require explicit version bump)

**Meshing**

- **FR-016**: System MUST measure per-chunk meshing time
- **FR-017**: System MUST implement incremental meshing: only re-mesh dirty chunks
- **FR-018**: System MUST implement meshing budget per tick (configurable, default: 2-4ms)
- **FR-019**: System MUST coalesce multiple updates to the same chunk within a tick to avoid redundant remeshing

**Anti-Regression**

- **FR-020**: System MUST provide a perf harness that executes at least "Idle" and "World Churn" scenarios
- **FR-021**: Perf harness MUST output stable JSON format for comparison
- **FR-022**: System MUST document how to run perf harness locally and interpret results

**Documentation**

- **FR-023**: System MUST provide `docs/perf/how-to-profile.md` with step-by-step instructions
- **FR-024**: System MUST provide `docs/perf/scenarios.md` describing each scenario and how to run it
- **FR-025**: System MUST provide `docs/perf/budgets.md` explaining tick budget, thresholds, and design decisions

### Key Entities

- **PerfReport**: The structured output of a profiling run, containing tick stats, allocation stats, network stats, and scenario metadata
- **PerfScenario**: A reproducible test configuration (name, duration, parameters) that exercises specific subsystems
- **TickBudget**: Configuration defining target tick time and priority order for subsystem scheduling
- **SubsystemMetrics**: Per-subsystem timing and allocation data (simulation, networking, meshing, mods)
- **NetMessageStats**: Per-message-type statistics (size distribution, frequency, bandwidth contribution)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: At least 3 reproducible profiling scenarios can be executed with documented commands
- **SC-002**: `perf_report.json` is generated containing tick stats, allocation stats, and network stats
- **SC-003**: Tick time p99 improves by at least 20% on one heavy-load scenario after optimizations
- **SC-004**: At least 2 allocation hotspots show measurable reduction (with before/after data)
- **SC-005**: Network KB/s decreases on "Net Stress" scenario without breaking protocol compatibility
- **SC-006**: Meshing time is measurable per chunk, and at least one optimization (incremental or budget) is implemented
- **SC-007**: Perf harness can be run locally and produces consistent output
- **SC-008**: All three documentation files exist and are usable by developers

## Assumptions

- Current TPS target is 30 or 60; the exact value will be determined from existing codebase configuration
- Compression library choice will be made during implementation; common options include lz4 or zstd
- Allocation tracking tools will be chosen based on platform support (jemalloc stats, dhat, or heaptrack)
- The existing tracing infrastructure can be extended for subsystem instrumentation
- "Sim clients" for the 64-player scenario may be simplified bots or replay-based if full simulation is too complex

## Constraints

- Optimizations MUST preserve simulation determinism (if currently deterministic)
- Network optimizations MUST maintain protocol compatibility or require explicit version bump
- Mod runtime security MUST NOT be compromised by performance changes
- Profiling overhead MUST be negligible when profiling is disabled
- All optimizations MUST be measured before/after with concrete evidence
