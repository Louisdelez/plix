# Feature 043: Content / Lore / Campaign (Adventure Mode)

This document describes the content authoring system for Plix, enabling designers to create quests, mobs, dungeons, and NPCs using TOML data files.

## Overview

The content system provides a data-driven approach to game content. All game entities are defined in TOML files under `assets/content/`, which are loaded and validated at server startup.

## Directory Structure

```
assets/content/
├── chapters/           # Campaign chapter definitions
│   └── the_broken_gate.toml
├── quests/             # Quest definitions
│   ├── tutorial_elder.toml
│   ├── clear_rats.toml
│   └── ...
├── mobs/               # Enemy mob definitions
│   ├── cave_rat.toml
│   ├── cultist.toml
│   └── gate_warden.toml
├── loot_tables/        # Drop tables for mobs and chests
│   ├── cave_rat_drops.toml
│   └── ...
├── spawns/             # Spawn point collections
│   └── overworld_spawns.toml
├── dungeons/           # Dungeon definitions
│   └── crypt_of_the_gate.toml
└── npcs/               # NPC definitions
    └── village_elder.toml
```

## Content Types

### Chapters

Chapters group quests into narrative arcs with intro/outro text.

```toml
chapter_id = "the_broken_gate"
title = "The Broken Gate"
intro_text = "Something stirs beneath..."
outro_text = "The village is saved!"
mainline_quests = ["tutorial_elder", "clear_rats", "defeat_warden"]
side_quests = ["collect_mushrooms"]
```

### Quests

Quests define player objectives with steps, prerequisites, and rewards.

```toml
quest_id = "clear_rats"
title = "Pest Control"
description_short = "Clear the cave rats"
description_long = "Detailed description..."
chapter_id = "the_broken_gate"
repeatable = false

[prerequisites]
completed_quests = ["tutorial_elder"]

[[steps]]
type = "KillMob"
mob_id = "cave_rat"
count = 5
description = "Kill 5 cave rats"

[[steps]]
type = "TalkToNpc"
npc_id = "village_elder"
description = "Return to Elder Theron"

[rewards]
xp = 100
currency = 50
```

#### Step Types

- `KillMob`: Kill X of a specific mob type (last-hit credit)
- `CollectItem`: Collect X of an item
- `VisitLocation`: Enter a specific region
- `TalkToNpc`: Interact with an NPC
- `DungeonClear`: Complete a dungeon (defeat boss + loot chest)

### Mobs

Mobs define enemy entities with stats and AI behavior.

```toml
mob_id = "cave_rat"
display_name = "Cave Rat"
loot_table = "cave_rat_drops"
xp_reward = 10
credit_reward = 5
tags = ["beast", "trash"]

[stats]
hp = 20
damage = 5
speed = 4.0
armor = 0
aggro_radius = 8.0
leash_radius = 20.0
attack_range = 1.5
attack_cooldown = 1.0

[behavior]
type = "Aggro"
```

#### Behavior Types

- `Aggro`: Basic melee, target closest player
- `Patrol`: Patrol waypoints, aggro on proximity
- `Ranged`: Maintain preferred distance
- `Boss`: HP-based phase transitions

### Loot Tables

Define drop rates for items.

```toml
loot_table_id = "cave_rat_drops"

[[guaranteed]]
item_id = { raw = 1 }
quantity = [1, 1]
chance = 1.0

[[entries]]
item_id = { raw = 10 }
quantity = [1, 1]
chance = 0.15
```

### Spawn Points

Define where mobs spawn in the world.

```toml
[region_limits]
mine_entrance = 8

[[spawns]]
spawn_id = "mine_rats_1"
mob_id = "cave_rat"
count = 3
respawn_secs = 60.0
radius = 5.0
position = [100.0, 10.0, 50.0]
region_id = "mine_entrance"
```

### Dungeons

Define instanced areas with bosses and rewards.

```toml
dungeon_id = "crypt_of_the_gate"
display_name = "Crypt of the Gate"
difficulty = 5
entry_location = [300.0, 5.0, 200.0]
boss_mob_id = "gate_warden"
boss_spawn_position = [375.0, 1.0, 275.0]
reward_loot_table = "gate_warden_drops"
boss_respawn_secs = 900.0

[[rooms]]
room_id = "entrance"
name = "Entrance Hall"
[rooms.bounds]
min = [300.0, 0.0, 200.0]
max = [320.0, 10.0, 220.0]
```

### NPCs

Define quest givers with dialogue.

```toml
npc_id = "village_elder"
name = "Elder Theron"
position = [50.0, 10.0, 50.0]
quest_giver = ["clear_rats", "defeat_warden"]
idle_dialogue = ["Greetings, traveler."]

[quest_offer_dialogue.clear_rats]
offer_lines = ["The mine is overrun with rats!"]
accept_text = "I'll handle it"
decline_text = "Not now"

[quest_complete_dialogue]
clear_rats = ["Excellent work!"]
```

## Validation

The server validates all content at startup:

- **Reference checking**: All quest/mob/loot IDs must exist
- **Constraint validation**: Stats must be positive, probabilities 0-1
- **Dev mode**: Fails fast on any error
- **Prod mode**: Logs warnings and skips invalid content

Run validation manually:
```bash
plix-server --validate-content
```

## Debug Commands

Console commands for testing:

- `/quest list` - List all quests
- `/quest start <id>` - Start a quest
- `/quest complete <id>` - Complete a quest step
- `/quest reset` - Reset all quest progress
- `/dungeon list` - List all dungeons
- `/dungeon reset <id>` - Reset dungeon state

## Mod Events

Content systems emit mod events for extensibility:

- `ModEvent::Quest(Started | StepCompleted | Completed | Abandoned)`
- `ModEvent::Mob(Spawned | Damaged | Killed | LootDropped)`
- `ModEvent::Dungeon(Entered | BossKilled | ChestOpened | Completed)`

## MVP Campaign

The included "The Broken Gate" chapter provides a complete playable experience:

1. **Tutorial Elder**: Introduction to NPC dialogue
2. **Clear Rats**: Combat tutorial (kill 5 cave rats)
3. **Investigate Cultists**: Mid-tier enemies (3 cultists)
4. **Find Crypt**: Location-based objective
5. **Defeat Warden**: Boss dungeon (Gate Warden)

## Performance

- Content loading: ~50ms for 100 TOML files
- Mob tick: O(n) where n = active mobs
- Quest check: O(active_quests * active_steps)
- Recommended: <500 active mobs, <50 spawn points per region
