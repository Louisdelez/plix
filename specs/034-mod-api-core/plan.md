# Implementation Plan: Mod API Core

**Branch**: `034-mod-api-core` | **Date**: 2025-12-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/034-mod-api-core/spec.md`

## Summary

Deliver a stable, versioned Mod API Core that serves as the official contract between the game engine (Rust) and future mod runtimes (WASM/script). This includes:

- **Manifest system**: Parse/validate `mod.toml` with capabilities, entrypoints, versioning
- **Event bus**: Stable event dispatch with FIFO ordering, error isolation, cancellation support
- **World/Entity APIs**: Bounded, permission-controlled access to game state
- **Networking**: Safe mod-to-mod messaging with rate limiting and size limits
- **Timers**: Bounded scheduling with enforced limits
- **Observability**: Structured logging and metrics for mod execution

The runtime itself (WASM/Lua/JS) is out of scope; this feature exposes a "host API" ready for future runtime integration.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-common (types), plix-server (game loop integration), serde + toml (manifest parsing), tracing (logging), glam (math types)
**Storage**: N/A (in-memory state only - mod registry, event subscriptions, timer state)
**Testing**: cargo test (unit + integration tests with dummy mod)
**Target Platform**: Linux server (plix-server), with traits designed for future WASM runtime
**Project Type**: Library crate (`plix-mod-core`) integrated into plix-server
**Performance Goals**: Event dispatch within 1 tick, <1ms permission check overhead
**Constraints**: No panics in API calls, all errors return Result<T, ModApiError>
**Scale/Scope**: Support 10+ concurrent mods, 9 MVP event types, bounded queries

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | PASS | Server-authoritative design, capability-based permissions, mod isolation |
| II. Performance (Low Latency) | PASS | Event-driven, bounded queries, rate limiting |
| III. Architecture (Engine-First) | PASS | New `plix-mod-core` crate, clean layer separation |
| IV. Modding (First-Class) | PASS | This feature IS the modding foundation |
| V. Code Quality (Explicit & Tested) | PASS | Typed errors, mandatory tests, no panics |
| VI. Technical Standards (Rust) | PASS | Stable Rust, cargo clippy/fmt compliant |
| VII. Player Experience | N/A | Server-side API, no direct player impact |
| VIII. Open Source | PASS | Public, documented API |
| IX. Scoping & Realism | PASS | MVP events only, no runtime in scope |
| X. Long-Term Vision | PASS | Versioned API, designed for 5+ year evolution |

**Gate Result**: PASS - No violations. Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/034-mod-api-core/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── mod-api.md       # API contract documentation
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-mod-core/           # NEW: Mod API Core library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Public exports
│       ├── manifest.rs      # Mod manifest parsing/validation
│       ├── capabilities.rs  # Capability enum and permission checks
│       ├── registry.rs      # Mod registry (loaded mods, state)
│       ├── events.rs        # Event bus, subscriptions, dispatch
│       ├── errors.rs        # ModApiError type and codes
│       ├── observability.rs # Logging and metrics
│       └── api/
│           ├── mod.rs       # API module exports
│           ├── world.rs     # World API (get_block, raycast, etc.)
│           ├── entities.rs  # Entity API (handles, read/write)
│           ├── net.rs       # Networking API (channels, messages)
│           └── timers.rs    # Timer API (set_timeout, set_interval)
│
├── plix-server/             # EXISTING: Integrates mod-core
│   └── src/
│       └── mods/            # NEW: Mod integration module
│           └── mod.rs       # Server-side mod loading/dispatch
│
└── plix-common/             # EXISTING: Shared types
    └── src/
        └── mod_types.rs     # NEW: Shared mod types (ModId, etc.)
```

**Structure Decision**: New `plix-mod-core` crate following existing multi-crate pattern. Integrates with `plix-server` for game loop hooks and `plix-common` for shared types.

## Locked Decisions (from Clarifications)

| Decision | Value | Source |
|----------|-------|--------|
| Timer min_interval | 50ms | Q1 Option B |
| Timer max_timers | 32 per mod | Q1 Option B |
| Raycast max_dist | 256 blocks | Q2 Option B |
| query_aabb limit | 128 results | Q2 Option B |
| Mod auto-disable threshold | 5 consecutive errors | Q3 Option B |
| Cancellable events | on_player_chat, on_block_placed, on_block_broken | Q4 Option B |

## Complexity Tracking

> No constitution violations detected. No complexity justifications required.

## Design Decisions

### Module Organization

1. **plix-mod-core**: Standalone crate for mod API, no game-specific logic
2. **Host API Traits**: Define `ModHost` trait for future runtime integration
3. **Error Isolation**: Each mod gets independent error counter, no cross-contamination

### Event Bus Design

- **Phase-based dispatch**: Events collected during tick, dispatched at end-of-tick
- **FIFO ordering**: Events dispatched in emission order per type
- **No re-entrancy**: Handlers cannot emit events that dispatch immediately
- **Cancellation**: Only specific events support cancellation, requires dedicated capability

### Capability Model

MVP Capabilities:
- `world.read` - Read world state (get_block, raycast, query_aabb)
- `world.write` - Modify world state (set_block)
- `entity.read` - Read entity state (transform, health, etc.)
- `entity.write` - Modify entities (apply_damage, spawn/despawn)
- `net.send` - Send network messages
- `event.cancel.chat` - Cancel chat events
- `event.cancel.blocks` - Cancel block placement/breaking events

### API Versioning

- `api_version` integer in manifest (MVP = 1)
- Engine exposes `get_api_version()` and `get_engine_version()`
- Mods can declare `min_api_version` / `max_api_version`
- Incompatible version → EMOD007, mod not loaded
