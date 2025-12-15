# Implementation Plan: Plix MVP v0.1 - Authoritative Server Network Architecture

**Branch**: `001-voxel-game-platform` | **Date**: 2025-12-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-voxel-game-platform/spec.md`

## Summary

MVP v0.1 validates the authoritative server architecture for a competitive multiplayer voxel game. The focus is proving network synchronization with 8-16 players in predefined arenas with basic PvP combat. All other features (mods, procedural generation, server browser) are deferred to post-MVP.

**Primary Objective**: Demonstrate stable, fair, low-latency multiplayer with server-authoritative game state.

## Technical Context

**Language/Version**: Rust stable (latest stable channel)
**Primary Dependencies**:
- Networking: Custom UDP transport layer (no external game networking crate)
- Rendering: wgpu or similar minimal 3D pipeline
- Windowing: winit for cross-platform window/input
- Math: glam for vector/matrix math
- Serialization: bincode or custom binary protocol
- Logging: tracing with structured output

**Storage**: File-based arena definitions (TOML/JSON), no database for MVP
**Testing**: cargo test (unit + integration), custom network simulator for stress tests
**Target Platform**: Desktop (Windows, Linux, macOS) - cross-platform from day one
**Project Type**: Cargo workspace with multiple crates

**Performance Goals**:
- Server: 60 TPS stable with 16 players, tick time < 10ms p95
- Client: 60+ FPS rendering, 20-30 Hz network updates
- Latency: Playable experience up to 200ms RTT

**Constraints**:
- No GC pauses (Rust guarantees this)
- Packet size: < 1400 bytes MTU-safe
- Memory: Server < 500MB for 16 players, Client < 1GB

**Scale/Scope**: 8-16 concurrent players per server (MVP), single arena active

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | PASS | Server-authoritative architecture is the MVP's core objective |
| II. Performance (Low Latency) | PASS | 60 TPS target, fixed tick, no GC, UDP transport |
| III. Architecture (Engine-First) | PASS | Modular crate structure separates concerns |
| IV. Modding | DEFERRED | Not in MVP scope - designed for future integration |
| V. Code Quality | PASS | Testing requirements defined, no panics policy |
| VI. Technical Standards | PASS | Rust stable, clippy, fmt, documented protocol |
| VII. Player Experience | PARTIAL | Direct IP only (browser deferred), but core gameplay validated |
| VIII. Open Source | PASS | All code public, no proprietary dependencies |
| IX. Scoping & Realism | PASS | Minimal MVP with clear boundaries |
| X. Long-Term Vision | PASS | Architecture designed for extension without breaking |

**Violations requiring justification**: None

## Project Structure

### Documentation (this feature)

```text
specs/001-voxel-game-platform/
├── plan.md              # This file
├── research.md          # Phase 0: Technical decisions
├── data-model.md        # Phase 1: Entity definitions
├── quickstart.md        # Phase 1: How to run
├── contracts/           # Phase 1: Protocol specification
│   └── protocol-v0.md
└── tasks.md             # Phase 2: Implementation tasks
```

### Source Code (repository root)

```text
/
├── Cargo.toml                    # Workspace manifest
├── crates/
│   ├── plix-common/              # Shared types, protocol, math
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── math.rs           # Vec3, Quat, AABB
│   │   │   ├── types.rs          # PlayerId, EntityId, BlockPos
│   │   │   ├── protocol.rs       # Message types, serialization
│   │   │   └── time.rs           # Tick, timestamps
│   │   └── Cargo.toml
│   │
│   ├── plix-net/                 # UDP transport layer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── transport.rs      # UDP socket wrapper
│   │   │   ├── channel.rs        # Unreliable, reliable, ordered
│   │   │   ├── connection.rs     # Handshake, keepalive, timeout
│   │   │   └── metrics.rs        # RTT, jitter, packet loss
│   │   └── Cargo.toml
│   │
│   ├── plix-server/              # Authoritative game server
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── main.rs           # CLI entry point
│   │   │   ├── tick.rs           # Fixed tick loop
│   │   │   ├── session.rs        # Player session management
│   │   │   ├── simulation.rs     # Physics, collision, combat
│   │   │   ├── replication.rs    # Snapshot generation, delta encoding
│   │   │   ├── validation.rs     # Anti-cheat, input validation
│   │   │   └── match_state.rs    # Round management, scoring
│   │   └── Cargo.toml
│   │
│   ├── plix-client/              # Game client
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── main.rs           # Entry point
│   │   │   ├── input.rs          # Input capture → commands
│   │   │   ├── prediction.rs     # Client-side prediction
│   │   │   ├── reconciliation.rs # Server correction handling
│   │   │   ├── interpolation.rs  # Remote entity smoothing
│   │   │   ├── render.rs         # Voxel rendering
│   │   │   └── hud.rs            # Minimal UI overlay
│   │   └── Cargo.toml
│   │
│   ├── plix-arena/               # Arena definitions
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── loader.rs         # Load arena from file
│   │   │   ├── format.rs         # Arena data structures
│   │   │   └── spawn.rs          # Spawn point logic
│   │   └── Cargo.toml
│   │
│   └── plix-tools/               # Development tools
│       ├── src/
│       │   ├── lib.rs
│       │   ├── arena_gen.rs      # Simple arena generator
│       │   ├── net_sim.rs        # Latency/loss simulator
│       │   └── bot.rs            # Headless test client
│       └── Cargo.toml
│
├── assets/
│   ├── arenas/
│   │   └── test_arena.toml       # Default test arena
│   └── textures/
│       └── placeholder.png
│
├── docs/
│   ├── architecture.md
│   ├── protocol.md
│   └── testing.md
│
├── scripts/
│   ├── run_server.sh
│   └── run_client.sh
│
└── README.md
```

**Structure Decision**: Cargo workspace with 6 crates following engine-first modularity principle. Each crate has a single responsibility and minimal dependencies on others. `plix-common` is the shared foundation, `plix-net` handles transport only, and higher-level crates compose these primitives.

## Complexity Tracking

No constitution violations requiring justification. The architecture follows all principles.

## MVP Scope Boundaries

### In Scope (MVP v0.1)

| Category | Features |
|----------|----------|
| **Network** | UDP transport, reliable/unreliable channels, handshake, RTT measurement |
| **Server** | Fixed tick loop, authoritative simulation, snapshot replication, input validation |
| **Client** | Prediction, reconciliation, interpolation, basic voxel rendering |
| **Gameplay** | FPS movement, simple melee combat, HP, respawn, scoring |
| **Arena** | Static predefined arenas, spawn points, round reset |
| **UI** | IP:PORT connect, HUD (ping, FPS, HP, score) |
| **Admin** | CLI config (port, tickrate, max players, arena) |

### Out of Scope (Post-MVP)

| Category | Deferred Features |
|----------|-------------------|
| **Network** | Server browser, matchmaking, relay servers |
| **World** | Procedural generation, block modification, persistence |
| **Gameplay** | Mobs, crafting, inventory, items, projectiles |
| **Mods** | All mod system features |
| **UI** | Customization, menus beyond connect, spectator mode |
| **Admin** | In-game admin UI, permission system |

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| UDP reliability too complex | Medium | High | Limit reliable channel usage to connection/events only |
| Hit registration feels unfair | Medium | High | Start with melee-only, strict server validation |
| Voxel rendering performance | Low | Medium | Simple renderer, optimize later |
| Cross-platform build issues | Low | Medium | CI builds for all 3 platforms from start |

## Definition of Done (MVP)

- [ ] Server launches headless, accepts 8-16 clients via IP
- [ ] Two human players can:
  - Move without visible jitter
  - See each other smoothly (interpolation)
  - Fight with server-authoritative hit detection
- [ ] Complete round cycle: start → combat → score → end → reset
- [ ] Logs show stable tick rate (within 5% of 60 TPS)
- [ ] No crashes after 10+ minutes with 8 bot clients
- [ ] Documentation: README, protocol.md, architecture.md
- [ ] CI: lint + format + tests pass on Linux/Windows/macOS
