# Implementation Plan: Account Identity

**Branch**: `025-account-identity` | **Date**: 2025-12-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/025-account-identity/spec.md`

## Summary

Add simple, stable player identity with configurable display name, local client profile, and server-authoritative session identity, preparing for future optional authentication without implementing login in v1. The server validates and disambiguates display names (via suffix like `#2`), assigns SessionId for logging/metrics, and replicates names in PlayerSnapshot. The client stores preferred name in `~/.config/plix/profile.toml` and supports `/name` command for in-game changes.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: serde (serialization), toml (profile format), tracing (logging), bincode (protocol)
**Storage**: Client: `~/.config/plix/profile.toml` (XDG compliant); Server: in-memory only (no persistence)
**Testing**: `cargo test` - unit tests for validation/registry, integration tests for connect/rename flows
**Target Platform**: Linux (primary), cross-platform (Windows, macOS)
**Project Type**: Existing Rust workspace with 6 crates (plix-common, plix-client, plix-server, plix-arena, plix-net, plix-tools)
**Performance Goals**: O(1) name validation (<1ms), O(n) disambiguation (<10ms for n=100 players)
**Constraints**: Zero server-side persistence, no authentication in v1, rate limit 1 rename per 60s
**Scale/Scope**: Up to 100 concurrent players per server

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | ✅ PASS | Server validates all names, client suggestions only |
| II. Performance | ✅ PASS | O(1) validation, O(n) disambiguation, no tick cost |
| III. Architecture | ✅ PASS | Clean separation: plix-common (types), plix-client (profile), plix-server (registry) |
| IV. Modding | ✅ PASS | N/A - core identity, not mod-related |
| V. Code Quality | ✅ PASS | All validation logic tested, structured logging |
| VI. Technical Standards | ✅ PASS | Stable Rust, TOML format documented, protocol versioned |
| VII. Player Experience | ✅ PASS | Simple profile, automatic name persistence |
| VIII. Open Source | ✅ PASS | No proprietary dependencies |
| IX. Scoping | ✅ PASS | Minimal v1 scope, no auth/login |
| X. Long-Term Vision | ✅ PASS | AccountId/AuthToken placeholders for future extensibility |

**All gates passed. No violations to justify.**

## Project Structure

### Documentation (this feature)

```text
specs/025-account-identity/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── messages.md      # Protocol message extensions
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-common/src/
│   ├── identity/
│   │   ├── mod.rs           # Module re-exports
│   │   ├── display_name.rs  # DisplayName type + validation
│   │   └── session.rs       # SessionId type
│   └── protocol/
│       └── messages.rs      # Extended with RenameRequest/RenameResult
│
├── plix-client/src/
│   ├── profile/
│   │   ├── mod.rs           # Module re-exports
│   │   └── player_profile.rs # PlayerProfile load/save
│   └── console.rs           # Extended with /name command
│
└── plix-server/src/
    ├── identity/
    │   ├── mod.rs           # Module re-exports
    │   ├── name_registry.rs # Display name uniqueness + suffixes
    │   └── rate_limit.rs    # Rename rate limiting
    └── session.rs           # Extended with SessionId
```

**Structure Decision**: Extends existing workspace crates with new `identity/` modules. No new crates needed. Profile persistence reuses existing `~/.config/plix/` directory (same as `config.toml`).

## Complexity Tracking

> No violations - table not required.
