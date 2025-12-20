# Implementation Plan: Content / Lore / Campaign (Adventure Mode)

**Branch**: `043-content-lore-campaign` | **Date**: 2025-12-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/043-content-lore-campaign/spec.md`

## Summary

Implement a data-driven Adventure Mode MVP comprising:
- **Quest System**: 5 step types (CollectItem, KillMob, VisitLocation, TalkToNpc, DungeonClear), server-authoritative progression, CEF-based Quest Log/Tracker HUD
- **Mob System**: Data-driven definitions, simple AI (aggro/leash/closest-player targeting), free-for-all loot, damage-proportional XP/credits with last-hit bonus
- **Dungeon System**: Shared-world prefab dungeon with boss, reward chest, completion events
- **Campaign Content**: 1 chapter ("The Broken Gate"), 3-5 quests, 3 mob types, 1 dungeon, 1 NPC quest-giver with simple dialogue
- **Content Validation**: Dev fail-fast, production skip-with-warning

## Technical Context

**Language/Version**: Rust 1.83 (stable, per workspace `rust-version`)
**Primary Dependencies**:
- `plix-common` (types, protocol, inventory, combat)
- `plix-server` (game loop, session, match state, mods integration)
- `plix-client` (CEF UI, HUD, console commands)
- `plix-arena` (spawn points, world loading)
- `plix-mod-core` (event system for mod integration)
- `serde` + `toml` (content serialization)
- `tracing` (structured logging)

**Storage**:
- Content definitions: TOML files in `assets/content/`
- Quest progress persistence: Via existing `plix-server::persist` (Feature 014)
- Runtime state: In-memory (server-authoritative)

**Testing**: `cargo test` (unit + integration)
**Target Platform**: Linux server (headless), Windows/Linux client (wgpu + CEF)
**Project Type**: Workspace multi-crate (existing structure)
**Performance Goals**:
- 10+ concurrent players engaging mobs without tick degradation
- Mob aggro response < 0.5s
- Content loading < 2s on server start

**Constraints**:
- Server-authoritative for all quest/mob/loot state
- TOML for content (human-readable, mod-friendly)
- CEF for UI (existing shell from Feature 030+)

**Scale/Scope**:
- MVP: 1 chapter, 5 quests, 3 mobs, 1 dungeon, 1 NPC
- Extensible to unlimited content via data files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | ✅ PASS | All quest/mob/loot state server-authoritative per FR-003, FR-004, FR-016 |
| II. Performance (Tick Stability) | ✅ PASS | Mob AI uses event-driven updates; spawn backpressure prevents tick overload (FR-015) |
| III. Architecture (Engine-First) | ✅ PASS | Content system uses engine primitives (events, entities); mods use provided hooks |
| IV. Modding (First-Class) | ✅ PASS | Data-only mods supported (TOML defs); events exposed per FR-035 |
| V. Code Quality (Tested) | ✅ PASS | Unit + integration tests required per DoD |
| VI. Technical Standards (Stable Rust) | ✅ PASS | Rust 1.83 stable; cargo clippy/fmt enforced |
| VII. Player Experience | ✅ PASS | Quest UI integrated via CEF; zero manual mod install for content |
| VIII. Open Source | ✅ PASS | All content formats documented; no proprietary lock-in |
| IX. Scoping (Minimal MVP) | ✅ PASS | MVP scoped to 1 chapter, 5 quests, 3 mobs, 1 dungeon |
| X. Long-Term Vision | ✅ PASS | Data-driven design ensures content extensibility without code changes |

**Gate Result**: PASS - Proceed to Phase 0

## Project Structure

### Documentation (this feature)

```text
specs/043-content-lore-campaign/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── quest-events.md
│   ├── mob-events.md
│   └── content-schema.md
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-common/
│   └── src/
│       ├── content/           # NEW: Content type definitions
│       │   ├── mod.rs
│       │   ├── quest.rs       # QuestDefinition, QuestStep, QuestProgress
│       │   ├── mob.rs         # MobDefinition, MobStats, MobBehavior
│       │   ├── dungeon.rs     # DungeonDefinition, RoomTemplate
│       │   ├── npc.rs         # NpcDefinition, DialogueLine
│       │   ├── loot.rs        # LootTable, LootEntry
│       │   ├── chapter.rs     # ChapterDefinition
│       │   └── spawn.rs       # SpawnPointDefinition
│       └── protocol/
│           └── messages.rs    # EXTEND: Quest/Mob/Dungeon protocol messages
│
├── plix-server/
│   └── src/
│       ├── content/           # NEW: Content loading & validation
│       │   ├── mod.rs
│       │   ├── loader.rs      # ContentLoader (TOML parsing)
│       │   └── validator.rs   # ContentValidator (dev/prod modes)
│       ├── quest/             # NEW: Quest system
│       │   ├── mod.rs
│       │   ├── system.rs      # QuestSystem (event handling, progression)
│       │   ├── progress.rs    # PlayerQuestProgress (persistence)
│       │   └── events.rs      # QuestEvent enum
│       ├── mob/               # NEW: Mob system
│       │   ├── mod.rs
│       │   ├── system.rs      # MobSystem (AI tick, spawning)
│       │   ├── ai.rs          # MobAI (aggro, pathfinding, leash)
│       │   ├── damage.rs      # DamageTracker (per-mob contributor tracking)
│       │   └── payout.rs      # XP/credit distribution logic
│       ├── dungeon/           # NEW: Dungeon system
│       │   ├── mod.rs
│       │   ├── system.rs      # DungeonSystem (state tracking)
│       │   └── chest.rs       # RewardChest logic
│       └── npc/               # NEW: NPC system
│           ├── mod.rs
│           └── dialogue.rs    # DialogueSystem
│
├── plix-client/
│   └── src/
│       ├── ui_cef/
│       │   ├── quest/         # NEW: Quest UI
│       │   │   ├── mod.rs
│       │   │   ├── log.rs     # QuestLog component bridge
│       │   │   └── tracker.rs # QuestTracker HUD bridge
│       │   ├── dialogue/      # NEW: NPC dialogue
│       │   │   └── mod.rs     # DialoguePanel bridge
│       │   └── dungeon/       # NEW: Dungeon HUD
│       │       └── mod.rs     # DungeonObjective bridge
│       └── console.rs         # EXTEND: /quest debug commands
│
└── assets/
    └── content/               # NEW: Data-driven content
        ├── chapters/
        │   └── the_broken_gate.toml
        ├── quests/
        │   ├── tutorial_elder.toml
        │   ├── clear_rats.toml
        │   ├── investigate_cultists.toml
        │   ├── find_crypt.toml
        │   └── defeat_warden.toml
        ├── mobs/
        │   ├── cave_rat.toml
        │   ├── cultist.toml
        │   └── gate_warden.toml
        ├── dungeons/
        │   └── crypt_of_the_gate.toml
        ├── spawns/
        │   └── overworld_spawns.toml
        ├── loot_tables/
        │   ├── cave_rat_drops.toml
        │   ├── cultist_drops.toml
        │   └── gate_warden_drops.toml
        └── npcs/
            └── village_elder.toml

assets/ui/
└── pages/
    ├── quest_log.html         # NEW: Quest log page
    ├── quest_log.js
    ├── dialogue.html          # NEW: NPC dialogue panel
    ├── dialogue.js
    └── dungeon_hud.html       # NEW: Dungeon objective overlay
```

**Structure Decision**: Extend existing multi-crate workspace. New modules under `plix-common` (shared types), `plix-server` (authoritative systems), `plix-client` (UI bridges). Content in `assets/content/` with TOML format.

## Complexity Tracking

> No constitution violations. Feature is appropriately scoped for MVP.

| Aspect | Justification |
|--------|---------------|
| 4 new server modules (quest/mob/dungeon/npc) | Each represents a distinct domain; separation aligns with Constitution III (layer separation) |
| TOML content format | Chosen over JSON for readability; Constitution IV requires mod-friendly formats |
| Damage tracking per mob | Required for proportional XP/credits (clarification Q3); minimal complexity |

## Locked Decisions (from Clarifications)

These decisions are **final** and must not be revisited:

1. **Dungeons**: Shared world model - single location, global boss state
2. **Loot**: Free-for-all - single drop per kill, first pickup wins
3. **Quest Kill Credit**: Last-hit only for KillMob progression
4. **XP/Credits**: Proportional to damage dealt + last-hit bonus (10-25%)
5. **Anti-Abuse**: Minimum 5% damage OR hit within 10s to receive XP/credits
6. **Content Validation**: Dev = fail-fast, Production = skip-with-warning
7. **Mob Targeting**: Always target closest player in aggro range
