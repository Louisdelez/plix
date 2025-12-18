# Feature Specification: Server Browser v1

**Feature Branch**: `026-server-browser`
**Created**: 2025-12-17
**Status**: Draft
**Input**: User description: "Feature 026 - Server Browser v1: Add server browser with tags, search, favorites, and master server integration for multiplayer server discovery"

## Clarifications

### Session 2025-12-17

- Q: How is server_id generated and who is responsible? → A: Master auto-generates server_id from host:port hash
- Q: What is the client browser interface model for v1? → A: Console commands only (e.g., /servers, /connect, /favorite)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse and Connect to Server (Priority: P1)

As a player, I want to see a list of available servers and connect to one so that I can easily find and join multiplayer games without manually entering server addresses.

**Why this priority**: This is the core functionality - without server listing and connection, the feature has no value. It enables the fundamental multiplayer discovery experience.

**Independent Test**: Can be fully tested by launching the client, viewing the server list, selecting a server, and successfully connecting to it.

**Acceptance Scenarios**:

1. **Given** the client is running and the master server is reachable, **When** I request the server list, **Then** I see a list of active servers with their name, player count, and region displayed
2. **Given** I see a list of servers, **When** I select a server and choose to connect, **Then** the client initiates connection to that server using the host and port
3. **Given** I attempt to connect to a server, **When** the server is offline or unreachable, **Then** I see an error message indicating the connection failed with appropriate details

---

### User Story 2 - Server Registration and Heartbeat (Priority: P1)

As a server administrator, I want my server to automatically announce itself to the master server so that players can discover it in the server browser.

**Why this priority**: Without server registration, the server list would be empty. This is a prerequisite for the browse functionality to work.

**Independent Test**: Can be fully tested by starting a game server with master registration enabled and verifying it appears in the master server's list.

**Acceptance Scenarios**:

1. **Given** a game server is configured with master server URL, **When** the server starts, **Then** it sends a registration to the master server with its name, host, port, region, tags, and current player count
2. **Given** a registered server is running, **When** 15-30 seconds pass, **Then** the server sends a heartbeat update to the master server with current player count
3. **Given** a registered server stops or fails to send heartbeats, **When** the expiration threshold is reached (60 seconds), **Then** the master server removes the server from the active list

---

### User Story 3 - Search and Filter Servers (Priority: P2)

As a player, I want to search and filter the server list by name, tags, and region so that I can quickly find servers that match my preferences.

**Why this priority**: Important for usability when there are many servers, but the feature is still useful without it (manual browsing).

**Independent Test**: Can be fully tested by populating the server list with multiple servers and verifying that search and filter options correctly narrow down the results.

**Acceptance Scenarios**:

1. **Given** I see a list of servers, **When** I enter a search term, **Then** only servers whose name, tags, or region contain the search term are displayed
2. **Given** I see a list of servers, **When** I apply the "has players" filter, **Then** only servers with player_count > 0 are displayed
3. **Given** I see a list of servers with mixed protocol versions, **When** I apply the "compatible version" filter, **Then** only servers matching my client's protocol version are displayed

---

### User Story 4 - Manage Favorite Servers (Priority: P2)

As a player, I want to mark servers as favorites and quickly access them so that I can easily return to servers I enjoy.

**Why this priority**: Improves repeat player experience, but the core browse/connect functionality works without it.

**Independent Test**: Can be fully tested by marking a server as favorite, closing and reopening the client, and verifying the favorite persists.

**Acceptance Scenarios**:

1. **Given** I see a server in the list, **When** I mark it as a favorite, **Then** the server is added to my favorites list and persisted locally
2. **Given** I have favorite servers saved, **When** I view my favorites, **Then** I see all my favorited servers even if some are currently offline
3. **Given** a server is marked as favorite, **When** I remove it from favorites, **Then** it no longer appears in my favorites list

---

### User Story 5 - Sort Server List (Priority: P3)

As a player, I want to sort the server list by player count or recency so that I can find active or fresh servers more easily.

**Why this priority**: Nice-to-have for better UX, but manual browsing is still functional without sorting options.

**Independent Test**: Can be fully tested by viewing a server list and applying different sort options, verifying the order changes accordingly.

**Acceptance Scenarios**:

1. **Given** I see a list of servers, **When** I sort by player count descending, **Then** servers with more players appear first
2. **Given** I see a list of servers, **When** I sort by most recent (last seen), **Then** servers with the most recent heartbeat appear first

---

### User Story 6 - Server Ping Display (Priority: P3)

As a player, I want to see the latency/ping to each server so that I can choose servers with good connection quality.

**Why this priority**: Optional quality-of-life feature. If not implemented, "unknown" is displayed instead.

**Independent Test**: Can be fully tested by viewing the server list and verifying ping values are displayed (or "unknown" if not implemented).

**Acceptance Scenarios**:

1. **Given** I view the server list, **When** ping measurement is enabled, **Then** each server displays its measured latency in milliseconds
2. **Given** ping measurement is not available or times out, **When** I view a server entry, **Then** the ping shows "unknown" instead of a number

---

### Edge Cases

- What happens when the master server is unreachable? The client displays an error message and allows retry; cached favorites remain accessible for direct connection
- What happens when a server sends malformed data to the master? The master rejects the registration and logs the validation failure
- What happens when too many servers register from the same IP? The master applies rate limiting and rejects excess registrations
- What happens when a server entry contains malicious strings? The client sanitizes all strings before display to prevent injection or rendering issues
- What happens when the favorites file is corrupted? The client falls back to an empty favorites list and logs a warning

## Requirements *(mandatory)*

### Functional Requirements

**Master Server:**
- **FR-001**: Master server MUST expose an HTTP endpoint to list active servers (read-only for clients)
- **FR-002**: Master server MUST accept server registrations containing: name, host, port, region, tags, player_count, max_players, game_modes, and protocol_version; master generates server_id as hash of host:port and sets last_seen timestamp
- **FR-003**: Master server MUST automatically remove servers that have not sent a heartbeat within 60 seconds
- **FR-004**: Master server MUST return only servers with last_seen within the configured freshness threshold
- **FR-005**: Master server MUST apply rate limiting per IP address for registration requests (max 10 registrations per minute per IP)
- **FR-006**: Master server MUST validate registration fields for size limits (name: 64 chars, region: 32 chars, tags: max 10 tags of 32 chars each) and allowed character sets (alphanumeric, spaces, hyphens, underscores)

**Game Server:**
- **FR-007**: Game server MUST be configurable to register with a master server URL
- **FR-008**: Game server MUST send periodic heartbeats (every 20 seconds) with current state when master registration is enabled
- **FR-009**: Game server MUST handle master server connection failures gracefully without blocking game operations
- **FR-010**: Game server MUST log heartbeat success and failure events

**Client Server Browser (console commands):**
- **FR-011**: Client MUST provide a `/servers` command to fetch and display the server list from the master server
- **FR-012**: Client MUST display server entries with: index number, name, player count/max, region, tags, and ping (or "unknown")
- **FR-013**: Client MUST allow text search filtering via `/servers <search>` on server name, tags, and region (case-insensitive substring match)
- **FR-014**: Client MUST allow filtering by: has_players (player_count > 0), compatible_version (protocol match), and tag inclusion/exclusion via command flags
- **FR-015**: Client MUST allow sorting by player_count descending or by last_seen (most recent first) via command flags
- **FR-016**: Client MUST provide a `/connect <index>` command to connect to a server by its list index
- **FR-017**: Client MUST preserve the player's display name (from Feature 025) when connecting via the server browser
- **FR-018**: Client MUST handle connection errors with appropriate user feedback (offline, incompatible version, timeout)
- **FR-019**: Client MUST sanitize all strings received from the master server before display
- **FR-020**: Client MUST use strict network timeouts (5 second timeout for master server requests)

**Favorites (console commands):**
- **FR-021**: Client MUST provide `/favorite <index>` and `/unfavorite <index>` commands to mark/unmark servers
- **FR-022**: Client MUST persist favorites locally to `~/.config/plix/servers.toml`
- **FR-023**: Client MUST identify favorites by server_id or host:port combination for stability
- **FR-024**: Client MUST provide a `/favorites` command to display saved favorites even when servers are offline

**Observability:**
- **FR-025**: Master server MUST log all registration and list requests
- **FR-026**: Client MUST log server list refresh attempts and connection attempts
- **FR-027**: Master server SHOULD track metrics: total requests, heartbeats received, servers returned

### Key Entities

- **ServerEntry**: Represents a game server in the directory; contains server_id (unique stable identifier, auto-generated by master as hash of host:port), name (display name), host (IP/hostname), port (game port), region (geographic region string), tags (list of descriptive strings), player_count (current players), max_players (capacity), game_modes (supported modes), protocol_version (for compatibility), last_seen (timestamp of last heartbeat)
- **FavoriteServer**: Represents a user's favorited server; contains server_id or host:port identifier, optional cached name for display when offline, date added
- **RegistrationRequest**: Data sent by game server to master; contains all ServerEntry fields plus authentication token if future auth is added
- **HeartbeatRequest**: Periodic update from game server; contains server_id and current player_count

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can discover and connect to a server from the browser in under 30 seconds from opening the browser
- **SC-002**: Server registration propagates to the server list within 5 seconds of the first heartbeat
- **SC-003**: Stale servers (no heartbeat for 60+ seconds) are removed from the list within 10 seconds of expiration
- **SC-004**: Search and filter operations return results within 1 second on a list of 1000 servers
- **SC-005**: The server browser remains responsive while refreshing or pinging servers (no UI freeze)
- **SC-006**: Favorite servers persist correctly across client restarts
- **SC-007**: Rate limiting successfully blocks excessive registration attempts (>10/minute from same IP)
- **SC-008**: All user-facing strings from external sources are safely displayed without rendering issues

## Assumptions

- The master server runs as a separate lightweight service (can be same host as game server for development)
- HTTP is acceptable for v1 master API (HTTPS can be added in future iteration)
- Console commands are the primary interface for v1 (e.g., `/servers`, `/connect <id>`, `/favorite <id>`); no native UI or CEF required
- Server operators are trusted to provide accurate server information (no verification of player counts in v1)
- Ping measurement is optional for v1; displaying "unknown" is acceptable
- The protocol_version field enables basic compatibility filtering without complex version negotiation

## Out of Scope

- User account creation or authentication
- Cloud synchronization of favorites
- Advanced master server moderation (ban lists, trust scores)
- Web-based or CEF UI
- Automatic matchmaking
- Server verification or anti-spoofing measures
- Geographic load balancing or region-based routing
