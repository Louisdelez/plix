# Mob & Dungeon Events Contract

**Feature**: 043-content-lore-campaign
**Date**: 2025-12-20
**Module**: `plix-server::mob`, `plix-server::dungeon`, `plix-common::protocol::messages`

---

## Overview

Mob and dungeon events handle combat, loot, XP/credit distribution, and dungeon state.

```
Server (MobSystem / DungeonSystem)
    │
    ├──> Client Protocol (combat feedback, HUD)
    │       └── ServerMessage::Mob* / Dungeon*
    │
    ├──> QuestSystem (kill credit, dungeon clear)
    │
    └──> Mod API (hooks)
            └── ModEvent::Mob* / Dungeon*
```

---

## Protocol Messages (Server → Client)

### Mob Messages

```rust
// crates/plix-common/src/protocol/messages.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    // ... existing variants ...

    /// Mob spawned in view
    MobSpawned(MobSpawnPayload),

    /// Mob state update (position, HP, target)
    MobUpdate(MobUpdatePayload),

    /// Mob took damage
    MobDamaged(MobDamagedPayload),

    /// Mob died
    MobDied(MobDiedPayload),

    /// Loot dropped in world
    LootDropped(LootDropPayload),

    /// XP/credit reward notification
    RewardGranted(RewardPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobSpawnPayload {
    pub instance_id: u64,
    pub mob_def_id: String,
    pub display_name: String,
    pub position: [f32; 3],
    pub hp: u32,
    pub max_hp: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobUpdatePayload {
    pub instance_id: u64,
    pub position: [f32; 3],
    pub hp: u32,
    pub target_player_id: Option<u32>, // For aggro indicator
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobDamagedPayload {
    pub instance_id: u64,
    pub damage: u32,
    pub new_hp: u32,
    pub attacker_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobDiedPayload {
    pub instance_id: u64,
    pub killer_id: Option<u32>, // Last hit player
    pub position: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootDropPayload {
    pub loot_id: u64,
    pub item_id: String,
    pub item_name: String,
    pub quantity: u32,
    pub position: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardPayload {
    pub reward_type: RewardType,
    pub amount: u32,
    pub source: String, // e.g., "Cave Rat", "Quest: Clear Rats"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardType {
    Xp,
    Currency,
}
```

### Dungeon Messages

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    // ... existing variants ...

    /// Player entered dungeon area
    DungeonEntered(DungeonEnteredPayload),

    /// Dungeon state update
    DungeonStateUpdate(DungeonStatePayload),

    /// Dungeon completed notification
    DungeonCompleted(DungeonCompletedPayload),

    /// Reward chest available
    ChestAvailable(ChestAvailablePayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonEnteredPayload {
    pub dungeon_id: String,
    pub display_name: String,
    pub objective: String, // e.g., "Defeat the Gate Warden"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonStatePayload {
    pub dungeon_id: String,
    pub boss_alive: bool,
    pub boss_hp_percent: Option<f32>, // None if boss not visible
    pub chest_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonCompletedPayload {
    pub dungeon_id: String,
    pub display_name: String,
    pub rewards: Vec<(String, u32)>, // (item_name, count)
    pub xp_gained: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChestAvailablePayload {
    pub chest_id: u64,
    pub position: [f32; 3],
    pub dungeon_id: String,
}
```

---

## Protocol Messages (Client → Server)

### Loot & Chest Interaction

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    // ... existing variants ...

    /// Attempt to pick up loot
    LootPickup { loot_id: u64 },

    /// Attempt to open reward chest
    ChestOpen { chest_id: u64 },
}
```

---

## Mod API Events

### MobEvent

```rust
// crates/plix-mod-core/src/events.rs

#[derive(Debug, Clone)]
pub enum ModEvent {
    // ... existing variants ...

    Mob(MobModEvent),
    Dungeon(DungeonModEvent),
}

#[derive(Debug, Clone)]
pub enum MobModEvent {
    /// Mob spawned
    Spawned {
        instance_id: u64,
        definition_id: String,
        position: Vec3,
        spawn_point_id: String,
    },

    /// Mob took damage
    Damaged {
        instance_id: u64,
        attacker: PlayerId,
        damage: u32,
        remaining_hp: u32,
    },

    /// Mob was killed
    Killed {
        instance_id: u64,
        definition_id: String,
        killer: Option<PlayerId>, // Last hit
        contributors: Vec<(PlayerId, u32)>, // (player, damage)
        position: Vec3,
    },

    /// Mob dropped loot
    LootDropped {
        instance_id: u64,
        loot_id: u64,
        item_id: String,
        quantity: u32,
        position: Vec3,
    },

    /// Player received XP/credits from mob kill
    RewardDistributed {
        instance_id: u64,
        player_id: PlayerId,
        xp: u32,
        credits: u32,
        is_killer: bool, // Got last-hit bonus
    },
}

#[derive(Debug, Clone)]
pub enum DungeonModEvent {
    /// Player entered dungeon zone
    Entered {
        player_id: PlayerId,
        dungeon_id: String,
    },

    /// Dungeon boss was killed
    BossKilled {
        dungeon_id: String,
        killer: Option<PlayerId>,
        participants: Vec<PlayerId>,
    },

    /// Reward chest opened
    ChestOpened {
        dungeon_id: String,
        player_id: PlayerId,
        rewards: Vec<(String, u32)>,
    },

    /// Dungeon marked as completed for player
    Completed {
        dungeon_id: String,
        player_id: PlayerId,
    },

    /// Boss respawned
    BossRespawned {
        dungeon_id: String,
    },
}
```

---

## Damage & Payout Flow

### Damage Attribution

When a player damages a mob:

```rust
// crates/plix-server/src/mob/damage.rs

impl DamageTracker {
    pub fn record_damage(&mut self, player_id: PlayerId, damage: u32, tick: Tick) {
        if let Some(contrib) = self.contributions.iter_mut()
            .find(|c| c.player_id == player_id) {
            contrib.damage += damage;
            contrib.last_hit_tick = tick;
        } else {
            self.contributions.push(DamageContribution {
                player_id,
                damage,
                last_hit_tick: tick,
            });
        }
        self.total_damage += damage;
    }
}
```

### XP/Credit Distribution on Kill

```rust
// crates/plix-server/src/mob/payout.rs

pub struct PayoutConfig {
    pub killer_bonus_percent: f32,      // 0.10 to 0.25 (10-25%)
    pub min_damage_share: f32,          // 0.05 (5%)
    pub assist_window_ticks: u64,       // 600 (10 seconds at 60 TPS)
}

pub fn calculate_payouts(
    tracker: &DamageTracker,
    mob_def: &MobDefinition,
    killer: PlayerId,
    death_tick: Tick,
    config: &PayoutConfig,
) -> Vec<(PlayerId, u32, u32)> { // (player, xp, credits)
    let mut payouts = Vec::new();

    let base_xp = mob_def.xp_reward;
    let base_credits = mob_def.credit_reward;

    for contrib in &tracker.contributions {
        // Anti-abuse filter
        let damage_share = contrib.damage as f32 / tracker.total_damage as f32;
        let recent_hit = death_tick.0 - contrib.last_hit_tick.0 <= config.assist_window_ticks;

        if damage_share < config.min_damage_share && !recent_hit {
            continue; // Skip minimal contributors
        }

        // Base payout proportional to damage
        let mut xp = (base_xp as f32 * damage_share) as u32;
        let mut credits = (base_credits as f32 * damage_share) as u32;

        // Last-hit bonus
        if contrib.player_id == killer {
            xp += (base_xp as f32 * config.killer_bonus_percent) as u32;
            credits += (base_credits as f32 * config.killer_bonus_percent) as u32;
        }

        payouts.push((contrib.player_id, xp, credits));
    }

    payouts
}
```

---

## Loot Flow (Free-for-All)

### Drop Creation

```
1. Mob dies
2. MobSystem rolls loot table (with optional seed for testing)
3. For each drop:
   a. Create LootEntity at mob's death position
   b. Emit ModEvent::Mob(LootDropped { ... })
   c. Send ServerMessage::LootDropped to nearby players
```

### Loot Pickup

```
1. Player sends ClientMessage::LootPickup { loot_id }
2. Server validates:
   a. Loot exists
   b. Player is within pickup range
   c. Player has inventory space
3. If valid:
   a. Add item to player inventory
   b. Remove loot entity
   c. Send ServerMessage::InventoryUpdate to player
4. If invalid:
   a. Send error message (inventory full, too far, already looted)
```

---

## Dungeon State Flow

### Shared World Boss

```
Initial State:
  boss_alive = true
  chest_available = false
  cleared_by = {}

On Boss Kill:
  boss_alive = false
  chest_available = true
  schedule boss respawn (boss_respawn_secs)
  emit DungeonModEvent::BossKilled

On Chest Open (by player P):
  if P not in cleared_by:
    add P to cleared_by
    grant loot to P
    emit DungeonModEvent::ChestOpened
    emit DungeonModEvent::Completed
    (chest remains for other players)

On Boss Respawn:
  boss_alive = true
  chest_available = false
  cleared_by = {}
  emit DungeonModEvent::BossRespawned
```

---

## CEF Bridge Messages

### Mob Combat

```javascript
// Combat feedback (damage numbers, death effects handled by renderer)
window.plix.on('mob_damaged', (payload) => {
    // payload: { instance_id, damage, new_hp, attacker_id }
    showDamageNumber(payload);
});

window.plix.on('mob_died', (payload) => {
    // payload: { instance_id, killer_id, position }
    playDeathEffect(payload);
});

// Reward notification
window.plix.on('reward_granted', (payload) => {
    // payload: { reward_type: "Xp"|"Currency", amount, source }
    showRewardToast(payload);
});
```

### Dungeon HUD

```javascript
// Dungeon state for HUD
window.plix.on('dungeon_state_update', (payload) => {
    // payload: { dungeon_id, boss_alive, boss_hp_percent, chest_available }
    updateDungeonHUD(payload);
});

window.plix.on('dungeon_completed', (payload) => {
    // payload: { dungeon_id, display_name, rewards, xp_gained }
    showDungeonCompleteScreen(payload);
});
```
