# Feature Specification: Account Identity

**Feature Branch**: `025-account-identity`
**Created**: 2025-12-17
**Status**: Draft
**Input**: User description: "Feature 025 – Account Identity: Add simple, stable player identity with configurable display name, local client profile, and server-authoritative session identity, preparing for future optional authentication without implementing login in v1."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Player Display Name Setup (Priority: P1)

As a player, I want to choose my display name (pseudo) so that other players can identify me during matches.

**Why this priority**: Core identity feature - without display names, players cannot be distinguished from each other, making multiplayer confusing and impersonal.

**Independent Test**: Can be fully tested by connecting a client with a custom name and verifying the name appears in server logs and is replicated to other players.

**Acceptance Scenarios**:

1. **Given** a player launches the client for the first time, **When** they connect to a server, **Then** the system uses a default display name (e.g., "Player") if no profile exists.

2. **Given** a player has configured a display name in their local profile, **When** they connect to a server, **Then** the server receives and validates the name, applying it if valid.

3. **Given** a player provides an invalid name (too long, invalid characters, empty), **When** they connect to a server, **Then** the server sanitizes or replaces it with a fallback name and informs the player.

4. **Given** a player provides a valid display name, **When** they are in a match, **Then** other players see this name in scoreboard, overlays, and event logs.

---

### User Story 2 - Local Profile Persistence (Priority: P1)

As a player, I want my display name to be saved locally so that I don't have to re-enter it every time I start the game.

**Why this priority**: Essential for user experience - players expect their preferences to persist between sessions.

**Independent Test**: Can be tested by setting a display name, closing the client, reopening it, and verifying the saved name is loaded automatically.

**Acceptance Scenarios**:

1. **Given** a player sets a display name, **When** they exit the game, **Then** the name is saved to a local profile file.

2. **Given** a player starts the game with an existing profile, **When** the client initializes, **Then** it loads the saved display name from the profile.

3. **Given** no profile file exists, **When** the client starts for the first time, **Then** it creates a default profile with a placeholder name.

4. **Given** a corrupted or invalid profile file, **When** the client starts, **Then** it creates a fresh default profile and logs a warning.

---

### User Story 3 - Display Name Uniqueness on Server (Priority: P2)

As a server operator, I want to prevent duplicate display names in a match so that players are not confused about who is who.

**Why this priority**: Important for gameplay clarity but not strictly required for basic identity to function.

**Independent Test**: Can be tested by connecting two clients with the same display name and verifying the server disambiguates them.

**Acceptance Scenarios**:

1. **Given** player "Alex" is already connected, **When** another player connects with name "Alex", **Then** the server automatically assigns a unique variant (e.g., "Alex#2").

2. **Given** "Alex#2" disconnects, **When** a new player connects as "Alex", **Then** the server assigns "Alex#2" (or lowest available suffix).

3. **Given** a player with a disambiguated name, **When** they view their name in-game, **Then** they see their actual assigned name (with suffix if applicable).

---

### User Story 4 - In-Game Name Change (Priority: P2)

As a player, I want to change my display name during a session so that I can correct mistakes or adopt a different identity.

**Why this priority**: Quality of life feature - useful but not essential for core identity functionality.

**Independent Test**: Can be tested by connecting with one name, using the `/name` command, and verifying the new name is applied and broadcast.

**Acceptance Scenarios**:

1. **Given** a connected player, **When** they enter `/name NewPseudo`, **Then** the server validates and updates their display name.

2. **Given** a player changed their name recently, **When** they try to change it again within the rate limit window, **Then** the server rejects the request with a "please wait" message.

3. **Given** a name change is accepted, **When** the update is processed, **Then** all connected clients receive the updated name for that player.

4. **Given** a player requests an invalid new name, **When** the server validates it, **Then** the request is rejected with an error message explaining why.

---

### User Story 5 - Session Identity Tracking (Priority: P3)

As a server operator, I want stable session identifiers for players so that I can track reconnections and correlate metrics/anti-cheat data.

**Why this priority**: Backend infrastructure feature - important for server health but not directly visible to players.

**Independent Test**: Can be tested by verifying that PlayerId and SessionId are assigned on connection and logged appropriately.

**Acceptance Scenarios**:

1. **Given** a player connects to the server, **When** their session is established, **Then** the server assigns a unique SessionId alongside their PlayerId.

2. **Given** a player is connected, **When** server logs are reviewed, **Then** connection events show PlayerId, SessionId, and final display name.

3. **Given** a player changes their display name, **When** the change is logged, **Then** the log includes their SessionId for correlation.

---

### Edge Cases

- **Empty name**: If a player provides an empty or whitespace-only name, the server assigns a fallback name (e.g., "Player").
- **Name too long**: Names exceeding the maximum length (32 characters) are truncated to the limit.
- **Invalid characters**: Names containing disallowed characters are sanitized (characters removed or replaced).
- **Rate limit exceeded**: Rapid name change attempts are rejected with a cooldown message.
- **Profile file permissions**: If the client cannot write to the profile location, it operates with defaults and logs a warning.
- **Name collision at capacity**: If all suffix variants are taken (e.g., Alex#1 through Alex#99), the server rejects the connection with "server full for this name".
- **Unicode handling**: Display names support common Unicode characters (letters, numbers, common symbols) but reject control characters and zero-width characters.

## Requirements *(mandatory)*

### Functional Requirements

#### Display Name Validation (Server)

- **FR-001**: Server MUST validate display names on connection and name change requests.
- **FR-002**: Server MUST enforce minimum name length of 1 character after trimming whitespace.
- **FR-003**: Server MUST enforce maximum name length of 32 characters.
- **FR-004**: Server MUST allow alphanumeric characters (a-z, A-Z, 0-9), underscores, hyphens, and spaces in names.
- **FR-005**: Server MUST trim leading/trailing whitespace from names.
- **FR-006**: Server MUST assign fallback name "Player" if provided name is empty or invalid after sanitization.
- **FR-007**: Server MUST apply automatic disambiguation suffix (e.g., "#2", "#3") when duplicate names occur.

#### Local Profile (Client)

- **FR-008**: Client MUST store player profile in a local file (`~/.config/plix/profile.toml` on Linux, equivalent on other platforms).
- **FR-009**: Client MUST load display name from profile on startup.
- **FR-010**: Client MUST create default profile with name "Player" if no profile exists.
- **FR-011**: Client MUST gracefully handle corrupted profile files by recreating defaults.
- **FR-012**: Client MUST save profile when display name is changed via `/name` command.

#### Session Identity (Server)

- **FR-013**: Server MUST assign a unique SessionId (64-bit) to each connection.
- **FR-014**: Server MUST maintain existing PlayerId assignment mechanism.
- **FR-015**: Server MUST log player connections with PlayerId, SessionId, and validated display name.
- **FR-016**: Server MUST log display name changes with SessionId for correlation.

#### Name Change Command (Client/Server)

- **FR-017**: Client MUST support `/name <new_name>` console command.
- **FR-018**: Server MUST rate-limit name changes to 1 per 60 seconds per player.
- **FR-019**: Server MUST broadcast name changes to all connected clients.
- **FR-020**: Server MUST include display name in PlayerSnapshot for replication.

#### Protocol & Extensibility

- **FR-021**: Protocol structures MUST include optional AccountId field (unused in v1, for future auth).
- **FR-022**: Protocol structures MUST include optional AuthToken field (unused in v1, for future auth).
- **FR-023**: Connect message MUST include display name field.
- **FR-024**: Server MUST send name change events to clients when any player's name changes.

### Key Entities

- **DisplayName**: A validated string (1-32 chars) representing a player's visible identity in-game. May include disambiguation suffix.
- **SessionId**: A unique 64-bit identifier assigned per connection, used for logging and correlation.
- **PlayerId**: Existing numeric identifier for in-game player slot (unchanged from current implementation).
- **PlayerProfile**: Client-side data structure containing display name and future extensible fields (AccountId placeholder).
- **AccountId** (v2 placeholder): Optional unique identifier for future authenticated accounts, absent in v1.
- **AuthToken** (v2 placeholder): Optional authentication credential for future login system, absent in v1.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can set and see their display name within 1 second of connecting.
- **SC-002**: Display names persist across client restarts with 100% reliability when filesystem is accessible.
- **SC-003**: Duplicate name disambiguation completes in under 10ms (O(n) where n = connected players).
- **SC-004**: Name validation completes in under 1ms per request (O(1) complexity).
- **SC-005**: Server handles name changes for 100 concurrent players without performance degradation.
- **SC-006**: All player connections are logged with SessionId and display name for 100% traceability.
- **SC-007**: Rate limiting correctly blocks rapid name changes (>1 per 60 seconds) with 100% enforcement.
- **SC-008**: Protocol remains backward-compatible with existing clients (graceful handling of optional fields).

## Assumptions

- Profile storage location follows XDG Base Directory specification on Linux (`~/.config/plix/`).
- Display name collisions within a single server are acceptable to resolve with suffixes; global uniqueness is not required.
- The 32-character name limit provides sufficient expressiveness while preventing abuse.
- 60-second rate limit for name changes balances user flexibility with spam prevention.
- SessionId is generated per connection (not persisted across reconnects in v1).
- Unicode support covers common scripts (Latin, Cyrillic, CJK) but excludes emoji and special symbols for v1 simplicity.

## Out of Scope

- Login/password authentication
- OAuth or SSO integration
- Persistent cloud accounts
- Cross-server identity
- Profanity filtering or blacklists
- Account banning by identity
- Avatar or cosmetic customization
