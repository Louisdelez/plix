# Implementation Plan: Block Physics Light

**Branch**: `015-block-physics` | **Date**: 2025-12-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/015-block-physics/spec.md`

## Summary

Add a minimal, event-driven block physics simulation to the plix voxel engine supporting optional gravity for certain block types (e.g., sand) and simple liquid spreading. The system uses a bounded event queue to guarantee stable performance under cascade scenarios, integrates with the existing chunked world system, and maintains server authority for multiplayer determinism.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-common (types, world, chunk), plix-server (game loop), bincode (serialization), glam (math)
**Storage**: N/A (in-memory event queue, block state in existing ChunkedWorld)
**Testing**: cargo test for unit/integration tests
**Target Platform**: Linux server (authoritative), Windows/Linux/macOS clients (reflect state)
**Project Type**: Workspace with multiple crates (plix-common, plix-server, plix-client)
**Performance Goals**: Process up to 100 gravity events + 50 liquid events per tick without exceeding 10% tick time overhead
**Constraints**: Event-driven only (no global iteration), deterministic, cross-chunk boundaries, budget-bounded
**Scale/Scope**: Infinite world (chunked), 60 TPS server tick rate

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | PASS | Server is authoritative for physics; clients reflect state only |
| II. Performance (Low Latency) | PASS | Event-driven, budget-bounded, no global iteration |
| III. Architecture (Engine-First) | PASS | Physics as engine primitive, gameplay builds on it |
| IV. Modding (Extensibility) | PASS | PhysicsConfig allows mods to configure block behaviors |
| V. Code Quality | PASS | Mandatory tests for physics logic |
| VI. Technical Standards | PASS | Rust stable, deterministic APIs |
| VII. Player Experience | PASS | Multiplayer-first, server authoritative |
| VIII. Open Source | PASS | No proprietary dependencies |
| IX. Scoping & Realism | PASS | Minimal MVP: gravity + simple liquids only |
| X. Long-Term Vision | PASS | Extensible design for future physics behaviors |

**Gate Result**: PASS - All principles satisfied

## Project Structure

### Documentation (this feature)

```text
specs/015-block-physics/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (internal API contracts)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-common/
│   └── src/
│       ├── physics/           # NEW: Physics types and config
│       │   ├── mod.rs         # Module exports
│       │   ├── config.rs      # PhysicsConfig
│       │   ├── event.rs       # PhysicsEvent enum
│       │   ├── queue.rs       # PhysicsQueue (bounded FIFO)
│       │   └── metrics.rs     # PhysicsMetrics counters
│       └── types.rs           # Extend BlockType with physics flags
│
├── plix-server/
│   └── src/
│       ├── physics/           # NEW: Server-side physics system
│       │   ├── mod.rs         # Module exports
│       │   ├── system.rs      # PhysicsSystem (tick processing)
│       │   ├── gravity.rs     # Gravity resolution logic
│       │   └── liquid.rs      # Liquid spreading logic (optional)
│       └── lib.rs             # Integrate physics into game loop
│
└── plix-client/
    └── src/
        └── (no physics logic - receives block updates from server)

tests/
├── physics_gravity_test.rs    # Gravity unit tests
├── physics_queue_test.rs      # Queue/budget tests
├── physics_liquid_test.rs     # Liquid spreading tests
└── physics_integration_test.rs # Full integration tests
```

**Structure Decision**: Physics types in plix-common (shared), physics execution in plix-server (server authoritative). Client receives block updates via existing replication, no client-side physics.

## Complexity Tracking

> No violations - design follows constitution principles

| Decision | Rationale |
|----------|-----------|
| Physics in plix-common + plix-server | Types shared, execution server-only per authority model |
| Event-driven queue | Constitution II.4: Event-driven updates over polling |
| No client prediction | Constitution I.1: Server authoritative, v1 simplicity |

## Architecture Overview

### Event Flow

```
Block Edit (player/world)
    → detect_physics_events()
    → PhysicsQueue.push()

Server Tick
    → PhysicsSystem.tick()
        → drain queue (up to budget)
        → resolve each event (gravity fall / liquid spread)
        → world.set_block() → mark dirty
        → detect new events from changes
    → broadcast BlockEditApplied to clients
```

### Key Design Decisions

1. **Step-based falling**: Blocks fall 1 cell per physics tick for visual smoothness
2. **FIFO queue**: Deterministic processing order
3. **Deduplication**: Queue rejects duplicate (pos, event_type) entries
4. **Cross-chunk**: Uses existing ChunkedWorld API which handles boundaries
5. **No retroactive simulation**: Physics only processes future events from chunk load onwards

## Integration Points

1. **Block Edit Hook**: After `set_block()` in server, call `detect_physics_events()`
2. **Game Loop**: Call `physics_system.tick()` after movement, before snapshots
3. **Config**: Add `PhysicsConfig` to `ServerConfig`
4. **Metrics**: Expose physics counters in server metrics system
