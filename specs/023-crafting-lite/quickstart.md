# Quickstart: Crafting Lite

**Feature**: 023-crafting-lite
**Date**: 2025-12-17

## Overview

This guide explains how to implement the Crafting Lite feature for plix.

## Prerequisites

- Rust 1.75+ (stable)
- Feature 021 (Inventory Hotbar) complete
- Feature 022 (Weapons & Items v1) complete

## Implementation Steps

### Step 1: Add SCRAP Item (plix-common)

1. Add `ItemId::SCRAP` constant in `types.rs`:
```rust
pub const SCRAP: Self = Self(5);
```

2. Add `ItemKind::Resource` variant in `inventory/item.rs`:
```rust
pub enum ItemKind {
    Weapon,
    Consumable,
    Tool,
    Resource,  // NEW
}
```

3. Add SCRAP definition in server's `item_registry.rs`:
```rust
pub static SCRAP_DEF: ItemDef = ItemDef::new(
    ItemId::SCRAP, "Scrap", ItemKind::Resource, 16, 0
);
```

### Step 2: Extend Hotbar (plix-common)

Add helper methods to `Hotbar`:

```rust
/// Count total quantity of an item across all slots
pub fn count_item(&self, item_id: ItemId) -> u8 {
    self.slots.iter()
        .filter_map(|s| s.as_ref())
        .filter(|stack| stack.item_id == item_id)
        .map(|stack| stack.quantity)
        .fold(0u8, |acc, q| acc.saturating_add(q))
}

/// Consume items from hotbar. Returns true if successful.
pub fn consume_items(&mut self, item_id: ItemId, mut quantity: u8) -> bool {
    // First verify we have enough
    if self.count_item(item_id) < quantity {
        return false;
    }
    // Consume from slots
    for slot in &mut self.slots {
        if quantity == 0 { break; }
        if let Some(stack) = slot {
            if stack.item_id == item_id {
                let to_take = quantity.min(stack.quantity);
                stack.quantity -= to_take;
                quantity -= to_take;
                if stack.quantity == 0 {
                    *slot = None;
                }
            }
        }
    }
    true
}
```

### Step 3: Add Protocol Messages (plix-common)

In `protocol/messages.rs`:

```rust
// Add to ClientMessage enum
CraftRequest {
    recipe_id: String,
}

// Add new enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CraftFailReason {
    UnknownRecipe,
    CraftingDisabled,
    RecipeDisabled,
    CooldownActive,
    MissingIngredients,
    HotbarFull,
    PlayerDead,
}

// Add to GameEvent enum
CraftResult {
    success: bool,
    recipe_id: String,
    output_item: Option<ItemId>,
    output_quantity: Option<u8>,
    fail_reason: Option<CraftFailReason>,
}
```

### Step 4: Create Crafting Module (plix-server)

Create `src/crafting/` module:

**mod.rs**:
```rust
pub mod recipe;
pub mod system;
pub mod cooldown;
pub mod errors;
pub mod metrics;

pub use recipe::{Recipe, RecipeId, RecipeRegistry, RECIPE_REGISTRY};
pub use system::CraftSystem;
pub use cooldown::CraftCooldown;
pub use errors::CraftValidation;
pub use metrics::CraftMetrics;
```

**recipe.rs**:
```rust
pub struct RecipeId(pub String);

pub struct Ingredient {
    pub item_id: ItemId,
    pub quantity: u8,
}

pub struct Recipe {
    pub id: RecipeId,
    pub inputs: Vec<Ingredient>,
    pub output_item: ItemId,
    pub output_quantity: u8,
}

pub static RECIPE_REGISTRY: Lazy<RecipeRegistry> = Lazy::new(|| {
    RecipeRegistry::new(vec![
        Recipe::new("health_pack", vec![(ItemId::SCRAP, 2)], ItemId::HEALTH_PACK, 1),
        Recipe::new("sword", vec![(ItemId::SCRAP, 3)], ItemId::SWORD, 1),
        Recipe::new("bow", vec![(ItemId::SCRAP, 4)], ItemId::BOW, 1),
    ])
});
```

**cooldown.rs**:
```rust
pub struct CraftCooldown {
    next_allowed_tick: Option<Tick>,
}

impl CraftCooldown {
    pub fn is_ready(&self, current_tick: Tick) -> bool {
        self.next_allowed_tick.map_or(true, |t| current_tick.0 >= t.0)
    }

    pub fn trigger(&mut self, current_tick: Tick, tick_rate: u32) {
        self.next_allowed_tick = Some(Tick(current_tick.0 + tick_rate));
    }
}
```

**system.rs**:
```rust
pub struct CraftSystem;

impl CraftSystem {
    pub fn try_craft(
        recipe_id: &str,
        hotbar: &mut Hotbar,
        cooldown: &mut CraftCooldown,
        mode: GameMode,
        current_tick: Tick,
        tick_rate: u32,
        is_alive: bool,
    ) -> Result<(ItemId, u8), CraftFailReason> {
        // 1. Validate recipe exists
        let recipe = RECIPE_REGISTRY.get(recipe_id)
            .ok_or(CraftFailReason::UnknownRecipe)?;

        // 2. Validate mode allows crafting
        let config = get_craft_config(mode);
        if !config.enabled {
            return Err(CraftFailReason::CraftingDisabled);
        }

        // 3. Validate player alive
        if !is_alive {
            return Err(CraftFailReason::PlayerDead);
        }

        // 4. Validate cooldown
        if !cooldown.is_ready(current_tick) {
            return Err(CraftFailReason::CooldownActive);
        }

        // 5. Validate ingredients
        for input in &recipe.inputs {
            if hotbar.count_item(input.item_id) < input.quantity {
                return Err(CraftFailReason::MissingIngredients);
            }
        }

        // 6. Validate output space
        let max_stack = get_max_stack(recipe.output_item);
        let remaining = hotbar.can_add_item(recipe.output_item, recipe.output_quantity, max_stack);
        if remaining > 0 {
            return Err(CraftFailReason::HotbarFull);
        }

        // 7. Apply craft (atomic)
        for input in &recipe.inputs {
            hotbar.consume_items(input.item_id, input.quantity);
        }
        hotbar.try_add_item(recipe.output_item, recipe.output_quantity, max_stack);
        cooldown.trigger(current_tick, tick_rate);

        Ok((recipe.output_item, recipe.output_quantity))
    }
}
```

### Step 5: Wire to Server (plix-server/src/lib.rs)

1. Add `CraftCooldown` to `ServerPlayer`
2. Handle `ClientMessage::CraftRequest` in message processing
3. Send `CraftResult` event to client

### Step 6: Update Training Loadout

In `inventory/config.rs`:
```rust
GameMode::Training => vec![
    (0, ItemStack::single(ItemId::SWORD)),
    (1, ItemStack::single(ItemId::BOW)),
    (2, ItemStack::new(ItemId::HEALTH_PACK, 5)),
    (3, ItemStack::new(ItemId::SCRAP, 5)),  // NEW
],
```

### Step 7: Add Console Command (plix-client)

Handle `/craft <recipe_id>` console command:
```rust
if input.starts_with("/craft ") {
    let recipe_id = input.trim_start_matches("/craft ").trim();
    send_message(ClientMessage::CraftRequest {
        recipe_id: recipe_id.to_string()
    });
}
```

## Testing

Run unit tests:
```bash
cargo test -p plix-server --lib crafting
```

Run integration tests:
```bash
cargo test -p plix-server --test crafting_test
```

## Verification Checklist

- [ ] SCRAP item spawns in BR Lite loot
- [ ] SCRAP appears in Training loadout
- [ ] `/craft health_pack` works with 2+ SCRAP
- [ ] `/craft sword` works with 3+ SCRAP
- [ ] `/craft bow` works with 4+ SCRAP
- [ ] Craft fails with insufficient ingredients
- [ ] Craft fails when hotbar is full
- [ ] 1-second cooldown enforced between crafts
- [ ] Crafting disabled in TDM/FFA/CTF modes
- [ ] Metrics increment on success/failure
