# Feature Specification: Training Mode

**Feature Branch**: `020-training-mode`
**Created**: 2025-12-17
**Status**: Draft
**Input**: User description: "Feature 020 – Training Mode: sandbox training mode allowing players to practice freely with permissive settings and basic bots serving as targets, while maintaining server-authoritative architecture and extensibility."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Practice Aiming on Target Bots (Priority: P1)

As a player, I want to practice aiming by shooting at bots in a dedicated training arena so that I can improve my accuracy before competitive matches.

**Why this priority**: This is the core value proposition of training mode - providing targets to shoot at for aim practice. Without bots, the training mode has minimal utility.

**Independent Test**: Can be fully tested by spawning into training arena with bots present, hitting bots with attacks, and observing hit registration and bot respawn behavior.

**Acceptance Scenarios**:

1. **Given** a player joins a training session with `bot_count = 5`, **When** the session starts, **Then** 5 bots spawn at designated positions in the arena
2. **Given** bots are present in the arena, **When** the player hits a bot with an attack, **Then** the hit is registered and damage is applied (unless bots are invincible)
3. **Given** a bot is eliminated (health reaches 0), **When** the configured respawn delay passes, **Then** the bot respawns at a spawn point
4. **Given** `invincibility_bots = true`, **When** the player hits a bot, **Then** the hit is registered for statistics but no damage is applied

---

### User Story 2 - Quick Warmup Session (Priority: P1)

As a competitive player, I want to quickly warm up in training mode without waiting for a match to start so that I can be ready for actual gameplay.

**Why this priority**: Fast entry and instant respawn are essential for the training mode to be practical - players need frictionless access to practice.

**Independent Test**: Can be tested by joining training mode, dying, and verifying instant/fast respawn without match-end conditions.

**Acceptance Scenarios**:

1. **Given** `game_mode = "training"` is configured, **When** a player joins, **Then** they spawn immediately in the training arena
2. **Given** a player dies in training mode, **When** `player_respawn_delay` has elapsed (default: 1 second), **Then** the player respawns at a spawn point
3. **Given** training mode is active, **When** time passes, **Then** no victory condition ends the session (infinite duration)

---

### User Story 3 - Reset Training Session (Priority: P2)

As a player, I want to reset my training session to restart fresh so that I can clear my stats and reposition all bots.

**Why this priority**: Session reset provides convenience for structured practice sessions but is not essential for basic training functionality.

**Independent Test**: Can be tested by playing a training session, issuing a reset command, and verifying player position, stats, and bots are all reset.

**Acceptance Scenarios**:

1. **Given** a player is in a training session with accumulated stats, **When** they request a session reset, **Then** the player is moved to a spawn point, all stats are cleared, and all bots are respawned
2. **Given** bots are in various positions after being hit/killed, **When** a reset is triggered, **Then** all bots return to their initial spawn positions

---

### User Story 4 - View Training Statistics (Priority: P2)

As a player, I want to see my training statistics (hits, kills, accuracy) so that I can track my improvement.

**Why this priority**: Stats provide feedback for practice but training mode can function without them initially.

**Independent Test**: Can be tested by hitting bots, killing bots, and verifying statistics are tracked and can be displayed.

**Acceptance Scenarios**:

1. **Given** a player hits bots during training, **When** they press the stats key, **Then** total hits, total kills, and session duration are printed to the debug console
2. **Given** a player has made 10 attack attempts and landed 7 hits, **When** they view accuracy, **Then** the system shows 70% accuracy
3. **Given** a session is reset, **When** stats are checked, **Then** all statistics show zero

---

### User Story 5 - Configure Bot Behavior (Priority: P3)

As a server administrator, I want to configure bot behavior (stationary, roaming, strafing) so that I can customize the training difficulty.

**Why this priority**: Configurable bot behavior enhances training variety but static "dummy" bots provide core functionality.

**Independent Test**: Can be tested by setting `bot_behavior` to different values and observing bot movement patterns.

**Acceptance Scenarios**:

1. **Given** `bot_behavior = "dummy"`, **When** bots are spawned, **Then** they remain stationary at their spawn positions
2. **Given** `bot_behavior = "roam"`, **When** bots are active, **Then** they move randomly within a bounded area around their spawn point
3. **Given** `bot_behavior = "strafe"`, **When** bots are active, **Then** they strafe left and right around a center point

---

### User Story 6 - Invincibility Options (Priority: P3)

As a player practicing movement or mechanics, I want to enable player invincibility so that I can focus on non-combat skills without dying.

**Why this priority**: Invincibility is a convenience feature for specific practice scenarios, not core training functionality.

**Independent Test**: Can be tested by enabling `invincibility_player = true`, taking damage, and verifying health remains full.

**Acceptance Scenarios**:

1. **Given** `invincibility_player = true`, **When** the player takes damage, **Then** their health remains unchanged
2. **Given** `invincibility_player = false` (default), **When** the player takes damage, **Then** normal damage is applied

---

### Edge Cases

- What happens when all spawn points are occupied by bots? Bots respawn at a random available spawn point or wait until one is free.
- How does the system handle a player disconnect mid-session? Session state is cleared; reconnecting starts a fresh session.
- What happens if `bot_count` exceeds available spawn points? System spawns as many bots as spawn points allow, logging a warning.
- What happens when the training arena has no defined spawn points? System uses default arena spawn points or fails gracefully with an error.
- How does the system handle a reset request while bots are mid-respawn? Pending respawns are cancelled and bots respawn immediately at initial positions.

## Requirements *(mandatory)*

### Functional Requirements

#### Training Mode Core
- **FR-001**: System MUST support a `game_mode = "training"` configuration option to enable training mode
- **FR-002**: Training mode MUST have no victory condition or score limit (session runs indefinitely)
- **FR-003**: System MUST load a training arena from existing arena definitions

#### Player Respawn
- **FR-004**: System MUST respawn players after death with configurable delay (`player_respawn_delay`, default: 1 second)
- **FR-005**: System MUST support `invincibility_player` configuration (default: false) to make player immune to damage

#### Bot System
- **FR-006**: System MUST spawn a configurable number of bots (`bot_count`, default: 5)
- **FR-007**: Bots MUST support three behavior modes: `dummy` (stationary), `roam` (random movement), `strafe` (side-to-side movement)
- **FR-008**: System MUST respawn bots after elimination with configurable delay (`bot_respawn_delay`, default: 3 seconds)
- **FR-009**: System MUST support `invincibility_bots` configuration (default: false) to make bots immune to damage while still registering hits
- **FR-010**: Bots MUST be distinguishable from players visually (different color/model indicator)

#### Hit Detection & Damage
- **FR-011**: System MUST use existing hit detection to register player attacks on bots
- **FR-012**: System MUST apply damage to bots when hit (unless `invincibility_bots = true`)
- **FR-013**: System MUST track hits even when target is invincible (for statistics)

#### Session Management
- **FR-014**: System MUST support a session reset triggered by a dedicated keyboard binding that repositions player to spawn, resets stats, and respawns all bots
- **FR-015**: System MUST clear session state when player disconnects

#### Statistics
- **FR-016**: System MUST track per-session statistics: hits, kills, attack attempts (if applicable), and session duration
- **FR-017**: System MUST calculate accuracy as hits divided by attack attempts (if attack system supports attempt tracking)
- **FR-018**: System MUST expose statistics to client via debug console output triggered by a dedicated keyboard binding

#### Configuration
- **FR-019**: System MUST provide default configuration values ready for immediate use without manual setup
- **FR-020**: Configuration options MUST include: `bot_count`, `bot_behavior`, `bot_respawn_delay`, `player_respawn_delay`, `invincibility_player`, `invincibility_bots`

#### Server Authority
- **FR-021**: Server MUST be authoritative for: bot spawn/despawn, bot positions, damage/hits, respawns, and statistics
- **FR-022**: Clients MUST only display state received from server without local simulation of critical game logic

#### Performance
- **FR-023**: Bot behavior updates MUST be bounded per tick (no expensive world scans)
- **FR-024**: System MUST maintain stable performance with up to 20 bots active simultaneously

#### Observability
- **FR-025**: System MUST log training events: bot spawn, bot death, session reset
- **FR-026**: System MUST expose metrics: active bot count, total bot respawns, total hits, total kills

### Key Entities

- **TrainingSession**: Represents an active training instance. Contains player reference, configuration, statistics, and active bot list.
- **TrainingBot**: Represents a bot entity. Contains position, health, behavior type, spawn point reference, and respawn timer.
- **TrainingConfig**: Configuration parameters for training mode. Contains bot_count, bot_behavior, respawn delays, invincibility flags.
- **TrainingStats**: Per-session statistics. Contains hits, kills, attack_attempts, session_start_time, derived accuracy.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can join and start practicing in training mode within 5 seconds of selecting the mode
- **SC-002**: Player respawn completes within the configured delay plus 100ms tolerance
- **SC-003**: Bot respawn completes within the configured delay plus 100ms tolerance
- **SC-004**: Hit registration on bots occurs with the same latency as player-vs-player combat (existing system)
- **SC-005**: Training mode supports at least 20 concurrent bots without frame rate degradation on server tick rate
- **SC-006**: Session reset completes within 500ms, returning player and all bots to spawn positions
- **SC-007**: Statistics accuracy calculation matches actual hit/attempt ratio within 0.1% precision
- **SC-008**: All three bot behaviors (dummy, roam, strafe) function correctly and distinctly
- **SC-009**: Invincibility options correctly prevent damage while still registering hits for stats

## Clarifications

### Session 2025-12-17

- Q: How does the player trigger a session reset? → A: Keyboard binding (dedicated key)
- Q: How are training statistics displayed to the player? → A: Debug console/log output (key press to print)

## Assumptions

- Training mode uses existing arena definitions (no new arena format required)
- Existing hit detection and damage systems are reusable for bot interactions
- Bots use a simplified entity representation (no full player state machine)
- Attack attempt tracking depends on existing combat system capabilities; if not available, accuracy calculation is omitted
- Bot behaviors are simple and do not require pathfinding or obstacle avoidance
- "Roam" behavior moves within a fixed radius of spawn point using random direction changes
- "Strafe" behavior oscillates left-right perpendicular to a fixed facing direction
- Training mode runs on a standard server (solo = local server, private = dedicated server)
