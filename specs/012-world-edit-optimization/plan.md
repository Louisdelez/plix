# Implementation Plan: World Edit Optimization

**Branch**: `012-world-edit-optimization` | **Date**: 2025-12-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/012-world-edit-optimization/spec.md`

## Summary

Optimize block modifications for build fights by extending the existing Feature 011 ChunkManager with retry logic, observability metrics, and automatic boundary-based dirty marking. The core dirty queue deduplication and mesh budget enforcement already exist; this feature adds failure handling, metrics exposure, and integration with block edit events.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-common (chunk types), plix-client (ChunkManager, meshing), tracing (metrics)
**Storage**: N/A (in-memory state only)
**Testing**: cargo test (unit + integration tests)
**Target Platform**: Linux/Windows/macOS desktop client
**Project Type**: Single workspace (existing Rust workspace)
**Performance Goals**: <5ms per chunk rebuild, 60 FPS baseline, 30 FPS minimum under heavy edit load
**Constraints**: <100ms worst-case frame time, no unbounded queues, no blocking operations
**Scale/Scope**: 8-chunk view radius (~2000 chunks loaded), 50 edits/second peak

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security | PASS | Client-side rendering optimization, no trust implications |
| II. Performance | PASS | Bounded frame times via mesh budget, no stop-the-world |
| III. Architecture | PASS | Extends existing ChunkManager, follows primitive provision |
| IV. Modding | N/A | Internal engine optimization, not mod-facing |
| V. Code Quality | PASS | Explicit error handling, mandatory tests per spec |
| VI. Technical Standards | PASS | Stable Rust, clippy/fmt compliance, deterministic APIs |
| VII. Player Experience | PASS | Maintains responsive UI during build fights |
| VIII. Open Source | PASS | No proprietary dependencies |
| IX. Scoping | PASS | Minimal scope - extends existing infrastructure |
| X. Long-Term | PASS | Clean extension, no technical debt |

**Gate Result**: PASS - No violations detected

## Project Structure

### Documentation (this feature)

```text
specs/012-world-edit-optimization/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── plix-common/src/
│   └── chunk.rs                    # [EXTEND] Add full neighbor detection (edges/corners)
│
├── plix-client/src/
│   ├── chunk_manager.rs            # [EXTEND] Add retry tracking, metrics, loaded-check on mark
│   ├── chunk_mesher.rs             # [READ-ONLY] Existing meshing pipeline
│   ├── render/engine.rs            # [EXTEND] Integrate metrics counters
│   ├── world.rs                    # [EXTEND] Add block edit -> dirty marking integration
│   └── lib.rs                      # [READ-ONLY] Module exports

tests/
└── (inline in modules)             # Unit tests in each modified module
```

**Structure Decision**: Extend existing crate structure. No new crates needed - ChunkManager already contains dirty queue infrastructure. Feature 012 adds:
1. Enhanced `boundary_neighbors()` to handle edges and corners (currently only handles faces)
2. Retry counter per chunk in ChunkManager
3. Metrics counters struct in ChunkManager
4. Block edit integration in ClientWorld

## Complexity Tracking

> No violations - table empty as expected.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| (none) | - | - |

## Design Decisions

### D1: Extend ChunkManager vs New Component

**Decision**: Extend existing ChunkManager

**Rationale**: ChunkManager already has dirty_queue + dirty_set + mesh_budget. Adding retry tracking and metrics keeps all dirty-related state in one place. Creating a separate DirtyTracker would split related state and add indirection.

**Alternatives Rejected**:
- Separate DirtyTracker struct: Would duplicate loaded-chunk checks, add coordination overhead

### D2: Retry Tracking Approach

**Decision**: Per-chunk retry counter in ChunkManager (HashMap<ChunkCoord, u8>)

**Rationale**: Simple, bounded memory (only tracks failing chunks), O(1) lookup. Counter increments on failure, resets when chunk succeeds or is skipped.

**Alternatives Rejected**:
- Separate retry queue: More complex, harder to coordinate with dirty queue
- Exponential backoff timer: Overkill for GPU exhaustion (retry immediately is fine)

### D3: Metrics Exposure

**Decision**: Counter struct with pub fields in ChunkManager, updated per frame

**Rationale**: Aligns with Feature 010 tracing-based metrics. Simple counters without histogram overhead. Debug tools can read counters directly.

**Alternatives Rejected**:
- Prometheus-style metrics: Overkill for game client
- Per-chunk detailed traces: Too much overhead for real-time rendering

### D4: Boundary Neighbor Detection

**Decision**: Extend existing `boundary_neighbors()` to enumerate all affected neighbors including edges and corners

**Rationale**: Current implementation only handles face neighbors (1 neighbor per boundary axis). Corners need up to 7 neighbors. Spec requires correct handling for all 26 configurations.

**Implementation**: Use bitmask approach - for each boundary axis, generate combinations of neighbor offsets.

### D5: Block Edit Integration Point

**Decision**: ClientWorld.set_block() calls ChunkManager.mark_dirty_with_neighbors()

**Rationale**: Single integration point. ClientWorld already manages block state and has access to ChunkManager. Adding a new method that computes boundary neighbors and marks all affected chunks.

**Alternatives Rejected**:
- Event system: Adds indirection, harder to trace
- Render loop polling: Would miss intermediate states

## Implementation Phases

### Phase 1: Boundary Detection Enhancement (FR-010 to FR-014)

1. Extend `boundary_neighbors()` in `plix-common/src/chunk.rs`
2. Add comprehensive unit tests for all 26 neighbor configurations
3. Ensure loaded-chunk filtering happens at mark time, not detection time

### Phase 2: ChunkManager Extensions (FR-001 to FR-005, FR-030 to FR-035)

1. Add `retry_counts: HashMap<ChunkCoord, u8>` to ChunkManager
2. Add `skipped_chunks: HashSet<ChunkCoord>` for chunks that exceeded retries
3. Modify `pop_dirty_batch()` to track rebuild results
4. Add `report_rebuild_result(coord, success)` method
5. Add configurable `max_retries: u8` to ChunkManagerConfig (default 3)

### Phase 3: Observability (FR-036 to FR-038)

1. Add `MeshMetrics` struct to ChunkManager:
   ```rust
   pub struct MeshMetrics {
       pub rebuilds_this_frame: u32,
       pub dirty_queue_depth: u32,
       pub skipped_chunks: u32,
       pub successful_rebuilds: u32,
       pub failed_rebuilds: u32,
   }
   ```
2. Update metrics at end of each `update()` call
3. Add `metrics()` accessor method

### Phase 4: Block Edit Integration (FR-020 to FR-024, FR-040 to FR-044)

1. Add `mark_dirty_with_neighbors(pos: BlockPos)` to ChunkManager
   - Computes chunk coord from block pos
   - Checks if chunk is loaded (ignore if not)
   - Computes boundary neighbors
   - Marks chunk and all loaded neighbors dirty
2. Integrate into ClientWorld.set_block()
3. Integrate into server-sent block update handling

### Phase 5: Testing & Validation (SC-001 to SC-008)

1. Unit tests for boundary detection (26 configurations)
2. Unit tests for retry logic (success, failure, skip)
3. Unit tests for metrics accuracy
4. Integration test for rapid edit simulation
5. Performance validation test
