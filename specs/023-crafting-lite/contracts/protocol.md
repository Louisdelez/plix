# Protocol Contract: Crafting Lite

**Feature**: 023-crafting-lite
**Protocol Version**: 1.x (extends existing)
**Date**: 2025-12-17

## Overview

This document defines the network protocol messages for the Crafting Lite feature. These messages extend the existing `ClientMessage` and `GameEvent` enums in `plix-common/src/protocol/messages.rs`.

## Client → Server Messages

### CraftRequest

Request to craft an item using a specific recipe.

**Added to**: `ClientMessage` enum

```rust
CraftRequest {
    /// Recipe identifier (e.g., "health_pack", "sword", "bow")
    recipe_id: String,
}
```

**Validation**:
- `recipe_id` must be non-empty
- `recipe_id` max length: 32 characters

**Server Processing**:
1. Validate recipe exists in registry
2. Validate crafting enabled for current game mode
3. Validate recipe allowed in current mode
4. Validate player cooldown expired
5. Validate player is alive
6. Validate player has all required ingredients
7. Validate output can fit in hotbar
8. If all valid: consume inputs, add output, trigger cooldown
9. Send CraftResult response

**Rate Limiting**: Server enforces 1-second cooldown between successful crafts

## Server → Client Messages

### CraftResult

Result of a craft attempt sent to the requesting player.

**Added to**: `GameEvent` enum

```rust
CraftResult {
    /// Whether the craft succeeded
    success: bool,
    /// Recipe that was attempted
    recipe_id: String,
    /// Output item (only if success)
    output_item: Option<ItemId>,
    /// Output quantity (only if success)
    output_quantity: Option<u8>,
    /// Failure reason (only if !success)
    fail_reason: Option<CraftFailReason>,
}
```

### CraftFailReason

**New enum** for craft failure causes.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CraftFailReason {
    /// Recipe not found in registry
    UnknownRecipe,
    /// Crafting disabled for current game mode
    CraftingDisabled,
    /// Recipe not allowed in current game mode
    RecipeDisabled,
    /// Player on cooldown from recent craft
    CooldownActive,
    /// Player doesn't have required ingredients
    MissingIngredients,
    /// No space in hotbar for output
    HotbarFull,
    /// Player is dead
    PlayerDead,
}
```

## Message Flow

### Successful Craft

```
Client                          Server
  |                               |
  |------ CraftRequest ---------->|
  |       { recipe_id: "sword" }  |
  |                               | Validate recipe ✓
  |                               | Validate cooldown ✓
  |                               | Validate ingredients ✓
  |                               | Validate output space ✓
  |                               | Consume 3x SCRAP
  |                               | Add 1x SWORD
  |                               | Set cooldown
  |<----- CraftResult ------------|
  |       { success: true,        |
  |         recipe_id: "sword",   |
  |         output_item: SWORD,   |
  |         output_quantity: 1 }  |
  |                               |
  |<----- InventoryUpdate --------|
  |       (existing message)      |
```

### Failed Craft (Insufficient Ingredients)

```
Client                          Server
  |                               |
  |------ CraftRequest ---------->|
  |       { recipe_id: "bow" }    |
  |                               | Validate recipe ✓
  |                               | Validate cooldown ✓
  |                               | Validate ingredients ✗
  |                               | (has 2 SCRAP, needs 4)
  |<----- CraftResult ------------|
  |       { success: false,       |
  |         recipe_id: "bow",     |
  |         fail_reason:          |
  |           MissingIngredients }|
  |                               |
  (No InventoryUpdate - no change)
```

### Failed Craft (Cooldown Active)

```
Client                          Server
  |                               |
  |------ CraftRequest ---------->|
  |       { recipe_id: "sword" }  |
  |                               | Validate cooldown ✗
  |                               | (45 ticks remaining)
  |<----- CraftResult ------------|
  |       { success: false,       |
  |         recipe_id: "sword",   |
  |         fail_reason:          |
  |           CooldownActive }    |
```

## Serialization

All messages use the existing bincode serialization format, maintaining backward compatibility with protocol version 1.x.

**Size Estimates**:
- CraftRequest: ~40 bytes (recipe_id max 32 + overhead)
- CraftResult (success): ~50 bytes
- CraftResult (failure): ~45 bytes

## Error Handling

| Error Condition | CraftFailReason | Client Action |
|-----------------|-----------------|---------------|
| Recipe not found | UnknownRecipe | Display "Unknown recipe" |
| Mode disabled | CraftingDisabled | Display "Crafting not available" |
| Recipe restricted | RecipeDisabled | Display "Recipe not available" |
| On cooldown | CooldownActive | Display "Cooldown active" |
| Missing items | MissingIngredients | Display "Not enough materials" |
| Hotbar full | HotbarFull | Display "Inventory full" |
| Player dead | PlayerDead | Suppress (can't craft while dead) |

## Console Command Interface

Client-side console command that sends CraftRequest:

```
/craft <recipe_id>

Examples:
  /craft health_pack  → CraftRequest { recipe_id: "health_pack" }
  /craft sword        → CraftRequest { recipe_id: "sword" }
  /craft bow          → CraftRequest { recipe_id: "bow" }
```

**Console Output**:
- Success: "Crafted 1x Sword"
- Failure: "Craft failed: Not enough materials"
