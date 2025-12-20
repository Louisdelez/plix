# Feature Specification: Server Mods + Client Sync

**Feature Branch**: `037-server-mods`
**Created**: 2025-12-19
**Status**: Draft
**Input**: Server-only mods execution with optional client data synchronization

## Overview

Enable server-only mod execution where WASM mods run exclusively on the server, while supporting an optional synchronization mechanism to send data/configuration payloads to clients when necessary. The server remains the source of truth - clients never execute mod code, only receive synchronized data, configuration, and safe network messages through existing APIs (Features 034/035).

**Key Principles**:
- Multiplayer compatibility: Players can join modded servers without client-side mod installation
- Security: Anti-spoof and anti-tampering through versioning, hashing, and integrity verification
- Robustness: Version locking and reproducible mod sets via lockfile (Feature 036)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Join Server-Only Modded Server (Priority: P1)

As a player, I can join a server running server-only mods without installing anything on my client. The mods execute entirely on the server, and I experience the modded gameplay seamlessly.

**Why this priority**: This is the core value proposition - enabling modded multiplayer without client-side requirements, reducing friction for players and simplifying server administration.

**Independent Test**: Connect a vanilla client to a server with 2-3 server-only mods active. Player should join successfully and experience mod effects (modified game rules, custom items, etc.) without any client-side downloads or installations.

**Acceptance Scenarios**:

1. **Given** a server with server-only mods configured, **When** a player connects with a vanilla client, **Then** the connection succeeds and player joins the game
2. **Given** a server with multiple server-only mods, **When** the server sends the mod set descriptor during handshake, **Then** the client receives mod metadata (IDs, versions) for display purposes only
3. **Given** a server-only mod modifies game rules, **When** a player is in-game, **Then** the player experiences the modified rules enforced by the server

---

### User Story 2 - Client-Required Mod Enforcement (Priority: P2)

As a server administrator, I can designate certain mods as "client-required" and block clients that don't meet the requirements. This ensures gameplay consistency when mods require client-side awareness.

**Why this priority**: Some mods may need client-side data for proper functionality (custom UI elements, item definitions). This story ensures compatibility enforcement when needed.

**Independent Test**: Configure a server with one client-required mod. Test with: (a) a client without the requirement - should be refused with clear message, (b) a client with cached payload matching - should join successfully.

**Acceptance Scenarios**:

1. **Given** a server with a client-required mod, **When** a client without the required payload connects, **Then** the client receives a clear refusal message listing missing requirements
2. **Given** a client-required mod with client payload, **When** a client with matching cached payload connects, **Then** the connection succeeds without re-download
3. **Given** a client-required mod, **When** the server configuration changes to allow client payload sync, **Then** clients can receive the payload during handshake

---

### User Story 3 - Client Data Payload Synchronization (Priority: P2)

As a server, I can synchronize data-only payloads (configuration, item definitions, UI strings) to clients in a secure and efficient manner with size limits, chunking, and integrity verification.

**Why this priority**: Equal to US2 because it enables the "client-required" enforcement to work in practice. Without sync capability, client-required mods would always fail for new players.

**Independent Test**: Configure a mod with a 5MB client payload. Connect a new client without cache. Verify: payload is transferred in chunks, SHA-256 verified, cached locally, and subsequent connections skip download.

**Acceptance Scenarios**:

1. **Given** a mod with client payload and sync enabled, **When** a client without the payload connects, **Then** the server streams the payload in configurable chunks (default 256KB)
2. **Given** a payload being transferred, **When** all chunks are received, **Then** the client verifies SHA-256 integrity before accepting
3. **Given** a client with cached payload matching server hash, **When** connecting, **Then** no re-download occurs (cache hit)
4. **Given** a payload exceeding max size limit, **When** server validates mod, **Then** the server rejects the mod configuration with clear error

---

### User Story 4 - Mod Network Channels (Priority: P3)

As a mod developer, I can send data messages from server-side mod code to connected clients via dedicated mod channels, enabling UI updates, notifications, and custom data display without executing any code on the client.

**Why this priority**: This enables server-only mods to provide rich client experiences (scoreboards, notifications, dynamic UI data) without requiring client-side mod installation, but is not critical for basic mod functionality.

**Independent Test**: Create a server-only mod that sends a custom message to a player. Verify the client receives the message on the correct channel and can display it (or ignore it safely if not supported).

**Acceptance Scenarios**:

1. **Given** a server-only mod with network capability declared, **When** the mod sends a message to a player, **Then** the client receives it on the `mod:<id>:*` channel
2. **Given** a mod declares allowed client-to-server channels, **When** a client sends a message on an allowed channel, **Then** the server mod receives it (rate-limited)
3. **Given** a client sends a message on a non-allowed channel, **When** the server processes it, **Then** the message is rejected silently

---

### Edge Cases

- What happens when client payload hash doesn't match after transfer? Client discards payload, clears cache entry, and connection fails with integrity error
- How does system handle interrupted payload transfer mid-stream? Client discards partial data, can retry on reconnection
- What happens when server mod set changes while client is connected? Current session continues; new session will use updated mod set
- How does system handle client without sync support connecting to server requiring sync? Connection refused with clear message if `require_client_payload_sync=true`
- What happens when payload size exactly equals max limit? Accepted; only payloads exceeding limit are rejected
- How does system handle malformed mod manifest (invalid runtime value)? Server startup fails with validation error listing the problematic mod

## Requirements *(mandatory)*

### Functional Requirements

#### Mod Classification

- **FR-001**: System MUST extend mod manifest to include a `runtime` field with values: "server" (default), "client", or "both"
- **FR-002**: System MUST support a `client_payload` boolean field in mod manifest indicating whether the mod has data to sync to clients
- **FR-003**: System MUST support a `client_payload_manifest` field listing files/data to include in the client payload when `client_payload=true`
- **FR-004**: System MUST validate mod manifests at server startup and reject invalid configurations with clear error messages

#### Handshake Protocol

- **FR-005**: Server MUST send a ModSetDescriptor to connecting clients containing: engine version, API version, and mod list (from lockfile)
- **FR-006**: ModSetDescriptor mod entries MUST include: id, version, SHA-256 hash, runtime mode, client payload hash (if applicable), required flag
- **FR-007**: Client MUST respond with: sync capability flag and list of cached payload hashes
- **FR-008**: Server MUST make join decision based on policy configuration and client response

#### Join Policy

- **FR-009**: Server MUST allow connections from clients when all mods are server-only (default behavior)
- **FR-010**: Server MUST refuse connections when client lacks required client-required mods (with clear error message)
- **FR-011**: Server MUST support configurable policy for payload sync: allow/require/deny
- **FR-012**: System MUST provide clear, user-friendly disconnect messages explaining why a join was refused

#### Payload Synchronization

- **FR-013**: System MUST support streaming client payloads in configurable chunks (default 256KB)
- **FR-014**: System MUST enforce a maximum payload size limit (default 25MB, configurable)
- **FR-015**: System MUST limit concurrent in-flight chunks (default 8) to prevent memory exhaustion
- **FR-016**: Client MUST verify SHA-256 hash of complete payload before accepting
- **FR-017**: Client MUST cache payloads by hash to avoid redundant downloads
- **FR-018**: System MUST delete/reject payloads failing integrity verification

#### Network Channels

- **FR-019**: Server MUST allow mods to send messages to clients on channels matching `mod:<mod_id>:*` pattern
- **FR-020**: Client-to-server messages MUST only be accepted on channels explicitly allowed by the mod manifest
- **FR-021**: All mod network messages MUST respect existing rate limits (default 20 msg/s) and size limits (8KB payload)

#### Configuration

- **FR-022**: Server MUST support join policy configuration via `server_mods.toml` or Feature 036 config extension
- **FR-023**: System MUST support configuring: sync enabled, max payload size, chunk size, max inflight chunks
- **FR-024**: Configuration MUST have sensible defaults that work out-of-the-box for common use cases

#### Observability

- **FR-025**: System MUST log handshake events: mod set sent, join decision, refusal reason
- **FR-026**: System MUST log sync events: payload requested, transfer progress (debug level), verification result
- **FR-027**: System MUST expose metrics: joins refused (mod mismatch), payload bytes transferred, sync failures, cache hits

### Key Entities

- **ModSetDescriptor**: Server's complete mod configuration sent to clients during handshake. Contains engine version, API version, and list of mod entries with their runtime classification and payload hashes.

- **ModEntry**: Single mod's metadata within ModSetDescriptor. Includes id, version, SHA-256 hash, runtime mode (server/client/both), optional client payload hash, and required flag.

- **ClientCapabilities**: Client's response during handshake indicating sync support and cached payload hashes.

- **JoinDecision**: Server's determination after evaluating client capabilities against mod requirements and configured policy.

- **ClientPayload**: Data-only archive (non-executable) containing configuration, definitions, strings for client-side consumption. Identified by SHA-256 hash.

- **PayloadChunk**: Fragment of client payload for streaming transfer. Contains sequence number, data, and parent payload reference.

- **JoinPolicy**: Server configuration defining rules for mod compatibility checking and sync behavior.

- **ModChannel**: Named communication channel for mod-to-client messages, following `mod:<id>:<subchannel>` naming convention.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can join server-only modded servers in under 5 seconds (same as unmodded join time)
- **SC-002**: Client payload sync completes within 30 seconds for payloads up to 25MB on typical broadband connections (10 Mbps)
- **SC-003**: Cache hit rate for returning players reaches 95%+ (no redundant payload downloads)
- **SC-004**: Join refusal messages are understood by 90%+ of players (clear, actionable text)
- **SC-005**: Server startup validates all mod configurations within 2 seconds for up to 50 mods
- **SC-006**: Memory overhead per connected client stays below 1MB during payload sync
- **SC-007**: System handles 100 concurrent payload syncs without degradation
- **SC-008**: Mod network messages are delivered to clients within 100ms of server dispatch

## Scope Boundaries

### In Scope

- Mod runtime classification (server/client/both)
- Client-server handshake with mod set descriptor
- Join policy enforcement
- Data-only payload synchronization with chunking
- Client payload caching by hash
- SHA-256 integrity verification
- Mod network channels (server-to-client primary, client-to-server allowlisted)
- Server configuration for policies and limits
- Logging and metrics for observability

### Out of Scope

- Client-side mod execution (WASM runtime on client) - future feature
- In-game UI for mod downloads/management - future feature
- Heavy asset streaming (HD textures, audio) - future optimization
- Delta patching for payloads - future optimization
- Peer-to-peer payload sync - not planned
- Signature verification of payloads (handled by Feature 036 at bundle level)

## Assumptions

- Feature 034 (Mod API Core) provides the network capability infrastructure
- Feature 035 (WASM Runtime) provides server-side mod execution
- Feature 036 (Mod Distribution) provides lockfile and mod integrity via SHA-256
- Clients have persistent local storage for payload caching
- Network bandwidth of at least 1 Mbps is available for payload sync
- Mod developers will use reasonable payload sizes (under 25MB for typical mods)

## Dependencies

- **Feature 034**: Mod API Core - network channel infrastructure
- **Feature 035**: WASM Runtime - server-side mod execution sandbox
- **Feature 036**: Mod Distribution - lockfile, SHA-256 hashes, mod metadata
