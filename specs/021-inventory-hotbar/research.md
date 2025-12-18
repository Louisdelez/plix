# Research: Inventory Hotbar

**Feature**: 021-inventory-hotbar
**Date**: 2025-12-17

## Overview

This document captures research findings and technical decisions for the inventory hotbar implementation. All technical choices align with existing codebase patterns and constitution requirements.

---

## 1. Hotbar Data Structure

### Decision
Use a fixed-size `Vec<Option<ItemStack>>` with configurable capacity (default 9 slots).

### Rationale
- **O(1) access** by slot index - critical for real-time gameplay
- **Option<ItemStack>** naturally represents empty slots
- **Fixed capacity** prevents runtime allocation during gameplay
- Matches existing patterns (e.g., `Vec<PlayerSnapshot>` in replication)

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| HashMap<SlotIndex, ItemStack> | Overhead for small fixed-size collection |
| Array [Option<ItemStack>; 9] | Less flexible for configurable slot count |
| SmallVec | External dependency, marginal benefit |

---

## 2. Item Identification

### Decision
Use `ItemId(u16)` newtype with static `ItemDef` registry lookup.

### Rationale
- **16-bit ID** sufficient for thousands of item types (v1 needs only 3)
- **Newtype pattern** follows existing `PlayerId(u16)`, `EntityId(u32)` conventions
- **Static registry** allows O(1) lookup without runtime allocation
- Serializes efficiently with bincode (2 bytes)

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| String item names | Inefficient serialization, comparison overhead |
| Enum per item | Not extensible for future items/mods |
| UUID | Overkill for game items, 16 bytes vs 2 bytes |

---

## 3. Item Stack Representation

### Decision
```rust
pub struct ItemStack {
    pub item_id: ItemId,
    pub quantity: u8,
}
```

### Rationale
- **u8 quantity** supports max 255, far exceeds max_stack=16 requirement
- **Compact** - only 3 bytes per stack (2 for ID, 1 for quantity)
- **quantity == 0** is invalid state, enforced by constructor

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| u16 quantity | Overkill, wastes bytes in serialization |
| Separate stackable/non-stackable types | Unnecessary complexity |

---

## 4. Item Types and Effects

### Decision
Use `ItemKind` enum with variant-specific effect data:
```rust
pub enum ItemKind {
    Weapon { damage: u8, range: f32 },
    Consumable { heal: u8, max_stack: u8 },
    Tool { block_type: BlockType },
}
```

### Rationale
- **Variant-specific data** avoids Option proliferation
- **Closed enum** enables exhaustive match for effect application
- Aligns with existing patterns (`FlagState`, `GameMode` enums)

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Trait objects (dyn ItemEffect) | Runtime overhead, not needed for 3 item types |
| Component-based (ECS) | Over-engineering for simple inventory |
| JSON/TOML definitions | Adds deserialization complexity for v1 |

---

## 5. Loot Entity Management

### Decision
Server maintains `HashMap<LootEntityId, LootEntity>` with position + item data.

### Rationale
- **HashMap** enables O(1) lookup by ID for pickup validation
- **LootEntityId(u32)** matches existing `EntityId` pattern
- Server-authoritative ownership prevents race conditions

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| ECS loot components | Project doesn't use ECS architecture |
| Vec with linear search | Scales poorly with many loot items |

---

## 6. Network Protocol Messages

### Decision
Add to existing `ClientMessage` and `ServerMessage` enums:

**Client → Server:**
- `SelectHotbarSlot { slot: u8 }`
- `UseActiveItem`
- (Pickup is automatic based on position)

**Server → Client:**
- `InventoryUpdate { slots: Vec<SlotUpdate> }` (diff-based)
- `LootSpawned { id: LootEntityId, position: Vec3, item_id: ItemId, quantity: u8 }`
- `LootRemoved { id: LootEntityId }`

### Rationale
- **Diff-based updates** minimize network traffic (only changed slots)
- **Matches existing event pattern** (`GameEvent` for broadcasts)
- **Slot selection is intent** - server validates and applies

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Full hotbar snapshot every tick | Wasteful bandwidth |
| Client-side pickup with server validation | Violates server-authority principle |

---

## 7. Pickup Detection

### Decision
Server checks pickup proximity every tick for all players near loot entities.

### Rationale
- **Automatic pickup** (FR-005) requires server-side proximity check
- **1.5 block range** is a simple distance check: `(player_pos - loot_pos).length() <= 1.5`
- Reuses existing collision/distance patterns from combat system

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Client-initiated pickup request | Adds latency, enables exploits |
| Spatial hash for large loot counts | Over-engineering for expected loot density |

---

## 8. Death Drop Behavior

### Decision
Mode-specific death drop logic controlled by `GameMode` enum match:

| Mode | Drop Behavior |
|------|---------------|
| Training | No drop (keep loadout) |
| TDM | No drop (team respawn) |
| FFA | Drop all items |
| CTF | Drop items (except flag) |
| BR Lite | Drop all items |

### Rationale
- **Mode-specific behavior** matches existing pattern in match_state.rs
- **Training/TDM no-drop** preserves intended respawn experience
- **FFA/BR drop** enables loot-based gameplay

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Config flag per arena | Over-engineering for clear mode semantics |
| Always drop | Breaks Training/TDM game design |

---

## 9. Starting Loadout

### Decision
Per-mode default loadouts defined in `InventoryConfig`, overridable by arena TOML.

**Defaults:**
- Training: Sword (slot 0), Health Pack x3 (slot 1)
- TDM/FFA: Sword (slot 0)
- CTF: Sword (slot 0)
- BR Lite: Empty (loot-based)

### Rationale
- **Mode defaults** provide sensible experience without config
- **Arena override** enables customization (US7)
- Follows existing pattern of mode-specific behavior

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| No defaults (always empty) | Poor UX for non-BR modes |
| Hard-coded per mode | Less flexible for arena designers |

---

## 10. AntiCheat Integration

### Decision
Add `ActionType::InventoryUse` and `ActionType::SlotSelect` to existing rate limiter.

### Rationale
- **Reuses existing AntiCheat infrastructure** (Feature 007)
- **Rate limits** prevent spam abuse (e.g., rapid item use)
- Follows established pattern in anti_cheat/state.rs

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Separate inventory-specific rate limiter | Code duplication |
| No rate limiting | Enables exploit vectors |

---

## Summary

All technical decisions follow existing codebase patterns and constitution principles. No external dependencies required. Implementation can proceed to Phase 1 (data model and contracts).
