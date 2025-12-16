# Feature Specification: World Edit Optimization

**Feature Branch**: `012-world-edit-optimization`
**Created**: 2025-12-16
**Status**: Draft
**Input**: User description: "Optimize block modifications for build fights - localized chunk updates, boundary handling, dirty chunk system, batched updates, performance stability"

## Clarifications

### Session 2025-12-16

- Q: What happens when mesh rebuild fails repeatedly (GPU exhaustion)? → A: Retry up to 3 times, then skip chunk
- Q: What level of mesh rebuild instrumentation is needed? → A: Counter metrics: rebuilds/frame, queue depth, skipped chunks
- Q: What happens when editing a block in an unloaded chunk? → A: Ignore - block edit has no effect on unloaded chunk mesh

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Localized Chunk Updates (Priority: P1)

As a player placing or removing blocks during a build fight, I want only the affected chunk(s) to be rebuilt so that my edits feel responsive and don't cause frame drops.

**Why this priority**: Core functionality - without localized updates, every block edit would trigger full world rebuilds, making build fights unplayable due to constant stuttering.

**Independent Test**: Can be fully tested by placing a block and verifying via metrics/logs that only the containing chunk's mesh is rebuilt. Frame time should remain stable (<16ms).

**Acceptance Scenarios**:

1. **Given** a chunk mesh exists, **When** a player places a block in the chunk's interior (not at boundary), **Then** only that single chunk's mesh is rebuilt within the mesh budget
2. **Given** a player edits a block, **When** the edit completes, **Then** the frame time spike is less than 5ms for a single chunk rebuild
3. **Given** 50 loaded chunks, **When** editing a block in one chunk, **Then** the other 49 chunk meshes remain unchanged (verifiable via debug counters)

---

### User Story 2 - Boundary Block Handling (Priority: P1)

As a player building at chunk boundaries, I want adjacent chunks to update correctly so that block faces render properly without visual gaps or z-fighting.

**Why this priority**: Boundary handling is critical for visual correctness - incorrect handling causes obvious visual artifacts that break immersion and confuse players about block placement.

**Independent Test**: Can be tested by placing and removing blocks at every chunk boundary face (6 faces per boundary) and verifying no visual artifacts appear.

**Acceptance Scenarios**:

1. **Given** a block is placed at position (15, y, z) in a chunk, **When** the adjacent chunk at (+1, 0, 0) exists, **Then** both chunks are marked dirty and rebuilt
2. **Given** a block at a corner touches 3 neighboring chunks, **When** the block is removed, **Then** all 4 affected chunks (origin + 3 neighbors) are marked dirty
3. **Given** a boundary block edit, **When** meshes rebuild, **Then** face visibility is correct: adjacent solid blocks share no visible face between them
4. **Given** an edge case at chunk boundary, **When** placing a block at (0, 0, 0) in chunk (1, 0, 0), **Then** chunk (0, 0, 0) neighbor is correctly identified and marked dirty

---

### User Story 3 - Dirty Queue Deduplication (Priority: P1)

As a player rapidly placing blocks in the same area, I want duplicate rebuild requests to be collapsed so that performance remains stable during intense building.

**Why this priority**: Without deduplication, rapid edits during build fights would queue hundreds of redundant rebuilds, causing cascading frame drops and potential game freezes.

**Independent Test**: Can be tested by marking the same chunk dirty 100 times in one frame and verifying only 1 rebuild occurs (via debug counter).

**Acceptance Scenarios**:

1. **Given** a chunk is already in the dirty queue, **When** marked dirty again, **Then** the queue length does not increase
2. **Given** 10 rapid block edits in the same chunk within one frame, **When** the dirty queue processes, **Then** only 1 mesh rebuild occurs for that chunk
3. **Given** rapid edits across 5 different chunks, **When** each is marked dirty twice, **Then** the dirty queue contains exactly 5 entries

---

### User Story 4 - Mesh Budget Enforcement (Priority: P2)

As a developer, I want a configurable per-frame mesh rebuild limit so that even during heavy block spam, frame rate remains playable.

**Why this priority**: Budget enforcement prevents worst-case scenarios from tanking performance. While P1 features reduce work, this provides the safety net guaranteeing bounded frame times.

**Independent Test**: Can be tested by queuing 100 dirty chunks and verifying only N (budget) are rebuilt per frame until queue drains.

**Acceptance Scenarios**:

1. **Given** mesh budget is set to 2 chunks/frame, **When** 10 chunks are dirty, **Then** only 2 rebuild this frame, 2 next frame, etc.
2. **Given** mesh budget is configurable, **When** budget is set to 4, **Then** up to 4 chunks rebuild per frame
3. **Given** dirty queue has items, **When** processing exceeds budget, **Then** remaining items stay queued for subsequent frames without loss
4. **Given** budget is 2 and queue has 3 items, **When** frame completes, **Then** dirty queue has 1 remaining item

---

### User Story 5 - Edit-Induced Neighbor Detection (Priority: P2)

As the system processing block edits, I want automatic detection of which neighbors are affected by a boundary edit so that developers don't manually calculate neighbor chunks.

**Why this priority**: Correct neighbor detection is error-prone if done manually. Automating it prevents bugs where adjacent chunks fail to update, causing visual artifacts.

**Independent Test**: Can be tested by calling the boundary detection function with various block positions and verifying correct neighbor chunk coordinates are returned.

**Acceptance Scenarios**:

1. **Given** a block at local position (0, 5, 5), **When** neighbor detection runs, **Then** the chunk at (-1, 0, 0) relative offset is returned as affected
2. **Given** a block at local position (8, 8, 8) (interior), **When** neighbor detection runs, **Then** no neighbor chunks are returned (empty set)
3. **Given** a block at corner (0, 0, 0), **When** neighbor detection runs, **Then** 7 neighbor chunks are returned (all adjacent to that corner)
4. **Given** a block at edge (0, 0, 8), **When** neighbor detection runs, **Then** 3 neighbor chunks are returned (the 3 sharing that edge)

---

### User Story 6 - Performance Stability Under Load (Priority: P2)

As a player in an intense build fight with multiple players editing blocks rapidly, I want frame rate to remain above 30 FPS so that gameplay remains responsive.

**Why this priority**: The ultimate validation - all optimizations must combine to deliver stable performance. Without this, the feature fails its core purpose.

**Independent Test**: Can be tested by simulating rapid block edits (50 edits/second across random chunks) and monitoring frame times.

**Acceptance Scenarios**:

1. **Given** steady state with no edits, **When** baseline is measured, **Then** frame time is below 16ms (60 FPS capable)
2. **Given** 10 block edits per second in different chunks, **When** performance is measured, **Then** 99th percentile frame time is below 33ms
3. **Given** 50 block edits per second concentrated in 5 chunks, **When** deduplication and budget work together, **Then** frame time remains below 50ms (20 FPS minimum)
4. **Given** a worst-case burst of 100 edits in one frame, **When** processed with budget=2, **Then** no single frame exceeds 100ms

---

### Edge Cases

- What happens when editing a block in an unloaded chunk? The edit is ignored (no mesh effect); when chunk eventually loads, it will mesh from current world state
- How does the system handle editing blocks at world boundaries (no neighbor exists)? Neighbor detection returns only existing chunks, no crash
- What happens if mesh rebuild fails (GPU resource exhaustion)? Error is logged, chunk retries up to 3 times then is skipped; game continues without that mesh until next edit triggers re-queue
- What happens with contradictory edits (place then immediately remove)? Each edit marks dirty independently, final state renders correctly
- How does system behave when dirty queue grows very large (1000+ chunks)? Budget ensures bounded work, queue drains over time without freeze

## Requirements *(mandatory)*

### Functional Requirements

#### Dirty Chunk Management

- **FR-001**: System MUST maintain a dirty queue tracking chunks needing mesh rebuild
- **FR-002**: Dirty queue MUST deduplicate entries (same chunk added twice = single entry)
- **FR-003**: System MUST provide O(1) lookup to check if a chunk is already dirty
- **FR-004**: Dirty queue MUST be FIFO ordered (first dirty = first rebuilt)
- **FR-005**: System MUST remove chunks from dirty tracking when rebuilt

#### Boundary Detection

- **FR-010**: System MUST detect when a block edit is at a chunk boundary (local coord 0 or 15)
- **FR-011**: System MUST identify all neighboring chunks affected by boundary edits
- **FR-012**: Boundary detection MUST handle corner cases (block at chunk corner affects up to 7 neighbors)
- **FR-013**: Boundary detection MUST handle edge cases (block at chunk edge affects up to 3 neighbors)
- **FR-014**: System MUST NOT return non-existent chunks as neighbors (graceful handling at world edges)

#### Localized Updates

- **FR-020**: Block edit MUST mark only the containing chunk as dirty (not the entire world)
- **FR-021**: Block edit at boundary MUST mark the containing chunk AND affected neighbors as dirty
- **FR-022**: Chunk marking MUST complete in O(1) time per chunk marked
- **FR-023**: System MUST provide batch marking API for multiple chunks
- **FR-024**: System MUST ignore dirty marking for chunks that are not currently loaded

#### Mesh Budget

- **FR-030**: System MUST enforce a configurable per-frame mesh rebuild limit
- **FR-031**: Default mesh budget MUST be 2 chunks per frame
- **FR-032**: Mesh budget MUST be adjustable at runtime (for tuning/debugging)
- **FR-033**: Budget enforcement MUST preserve dirty queue state (no data loss)
- **FR-034**: System MUST process dirty queue until empty over multiple frames
- **FR-035**: System MUST retry failed mesh rebuilds up to 3 times before skipping the chunk

#### Observability

- **FR-036**: System MUST expose counter metric for mesh rebuilds per frame
- **FR-037**: System MUST expose counter metric for current dirty queue depth
- **FR-038**: System MUST expose counter metric for skipped chunks (failed after retries)

#### Integration

- **FR-040**: Block placement events MUST trigger dirty marking automatically
- **FR-041**: Block removal events MUST trigger dirty marking automatically
- **FR-042**: Server-sent block updates MUST trigger dirty marking
- **FR-043**: System MUST integrate with existing ChunkManager from feature 011
- **FR-044**: System MUST work with existing chunk meshing pipeline

### Key Entities

- **DirtyQueue**: FIFO queue of ChunkCoords needing rebuild, with HashSet for O(1) deduplication. Part of ChunkManager.
- **ChunkCoord**: Existing type from feature 011 - identifies chunk position in chunk space.
- **LocalPos**: Position within a chunk (0-15 per axis). Used for boundary detection.
- **BoundaryMask**: Bitmask or set indicating which faces of a block position are at chunk boundaries (used for neighbor calculation).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Single interior block edit triggers exactly 1 chunk mesh rebuild (verified via counter)
- **SC-002**: Boundary block edit triggers correct number of chunk rebuilds (1-8 depending on position)
- **SC-003**: 100 rapid edits to same chunk in one frame results in 1 queued rebuild (deduplication works)
- **SC-004**: With 50 dirty chunks and budget=2, queue drains in exactly 25 frames
- **SC-005**: Under 10 edits/second load, 99th percentile frame time remains below 33ms
- **SC-006**: All existing tests from feature 011 continue to pass (non-regression)
- **SC-007**: Boundary detection correctly identifies all 26 possible neighbor configurations (unit tested)
- **SC-008**: No frame exceeds 100ms even under 100 edits/frame burst stress test

## Assumptions

- Feature 011 (Chunked World) is implemented and provides ChunkManager, ChunkCoord, and meshing infrastructure
- Chunk size remains 16x16x16 blocks (boundary positions are 0 and 15)
- Mesh rebuild time for a single chunk is bounded (~2-5ms typical based on feature 011)
- Block edit events are already generated by feature 004 (Block Interaction)
- Server-authoritative block updates use existing protocol from feature 002
- This feature focuses on client-side optimization; server processing is out of scope
