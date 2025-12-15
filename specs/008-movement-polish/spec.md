# Feature Specification: Movement Polish

**Feature Branch**: `008-movement-polish`
**Created**: 2025-12-15
**Status**: Draft
**Input**: User description: Feature 008 - Movement Polish - Improve player movement to feel solid, predictable, and competitive

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Reliable Collision (Priority: P1)

As a player, I want collision with voxels to be solid and consistent, so I never clip into blocks or get stuck on edges.

**Why this priority**: Collision is the foundation of all movement. Without solid collision, all other movement features (jumping, stepping, friction) are meaningless. Players cannot play if they fall through the world or get stuck in walls.

**Independent Test**: Can be tested by spawning a player adjacent to walls/floors/ceilings and moving in all directions. Delivers the core promise of a solid world.

**Acceptance Scenarios**:

1. **Given** a player standing next to a wall, **When** the player walks into the wall, **Then** the player stops at the wall surface without penetrating it
2. **Given** a player moving diagonally into a corner, **When** collision occurs, **Then** the player slides smoothly along the wall without jitter
3. **Given** a player standing still against a wall, **When** no input is applied, **Then** the player remains stationary without vibrating or shifting
4. **Given** a client and server simulating the same movement, **When** collision is resolved, **Then** both produce identical final positions

---

### User Story 2 - Jumping (Priority: P1)

As a player, I want a responsive jump with predictable height, so movement feels tight and controllable.

**Why this priority**: Jumping is a core movement mechanic in any FPS/voxel game. Players need to navigate vertical terrain, and jumping must be reliable for competitive play.

**Independent Test**: Can be tested by pressing jump while grounded and observing consistent jump height. Delivers vertical navigation capability.

**Acceptance Scenarios**:

1. **Given** a player standing on the ground, **When** the player presses jump, **Then** the player gains upward velocity and leaves the ground
2. **Given** a player in the air (not grounded), **When** the player presses jump, **Then** nothing happens (jump ignored)
3. **Given** a player holding the jump button continuously, **When** the player lands after a jump, **Then** the player does not automatically jump again (requires button release and re-press)
4. **Given** two clients performing the same jump, **When** simulation completes, **Then** both achieve the same jump height

---

### User Story 3 - Step-Up Movement (Priority: P2)

As a player, I want to smoothly walk up small ledges, so movement feels natural in voxel environments.

**Why this priority**: Step-up prevents frustrating micro-jumps on single-block ledges. Important for flow but not as critical as basic collision and jumping.

**Independent Test**: Can be tested by walking toward a 1-block ledge and observing automatic elevation. Delivers smooth terrain navigation.

**Acceptance Scenarios**:

1. **Given** a player walking toward a step-height obstacle (0.5 blocks or less), **When** the player continues moving forward, **Then** the player automatically steps up onto the ledge
2. **Given** a player facing a wall (obstacle taller than step_height), **When** the player walks into it, **Then** the player does not step up (stops at wall)
3. **Given** a player in the air, **When** the player moves toward a step-height obstacle, **Then** no step-up occurs (requires grounded state)
4. **Given** a step-up that would cause head collision with ceiling, **When** the player attempts to step up, **Then** the step-up fails and player stops

---

### User Story 4 - Friction & Ground Control (Priority: P2)

As a player, I want grounded movement to feel responsive and air movement to feel floatier, so control matches modern FPS expectations.

**Why this priority**: Friction affects game feel significantly. Ground control should be tight, air control should be reduced. Needed for competitive gameplay but secondary to basic locomotion.

**Independent Test**: Can be tested by releasing movement input on ground vs. in air and observing deceleration differences. Delivers proper movement feel.

**Acceptance Scenarios**:

1. **Given** a player moving on the ground, **When** the player releases all movement input, **Then** the player decelerates to a stop within a short distance (ground friction applied)
2. **Given** a player in the air, **When** the player attempts to change direction, **Then** the turning rate is noticeably reduced compared to ground movement
3. **Given** a player standing on a flat surface with no input, **When** time passes, **Then** the player does not slide (no infinite sliding)
4. **Given** the same inputs on client and server, **When** friction is applied, **Then** both produce identical velocity changes

---

### User Story 5 - Stable Hitbox (Priority: P2)

As a player, I want my hitbox to be stable and fair, so hits feel consistent and predictable.

**Why this priority**: Combat relies on hitboxes. If hitboxes jitter or don't match visual positions, combat feels unfair. Required for competitive integrity.

**Independent Test**: Can be tested by performing combat and verifying hit registration matches visual positions. Delivers fair combat.

**Acceptance Scenarios**:

1. **Given** a player at any position, **When** the hitbox is queried, **Then** it returns a fixed capsule shape with consistent dimensions
2. **Given** a player's rendered position, **When** compared to hitbox position, **Then** they match exactly (no offset or desync)
3. **Given** a player performing any movement, **When** the hitbox is observed, **Then** it does not shrink, stretch, or jitter
4. **Given** combat validation on the server, **When** a hit is checked, **Then** the server uses the authoritative hitbox position

---

### User Story 6 - Desync & Prediction Fixes (Priority: P3)

As a developer/player, I want to minimize visible corrections and rubber-banding, so movement feels smooth even with latency.

**Why this priority**: Network smoothing improves perceived quality but is polish rather than core functionality. The game is playable without it, but feels rough.

**Independent Test**: Can be tested by simulating network latency and observing correction smoothness. Delivers polished network experience.

**Acceptance Scenarios**:

1. **Given** client prediction code, **When** compared to server simulation, **Then** they use identical movement logic
2. **Given** a misprediction occurs, **When** the server sends a correction, **Then** the client applies a smooth interpolation rather than a hard snap
3. **Given** a large prediction error (>1 block), **When** correction is applied, **Then** the correction is clamped and eased over time to prevent jarring teleports
4. **Given** the movement tick rate, **When** measured, **Then** it is consistent at 60Hz on both client and server

---

### Edge Cases

- **Standing on block edges**: Player should not fall through or jitter when standing at the edge of a block
- **Stepping while turning**: Step-up should work correctly even when player is simultaneously rotating
- **Jumping against ceilings**: Player should stop at ceiling, not clip through, and fall back down
- **Diagonal movement into corners**: Collision should resolve smoothly without getting stuck
- **High latency (>150ms)**: Corrections should remain smooth, not cause violent rubber-banding
- **Maximum speed tunneling**: Even at max speed, player should not pass through thin walls

## Requirements *(mandatory)*

### Functional Requirements

#### Movement Core
- **FR-001**: Movement simulation MUST run at a fixed tick rate of 60Hz
- **FR-002**: All physics calculations MUST be deterministic (same input produces same output)
- **FR-003**: Server MUST be authoritative over final player position

#### Collision
- **FR-010**: System MUST detect collisions between player capsule and voxel geometry
- **FR-011**: Collision resolution MUST be axis-independent (resolve X, Y, Z separately)
- **FR-012**: System MUST prevent tunneling at maximum player speed

#### Step-Up
- **FR-020**: Step-up height MUST be configurable (default: 0.5 blocks)
- **FR-021**: Step-up MUST only be attempted when horizontal collision is detected
- **FR-022**: Step-up MUST fail if it would cause head collision with geometry above

#### Jumping
- **FR-030**: Jump MUST apply a vertical impulse once per ground contact
- **FR-031**: Jump input MUST be ignored while player is airborne

#### Friction
- **FR-040**: System MUST apply separate ground_friction and air_control coefficients (air_control = 30% of ground control)
- **FR-041**: Ground friction MUST only be applied when player is grounded

#### Hitbox
- **FR-050**: Player hitbox MUST be a capsule with fixed radius and height
- **FR-051**: Hitbox origin MUST be aligned to player feet position

#### Networking
- **FR-060**: Client prediction MUST use identical movement code as server
- **FR-061**: Server corrections MUST include both velocity and position
- **FR-062**: Correction smoothing MUST complete within 100ms

### Key Entities

- **Player Capsule**: The collision shape representing the player (0.4m radius, 1.8m height, feet-aligned origin)
- **Movement State**: Current position, velocity, and grounded status for a player
- **Movement Config**: Physics parameters including speed (6 m/s), gravity (20 m/s²), friction, step_height (0.5 blocks), jump_impulse (tuned for 1.25 block jump height)
- **Correction Data**: Server-to-client position/velocity correction with smoothing metadata

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: No clipping or stuck states occur during a 10-minute stress test with 8 players
- **SC-002**: Prediction error is less than 0.2 blocks in 95% of movement samples
- **SC-003**: Movement feels identical when two players perform the same inputs side-by-side
- **SC-004**: Existing load tests continue to pass without degradation
- **SC-005**: Players can complete an obstacle course (walk, jump, step-up) without getting stuck
- **SC-006**: Jump height is consistent within 1% variance across all clients
- **SC-007**: No visible jitter when standing still against walls

## Clarifications

### Session 2025-12-15
- Q: What should the player capsule dimensions be? → A: Standard: 1.8m height, 0.4m radius
- Q: What should the target jump height be? → A: Standard: 1.25 blocks
- Q: What gravity value should be used? → A: Arcade: 20 m/s²
- Q: What should the air control ratio be compared to ground control? → A: Moderate: 30%
- Q: What should the base player movement speed be? → A: Standard: 6 m/s

## Assumptions

- The existing 60Hz tick rate is sufficient and will not be changed
- Player capsule dimensions: 1.8m height, 0.4m radius (standard FPS proportions)
- The anti-cheat system (feature 007) will validate movement independently; this feature focuses on correct movement, not cheat detection
- Network protocol already supports position/velocity corrections; this feature improves how they are applied

## Out of Scope

- Sprinting, crouching, or sliding mechanics
- Advanced movement techniques (bunny hopping, air strafing)
- Parkour or wall climbing
- Animation blending or visual polish
- New input bindings

## Constraints

- Must work in headless server mode (no rendering dependencies)
- Must not significantly increase server tick time
- Must not break existing combat or block interaction features
