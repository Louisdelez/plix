# Feature Specification: BR Lite Mode (Mini Battle Royale)

**Feature Branch**: `019-br-lite`
**Created**: 2025-12-16
**Status**: Draft
**Input**: User description: "Mode BR Lite – simplified battle royale with shrinking zone, permanent elimination, minimal loot, last player standing wins"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Last Player Standing Victory (Priority: P1)

As a player, I want to survive longer than all other players in a free-for-all match to win the game.

**Why this priority**: This is the core game loop of battle royale - the fundamental win condition that defines the entire mode.

**Independent Test**: Can be tested by having 2+ players join a match, with combat enabled. When all but one player dies, the survivor is declared winner and match ends.

**Acceptance Scenarios**:

1. **Given** a match with 3+ alive players, **When** all but one player are eliminated, **Then** the last surviving player is declared the winner
2. **Given** a player is eliminated, **When** they attempt to respawn, **Then** they remain eliminated with no respawn option
3. **Given** a match ends with a winner, **When** the winner is declared, **Then** the match transitions to PostMatch state displaying the winner
4. **Given** a player dies, **When** they are eliminated, **Then** they can optionally enter spectator mode to watch remaining players

---

### User Story 2 - Shrinking Safe Zone (Priority: P1)

As a player, I want the playable area to shrink over time, forcing me to move toward the center and engage other players.

**Why this priority**: The shrinking zone is the signature mechanic that creates tension, prevents camping, and ensures matches end within a reasonable time.

**Independent Test**: Can be tested by starting a match, observing the zone shrink through phases, and verifying players outside the zone take damage.

**Acceptance Scenarios**:

1. **Given** a match starts, **When** the initial phase begins, **Then** the full arena is the safe zone with no damage outside
2. **Given** a phase timer expires, **When** a shrinking phase begins, **Then** the safe zone progressively decreases in size
3. **Given** a player is outside the safe zone, **When** the damage tick occurs, **Then** the player takes periodic damage
4. **Given** multiple zone phases are configured, **When** each phase completes, **Then** the zone shrinks further and damage may increase
5. **Given** the server controls zone state, **When** clients receive zone updates, **Then** they display the correct zone boundaries

---

### User Story 3 - Zone Phase Progression (Priority: P2)

As a player, I want clear phases where the zone is stable and phases where it shrinks, so I can plan my movement and combat strategy.

**Why this priority**: Phase-based zone progression provides strategic depth and predictability, allowing players to make tactical decisions.

**Independent Test**: Can be tested by configuring multiple phases and observing transitions between stable and shrinking phases.

**Acceptance Scenarios**:

1. **Given** a match is in progress, **When** a stable phase is active, **Then** the zone remains at its current size
2. **Given** a stable phase ends, **When** the shrinking phase begins, **Then** the zone progressively shrinks to the next target size
3. **Given** zone phase parameters are configured, **When** the match runs, **Then** phases follow configured durations and damage values
4. **Given** the server advances a phase, **When** clients are notified, **Then** they see updated phase information

---

### User Story 4 - Minimal Loot Collection (Priority: P2)

As a player, I want to pick up simple items like weapons and temporary bonuses to improve my survival chances.

**Why this priority**: Loot adds variety and strategic resource competition without complex inventory management.

**Independent Test**: Can be tested by placing loot items in the arena and having a player walk over them to collect instantly.

**Acceptance Scenarios**:

1. **Given** a loot item is placed in the arena, **When** a player contacts the item, **Then** the item is collected instantly with no inventory UI
2. **Given** a player contacts a weapon pickup, **When** collected, **Then** the player receives the weapon (or replaces current weapon)
3. **Given** a player contacts a temporary bonus, **When** collected, **Then** the bonus effect is applied (e.g., health restore, speed boost)
4. **Given** a player attempts to collect loot, **When** the server validates the pickup, **Then** only valid pickups are confirmed

---

### User Story 5 - Match Lifecycle Management (Priority: P2)

As a server administrator, I want BR matches to follow a clear lifecycle with automatic reset, so matches run smoothly without manual intervention.

**Why this priority**: Proper lifecycle management ensures reliable match flow and allows back-to-back games.

**Independent Test**: Can be tested by running a match through all states from Lobby to PostMatch and observing automatic reset.

**Acceptance Scenarios**:

1. **Given** the server starts a BR Lite match, **When** players join, **Then** they enter a Lobby/Warmup state
2. **Given** the match transitions to InProgress, **When** zone phases activate, **Then** the zone mechanics begin
3. **Given** a winner is declared, **When** PostMatch begins, **Then** the winner is displayed for a configurable duration
4. **Given** PostMatch completes, **When** reset triggers, **Then** the match resets and can restart automatically

---

### User Story 6 - Server Observability (Priority: P3)

As a server administrator, I want visibility into match state including alive players, zone phase, and eliminations for monitoring and debugging.

**Why this priority**: Observability enables troubleshooting, balancing, and match oversight without being core gameplay.

**Independent Test**: Can be tested by running a match and verifying logs and metrics expose the required information.

**Acceptance Scenarios**:

1. **Given** a match is in progress, **When** querying server state, **Then** the count of alive players is available
2. **Given** zone phases progress, **When** a phase changes, **Then** a log event is recorded
3. **Given** a player is eliminated, **When** the elimination occurs, **Then** a log event is recorded (not per-tick)
4. **Given** a match ends, **When** the winner is declared, **Then** a log event records the match outcome

---

### Edge Cases

- What happens when all remaining players die simultaneously (e.g., zone damage)? → The last player to die wins, or if truly simultaneous, the server picks deterministically (e.g., lowest player ID)
- What happens when a player disconnects? → They are eliminated (counted as a death)
- What happens if the zone shrinks to zero size? → Final phase should always leave a minimal safe area; if all players are outside, damage continues until one remains
- What happens when there's only 1 player from the start? → Match waits in Lobby until minimum player count is reached
- What happens if loot spawns outside the safe zone? → Loot remains collectible but players risk zone damage to reach it
- What happens if a player is eliminated while carrying a temporary bonus? → Bonus effect ends immediately upon death

## Requirements *(mandatory)*

### Functional Requirements

#### Core BR Mechanics

- **FR-001**: System MUST support a `br_lite` game mode selectable via arena configuration (`game_mode = "br_lite"`)
- **FR-002**: System MUST treat all players as enemies (free-for-all, no teams)
- **FR-003**: System MUST permanently eliminate players upon death (no respawn during match)
- **FR-004**: System MUST declare the last surviving player as the winner
- **FR-005**: System MUST transition to PostMatch state when a winner is determined

#### Zone Management

- **FR-006**: System MUST define an initial safe zone covering the playable arena
- **FR-007**: System MUST support multiple zone phases with configurable durations
- **FR-008**: System MUST implement two phase types: stable (zone maintains size) and shrinking (zone progressively decreases)
- **FR-009**: System MUST apply periodic damage to players outside the safe zone
- **FR-010**: System MUST support configurable damage values per phase (damage can increase in later phases)
- **FR-011**: Server MUST be the sole authority for zone state, size, and phase progression
- **FR-012**: System MUST synchronize zone state to all connected clients

#### Zone Configuration

- **FR-013**: System MUST support zone configuration via arena TOML file including:
  - Number of phases
  - Duration of each phase (stable and shrink durations)
  - Zone damage per phase
  - Target zone size per phase
- **FR-014**: System MUST provide sensible default values for all zone parameters to enable playable matches without custom tuning

#### Loot System

- **FR-015**: System MUST support placing loot items in the arena (weapons, temporary bonuses)
- **FR-016**: System MUST implement instant pickup on player contact (no inventory management)
- **FR-017**: System MUST support at minimum: basic weapon pickup and health restore pickup
- **FR-018**: Server MUST validate all loot pickups before confirming to clients
- **FR-019**: System MUST remove collected loot from the world

#### Elimination and Spectating

- **FR-020**: System MUST mark eliminated players with a permanent eliminated state
- **FR-021**: System MUST allow eliminated players to enter spectator mode (basic implementation)
- **FR-022**: System MUST handle player disconnection as elimination

#### Match Lifecycle

- **FR-023**: System MUST implement match states: Lobby/Warmup, InProgress, PostMatch, Reset
- **FR-024**: Server MUST manage all state transitions
- **FR-025**: System MUST support configurable PostMatch duration before reset
- **FR-026**: System MUST support automatic match restart after reset

#### Observability

- **FR-027**: Server MUST expose: alive player count, current zone phase, zone size/state, total eliminations
- **FR-028**: Server MUST log phase change events
- **FR-029**: Server MUST log player elimination events
- **FR-030**: Server MUST log match end events with winner information
- **FR-031**: Server MUST NOT log per-tick events for zone or damage

### Key Entities

- **Match**: Represents a BR Lite game session with state (Lobby, InProgress, PostMatch, Reset), alive player list, eliminated player list, zone state, and winner
- **Zone**: The shrinking safe area defined by center position, current radius, target radius, current phase, and damage value
- **ZonePhase**: Configuration for a single phase including type (stable/shrink), duration, target radius, and damage per tick outside zone
- **LootItem**: A collectible item in the arena with type (weapon, health, speed boost), position, and collected state
- **PlayerBRState**: Player's BR-specific state including alive/eliminated status, spectating flag, and any active temporary bonuses

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: BR Lite matches with 8+ players complete within 10 minutes when using default zone configuration
- **SC-002**: Players outside the zone receive damage within 1 second of leaving the safe area
- **SC-003**: Zone phase transitions occur within 100ms of configured timing
- **SC-004**: Loot pickups are confirmed within 200ms of player contact
- **SC-005**: Winner declaration occurs within 500ms of last opponent elimination
- **SC-006**: Match state transitions complete without player-visible delays
- **SC-007**: Server handles 16 concurrent players in BR mode without performance degradation
- **SC-008**: All elimination and phase change events are logged accurately for post-match analysis
- **SC-009**: Eliminated players can enter spectator mode within 2 seconds of death
- **SC-010**: Zone visualization updates on clients match server state within 100ms

## Clarifications

### Session 2025-12-16

- Q: What is the default minimum player count to start a BR Lite match? → A: 4 players (small competitive match)
- Q: What is the default duration for temporary bonus effects? → A: 10 seconds (balanced tactical window)

## Assumptions

- Zone shape is circular (cylindrical in 3D voxel space) for simplicity; rectangular zones are out of scope
- Loot items are pre-placed in arena TOML configuration, not randomly spawned during match
- "Basic weapon pickup" replaces the current weapon if the player has one (no weapon switching UI)
- Temporary bonuses (speed boost) last 10 seconds; health restore is instant
- Minimum player count to start a match is 4 (configurable, minimum 2)
- Spectator mode is view-only with free camera, no complex spectator UI features
- Zone damage is applied once per second (configurable tick rate is out of scope)
- Zone shrinks linearly toward the target radius during shrinking phases

## Out of Scope

- Complex inventory management or crafting systems
- Airdrops, vehicles, or large open-world maps
- Team-based BR modes (duos, squads)
- Advanced BR UI features (minimap with zone preview, drop trajectories)
- Random loot spawning or loot tables with rarity
- Knockdown/revive mechanics
- Pre-game lobby with player-ready system (uses existing warmup)
