# Feature Specification: Server-Authoritative Block Interaction

**Feature Branch**: `004-block-interaction`
**Created**: 2025-12-14
**Status**: Draft
**Input**: User description: "Add server-authoritative block interactions (place/remove) so the voxel world becomes interactive in multiplayer, while keeping the competitive/low-latency architecture intact."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Remove Block (Priority: P1)

As a player, I can remove (break) blocks from the voxel world by targeting them and triggering a remove action, and see them disappear for myself and all other players in real-time.

**Why this priority**: Block removal is the most fundamental world interaction - it enables players to modify terrain and creates the core interactive gameplay loop. Without this, the world is static.

**Independent Test**: Start server + 2 clients, have player A remove a block, verify both player A and player B see the block disappear within acceptable latency.

**Acceptance Scenarios**:

1. **Given** a player is within interaction range of a solid block, **When** they aim at the block and trigger the remove action, **Then** the block is removed from the world and disappears for all connected clients.

2. **Given** a player aims at a solid block beyond interaction range, **When** they trigger the remove action, **Then** the action is rejected and the block remains.

3. **Given** a player aims at empty space (no block), **When** they trigger the remove action, **Then** the action is rejected with no effect.

4. **Given** a player triggers remove actions rapidly, **When** the rate exceeds the cooldown limit, **Then** excess actions are rejected until the cooldown expires.

---

### User Story 2 - Place Block (Priority: P1)

As a player, I can place blocks in the voxel world by targeting an adjacent empty cell and triggering a place action, and see the new block appear for myself and all other players in real-time.

**Why this priority**: Block placement complements removal and is equally critical for interactive gameplay - enables building, creating cover, and modifying the arena strategically.

**Independent Test**: Start server + 2 clients, have player A place a block on an empty cell adjacent to existing terrain, verify both player A and player B see the new block appear.

**Acceptance Scenarios**:

1. **Given** a player is within interaction range of an empty cell adjacent to a solid block, **When** they aim at the target face and trigger the place action, **Then** a block is placed in the empty cell and appears for all connected clients.

2. **Given** a player aims at a cell beyond interaction range, **When** they trigger the place action, **Then** the action is rejected and no block is placed.

3. **Given** a player aims at a cell that is already occupied, **When** they trigger the place action, **Then** the action is rejected and no change occurs.

4. **Given** a player attempts to place a block where another player is standing, **When** they trigger the place action, **Then** the action is rejected to prevent trapping players inside blocks.

---

### User Story 3 - Server Validation (Priority: P2)

As a developer/tester, I can confirm that all block edits are validated server-side, ensuring clients cannot cheat by sending invalid edit requests.

**Why this priority**: Server authority is critical for competitive integrity but depends on US1/US2 working first. This story ensures the anti-cheat foundation is solid.

**Independent Test**: Send malformed or cheating edit requests directly to server (bypassing normal client), verify all are rejected with appropriate reasons.

**Acceptance Scenarios**:

1. **Given** a client sends an edit request for out-of-bounds coordinates, **When** the server processes it, **Then** the request is rejected with "out of bounds" reason.

2. **Given** a client sends an edit request for a position beyond max interaction range from their position, **When** the server processes it, **Then** the request is rejected with "out of range" reason.

3. **Given** a client sends edit requests faster than the allowed rate limit, **When** the server processes them, **Then** excess requests are rejected with "rate limited" reason.

---

### User Story 4 - Late Joiner World Sync (Priority: P2)

As a player joining a server mid-match, I see the correct current state of the world including all block edits made before I joined.

**Why this priority**: Essential for multiplayer usability but depends on edit replication (US1/US2) working first. Late joiners must see consistent world state.

**Independent Test**: Start server, connect client A, make several block edits, then connect client B - verify client B sees the exact same world state as client A.

**Acceptance Scenarios**:

1. **Given** a server has been running with players making block edits, **When** a new player connects, **Then** they receive and display the current world state including all prior edits.

2. **Given** a player disconnects and reconnects during an ongoing match, **When** they reconnect, **Then** they see the current world state with any edits made while they were away.

---

### User Story 5 - Debug Feedback (Priority: P3)

As a developer/tester, I see minimal debug feedback for block interactions showing what actions succeeded or were rejected.

**Why this priority**: Lower priority than core functionality but valuable for testing and debugging. Minimal scope - just enough to verify behavior.

**Independent Test**: Perform various block actions (valid and invalid), verify debug output shows appropriate feedback for each action type.

**Acceptance Scenarios**:

1. **Given** a player successfully removes a block, **When** the action completes, **Then** a "Block removed" indicator briefly appears in the debug HUD.

2. **Given** a player successfully places a block, **When** the action completes, **Then** a "Block placed" indicator briefly appears in the debug HUD.

3. **Given** a player's block action is rejected, **When** the rejection occurs, **Then** a brief "Edit rejected: [reason]" message appears in the debug HUD.

---

### Edge Cases

- What happens when a player tries to remove/place blocks while dead? (Actions should be ignored)
- What happens when two players try to edit the same block simultaneously? (Server processes in tick order - first valid edit wins)
- How does the system handle block edits during match phase transitions? (Edits should only be allowed during Playing phase)
- What happens when a player disconnects mid-edit? (Pending edits are discarded)
- What happens when world boundary blocks are targeted? (Follow arena's protected region rules if any)

## Requirements *(mandatory)*

### Functional Requirements

#### Client Input

- **FR-001**: Client MUST support a discrete "remove block" action (not continuous/held)
- **FR-002**: Client MUST support a discrete "place block" action with a default block type
- **FR-003**: Client MUST perform client-side raycasting to determine target block position
- **FR-004**: Client MUST send target block coordinates to server (not apply changes locally)
- **FR-005**: Client MUST include both target position and action type in the request

#### Server Validation

- **FR-006**: Server MUST validate that target coordinates are within world bounds
- **FR-007**: Server MUST validate that target is within max interaction range (5 blocks) from player position
- **FR-008**: Server MUST validate that remove actions target occupied cells
- **FR-009**: Server MUST validate that place actions target empty cells
- **FR-010**: Server MUST enforce rate limiting (max 4 edits per second per player)
- **FR-011**: Server MUST reject place actions that would trap a player inside a block
- **FR-012**: Server MUST only process block actions during Playing match phase
- **FR-013**: Server MUST reject block actions from dead players

#### World State

- **FR-014**: Server MUST be the single source of truth for world block data
- **FR-015**: Server MUST apply valid block edits atomically at tick boundaries
- **FR-016**: Server MUST track all block changes since match start for late joiners

#### Replication

- **FR-017**: Server MUST broadcast block edit events to all connected clients reliably
- **FR-018**: Server MUST send current world state (including edits) to newly connecting clients
- **FR-019**: Block edit events MUST include position, action type, and block type (for place)
- **FR-020**: All clients MUST receive block edits in consistent order

#### Client Rendering

- **FR-021**: Client MUST update rendered voxel mesh when receiving block edit events
- **FR-022**: Client rendering MUST remain responsive during mesh updates (no freezing)
- **FR-023**: Client MUST display debug feedback for successful and rejected actions

### Key Entities

- **BlockPosition**: Integer coordinates (x, y, z) identifying a cell in the voxel grid
- **BlockType**: Type of block to place (for MVP: single default type, e.g., "Stone")
- **BlockAction**: Request containing action type (Remove/Place), target position, and optionally block type
- **BlockEditEvent**: Server broadcast containing position, action type, result, and tick number
- **WorldState**: Complete set of block data representing current arena state

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players see block changes reflected on all clients within 200ms of server confirmation (at 60 tick rate + network latency)
- **SC-002**: Server validates and processes block actions within a single tick (16ms at 60Hz)
- **SC-003**: Client mesh updates complete without frame drops exceeding 50ms per edit
- **SC-004**: Late joiners see correct world state within 2 seconds of connection
- **SC-005**: 100% of invalid edit attempts (out of bounds, out of range, wrong cell state) are rejected by server
- **SC-006**: Two windowed clients can place/remove blocks and see changes replicated in real-time
- **SC-007**: All existing tests pass (cargo test --workspace)
- **SC-008**: Headless mode and load tests continue to function without errors
- **SC-009**: Block edits have no measurable impact on combat/movement latency

## Scope Boundaries

### In Scope

- Block removal via discrete action
- Block placement via discrete action (single default block type)
- Server-side validation (bounds, range, cell state, rate limit, player collision)
- Reliable replication to all clients via events
- Late joiner world sync
- Minimal debug HUD feedback

### Out of Scope

- Inventory system (unlimited blocks)
- Multiple block types / block selection UI
- Crafting
- Physics-based block falling (gravity for sand/gravel)
- Building permissions / claims
- Persistence to disk
- Procedural generation
- Mods API
- CEF / web UI

## Assumptions

- Default block type for placement is a simple solid block (e.g., "Stone")
- Interaction range of 5 blocks is appropriate for gameplay balance
- Rate limit of 4 edits/second prevents spam while allowing reasonable building pace
- Players can place blocks without needing inventory - infinite blocks available
- Block edits do not affect spawn points or protected arena regions (future consideration)
- Mesh update performance is acceptable with naive rebuild approach initially
