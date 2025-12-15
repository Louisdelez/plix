# Implementation Plan: Logging & Metrics

**Branch**: `010-logging-metrics` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/010-logging-metrics/spec.md`

## Summary

Add lightweight observability to Plix with server-side tick/network metrics and client debug overlay. Server tracks tick time (avg/p95/max over 10s rolling window), per-session network metrics (RTT, jitter, loss), and logs summaries every 5s. Client gains F3-toggled debug overlay showing network stats at 2Hz. RTT is measured by piggybacking an echo nonce on existing input messages.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: tokio (async), bincode (serialization), glam (math), wgpu (client rendering), tracing (logging)
**Storage**: N/A (in-memory metrics only)
**Testing**: cargo test (unit + integration tests)
**Target Platform**: Linux server + client, cross-platform client
**Project Type**: Workspace with multiple crates (plix-common, plix-server, plix-client, plix-net)
**Performance Goals**: < 0.167ms tick measurement overhead, < 1ms overlay render, no allocations in hot path
**Constraints**: No per-tick logging, headless mode compatibility, no external telemetry dependencies
**Scale/Scope**: 8-16 concurrent players, 60Hz tick rate

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security | PASS | Metrics are server-computed, no client trust |
| II. Performance | PASS | Ring buffers, no per-tick logging, no allocations |
| III. Architecture | PASS | Metrics in plix-net/plix-server, overlay in plix-client |
| IV. Modding | N/A | No mod API changes |
| V. Code Quality | PASS | Unit tests for stats, structured logging |
| VI. Technical Standards | PASS | Stable Rust, clippy/fmt compliant |
| VII. Player Experience | PASS | F3 overlay aids debugging |
| VIII. Open Source | PASS | No proprietary dependencies |
| IX. Scoping | PASS | Minimal scope, reuses existing infrastructure |
| X. Long-Term Vision | PASS | Foundation for future observability |

## Project Structure

### Documentation (this feature)

```text
specs/010-logging-metrics/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-common/src/
│   ├── protocol/
│   │   └── messages.rs    # Add rtt_nonce to PlayerInput, echo in WorldSnapshot
│   └── metrics.rs         # NEW: RollingWindow<T>, Stats struct (shared)
├── plix-net/src/
│   └── metrics.rs         # EXISTING: NetworkMetrics - extend for 10s window
├── plix-server/src/
│   ├── tick.rs            # Add tick time recording
│   ├── metrics.rs         # NEW: ServerMetricsCollector, periodic logging
│   └── session.rs         # Add per-session net metrics
└── plix-client/src/
    ├── config.rs          # Add ToggleDebugOverlay action
    ├── ui/
    │   └── net_debug.rs   # EXISTING: Implement render(), add F3 handling
    └── render/
        └── engine.rs      # Call overlay render when visible

tests/
└── crates/plix-server/tests/
    └── metrics_test.rs    # NEW: Rolling window, stats tests
```

**Structure Decision**: Existing workspace crate structure. Metrics shared via plix-common, server metrics in plix-server, client overlay in plix-client/ui.

## Complexity Tracking

No constitution violations requiring justification. Feature uses existing patterns and infrastructure.

## Phases

### Phase 1: Metrics Core (Common Utilities)

**Goal**: Shared rolling window + stats helpers

- Implement `RollingWindow<T>` fixed-size ring buffer (no alloc in hot path)
- Compute avg / p95 / max for time-based 10s window
- Reusable for tick time and network metrics

**Deliverables**:
- `crates/plix-common/src/metrics.rs`: `RollingWindow<T>`, `Stats { avg, p95, max }`
- Unit tests for p95 correctness

### Phase 2: RTT Protocol Plumbing (Echo Timestamp)

**Goal**: Get RTT samples without new message types

- Extend `PlayerInput` with `rtt_nonce: u64` field
- Extend `WorldSnapshot` to echo latest `rtt_nonce`
- Client stores send timestamps, computes RTT on echo

**Deliverables**:
- Protocol fields updated in `plix-common/src/protocol/messages.rs`
- Server echoes nonce in `plix-server/src/netloop.rs`
- Client RTT computation in `plix-client/src/net.rs`

### Phase 3: Server Metrics (Tick Time + Aggregates)

**Goal**: Server visibility into simulation performance

- Measure tick duration each server tick using `std::time::Instant`
- Update rolling stats window (10s = 600 samples at 60Hz)
- Log summary every 5s: tick avg/p95/max, players connected, sessions active

**Deliverables**:
- `crates/plix-server/src/metrics.rs`: `ServerMetricsCollector`
- Structured logs (info level) via tracing

### Phase 4: Server Network Metrics (Per Session)

**Goal**: Per-session health metrics for debugging

- Track RTT samples server-side (from snapshot ack timing)
- Compute jitter (std dev) and loss (sequence gaps) in 10s window
- Track PPS in/out per session

**Deliverables**:
- `SessionNetMetricsCollector` in session management
- Per-client metrics snapshot (optional: include in server logs)

### Phase 5: Client Debug Overlay

**Goal**: In-game text overlay for testers/dev

- Add `ToggleDebugOverlay` action bound to F3
- Implement `NetDebugOverlay::render()` using existing UI text patterns
- Update at 2Hz (cached text lines)
- Display: FPS, RTT, Jitter, Loss%, Server tick, PlayerId

**Deliverables**:
- F3 keybind in `plix-client/src/config.rs`
- Render implementation in `plix-client/src/ui/net_debug.rs`
- Integration with render loop

### Phase 6: Logging Policy & Developer Ergonomics

**Goal**: Useful logs without noise

- Ensure logs are rate-limited (summary every 5s)
- Use tracing spans for structured output
- Document log format in quickstart.md

**Deliverables**:
- No per-tick spam verified
- Log format documented

### Phase 7: Validation & Non-Regression

**Goal**: Lock it in

**Automated**:
- Rolling window stats tests
- Protocol encode/decode tests for nonce fields
- Smoke test: server runs and logs summary
- Headless + load tests compile and pass

**Manual**:
- Run server + client, toggle F3, verify overlay updates
- Simulate jitter/loss and confirm overlay reflects changes

**Definition of Done**:
- Server logs periodic tick summary
- Overlay displays stable RTT/jitter/loss
- No headless regression
- `cargo test --workspace` passes, clippy/fmt clean
