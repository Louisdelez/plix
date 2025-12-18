# Quickstart: Inventory Hotbar Implementation

**Feature**: 021-inventory-hotbar
**Date**: 2025-12-17

This guide provides a rapid implementation path for the inventory hotbar system.

---

## Prerequisites

- Rust 1.75+ (stable)
- Existing plix workspace with plix-common, plix-server, plix-client crates
- Familiarity with existing protocol (ClientMessage, ServerMessage, GameEvent)

---

## Implementation Order

### Phase 1: Core Types (plix-common)

**Files to create/modify:**

1. `crates/plix-common/src/types.rs` - Add `ItemId`, `LootEntityId`
2. `crates/plix-common/src/inventory/mod.rs` - New module
3. `crates/plix-common/src/inventory/item.rs` - `ItemKind`, `ItemDef`
4. `crates/plix-common/src/inventory/item_stack.rs` - `ItemStack`
5. `crates/plix-common/src/inventory/hotbar.rs` - `Hotbar`
6. `crates/plix-common/src/lib.rs` - Export inventory module

**Minimal code to get started:**

```rust
// crates/plix-common/src/inventory/mod.rs
pub mod item;
pub mod item_stack;
pub mod hotbar;

pub use item::{ItemDef, ItemId, ItemKind};
pub use item_stack::ItemStack;
pub use hotbar::Hotbar;
```

### Phase 2: Protocol Messages (plix-common)

**File to modify:**
- `crates/plix-common/src/protocol/messages.rs`

**Add to ClientMessage:**
```rust
SelectHotbarSlot { slot: u8 },
UseActiveItem,
```

**Add to ServerMessage:**
```rust
InventoryUpdate {
    updates: Vec<SlotUpdate>,
    active_slot: u8,
},
```

**Add new struct:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotUpdate {
    pub slot: u8,
    pub item: Option<ItemStack>,
}
```

**Add to GameEvent:**
```rust
LootSpawned { id: LootEntityId, position: Vec3, item_id: ItemId, quantity: u8 },
LootRemoved { id: LootEntityId },
LootPickedUp { player_id: PlayerId, loot_id: LootEntityId, item_id: ItemId, quantity: u8 },
ItemUsed { player_id: PlayerId, item_id: ItemId },
```

### Phase 3: Server Inventory Logic (plix-server)

**Files to create:**

1. `crates/plix-server/src/inventory/mod.rs`
2. `crates/plix-server/src/inventory/config.rs`
3. `crates/plix-server/src/inventory/player_inventory.rs`
4. `crates/plix-server/src/inventory/item_registry.rs`
5. `crates/plix-server/src/inventory/use_system.rs`
6. `crates/plix-server/src/inventory/pickup_system.rs`

**Item Registry (static):**
```rust
// crates/plix-server/src/inventory/item_registry.rs
use plix_common::inventory::{ItemDef, ItemId, ItemKind};
use plix_common::types::BlockType;

pub fn get_item_def(id: ItemId) -> Option<&'static ItemDef> {
    match id {
        ItemId::SWORD => Some(&SWORD_DEF),
        ItemId::HEALTH_PACK => Some(&HEALTH_PACK_DEF),
        ItemId::BLOCK_PLACER => Some(&BLOCK_PLACER_DEF),
        _ => None,
    }
}

static SWORD_DEF: ItemDef = ItemDef {
    id: ItemId::SWORD,
    name: "Sword",
    kind: ItemKind::Weapon { damage: 25, range: 2.0 },
};

static HEALTH_PACK_DEF: ItemDef = ItemDef {
    id: ItemId::HEALTH_PACK,
    name: "Health Pack",
    kind: ItemKind::Consumable { heal: 50, max_stack: 16 },
};

static BLOCK_PLACER_DEF: ItemDef = ItemDef {
    id: ItemId::BLOCK_PLACER,
    name: "Block Placer",
    kind: ItemKind::Tool { block_type: BlockType::STONE },
};
```

### Phase 4: Loot Entity System (plix-server)

**Files to create:**

1. `crates/plix-server/src/loot/mod.rs`
2. `crates/plix-server/src/loot/entity.rs`
3. `crates/plix-server/src/loot/spawner.rs`

**Loot Manager:**
```rust
// crates/plix-server/src/loot/mod.rs
use std::collections::HashMap;
use plix_common::types::LootEntityId;

pub struct LootManager {
    entities: HashMap<LootEntityId, LootEntity>,
    next_id: u32,
}

impl LootManager {
    pub fn spawn(&mut self, position: Vec3, item: ItemStack, tick: Tick) -> LootEntityId {
        let id = LootEntityId(self.next_id);
        self.next_id += 1;
        self.entities.insert(id, LootEntity::new(id, position, item, tick));
        id
    }

    pub fn remove(&mut self, id: LootEntityId) -> Option<LootEntity> {
        self.entities.remove(&id)
    }

    pub fn find_near(&self, position: Vec3, range: f32) -> Option<LootEntityId> {
        self.entities.iter()
            .find(|(_, e)| (e.position - position).length() <= range)
            .map(|(id, _)| *id)
    }
}
```

### Phase 5: Game Loop Integration (plix-server)

**File to modify:**
- `crates/plix-server/src/lib.rs`

**Key integration points:**

1. **Add Hotbar to ServerPlayer:**
```rust
pub struct ServerPlayer {
    // ... existing fields ...
    pub hotbar: Hotbar,
}
```

2. **Handle ClientMessage variants:**
```rust
ClientMessage::SelectHotbarSlot { slot } => {
    self.handle_select_slot(player_id, slot);
}
ClientMessage::UseActiveItem => {
    self.handle_use_item(player_id);
}
```

3. **Pickup check in tick loop:**
```rust
// After movement processing
self.check_pickups();
```

4. **Death drop logic:**
```rust
// In handle_player_death
if mode_drops_items(self.game_mode) {
    self.drop_player_items(player_id);
}
```

5. **Spawn loadout on respawn:**
```rust
// In handle_player_respawn
self.give_starting_loadout(player_id);
```

### Phase 6: AntiCheat Integration

**File to modify:**
- `crates/plix-server/src/anti_cheat/mod.rs`

**Add action types:**
```rust
pub enum ActionType {
    // ... existing ...
    SlotSelect,
    InventoryUse,
}
```

### Phase 7: Client UI (plix-client)

**File to create:**
- `crates/plix-client/src/ui/hotbar.rs`

**Minimal hotbar rendering:**
```rust
pub fn render_hotbar(
    hotbar: &Hotbar,
    item_registry: &impl Fn(ItemId) -> Option<&ItemDef>,
    screen_width: f32,
    screen_height: f32,
) -> Vec<UIRect> {
    let slot_size = 48.0;
    let padding = 4.0;
    let total_width = hotbar.capacity() as f32 * (slot_size + padding);
    let start_x = (screen_width - total_width) / 2.0;
    let y = screen_height - slot_size - 20.0;

    let mut rects = Vec::new();
    for i in 0..hotbar.capacity() {
        let x = start_x + i as f32 * (slot_size + padding);
        let is_active = i == hotbar.active_slot();

        // Background rect
        rects.push(UIRect {
            x, y,
            width: slot_size,
            height: slot_size,
            color: if is_active { ACTIVE_COLOR } else { INACTIVE_COLOR },
        });

        // Item icon + quantity (if present)
        if let Some(stack) = hotbar.get_slot(i) {
            // Render item icon and quantity text
        }
    }
    rects
}
```

---

## Testing Checklist

### Unit Tests

- [ ] `Hotbar::select_slot` validates bounds
- [ ] `Hotbar::find_stackable_slot` finds correct slot
- [ ] `ItemStack::try_add` respects max_stack
- [ ] `ItemStack::remove` returns true when empty

### Integration Tests

- [ ] Player receives starting loadout on spawn
- [ ] Slot selection syncs to client
- [ ] Pickup adds item to hotbar
- [ ] Consumable use heals and decrements
- [ ] Death drops items (FFA/BR modes)
- [ ] Death keeps items (Training/TDM modes)

---

## Common Gotchas

1. **Forgot to export inventory module** - Add `pub mod inventory;` to lib.rs
2. **Serde derive missing** - Ensure `#[derive(Serialize, Deserialize)]` on all protocol types
3. **Pickup range units** - 1.5 is in blocks, not world units (same thing in this codebase)
4. **Active slot invariant** - Always ensure `active_slot < capacity` after any operation
5. **Empty ItemStack** - Never create `ItemStack` with `quantity = 0`

---

## Verification Commands

```bash
# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Check inventory-specific tests
cargo test -p plix-server inventory

# Clippy check
cargo clippy --workspace -- -D warnings
```

---

## Next Steps

After implementation:
1. Run `/speckit.tasks` to generate detailed implementation tasks
2. Create tests first (TDD approach recommended)
3. Implement in order: types → protocol → server → client
