# Data Model: Content / Lore / Campaign (Adventure Mode)

**Feature**: 043-content-lore-campaign
**Date**: 2025-12-20
**Location**: `crates/plix-common/src/content/`

---

## Entity Overview

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Chapter   │────>│    Quest    │────>│  QuestStep  │
└─────────────┘     └─────────────┘     └─────────────┘
                           │
                           v
                    ┌─────────────┐
                    │   Reward    │
                    └─────────────┘

┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│     Mob     │────>│  LootTable  │────>│  LootEntry  │
└─────────────┘     └─────────────┘     └─────────────┘
       │
       v
┌─────────────┐
│ SpawnPoint  │
└─────────────┘

┌─────────────┐     ┌─────────────┐
│   Dungeon   │────>│ RoomDef     │
└─────────────┘     └─────────────┘
       │
       v
┌─────────────┐
│  RewardDef  │
└─────────────┘

┌─────────────┐     ┌─────────────┐
│     NPC     │────>│ DialogueLine│
└─────────────┘     └─────────────┘
```

---

## Core Entities

### Chapter

Narrative container grouping related quests.

```rust
// crates/plix-common/src/content/chapter.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterDefinition {
    /// Unique stable identifier (e.g., "the_broken_gate")
    pub chapter_id: ChapterId,

    /// Display title
    pub title: String,

    /// Introduction text shown when chapter begins
    pub intro_text: String,

    /// Outro text shown when all mainline quests complete
    pub outro_text: String,

    /// Ordered list of mainline quest IDs (must complete in order)
    pub mainline_quests: Vec<QuestId>,

    /// Optional side quests (can complete in any order)
    #[serde(default)]
    pub side_quests: Vec<QuestId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChapterId(pub String);
```

### Quest

Player objective with ordered steps.

```rust
// crates/plix-common/src/content/quest.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestDefinition {
    /// Unique stable identifier (e.g., "clear_rats")
    pub quest_id: QuestId,

    /// Display title
    pub title: String,

    /// Short description for list view
    pub description_short: String,

    /// Long description with lore
    pub description_long: String,

    /// Associated chapter (optional)
    #[serde(default)]
    pub chapter_id: Option<ChapterId>,

    /// Prerequisites to accept this quest
    #[serde(default)]
    pub prerequisites: QuestPrerequisites,

    /// Ordered steps to complete
    pub steps: Vec<QuestStep>,

    /// Rewards on completion
    pub rewards: QuestRewards,

    /// Whether quest can be repeated after completion
    #[serde(default)]
    pub repeatable: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestPrerequisites {
    /// Quests that must be completed first
    #[serde(default)]
    pub completed_quests: Vec<QuestId>,

    /// Minimum player level (if leveling exists)
    #[serde(default)]
    pub min_level: Option<u32>,

    /// Required items in inventory
    #[serde(default)]
    pub required_items: Vec<ItemId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestId(pub String);
```

### QuestStep

Individual objective within a quest.

```rust
// crates/plix-common/src/content/quest.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum QuestStep {
    /// Collect X of item_id
    CollectItem {
        item_id: ItemId,
        count: u32,
        description: String,
    },

    /// Kill X of mob_id (last-hit credit only)
    KillMob {
        mob_id: MobDefId,
        count: u32,
        description: String,
    },

    /// Enter a specific region/zone
    VisitLocation {
        region_id: RegionId,
        description: String,
    },

    /// Interact with an NPC
    TalkToNpc {
        npc_id: NpcId,
        description: String,
    },

    /// Complete a dungeon (boss killed + chest looted)
    DungeonClear {
        dungeon_id: DungeonId,
        description: String,
    },
}

impl QuestStep {
    pub fn description(&self) -> &str {
        match self {
            Self::CollectItem { description, .. } => description,
            Self::KillMob { description, .. } => description,
            Self::VisitLocation { description, .. } => description,
            Self::TalkToNpc { description, .. } => description,
            Self::DungeonClear { description, .. } => description,
        }
    }
}
```

### QuestRewards

Rewards granted on quest completion.

```rust
// crates/plix-common/src/content/quest.rs

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestRewards {
    /// XP awarded
    #[serde(default)]
    pub xp: u32,

    /// Items awarded (item_id, count)
    #[serde(default)]
    pub items: Vec<(ItemId, u32)>,

    /// Currency awarded
    #[serde(default)]
    pub currency: u32,

    /// Unlocks (e.g., new areas, features)
    #[serde(default)]
    pub unlocks: Vec<String>,
}
```

### Mob

Enemy entity definition.

```rust
// crates/plix-common/src/content/mob.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobDefinition {
    /// Unique stable identifier (e.g., "cave_rat")
    pub mob_id: MobDefId,

    /// Display name
    pub display_name: String,

    /// Combat stats
    pub stats: MobStats,

    /// AI behavior type
    pub behavior: MobBehavior,

    /// Loot table ID for drops
    pub loot_table: LootTableId,

    /// Base XP reward (distributed proportionally)
    pub xp_reward: u32,

    /// Base credit reward (distributed proportionally)
    #[serde(default)]
    pub credit_reward: u32,

    /// Tags for filtering/grouping
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobStats {
    pub hp: u32,
    pub damage: u32,
    pub speed: f32,        // blocks per second
    pub armor: u32,
    pub aggro_radius: f32, // blocks
    pub leash_radius: f32, // blocks (return to spawn if exceeded)
    pub attack_range: f32, // blocks
    pub attack_cooldown: f32, // seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MobBehavior {
    /// Basic melee aggro (target closest player)
    Aggro,

    /// Patrol between waypoints, aggro on proximity
    Patrol { waypoints: Vec<Vec3> },

    /// Ranged attacks, maintain distance
    Ranged { preferred_range: f32 },

    /// Boss with phase transitions
    Boss { phases: Vec<BossPhase> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossPhase {
    /// HP percentage threshold to enter this phase (e.g., 0.5 for 50%)
    pub hp_threshold: f32,

    /// Modified behavior in this phase
    pub behavior_modifier: BossPhaseModifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BossPhaseModifier {
    /// Increase attack speed
    Enrage { attack_speed_multiplier: f32 },

    /// Spawn additional mobs
    SummonMinions { mob_id: MobDefId, count: u32 },

    /// Area attack
    AreaAttack { radius: f32, damage: u32, cooldown: f32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MobDefId(pub String);
```

### LootTable

Drop definitions for mobs and chests.

```rust
// crates/plix-common/src/content/loot.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootTable {
    /// Unique stable identifier
    pub loot_table_id: LootTableId,

    /// List of possible drops
    pub entries: Vec<LootEntry>,

    /// Guaranteed drops (always included)
    #[serde(default)]
    pub guaranteed: Vec<LootEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootEntry {
    /// Item to drop
    pub item_id: ItemId,

    /// Quantity range (min, max) - random within range
    pub quantity: (u32, u32),

    /// Drop chance (0.0 to 1.0)
    pub chance: f32,
}

impl LootTable {
    /// Roll loot with optional seed for deterministic testing
    pub fn roll(&self, seed: Option<u64>) -> Vec<(ItemId, u32)> {
        let mut rng = match seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };

        let mut drops = Vec::new();

        // Add guaranteed drops
        for entry in &self.guaranteed {
            let qty = rng.gen_range(entry.quantity.0..=entry.quantity.1);
            drops.push((entry.item_id.clone(), qty));
        }

        // Roll random drops
        for entry in &self.entries {
            if rng.gen::<f32>() < entry.chance {
                let qty = rng.gen_range(entry.quantity.0..=entry.quantity.1);
                drops.push((entry.item_id.clone(), qty));
            }
        }

        drops
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LootTableId(pub String);
```

### SpawnPoint

Mob spawn location definition.

```rust
// crates/plix-common/src/content/spawn.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPointDefinition {
    /// Unique stable identifier
    pub spawn_id: SpawnId,

    /// Mob type to spawn
    pub mob_id: MobDefId,

    /// Maximum simultaneous spawns from this point
    pub count: u32,

    /// Respawn delay in seconds
    pub respawn_secs: f32,

    /// Spawn radius around center position
    pub radius: f32,

    /// Center position in world coordinates
    pub position: Vec3,

    /// Region this spawn belongs to (for limits)
    pub region_id: RegionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpawnId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegionId(pub String);
```

### Dungeon

Dungeon definition.

```rust
// crates/plix-common/src/content/dungeon.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonDefinition {
    /// Unique stable identifier
    pub dungeon_id: DungeonId,

    /// Display name
    pub display_name: String,

    /// Suggested level/difficulty
    pub difficulty: u32,

    /// Entry portal location in world
    pub entry_location: Vec3,

    /// Room definitions (ordered progression)
    pub rooms: Vec<RoomDefinition>,

    /// Boss mob ID
    pub boss_mob_id: MobDefId,

    /// Boss spawn location (relative to dungeon)
    pub boss_spawn_position: Vec3,

    /// Reward chest loot table
    pub reward_loot_table: LootTableId,

    /// Boss respawn timer in seconds (shared world)
    pub boss_respawn_secs: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomDefinition {
    /// Room identifier within dungeon
    pub room_id: String,

    /// Display name
    pub name: String,

    /// Spawn points within this room
    pub spawns: Vec<SpawnPointDefinition>,

    /// Bounds of this room (for tracking player location)
    pub bounds: AABB,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DungeonId(pub String);
```

### NPC

Non-player character with dialogue.

```rust
// crates/plix-common/src/content/npc.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcDefinition {
    /// Unique stable identifier
    pub npc_id: NpcId,

    /// Display name
    pub name: String,

    /// Position in world
    pub position: Vec3,

    /// Rotation (facing direction)
    pub rotation: f32,

    /// Quests this NPC can give
    #[serde(default)]
    pub quest_giver: Vec<QuestId>,

    /// Default dialogue lines (when no quest available)
    #[serde(default)]
    pub idle_dialogue: Vec<String>,

    /// Dialogue for offering quests
    #[serde(default)]
    pub quest_offer_dialogue: HashMap<QuestId, QuestDialogue>,

    /// Dialogue for completing quests
    #[serde(default)]
    pub quest_complete_dialogue: HashMap<QuestId, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestDialogue {
    /// Lines to show when offering quest
    pub offer_lines: Vec<String>,

    /// Text for accept button
    #[serde(default = "default_accept_text")]
    pub accept_text: String,

    /// Text for decline button
    #[serde(default = "default_decline_text")]
    pub decline_text: String,
}

fn default_accept_text() -> String { "Accept".to_string() }
fn default_decline_text() -> String { "Decline".to_string() }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NpcId(pub String);
```

---

## Runtime State Entities

These are server-side runtime entities, not persisted in TOML.

### MobInstance

Active mob in the world.

```rust
// crates/plix-server/src/mob/system.rs

pub struct MobInstance {
    /// Unique runtime ID
    pub instance_id: MobInstanceId,

    /// Definition reference
    pub definition_id: MobDefId,

    /// Current position
    pub position: Vec3,

    /// Current HP
    pub hp: u32,

    /// AI state
    pub ai_state: MobAiState,

    /// Spawn point origin (for leash)
    pub spawn_origin: Vec3,

    /// Damage tracking for XP/credit distribution
    pub damage_tracker: DamageTracker,
}

pub enum MobAiState {
    Idle,
    Aggro { target: PlayerId },
    Attacking { target: PlayerId, cooldown_remaining: f32 },
    Returning { destination: Vec3 },
}

pub struct DamageTracker {
    pub total_damage: u32,
    pub contributions: Vec<DamageContribution>,
}

pub struct DamageContribution {
    pub player_id: PlayerId,
    pub damage: u32,
    pub last_hit_tick: Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MobInstanceId(pub u64);
```

### QuestProgress (Per-Player)

Player quest state.

```rust
// crates/plix-server/src/quest/progress.rs

pub struct PlayerQuestProgress {
    pub player_id: PlayerId,
    pub active_quests: HashMap<QuestId, ActiveQuestState>,
    pub completed_quests: HashSet<QuestId>,
    pub pinned_quest: Option<QuestId>,
}

pub struct ActiveQuestState {
    pub quest_id: QuestId,
    pub current_step_index: usize,
    pub step_progress: StepProgress,
}

pub enum StepProgress {
    CollectItem { current: u32, required: u32 },
    KillMob { current: u32, required: u32 },
    VisitLocation { visited: bool },
    TalkToNpc { talked: bool },
    DungeonClear { cleared: bool },
}
```

### DungeonState (Shared World)

Global dungeon state.

```rust
// crates/plix-server/src/dungeon/system.rs

pub struct DungeonState {
    pub dungeon_id: DungeonId,
    pub boss_alive: bool,
    pub boss_instance_id: Option<MobInstanceId>,
    pub chest_available: bool,
    pub last_boss_kill_tick: Option<Tick>,
    /// Players who have cleared since last boss respawn
    pub cleared_by: HashSet<PlayerId>,
}
```

---

## Validation Rules

### Uniqueness
- All `*_id` fields must be unique within their type
- Cross-file uniqueness enforced by ContentValidator

### Reference Integrity
| Field | Must Reference |
|-------|----------------|
| `QuestDefinition.chapter_id` | Valid `ChapterId` |
| `QuestStep::KillMob.mob_id` | Valid `MobDefId` |
| `QuestStep::DungeonClear.dungeon_id` | Valid `DungeonId` |
| `QuestStep::TalkToNpc.npc_id` | Valid `NpcId` |
| `MobDefinition.loot_table` | Valid `LootTableId` |
| `LootEntry.item_id` | Valid `ItemId` (from inventory system) |
| `SpawnPointDefinition.mob_id` | Valid `MobDefId` |
| `DungeonDefinition.boss_mob_id` | Valid `MobDefId` |
| `DungeonDefinition.reward_loot_table` | Valid `LootTableId` |
| `NpcDefinition.quest_giver` | Valid `QuestId` |

### Value Constraints
| Field | Constraint |
|-------|------------|
| `MobStats.hp` | > 0 |
| `MobStats.damage` | >= 0 |
| `MobStats.speed` | > 0 |
| `MobStats.aggro_radius` | > 0 |
| `MobStats.leash_radius` | >= aggro_radius |
| `SpawnPointDefinition.count` | >= 1 |
| `SpawnPointDefinition.respawn_secs` | > 0 |
| `LootEntry.chance` | 0.0 to 1.0 |
| `LootEntry.quantity` | min <= max, min >= 1 |
| `BossPhase.hp_threshold` | 0.0 to 1.0, descending order |
