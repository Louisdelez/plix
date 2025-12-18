# Feature Specification: TDM Arena Mode

**Feature Branch**: `016-tdm-arena`
**Created**: 2025-12-16
**Status**: Draft
**Input**: Team Deathmatch game mode with Red vs Blue teams, score tracking, respawn system, and match flow management

## Clarifications

### Session 2025-12-16

- Q: What should a player see during respawn delay (dead state)? → A: Spectate own killer (follow camera on killer until respawn)
- Q: After match ends, should the server automatically reset to a new match? → A: Auto-reset after delay (~10-15 seconds showing scoreboard, then new match)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Team Scoring (Priority: P1) 🎯 MVP

When a player kills an enemy, their team earns a point. The current score is visible to all players.

**Why this priority**: Core TDM gameplay - without team scoring, there's no TDM mode. This is the fundamental mechanic.

**Independent Test**: Kill an enemy player → team score increments by 1 → score broadcast to all connected clients

**Acceptance Scenarios**:

1. **Given** a match in progress with Red at 5 points and Blue at 3 points, **When** a Red player kills a Blue player, **Then** Red score becomes 6, Blue stays at 3, and all clients receive score update
2. **Given** any match state, **When** a player kills a teammate (friendly fire), **Then** team score does NOT change (no self-scoring)
3. **Given** a match in progress, **When** a player disconnects, **Then** no score is awarded for the disconnect

---

### User Story 2 - Respawn System (Priority: P1) 🎯 MVP

When a player dies, they wait a configurable delay then respawn at their team's spawn point.

**Why this priority**: Without respawn, TDM becomes one-life elimination. Respawn is essential for TDM flow.

**Independent Test**: Player dies → waits respawn_delay seconds → spawns at team spawn point with full health

**Acceptance Scenarios**:

1. **Given** a player on Red team dies, **When** respawn_delay (default 3s) elapses, **Then** player respawns at a Red team spawn point with full health
2. **Given** a player dies with respawn_delay=5, **When** only 3 seconds have passed, **Then** player remains in dead state (spectating killer's view)
3. **Given** multiple Red spawn points exist, **When** player respawns, **Then** spawn point is selected (can be random or round-robin)

---

### User Story 3 - Match End on Score Limit (Priority: P1) 🎯 MVP

When a team reaches the score limit, the match ends and that team wins.

**Why this priority**: Matches need a win condition. Score limit is the defining TDM victory mechanic.

**Independent Test**: Team reaches score_limit → match state changes to Ended → winner announced

**Acceptance Scenarios**:

1. **Given** score_limit=25 and Red has 24 points, **When** Red scores 1 more kill, **Then** match ends with Red as winner
2. **Given** match ends, **When** any player attempts to attack, **Then** no damage is dealt and no score changes
3. **Given** match ends, **Then** all clients receive match_ended message with winning team

---

### User Story 4 - Team Assignment (Priority: P2)

Players are assigned to Red or Blue team when joining, with auto-balance for fairness.

**Why this priority**: Needed for multiplayer TDM but can be manually tested with forced team assignment initially.

**Independent Test**: Player joins → assigned to team with fewer players → can see teammates and enemies correctly

**Acceptance Scenarios**:

1. **Given** Red has 3 players and Blue has 2, **When** new player joins, **Then** player is assigned to Blue team
2. **Given** teams are equal (3v3), **When** new player joins, **Then** player is assigned to either team (implementation choice)
3. **Given** player is on Red team, **When** viewing other players, **Then** Red players show as friendly, Blue as enemy

---

### User Story 5 - Match State Transitions (Priority: P2)

Match progresses through states: Lobby → Playing → Ended, with appropriate behaviors in each.

**Why this priority**: State management enables proper match flow but basic scoring can work without full state machine.

**Independent Test**: Start server → lobby state → enough players join → playing state → score limit reached → ended state

**Acceptance Scenarios**:

1. **Given** server starts, **When** not enough players (< min_players), **Then** match is in Lobby state, kills don't count
2. **Given** Lobby state with min_players reached, **When** countdown completes (or immediate start), **Then** match transitions to Playing
3. **Given** Playing state, **When** score_limit reached, **Then** match transitions to Ended, scores frozen
4. **Given** Ended state, **When** reset delay (default 15s) elapses, **Then** match automatically resets to Lobby for new round

---

### User Story 6 - Match Configuration (Priority: P3)

Server operators can configure TDM parameters: score_limit, respawn_delay, team_size.

**Why this priority**: Defaults work for testing; configuration is polish for production use.

**Independent Test**: Start server with custom config → verify parameters are respected during gameplay

**Acceptance Scenarios**:

1. **Given** config with score_limit=50, **When** match plays, **Then** match ends at 50 kills not default
2. **Given** config with respawn_delay=10, **When** player dies, **Then** respawn takes 10 seconds
3. **Given** config with team_size=8, **When** teams are full (8v8), **Then** new players cannot join (or spectate)

---

### User Story 7 - Match Observability (Priority: P3)

Server exposes TDM metrics: current scores, match state, player counts per team, kills/deaths stats.

**Why this priority**: Useful for debugging and monitoring but not required for gameplay.

**Independent Test**: Query server metrics → see accurate team scores, match state, player counts

**Acceptance Scenarios**:

1. **Given** match in progress, **When** querying metrics, **Then** response includes team_scores, match_state, player_counts
2. **Given** debug logging enabled, **Then** kill events, respawns, and state transitions are logged

---

### Edge Cases

- **Simultaneous kills**: Both players kill each other in same tick → both teams get 1 point, both respawn
- **Kill during respawn delay**: Player dies, killer disconnects before respawn → death still counts, score still awarded
- **Player disconnect mid-match**: Score is preserved, player slot opens for new player
- **All players on one team disconnect**: Match continues (can become 0 vs N), no automatic forfeit
- **Score exactly at limit from multiple kills**: First kill that triggers limit wins, subsequent kills in same tick don't count
- **Player joins during Ended state**: Waits in lobby for match reset

## Requirements *(mandatory)*

### Functional Requirements

**Team Management**
- **FR-001**: System MUST support exactly two teams: Red and Blue
- **FR-002**: System MUST assign players to teams on join (auto-balance to smaller team)
- **FR-003**: System MUST track each player's team assignment in server state
- **FR-004**: System MUST broadcast team rosters to all clients

**Scoring**
- **FR-005**: System MUST award 1 point to killer's team on enemy kill
- **FR-006**: System MUST NOT award points for friendly fire or self-kills
- **FR-007**: System MUST NOT award points for kills outside Playing state
- **FR-008**: System MUST broadcast score updates to all clients within same tick

**Match Flow**
- **FR-009**: System MUST support match states: Lobby, Playing, Ended
- **FR-010**: System MUST transition to Ended when any team reaches score_limit
- **FR-011**: System MUST identify winning team as the team that reached score_limit
- **FR-012**: System MUST prevent combat actions (damage, kills) in non-Playing states
- **FR-012b**: System MUST automatically reset match to Lobby after reset_delay expires in Ended state

**Respawn**
- **FR-013**: System MUST track player death state with respawn timer
- **FR-014**: System MUST respawn player after respawn_delay seconds
- **FR-015**: System MUST respawn player at their team's designated spawn point(s)
- **FR-016**: System MUST restore full health on respawn
- **FR-016b**: Client MUST display killer's viewpoint during respawn delay (spectate killer mode)

**Configuration**
- **FR-017**: System MUST support configurable score_limit (default: 25)
- **FR-018**: System MUST support configurable respawn_delay in seconds (default: 3.0)
- **FR-019**: System MUST support configurable team_size (default: 8)
- **FR-020**: System MUST support configurable min_players to start match (default: 2)
- **FR-020b**: System MUST support configurable reset_delay in seconds (default: 15.0)

**Server Authority**
- **FR-021**: Server MUST be authoritative for all scoring decisions
- **FR-022**: Server MUST be authoritative for team assignments
- **FR-023**: Server MUST be authoritative for match state transitions
- **FR-024**: Server MUST validate all kill events before awarding points

**Arena Integration**
- **FR-025**: System MUST read team spawn points from arena definition
- **FR-026**: Arena MUST define spawn_points with team affiliation (red_spawns, blue_spawns)

### Key Entities

- **TdmMatchConfig**: Configuration for TDM mode - score_limit, respawn_delay, team_size, min_players, reset_delay
- **TdmMatchState**: Current match state enum (Lobby, Playing, Ended) with associated data (scores, winner)
- **Team**: Enum with Red and Blue variants
- **TeamScore**: Mapping of Team → u32 score
- **PlayerTeamAssignment**: Associates player_id with their Team
- **PlayerDeathState**: Tracks dead players with death_time for respawn timing
- **KillEvent**: Source (killer_id, team) and target (victim_id, team) for scoring validation

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Kill correctly awards 1 point to killer's team (100% accuracy in tests)
- **SC-002**: Match ends exactly when score_limit is reached (not before, not after)
- **SC-003**: Player respawns within 100ms of respawn_delay expiring
- **SC-004**: Score updates reach all clients within 1 tick (16ms at 60Hz)
- **SC-005**: Team assignment maintains balance (difference never exceeds 1 player)
- **SC-006**: No score can be awarded outside Playing state
- **SC-007**: Match state transitions are atomic and consistent across all clients
- **SC-008**: All TDM logic runs server-side (client only displays received state)

## Assumptions

- Existing combat system provides KillEvent with killer_id and victim_id
- Existing player management tracks connected players by ID
- Arena files can be extended to include team spawn point definitions
- Client already has UI infrastructure for displaying score/state (or will use simple text overlay)

## Out of Scope

- Team switching/swapping during match
- Detailed end-of-match statistics (K/D ratios, MVP)
- Spectator mode
- Match recording/replay
- Tournament brackets or multiple rounds
- Time-based match ending (time limit)
- Class/loadout selection
