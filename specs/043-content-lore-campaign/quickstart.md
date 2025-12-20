# Quickstart: Content / Lore / Campaign (Adventure Mode)

**Feature**: 043-content-lore-campaign
**Date**: 2025-12-20

---

## Prerequisites

- Rust 1.83+ (stable)
- Existing plix workspace builds successfully
- Features 014 (persistence), 021 (inventory), 030 (CEF UI) implemented

---

## Build & Test

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Check formatting and lints
cargo fmt --check
cargo clippy
```

---

## Creating Content

### 1. Add a New Mob

Create `assets/content/mobs/skeleton.toml`:

```toml
mob_id = "skeleton"
display_name = "Skeleton Warrior"
loot_table = "skeleton_drops"
xp_reward = 25
credit_reward = 10
tags = ["undead", "dungeon"]

[stats]
hp = 50
damage = 12
speed = 3.0
armor = 5
aggro_radius = 10.0
leash_radius = 25.0
attack_range = 2.0
attack_cooldown = 1.5

[behavior]
type = "Aggro"
```

### 2. Add a Loot Table

Create `assets/content/loot_tables/skeleton_drops.toml`:

```toml
loot_table_id = "skeleton_drops"

[[guaranteed]]
item_id = "bone"
quantity = [1, 2]
chance = 1.0

[[entries]]
item_id = "rusty_sword"
quantity = [1, 1]
chance = 0.1

[[entries]]
item_id = "gold_coin"
quantity = [5, 15]
chance = 0.25
```

### 3. Add a Spawn Point

Edit `assets/content/spawns/overworld_spawns.toml`:

```toml
[[spawns]]
spawn_id = "graveyard_skeletons"
mob_id = "skeleton"
count = 4
respawn_secs = 120.0
radius = 8.0
position = [250.0, 10.0, 150.0]
region_id = "graveyard"
```

### 4. Add a Quest

Create `assets/content/quests/clear_skeletons.toml`:

```toml
quest_id = "clear_skeletons"
title = "Bone Collector"
description_short = "Defeat the skeletons in the graveyard."
description_long = """
The old graveyard has become overrun with undead.
Clear out the skeletons and collect proof of your deeds.
"""

repeatable = false

[prerequisites]
completed_quests = ["clear_rats"]

[[steps]]
type = "KillMob"
mob_id = "skeleton"
count = 10
description = "Defeat 10 skeletons"

[[steps]]
type = "CollectItem"
item_id = "bone"
count = 5
description = "Collect 5 bones as proof"

[[steps]]
type = "TalkToNpc"
npc_id = "village_elder"
description = "Return to the Village Elder"

[rewards]
xp = 200
currency = 100
items = [["steel_sword", 1]]
```

---

## Validate Content

```bash
# Run content validator (dev mode - will fail on errors)
cargo run -p plix-server -- --validate-content

# Run content validator (prod mode - logs warnings, continues)
cargo run -p plix-server -- --validate-content --prod
```

---

## Debug Commands

In-game console commands for testing:

```
/quest list              - List all active/completed quests
/quest start <id>        - Force-start a quest (skip prerequisites)
/quest complete <id>     - Force-complete a quest
/quest step <id>         - Complete current step
/quest reset             - Reset all quest progress

/mob spawn <mob_id>      - Spawn a mob at player position
/mob kill                - Kill targeted mob

/dungeon reset <id>      - Reset dungeon state (respawn boss)
/dungeon complete <id>   - Force-complete dungeon
```

---

## Architecture Overview

### Server-Side (Authoritative)

```
plix-server/src/
├── content/
│   ├── loader.rs       # Load TOML files from assets/content/
│   └── validator.rs    # Validate references and constraints
├── quest/
│   ├── system.rs       # QuestSystem - event handling, progression
│   ├── progress.rs     # PlayerQuestProgress - per-player state
│   └── events.rs       # Quest event types
├── mob/
│   ├── system.rs       # MobSystem - AI tick, spawning
│   ├── ai.rs           # FSM: Idle → Aggro → Attack → Return
│   ├── damage.rs       # DamageTracker per mob
│   └── payout.rs       # XP/credit distribution
├── dungeon/
│   ├── system.rs       # DungeonSystem - state tracking
│   └── chest.rs        # Reward chest logic
└── npc/
    └── dialogue.rs     # NPC interaction handling
```

### Client-Side (UI)

```
plix-client/src/ui_cef/
├── quest/
│   ├── log.rs          # QuestLog CEF bridge
│   └── tracker.rs      # QuestTracker HUD bridge
├── dialogue/
│   └── mod.rs          # NPC dialogue panel bridge
└── dungeon/
    └── mod.rs          # Dungeon objective HUD bridge

assets/ui/pages/
├── quest_log.html/js   # Quest log interface
├── dialogue.html/js    # NPC dialogue panel
└── dungeon_hud.html/js # Dungeon objective overlay
```

### Content Data

```
assets/content/
├── chapters/           # Campaign chapters
├── quests/             # Quest definitions
├── mobs/               # Mob definitions
├── dungeons/           # Dungeon definitions
├── spawns/             # Spawn point definitions
├── loot_tables/        # Loot tables
└── npcs/               # NPC definitions
```

---

## Key Decisions (Locked)

| Decision | Choice | Reference |
|----------|--------|-----------|
| Dungeon model | Shared world, global boss state | Clarification Q1 |
| Loot distribution | Free-for-all (first pickup wins) | Clarification Q2 |
| Quest kill credit | Last-hit only | Clarification Q3 |
| XP/credit distribution | Proportional to damage + killer bonus | Clarification Q3 |
| Anti-abuse threshold | ≥5% damage OR hit within 10s | Clarification Q3 |
| Content validation | Dev: fail-fast, Prod: skip-with-warning | Clarification Q4 |
| Mob targeting | Closest player in aggro range | Clarification Q5 |

---

## Testing Strategy

### Unit Tests

```bash
# Content parsing
cargo test -p plix-common content::

# Quest progression
cargo test -p plix-server quest::

# Loot rolls (deterministic)
cargo test -p plix-server mob::payout

# Content validation
cargo test -p plix-server content::validator
```

### Integration Tests

```bash
# Full quest flow
cargo test -p plix-server --test quest_integration

# Mob kill → quest progress
cargo test -p plix-server --test mob_quest_integration

# Dungeon completion
cargo test -p plix-server --test dungeon_integration
```

---

## Next Steps

1. Run `/speckit.tasks` to generate implementation tasks
2. Implement content types in `plix-common/src/content/`
3. Implement server systems in `plix-server/src/{quest,mob,dungeon,npc}/`
4. Add CEF UI pages and bridges
5. Create MVP campaign content
6. Write tests
7. Document in `docs/feature-043.md`
