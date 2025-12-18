# Data Model: Crafting Lite

**Feature**: 023-crafting-lite
**Date**: 2025-12-17

## Entities

### RecipeId

Unique identifier for a crafting recipe.

| Field | Type | Constraints |
|-------|------|-------------|
| id | String | Non-empty, lowercase alphanumeric + underscore |

**Constants**:
- `HEALTH_PACK` = "health_pack"
- `SWORD` = "sword"
- `BOW` = "bow"

### Ingredient

A required input for a recipe.

| Field | Type | Constraints |
|-------|------|-------------|
| item_id | ItemId | Must be valid item |
| quantity | u8 | 1-255 |

### Recipe

Defines a crafting transformation.

| Field | Type | Constraints |
|-------|------|-------------|
| id | RecipeId | Unique across registry |
| inputs | Vec<Ingredient> | Non-empty, no duplicates |
| output_item | ItemId | Must be valid item |
| output_quantity | u8 | 1-255 |

**Validation Rules**:
- At least one input required
- No duplicate item_ids in inputs
- Output item must exist in item registry
- Output quantity must not exceed max_stack

**Initial Recipes (v1)**:

| Recipe ID | Inputs | Output |
|-----------|--------|--------|
| health_pack | 2x SCRAP | 1x HEALTH_PACK |
| sword | 3x SCRAP | 1x SWORD |
| bow | 4x SCRAP | 1x BOW |

### RecipeRegistry

Immutable collection of all recipes.

| Field | Type | Constraints |
|-------|------|-------------|
| recipes | HashMap<RecipeId, Recipe> | Immutable after init |

**Methods**:
- `get(recipe_id) -> Option<&Recipe>`
- `exists(recipe_id) -> bool`
- `all() -> impl Iterator<Item = &Recipe>`

### CraftCooldown

Per-player cooldown state.

| Field | Type | Constraints |
|-------|------|-------------|
| next_allowed_tick | Option<Tick> | None = ready |

**Methods**:
- `is_ready(current_tick) -> bool`
- `trigger(current_tick, tick_rate)` - Sets cooldown for 1 second
- `remaining_ticks(current_tick) -> u32`

### CraftConfig

Per-game-mode crafting configuration.

| Field | Type | Constraints |
|-------|------|-------------|
| enabled | bool | true/false |
| allowed_recipes | Option<HashSet<String>> | None = all allowed |

**Defaults by GameMode**:

| Mode | Enabled | Allowed Recipes |
|------|---------|-----------------|
| Training | true | All |
| BrLite | true | All |
| Tdm | false | N/A |
| Ffa | false | N/A |
| Ctf | false | N/A |

### CraftFailReason

Enumeration of craft failure causes.

| Variant | Description |
|---------|-------------|
| UnknownRecipe | Recipe ID not in registry |
| RecipeDisabled | Recipe not allowed in current mode |
| CraftingDisabled | Crafting disabled for current mode |
| CooldownActive | 1-second cooldown not expired |
| MissingIngredients | Player lacks required items |
| HotbarFull | No space for output item |
| PlayerDead | Player is dead |

### CraftMetrics

Observability counters.

| Field | Type | Description |
|-------|------|-------------|
| crafts_success | u64 | Total successful crafts |
| crafts_failed | HashMap<CraftFailReason, u64> | Failures by reason |

### ItemId::SCRAP

New resource item type.

| Property | Value |
|----------|-------|
| ID | 5 |
| Name | "Scrap" |
| Kind | Resource |
| Max Stack | 16 |
| Param | 0 (unused) |

## State Transitions

### Craft Request Flow

```
[Idle] --CraftRequest--> [Validating]
[Validating] --Valid--> [Applying] --Success--> [Idle + Cooldown]
[Validating] --Invalid--> [Idle] (no state change)
```

### Cooldown State

```
[Ready] --CraftSuccess--> [OnCooldown]
[OnCooldown] --TickPasses(60)--> [Ready]
```

## Relationships

```
RecipeRegistry (1) --- contains --- (*) Recipe
Recipe (1) --- has --- (1..*) Ingredient
Recipe (1) --- produces --- (1) ItemId
ServerPlayer (1) --- has --- (1) CraftCooldown
ServerPlayer (1) --- has --- (1) Hotbar
GameMode (1) --- has --- (1) CraftConfig
```

## Data Constraints Summary

1. **Uniqueness**: RecipeId must be unique in registry
2. **Referential Integrity**: All ItemIds in recipes must exist in item registry
3. **Invariants**:
   - Recipe inputs non-empty
   - Recipe quantities > 0
   - Cooldown tick monotonically increases
4. **Atomicity**: Craft consumes inputs AND adds output, or neither
