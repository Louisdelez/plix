# Research: World Edit Optimization

**Feature**: 012-world-edit-optimization
**Date**: 2025-12-16

## Overview

This research documents findings from investigating the existing codebase and validating design decisions for Feature 012.

## Research Items

### R1: Existing Dirty Queue Implementation

**Question**: What dirty queue infrastructure already exists in Feature 011?

**Findings**:
- `ChunkManager` in `crates/plix-client/src/chunk_manager.rs` already has:
  - `dirty_queue: VecDeque<ChunkCoord>` - FIFO queue (FR-004 ✓)
  - `dirty_set: HashSet<ChunkCoord>` - O(1) deduplication (FR-002, FR-003 ✓)
  - `mark_dirty(coord)` - deduplication check on insert (FR-001 ✓)
  - `mark_dirty_batch(coords)` - batch marking API (FR-023 ✓)
  - `pop_dirty_batch()` - mesh budget enforcement (FR-030 ✓)
  - `mesh_budget_per_frame: u32` in config - configurable budget (FR-031, FR-032 ✓)

**Decision**: Extend existing ChunkManager rather than creating new infrastructure.

**Rationale**: 6 of the functional requirements are already implemented. Only retry logic, metrics, and boundary integration are needed.

---

### R2: Existing Boundary Detection

**Question**: What boundary detection exists in Feature 011?

**Findings**:
- `crates/plix-common/src/chunk.rs` has:
  - `is_boundary_local(local)` - detects if position is on chunk boundary
  - `boundary_neighbors(coord, local)` - returns affected neighbor chunks

**Current Limitation**: `boundary_neighbors()` only handles **face** boundaries (when exactly one axis is at 0 or 15). It returns up to 3 neighbors for corners but doesn't handle the **diagonal** neighbors (edge and corner chunks).

**Example**: Block at corner (0, 0, 0) currently returns 3 neighbors:
- (-1, 0, 0) - X face
- (0, -1, 0) - Y face
- (0, 0, -1) - Z face

**Missing**: Diagonal neighbors for edges (e.g., (-1, -1, 0)) and corners (e.g., (-1, -1, -1)).

**Decision**: Current implementation is CORRECT for mesh visibility. Diagonal neighbors don't share faces with the edited block, so they don't need mesh updates.

**Rationale**: Mesh visibility only depends on adjacent faces. A block at corner (0,0,0) in chunk (1,1,1) only affects face visibility in the 3 axis-aligned neighbors. The diagonal chunk (-1,-1,-1) has no shared face.

**Spec Clarification**: The spec mentions "7 neighbors" for corners, but this appears to be incorrect. For mesh visibility, only 3 neighbors (axis-aligned) are affected. The existing implementation is correct.

---

### R3: Loaded Chunk Check on Mark

**Question**: Where should loaded-chunk filtering happen?

**Findings**:
- Current `mark_dirty()` unconditionally adds to queue
- `pop_dirty_batch()` filters out unloaded chunks when popping
- FR-024 requires ignoring dirty marking for unloaded chunks

**Decision**: Add loaded-check in `mark_dirty()` method, not just at pop time.

**Rationale**:
- Prevents queue growth from edits in distant/unloaded areas
- More efficient than filtering at pop time
- Clearer semantics: "mark_dirty only affects loaded chunks"

---

### R4: Retry Strategy

**Question**: How should mesh rebuild retries be tracked?

**Findings**:
- No existing retry mechanism in ChunkManager
- GPU exhaustion is rare but can happen during heavy load
- Spec requires max 3 retries then skip

**Decision**: Add `retry_counts: HashMap<ChunkCoord, u8>` to ChunkManager

**Rationale**:
- HashMap only grows when failures occur (typically empty)
- u8 sufficient for max retry count
- Clear lifecycle: increment on failure, remove on success/skip

**Alternative Rejected**: Storing retry count in Chunk struct
- Would require mutable Chunk access from ChunkManager
- Violates separation (ChunkManager shouldn't modify Chunk internals)

---

### R5: Metrics Design

**Question**: What metrics format aligns with Feature 010?

**Findings**:
- Feature 010 uses `tracing` crate for structured logging
- No explicit metrics system defined yet
- Debug overlay displays basic counters

**Decision**: Simple counter struct with pub fields

```rust
#[derive(Debug, Clone, Default)]
pub struct MeshMetrics {
    pub rebuilds_this_frame: u32,
    pub dirty_queue_depth: u32,
    pub skipped_chunks_total: u32,
}
```

**Rationale**:
- Matches existing pattern (ChunkManagerUpdate has pub fields)
- No runtime overhead for metric collection
- Easy to display in debug overlay
- Can later wrap with tracing if needed

---

## Spec Corrections

### SC-007 Neighbor Count

**Original Spec**: "Boundary detection correctly identifies all 26 possible neighbor configurations"

**Correction**: For mesh visibility, only **axis-aligned** neighbors matter. The 26 configurations include diagonals which don't affect mesh faces. Correct count:
- Interior: 0 neighbors
- Single face boundary: 1 neighbor
- Single edge boundary: 2 neighbors
- Single corner boundary: 3 neighbors

**Max neighbors**: 3 (not 7 as originally stated in US5 acceptance scenario 3)

---

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| plix-common | workspace | ChunkCoord, boundary_neighbors |
| plix-client | workspace | ChunkManager, ClientWorld |
| std::collections | stdlib | HashMap, HashSet, VecDeque |

No new external dependencies required.

---

## Conclusion

All NEEDS CLARIFICATION items resolved. The implementation extends existing Feature 011 infrastructure with:
1. Loaded-check in mark_dirty()
2. Retry tracking via HashMap
3. MeshMetrics counter struct
4. Integration point in ClientWorld.set_block()

Existing boundary detection is correct for mesh visibility (axis-aligned neighbors only).
