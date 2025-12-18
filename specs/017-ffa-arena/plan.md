# Implementation Plan: FFA Arena Mode

**Branch**: `017-ffa-arena` | **Date**: 2025-12-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/017-ffa-arena/spec.md`

## Summary

Implement Free-for-All (FFA) game mode where players score points individually by eliminating others. Primary change is adding `game_mode` field to arena config to branch between FFA individual scoring and TDM team scoring. Reuses 95%+ of existing TDM infrastructure (MatchState, respawn system, individual scoring via `check_score_limit`).

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-arena (arena loading), plix-server (match logic), plix-common (types, protocol), glam (math), bincode (serialization), tokio (async)
**Storage**: N/A (in-memory state only, arena definitions in TOML files)
**Testing**: `cargo test` (unit + integration tests)
**Target Platform**: Linux server (primary), cross-platform client
**Project Type**: Workspace with multiple crates (plix-arena, plix-server, plix-client, plix-common, plix-net, plix-tools)
**Performance Goals**: 60Hz server tick rate, O(1) per kill/respawn event, no global scans
**Constraints**: Server-authoritative, <16ms tick budget, deterministic behavior
**Scale/Scope**: 8+ concurrent players in FFA mode

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| **I. Security (Server Authority)** | ✅ PASS | Server-authoritative scoring, clients cannot force scores/winner |
| **II. Performance (Low Latency)** | ✅ PASS | O(1) scoring operations, no polling, reuses existing tick loop |
| **III. Architecture (Engine-First)** | ✅ PASS | Reuses existing engine primitives (MatchState, respawn, scoring) |
| **IV. Modding (First-Class)** | ✅ PASS | Data-driven via arena TOML config, no hardcoded game mode |
| **V. Code Quality (Explicit & Tested)** | ✅ PASS | Mandatory tests for scoring, respawn, state transitions |
| **VI. Technical Standards (Rust)** | ✅ PASS | Stable Rust only, cargo clippy/fmt compliant |
| **VII. Player Experience (Multiplayer)** | ✅ PASS | Multiplayer-first design, FFA is team-independent |
| **VIII. Open Source** | ✅ PASS | No proprietary dependencies |
| **IX. Scoping (Minimal)** | ✅ PASS | Minimal changes - single `game_mode` field + branching logic |
| **X. Long-Term Vision** | ✅ PASS | Game mode is platform feature, not product-specific |

**Gate Result**: PASS - No constitution violations. Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/017-ffa-arena/
├── spec.md              # Feature specification (complete)
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (event schemas)
├── checklists/          # Quality checklists
│   └── requirements.md  # Spec quality checklist (complete)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-arena/
│   └── src/
│       ├── format.rs      # Arena/ArenaMetadata/SpawnPoint - ADD game_mode field
│       ├── validate.rs    # Arena validation - ADD FFA spawn validation
│       └── spawn.rs       # SpawnManager - ADD neutral spawn selection
├── plix-server/
│   └── src/
│       ├── match_state.rs # MatchStateMachine - ADD ffa_default() config
│       └── lib.rs         # Kill processing - BRANCH on game_mode for scoring
├── plix-common/
│   └── src/
│       └── protocol/
│           └── messages.rs # MatchState - ADD game_mode field for client awareness
└── plix-client/
    └── src/
        └── ui/            # (minimal) display individual scores in FFA

assets/arenas/
├── test_arena.toml        # ADD game_mode = "tdm" (preserve TDM default)
└── ffa_arena.toml         # NEW example FFA arena config
```

**Structure Decision**: Existing workspace structure preserved. Changes are minimal additions to existing files in plix-arena, plix-server, plix-common.

## Complexity Tracking

> No violations requiring justification. Feature reuses existing infrastructure with minimal additions.

| Aspect | Complexity | Justification |
|--------|------------|---------------|
| New files | 1 (ffa_arena.toml) | Example arena only |
| Modified files | ~6 | Minimal changes per file |
| New types | 1 (GameMode enum) | Simple enum, 2 variants |
| New logic | ~50 LOC | Branch on game_mode in kill processing |
