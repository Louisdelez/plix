# Implementation Plan: Training Mode

**Branch**: `020-training-mode` | **Date**: 2025-12-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification for sandbox training mode with basic bots

## Summary

Implement a sandbox training mode (`game_mode = "training"`) that allows players to practice freely against basic bots with configurable behaviors (dummy/roam/strafe), fast respawns, session reset, and debug statistics output. The server remains authoritative; bots are simplified entities managed by a dedicated `TrainingCoordinator`.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-common (types, protocol), plix-server (match state, game loop), plix-arena (arena loading), glam (Vec3), bincode (serialization), tokio (async), tracing (logging)
**Storage**: N/A (in-memory state only - bots, stats reset on match end/disconnect)
**Testing**: cargo test (unit tests for bot behaviors, stats, reset logic)
**Target Platform**: Linux server (same as existing server)
**Project Type**: workspace crate extension (plix-server)
**Performance Goals**: Stable 60Hz tick with up to 20 bots; O(n) per-tick bot updates
**Constraints**: No AI pathfinding, no complex behavior; bots are simple state machines
**Scale/Scope**: Single player per training session (solo/private server)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | ✅ PASS | Server authoritative for all bot state, hits, stats |
| II. Performance (Tick Stability) | ✅ PASS | Bot updates bounded O(n), no world scans |
| III. Architecture (Engine-First) | ✅ PASS | Uses existing primitives (spawn, damage, tick loop) |
| IV. Modding (Extensibility) | ✅ PASS | TrainingConfig is data-driven, extensible later |
| V. Code Quality (Tested) | ✅ PASS | Unit tests required for bot/stats/reset logic |
| VI. Technical Standards (Stable Rust) | ✅ PASS | Stable Rust, cargo fmt/clippy enforced |
| VII. Player Experience (Multiplayer-First) | ✅ PASS | Training mode is local server case |
| VIII. Open Source | ✅ PASS | No proprietary dependencies |
| IX. Scoping (Minimal MVP) | ✅ PASS | Simple bots, no AI, debug-only stats |
| X. Long-Term Vision | ✅ PASS | Bot base extensible for future AI |

## Project Structure

### Documentation (this feature)

```text
specs/020-training-mode/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (protocol messages)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/plix-server/src/
├── training/                    # New module for Training Mode
│   ├── mod.rs                  # Module exports
│   ├── config.rs               # TrainingConfig struct
│   ├── bot.rs                  # TrainingBot, BotState, BotBehavior
│   ├── stats.rs                # TrainingStats tracking
│   └── coordinator.rs          # TrainingCoordinator (orchestration)
├── lib.rs                       # Add training module, GameMode::Training
└── match_state.rs               # Add training_default() config

crates/plix-common/src/
├── types.rs                     # Add GameMode::Training variant
└── protocol/
    └── messages.rs              # Add TrainingStatsResponse, TrainingReset events

crates/plix-arena/src/
└── format.rs                    # Add TrainingArenaConfig (optional bot spawn points)

assets/arenas/
└── training_arena.toml          # Sample training arena definition

crates/plix-server/tests/
├── training_bot_test.rs         # Bot spawn, respawn, behavior tests
├── training_stats_test.rs       # Stats tracking, accuracy calculation
└── training_reset_test.rs       # Session reset tests
```

**Structure Decision**: Extend existing plix-server with a new `training/` submodule following the pattern of `ctf/` and `br_lite/`. This keeps training mode logic isolated while reusing the existing match state machine, session management, and tick loop infrastructure.

## Complexity Tracking

No violations to justify - design uses existing patterns and minimal new abstractions.
