# Feature Specification: Match Flow

**Feature Branch**: `006-match-flow`
**Created**: 2025-12-15
**Status**: Draft
**Input**: User description: "Implements the full competitive match lifecycle: Lobby → Ready Check → Countdown → Playing → End Screen → Restart / Arena Rotation"

## Clarifications

### Session 2025-12-15

- Q: Respawn behavior during Playing phase? → A: Unlimited respawn (match ends by score or time only)
- Q: Winner determination when scores are tied? → A: Tie declared (no unique winner if scores equal)

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ready Up and Start Match (Priority: P1)

Players join a game server and signal their readiness. When all connected players are ready and the minimum player count is met, the match begins after a countdown.

**Why this priority**: This is the core flow that enables competitive matches to start in a controlled, fair manner. Without this, there's no structured way to begin gameplay.

**Independent Test**: Two players connect, both press Ready, observe 3-second countdown, then match begins with players at spawn points.

**Acceptance Scenarios**:

1. **Given** a server in Lobby phase with 2 players connected (min_players=2), **When** both players toggle Ready, **Then** the server transitions to Countdown phase
2. **Given** a server in Countdown phase, **When** 3 seconds elapse, **Then** the server transitions to Playing phase and all players are spawned at arena spawn points
3. **Given** a server in Countdown phase, **When** a player disconnects or un-readies, **Then** the countdown cancels and server returns to ReadyCheck phase
4. **Given** a server in Lobby phase, **When** a player toggles Ready but minimum players not reached, **Then** server remains in Lobby phase

---

### User Story 2 - Play Match with Scoring (Priority: P1)

During the Playing phase, players engage in combat. Kills are tracked per player and contribute to score. The match ends when a score limit is reached, time expires, or only one team/player remains.

**Why this priority**: Scoring and match end conditions are essential for competitive gameplay. Without them, matches have no goal or conclusion.

**Independent Test**: Start a match, one player eliminates another, verify kill count increases and score updates. Continue until score limit triggers match end.

**Acceptance Scenarios**:

1. **Given** a match in Playing phase, **When** Player A eliminates Player B, **Then** Player A's kill count increments by 1 and score updates
2. **Given** a match in Playing phase with score_limit=5, **When** a player reaches 5 kills, **Then** the match transitions to EndScreen phase
3. **Given** a match in Playing phase with time_limit=300 seconds, **When** 300 seconds elapse, **Then** the match transitions to EndScreen phase
4. **Given** a match in Playing phase with unlimited respawn, **When** a player dies, **Then** they respawn at an arena spawn point after a brief delay

---

### User Story 3 - View End Screen and Restart (Priority: P1)

After a match ends, players see final scores for a configurable duration. Then the server resets and returns to Lobby phase, allowing a new match to begin without server restart.

**Why this priority**: Match reset is critical for continuous play sessions. Without it, players must reconnect after every match.

**Independent Test**: Complete a match, verify end screen displays for 5 seconds with final scores, then server resets to Lobby phase with all players still connected.

**Acceptance Scenarios**:

1. **Given** a match transitioning to EndScreen, **When** EndScreen begins, **Then** all player inputs (except UI) are disabled and final scores are broadcast
2. **Given** a server in EndScreen phase, **When** 5 seconds elapse, **Then** server transitions to Resetting phase
3. **Given** a server in Resetting phase, **When** reset completes, **Then** server transitions to Lobby phase with world reset and all players' scores cleared
4. **Given** players connected during EndScreen, **When** server returns to Lobby, **Then** all players remain connected with ready state cleared

---

### User Story 4 - Arena Rotation (Priority: P2)

After a match ends, the server can optionally rotate to a different arena from a configured list instead of replaying the same arena.

**Why this priority**: Arena variety enhances replayability but is not essential for core match flow.

**Independent Test**: Configure server with arena rotation list of 2 arenas. Complete match on arena 1, verify next match loads on arena 2.

**Acceptance Scenarios**:

1. **Given** a server with arena_rotation=[arena1, arena2] and current arena is arena1, **When** match resets, **Then** next match loads arena2
2. **Given** a server at the end of arena_rotation list, **When** match resets, **Then** rotation wraps to first arena
3. **Given** a server with arena_rotation disabled (empty list), **When** match resets, **Then** same arena is reloaded

---

### User Story 5 - Lobby Phase Restrictions (Priority: P2)

In Lobby phase, players can move freely but cannot deal damage, place/remove blocks, or accumulate score. This provides a safe warm-up period.

**Why this priority**: Prevents unfair advantages before match start, but gameplay still functions without this.

**Independent Test**: In Lobby phase, attempt to attack another player. Verify no damage is dealt.

**Acceptance Scenarios**:

1. **Given** a server in Lobby phase, **When** Player A attacks Player B, **Then** no damage is dealt
2. **Given** a server in Lobby phase, **When** a player attempts to place a block, **Then** the action is rejected
3. **Given** a server in Lobby phase, **When** players move around, **Then** movement is processed normally

---

### User Story 6 - Late Joiner Handling (Priority: P3)

Players who join during an active match are placed in a waiting state until the next match begins.

**Why this priority**: Important for continuous server operation but edge case for core match flow.

**Independent Test**: Start a match, have a new player connect mid-game, verify they cannot participate until next match.

**Acceptance Scenarios**:

1. **Given** a server in Playing phase, **When** a new player connects, **Then** they are placed in Lobby-only state (cannot spawn into active match)
2. **Given** a late joiner waiting, **When** match resets to Lobby, **Then** the late joiner is included in the next match
3. **Given** a late joiner, **When** they view the game, **Then** they see current match state and scores

---

### Edge Cases

- **Player disconnects during Countdown**: Server reverts to ReadyCheck phase if remaining players drop below minimum
- **Player disconnects during Playing**: Score preserved, match continues if still valid (enough players/teams remain)
- **All players disconnect during Playing**: Match ends immediately, server resets to Lobby
- **Server receives ReadyToggle during invalid phase**: Request is ignored
- **Player joins during EndScreen**: Treated as late joiner, waits for Lobby phase

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Server MUST start in Lobby phase upon initialization
- **FR-002**: Server MUST allow players to join and leave freely during Lobby phase
- **FR-003**: Server MUST prevent damage dealing during Lobby phase
- **FR-004**: Server MUST prevent block editing during Lobby phase
- **FR-005**: Server MUST allow player movement during Lobby phase
- **FR-006**: Server MUST track ready state per connected player
- **FR-007**: Server MUST allow players to toggle ready/unready via ReadyToggle message
- **FR-008**: Server MUST transition to Countdown when all players are ready AND minimum player count is met
- **FR-009**: Server MUST broadcast countdown ticks to all clients during Countdown phase
- **FR-010**: Server MUST cancel countdown and revert to ReadyCheck if a player disconnects or un-readies during Countdown
- **FR-011**: Server MUST transition to Playing phase when countdown reaches zero
- **FR-012**: Server MUST respawn all players at arena spawn points when Playing phase begins
- **FR-013**: Server MUST reset player health, inventory, and match stats when Playing phase begins
- **FR-014**: Server MUST enable combat and block editing during Playing phase
- **FR-015**: Server MUST track kills, deaths, and score per player during Playing phase
- **FR-015b**: Server MUST respawn dead players at arena spawn points during Playing phase (unlimited respawn)
- **FR-016**: Server MUST detect match end conditions: score limit reached OR time limit reached (unlimited respawn enabled by default)
- **FR-017**: Server MUST transition to EndScreen phase when any match end condition is met
- **FR-018**: Server MUST broadcast final match results to all clients when entering EndScreen (ties declared if scores equal, no tiebreaker)
- **FR-019**: Server MUST disable all gameplay inputs during EndScreen phase
- **FR-020**: Server MUST transition to Resetting phase after configurable end screen duration
- **FR-021**: Server MUST reset world state (blocks, entities) during Resetting phase
- **FR-022**: Server MUST clear all player ready states and match stats during Resetting phase
- **FR-023**: Server MUST transition to Lobby phase after reset completes
- **FR-024**: Server MUST support arena rotation with configurable arena list
- **FR-025**: Server MUST load next arena in rotation during Resetting phase (if configured)
- **FR-026**: Server MUST handle late joiners by placing them in lobby-only state during Playing/EndScreen phases
- **FR-027**: All phase transitions MUST be server-authoritative (clients cannot force transitions)
- **FR-028**: Server MUST broadcast MatchPhaseChanged event when phase transitions occur

### Key Entities

- **MatchPhase**: Current phase of the match lifecycle (Lobby, ReadyCheck, Countdown, Playing, EndScreen, Resetting)
- **PlayerMatchState**: Per-player match data including ready status, score (= kills), kills, deaths; highest score wins, ties possible
- **MatchConfig**: Server configuration for match rules (min_players, countdown_seconds, score_limit, time_limit_seconds, end_screen_seconds, arena_rotation)
- **MatchState**: Central match state containing current phase, elapsed time, countdown remaining, and player states

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Two players can complete a full match cycle (Lobby → Ready → Countdown → Playing → EndScreen → Lobby) in under 10 minutes
- **SC-002**: Match countdown is accurate within 1 tick of configured duration (default 3 seconds at 60Hz)
- **SC-003**: Score updates are reflected to all clients within 1 tick of the scoring event
- **SC-004**: Server can reset and begin a new match without restart within 2 seconds
- **SC-005**: 100% of phase transitions are server-authoritative (zero client-forced transitions possible)
- **SC-006**: Late joiners receive correct current match phase within 1 second of connecting
- **SC-007**: All existing tests (combat, movement, block interaction) continue to pass
- **SC-008**: Headless clients and load tests remain functional with new match flow
