# Implementation Plan: Performance Pass

**Branch**: `039-performance-pass` | **Date**: 2025-12-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/039-performance-pass/spec.md`

## Summary

Production-grade performance optimization for Plix delivering:
- **Reproducible profiling scenarios** (Idle, World Churn, Net Stress) with structured JSON reports
- **Tick stability** via subsystem instrumentation, budgets, and backpressure mechanisms
- **Measured optimizations** for allocations (2+ hotspots), network bandwidth, and meshing (incremental/budgeted)
- **Anti-regression harness** for CI integration

Approach: Measure first, identify 80/20 bottlenecks, apply targeted optimizations with before/after evidence.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: tokio 1.0 (async), tracing 0.1 (instrumentation), bincode 1.3 (serialization), wgpu 23.0 (rendering)
**Storage**: N/A (in-memory metrics, optional JSON report export)
**Testing**: cargo test (unit/integration), perf harness binary (scenario execution)
**Target Platform**: Linux server (primary), Windows/macOS client
**Project Type**: Multi-crate workspace (plix-server, plix-client, plix-common, plix-net)

**Performance Goals**:
- 60 TPS server tick rate (16.67ms budget)
- p95 tick time < 12ms under normal load
- p99 tick time < 16ms under heavy load
- Network overhead < 50 KB/s per player at rest

**Constraints**:
- Profiling overhead negligible when disabled
- No breaking protocol changes without version bump
- Preserve simulation determinism
- Feature-gate heavy tools (allocation tracking, flame graphs)

**Scale/Scope**:
- Up to 64 concurrent players
- ~1000 chunks loaded per player view
- ~15+ network message types

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| II. Performance (Low Latency & Stability) | ✅ ALIGNED | Core focus of this feature |
| II. Tick Stability | ✅ ALIGNED | Explicit goal: stable tick rate 20-60 TPS |
| II. Event-Driven Updates | ✅ ALIGNED | Backpressure uses event-driven throttling |
| V. Code Quality (Explicit & Tested) | ✅ ALIGNED | All optimizations require before/after measurements |
| V. No Temporary Hacks | ✅ ALIGNED | Structural optimizations only, no workarounds |
| V. Structured Logging | ✅ ALIGNED | JSON report format, structured metrics |
| VI. Stable Rust Only | ✅ ALIGNED | No nightly features required |
| VI. Tooling Compliance | ✅ ALIGNED | clippy/fmt enforced |
| VI. Documented Protocols | ✅ ALIGNED | perf_report.json schema documented |
| IX. Simple Over Complex | ✅ ALIGNED | 80/20 optimizations, not micro-tuning |

**No violations detected.** All work aligns with constitution principles.

## Project Structure

### Documentation (this feature)

```text
specs/039-performance-pass/
├── plan.md              # This file
├── research.md          # Phase 0: Technical research
├── data-model.md        # Phase 1: PerfReport schema, metrics types
├── quickstart.md        # Phase 1: How to run profiling
├── contracts/           # Phase 1: perf_report.json schema
└── tasks.md             # Phase 2: Implementation tasks
```

### Source Code (repository root)

```text
crates/
├── plix-server/
│   └── src/
│       ├── lib.rs              # Main loop (tick instrumentation)
│       ├── tick.rs             # TickConfig, budgets
│       ├── metrics.rs          # ServerMetricsCollector (enhance)
│       ├── perf/               # NEW: Performance subsystem
│       │   ├── mod.rs          # Perf module exports
│       │   ├── scenarios.rs    # Scenario definitions (Idle, WorldChurn, NetStress)
│       │   ├── reporter.rs     # JSON report generation
│       │   ├── budgets.rs      # Tick budget config, backpressure
│       │   └── harness.rs      # Perf harness entry point
│       └── netloop.rs          # Network instrumentation
├── plix-client/
│   └── src/
│       ├── chunk_mesher.rs     # Meshing optimization
│       └── chunk_manager.rs    # Dirty chunk tracking
├── plix-common/
│   └── src/
│       └── protocol/
│           └── messages.rs     # Message size instrumentation
└── plix-net/
    └── src/
        └── lib.rs              # Bandwidth tracking

docs/
└── perf/
    ├── how-to-profile.md       # Step-by-step profiling guide
    ├── scenarios.md            # Scenario descriptions
    └── budgets.md              # Budget configuration guide

benches/                        # NEW: Performance harness
└── perf_harness.rs             # Scenario runner binary
```

**Structure Decision**: Extend existing crate structure with new `plix-server/src/perf/` module for performance subsystem. Add `docs/perf/` for documentation and `benches/` for harness.

## Complexity Tracking

> No constitution violations to justify.

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| New module `perf/` | Accept | Isolates perf code from core server logic |
| Feature gates | Required | Heavy profiling tools must not impact prod |
| Harness binary | Accept | Separate binary simpler than test framework |

## Implementation Phases

### Phase 0: Research (Complete)

See [research.md](./research.md) for:
- Tracing span strategy (existing infrastructure extension)
- Allocation tracking options (feature-gated)
- Network instrumentation approach
- Meshing optimization techniques

### Phase 1: Design & Contracts

See:
- [data-model.md](./data-model.md) - PerfReport, SubsystemMetrics, NetMessageStats schemas
- [contracts/perf_report.schema.json](./contracts/perf_report.schema.json) - JSON schema
- [quickstart.md](./quickstart.md) - How to run profiling

### Phase 2: Tasks

See [tasks.md](./tasks.md) (generated by `/speckit.tasks`)

## Key Design Decisions

### 1. Tick Budget Strategy

- **Target**: 16.67ms (60 TPS) with configurable override
- **Subsystem budgets**: net (1-2ms), meshing (2-4ms), mods (1-2ms)
- **Backpressure**: Skip non-critical meshing, coalesce updates, throttle net flush

### 2. Instrumentation Approach

- Extend existing `ServerMetricsCollector` with per-subsystem timing
- Add `#[instrument]` spans on hot paths (feature-gated for overhead control)
- Network: instrument at encode/decode boundary per message type

### 3. Optimization Targets

- **Allocations**: Network encoding buffers, snapshot serialization
- **Network**: Message batching, optional compression (>1KB threshold)
- **Meshing**: Dirty chunk tracking, per-tick budget with deferral

### 4. Report Format

- JSON for machine parsing, human-readable summary section
- Includes: metadata, tick_stats, subsystem_stats, net_stats, meshing_stats
- Stable schema for CI comparison

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Profiling overhead in prod | Feature-gate all heavy instrumentation |
| Breaking protocol | Version bump if any format changes |
| False optimization | Require before/after measurements |
| Scope creep | Focus on 80/20: 2 alloc hotspots, 1 net opt, 1 mesh opt |

## Definition of Done

From spec success criteria:
- [ ] 3 reproducible scenarios documented (Idle, World Churn, Net Stress)
- [ ] `perf_report.json` generated with tick/net/alloc/mesh stats
- [ ] Tick p99 improves ≥20% on one heavy scenario
- [ ] 2 allocation hotspots reduced with evidence
- [ ] Network KB/s reduced on Net Stress scenario
- [ ] Meshing optimization (incremental or budget) implemented
- [ ] Perf harness runs locally and in CI
- [ ] Documentation complete (how-to-profile, scenarios, budgets)
