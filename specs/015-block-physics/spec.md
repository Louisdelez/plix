# Feature Specification: Block Physics Light

**Feature Branch**: `015-block-physics`
**Created**: 2025-12-16
**Status**: Draft
**Input**: User description: "Feature 015 - Block Physics Light: minimal voxel physics simulation with optional gravity for blocks and simple liquids, with bounded performance guarantees"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Gravity-Affected Blocks Fall (Priority: P1)

As a sandbox player, I want certain blocks (like sand) to fall when their support is removed, so the world feels more dynamic and interactive.

**Why this priority**: Core mechanic that makes the world feel alive. Without gravity, there's no physics simulation at all - this is the foundation feature.

**Independent Test**: Can be fully tested by placing a sand block mid-air and observing it falls to the ground, or by breaking a support block and watching blocks above collapse.

**Acceptance Scenarios**:

1. **Given** a gravity-affected block (e.g., sand) is placed in mid-air, **When** it is placed, **Then** it falls until it lands on a solid surface below.
2. **Given** a gravity-affected block is resting on a solid block, **When** the support block is removed, **Then** the gravity-affected block falls down.
3. **Given** a column of gravity-affected blocks, **When** the bottom support is removed, **Then** all blocks in the column fall in sequence.
4. **Given** a gravity-affected block falls, **When** it lands on a solid surface, **Then** it stops and becomes a static block at that position.

---

### User Story 2 - Physics Toggle Per World/Mode (Priority: P1)

As a game administrator, I want to enable or disable block physics per world or game mode, so competitive arenas can disable it while sandbox worlds can enjoy dynamic physics.

**Why this priority**: Essential for mode differentiation - competitive modes need predictable static worlds, while sandbox benefits from physics.

**Independent Test**: Can be tested by creating two worlds with different physics settings and verifying blocks behave accordingly in each.

**Acceptance Scenarios**:

1. **Given** physics is disabled for the current world, **When** a gravity-affected block is placed mid-air, **Then** it stays in place.
2. **Given** physics is enabled for the current world, **When** a gravity-affected block is placed mid-air, **Then** it falls as expected.
3. **Given** physics toggle is changed at runtime, **When** the setting is toggled, **Then** future block placements respect the new setting.

---

### User Story 3 - Bounded Performance Under Cascade (Priority: P1)

As a server administrator, I want physics simulation to be performance-bounded, so that large cascades (many falling blocks) don't cause server lag or freeze the game.

**Why this priority**: Without performance bounds, a single player could crash the server by triggering a massive cascade. This is a stability requirement.

**Independent Test**: Can be tested by triggering a large cascade (100+ falling blocks) and verifying tick time stays within acceptable bounds.

**Acceptance Scenarios**:

1. **Given** physics budget is set to N events per tick, **When** more than N events are pending, **Then** only N events are processed this tick and the rest are queued.
2. **Given** a large cascade is triggered, **When** processing occurs, **Then** game tick time remains stable (no spikes blocking gameplay).
3. **Given** events are queued for later processing, **When** subsequent ticks occur, **Then** queued events are processed without loss.

---

### User Story 4 - Cross-Chunk Physics (Priority: P2)

As a player, I want blocks to fall correctly even when they cross chunk boundaries, so the world behaves consistently regardless of chunk layout.

**Why this priority**: Important for correctness but secondary to core mechanics. A block that stops at chunk boundaries would be a visible bug.

**Independent Test**: Can be tested by placing a gravity block at a chunk boundary and verifying it falls through to the next chunk.

**Acceptance Scenarios**:

1. **Given** a gravity-affected block at chunk boundary, **When** it falls, **Then** it continues falling into the adjacent chunk below.
2. **Given** a support block at chunk boundary is removed, **When** physics updates, **Then** blocks in adjacent chunk are notified and updated.

---

### User Story 5 - Simple Liquid Spreading (Priority: P3)

As a sandbox player, I want simple liquids (water) that spread horizontally when placed, so I can create ponds and water features.

**Why this priority**: Optional enhancement that adds variety. Core physics works without liquids.

**Independent Test**: Can be tested by placing a water source block and observing it spreads to adjacent empty cells.

**Acceptance Scenarios**:

1. **Given** a liquid source block is placed, **When** physics updates, **Then** liquid spreads to adjacent air blocks (horizontally and downward).
2. **Given** liquid is spreading, **When** it reaches maximum spread distance, **Then** spreading stops.
3. **Given** liquid spreading is in progress, **When** spread budget per tick is reached, **Then** remaining spread is deferred to next tick.
4. **Given** liquids are disabled in config, **When** a liquid block is placed, **Then** it behaves as a static block (no spreading).

---

### User Story 6 - Physics Observability Metrics (Priority: P3)

As a server administrator, I want to monitor physics simulation load, so I can tune budgets and identify performance issues.

**Why this priority**: Useful for debugging and tuning but not required for basic functionality.

**Independent Test**: Can be tested by triggering physics events and checking exposed metrics show correct counts.

**Acceptance Scenarios**:

1. **Given** physics simulation is running, **When** metrics are queried, **Then** events processed per tick is reported.
2. **Given** events are queued, **When** metrics are queried, **Then** queue depth is reported.
3. **Given** blocks have fallen, **When** metrics are queried, **Then** total blocks fallen is reported.

---

### Edge Cases

- What happens when a gravity block falls into an unloaded chunk? Block stops at chunk boundary and queues for processing when chunk loads.
- How does system handle circular dependencies? Not applicable - gravity only goes down, no cycles possible.
- What happens when physics budget is set to 0? Physics effectively disabled - no events processed.
- What happens when a player places a block where a block is falling? Server authority - falling block lands first, placed block replaces it or is rejected based on timing.
- How does system handle liquid meeting gravity blocks? Liquid displaces air, gravity blocks fall through liquid.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support configuring block types as "gravity-affected" or "static"
- **FR-002**: System MUST make gravity-affected blocks fall vertically when unsupported (air below)
- **FR-003**: System MUST provide a per-world configuration to enable/disable block physics globally
- **FR-004**: System MUST limit physics events processed per tick to a configurable budget
- **FR-005**: System MUST queue unprocessed physics events without loss for future ticks
- **FR-006**: System MUST propagate physics updates across chunk boundaries seamlessly
- **FR-007**: System MUST trigger physics checks on neighboring blocks when a block is placed or removed
- **FR-008**: System MUST integrate with existing chunk dirty tracking for visual updates
- **FR-009**: Server MUST be authoritative for all physics state - clients reflect server state
- **FR-010**: Physics simulation MUST be deterministic given the same initial state and event order
- **FR-011**: System MUST expose metrics for events processed, queue depth, and blocks updated
- **FR-012**: System SHOULD support simple liquid blocks that spread horizontally and downward (configurable, can be disabled)
- **FR-013**: Liquid spreading MUST respect a maximum spread distance and per-tick budget
- **FR-014**: System MUST NOT produce verbose per-block logs in production mode

### Key Entities

- **PhysicsConfig**: World-level physics configuration (enabled, gravity budget, liquid budget, max spread distance)
- **BlockPhysicsType**: Per-block-type physics behavior (Static, GravityAffected, Liquid)
- **PhysicsEvent**: A pending physics update (block position, event type: Fall, LiquidSpread)
- **PhysicsQueue**: FIFO queue of pending physics events with budget enforcement
- **PhysicsMetrics**: Counters for events processed, queue depth, blocks fallen, liquid cells updated

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Gravity-affected blocks fall at a consistent rate (1 block per tick minimum) until landing
- **SC-002**: Physics simulation processes up to the configured budget per tick without exceeding tick time by more than 10%
- **SC-003**: Cascades of 1000+ blocks complete processing within 60 seconds of real time
- **SC-004**: Queue depth never drops events - all queued events are eventually processed
- **SC-005**: Cross-chunk physics works seamlessly - blocks fall through chunk boundaries without visual glitches
- **SC-006**: Liquid spreading completes within bounded distance (configurable, default 7 blocks from source)
- **SC-007**: Physics behavior is identical on server and all clients (deterministic)
- **SC-008**: Metrics are accessible and accurately reflect simulation state

## Assumptions

- Gravity only operates vertically downward (no lateral sliding in v1)
- Block physics is discrete (blocks move 1 full block position per step, no partial positions)
- Liquids use simple flood-fill spreading, not realistic fluid dynamics
- Physics events are processed in FIFO order for determinism
- Default budget: 100 gravity events + 50 liquid events per tick (configurable)
- Unloaded chunks do not process physics - events at boundaries wait for chunk load
- No entity physics in this feature - only block-to-block interactions
