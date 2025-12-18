# Feature Specification: Crafting Lite

**Feature Branch**: `023-crafting-lite`
**Created**: 2025-12-17
**Status**: Draft
**Input**: User description: "Système de crafting minimaliste et serveur-autoritaire permettant au joueur de fabriquer quelques items utiles via des recettes simples, sans inventaire complexe ni UI lourde, et intégré à la hotbar existante."

## Clarifications

### Session 2025-12-17

- Q: How does the client trigger a craft request? → A: Console command only (e.g., `/craft health_pack`) - debug-friendly MVP approach with no UI required.
- Q: Is there a cooldown between craft attempts? → A: 1 second cooldown between successful crafts per player (anti-spam, no cooldown on failures).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Simple Item Crafting (Priority: P1)

As a player, I want to craft a useful item (like a health pack or weapon) by requesting a recipe when I have the required ingredients in my hotbar, so that I can convert looted resources into tactical advantages without navigating complex menus.

**Why this priority**: Core functionality - without this, there is no crafting system. This enables the fundamental crafting loop.

**Independent Test**: Can be fully tested by having a player with required ingredients request a craft and verifying the ingredients are consumed and output is received.

**Acceptance Scenarios**:

1. **Given** a player with 2x SCRAP items in their hotbar, **When** they request the "craft health pack" recipe, **Then** 2x SCRAP is consumed and 1x HEALTH_PACK is added to their hotbar.
2. **Given** a player with only 1x SCRAP in their hotbar, **When** they request the "craft health pack" recipe, **Then** the craft fails with "insufficient ingredients" and their hotbar remains unchanged.
3. **Given** a player with required ingredients, **When** they request an unknown recipe, **Then** the craft fails with "unknown recipe" and their hotbar remains unchanged.

---

### User Story 2 - Server-Authoritative Validation (Priority: P1)

As a server, I must validate all craft requests to ensure players have required ingredients and can receive the output, maintaining game integrity and preventing exploits.

**Why this priority**: Security-critical - the server must be authoritative over all crafting operations to prevent cheating.

**Independent Test**: Can be tested by sending invalid craft requests and verifying the server rejects them appropriately.

**Acceptance Scenarios**:

1. **Given** a craft request with invalid recipe ID, **When** the server processes it, **Then** the request is rejected with "unknown recipe" error.
2. **Given** a craft request where player lacks ingredients, **When** the server processes it, **Then** the request is rejected with "insufficient ingredients" error and no items are consumed.
3. **Given** a craft request where player's hotbar is full and output cannot stack, **When** the server processes it, **Then** the request is rejected with "inventory full" error and no items are consumed.

---

### User Story 3 - Atomic Craft Operations (Priority: P1)

As a player, I expect crafting to be all-or-nothing - either my ingredients are consumed and I receive the output, or nothing changes. Partial states must never occur.

**Why this priority**: Data integrity - partial crafts would corrupt player inventories and create inconsistent game states.

**Independent Test**: Can be tested by simulating craft failures at various stages and verifying inventory consistency.

**Acceptance Scenarios**:

1. **Given** a valid craft request that succeeds, **When** the server processes it, **Then** all ingredients are consumed AND output is added in a single atomic operation.
2. **Given** a craft request that fails validation, **When** the server processes it, **Then** no ingredients are consumed AND no output is added.

---

### User Story 4 - Game Mode Configuration (Priority: P2)

As a server administrator, I want to enable or disable crafting per game mode and control which recipes are available, so that each mode can have appropriate crafting rules.

**Why this priority**: Enables customization per game mode, but basic crafting can work with defaults initially.

**Independent Test**: Can be tested by verifying crafting is enabled/disabled based on mode configuration.

**Acceptance Scenarios**:

1. **Given** a Training mode match with crafting enabled, **When** a player requests a craft, **Then** the craft is processed normally.
2. **Given** a TDM match with crafting disabled (default), **When** a player requests a craft, **Then** the craft is rejected with "crafting disabled" error.
3. **Given** a BR Lite match with limited recipes, **When** a player requests a disabled recipe, **Then** the craft is rejected with "recipe not available" error.

---

### User Story 5 - Resource Items for Crafting (Priority: P2)

As a player, I want to find resource items (like SCRAP) as loot that I can convert into useful equipment through crafting.

**Why this priority**: Provides crafting ingredients - can be tested once basic crafting works.

**Independent Test**: Can be tested by verifying resource items spawn as loot and can be picked up.

**Acceptance Scenarios**:

1. **Given** a BR Lite arena with resource loot spawns, **When** a player approaches a SCRAP loot drop, **Then** they can pick it up and it appears in their hotbar.
2. **Given** a Training mode, **When** a player spawns, **Then** they receive some starter resources for testing crafts.

---

### User Story 6 - Crafting Feedback (Priority: P3)

As a player, I want clear feedback when I attempt to craft, so I know whether it succeeded or why it failed.

**Why this priority**: Quality of life - improves user experience but not required for basic functionality.

**Independent Test**: Can be tested by verifying appropriate events are sent to clients on craft attempts.

**Acceptance Scenarios**:

1. **Given** a successful craft, **When** the server processes it, **Then** the player receives a "craft success" event with the crafted item info.
2. **Given** a failed craft due to insufficient ingredients, **When** the server processes it, **Then** the player receives a "craft failed" event with the reason.

---

### User Story 7 - Extensible Recipe System (Priority: P3)

As a developer, I want recipes defined in a registry that can be easily extended later, so new recipes can be added without major refactoring.

**Why this priority**: Developer experience - enables future expansion but not user-facing.

**Independent Test**: Can be tested by adding a new recipe to the registry and verifying it works.

**Acceptance Scenarios**:

1. **Given** a recipe registry with existing recipes, **When** a new recipe is added to the registry, **Then** players can craft it immediately without code changes to the crafting logic.

---

### Edge Cases

- What happens when a player tries to craft while dead? Craft is rejected.
- What happens when ingredients are spread across multiple hotbar slots? System consumes from any slots containing the required items.
- What happens when output item can partially stack with existing stack? System fills existing stack first, then uses empty slot for remainder if available.
- What happens when craft request arrives during respawn? Craft is rejected until player is alive.
- What happens when two craft requests arrive simultaneously? Server processes them sequentially, second may fail if first consumed ingredients.
- What happens when a player spams craft requests? 1-second cooldown after each successful craft; requests during cooldown are rejected with "cooldown active" error.

## Requirements *(mandatory)*

### Functional Requirements

#### Recipe System

- **FR-001**: System MUST provide a recipe registry containing all available crafting recipes.
- **FR-002**: Each recipe MUST define a unique recipe identifier.
- **FR-003**: Each recipe MUST define required ingredients as a list of (ItemId, quantity) pairs.
- **FR-004**: Each recipe MUST define exactly one output as an (ItemId, quantity) pair.
- **FR-005**: Recipe definitions MUST be immutable during gameplay.

#### Craft Request Processing

- **FR-006**: System MUST accept craft requests containing a recipe identifier from clients.
- **FR-007**: System MUST validate that the requested recipe exists in the registry.
- **FR-008**: System MUST validate that the player possesses all required ingredients across their hotbar.
- **FR-009**: System MUST validate that the output can be added to the player's hotbar (stacking or empty slot).
- **FR-010**: System MUST reject craft requests from dead players.
- **FR-011**: System MUST execute valid crafts atomically (all-or-nothing).
- **FR-012**: System MUST consume ingredients from hotbar slots when craft succeeds.
- **FR-013**: System MUST add output to player's hotbar when craft succeeds (stacking with existing compatible stack first, then empty slot).
- **FR-035**: System MUST enforce a 1-second cooldown between successful crafts per player.
- **FR-036**: System MUST reject craft requests during cooldown with "cooldown active" error.
- **FR-037**: Failed craft attempts MUST NOT trigger or extend the cooldown.

#### Game Mode Integration

- **FR-014**: System MUST support enabling/disabling crafting per game mode.
- **FR-015**: System MUST support restricting available recipes per game mode.
- **FR-016**: Training mode MUST have crafting enabled by default.
- **FR-017**: BR Lite mode MUST have crafting enabled by default.
- **FR-018**: TDM, FFA, and CTF modes MUST have crafting disabled by default.

#### Resource Items

- **FR-019**: System MUST define at least one resource item type (SCRAP) for crafting ingredients.
- **FR-020**: Resource items MUST be lootable in BR Lite arenas.
- **FR-021**: Resource items MUST be included in Training mode starter loadout (quantity: 5).
- **FR-022**: Resource items MUST be stackable (max stack: 16).

#### Initial Recipes (v1)

- **FR-023**: System MUST include recipe: 2x SCRAP → 1x HEALTH_PACK.
- **FR-024**: System MUST include recipe: 4x SCRAP → 1x BOW.
- **FR-025**: System MUST include recipe: 3x SCRAP → 1x SWORD.

#### Client Communication

- **FR-026**: System MUST define a CraftRequest message (recipe_id) from client to server.
- **FR-027**: System MUST define a CraftResult event (success/failure, reason, output_item) from server to client.
- **FR-028**: System MUST send CraftResult to the requesting player after processing.
- **FR-033**: Client MUST provide a console command interface for crafting (e.g., `/craft <recipe_name>`).
- **FR-034**: Client MUST NOT require UI or keybinds for crafting in v1 (console-only MVP).

#### Observability

- **FR-029**: System MUST track total successful crafts (metric: crafts_total).
- **FR-030**: System MUST track failed crafts by reason (metric: crafts_failed_total with reason label).
- **FR-031**: System MUST log craft success events (player_id, recipe_id, tick).
- **FR-032**: System MUST log craft failure events (player_id, recipe_id, reason, tick).

### Key Entities

- **Recipe**: Defines a crafting transformation with unique ID, list of required ingredients (ItemId + quantity), and single output (ItemId + quantity).
- **RecipeRegistry**: Immutable collection of all recipes indexed by recipe ID, with query methods.
- **CraftRequest**: Client message containing recipe_id to attempt crafting.
- **CraftResult**: Server event indicating success/failure with reason and crafted item details.
- **CraftConfig**: Per-mode configuration controlling crafting enabled state and allowed recipe list.
- **SCRAP (ItemId)**: New resource item type used as crafting ingredient, stackable to 16.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can complete a craft operation in under 100ms from request to hotbar update.
- **SC-002**: System maintains 100% atomicity - no partial craft states ever occur under any failure condition.
- **SC-003**: All craft validation (ingredients, output space, permissions) completes before any inventory modifications.
- **SC-004**: Server rejects 100% of invalid craft requests (unknown recipe, insufficient ingredients, full inventory, wrong mode).
- **SC-005**: Crafting operations have no measurable impact on server tick rate (< 1% increase in tick time).
- **SC-006**: All craft operations (success and failure) are logged with full context for debugging.
- **SC-007**: Recipe system supports adding new recipes without modifying core crafting logic.

## Assumptions

- The existing Hotbar from Feature 021 provides methods to query item quantities, consume items, and add items.
- ItemId can be extended with new constants (SCRAP) without breaking existing code.
- The item registry can be extended with a new SCRAP item definition.
- Game mode configuration already exists and can be extended with crafting-related flags.
- The existing loot spawning system can spawn new item types (SCRAP) without modification.
- Training loadout configuration can be extended to include SCRAP items.

## Out of Scope

- Crafting grid (2x2, 3x3) - recipes are direct item-to-item transformations.
- Crafting stations or workbenches - all crafting is portable/anywhere.
- Tech tree or recipe unlocking - all recipes available immediately.
- Cross-match persistence - crafted items reset each match.
- Advanced UI (drag-and-drop, crafting preview) - simple request-based system.
- Multiple recipe outputs - each recipe produces exactly one output type.
- Recipe discovery or hints - players must know recipes.
