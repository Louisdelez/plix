# Implementation Plan: CTF Mode (Capture The Flag)

**Branch**: `018-ctf-mode` | **Date**: 2025-12-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/018-ctf-mode/spec.md`

## Summary

Implement a Capture The Flag (CTF) objective-based game mode where two teams compete to capture the enemy flag and return it to their base. The implementation follows the existing TDM/FFA architecture pattern, extending `GameMode` enum and `MatchStateMachine` with CTF-specific state (flags, zones, capture scoring). Server-authoritative architecture ensures all flag interactions are validated server-side.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-common (types, protocol), plix-server (match state, game logic), plix-arena (zone definitions), glam (math/Vec3), bincode (serialization), tokio (async)
**Storage**: N/A (in-memory state only - flag states, scores reset on match end)
**Testing**: cargo test (unit tests for state transitions, zone collisions, capture logic)
**Target Platform**: Linux server (server-authoritative), cross-platform client
**Project Type**: Single workspace with multiple crates (existing structure)
**Performance Goals**: 60 TPS server tick rate, event-driven O(1) flag state updates
**Constraints**: <16ms tick budget, no global iteration loops (event-driven per constitution)
**Scale/Scope**: 16 concurrent players (8v8), 2 teams, 2 flags

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Server Authority | ✅ PASS | All flag state managed server-side, client sends pickup intents only |
| II. Performance (Event-Driven) | ✅ PASS | Flag interactions are event-driven (pickup/drop/capture events), no polling |
| III. Architecture (Engine-First) | ✅ PASS | Extends existing GameMode enum and MatchStateMachine, no new simulation loops |
| V. Code Quality | ✅ PASS | Mandatory tests for flag state transitions and zone collisions |
| VI. Technical Standards | ✅ PASS | Rust stable, cargo clippy/fmt compliance |
| IX. Scoping | ✅ PASS | Minimal MVP - 2 teams, classic capture rule, no flag physics |

## Project Structure

### Documentation (this feature)

```text
specs/018-ctf-mode/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── plix-common/
│   └── src/
│       ├── types.rs           # GameMode::Ctf variant, FlagState, FlagZone types
│       └── protocol/
│           └── messages.rs    # CTF-specific messages (FlagUpdate, CaptureEvent)
├── plix-server/
│   └── src/
│       ├── match_state.rs     # CTFMatchState, MatchConfig::ctf_default()
│       ├── ctf/               # NEW: CTF subsystem
│       │   ├── mod.rs         # Module exports
│       │   ├── state.rs       # CtfState (flags, zones, scores)
│       │   ├── rules.rs       # CtfRules (pickup/drop/capture logic)
│       │   └── coordinator.rs # CtfCoordinator (event orchestration)
│       └── lib.rs             # Integration with game server loop
├── plix-arena/
│   └── src/
│       └── format.rs          # CTF zone definitions (flag_base, capture_zone)
└── tests/
    └── ctf/                   # CTF integration tests
        ├── capture_test.rs    # Flag capture scenarios
        ├── state_test.rs      # State transition tests
        └── zone_test.rs       # Zone collision tests

assets/arenas/
└── ctf_arena.toml             # Example CTF arena configuration
```

**Structure Decision**: Extends existing workspace structure. New `ctf/` module in plix-server follows existing pattern (match_state.rs for TDM/FFA). CTF-specific types added to plix-common for shared client/server use.

## Complexity Tracking

> No constitution violations - design follows existing patterns.
