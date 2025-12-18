# Quickstart Guide: Weapons & Items v1

**Feature**: 022-weapons-items-v1
**Date**: 2025-12-17

## Overview

This guide explains how to integrate the weapons system with existing plix systems.

---

## Prerequisites

- Feature 021 (Inventory Hotbar) must be implemented
- Existing item system with `ItemId::SWORD` defined
- Existing combat system in `plix-server/src/sim/combat.rs`

---

## Step 1: Add New Types to plix-common

### Add ProjectileId to types.rs

```rust
// In plix-common/src/types.rs

/// Unique projectile identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectileId {
    pub index: u16,
    pub generation: u16,
}

impl ProjectileId {
    pub const NONE: Self = Self { index: 0xFFFF, generation: 0 };
}
```

### Add ItemId::BOW

```rust
// In plix-common/src/types.rs (extend existing ItemId)

impl ItemId {
    // ... existing constants ...
    pub const BOW: Self = Self(4);
}
```

### Add Protocol Messages

```rust
// In plix-common/src/protocol/messages.rs (extend GameEvent enum)

// See contracts/protocol.md for full definitions
ProjectileSpawn { ... },
ProjectileImpact { ... },
ProjectileDespawn { ... },
WeaponCooldown { ... },
ProjectileLimitReached { ... },
```

---

## Step 2: Create weapons Module in plix-server

### Module Structure

```
crates/plix-server/src/weapons/
├── mod.rs           # Module exports
├── defs.rs          # WeaponDef constants
├── cooldown.rs      # CooldownState
├── recoil.rs        # RecoilState
├── melee.rs         # MeleeSystem
├── ranged.rs        # RangedSystem
└── projectiles.rs   # ProjectileManager
```

### mod.rs

```rust
pub mod defs;
pub mod cooldown;
pub mod recoil;
pub mod melee;
pub mod ranged;
pub mod projectiles;

pub use defs::{WeaponDef, SWORD_DEF, BOW_DEF, FIST_DEF};
pub use cooldown::CooldownState;
pub use recoil::RecoilState;
pub use melee::MeleeSystem;
pub use ranged::RangedSystem;
pub use projectiles::ProjectileManager;

/// Combined weapon state for a player
pub struct PlayerWeaponState {
    pub cooldowns: CooldownState,
    pub recoil: RecoilState,
}

impl Default for PlayerWeaponState {
    fn default() -> Self {
        Self {
            cooldowns: CooldownState::default(),
            recoil: RecoilState::default(),
        }
    }
}
```

---

## Step 3: Integrate with UseActiveItem

### Modify use_system.rs

```rust
// In plix-server/src/inventory/use_system.rs

use crate::weapons::{MeleeSystem, RangedSystem};

pub enum WeaponUseResult {
    MeleeAttack { damage: u16 },
    RangedShot { projectile_spawned: bool },
    Cooldown { remaining_ticks: u32 },
    ProjectileLimitReached,
    NotAWeapon,
}

pub fn try_use_weapon(
    hotbar: &Hotbar,
    weapon_state: &mut PlayerWeaponState,
    projectile_mgr: &mut ProjectileManager,
    current_tick: Tick,
    player_id: PlayerId,
    player_pos: Vec3,
    player_forward: Vec3,
    is_moving: bool,
) -> WeaponUseResult {
    let item_id = hotbar.active_item()
        .map(|s| s.item_id)
        .unwrap_or(ItemId::NONE);

    match item_id {
        ItemId::SWORD => {
            // Check cooldown
            if !weapon_state.cooldowns.is_ready(item_id, current_tick) {
                return WeaponUseResult::Cooldown {
                    remaining_ticks: weapon_state.cooldowns.remaining(item_id, current_tick),
                };
            }
            // Trigger cooldown
            weapon_state.cooldowns.trigger(item_id, current_tick, SWORD_DEF.cooldown_ticks);
            WeaponUseResult::MeleeAttack { damage: SWORD_DEF.damage }
        }

        ItemId::BOW => {
            // Check cooldown
            if !weapon_state.cooldowns.is_ready(item_id, current_tick) {
                return WeaponUseResult::Cooldown {
                    remaining_ticks: weapon_state.cooldowns.remaining(item_id, current_tick),
                };
            }
            // Check projectile limit
            if projectile_mgr.is_full() {
                // Trigger cooldown anyway (spec: cooldown triggers but no arrow spawns)
                weapon_state.cooldowns.trigger(item_id, current_tick, BOW_DEF.cooldown_ticks);
                return WeaponUseResult::ProjectileLimitReached;
            }
            // Calculate spread
            let spread = weapon_state.recoil.get_effective_spread(
                BOW_DEF.base_spread_deg,
                is_moving,
                current_tick,
            );
            // Update recoil
            weapon_state.recoil.add_shot(BOW_DEF.recoil_per_shot_deg, current_tick);
            // Trigger cooldown
            weapon_state.cooldowns.trigger(item_id, current_tick, BOW_DEF.cooldown_ticks);
            // Spawn projectile
            let direction = apply_spread(player_forward, spread, current_tick);
            projectile_mgr.spawn(player_id, player_pos, direction, BOW_DEF, current_tick);
            WeaponUseResult::RangedShot { projectile_spawned: true }
        }

        ItemId::NONE => {
            // Default melee (fist)
            if !weapon_state.cooldowns.is_ready(ItemId::NONE, current_tick) {
                return WeaponUseResult::Cooldown {
                    remaining_ticks: weapon_state.cooldowns.remaining(ItemId::NONE, current_tick),
                };
            }
            weapon_state.cooldowns.trigger(ItemId::NONE, current_tick, FIST_DEF.cooldown_ticks);
            WeaponUseResult::MeleeAttack { damage: FIST_DEF.damage }
        }

        _ => WeaponUseResult::NotAWeapon,
    }
}
```

---

## Step 4: Integrate ProjectileManager with Game Loop

### In session.rs or netloop.rs

```rust
// Add to server state
pub struct ServerState {
    // ... existing fields ...
    pub projectile_mgr: ProjectileManager,
}

// In tick loop
fn tick(&mut self) {
    // ... existing tick logic ...

    // Tick projectiles
    let impacts = self.projectile_mgr.tick(
        &self.players,
        &self.bots,
        &self.world,
        self.current_tick,
    );

    // Process impacts
    for impact in impacts {
        // Apply damage
        if let Some(target) = impact.target_player {
            self.apply_damage(target, impact.damage, impact.owner);
        }
        // Broadcast event
        self.broadcast(GameEvent::ProjectileImpact { ... });
    }

    // ... rest of tick ...
}
```

---

## Step 5: Add Bow to Item Registry

### In item_registry.rs

```rust
pub static ITEM_DEFS: &[ItemDef] = &[
    ItemDef::new(ItemId::SWORD, "Sword", ItemKind::Weapon, 1, 25),
    ItemDef::new(ItemId::HEALTH_PACK, "Health Pack", ItemKind::Consumable, 16, 50),
    ItemDef::new(ItemId::BLOCK_PLACER, "Block Placer", ItemKind::Tool, 1, 0),
    // NEW
    ItemDef::new(ItemId::BOW, "Bow", ItemKind::Weapon, 1, 15),
];
```

---

## Step 6: Update Loadouts

### In loadout logic (mode-specific)

```rust
// Training/TDM/FFA/CTF: Give Sword + Bow
fn get_starting_loadout(mode: GameMode) -> Vec<(u8, ItemStack)> {
    match mode {
        GameMode::Training | GameMode::Tdm | GameMode::Ffa | GameMode::Ctf => {
            vec![
                (0, ItemStack::single(ItemId::SWORD)),
                (1, ItemStack::single(ItemId::BOW)),
            ]
        }
        GameMode::BrLite => {
            // BR Lite: Start empty, find weapons as loot
            vec![]
        }
    }
}
```

---

## Testing

### Run Unit Tests

```bash
cargo test -p plix-server melee_combat
cargo test -p plix-server ranged_combat
cargo test -p plix-server cooldown
cargo test -p plix-server projectile_limit
cargo test -p plix-server spread_recoil
```

### Integration Test

```bash
cargo test -p plix-server weapon_integration
```

---

## Checklist

- [ ] ProjectileId added to plix-common/src/types.rs
- [ ] ItemId::BOW constant added
- [ ] Protocol messages added to messages.rs
- [ ] weapons/ module created with all submodules
- [ ] WeaponDef constants defined (SWORD_DEF, BOW_DEF, FIST_DEF)
- [ ] CooldownState implemented
- [ ] RecoilState implemented
- [ ] MeleeSystem with cone hit detection
- [ ] RangedSystem with spread calculation
- [ ] ProjectileManager with tick/collision logic
- [ ] use_system.rs routes to weapon systems
- [ ] Bow added to ITEM_DEFS registry
- [ ] Loadouts updated for all game modes
- [ ] All tests passing
- [ ] cargo clippy clean
- [ ] cargo fmt applied
