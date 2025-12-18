# Inventory Protocol Contract

**Feature**: 021-inventory-hotbar
**Version**: 1.0.0
**Date**: 2025-12-17

This document defines the network protocol extensions for the inventory hotbar system.

---

## Message Types Summary

| Direction | Message | Purpose |
|-----------|---------|---------|
| C→S | `SelectHotbarSlot` | Change active slot |
| C→S | `UseActiveItem` | Use item in active slot |
| S→C | `InventoryUpdate` | Sync hotbar state changes |
| S→C | `LootSpawned` (event) | Notify loot entity created |
| S→C | `LootRemoved` (event) | Notify loot entity removed |
| S→C | `LootPickedUp` (event) | Notify loot pickup |
| S→C | `ItemUsed` (event) | Notify item usage |

---

## Client → Server Messages

### SelectHotbarSlot

Select a different hotbar slot.

```rust
ClientMessage::SelectHotbarSlot {
    slot: u8,  // 0-indexed slot number
}
```

**Validation:**
- `slot < hotbar_capacity` (default 9)
- Rate limited by AntiCheat (ActionType::SlotSelect)

**Server Response:**
- On success: `InventoryUpdate` with new `active_slot`
- On failure (invalid slot): Silent reject, no response

---

### UseActiveItem

Use the item in the currently selected slot.

```rust
ClientMessage::UseActiveItem
```

**Validation:**
- Player not dead
- Match phase is `Playing`
- Active slot contains an item
- Rate limited by AntiCheat (ActionType::InventoryUse)

**Server Response:**
- On success: `ItemUsed` event + effect applied
- On failure: Silent reject, no response

**Effect Application by ItemKind:**

| ItemKind | Effect | Stack Change |
|----------|--------|--------------|
| Weapon | Trigger attack (existing combat) | None |
| Consumable | Apply heal to player | Decrement by 1 (remove if 0) |
| Tool | Place block at raycast target | None |

---

## Server → Client Messages

### InventoryUpdate

Synchronize hotbar state changes to client.

```rust
ServerMessage::InventoryUpdate {
    updates: Vec<SlotUpdate>,  // Changed slots only
    active_slot: u8,           // Currently selected slot
}

struct SlotUpdate {
    slot: u8,
    item: Option<ItemStack>,  // None = slot is empty
}

struct ItemStack {
    item_id: ItemId,  // u16
    quantity: u8,
}
```

**Sent When:**
- Player picks up item
- Player uses consumable (quantity change)
- Player slot selection changes
- Player respawns (loadout reset)
- Match starts (initial loadout)

**Optimization:**
- Only includes changed slots (diff-based)
- Batched per tick (not per operation)

---

## Game Events

### LootSpawned

Broadcast when loot entity is created in world.

```rust
GameEvent::LootSpawned {
    id: LootEntityId,    // u32
    position: Vec3,      // World position
    item_id: ItemId,     // u16
    quantity: u8,
}
```

**Sent When:**
- Player dies (mode-dependent)
- Arena loads with loot spawns
- Server spawns loot (admin/debug)

**Broadcast To:** All connected clients

---

### LootRemoved

Broadcast when loot entity is removed from world.

```rust
GameEvent::LootRemoved {
    id: LootEntityId,  // u32
}
```

**Sent When:**
- Loot picked up by player
- Loot despawns (timeout, if implemented)

**Broadcast To:** All connected clients

---

### LootPickedUp

Notify that a player picked up loot.

```rust
GameEvent::LootPickedUp {
    player_id: PlayerId,
    loot_id: LootEntityId,
    item_id: ItemId,
    quantity: u8,
}
```

**Sent When:**
- Server processes successful pickup

**Broadcast To:** All connected clients

---

### ItemUsed

Notify that a player used an item.

```rust
GameEvent::ItemUsed {
    player_id: PlayerId,
    item_id: ItemId,
}
```

**Sent When:**
- Player successfully uses consumable or tool
- NOT sent for weapons (covered by existing HitConfirmed)

**Broadcast To:** All connected clients (for visual effects)

---

## Snapshot Extension

### PlayerSnapshot.hotbar

Optional hotbar state in player snapshots.

```rust
struct PlayerSnapshot {
    // ... existing fields ...
    #[serde(default)]
    hotbar: Option<HotbarSnapshot>,
}

struct HotbarSnapshot {
    slots: Vec<Option<(ItemId, u8)>>,  // (item_id, quantity) pairs
    active_slot: u8,
}
```

**Included For:**
- The receiving player (their own hotbar)
- Spectate target (when spectating)

**NOT Included For:**
- Other players (privacy, bandwidth)

---

## Wire Format

All messages use existing bincode serialization.

### Size Estimates

| Message | Typical Size (bytes) |
|---------|---------------------|
| SelectHotbarSlot | 2 |
| UseActiveItem | 1 |
| InventoryUpdate (1 slot) | 6 |
| InventoryUpdate (full 9 slots) | 30 |
| LootSpawned | 18 |
| LootRemoved | 5 |

---

## Error Handling

| Scenario | Server Behavior |
|----------|-----------------|
| Invalid slot index | Silent reject |
| Use empty slot | Silent reject |
| Use while dead | Silent reject |
| Rate limit exceeded | AntiCheat strike + reject |
| Pickup race condition | First valid request wins |

---

## Sequence Diagrams

### Slot Selection

```
Client                    Server
  │                         │
  │ SelectHotbarSlot(3)     │
  │────────────────────────▶│
  │                         │ validate slot < capacity
  │                         │ update active_slot
  │      InventoryUpdate    │
  │◀────────────────────────│
  │  {updates:[], active:3} │
```

### Item Pickup (Automatic)

```
Client                    Server                   World
  │                         │                        │
  │      (move near loot)   │                        │
  │────────────────────────▶│                        │
  │                         │ check proximity        │
  │                         │ ≤1.5 blocks?           │
  │                         │──────────────────────▶ │
  │                         │     LootEntity         │
  │                         │◀──────────────────────│
  │                         │ add to hotbar          │
  │                         │ remove LootEntity      │
  │    InventoryUpdate      │                        │
  │◀────────────────────────│                        │
  │                         │                        │
  │    LootPickedUp (evt)   │                        │
  │◀────────────────────────│ (broadcast)            │
  │    LootRemoved (evt)    │                        │
  │◀────────────────────────│ (broadcast)            │
```

### Consumable Use

```
Client                    Server
  │                         │
  │    UseActiveItem        │
  │────────────────────────▶│
  │                         │ validate: alive, has item
  │                         │ apply effect (heal +50)
  │                         │ decrement quantity
  │                         │
  │    InventoryUpdate      │
  │◀────────────────────────│
  │  {slot 0: qty 2→1}      │
  │                         │
  │    ItemUsed (event)     │
  │◀────────────────────────│ (broadcast)
```
