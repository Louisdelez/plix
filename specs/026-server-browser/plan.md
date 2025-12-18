# Implementation Plan: Server Browser v1

**Branch**: `026-server-browser` | **Date**: 2025-12-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/026-server-browser/spec.md`

## Summary

Implement a server browser system enabling multiplayer server discovery through a master server directory. The feature introduces:
1. A new `plix-master` binary/service exposing HTTP API for server registration and listing
2. Heartbeat integration in `plix-server` to announce to the master
3. Console commands in `plix-client` for browsing, filtering, and connecting to servers
4. Local favorites persistence using TOML

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**:
- `axum` (HTTP server for master - lightweight, tokio-native)
- `reqwest` (HTTP client for game server and client)
- `tokio` (async runtime - already in workspace)
- `serde` + `serde_json` (JSON serialization - already in workspace)
- `tracing` (logging - already in workspace)
- `toml` (favorites persistence - already in workspace)
**Storage**:
- Master: In-memory HashMap (no persistence)
- Client: `~/.config/plix/servers.toml` for favorites
**Testing**: `cargo test` (unit + integration)
**Target Platform**: Linux (server/client), cross-platform client
**Project Type**: Workspace with new crate `plix-master` + modifications to `plix-server` and `plix-client`
**Performance Goals**:
- Server list fetch < 5s
- Search/filter < 1s on 1000 servers
- Heartbeat overhead negligible (20s interval)
**Constraints**:
- Master server HTTP API (no persistence, in-memory)
- Console-only interface (no CEF/UI)
- Non-blocking async operations
**Scale/Scope**: ~1000 concurrent servers in directory

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | PASS | Master is read-only for clients; game server is source of truth for its own info; rate limiting prevents abuse |
| II. Performance (Low Latency) | PASS | Async HTTP calls; non-blocking heartbeat; no impact on game tick |
| III. Architecture (Engine-First) | PASS | New master crate is separate service; clean integration via HTTP |
| IV. Modding (Extensibility) | N/A | No mod API changes |
| V. Code Quality | PASS | Explicit types, tests required, structured logging |
| VI. Technical Standards | PASS | Stable Rust, clippy/fmt compliance, documented protocol |
| VII. Player Experience | PASS | Integrated server browser; no external tools required |
| VIII. Open Source | PASS | No proprietary dependencies |
| IX. Scoping & Realism | PASS | MVP scope: console-only, in-memory, no auth |
| X. Long-Term Vision | PASS | HTTP API is extensible; master can be enhanced later |

**Gate Result**: PASS - No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/026-server-browser/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (HTTP API specs)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── plix-master/              # NEW: Master server binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs           # Entry point, CLI args
│       ├── lib.rs            # Core logic exports
│       ├── api.rs            # Axum routes (GET /servers, POST /heartbeat)
│       ├── state.rs          # ServerRegistry (HashMap + TTL)
│       ├── types.rs          # ServerEntry, HeartbeatRequest
│       ├── validation.rs     # Field validation (size, charset)
│       └── rate_limit.rs     # IP-based rate limiting
│
├── plix-common/
│   └── src/
│       └── server_browser/   # NEW: Shared types
│           ├── mod.rs
│           └── types.rs      # ServerEntry, ServerListResponse (shared)
│
├── plix-server/
│   └── src/
│       ├── master_announce/  # NEW: Heartbeat task
│       │   ├── mod.rs
│       │   ├── config.rs     # MasterConfig (url, name, region, tags)
│       │   └── heartbeat.rs  # Async heartbeat loop
│       └── main.rs           # Integration point
│
└── plix-client/
    └── src/
        ├── server_browser/   # NEW: Browser logic
        │   ├── mod.rs
        │   ├── fetch.rs      # HTTP client for master
        │   ├── filter.rs     # Search/filter/sort logic
        │   └── favorites.rs  # TOML persistence
        ├── console.rs        # Extended with /servers, /connect, /favorite commands
        └── main.rs           # Integration point
```

**Structure Decision**: Multi-crate workspace pattern (consistent with existing plix-* crates). New `plix-master` crate for the master server, shared types in `plix-common/server_browser`, heartbeat integration in `plix-server`, and console commands in `plix-client`.

## Complexity Tracking

No violations requiring justification. Feature follows MVP principles:
- Single new crate (plix-master)
- Minimal changes to existing crates
- In-memory storage (no database)
- Console-only UI
