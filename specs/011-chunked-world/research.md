# Research: Chunked World

**Feature**: 011-chunked-world
**Date**: 2025-12-16
**Status**: Complete

## Executive Summary

The plix codebase already has chunk coordinate infrastructure (`ChunkPos`, `BlockPos.chunk_pos()`, `BlockPos.local_pos()`) in `plix-common/src/types.rs`. This feature builds chunk storage, meshing, streaming, and culling on top of existing primitives.

## Research Findings

### 1. Existing Chunk Coordinate System

**Decision**: Leverage existing `ChunkPos` and coordinate conversion methods
**Rationale**: Code already exists and is tested; avoid duplication
**Alternatives Rejected**: Creating new types (redundant), different chunk size (would break existing math)

**Existing Code** (`plix-common/src/types.rs`):
```rust
pub struct ChunkPos { pub x: i32, pub y: i32, pub z: i32 }
pub struct BlockPos { pub x: i32, pub y: i32, pub z: i32 }

impl BlockPos {
    fn chunk_pos(&self) -> ChunkPos  // Divides by 16
    fn local_pos(&self) -> (usize, usize, usize)  // 0-15 range
}
```

**Gap**: Need to add `Chunk` struct and `ChunkedWorld` container.

---

### 2. Current Block Storage

**Decision**: Replace flat `Vec<BlockType>` with `HashMap<ChunkCoord, Chunk>`
**Rationale**: Sparse storage scales better; enables lazy loading and unloading
**Alternatives Rejected**:
- Keep flat Vec (doesn't support streaming, wastes memory for sparse worlds)
- Octree (overkill complexity for MVP, premature optimization)

**Current Storage** (`plix-arena/src/format.rs`):
```rust
pub struct LoadedArena {
    pub definition: Arena,
    pub blocks: Vec<BlockType>,  // Flattened 3D array, index = z*sy*sx + y*sx + x
}
```

**Migration Path**: Add `LoadedArena::to_chunked_world()` method to convert flat storage to chunked.

---

### 3. Rendering Pipeline

**Decision**: Generate per-chunk meshes with visible faces only
**Rationale**: Reduces GPU work, enables partial rebuilds, standard voxel technique
**Alternatives Rejected**:
- Greedy meshing (deferred to future optimization)
- Instance rendering (complex, not needed for MVP chunk counts)

**Current State** (`plix-client/src/render/voxels.rs`):
- Placeholder VoxelRenderer with TODO for greedy meshing
- Engine supports vertex/index buffers via wgpu

**Vertex Format** (`plix-client/src/render/engine.rs`):
```rust
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}
```

**Implementation**: Create `ChunkMesher` that iterates blocks, emits faces where neighbor is air.

---

### 4. Block Edit Integration

**Decision**: Hook into existing `BlockEditApplied` event to mark chunks dirty
**Rationale**: Minimal change to existing flow; server remains authoritative
**Alternatives Rejected**: Client-side prediction of edits (violates server authority)

**Existing Flow**:
1. Client sends `BlockEditRequest`
2. Server validates and applies edit
3. Server broadcasts `BlockEditApplied { pos, new_block, tick }`
4. Client receives event, calls `ClientWorld::apply_edit()`

**New Hook**: In `apply_edit()`, mark affected chunk dirty. If block is on boundary (local coord 0 or 15), also mark neighbor chunk dirty.

---

### 5. Frustum Culling

**Decision**: Use AABB-frustum plane test per chunk
**Rationale**: Simple, effective, well-documented algorithm
**Alternatives Rejected**:
- Hierarchical BVH (overkill for ~32 chunks at 8-chunk radius)
- Sphere-frustum (AABB is more accurate for cubic chunks)

**Implementation**: Extract 6 frustum planes from view-projection matrix, test each chunk AABB.

---

### 6. Memory Layout

**Decision**: Dense `[BlockType; 4096]` array per chunk
**Rationale**: Simple indexing, predictable cache behavior, 4KB per chunk (BlockType is u8)
**Alternatives Rejected**:
- Run-length encoding (complexity, deferred)
- Palette compression (complexity, deferred)

**Memory Estimate**:
- 8-chunk radius = 17x17x17 chunks max loaded ≈ 4913 chunks (worst case)
- 4KB/chunk = ~20MB block data (acceptable)
- Mesh data varies; ~10KB/chunk average → ~50MB (acceptable)

---

### 7. Streaming Strategy

**Decision**: Load all chunks within radius, unload outside, budget work per frame
**Rationale**: Simple deterministic behavior, prevents hitches
**Alternatives Rejected**:
- Priority queue by distance (complexity, marginal benefit)
- Async loading (requires threading, out of scope)

**Configuration**:
- `view_distance_chunks: u8 = 8` (configurable)
- `mesh_budget_per_frame: u32 = 2` (configurable)

---

### 8. Network Compatibility

**Decision**: No protocol changes for MVP; chunk system is client-only view
**Rationale**: Server continues sending `BlockEditApplied` events; client builds chunked view
**Alternatives Rejected**:
- Server sends chunk data (protocol change, not needed for current arena sizes)
- Delta compression per chunk (deferred optimization)

**Late Joiner Flow**:
1. Server sends `Connected { arena_data }` with arena definition
2. Client builds full arena from definition
3. Client converts to `ChunkedWorld`
4. Subsequent `BlockEditApplied` events update chunks

---

## Open Questions (Resolved)

| Question | Resolution |
|----------|------------|
| Chunk size? | 16x16x16 (matches existing `chunk_pos()` math) |
| Storage structure? | HashMap<ChunkCoord, Chunk> for sparse access |
| Mesh format? | Existing `Vertex { position, color }` format |
| Cross-chunk neighbors? | `WorldView` trait abstracts block lookup across chunks |
| Dirty queue behavior? | VecDeque with HashSet for deduplication |

## Dependencies Identified

| Dependency | Version | Purpose |
|------------|---------|---------|
| wgpu | 23.0 | GPU mesh buffers |
| glam | (workspace) | Math for frustum planes, AABB |
| bincode | (workspace) | Serialization (existing) |

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Visual regression | Medium | High | Automated visual diff tests, manual QA |
| Performance degradation | Low | Medium | Benchmark before/after, mesh budget |
| Memory bloat | Low | Low | Monitor chunk count, unload aggressively |

## Conclusion

No NEEDS CLARIFICATION items remain. The implementation path is clear:
1. Add `Chunk` and `ChunkedWorld` types to `plix-common`
2. Add `ChunkManager` and `ChunkMesher` to `plix-client`
3. Integrate with arena loading and block edit events
4. Add frustum culling to render loop
5. Comprehensive testing

Proceed to Phase 1 design artifacts.
