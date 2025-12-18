# Feature Specification: FFA Arena Mode

**Feature Branch**: `017-ffa-arena`
**Created**: 2025-12-16
**Status**: Draft
**Input**: User description: "Implement FFA Arena Mode - Free-for-All game mode where each player scores points individually by eliminating others, with configurable score limit, respawn system, individual victory conditions, and server-authoritative gameplay."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Individual Kill Scoring (Priority: P1)

As a player in FFA mode, I want to earn points by eliminating other players so that I can progress toward winning the match.

**Why this priority**: Core gameplay loop - without scoring, there is no game mode. Every kill must award a point to the killer, and this is the foundation of FFA competition.

**Independent Test**: Kill another player → verify killer's score increases by 1 → verify score is broadcast to all clients

**Acceptance Scenarios**:

1. **Given** a match in Playing phase with 2+ players, **When** Player A eliminates Player B, **Then** Player A's score increases by 1 and is broadcast to all clients
2. **Given** a player with 0 kills, **When** they eliminate another player, **Then** their score becomes 1
3. **Given** a player at score 4 (score_limit=5), **When** they get another kill, **Then** their score becomes 5 and triggers match end

---

### User Story 2 - FFA Respawn System (Priority: P1)

As a player who was eliminated, I want to respawn after a short delay so that I can continue playing and competing.

**Why this priority**: Without respawn, eliminated players cannot continue - essential for the continuous gameplay loop required by FFA.

**Independent Test**: Player dies → waits respawn_delay → spawns at neutral spawn point with full health

**Acceptance Scenarios**:

1. **Given** a player in Playing phase, **When** they are eliminated, **Then** they enter dead state for respawn_delay duration
2. **Given** a dead player after respawn_delay expires, **When** respawn triggers, **Then** they appear at a valid neutral spawn with 100 health
3. **Given** multiple spawn points, **When** a player respawns, **Then** they spawn at a neutral FFA spawn (not team-specific)

---

### User Story 3 - Match End on Score Limit (Priority: P1)

As a player, I want the match to end when someone reaches the score limit so that there is a clear winner and the competition has closure.

**Why this priority**: Victory conditions define the game's goal - players need to know how to win and when the match concludes.

**Independent Test**: Player reaches score_limit → match transitions to EndScreen → winner declared

**Acceptance Scenarios**:

1. **Given** a match in Playing phase, **When** a player reaches score_limit kills, **Then** match immediately transitions to EndScreen phase
2. **Given** match end triggered, **When** EndScreen begins, **Then** the winning player's PlayerId is declared as winner
3. **Given** EndScreen phase, **When** end_screen_delay expires, **Then** match resets to Lobby for next round

---

### User Story 4 - Match State Transitions (Priority: P2)

As a server operator, I want matches to flow through proper phases (Lobby → Playing → EndScreen → Reset) so that gameplay is organized and predictable.

**Why this priority**: State machine ensures orderly match flow, but P1 features can work with simplified states initially.

**Independent Test**: Server starts → Lobby phase → countdown → Playing → score_limit → EndScreen → auto-reset to Lobby

**Acceptance Scenarios**:

1. **Given** server starts, **When** match initializes, **Then** phase is Lobby
2. **Given** minimum players ready in Lobby, **When** countdown completes, **Then** phase transitions to Playing
3. **Given** EndScreen phase, **When** end_screen_delay expires, **Then** match resets: scores cleared, phase returns to Lobby

---

### User Story 5 - FFA Configuration (Priority: P2)

As a server operator, I want to configure score_limit, respawn_delay, and end_screen_delay so that I can tune match intensity and duration.

**Why this priority**: Customization is important but reasonable defaults allow FFA to work without configuration.

**Independent Test**: Start server with custom config values → verify match uses those values

**Acceptance Scenarios**:

1. **Given** server config with score_limit=10, **When** match runs, **Then** match ends when a player reaches 10 kills
2. **Given** server config with respawn_delay=5 seconds, **When** player dies, **Then** they respawn after 5 seconds
3. **Given** no custom config, **When** server starts, **Then** reasonable defaults apply (score_limit=15, respawn_delay=3s, end_screen=10s)

---

### User Story 6 - FFA Observability (Priority: P3)

As a server operator, I want to see match state, top scores, and winner information so that I can monitor and debug matches.

**Why this priority**: Debug/monitoring is valuable but not essential for core gameplay.

**Independent Test**: Query server state → see match phase, leader scores, winner if applicable

**Acceptance Scenarios**:

1. **Given** a match in progress, **When** checking server state, **Then** current phase and top player scores are visible
2. **Given** match ended with winner, **When** checking server state, **Then** winner PlayerId is accessible
3. **Given** FFA gameplay, **When** events occur, **Then** relevant logs are emitted (kills, respawns, phase changes) without per-tick spam

---

### Edge Cases

- What happens when a player disconnects mid-match? Their score is retained until match reset; no score awarded for disconnects.
- What happens if two players reach score_limit on the same tick? First player to register the kill wins (deterministic by processing order).
- What happens if a player kills themselves (suicide/fall damage)? No score awarded to anyone; victim respawns normally.
- What happens if the last player disconnects during EndScreen? Match continues to auto-reset as normal.
- What happens if score_limit is set to 1? First kill wins the match - valid edge case that should work.

## Requirements *(mandatory)*

### Functional Requirements

**Core Scoring (US1)**
- **FR-001**: System MUST award 1 point to the killer when they eliminate another player
- **FR-002**: System MUST NOT award points for self-inflicted deaths or environmental kills
- **FR-003**: System MUST broadcast updated player scores to all connected clients via WorldSnapshot
- **FR-004**: System MUST track kills and deaths per player throughout the match

**Respawn System (US2)**
- **FR-005**: System MUST place eliminated players in a dead state immediately upon death
- **FR-006**: System MUST respawn dead players after respawn_delay_ticks have elapsed
- **FR-007**: System MUST spawn players at neutral spawn points (team-agnostic)
- **FR-008**: System MUST reset player health to 100 on respawn
- **FR-009**: System MUST clear player dead state and respawn_tick on respawn

**Match End Conditions (US3)**
- **FR-010**: System MUST end the match immediately when a player reaches score_limit
- **FR-011**: System MUST set winner to the PlayerId of the player who reached score_limit
- **FR-012**: System MUST transition to EndScreen phase on match end
- **FR-013**: System MUST broadcast MatchEnd event with winner and final scores
- **FR-014**: System MUST prevent scoring during non-Playing phases

**State Transitions (US4)**
- **FR-015**: System MUST initialize matches in Lobby phase
- **FR-016**: System MUST transition Lobby → Countdown → Playing when minimum players are ready
- **FR-017**: System MUST transition Playing → EndScreen when match ends
- **FR-018**: System MUST transition EndScreen → Resetting → Lobby after end_screen_delay
- **FR-019**: System MUST reset all player scores to 0 on match reset

**Configuration (US5)**
- **FR-020**: System MUST support configurable score_limit (default: 15)
- **FR-021**: System MUST support configurable respawn_delay_ticks (default: 180 ticks = 3 seconds at 60Hz)
- **FR-022**: System MUST support configurable end_screen_ticks (default: 600 ticks = 10 seconds at 60Hz)
- **FR-026**: Arena config MUST include a `game_mode` field ("ffa" or "tdm") that determines scoring rules

**Observability (US6)**
- **FR-023**: System MUST log kill events with killer and victim IDs
- **FR-024**: System MUST log phase transitions
- **FR-025**: System MUST NOT log per-tick during normal gameplay

### Key Entities

- **PlayerScore**: Tracks individual player's kills and deaths (existing from TDM implementation)
- **MatchState**: Current phase, time_remaining, score_limit, player_scores, winner (existing)
- **MatchConfig**: Configuration values for score_limit, respawn_delay, end_screen duration (existing, with FFA defaults)
- **SpawnPoint**: Neutral spawn positions in arena (existing, use team-agnostic selection)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can complete a full FFA match cycle (join → play → win/lose → new match) in under 10 minutes with default settings
- **SC-002**: Kill-to-score updates are visible to all players within 1 game tick (16ms at 60Hz)
- **SC-003**: Respawn occurs within 1 tick of respawn_delay expiring
- **SC-004**: Match end triggers within 1 tick of score_limit being reached
- **SC-005**: System supports 8+ concurrent players in FFA mode without degradation
- **SC-006**: No scoring occurs outside Playing phase (100% enforcement)
- **SC-007**: All match state transitions are server-authoritative (clients cannot force transitions)

## Assumptions

- FFA mode reuses existing infrastructure from TDM (match_state, respawn system, scoring)
- Arenas define neutral spawn points (team=None or team ignored for FFA)
- Existing individual player scoring (`check_score_limit`, `update_player_score`) is sufficient
- No spectate-killer mode for FFA (player respawns immediately to action)
- Time limit is optional - score_limit is primary victory condition
- Minimum players for match start defaults to 2 (can be configured)
- **Game mode is determined by arena config**: Arena TOML files include a `game_mode` field ("ffa" or "tdm") that determines scoring rules

## Out of Scope

- Matchmaking, ranking, or ELO systems
- Complex loadouts or economy
- Advanced scoreboard UI (full leaderboard, detailed panels)
- Map voting or advanced rotation
- Spectator mode for eliminated players
