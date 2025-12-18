# Feature Specification: Matchmaking v1 (Quick Join)

**Feature Branch**: `027-matchmaking-v1`
**Created**: 2025-12-17
**Status**: Draft
**Input**: User description: "Feature 027 – Matchmaking v1: Quick Join system for automatic server selection based on game mode and region, using existing master server infrastructure"

## Clarifications

### Session 2025-12-17

- Q: When a quick join connection fails, should the client automatically retry with another server? → A: Auto-retry up to 3 times, then show error
- Q: Where should quick join preferences be stored? → A: Add to existing `~/.config/plix/profile.toml` alongside identity (Feature 025)
- Q: When multiple servers have identical scores, how should the tie be broken? → A: Random selection among tied servers

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Quick Join with Mode and Region (Priority: P1)

As a player, I want to quickly join a game by specifying my preferred game mode and region so that I can start playing immediately without browsing through a list of servers.

**Why this priority**: This is the core functionality of matchmaking v1. Without the ability to request quick join with preferences, the feature provides no value over the manual server browser.

**Independent Test**: Can be fully tested by having multiple servers registered with different modes and regions, issuing a quick join command with specific preferences, and verifying the client connects to an appropriate server.

**Acceptance Scenarios**:

1. **Given** multiple servers are available with different game modes, **When** I request quick join for "TDM" mode, **Then** the system selects a server that supports TDM mode and initiates connection
2. **Given** multiple servers are available in different regions, **When** I request quick join for "eu" region with "FFA" mode, **Then** the system prefers servers in EU region that support FFA
3. **Given** I issue a quick join request, **When** the request succeeds, **Then** my player identity (display name from Feature 025) is preserved when connecting
4. **Given** I issue a quick join request, **When** a suitable server is found, **Then** the connection happens automatically without additional user input

---

### User Story 2 - Intelligent Server Selection (Priority: P1)

As a player, I want the system to choose the best available server based on multiple criteria so that I have a good gaming experience without manually evaluating each server.

**Why this priority**: The scoring algorithm is essential to differentiate matchmaking from random server selection. Without intelligent selection, quick join provides a poor user experience.

**Independent Test**: Can be fully tested by registering multiple servers with varying characteristics (player counts, regions, versions) and verifying the selection algorithm consistently picks appropriate servers.

**Acceptance Scenarios**:

1. **Given** servers with different player counts exist, **When** I quick join, **Then** the system prefers partially-filled servers (has players but not full) over empty or full servers
2. **Given** servers in my preferred region and other regions exist, **When** I quick join with a region preference, **Then** servers in my region receive higher priority than servers in other regions
3. **Given** a server with incompatible protocol version exists, **When** I quick join, **Then** that server is never selected regardless of other criteria
4. **Given** a full server and an available server both match my criteria, **When** I quick join, **Then** the available server is selected

---

### User Story 3 - Fallback When No Exact Match (Priority: P2)

As a player, I want the system to find a suitable server even if no exact match exists so that I can still play rather than getting a "no servers" error.

**Why this priority**: Fallback logic improves the user experience significantly but the core quick join works without it for exact matches.

**Independent Test**: Can be fully tested by requesting a mode/region combination with no exact matches and verifying the system falls back to alternative servers appropriately.

**Acceptance Scenarios**:

1. **Given** no servers in my preferred region support my mode, **When** I quick join, **Then** the system expands to any region and finds a compatible server
2. **Given** no servers support my requested mode at all, **When** I quick join, **Then** the system expands to "any mode" and finds an available server
3. **Given** no servers are available at all, **When** I quick join, **Then** I see a clear message "No servers available" with suggestion to use the server browser
4. **Given** fallback was used, **When** I connect successfully, **Then** I see feedback indicating the actual server's mode/region differs from my preference

---

### User Story 4 - Preferences Persistence (Priority: P2)

As a player, I want the system to remember my preferred mode and region so that I can quick join with a single command without specifying preferences each time.

**Why this priority**: Convenience feature that improves repeat player experience but not essential for core functionality.

**Independent Test**: Can be fully tested by setting preferences, restarting the client, and verifying the preferences are loaded correctly.

**Acceptance Scenarios**:

1. **Given** I have previously used quick join with "TDM" mode, **When** I next quick join without specifying a mode, **Then** the system uses "TDM" as my default mode
2. **Given** I have saved region preference "eu", **When** I quick join, **Then** the system uses "eu" as my default region preference
3. **Given** I change my preferred mode via command, **When** I close and reopen the client, **Then** my updated preference is persisted and loaded

---

### User Story 5 - Connection Error Handling (Priority: P2)

As a player, I want clear feedback when quick join fails so that I understand what happened and can take appropriate action.

**Why this priority**: Error handling is important for user experience but the happy path works without it.

**Independent Test**: Can be fully tested by simulating various failure conditions (timeout, server full, version mismatch) and verifying appropriate error messages.

**Acceptance Scenarios**:

1. **Given** the selected server becomes full between selection and connection, **When** connection fails, **Then** the system automatically retries with another server (up to 3 attempts) before showing "Server is full" error
2. **Given** connection to the selected server times out, **When** 5 seconds pass without response, **Then** I see "Connection timed out" message
3. **Given** the selected server has a version mismatch (race condition), **When** connection is rejected, **Then** I see "Incompatible version" message

---

### User Story 6 - Quick Play Menu Option (Priority: P3)

As a player, I want a "Quick Play" option in the main menu so that I can start playing with minimal interaction.

**Why this priority**: UI convenience feature. Console commands provide full functionality; menu option is a UX enhancement.

**Independent Test**: Can be fully tested by selecting Quick Play from the menu and verifying it triggers the quick join flow with default preferences.

**Acceptance Scenarios**:

1. **Given** I am in the main menu or pause menu, **When** I select "Quick Play", **Then** the quick join process starts with my saved preferences
2. **Given** Quick Play is initiated from menu, **When** a server is found, **Then** I am connected automatically without additional prompts

---

### Edge Cases

- What happens when the master server is unreachable? The quick join displays an error "Cannot reach server directory" and suggests using direct connect or retrying later
- What happens when all matching servers become full during selection? The system expands criteria or reports "All matching servers are full"
- What happens when server list is stale (cached from previous request)? Quick join fetches a fresh server list before selection to ensure accuracy
- What happens when the user spams quick join requests? Client-side debounce prevents excessive requests (minimum 2 seconds between requests)
- What happens when ping information is unavailable? Ping is ignored in scoring; server is still considered based on other criteria

## Requirements *(mandatory)*

### Functional Requirements

**Matchmaking Request:**
- **FR-001**: Client MUST provide a `/quickjoin <mode> <region>` command to initiate matchmaking with explicit preferences
- **FR-002**: Client MUST provide a `/play <mode>` alias that uses saved region preference (default: any)
- **FR-003**: Client MUST support mode values: tdm, ffa, ctf, br, training, any (case-insensitive)
- **FR-004**: Client MUST support region values: eu, us, asia, any (case-insensitive)
- **FR-005**: Client MUST fetch fresh server list from master server (Feature 026) before each quick join attempt

**Server Selection Algorithm:**
- **FR-006**: Client MUST exclude servers with incompatible protocol version (mandatory filter)
- **FR-007**: Client MUST exclude servers where requested mode is not in server's game_modes list (unless mode=any)
- **FR-008**: Client MUST exclude servers that are full (player_count >= max_players)
- **FR-009**: Client MUST assign score bonuses for: matching region (+50 points), partially filled server (+30 points for 1-80% capacity), recent heartbeat within 30 seconds (+20 points)
- **FR-010**: Client MUST prefer servers with more players over empty servers (up to 80% capacity) with scoring: +1 point per player up to 80% capacity
- **FR-011**: Client MUST include optional ping bonus if ping data is available: +40 points for ping < 50ms, +20 points for ping < 100ms
- **FR-012**: Client MUST select the server with highest total score
- **FR-013**: Client MUST use random selection when multiple servers have identical highest scores (tie-breaking)

**Fallback Behavior:**
- **FR-014**: Client MUST attempt selection with exact criteria first (mode + region)
- **FR-015**: If no server found, Client MUST retry with region=any (keep mode)
- **FR-016**: If still no server found, Client MUST retry with mode=any and region=any
- **FR-017**: If no server available after all fallbacks, Client MUST display "No servers available" message

**Connection:**
- **FR-018**: Client MUST automatically connect to selected server using host:port from ServerEntry
- **FR-019**: Client MUST preserve player identity (display_name from Feature 025) when connecting
- **FR-020**: Client MUST use 5-second connection timeout
- **FR-021**: Client MUST automatically retry with a different server on connection failure (up to 3 total attempts)
- **FR-022**: Client MUST exclude previously failed servers from retry selection within the same quick join session
- **FR-023**: Client MUST display appropriate error message after all retry attempts exhausted (timeout, full, version mismatch)

**Preferences:**
- **FR-024**: Client MUST persist quick join preferences to `~/.config/plix/profile.toml` (same file as Feature 025 identity)
- **FR-025**: Client MUST store: last_mode (default: tdm), preferred_region (default: any) under `[matchmaking]` section
- **FR-026**: Client MUST provide `/quickjoin-prefs` command to view current preferences
- **FR-027**: Client MUST provide `/quickjoin-prefs mode <value>` and `/quickjoin-prefs region <value>` to update preferences

**UI Integration (optional):**
- **FR-028**: Client SHOULD provide "Quick Play" menu item in pause menu after "Servers" option
- **FR-029**: Menu item MUST trigger quick join with saved preferences when selected

**Observability:**
- **FR-030**: Client MUST log quick join request with parameters (mode, region)
- **FR-031**: Client MUST log number of candidate servers after filtering
- **FR-032**: Client MUST log selected server details (name, host:port, score)
- **FR-033**: Client MUST log failure reason if no server selected or connection fails

### Key Entities

- **QuickJoinRequest**: Represents a matchmaking request; contains requested_mode (string or "any"), requested_region (string or "any"), allows fallback expansion
- **ServerScore**: Represents scoring result for a server; contains server reference, total_score, breakdown (region_bonus, capacity_bonus, freshness_bonus, ping_bonus, player_bonus)
- **QuickJoinPreferences**: User's saved preferences for quick join; contains last_mode, preferred_region, persisted to local config
- **QuickJoinResult**: Outcome of quick join attempt; contains selected_server (or null), fallback_used (boolean), fallback_reason (string), error_message (if failed)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can find and connect to a suitable server in under 10 seconds from issuing quick join command
- **SC-002**: Quick join selects appropriate servers 95% of the time when multiple valid options exist (matching mode and region)
- **SC-003**: Fallback mechanism finds an alternative server in 90% of cases where exact match is unavailable
- **SC-004**: Server selection scoring runs in under 100ms for a list of 1000 servers
- **SC-005**: Player preferences persist correctly across client restarts 100% of the time
- **SC-006**: Connection errors display clear, actionable feedback within 1 second of failure detection
- **SC-007**: Quick join requests do not cause UI freeze or lag (non-blocking operation)
- **SC-008**: Players using quick join join games 50% faster than players using manual server browser

## Assumptions

- The master server (Feature 026) is operational and returns accurate server data
- Servers accurately report their game_modes in heartbeat registration
- Console commands are the primary interface for v1; menu integration is optional enhancement
- Ping measurement from server browser (Feature 026) may not be available; scoring handles missing ping gracefully
- Client-side matchmaking is acceptable for v1; server-side matchmaking can be added in future iteration
- Region strings are simple identifiers (eu, us, asia) without complex geographic mapping

## Out of Scope

- Skill-based matchmaking (ELO, MMR, ranking)
- Queue/lobby system with waiting for other players
- Automatic server creation/provisioning
- Party/group matchmaking
- Private/password-protected server matching
- Cross-region routing optimization
- Server-side matchmaking orchestration
- Match history or statistics tracking
