# Research: Crafting Lite

**Feature**: 023-crafting-lite
**Date**: 2025-12-17

## Overview

This document captures research findings and design decisions for the Crafting Lite feature.

## 1. Hotbar Integration

### Decision: Extend existing Hotbar methods

**Rationale**: The existing `Hotbar` struct in `plix-common/src/inventory/hotbar.rs` already provides:
- `try_add_item(item_id, quantity, max_stack)` - for adding craft outputs
- `find_stackable_slot(item_id, max_stack)` - for checking output space
- `find_empty_slot()` - for output placement
- `get(slot)` / `set(slot, stack)` - for reading/modifying slots

**Required additions**:
1. `count_item(&self, item_id: ItemId) -> u8` - Count total quantity of an item across all slots
2. `consume_items(&mut self, item_id: ItemId, quantity: u8) -> bool` - Atomically consume items from slots

**Alternatives considered**:
- External helper functions: Rejected - would duplicate slot iteration logic
- Crafting-specific inventory wrapper: Rejected - over-engineering for simple needs

## 2. Recipe Identifier Format

### Decision: Use string-based RecipeId

**Rationale**: String identifiers (e.g., "health_pack", "sword", "bow") are:
- Human-readable in console commands (`/craft health_pack`)
- Easy to extend without recompiling
- Serializable without special handling

**Implementation**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecipeId(pub String);

impl RecipeId {
    pub const HEALTH_PACK: &'static str = "health_pack";
    pub const SWORD: &'static str = "sword";
    pub const BOW: &'static str = "bow";
}
```

**Alternatives considered**:
- Enum-based: Rejected - requires code changes for new recipes
- u16 index: Rejected - not human-readable for console commands

## 3. Atomic Craft Transaction

### Decision: Validate-then-apply pattern with no rollback needed

**Rationale**: Since all validation happens before any mutation:
1. Check recipe exists and is enabled for mode
2. Check cooldown not active
3. Count ingredients in hotbar (read-only)
4. Check output can fit (read-only)
5. **If all pass**: Consume inputs, add output (mutations)

No partial state is ever created because mutations only happen after complete validation.

**Implementation**:
```rust
pub enum CraftValidation {
    Valid,
    UnknownRecipe,
    RecipeDisabled,
    CooldownActive { remaining_ticks: u32 },
    MissingIngredients { item_id: ItemId, required: u8, available: u8 },
    HotbarFull,
    PlayerDead,
}

pub fn validate_craft(...) -> CraftValidation { /* read-only checks */ }
pub fn apply_craft(...) { /* mutations - only called if validation passed */ }
```

## 4. Cooldown Implementation

### Decision: Per-player tick-based cooldown (60 ticks = 1 second at 60Hz)

**Rationale**: Aligns with existing cooldown patterns in `weapons/cooldown.rs`.

**Implementation**:
```rust
pub struct CraftCooldown {
    /// Tick when cooldown expires (next craft allowed)
    next_allowed_tick: Option<Tick>,
}

impl CraftCooldown {
    pub fn is_ready(&self, current_tick: Tick) -> bool {
        self.next_allowed_tick.map_or(true, |t| current_tick.0 >= t.0)
    }

    pub fn trigger(&mut self, current_tick: Tick, tick_rate: u32) {
        // 1 second = tick_rate ticks
        self.next_allowed_tick = Some(Tick(current_tick.0 + tick_rate));
    }
}
```

**Alternatives considered**:
- Duration-based with Instant: Rejected - Tick-based is consistent with game systems
- Global cooldown: Rejected - per-player allows concurrent crafting by different players

## 5. Protocol Messages

### Decision: Add to existing ClientMessage/GameEvent enums

**Rationale**: Follows existing patterns for UseActiveItem, BlockEdit, etc.

**Implementation**:
```rust
// ClientMessage enum (client → server)
CraftRequest {
    recipe_id: String,
}

// GameEvent enum (server → client)
CraftResult {
    success: bool,
    recipe_id: String,
    output_item: Option<ItemId>,
    output_quantity: Option<u8>,
    fail_reason: Option<CraftFailReason>,
}
```

## 6. Game Mode Integration

### Decision: Use CraftConfig per GameMode with bitflags for recipe restrictions

**Rationale**: Simple configuration that integrates with existing GameMode enum.

**Implementation**:
```rust
pub struct CraftConfig {
    pub enabled: bool,
    pub allowed_recipes: Option<HashSet<String>>, // None = all allowed
}

pub fn get_craft_config(mode: GameMode) -> CraftConfig {
    match mode {
        GameMode::Training | GameMode::BrLite => CraftConfig { enabled: true, allowed_recipes: None },
        GameMode::Tdm | GameMode::Ffa | GameMode::Ctf => CraftConfig { enabled: false, allowed_recipes: None },
    }
}
```

## 7. SCRAP Item Definition

### Decision: Add ItemId::SCRAP with ItemKind::Resource

**Rationale**: New item category for crafting ingredients that aren't weapons, consumables, or tools.

**Implementation**:
```rust
// types.rs
impl ItemId {
    pub const SCRAP: Self = Self(5);
}

// item.rs
pub enum ItemKind {
    Weapon,
    Consumable,
    Tool,
    Resource,  // NEW
}

// item_registry.rs
pub static SCRAP_DEF: ItemDef = ItemDef::new(
    ItemId::SCRAP,
    "Scrap",
    ItemKind::Resource,
    16,  // max_stack
    0,   // param (unused for resources)
);
```

## 8. Metrics and Logging

### Decision: Follow existing metrics pattern with tracing macros

**Rationale**: Consistent with `plix-server/src/metrics.rs` patterns.

**Implementation**:
```rust
pub struct CraftMetrics {
    pub crafts_success: u64,
    pub crafts_failed: HashMap<CraftFailReason, u64>,
}

// Logging
tracing::info!(player_id = %player_id, recipe_id = %recipe_id, "Craft succeeded");
tracing::debug!(player_id = %player_id, recipe_id = %recipe_id, reason = ?reason, "Craft failed");
```

## Summary of Decisions

| Topic | Decision | Key Rationale |
|-------|----------|---------------|
| Hotbar integration | Extend with count_item/consume_items | Reuse existing patterns |
| Recipe ID | String-based | Human-readable for console |
| Atomicity | Validate-then-apply | No rollback complexity |
| Cooldown | Per-player tick-based | Consistent with weapons |
| Protocol | Extend ClientMessage/GameEvent | Follows existing patterns |
| Mode config | CraftConfig struct | Simple per-mode settings |
| SCRAP item | ItemKind::Resource | Clear category separation |
| Metrics | Counter struct + tracing | Consistent with codebase |
