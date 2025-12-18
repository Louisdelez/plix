# Implementation Plan: Crafting Lite

**Branch**: `023-crafting-lite` | **Date**: 2025-12-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/023-crafting-lite/spec.md`

## Summary

Implement a minimalist server-authoritative crafting system that allows players to convert resource items (SCRAP) into useful equipment (HEALTH_PACK, SWORD, BOW) via simple recipes. The system integrates with the existing hotbar inventory (Feature 021), uses console commands for triggering (`/craft <recipe>`), and enforces a 1-second cooldown between successful crafts.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-common (types, inventory, protocol), plix-server (game loop, session), plix-client (console commands)
**Storage**: N/A (in-memory state only - crafting state resets on match end)
**Testing**: cargo test (unit + integration tests)
**Target Platform**: Linux server + cross-platform client
**Project Type**: Rust workspace with multiple crates
**Performance Goals**: <100ms craft latency, <1% tick time impact
**Constraints**: Atomic operations, server-authoritative, no UI required
**Scale/Scope**: 3 recipes v1, 9-slot hotbar, ~32 players max

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | ✅ PASS | Server validates all craft requests; client only sends recipe_id |
| II. Performance | ✅ PASS | O(9 slots) validation per craft; event-driven, no polling |
| III. Architecture (Modularity) | ✅ PASS | New `crafting/` module follows weapons/ pattern |
| IV. Modding | ✅ PASS | Static recipes v1; future: data-driven recipes via TOML |
| V. Code Quality | ✅ PASS | Atomic operations, explicit error handling, mandatory tests |
| VI. Technical Standards | ✅ PASS | Stable Rust, clippy/fmt compliant, versioned protocol |
| VII. Player Experience | ✅ PASS | Console command MVP; multiplayer-first design |
| VIII. Open Source | ✅ PASS | No proprietary dependencies |
| IX. Scoping (MVP) | ✅ PASS | 3 recipes, 1 resource type, console-only, no UI |
| X. Long-Term Vision | ✅ PASS | Extensible registry pattern for future recipes |

**Gate Result**: PASS - No violations. Proceeding to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/023-crafting-lite/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (protocol messages)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/plix-common/src/
├── types.rs                    # Add ItemId::SCRAP constant
├── inventory/
│   ├── item.rs                 # ItemKind::Resource variant
│   └── hotbar.rs               # (existing) count_item, consume_items methods
└── protocol/
    └── messages.rs             # CraftRequest, CraftResult messages

crates/plix-server/src/
├── crafting/                   # NEW MODULE
│   ├── mod.rs                  # Module exports
│   ├── recipe.rs               # RecipeId, Recipe, RecipeRegistry
│   ├── system.rs               # CraftSystem (validate + apply)
│   ├── cooldown.rs             # CraftCooldown (1s per player)
│   ├── errors.rs               # CraftFailReason enum
│   └── metrics.rs              # CraftMetrics counters
├── inventory/
│   ├── config.rs               # Update Training loadout (+5 SCRAP)
│   └── item_registry.rs        # Add SCRAP item definition
├── lib.rs                      # Wire CraftSystem to Server
└── session.rs                  # Add last_craft_tick to ServerPlayer

crates/plix-server/tests/
├── crafting_test.rs            # Unit tests for crafting system
└── crafting_integration_test.rs # Integration tests with full server

crates/plix-client/src/
└── console.rs                  # Add /craft command handler
```

**Structure Decision**: Follows existing `weapons/` module pattern with submodules for separation of concerns. New `crafting/` module in plix-server for server-side logic.

## Complexity Tracking

> No violations - table not required.
