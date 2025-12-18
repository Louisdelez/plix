# Feature Specification: CTF Mode (Capture The Flag)

**Feature Branch**: `018-ctf-mode`
**Created**: 2025-12-16
**Status**: Draft
**Input**: User description: "Mode CTF (Capture The Flag) - Implement objective-based game mode where two teams compete to capture the enemy flag and return it to their base, with server-authoritative objective state, scoring, flag states, and victory conditions."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Flag Capture (Priority: P1)

As a player, I want to pick up the enemy flag and return it to my base to score a point for my team.

**Why this priority**: This is the core gameplay loop of CTF - without flag capture, there is no CTF mode. This defines the primary objective that makes CTF distinct from other modes.

**Independent Test**: Player enters enemy flag zone, picks up flag, returns to own capture zone, flag is captured, team scores 1 point, flags reset to bases.

**Acceptance Scenarios**:

1. **Given** an enemy flag at its base and a player on the opposing team, **When** the player enters the flag zone, **Then** the player picks up the flag and becomes the flag carrier.

2. **Given** a player carrying the enemy flag, **When** the player enters their team's capture zone, **Then** their team scores 1 point and both flags reset to their respective bases.

3. **Given** a player carrying the enemy flag, **When** the player enters their capture zone but their own flag is not at base, **Then** the capture fails and the player continues carrying the flag.

---

### User Story 2 - Flag Drop on Death (Priority: P1)

As a player, when I die while carrying the enemy flag, the flag should be dropped at my position so teammates or enemies can interact with it.

**Why this priority**: Flag drop on death is essential to the CTF dynamic - it creates opportunities for flag recovery, defense, and strategic gameplay.

**Independent Test**: Flag carrier is killed, flag drops at carrier's position, flag enters "dropped" state with return timer.

**Acceptance Scenarios**:

1. **Given** a player carrying the enemy flag, **When** the player is killed, **Then** the flag is dropped at the player's last position.

2. **Given** a dropped flag on the ground, **When** an enemy player touches it, **Then** that player picks up the flag and becomes the new carrier.

3. **Given** a dropped flag on the ground, **When** a teammate (flag's team) touches it, **Then** the flag is returned to its base immediately.

4. **Given** a dropped flag on the ground, **When** the return timer expires, **Then** the flag automatically returns to its base.

---

### User Story 3 - Match Victory (Priority: P1)

As a player, I want the match to end when my team reaches the capture limit, declaring us the winners.

**Why this priority**: Victory conditions are essential to define match end state and declare winners - without this, matches would have no conclusion.

**Independent Test**: Team captures flag, score increments, when capture limit is reached match ends with team declared winner.

**Acceptance Scenarios**:

1. **Given** a team has scored (capture_limit - 1) captures, **When** they capture another flag, **Then** the match ends with that team declared winner.

2. **Given** a match has ended, **When** the end screen duration expires, **Then** the match resets to lobby state.

3. **Given** time limit is reached before capture limit, **When** one team has more captures, **Then** that team wins.

4. **Given** time limit is reached with equal captures, **When** the match ends, **Then** the result is a tie (no winner).

---

### User Story 4 - CTF Configuration (Priority: P2)

As a server administrator, I want to configure CTF match parameters (capture limit, flag return delay, respawn delay) via arena configuration.

**Why this priority**: Configuration allows server operators to customize match intensity and duration, but the mode works with sensible defaults.

**Independent Test**: Load arena with custom CTF config values, verify match uses those values instead of defaults.

**Acceptance Scenarios**:

1. **Given** an arena with `game_mode = "ctf"` and `capture_limit = 5`, **When** the server loads the arena, **Then** the match uses 5 captures as the victory condition.

2. **Given** an arena with no custom config values, **When** the server loads the arena, **Then** the match uses default CTF values (capture_limit=3, flag_return_delay=10s, respawn_delay=5s).

---

### User Story 5 - Flag State Visibility (Priority: P2)

As a player, I want to know the current state of both flags (at base, carried, dropped) so I can make strategic decisions.

**Why this priority**: Flag state visibility is important for gameplay but not strictly required for the core capture mechanic.

**Independent Test**: Flag state changes are broadcast to all clients, clients receive flag position and carrier information.

**Acceptance Scenarios**:

1. **Given** a flag at its base, **When** a client queries flag state, **Then** the state shows "AtBase" with base position.

2. **Given** a flag being carried, **When** a client queries flag state, **Then** the state shows "Carried" with carrier's PlayerId.

3. **Given** a flag dropped on the ground, **When** a client queries flag state, **Then** the state shows "Dropped" with world position and return timer remaining.

---

### User Story 6 - CTF Observability (Priority: P3)

As a server operator, I want to see logs and state for CTF events to monitor and debug matches.

**Why this priority**: Observability is valuable for server operators but not required for core gameplay.

**Independent Test**: CTF events generate structured logs, match state is queryable.

**Acceptance Scenarios**:

1. **Given** a flag pickup event, **When** logged, **Then** the log includes player ID, flag team, and timestamp.

2. **Given** a flag capture event, **When** logged, **Then** the log includes capturing player, team scores, and match state.

3. **Given** a running CTF match, **When** server state is queried, **Then** it includes flag states, team scores, and match phase.

---

### Edge Cases

- What happens when a flag carrier disconnects? Flag is dropped at last position, same as death.
- What happens if a flag is dropped out of bounds? Flag automatically returns to base immediately.
- What happens if both teams reach capture limit on the same tick? First capture processed wins (deterministic order).
- What happens if a player tries to pick up their own team's flag? No effect - players can only pick up enemy flags.
- What happens if the flag carrier enters a solid block (collision)? Physics system prevents this; if somehow occurs, flag returns to base.

## Requirements *(mandatory)*

### Functional Requirements

#### Game Mode Selection

- **FR-001**: System MUST support `game_mode = "ctf"` in arena configuration to enable CTF mode.
- **FR-002**: System MUST use CTF-specific defaults when arena specifies CTF mode.

#### Flag System

- **FR-010**: System MUST create exactly one flag per team at match start, positioned at team's flag base location.
- **FR-011**: System MUST track flag state as one of: AtBase, Carried(PlayerId), Dropped(Position, ReturnTick).
- **FR-012**: System MUST allow only enemy players to pick up a flag.
- **FR-013**: System MUST prevent a player from carrying more than one flag at a time.
- **FR-014**: System MUST drop the flag at carrier's position when carrier dies.
- **FR-015**: System MUST drop the flag at carrier's position when carrier disconnects.
- **FR-016**: System MUST return dropped flag to base after configurable return delay expires.
- **FR-017**: System MUST return dropped flag to base immediately when touched by a teammate.
- **FR-018**: System MUST return flag to base immediately if dropped out of valid arena bounds.

#### Capture System

- **FR-020**: System MUST award 1 point to capturing team when player enters capture zone with enemy flag.
- **FR-021**: System MUST require own flag to be at base for capture to succeed (classic rule).
- **FR-022**: System MUST reset both flags to their bases after a successful capture.
- **FR-023**: System MUST broadcast capture event to all clients after successful capture.

#### Scoring & Victory

- **FR-030**: System MUST track team scores as number of successful captures.
- **FR-031**: System MUST end match when a team reaches capture_limit.
- **FR-032**: System MUST end match when time_limit is reached.
- **FR-033**: System MUST declare team with higher capture score as winner.
- **FR-034**: System MUST declare tie if scores are equal at time limit.

#### Arena Zones

- **FR-040**: Arena MUST define flag_base zone per team (where flag spawns).
- **FR-041**: Arena MUST define capture_zone per team (where flag is captured).
- **FR-042**: Zones MUST be defined as axis-aligned bounding boxes (min/max positions).
- **FR-043**: System MUST detect player-zone collisions each tick.

#### Match State

- **FR-050**: System MUST support CTF-specific match phases: Lobby, Countdown, Playing, EndScreen, Resetting.
- **FR-051**: System MUST reset flag states on match reset.
- **FR-052**: System MUST reset team capture scores on match reset.
- **FR-053**: System MUST include flag states in match state broadcast.

#### Configuration

- **FR-060**: System MUST support configurable capture_limit (default: 3).
- **FR-061**: System MUST support configurable flag_return_delay in seconds (default: 10).
- **FR-062**: System MUST support configurable respawn_delay in seconds (default: 5).
- **FR-063**: System MUST support configurable time_limit in seconds (default: 600 = 10 minutes).
- **FR-064**: System MUST support configurable end_screen_delay in seconds (default: 10).

#### Respawn

- **FR-070**: System MUST use team-specific spawn points (same as TDM).
- **FR-071**: System MUST apply respawn_delay before player respawns.
- **FR-072**: System MUST clear flag carrier state when player dies.

### Key Entities

- **Flag**: Represents a team's flag. Has owner team, current state (AtBase/Carried/Dropped), position (when dropped), carrier ID (when carried), return tick (when dropped).

- **FlagZone**: Defines a spatial region in the arena. Has team owner, zone type (FlagBase or CaptureZone), bounding box (min/max Vec3).

- **CTFMatchState**: Extended match state for CTF. Has team capture scores, flag states for both teams, capture_limit configuration.

- **CTFConfig**: Configuration for CTF mode. Has capture_limit, flag_return_delay, respawn_delay, time_limit, end_screen_delay.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can complete a full capture (pickup to return to base) within the expected arena traversal time (arena-dependent, typically 15-30 seconds).

- **SC-002**: Flag state changes (pickup, drop, return, capture) are reflected to all players within 1 game tick (16ms at 60Hz).

- **SC-003**: Match correctly ends when capture_limit is reached in 100% of test scenarios.

- **SC-004**: Dropped flags return to base after exactly the configured flag_return_delay duration.

- **SC-005**: Flag carrier death results in flag drop at carrier position in 100% of cases.

- **SC-006**: Capture is blocked when own flag is not at base in 100% of test scenarios.

- **SC-007**: CTF mode works with default configuration requiring zero custom arena configuration beyond `game_mode = "ctf"` and zone definitions.

- **SC-008**: Server processes flag interactions without frame drops with 16 concurrent players.

- **SC-009**: All CTF events (pickup, drop, capture, return) are logged with player ID and timestamp.

- **SC-010**: Match state correctly transitions through all phases: Lobby to Countdown to Playing to EndScreen to Resetting to Lobby.

## Assumptions

- Arena designers will provide valid flag_base and capture_zone definitions in arena TOML files.
- Two teams only (Red/Blue or Team 0/Team 1) - no multi-team CTF.
- Flags have no physics (they appear at positions but don't fall or collide with blocks).
- Flag pickup is instantaneous on contact (no interaction delay).
- Classic rule (own flag must be at base to capture) is always enabled in this implementation.
- Client-side flag rendering/visualization is out of scope for this feature (server state only).
