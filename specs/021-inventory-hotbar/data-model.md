# Data Model: Inventory Hotbar

**Feature**: 021-inventory-hotbar
**Date**: 2025-12-17

---

## Entity Relationship Diagram

```
┌─────────────┐     owns      ┌─────────────┐
│   Player    │──────────────▶│   Hotbar    │
│  (PlayerId) │               │             │
└─────────────┘               └──────┬──────┘
                                     │ contains
                                     ▼
                              ┌─────────────┐
                              │    Slot     │
                              │ [0..N-1]    │
                              └──────┬──────┘
                                     │ holds (optional)
                                     ▼
                              ┌─────────────┐     references    ┌─────────────┐
                              │  ItemStack  │─────────────────▶│   ItemDef   │
                              │             │                   │  (static)   │
                              └─────────────┘                   └─────────────┘
                                     ▲
                                     │ contains
┌─────────────┐                      │
│ LootEntity  │──────────────────────┘
│ (world)     │
└─────────────┘
```

---

## Core Types (plix-common/src/inventory/)

### ItemId

Unique identifier for item definitions.

```rust
// File: crates/plix-common/src/types.rs (addition)

/// Unique item type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u16);

impl ItemId {
    pub const NONE: Self = Self(0);
    pub const SWORD: Self = Self(1);
    pub const HEALTH_PACK: Self = Self(2);
    pub const BLOCK_PLACER: Self = Self(3);

    pub fn is_valid(&self) -> bool {
        *self != Self::NONE
    }
}
```

**Validation Rules:**
- `NONE (0)` is invalid/placeholder
- Valid IDs: 1-65535
- Must exist in ItemDef registry for server operations

---

### ItemKind

Discriminates item behavior categories.

```rust
// File: crates/plix-common/src/inventory/item.rs

/// Category of item determining its behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    /// Deals damage on use
    Weapon {
        damage: u8,
        range: f32,
    },
    /// Applies effect on use, consumed
    Consumable {
        heal: u8,
        max_stack: u8,
    },
    /// Performs action on use (e.g., place block)
    Tool {
        block_type: BlockType,
    },
}
```

**Validation Rules:**
- Weapon: `damage > 0`, `range > 0.0`
- Consumable: `heal > 0`, `max_stack >= 1`
- Tool: `block_type != BlockType::AIR`

---

### ItemDef

Static definition of an item type.

```rust
// File: crates/plix-common/src/inventory/item.rs

/// Static item definition (registered at startup)
#[derive(Debug, Clone)]
pub struct ItemDef {
    pub id: ItemId,
    pub name: &'static str,
    pub kind: ItemKind,
}

impl ItemDef {
    /// Check if this item can stack
    pub fn is_stackable(&self) -> bool {
        matches!(self.kind, ItemKind::Consumable { .. })
    }

    /// Get max stack size (1 for non-stackable)
    pub fn max_stack(&self) -> u8 {
        match self.kind {
            ItemKind::Consumable { max_stack, .. } => max_stack,
            _ => 1,
        }
    }
}
```

**Initial Item Definitions (v1):**

| ID | Name | Kind | Properties |
|----|------|------|------------|
| 1 | Sword | Weapon | damage: 25, range: 2.0 |
| 2 | Health Pack | Consumable | heal: 50, max_stack: 16 |
| 3 | Block Placer | Tool | block_type: Stone |

---

### ItemStack

Instance of item(s) in inventory or world.

```rust
// File: crates/plix-common/src/inventory/item_stack.rs

/// Stack of items (item type + quantity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemStack {
    pub item_id: ItemId,
    pub quantity: u8,
}

impl ItemStack {
    /// Create a new stack (panics if quantity is 0)
    pub fn new(item_id: ItemId, quantity: u8) -> Self {
        assert!(quantity > 0, "ItemStack quantity must be > 0");
        Self { item_id, quantity }
    }

    /// Create a single item stack
    pub fn single(item_id: ItemId) -> Self {
        Self { item_id, quantity: 1 }
    }

    /// Try to add items, returns overflow
    pub fn try_add(&mut self, amount: u8, max_stack: u8) -> u8 {
        let space = max_stack.saturating_sub(self.quantity);
        let to_add = amount.min(space);
        self.quantity += to_add;
        amount - to_add // overflow
    }

    /// Remove items, returns true if stack is now empty
    pub fn remove(&mut self, amount: u8) -> bool {
        self.quantity = self.quantity.saturating_sub(amount);
        self.quantity == 0
    }
}
```

**Validation Rules:**
- `quantity >= 1` (invariant)
- `quantity <= ItemDef::max_stack()` (enforced at modification)
- `item_id` must be valid

**State Transitions:**
```
Created (qty=N) → try_add() → qty increased (capped at max_stack)
                → remove() → qty decreased → if qty=0: Destroyed
```

---

### Hotbar

Player's equipped item slots.

```rust
// File: crates/plix-common/src/inventory/hotbar.rs

/// Player's hotbar (equipped items)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotbar {
    slots: Vec<Option<ItemStack>>,
    active_slot: u8,
}

impl Hotbar {
    /// Create empty hotbar with given capacity
    pub fn new(capacity: u8) -> Self {
        Self {
            slots: vec![None; capacity as usize],
            active_slot: 0,
        }
    }

    /// Get number of slots
    pub fn capacity(&self) -> u8 {
        self.slots.len() as u8
    }

    /// Get currently selected slot index
    pub fn active_slot(&self) -> u8 {
        self.active_slot
    }

    /// Select a slot (returns false if out of bounds)
    pub fn select_slot(&mut self, slot: u8) -> bool {
        if (slot as usize) < self.slots.len() {
            self.active_slot = slot;
            true
        } else {
            false
        }
    }

    /// Get item in active slot
    pub fn active_item(&self) -> Option<&ItemStack> {
        self.slots.get(self.active_slot as usize)?.as_ref()
    }

    /// Get mutable item in active slot
    pub fn active_item_mut(&mut self) -> Option<&mut ItemStack> {
        self.slots.get_mut(self.active_slot as usize)?.as_mut()
    }

    /// Get item in specific slot
    pub fn get_slot(&self, slot: u8) -> Option<&ItemStack> {
        self.slots.get(slot as usize)?.as_ref()
    }

    /// Set item in specific slot
    pub fn set_slot(&mut self, slot: u8, item: Option<ItemStack>) -> bool {
        if let Some(s) = self.slots.get_mut(slot as usize) {
            *s = item;
            true
        } else {
            false
        }
    }

    /// Find first empty slot
    pub fn first_empty_slot(&self) -> Option<u8> {
        self.slots.iter()
            .position(|s| s.is_none())
            .map(|i| i as u8)
    }

    /// Find slot with matching stackable item (not full)
    pub fn find_stackable_slot(&self, item_id: ItemId, max_stack: u8) -> Option<u8> {
        self.slots.iter()
            .position(|s| {
                s.as_ref()
                    .map(|stack| stack.item_id == item_id && stack.quantity < max_stack)
                    .unwrap_or(false)
            })
            .map(|i| i as u8)
    }

    /// Clear active slot (remove item)
    pub fn clear_active_slot(&mut self) {
        if let Some(s) = self.slots.get_mut(self.active_slot as usize) {
            *s = None;
        }
    }

    /// Get all slots for serialization
    pub fn slots(&self) -> &[Option<ItemStack>] {
        &self.slots
    }

    /// Clear all slots
    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
        self.active_slot = 0;
    }
}
```

**Validation Rules:**
- `capacity` in range 5-9 (configurable)
- `active_slot < capacity` (invariant)
- Slot indices 0-indexed

---

## Server Types (plix-server/src/inventory/)

### LootEntityId

Unique identifier for loot entities in world.

```rust
// File: crates/plix-common/src/types.rs (addition)

/// Unique loot entity identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LootEntityId(pub u32);

impl LootEntityId {
    pub const NONE: Self = Self(0);
}
```

---

### LootEntity

Loot item in the world available for pickup.

```rust
// File: crates/plix-server/src/loot/entity.rs

/// Loot entity in the world
#[derive(Debug, Clone)]
pub struct LootEntity {
    pub id: LootEntityId,
    pub position: Vec3,
    pub item: ItemStack,
    pub spawn_tick: Tick,
}

impl LootEntity {
    pub fn new(id: LootEntityId, position: Vec3, item: ItemStack, tick: Tick) -> Self {
        Self {
            id,
            position,
            item,
            spawn_tick: tick,
        }
    }
}
```

**Validation Rules:**
- `id` must be unique in world
- `position` must be within world bounds
- `item.quantity > 0`

---

### InventoryConfig

Server configuration for inventory system.

```rust
// File: crates/plix-server/src/inventory/config.rs

/// Inventory system configuration
#[derive(Debug, Clone)]
pub struct InventoryConfig {
    /// Number of hotbar slots (5-9)
    pub hotbar_slots: u8,
    /// Pickup range in blocks
    pub pickup_range: f32,
    /// Default starting items per game mode
    pub starting_loadouts: HashMap<GameMode, Vec<ItemStack>>,
}

impl Default for InventoryConfig {
    fn default() -> Self {
        let mut starting_loadouts = HashMap::new();

        // Training: Sword + 3 Health Packs
        starting_loadouts.insert(GameMode::Training, vec![
            ItemStack::single(ItemId::SWORD),
            ItemStack::new(ItemId::HEALTH_PACK, 3),
        ]);

        // TDM/FFA/CTF: Sword only
        starting_loadouts.insert(GameMode::Tdm, vec![
            ItemStack::single(ItemId::SWORD),
        ]);
        starting_loadouts.insert(GameMode::Ffa, vec![
            ItemStack::single(ItemId::SWORD),
        ]);
        starting_loadouts.insert(GameMode::Ctf, vec![
            ItemStack::single(ItemId::SWORD),
        ]);

        // BR Lite: Empty (loot-based)
        starting_loadouts.insert(GameMode::BrLite, vec![]);

        Self {
            hotbar_slots: 9,
            pickup_range: 1.5,
            starting_loadouts,
        }
    }
}
```

---

## Protocol Messages (additions to plix-common/src/protocol/messages.rs)

### Client → Server

```rust
/// Messages sent from client to server (additions)
pub enum ClientMessage {
    // ... existing variants ...

    /// Select hotbar slot
    SelectHotbarSlot {
        slot: u8,
    },

    /// Use item in active slot
    UseActiveItem,
}
```

### Server → Client

```rust
/// Messages sent from server to client (additions)
pub enum ServerMessage {
    // ... existing variants ...

    /// Inventory update (diff-based)
    InventoryUpdate {
        /// Changed slots only
        updates: Vec<SlotUpdate>,
        /// Currently selected slot
        active_slot: u8,
    },
}

/// Single slot update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotUpdate {
    pub slot: u8,
    pub item: Option<ItemStack>,
}
```

### Game Events

```rust
/// Game events (additions)
pub enum GameEvent {
    // ... existing variants ...

    /// Loot entity spawned in world
    LootSpawned {
        id: LootEntityId,
        position: Vec3,
        item_id: ItemId,
        quantity: u8,
    },

    /// Loot entity removed (picked up or despawned)
    LootRemoved {
        id: LootEntityId,
    },

    /// Player picked up loot
    LootPickedUp {
        player_id: PlayerId,
        loot_id: LootEntityId,
        item_id: ItemId,
        quantity: u8,
    },

    /// Item used (consumable consumed, weapon attacked, tool used)
    ItemUsed {
        player_id: PlayerId,
        item_id: ItemId,
    },
}
```

---

## Snapshot Extension (WorldSnapshot)

```rust
/// Player snapshot extension
pub struct PlayerSnapshot {
    // ... existing fields ...

    /// Hotbar state (optional, only for this player or spectating)
    #[serde(default)]
    pub hotbar: Option<HotbarSnapshot>,
}

/// Compact hotbar snapshot for replication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotbarSnapshot {
    pub slots: Vec<Option<(ItemId, u8)>>, // (item_id, quantity)
    pub active_slot: u8,
}
```

---

## Data Flow Summary

```
[Player Input]
     │
     ▼
┌─────────────────┐
│ SelectHotbarSlot│──▶ Server validates → Updates Hotbar.active_slot
│ UseActiveItem   │──▶ Server validates → Applies effect → Decrements stack
└─────────────────┘
     │
     ▼
┌─────────────────┐
│ Pickup (auto)   │──▶ Server proximity check → Adds to Hotbar → Removes LootEntity
└─────────────────┘
     │
     ▼
┌─────────────────┐
│ Death           │──▶ Server mode check → Spawns LootEntities → Clears Hotbar
└─────────────────┘
     │
     ▼
[InventoryUpdate to client]
```
