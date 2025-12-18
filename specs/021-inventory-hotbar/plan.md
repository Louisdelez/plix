# Implementation Plan: Inventory Hotbar

**Branch**: `021-inventory-hotbar` | **Date**: 2025-12-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/021-inventory-hotbar/spec.md`

## Summary

Add a minimal hotbar-based inventory system to enable item management in all game modes. Players can select slots (1-9 keys or scroll), pick up loot automatically within 1.5 blocks, use items (weapons deal damage, consumables heal, tools place blocks), and drop items on death (mode-dependent). Server-authoritative validation ensures anti-cheat compliance. Three initial items: Sword (25 dmg), Health Pack (+50 HP, stackable to 16), Block Placer (places Stone).

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-common (types, protocol), plix-server (game loop, match state), plix-client (UI), glam (math), bincode (serialization), serde (derive), tokio (async)
**Storage**: N/A (in-memory state only, no persistence in v1)
**Testing**: cargo test for unit/integration tests
**Target Platform**: Linux server + cross-platform client
**Project Type**: Workspace with multiple crates
**Performance Goals**: O(1) hotbar operations, 60fps UI, 1-tick sync latency
**Constraints**: Server-authoritative, no client trust, integrate with AntiCheat rate limiting
**Scale/Scope**: 5-9 hotbar slots per player, 3 item types (v1), all 5 game modes supported

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Justification |
|-----------|--------|---------------|
| I. Security (Server Authority) | ✅ PASS | All inventory ops server-validated (FR-007, FR-011) |
| II. Performance (Low Latency) | ✅ PASS | O(1) ops, event-driven replication, no polling |
| III. Architecture (Engine-First) | ✅ PASS | Modular inventory/ submodule, reuses existing combat/block systems |
| IV. Modding (Extensibility) | ✅ PASS | ItemDef registry allows future items via data |
| V. Code Quality (Tested) | ✅ PASS | Unit tests for hotbar/items, integration tests for full cycle |
| VI. Technical Standards (Stable Rust) | ✅ PASS | No nightly features, standard serde/bincode |
| VII. Player Experience (Multiplayer-First) | ✅ PASS | Designed for multiplayer, single-player = local server |
| VIII. Open Source | ✅ PASS | No proprietary dependencies |
| IX. Scoping (Minimal MVP) | ✅ PASS | Hotbar only, 3 items, no crafting/extended inventory |
| X. Long-Term Vision | ✅ PASS | Extensible ItemDef system, versioned protocol |

**Gate Result**: ✅ All gates pass - proceed to Phase 0

## Project Structure

### Documentation (this feature)

```text
specs/021-inventory-hotbar/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-common/src/
│   ├── inventory/           # NEW: Shared inventory types
│   │   ├── mod.rs           # Module exports
│   │   ├── item.rs          # ItemId, ItemKind, ItemDef
│   │   ├── item_stack.rs    # ItemStack (item_id + quantity)
│   │   └── hotbar.rs        # Hotbar, Slot types
│   ├── protocol/
│   │   └── messages.rs      # ADD: InventoryUpdate, SelectSlot, UseItem messages
│   └── types.rs             # ADD: ItemId, LootEntityId types
│
├── plix-server/src/
│   ├── inventory/           # NEW: Server inventory logic
│   │   ├── mod.rs           # Module exports
│   │   ├── config.rs        # InventoryConfig (hotbar_size, starting_items)
│   │   ├── player_inventory.rs  # Per-player hotbar state
│   │   ├── item_registry.rs # ItemDef registry
│   │   ├── use_system.rs    # Item usage validation + effects
│   │   ├── pickup_system.rs # Loot pickup logic
│   │   └── replication.rs   # Hotbar snapshot/diff
│   ├── loot/                # NEW: Loot entity management
│   │   ├── mod.rs
│   │   ├── entity.rs        # LootEntity struct
│   │   └── spawner.rs       # Loot spawning on death/arena load
│   └── lib.rs               # MODIFY: Add inventory to game loop
│
├── plix-client/src/
│   └── ui/
│       └── hotbar.rs        # NEW: Hotbar UI rendering
│
└── tests/
    └── inventory_test.rs    # Integration tests
```

**Structure Decision**: Follows existing crate workspace pattern. New `inventory/` module in both plix-common (shared types) and plix-server (game logic). Loot entities managed in server-side `loot/` module.

## Complexity Tracking

> No constitution violations requiring justification.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
