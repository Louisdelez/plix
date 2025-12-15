# Feature Specification: Server-Authoritative Combat System

**Feature Branch**: `003-combat-visible`
**Created**: 2025-12-14
**Status**: Draft
**Input**: User description: "Server-authoritative combat visible: attack input, hit validation, HP updates, death/respawn, minimal HUD feedback. No CEF."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Attack Another Player (Priority: P1)

As a player in a voxel arena, I can attack another player and receive immediate visual feedback when my attack lands, proving the combat system works end-to-end.

**Why this priority**: This is the core combat loop - without attack and hit feedback, no other combat features have meaning. Validates the entire client-server-client round trip for combat.

**Independent Test**: Can be tested by two players in an arena - one attacks, both see visual confirmation when a hit is registered by the server.

**Acceptance Scenarios**:

1. **Given** two players are in the arena within attack range, **When** player A presses the attack key, **Then** the server validates the attack and both players see hit feedback if successful
2. **Given** player A is in the arena, **When** player A attacks but no enemy is in range, **Then** no hit feedback is shown and no damage is applied
3. **Given** player A has just attacked, **When** player A attacks again before cooldown expires, **Then** the attack is ignored by the server

---

### User Story 2 - Kill and Respawn Flow (Priority: P2)

As a player, when I deal the final blow to another player, their death is clearly communicated and they respawn after a fixed delay, proving the full death/respawn cycle works.

**Why this priority**: Death and respawn are essential for any combat game loop. Without them, combat has no consequence. This validates HP tracking and player state management.

**Independent Test**: Can be tested by having one player repeatedly attack another until death occurs, observing the death notification, and confirming respawn happens at a spawn point.

**Acceptance Scenarios**:

1. **Given** a player has low HP, **When** they receive a hit that reduces HP to 0, **Then** the server declares them dead and broadcasts a death event to all clients
2. **Given** a player has died, **When** the respawn delay elapses, **Then** the player respawns at a spawn point with full HP
3. **Given** a player is dead (before respawn), **When** they try to attack, **Then** their attack input is ignored by the server

---

### User Story 3 - View Local Player HP (Priority: P3)

As a player, I can see my current HP displayed in the debug HUD, allowing me to understand my combat state without guessing.

**Why this priority**: HP visibility is important for gameplay awareness but not strictly required to prove combat works. The debug HUD already exists; this extends it.

**Independent Test**: Can be tested by observing the HUD while taking damage - HP value should decrease each time a hit is registered.

**Acceptance Scenarios**:

1. **Given** a player is in the arena with full HP, **When** they look at the debug HUD, **Then** their current HP is displayed
2. **Given** a player takes damage, **When** the server confirms the hit, **Then** the HUD HP value updates to reflect the new HP

---

### User Story 4 - Observe Combat Events (Priority: P3)

As a developer or tester, I can see combat events (hits, kills) in the debug HUD, allowing me to verify that the server-client event flow is working correctly.

**Why this priority**: Debugging visibility is essential for development but not part of the core combat loop. Extends the existing debug HUD.

**Independent Test**: Can be tested by triggering combat actions and observing that event messages appear in the HUD.

**Acceptance Scenarios**:

1. **Given** player A hits player B, **When** the hit is validated, **Then** both clients display a hit event in the HUD
2. **Given** player A kills player B, **When** the death occurs, **Then** all clients display a kill event (e.g., "Player A killed Player B")

---

### Edge Cases

- What happens when a player attacks at the exact moment they die? → The attack is rejected because dead players cannot attack
- What happens when two players kill each other simultaneously? → Both deaths are processed; both respawn independently after their respective delays
- What happens when a player disconnects during the respawn delay? → The respawn is cancelled; player state is cleaned up
- How does the system handle attack input spam? → Cooldown enforced server-side; excess inputs are ignored
- What happens when a player attacks during the match countdown/warmup phase? → Attacks are rejected until the match phase is "playing"
- What happens if network latency is very high (500ms+)? → Hits are still validated correctly server-side; visual feedback may be delayed but state remains consistent

## Requirements *(mandatory)*

### Functional Requirements

#### Attack Input (Client)
- **FR-001**: Client MUST send an attack action to the server when the player presses the attack key (mouse click or dedicated key)
- **FR-002**: Client MUST NOT perform any hit detection or damage calculation locally
- **FR-003**: Client MUST send attack input as a discrete event (not continuous)

#### Server-Side Combat Validation
- **FR-004**: Server MUST validate each attack against range and facing direction (closest enemy in cone)
- **FR-005**: Server MUST enforce attack cooldown per player
- **FR-006**: Server MUST reject attacks from dead or respawning players
- **FR-007**: Server MUST be the sole authority on whether a hit is valid
- **FR-008**: Server MUST be the sole authority on damage amounts applied
- **FR-009**: Server MUST be the sole authority on player death determination
- **FR-010**: Server MUST reject all attack inputs during non-playing match phases

#### Health Points and Death
- **FR-011**: Server MUST track HP for each connected player
- **FR-012**: Server MUST initialize player HP to a fixed starting value on join and respawn
- **FR-013**: Server MUST broadcast death events when a player's HP reaches 0
- **FR-013b**: Client MUST immediately remove dead players from rendering upon death event
- **FR-014**: Server MUST trigger player respawn after a fixed delay following death
- **FR-015**: Server MUST respawn players at existing spawn points with full HP

#### Visual Feedback (Client)
- **FR-016**: Client MUST display "damage taken" feedback when the local player is hit (e.g., color flash, screen effect)
- **FR-016b**: Client MUST display "hit confirmed" feedback when the local player's attack lands on an enemy
- **FR-017**: Client MUST display kill notifications when any player dies (e.g., "Player X killed Player Y")
- **FR-018**: Client MUST NOT display hit feedback unless confirmed by server event

#### Event Synchronization
- **FR-019**: Server MUST broadcast combat events (hit, death, respawn) to all connected clients
- **FR-020**: Client MUST consume server events to update visual state and HUD
- **FR-021**: Client HP display (if shown) MUST reflect server-authoritative state only

#### Debug HUD Extension
- **FR-022**: Debug HUD MUST display local player's current HP
- **FR-023**: Debug HUD SHOULD display the most recent combat event (hit or kill)

#### Compatibility
- **FR-024**: Headless server mode MUST continue to function with combat system
- **FR-025**: Existing load tests MUST continue to pass
- **FR-026**: Existing unit and integration tests MUST continue to pass

### Key Entities

- **Player**: Game participant with HP, alive/dead state, position, and attack cooldown timer
- **Attack**: Discrete action from a player, containing attacker ID and direction/target info
- **Hit Event**: Server-validated outcome indicating which player was hit and damage dealt
- **Death Event**: Server notification that a player has died, including killer and victim IDs
- **Respawn Event**: Server notification that a dead player has respawned with full HP

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Two windowed clients can attack each other and both observe hit feedback within 200ms of server confirmation
- **SC-002**: 100% of hit validations are performed by the server; clients with modified inputs cannot cause false hits
- **SC-003**: When a player's HP reaches 0, death is detected and broadcast within one server tick
- **SC-004**: Dead players respawn within 5 seconds at a valid spawn point with full HP
- **SC-005**: All combat events (hit, death, respawn) are visible to clients within 200ms of occurrence
- **SC-006**: Local player HP is accurately displayed in HUD, updating within 100ms of taking damage
- **SC-007**: `cargo test --workspace` passes with all existing and new tests
- **SC-008**: Load tests with 50+ concurrent connections continue to function without degradation
- **SC-009**: Combat system performs correctly with simulated network latency up to 200ms (no desync)

## Clarifications

### Session 2025-12-14

- Q: How are attack targets selected? → A: Closest enemy in facing direction (cone/angle check)
- Q: What hit feedback do attacker vs victim see? → A: Attacker sees "hit confirmed" feedback; victim sees "damage taken" feedback (distinct)
- Q: How do dead players appear during respawn delay? → A: Immediately disappear; reappear at spawn point on respawn

## Assumptions

- Attack targeting uses facing direction with cone/angle check to find closest enemy in range
- Hit feedback is role-specific: attacker sees "hit confirmed", victim sees "damage taken"
- Dead players immediately disappear from rendering; no corpse/ragdoll visuals
- Attack range, cooldown duration, damage amount, and respawn delay are fixed constants (no balancing required)
- Simple melee attack only (no projectiles, no weapon switching)
- Spawn points already exist in the arena from the previous feature
- The existing input system can be extended to send attack actions
- Visual hit feedback can be a simple color change or HUD message (no particle effects required)
- HP is an integer value (e.g., 100 HP starting, 25 damage per hit)

## Out of Scope

- CEF / Web UI
- Complex weapons or weapon variety
- Inventory system
- Loot drops
- Projectile-based attacks
- Data persistence between sessions
- Matchmaking
- Fine-tuned game balance
