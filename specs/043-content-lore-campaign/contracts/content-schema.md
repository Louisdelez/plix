# Content Schema: TOML Format Reference

**Feature**: 043-content-lore-campaign
**Date**: 2025-12-20
**Location**: `assets/content/`

---

## Directory Structure

```
assets/content/
├── chapters/
│   └── *.toml          # Chapter definitions
├── quests/
│   └── *.toml          # Quest definitions
├── mobs/
│   └── *.toml          # Mob definitions
├── dungeons/
│   └── *.toml          # Dungeon definitions
├── spawns/
│   └── *.toml          # Spawn point definitions
├── loot_tables/
│   └── *.toml          # Loot table definitions
└── npcs/
    └── *.toml          # NPC definitions
```

---

## Chapter Schema

**File**: `chapters/<chapter_id>.toml`

```toml
# Example: chapters/the_broken_gate.toml

chapter_id = "the_broken_gate"
title = "The Broken Gate"

intro_text = """
Something stirs beneath the old mine. The Village Elder speaks of dark rituals
and an ancient gate that must remain sealed...
"""

outro_text = """
The Gate Warden is defeated, but the gate remains. What lies beyond?
"""

# Ordered list - player must complete in sequence
mainline_quests = [
    "tutorial_elder",
    "clear_rats",
    "investigate_cultists",
    "find_crypt",
    "defeat_warden"
]

# Optional - can complete in any order
side_quests = [
    "collect_mushrooms"
]
```

---

## Quest Schema

**File**: `quests/<quest_id>.toml`

```toml
# Example: quests/clear_rats.toml

quest_id = "clear_rats"
title = "Pest Control"
description_short = "Clear the cave rats from the mine entrance."
description_long = """
The mine entrance is overrun with cave rats. The miners can't work
until the infestation is cleared out. Hunt down the rats and
report back to the Village Elder.
"""

chapter_id = "the_broken_gate"  # Optional
repeatable = false              # Optional, default false

[prerequisites]
completed_quests = ["tutorial_elder"]  # Optional
min_level = 1                          # Optional
required_items = []                    # Optional

[[steps]]
type = "KillMob"
mob_id = "cave_rat"
count = 5
description = "Kill 5 cave rats"

[[steps]]
type = "TalkToNpc"
npc_id = "village_elder"
description = "Return to the Village Elder"

[rewards]
xp = 100
currency = 50
items = [
    ["iron_sword", 1]
]
unlocks = []
```

### Step Types

```toml
# CollectItem
[[steps]]
type = "CollectItem"
item_id = "mushroom"
count = 10
description = "Collect 10 mushrooms"

# KillMob (last-hit credit only)
[[steps]]
type = "KillMob"
mob_id = "cave_rat"
count = 5
description = "Kill 5 cave rats"

# VisitLocation
[[steps]]
type = "VisitLocation"
region_id = "mine_entrance"
description = "Enter the mine"

# TalkToNpc
[[steps]]
type = "TalkToNpc"
npc_id = "village_elder"
description = "Speak with the Village Elder"

# DungeonClear
[[steps]]
type = "DungeonClear"
dungeon_id = "crypt_of_the_gate"
description = "Complete the Crypt of the Gate"
```

---

## Mob Schema

**File**: `mobs/<mob_id>.toml`

```toml
# Example: mobs/cave_rat.toml

mob_id = "cave_rat"
display_name = "Cave Rat"
loot_table = "cave_rat_drops"
xp_reward = 10
credit_reward = 5
tags = ["beast", "trash"]

[stats]
hp = 20
damage = 5
speed = 4.0          # blocks per second
armor = 0
aggro_radius = 8.0   # blocks
leash_radius = 20.0  # blocks
attack_range = 1.5   # blocks
attack_cooldown = 1.0 # seconds

[behavior]
type = "Aggro"
```

### Behavior Types

```toml
# Basic melee aggro
[behavior]
type = "Aggro"

# Patrol between waypoints
[behavior]
type = "Patrol"
waypoints = [
    [10.0, 5.0, 20.0],
    [15.0, 5.0, 20.0],
    [15.0, 5.0, 25.0]
]

# Ranged attacks
[behavior]
type = "Ranged"
preferred_range = 10.0

# Boss with phases
[behavior]
type = "Boss"

[[behavior.phases]]
hp_threshold = 0.5

[behavior.phases.behavior_modifier]
type = "Enrage"
attack_speed_multiplier = 1.5

[[behavior.phases]]
hp_threshold = 0.25

[behavior.phases.behavior_modifier]
type = "SummonMinions"
mob_id = "cave_rat"
count = 3
```

---

## Loot Table Schema

**File**: `loot_tables/<loot_table_id>.toml`

```toml
# Example: loot_tables/cave_rat_drops.toml

loot_table_id = "cave_rat_drops"

# Always dropped
[[guaranteed]]
item_id = "rat_tail"
quantity = [1, 1]  # [min, max]
chance = 1.0       # Ignored for guaranteed, but required

# Random drops
[[entries]]
item_id = "leather_scrap"
quantity = [1, 2]
chance = 0.3       # 30% chance

[[entries]]
item_id = "gold_coin"
quantity = [1, 5]
chance = 0.1       # 10% chance
```

---

## Spawn Point Schema

**File**: `spawns/<spawn_group>.toml`

```toml
# Example: spawns/overworld_spawns.toml

[[spawns]]
spawn_id = "mine_entrance_rats_1"
mob_id = "cave_rat"
count = 3
respawn_secs = 60.0
radius = 5.0
position = [100.0, 10.0, 50.0]
region_id = "mine_entrance"

[[spawns]]
spawn_id = "mine_entrance_rats_2"
mob_id = "cave_rat"
count = 2
respawn_secs = 90.0
radius = 3.0
position = [110.0, 10.0, 55.0]
region_id = "mine_entrance"

[[spawns]]
spawn_id = "forest_cultists"
mob_id = "cultist"
count = 2
respawn_secs = 120.0
radius = 8.0
position = [200.0, 15.0, 100.0]
region_id = "dark_forest"

# Region limits (optional, per file)
[region_limits]
mine_entrance = 10
dark_forest = 15
```

---

## Dungeon Schema

**File**: `dungeons/<dungeon_id>.toml`

```toml
# Example: dungeons/crypt_of_the_gate.toml

dungeon_id = "crypt_of_the_gate"
display_name = "Crypt of the Gate"
difficulty = 5
entry_location = [300.0, 5.0, 200.0]
boss_mob_id = "gate_warden"
boss_spawn_position = [350.0, 0.0, 250.0]
reward_loot_table = "gate_warden_drops"
boss_respawn_secs = 900.0  # 15 minutes

[[rooms]]
room_id = "entrance"
name = "Entrance Hall"
bounds = { min = [300.0, 0.0, 200.0], max = [320.0, 10.0, 220.0] }

[[rooms.spawns]]
spawn_id = "crypt_entrance_rats"
mob_id = "cave_rat"
count = 2
respawn_secs = 0  # No respawn in dungeons
radius = 3.0
position = [310.0, 1.0, 210.0]
region_id = "crypt_entrance"

[[rooms]]
room_id = "ritual_chamber"
name = "Ritual Chamber"
bounds = { min = [320.0, 0.0, 220.0], max = [350.0, 15.0, 250.0] }

[[rooms.spawns]]
spawn_id = "ritual_cultists"
mob_id = "cultist"
count = 3
respawn_secs = 0
radius = 5.0
position = [335.0, 1.0, 235.0]
region_id = "ritual_chamber"

[[rooms]]
room_id = "boss_room"
name = "The Gate Chamber"
bounds = { min = [350.0, 0.0, 250.0], max = [400.0, 20.0, 300.0] }
# Boss spawned separately via boss_spawn_position
```

---

## NPC Schema

**File**: `npcs/<npc_id>.toml`

```toml
# Example: npcs/village_elder.toml

npc_id = "village_elder"
name = "Elder Theron"
position = [50.0, 10.0, 50.0]
rotation = 0.0  # Facing direction in radians

quest_giver = [
    "tutorial_elder",
    "clear_rats",
    "investigate_cultists",
    "find_crypt",
    "defeat_warden"
]

idle_dialogue = [
    "Greetings, traveler.",
    "The times grow dark...",
    "Have you heard the sounds from the mine?"
]

[quest_offer_dialogue.tutorial_elder]
offer_lines = [
    "Ah, a new face! Welcome to our village.",
    "These are troubling times. Speak with me, and I shall explain."
]
accept_text = "Tell me more"
decline_text = "Maybe later"

[quest_offer_dialogue.clear_rats]
offer_lines = [
    "The mine entrance is overrun with rats!",
    "The miners can't work. Will you help clear them out?"
]
accept_text = "I'll handle it"
decline_text = "Not now"

[quest_complete_dialogue]
tutorial_elder = [
    "Good, you understand the situation.",
    "Now, there is something you can help with..."
]
clear_rats = [
    "Excellent work! The miners can return to work.",
    "Here is your reward. But there is more trouble brewing..."
]
```

---

## Validation Errors

When content validation fails, expect these error formats:

```
ERROR [content::validator] Duplicate ID: mob_id "cave_rat" defined in both mobs/cave_rat.toml and mobs/cave_rat_v2.toml
ERROR [content::validator] Invalid reference: quest "clear_rats" references unknown mob_id "cave_rat_typo"
ERROR [content::validator] Missing required field: loot_tables/test.toml missing "loot_table_id"
WARN  [content::validator] Value out of range: mob "overpowered" has hp=0, must be > 0
```

**Dev mode**: Process terminates with exit code 1
**Prod mode**: Invalid content skipped, warnings logged, server continues
