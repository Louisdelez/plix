# Feature Specification: Procedural Generation v1

**Feature Branch**: `013-procedural-generation`
**Created**: 2025-12-16
**Status**: Draft
**Input**: User description: "Seed-based deterministic world generation with heightmap terrain and basic biome system"

## Clarifications

### Session 2025-12-16

- Q: Which noise algorithm should be used for terrain generation? → A: Use `noise-rs` crate with Perlin/Simplex noise
- Q: How should negative Y chunks (below bedrock) be handled? → A: Generate as AIR (void below bedrock)
- Q: How should biome transitions be handled? → A: Per-block biome sampling (each block samples biome noise at its position)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Deterministic World Generation (Priority: P1)

As a game developer, I need world generation to be deterministic so that the same seed always produces the same terrain, enabling reproducible testing and multiplayer synchronization.

**Why this priority**: Determinism is foundational - without it, clients and server would generate different worlds, breaking multiplayer. All other features depend on this.

**Independent Test**: Can be tested by generating the same chunk with the same seed twice and verifying byte-for-byte identical output.

**Acceptance Scenarios**:

1. **Given** seed `12345` and chunk coordinate `(0, 0, 0)`, **When** generating the chunk twice, **Then** both outputs are identical block-for-block
2. **Given** seed `12345` and chunk coordinate `(0, 0, 0)`, **When** generating on different machines with same Rust version, **Then** outputs are identical
3. **Given** seed `12345`, **When** generating chunks in any order, **Then** each chunk's content depends only on seed and coordinate, not generation order

---

### User Story 2 - Heightmap-Based Terrain (Priority: P1)

As a player, I want to explore terrain with natural-looking hills and valleys created from heightmap noise, providing interesting gameplay geography.

**Why this priority**: Heightmap is the core terrain algorithm that all other features (biomes, layers) build upon. Without it, there's no terrain.

**Independent Test**: Can be tested by generating a chunk and verifying blocks follow heightmap pattern (solid below height, air above).

**Acceptance Scenarios**:

1. **Given** a flat seed (height=64 everywhere), **When** generating chunk at y=0, **Then** blocks y<64 are solid, y>=64 are air
2. **Given** a hilly seed, **When** generating adjacent chunks, **Then** height transitions smoothly at chunk boundaries (no visible seams)
3. **Given** any seed, **When** generating terrain, **Then** height varies naturally between min (32) and max (96) based on noise

---

### User Story 3 - Basic Biome System (Priority: P2)

As a player, I want to see different biomes (plains, mountains, desert) as I explore, providing visual variety and distinct gameplay areas.

**Why this priority**: Biomes add visual interest but aren't required for basic gameplay. The game is playable with a single biome.

**Independent Test**: Can be tested by generating chunks at different world positions and verifying biome selection based on noise.

**Acceptance Scenarios**:

1. **Given** seed and position with low biome noise, **When** generating, **Then** plains biome is selected (grass surface, dirt subsurface)
2. **Given** seed and position with high biome noise, **When** generating, **Then** mountains biome is selected (stone surface, increased height amplitude)
3. **Given** seed and position with mid biome noise + high temperature, **When** generating, **Then** desert biome is selected (sand surface, sandstone subsurface)

---

### User Story 4 - Layer-Based Block Placement (Priority: P2)

As a player, I want terrain to have realistic layers (surface grass, subsurface dirt, deep stone, bedrock bottom) for immersive mining gameplay.

**Why this priority**: Layers enhance realism but basic gameplay works with uniform block types. Builds on heightmap.

**Independent Test**: Can be tested by examining a generated column and verifying correct block types at each depth.

**Acceptance Scenarios**:

1. **Given** plains biome at surface height 64, **When** examining block at y=64, **Then** block is GRASS
2. **Given** plains biome at surface height 64, **When** examining blocks at y=61-63, **Then** blocks are DIRT (3 layers)
3. **Given** any biome, **When** examining blocks at y=0, **Then** blocks are BEDROCK
4. **Given** any biome, **When** examining blocks below surface but above bedrock, **Then** blocks are STONE

---

### User Story 5 - Per-Chunk Independent Generation (Priority: P1)

As a server operator, I need chunks to generate independently without neighbor data, enabling efficient on-demand generation and chunk streaming.

**Why this priority**: Independence is required for multiplayer chunk streaming. Without it, generating one chunk would require generating neighbors first, creating cascading dependencies.

**Independent Test**: Can be tested by generating a chunk in isolation (no neighbors loaded) and verifying valid output.

**Acceptance Scenarios**:

1. **Given** empty world, **When** requesting chunk (5, 0, 3) with no neighbors loaded, **Then** chunk generates successfully with valid terrain
2. **Given** chunk (0, 0, 0) already generated, **When** generating chunk (1, 0, 0), **Then** generation does not read or modify chunk (0, 0, 0)
3. **Given** parallel generation requests for chunks (0,0,0) and (1,0,0), **When** processing simultaneously, **Then** both complete without race conditions

---

### User Story 6 - Generation Performance (Priority: P3)

As a player, I want chunk generation to be fast enough that I don't notice loading delays when exploring new areas.

**Why this priority**: Performance is important for user experience but the game is functional with slower generation. Can be optimized later.

**Independent Test**: Can be tested by timing chunk generation and verifying it meets target threshold.

**Acceptance Scenarios**:

1. **Given** any seed and chunk coordinate, **When** generating a single chunk, **Then** generation completes in under 10ms
2. **Given** view distance of 8 chunks, **When** generating initial spawn area (8x8x8=512 chunks), **Then** total generation completes in under 5 seconds
3. **Given** player moving at normal speed, **When** generating new chunks ahead of player, **Then** generation keeps pace without visible pop-in

---

### Edge Cases

- What happens when seed is 0 or u64::MAX? Generation should still produce valid terrain
- What happens at world coordinate extremes (i32::MIN, i32::MAX)? Should produce valid terrain without overflow
- What happens when chunk spans y=0 (bedrock layer)? Bedrock should appear regardless of heightmap
- How does system handle negative Y chunks (below bedrock)? Generate as AIR (void below bedrock)
- What happens at biome boundaries? Per-block biome sampling ensures smooth natural transitions

## Requirements *(mandatory)*

### Functional Requirements

#### Generation Core
- **FR-001**: System MUST accept a u64 seed value for world generation
- **FR-002**: System MUST produce identical chunk output for identical (seed, chunk_coord) inputs
- **FR-003**: System MUST generate chunks independently without reading neighbor chunk data
- **FR-004**: System MUST support generating chunks in any order with identical results

#### Heightmap Terrain
- **FR-005**: System MUST use `noise-rs` crate with Perlin or Simplex noise to determine surface height for each (x, z) column
- **FR-006**: System MUST produce heights in range [32, 96] blocks (64 blocks of vertical variation)
- **FR-007**: System MUST ensure smooth height transitions at chunk boundaries (continuous noise)
- **FR-008**: System MUST fill blocks below height with solid material, above with air

#### Biome System
- **FR-009**: System MUST support at least 3 biomes: plains, mountains, desert
- **FR-010**: System MUST select biome using per-block sampling (each block's x,z position samples biome noise independently)
- **FR-011**: System MUST vary heightmap amplitude per biome (mountains=high, plains=low, desert=medium)
- **FR-012**: System MUST assign biome-specific surface and subsurface block types

#### Layer System
- **FR-013**: System MUST place bedrock at y=0 regardless of biome
- **FR-013a**: System MUST generate chunks with y<0 as entirely AIR (void below bedrock)
- **FR-014**: System MUST place stone as default fill material below surface
- **FR-015**: System MUST place biome-specific surface block at terrain surface (1 layer)
- **FR-016**: System MUST place biome-specific subsurface block below surface (3 layers)

#### Block Types by Biome
- **FR-017**: Plains biome: surface=GRASS, subsurface=DIRT
- **FR-018**: Mountains biome: surface=STONE, subsurface=STONE
- **FR-019**: Desert biome: surface=SAND, subsurface=SANDSTONE

#### Block Type Extensions
- **FR-020**: System MUST add GRASS block type (id 4) for plains surface
- **FR-021**: System MUST add DIRT block type (id 5) for plains subsurface
- **FR-022**: System MUST add SAND block type (id 6) for desert surface
- **FR-023**: System MUST add SANDSTONE block type (id 7) for desert subsurface
- **FR-024**: System MUST add BEDROCK block type (id 8) for world floor

#### Integration
- **FR-025**: System MUST integrate with existing ChunkedWorld chunk storage
- **FR-026**: System MUST integrate with existing ChunkManager dirty tracking
- **FR-027**: System MUST be callable from both server (authoritative) and client (prediction)

### Key Entities

- **WorldGenerator**: Core generator taking seed, producing chunks on demand. Stateless except for seed.
- **BiomeConfig**: Per-biome configuration (height amplitude, surface block, subsurface block, subsurface depth)
- **NoiseSource**: Seeded noise function provider using `noise-rs` crate (heightmap noise, biome noise, temperature noise)
- **ChunkCoord**: Existing chunk coordinate type (i32, i32, i32) from plix-common
- **BlockType**: Extended block type enum from plix-common. Existing: (AIR=0, STONE=1, BRICK=2, METAL=3). New: (GRASS=4, DIRT=5, SAND=6, SANDSTONE=7, BEDROCK=8)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Same seed + chunk coordinate produces bit-identical chunk data across 1000 test runs
- **SC-002**: Single chunk generation completes in under 10ms (average over 100 chunks)
- **SC-003**: Generated terrain has no visible seams at chunk boundaries when rendered
- **SC-004**: All 3 biomes appear within 1000 chunks of spawn given varied seeds
- **SC-005**: Block layers follow specification: bedrock at y=0, stone fill, subsurface layers, surface layer
- **SC-006**: Generation works correctly for negative chunk coordinates
- **SC-007**: Chunk generation is thread-safe (can generate multiple chunks in parallel)
- **SC-008**: New block types (GRASS, DIRT, SAND, SANDSTONE, BEDROCK) are added to existing BlockType enum (currently has AIR, STONE, BRICK, METAL)
