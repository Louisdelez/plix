# Feature Specification: Inventory Hotbar

**Feature Branch**: `021-inventory-hotbar`
**Created**: 2025-12-17
**Status**: Draft
**Input**: User description: "Système d'inventaire minimal basé sur une hotbar avec slots fixes, types d'items (Weapon, Consumable, Tool), logique server-authoritative, sélection de slot et utilisation, stacking pour consumables, intégration loot pickup, et compatibilité avec tous les modes de jeu."

## Clarifications

### Session 2025-12-17

- Q: What should the maximum stack size be for consumable items? → A: 16 (standard, balanced gameplay)
- Q: What specific items should be defined for the initial implementation? → A: Minimal set: Sword (25dmg), Health Pack (+50HP), Block Placer

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic Hotbar Display and Slot Selection (Priority: P1)

As a player, I want to see my hotbar with available slots and select different items so that I can quickly switch between equipment during gameplay.

**Why this priority**: This is the foundation of the inventory system. Without hotbar display and slot selection, no other inventory features can function. It provides immediate visual feedback and basic interaction.

**Independent Test**: Can be fully tested by spawning a player, rendering the hotbar UI, and using keyboard (1-9) or scroll wheel to select slots. Delivers value by allowing players to see and interact with their equipment.

**Acceptance Scenarios**:

1. **Given** a player spawns in any game mode, **When** the game renders, **Then** a hotbar with 5-9 slots (configurable) is displayed at the bottom of the screen with the first slot selected.
2. **Given** a player has a hotbar displayed, **When** they press keys 1-9, **Then** the corresponding slot becomes selected (highlighted).
3. **Given** a player has a hotbar displayed, **When** they scroll the mouse wheel up/down, **Then** the selection cycles through slots (wrapping around).
4. **Given** a player has items in their hotbar, **When** a slot is selected, **Then** that slot shows visual highlighting and the item icon/count is visible.

---

### User Story 2 - Item Pickup and Hotbar Population (Priority: P1)

As a player, I want to pick up items from the world and have them automatically added to my hotbar so that I can collect equipment during matches.

**Why this priority**: Acquiring items is essential for gameplay. Without pickup functionality, players cannot obtain equipment, making the inventory system useless.

**Independent Test**: Can be tested by placing a loot item in the world, having a player walk over it, and verifying the item appears in the hotbar. Delivers value by enabling item collection.

**Acceptance Scenarios**:

1. **Given** a player is near a loot item on the ground, **When** they move within pickup range (1.5 blocks), **Then** the item is automatically picked up and added to the first available hotbar slot.
2. **Given** a player's hotbar is full with non-stackable items, **When** they try to pick up a new item, **Then** the pickup fails and the item remains on the ground.
3. **Given** a player has a consumable with count < max_stack in their hotbar, **When** they pick up the same consumable type, **Then** the items stack (count increases) instead of using a new slot.
4. **Given** a player picks up an item, **When** the server processes the pickup, **Then** a ServerMessage::InventoryUpdate is sent to sync the client state.

---

### User Story 3 - Item Usage (Priority: P1)

As a player, I want to use items from my selected hotbar slot so that I can attack with weapons, consume health items, or use tools.

**Why this priority**: Item usage is the primary purpose of having an inventory. Without it, collected items provide no gameplay value.

**Independent Test**: Can be tested by selecting a weapon slot and clicking to attack, or selecting a consumable and pressing a use key to consume it. Delivers combat and survival functionality.

**Acceptance Scenarios**:

1. **Given** a player has a weapon selected, **When** they perform an attack action, **Then** the attack uses the equipped weapon's damage value (not default melee).
2. **Given** a player has a consumable selected (e.g., health pack), **When** they press the use key, **Then** the consumable effect is applied (e.g., health restored) and the stack count decreases by 1.
3. **Given** a consumable stack count reaches 0, **When** the last item is used, **Then** the slot becomes empty.
4. **Given** a player has a tool selected (e.g., block placer), **When** they use it, **Then** the tool's action is performed (e.g., place specific block type).
5. **Given** a player with an empty slot selected, **When** they try to attack/use, **Then** default melee attack is performed (or action is blocked for tools).

---

### User Story 4 - Server-Authoritative Inventory Validation (Priority: P2)

As the server, I want to validate all inventory operations so that I can prevent cheating and maintain game integrity.

**Why this priority**: Essential for multiplayer fairness, but can be implemented after basic functionality works in single-player/training mode.

**Independent Test**: Can be tested by simulating a malicious client sending invalid inventory commands and verifying the server rejects them. Delivers anti-cheat protection.

**Acceptance Scenarios**:

1. **Given** a client sends a pickup request for an item that doesn't exist, **When** the server validates, **Then** the request is rejected and no inventory change occurs.
2. **Given** a client sends a use request for an item they don't have, **When** the server validates, **Then** the request is rejected and a warning is logged.
3. **Given** a client sends an invalid slot index, **When** the server validates, **Then** the request is rejected.
4. **Given** a client attempts to pick up an item already picked up by another player, **When** the server processes both requests, **Then** only the first valid request succeeds.

---

### User Story 5 - Item Drops on Death (Priority: P2)

As a player, when I die I want my items to drop so that other players can pick them up (in applicable game modes).

**Why this priority**: Adds tactical depth to combat and enables loot-based gameplay in BR Lite and FFA modes.

**Independent Test**: Can be tested by killing a player with items and verifying dropped loot entities spawn at death location. Delivers looting gameplay.

**Acceptance Scenarios**:

1. **Given** a player dies with items in their hotbar, **When** death is processed, **Then** each item is spawned as a loot entity at the death location.
2. **Given** a player dies in Training mode, **When** death is processed, **Then** items are NOT dropped (respawn with same loadout).
3. **Given** a player dies in TDM mode, **When** death is processed, **Then** items are NOT dropped (team-based respawn with loadout).
4. **Given** loot is dropped, **When** items spawn, **Then** they are spread slightly (0.5-1.0 block radius) to prevent overlap.

---

### User Story 6 - Game Mode Compatibility (Priority: P2)

As a game developer, I want the inventory system to work correctly with all game modes so that each mode has appropriate item mechanics.

**Why this priority**: Ensures the system integrates with existing game modes without breaking them.

**Independent Test**: Can be tested by starting each game mode and verifying inventory behavior matches mode requirements. Delivers cross-mode compatibility.

**Acceptance Scenarios**:

1. **Given** Training mode starts, **When** player spawns, **Then** player starts with default loadout (configurable: weapon + consumables).
2. **Given** TDM/FFA mode starts, **When** player spawns, **Then** player starts with mode-specific loadout.
3. **Given** BR Lite mode starts, **When** player spawns, **Then** player starts with empty hotbar (must find loot).
4. **Given** CTF mode starts, **When** player has flag, **Then** flag occupies a special slot (cannot be dropped via normal means).
5. **Given** any mode with configurable loadouts, **When** arena is loaded, **Then** spawn loadout is read from arena config.

---

### User Story 7 - Hotbar Configuration (Priority: P3)

As a game designer, I want to configure hotbar size and default loadouts per arena/mode so that different modes can have different item economies.

**Why this priority**: Nice to have for customization but not essential for core functionality.

**Independent Test**: Can be tested by modifying arena TOML configuration and verifying hotbar size changes. Delivers design flexibility.

**Acceptance Scenarios**:

1. **Given** an arena TOML with `hotbar_slots = 5`, **When** match starts, **Then** players have 5 hotbar slots.
2. **Given** an arena TOML with `default_loadout = [...]`, **When** player spawns, **Then** player's hotbar is populated with specified items.
3. **Given** no hotbar configuration in arena, **When** match starts, **Then** default values are used (9 slots, empty or mode-default loadout).

---

### Edge Cases

- What happens when a player tries to pick up an item while hotbar is full with non-stackable items? → Pickup fails, item stays on ground.
- What happens when two players try to pick up the same item simultaneously? → Server resolves race condition, first valid request wins.
- What happens when a consumable stack exceeds max_stack during pickup? → Item splits: fills current stack to max, remaining becomes new stack or stays on ground.
- What happens when player disconnects with items? → Items are lost (no persistence in current scope).
- What happens when a player tries to use an empty slot? → Default melee for attack, no-op for tool use.
- What happens with invalid item IDs in network messages? → Server rejects and logs anti-cheat warning.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a hotbar with configurable slot count (5-9 slots).
- **FR-002**: System MUST support three item types: Weapon, Consumable, Tool.
- **FR-003**: System MUST allow slot selection via keyboard (1-9) and mouse scroll wheel.
- **FR-004**: System MUST render hotbar UI at bottom of screen with item icons and stack counts.
- **FR-005**: System MUST implement automatic pickup when player is within 1.5 blocks of loot.
- **FR-006**: System MUST support item stacking for Consumable type up to max_stack of 16.
- **FR-007**: System MUST validate all inventory operations server-side before applying.
- **FR-008**: System MUST sync inventory state via ServerMessage::InventoryUpdate.
- **FR-009**: System MUST apply item effects on use (weapon damage, consumable effects, tool actions).
- **FR-010**: System MUST handle item drops on death according to game mode rules.
- **FR-011**: System MUST prevent duplicate pickups via server-authoritative loot ownership.
- **FR-012**: System MUST integrate with existing AntiCheat rate limiting.

### Key Entities

- **Item**: Represents an item instance with type (Weapon/Consumable/Tool), item_id, damage/effect values, and stack count.
- **ItemType**: Enum defining item categories with different behaviors (stackable, usable, equippable).
- **Hotbar**: Player's equipped item slots, ordered array of Option<Item>.
- **LootEntity**: World entity representing dropped/spawned items available for pickup.
- **InventoryUpdate**: Network message containing slot changes for client sync.

### Initial Item Definitions

| Item ID | Name | Type | Properties |
|---------|------|------|------------|
| 1 | Sword | Weapon | damage: 25, range: 2.0 blocks |
| 2 | Health Pack | Consumable | heal: +50 HP, max_stack: 16 |
| 3 | Block Placer | Tool | places: Stone block |

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can select any hotbar slot in under 100ms using keyboard or scroll wheel.
- **SC-002**: Item pickups are processed and synced to client within 1 tick (16.67ms at 60Hz).
- **SC-003**: All inventory operations are validated server-side with 0 client-authoritative exploits possible.
- **SC-004**: Hotbar renders at 60fps without performance degradation.
- **SC-005**: All existing game modes (Training, TDM, FFA, CTF, BR Lite) function correctly with inventory system.
- **SC-006**: 100% of integration tests pass covering all user scenarios and edge cases.
