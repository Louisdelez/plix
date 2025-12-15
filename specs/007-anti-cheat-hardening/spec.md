# Feature Specification: Anti-Cheat Hardening

**Feature Branch**: `007-anti-cheat-hardening`
**Created**: 2025-12-15
**Status**: Draft
**Input**: MVP anti-cheat layer with strict input validation, rate limiting, physics sanity checks, and automatic progressive sanctions

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Strict Input Validation (Priority: P1)

As a game server, I must reject any invalid or inconsistent input to prevent modified clients from corrupting game state. This is the foundation of all anti-cheat measures - if inputs cannot be trusted, no other protection matters.

**Why this priority**: This is the most fundamental anti-cheat measure. Invalid inputs (NaN, INF, out-of-bounds values) can crash the server or corrupt game state. Without this, all other protections are meaningless.

**Independent Test**: Can be fully tested by sending malformed inputs from a test client and verifying the server rejects them without crashing, while legitimate inputs continue to work.

**Acceptance Scenarios**:

1. **Given** a connected player, **When** they send an input containing NaN or INF values, **Then** the input is rejected, an infraction is recorded, and the server continues operating normally
2. **Given** a connected player, **When** they send position/rotation values outside valid bounds, **Then** the input is rejected and an infraction is recorded
3. **Given** a connected player, **When** they send an input with a sequence number that is out of order or duplicated, **Then** the input is ignored
4. **Given** any possible byte sequence from a client, **When** the server attempts to process it, **Then** the server never panics or crashes

---

### User Story 2 - Rate Limiting (Priority: P1)

As a game server, I must limit the frequency of player actions to prevent spam and flood attacks. Players should not be able to gain unfair advantages by sending actions faster than intended.

**Why this priority**: Rate limiting prevents the most obvious exploits (attack spam, action flood) and is essential for fair gameplay. Without rate limits, a modified client could fire attacks infinitely or flood the server.

**Independent Test**: Can be fully tested by having a test client send actions faster than allowed rates and verifying infractions are recorded while legitimate-rate actions work normally.

**Acceptance Scenarios**:

1. **Given** a player sending movement inputs, **When** they exceed the maximum inputs per second, **Then** excess inputs are rejected and an infraction is recorded
2. **Given** a player sending attack commands, **When** they exceed the maximum attacks per second, **Then** excess attacks are rejected and an infraction is recorded
3. **Given** a player sending block edit requests, **When** they exceed the maximum block edits per second, **Then** excess edits are rejected and an infraction is recorded
4. **Given** a player toggling ready status, **When** they spam the toggle action, **Then** excess toggles are rejected and an infraction is recorded

---

### User Story 3 - Physics Sanity Checks (Priority: P1)

As a game server, I must detect and block physically impossible movements to prevent speed hacks and teleportation exploits. The server's physics simulation is authoritative.

**Why this priority**: Speed hacks and teleportation are common cheats that completely break gameplay. Since the server is authoritative, it must validate that client-claimed positions are physically achievable.

**Independent Test**: Can be fully tested by having a test client claim positions that would require impossible speed or teleportation, and verifying the server rejects them while allowing normal movement.

**Acceptance Scenarios**:

1. **Given** a player's previous position, **When** they claim a new position that exceeds the maximum distance per tick, **Then** the server uses its authoritative position and records an infraction
2. **Given** a player's velocity history, **When** they claim acceleration exceeding the maximum allowed, **Then** the server uses its authoritative velocity and records an infraction
3. **Given** a player's movement, **When** their implied speed exceeds the maximum allowed, **Then** the server uses its authoritative state and records an infraction

---

### User Story 4 - Automatic Sanctions (Priority: P2)

As a game server, I must apply progressive sanctions to players who accumulate infractions. The system should be fair (warnings first) but firm (kick then ban for repeat offenders).

**Why this priority**: Sanctions are the enforcement mechanism for all detections. Without consequences, detection is meaningless. However, the detection systems (US1-3) are functional without sanctions - they still protect game state.

**Independent Test**: Can be fully tested by simulating a player accumulating infractions and verifying they receive warnings, then get kicked, then get banned at the correct thresholds.

**Acceptance Scenarios**:

1. **Given** a player with few infractions, **When** they commit another infraction below the kick threshold, **Then** a warning is logged and the player receives a warning message
2. **Given** a player who reaches the kick threshold, **When** they commit another infraction, **Then** they are disconnected with a reason message explaining why
3. **Given** a player who reaches the ban threshold, **When** they commit another infraction, **Then** they are banned for the configured duration and receive a message with the ban reason and duration
4. **Given** a banned player, **When** they attempt to reconnect before the ban expires, **Then** the connection is rejected with a message explaining the remaining ban time
5. **Given** a kicked (not banned) player, **When** they reconnect, **Then** they are allowed to join normally

---

### Edge Cases

- What happens when a client sends messages before completing the connection handshake? They are rejected - no player state exists yet.
- What happens when a client sends inputs after being kicked? They are ignored - the player session no longer exists.
- What happens when a legitimate player has network lag causing bursty inputs? Rate limits are tuned to allow normal lag bursts (e.g., 120 inputs/sec allows ~2 seconds of queued inputs at 60Hz tick rate).
- What happens when a player reconnects after being kicked? They are allowed to rejoin - kicks are temporary disconnections.
- What happens when a player reconnects after being banned? The connection is rejected until the ban expires.
- What happens when the server restarts? Ban list is cleared (in-memory only for MVP).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Server MUST validate all numeric fields in player inputs (position, rotation, velocity) for NaN and INF values
- **FR-002**: Server MUST validate all numeric fields are within defined bounds (e.g., position within world limits, rotation within valid angles)
- **FR-003**: Server MUST track input sequence numbers and reject out-of-order or duplicate inputs
- **FR-004**: Server MUST apply per-action rate limits: movement inputs, attacks, block edits, ready toggles
- **FR-005**: Rate limits MUST be configurable without code changes
- **FR-006**: Server MUST verify distance traveled per tick does not exceed maximum allowed speed
- **FR-007**: Server MUST verify velocity changes do not exceed maximum allowed acceleration
- **FR-008**: Server MUST maintain an infraction counter per connected player
- **FR-009**: Server MUST apply warning sanctions at configurable warning threshold
- **FR-010**: Server MUST apply kick sanctions at configurable kick threshold
- **FR-011**: Server MUST apply temporary ban sanctions at configurable ban threshold
- **FR-012**: Server MUST send sanction reason to the client when kicking or banning
- **FR-013**: Server MUST maintain a ban list (in memory) with player identifier and expiry time
- **FR-014**: Server MUST reject connection attempts from banned players
- **FR-015**: All validation MUST be performed without causing server panics regardless of input

### Key Entities

- **PlayerInfraction**: Tracks per-player infraction count, types of infractions, and timestamps
- **RateLimitState**: Tracks per-player action counts within rolling time windows
- **BanEntry**: Contains player identifier, ban reason, and expiry timestamp
- **AntiCheatConfig**: Contains all configurable thresholds and limits

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Server handles any malformed input without crashing or panicking (100% crash resistance)
- **SC-002**: All anti-cheat checks complete in under 1 microsecond per player per tick on average
- **SC-003**: Anti-cheat checks require no heap allocations during normal operation (per-tick)
- **SC-004**: Legitimate players with normal network conditions experience zero false positive infractions
- **SC-005**: Speed hack attempts (>150% normal speed) are detected within 3 ticks
- **SC-006**: Input flood attacks (>200% normal rate) are detected and rate-limited within 1 second
- **SC-007**: Players reaching ban threshold are disconnected within 1 tick of the triggering infraction

## Assumptions

- Server tick rate is 60 Hz (16.67ms per tick)
- Normal player input rate is approximately 60 inputs per second (one per tick)
- Maximum legitimate movement speed is known and configured (based on game movement system)
- Player identification for bans uses socket address (IP:port) for MVP
- Ban list does not persist across server restarts (MVP simplification)
- Network latency jitter of up to 200ms is considered normal and should not cause false positives

## Configuration Defaults

- `max_inputs_per_second`: 120 (allows 2x normal rate for lag compensation)
- `max_attacks_per_second`: 4 (based on attack cooldown system)
- `max_block_edits_per_second`: 10 (based on block edit cooldown)
- `max_speed_per_tick`: 0.25 units (based on movement system max speed)
- `max_acceleration`: 1.5 units/tick^2 (based on movement physics)
- `warning_threshold`: 3 infractions
- `kick_threshold`: 5 infractions
- `ban_threshold`: 10 infractions
- `ban_duration_seconds`: 3600 (1 hour)
