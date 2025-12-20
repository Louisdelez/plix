# Feature Specification: Mod API Core

**Feature Branch**: `034-mod-api-core`
**Created**: 2025-12-18
**Status**: Draft
**Input**: User description: "Mod API Core - Stable API for mods: events, world, entities, networking safe"

## Clarifications

### Session 2025-12-18

- Q: What are the timer limits per mod? → A: min_interval: 50ms, max_timers: 32
- Q: What are the default World API query bounds? → A: raycast max: 256 blocks, query_aabb limit: 128 results
- Q: After how many consecutive handler errors should a mod be auto-disabled? → A: 5 consecutive errors
- Q: Which events should be cancellable by mods? → A: on_player_chat, on_block_placed, on_block_broken only

## Overview

This feature delivers a stable core API serving as the official contract between the game engine (Rust) and mod runtimes (script/WASM in future features). It provides:

- A versioned, stable event bus system
- Safe World/Entities API (controlled access, bounded, server-authoritative)
- Safe networking API (typed messages, limits, permissions, anti-spam)
- Permissions/capabilities foundation for proper sandboxing

### Non-Negotiable Principles

1. **Server-Authoritative**: A mod can never unilaterally decide gameplay "source of truth" state
2. **Determinism/Stability**: Events and APIs must be stable enough to survive across releases
3. **Safety & Performance**: No API allowing unbounded world traversal per tick
4. **Versioning**: All exposed surface must be versioned (SemVer or API version integer) with compatibility
5. **Observability**: Mod execution logs/metrics (at minimum engine-side) and explicit errors

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Subscribe to Gameplay Events (Priority: P1)

As a mod developer, I want to subscribe to stable gameplay events (player join, chat, block placed, etc.) without polling, so my mod can react to game state changes efficiently.

**Why this priority**: Event subscription is the foundation of mod interaction with the game. Without events, mods cannot respond to anything happening in the game world.

**Independent Test**: Can be fully tested by loading a test mod that subscribes to `on_player_join` event and logs the player name. Delivers immediate value by enabling reactive mod behavior.

**Acceptance Scenarios**:

1. **Given** a mod with valid manifest and event.chat subscription, **When** a player sends a chat message, **Then** the mod's handler receives the event with player_id and text
2. **Given** a mod subscribed to on_block_placed, **When** a player places a block, **Then** the mod receives pos, block_id, and optional player_id
3. **Given** a mod handler that throws an error, **When** the event is dispatched, **Then** the engine logs the error with mod_id but does not crash
4. **Given** multiple mods subscribed to same event, **When** the event fires, **Then** handlers execute in deterministic FIFO order

---

### User Story 2 - Read and Write World Data Safely (Priority: P1)

As a mod developer, I want to read and modify world data (blocks, entities) through a safe, bounded API, so I can create gameplay modifications without risking server stability.

**Why this priority**: World manipulation is core to voxel game modding. Mods need to read terrain and spawn/modify entities to create meaningful gameplay changes.

**Independent Test**: Can be tested by loading a mod that calls `get_block(pos)` to read terrain and `set_block(pos, block_id)` with permission to modify it. Verifies bounded queries work correctly.

**Acceptance Scenarios**:

1. **Given** a mod with world.read capability, **When** calling get_block(valid_pos), **Then** returns the block type at that position
2. **Given** a mod with world.read capability, **When** calling raycast(origin, dir, 100.0), **Then** returns hit result within bounded distance
3. **Given** a mod without world.write capability, **When** calling set_block(), **Then** returns EMOD002 (permission_denied)
4. **Given** a mod calling query_aabb with a 1000-block volume, **When** limit is 100, **Then** returns at most 100 results
5. **Given** a position in an unloaded chunk, **When** get_block() is called, **Then** returns EMOD006 (world_not_ready)

---

### User Story 3 - Control Mod Permissions (Priority: P1)

As a server administrator, I want to control what mods are allowed to do via capabilities, so I can ensure server security and prevent abuse.

**Why this priority**: Without permission control, any mod could manipulate the game arbitrarily, creating security risks and unfair gameplay.

**Independent Test**: Can be tested by configuring server to deny world.write capability for a specific mod, then verifying the mod's set_block calls return permission_denied.

**Acceptance Scenarios**:

1. **Given** a mod manifest requesting world.write, **When** server config denies this capability, **Then** the mod loads but set_block returns EMOD002
2. **Given** a mod manifest without net.send capability, **When** mod tries to send network message, **Then** call fails with EMOD002
3. **Given** a mod with all requested capabilities granted, **When** mod makes authorized API calls, **Then** calls succeed normally
4. **Given** an invalid manifest (missing required fields), **When** engine loads mod, **Then** load fails with clear error message

---

### User Story 4 - Send/Receive Mod Network Messages (Priority: P2)

As a mod developer, I want to send and receive typed network messages between server and clients, so I can implement client-side mod features that communicate with the server.

**Why this priority**: Many mods require client-server communication (custom UIs, synchronized state). This enables richer mod experiences but is secondary to core event/world access.

**Independent Test**: Can be tested by loading a mod that sends a message on channel "mod:testmod:ping" from client to server, and verifying server receives it with correct payload.

**Acceptance Scenarios**:

1. **Given** a mod with net.send capability, **When** sending a 1KB message, **Then** message is delivered successfully
2. **Given** a mod sending messages at 30 msg/s, **When** rate limit is 20 msg/s, **Then** excess messages return EMOD005 (rate_limited)
3. **Given** a mod sending a 16KB message, **When** max payload is 8KB, **Then** returns EMOD001 (invalid_argument)
4. **Given** a mod without net.send capability, **When** trying to send message, **Then** returns EMOD002 (permission_denied)
5. **Given** a mod subscribed to on_mod_message, **When** message arrives on its channel, **Then** handler receives channel, from, and payload

---

### User Story 5 - Validate API Version Compatibility (Priority: P2)

As a mod developer, I want the engine to validate my mod's API version requirements at load time, so I know immediately if my mod is incompatible.

**Why this priority**: Version compatibility prevents runtime crashes and confusing behavior. It's essential for a healthy mod ecosystem but doesn't block basic functionality.

**Independent Test**: Can be tested by creating a mod manifest with api_version=99, attempting to load it, and verifying EMOD007 error is returned.

**Acceptance Scenarios**:

1. **Given** a mod requiring api_version=1 and engine providing api_version=1, **When** mod loads, **Then** load succeeds
2. **Given** a mod requiring min_api_version=2 and engine providing api_version=1, **When** mod loads, **Then** returns EMOD007 (unsupported)
3. **Given** a loaded mod, **When** calling get_api_version(), **Then** returns the engine's current API version
4. **Given** a loaded mod, **When** calling get_engine_version(), **Then** returns the engine's SemVer string

---

### User Story 6 - Use Bounded Timers (Priority: P3)

As a mod developer, I want to schedule timed callbacks with set_timeout/set_interval, so I can implement delayed or periodic mod logic without blocking the main loop.

**Why this priority**: Timers enable advanced mod patterns but are not required for basic mod functionality. Event-driven patterns are preferred.

**Independent Test**: Can be tested by calling set_timeout(500ms, callback) and verifying the callback executes approximately 500ms later.

**Acceptance Scenarios**:

1. **Given** a mod calling set_timeout(1000, callback), **When** 1000ms passes, **Then** callback executes
2. **Given** a mod calling set_interval(100, callback), **When** min_interval is 200ms, **Then** interval is clamped to 200ms
3. **Given** a mod with 50 active timers, **When** max_timers is 50, **Then** next set_timeout returns EMOD004 (out_of_bounds)
4. **Given** a mod subscribed to on_tick, **When** tick executes, **Then** handler runs (with throttle warnings if overused)

---

### Edge Cases

- What happens when a mod handler takes too long? Engine should log warning and potentially throttle/disable mod
- How does system handle recursive event emission? Re-entrancy must be prevented (queue events, don't dispatch immediately from handler)
- What happens when block position is outside world bounds? Return EMOD004 (out_of_bounds)
- How does system handle malformed mod manifest? Clear error message with specific validation failure
- What happens if mod tries to iterate all entities? Must require filter + limit; unbounded iteration not allowed

## Requirements *(mandatory)*

### Functional Requirements

#### Mod Manifest

- **FR-001**: System MUST support mod.toml manifest format with id, name, version (SemVer), optional author
- **FR-002**: System MUST require api_version field in manifest (integer)
- **FR-003**: System MUST support optional entrypoints declaration (server, client, ui)
- **FR-004**: System MUST support capabilities/permissions list in manifest
- **FR-005**: System MUST support optional dependencies list in manifest
- **FR-006**: System MUST validate manifest strictly at load time with clear error messages

#### Event Bus

- **FR-007**: System MUST provide event subscription mechanism for game events
- **FR-008**: System MUST emit events in deterministic FIFO order per event type
- **FR-009**: System MUST prevent re-entrancy (events queued, not immediate dispatch from handlers)
- **FR-010**: System MUST isolate mod handler errors (no engine crash on mod error)
- **FR-011**: System MUST auto-disable a mod after 5 consecutive handler errors (with warning logs)
- **FR-012**: System MUST provide minimum MVP events: on_server_start, on_server_stop, on_player_join, on_player_leave, on_player_chat, on_block_placed, on_block_broken, on_entity_damaged, on_mod_message
- **FR-012a**: System MUST support event cancellation for: on_player_chat, on_block_placed, on_block_broken (other events are read-only notifications)

#### World API

- **FR-013**: System MUST provide get_block(pos) returning block type
- **FR-014**: System MUST provide raycast(origin, dir, max_dist) with max_dist capped at 256 blocks
- **FR-015**: System MUST provide query_aabb(bounds, limit) with limit capped at 128 results
- **FR-016**: System MUST provide set_block(pos, block_id) requiring world.write capability
- **FR-017**: System MUST provide spawn_entity(type, transform) requiring entity.write capability
- **FR-018**: System MUST provide despawn_entity(id) requiring entity.write capability
- **FR-019**: System MUST validate all positions (chunk loaded, in bounds, valid block_id)
- **FR-020**: System MUST return Result with typed error codes (never panic)

#### Entity API

- **FR-021**: System MUST provide get_transform(id) for entity position/rotation
- **FR-022**: System MUST provide get_velocity(id) if applicable
- **FR-023**: System MUST provide get_health(id) if applicable
- **FR-024**: System MUST provide get_owner(id)/get_team(id) if applicable
- **FR-025**: System MUST provide apply_damage(id, amount) requiring entity.write capability
- **FR-026**: System MUST provide apply_impulse(id, vec3) requiring entity.write capability
- **FR-027**: System MUST NOT allow direct memory/ECS access
- **FR-028**: System MUST NOT allow unbounded entity iteration (filter + limit required)

#### Permissions/Capabilities

- **FR-029**: System MUST require capability declaration in manifest
- **FR-030**: System MUST check capabilities at API call time
- **FR-031**: System MUST support server config override of mod capabilities
- **FR-032**: System MUST support MVP capabilities: world.read, world.write, entity.read, entity.write, net.send
- **FR-033**: System MUST reject undeclared capability usage with EMOD002

#### Networking

- **FR-034**: System MUST support mod channels in format mod:<mod_id>:<name>
- **FR-035**: System MUST enforce max payload size (8KB default)
- **FR-036**: System MUST enforce rate limiting (20 msg/s default, configurable)
- **FR-037**: System MUST require net.send capability for message sending
- **FR-038**: System MUST support server->client and client->server messaging
- **FR-039**: System MUST log rate limit violations and abuse

#### Timers

- **FR-040**: System MUST provide set_timeout(ms, callback) with minimum interval of 50ms
- **FR-041**: System MUST provide set_interval(ms, callback) with minimum interval of 50ms
- **FR-042**: System MUST enforce maximum of 32 active timers per mod
- **FR-043**: System MUST provide on_tick event with throttle warnings for overuse

#### Error Model

- **FR-044**: System MUST return Result<T, ModApiError> from all API calls
- **FR-045**: System MUST use standard error codes: EMOD001-EMOD007
- **FR-046**: System MUST log errors with mod_id, API call name, and parameter summary

#### Versioning

- **FR-047**: System MUST expose get_api_version() returning current API version integer
- **FR-048**: System MUST expose get_engine_version() returning SemVer string
- **FR-049**: System MUST check api_version compatibility at mod load time
- **FR-050**: System MUST support optional min_api_version/max_api_version in manifest

### Key Entities

- **ModManifest**: Represents mod metadata (id, name, version, api_version, capabilities, entrypoints, dependencies)
- **ModInstance**: A loaded mod with granted capabilities, subscribed events, active timers
- **GameEvent**: An event emitted by the engine (type, payload, timestamp, cancellable flag)
- **ModApiError**: Error type with code (EMOD001-007), message, and context (mod_id, api_call)
- **Capability**: Permission type that mods request and server grants/denies
- **ModChannel**: Network channel for mod messages (mod_id, channel_name, direction)
- **EntityHandle**: Opaque handle to an entity for safe API access

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Mods can subscribe to and receive events within 1 server tick of the event occurring
- **SC-002**: World API queries (get_block, raycast, query_aabb) complete within bounds without affecting server tick rate
- **SC-003**: Permission checks add less than 1ms overhead per API call on average
- **SC-004**: Invalid mod manifests are rejected with error messages identifying the specific validation failure
- **SC-005**: A mod handler crash/error does not crash the game server
- **SC-006**: Rate-limited messages return EMOD005 within 10ms of limit violation
- **SC-007**: API version mismatch is detected and reported with EMOD007 before any mod code executes
- **SC-008**: All API calls return typed Result with appropriate error codes (no panics)
- **SC-009**: Mod execution metrics (event count, API calls, errors) are observable via engine logs
- **SC-010**: 100% of "sensitive" APIs (write operations) verify capability before execution

## Assumptions

- The WASM/script runtime will be implemented in Feature 035; this feature provides the Rust-side API surface and traits
- Mods will be loaded from local filesystem; distribution/workshop is out of scope
- Hot reload of mods is not supported in this version
- File system and HTTP access from mods are explicitly out of scope for security
- The API is designed to be callable via FFI/ABI for future WASM integration

## Out of Scope

- Full sandbox runtime (WASM/Lua/JS) - Feature 035
- Mod distribution, workshop, signatures - future feature
- UI injection from mods - future feature
- Native FS/HTTP access from mods
- Hot reload of mods
- Fine-grained per-world-area permissions
