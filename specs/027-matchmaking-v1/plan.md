# Implementation Plan: Matchmaking v1 (Quick Join)

**Branch**: `027-matchmaking-v1` | **Date**: 2025-12-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/027-matchmaking-v1/spec.md`

## Summary

Implement a client-side Quick Join system that automatically selects the best available game server based on game mode, region, and server scoring criteria. The system reuses the existing master server infrastructure (Feature 026) for server discovery, adds a scoring algorithm for intelligent selection, implements auto-retry on connection failure (up to 3 attempts), and persists user preferences to the existing profile.toml (Feature 025).

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-common (types, server_browser), plix-client (console, server_browser, profile), reqwest (HTTP), serde/toml (serialization), rand (tie-breaking)
**Storage**: `~/.config/plix/profile.toml` (extends Feature 025 profile with `[matchmaking]` section)
**Testing**: `cargo test` for unit/integration tests
**Target Platform**: Linux (primary), cross-platform via Rust
**Project Type**: Workspace with multiple crates (existing structure)
**Performance Goals**: Server selection <100ms for 1000 servers, total quick join <10 seconds
**Constraints**: 5-second connection timeout, 2-second debounce between requests, 3 retry attempts max
**Scale/Scope**: Client-side only, no server-side matchmaking orchestration

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | PASS | Client-side matchmaking is read-only; server remains authoritative for game state |
| II. Performance (Low Latency) | PASS | Non-blocking operation (SC-007), <100ms scoring (SC-004) |
| III. Architecture (Engine-First) | PASS | Uses existing primitives (server_browser, profile), no new engine changes |
| IV. Modding (First-Class) | N/A | Feature does not affect mod system |
| V. Code Quality (Explicit & Tested) | PASS | All scoring and retry logic will have unit tests |
| VI. Technical Standards (Rust) | PASS | Stable Rust only, cargo clippy/fmt compliance |
| VII. Player Experience (Multiplayer-First) | PASS | Directly improves multiplayer connection UX |
| VIII. Open Source | PASS | All code public, no proprietary dependencies |
| IX. Scoping & Realism | PASS | Minimal scope: client-side only, reuses existing infrastructure |
| X. Long-Term Vision | PASS | Modular design allows future server-side matchmaking upgrade |

**Gate Result**: PASS - No violations detected.

## Project Structure

### Documentation (this feature)

```text
specs/027-matchmaking-v1/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (internal API contracts)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/plix-client/
├── src/
│   ├── matchmaking/           # NEW: Quick join module
│   │   ├── mod.rs             # Module exports
│   │   ├── request.rs         # QuickJoinRequest handling
│   │   ├── scoring.rs         # Server scoring algorithm
│   │   ├── selection.rs       # Server selection with tie-breaking
│   │   └── retry.rs           # Auto-retry logic
│   ├── profile/               # EXISTING: Feature 025
│   │   └── mod.rs             # Extended with matchmaking preferences
│   ├── server_browser/        # EXISTING: Feature 026
│   │   └── mod.rs             # Reused for server list fetching
│   ├── console.rs             # EXTENDED: New /quickjoin, /play commands
│   └── main.rs                # EXTENDED: Quick Play menu item
└── tests/
    ├── matchmaking_test.rs    # NEW: Quick join integration tests
    └── scoring_test.rs        # NEW: Scoring algorithm unit tests

crates/plix-common/
└── src/
    └── server_browser/
        └── types.rs           # EXISTING: ServerEntry (already has game_modes, region)
```

**Structure Decision**: Extend existing plix-client crate with new `matchmaking/` module. Reuse `server_browser` for server list fetching and `profile` for preference storage. No new crates needed.

## Complexity Tracking

> No violations to justify - all gates passed.

| Item | Complexity | Justification |
|------|------------|---------------|
| Scoring algorithm | Low | Simple additive scoring with fixed weights (per spec FR-009 to FR-011) |
| Auto-retry | Low | Simple loop with counter and exclusion set |
| Preference persistence | Low | Extend existing TOML structure from Feature 025 |
