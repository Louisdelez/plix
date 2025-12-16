# Data Model: Chunked World

**Feature**: 011-chunked-world
**Date**: 2025-12-16

## Entity Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        ChunkedWorld                             │
│  HashMap<ChunkCoord, Chunk>                                     │
│  - get_block(BlockPos) -> BlockType                             │
│  - set_block(BlockPos, BlockType)                               │
│  - get_chunk(ChunkCoord) -> Option<&Chunk>                      │
│  - get_chunk_mut(ChunkCoord) -> Option<&mut Chunk>              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ contains
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                          Chunk                                  │
│  - blocks: [BlockType; 4096]  // 16x16x16                       │
│  - dirty: bool                                                  │
│  - aabb: AABB                                                   │
│  - mesh: Option<ChunkMesh>                                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ references
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       ChunkCoord                                │
│  - cx: i32                                                      │
│  - cy: i32                                                      │
│  - cz: i32                                                      │
│  (type alias for ChunkPos from plix-common)                     │
└─────────────────────────────────────────────────────────────────┘
```

## Entities

### ChunkCoord (alias: ChunkPos)

**Location**: `plix-common/src/types.rs` (existing)

**Purpose**: Identifies a chunk's position in chunk space.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| x | i32 | Chunk X coordinate | Signed for negative world coords |
| y | i32 | Chunk Y coordinate | Signed for negative world coords |
| z | i32 | Chunk Z coordinate | Signed for negative world coords |

**Identity**: (x, y, z) tuple uniquely identifies a chunk.

**Derived**:
- `Hash` + `Eq` for HashMap key usage
- `Copy` for value semantics

---

### BlockPos

**Location**: `plix-common/src/types.rs` (existing)

**Purpose**: Identifies a block's position in world space.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| x | i32 | World X coordinate | Signed |
| y | i32 | World Y coordinate | Signed |
| z | i32 | World Z coordinate | Signed |

**Methods**:
- `chunk_pos() -> ChunkCoord`: Returns containing chunk (floor division by 16)
- `local_pos() -> (usize, usize, usize)`: Returns position within chunk (0-15)

---

### BlockType

**Location**: `plix-common/src/types.rs` (existing)

**Purpose**: Identifies block material/type.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| 0 | u8 | Block type ID | 0 = AIR, 1+ = solid types |

**Constants**:
- `AIR = 0`
- `STONE = 1`
- `BRICK = 2`
- `METAL = 3`

**Methods**:
- `is_solid() -> bool`: Returns false for AIR, true otherwise

---

### Chunk

**Location**: `plix-common/src/chunk.rs` (NEW)

**Purpose**: Stores block data for a 16x16x16 region.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| blocks | [BlockType; 4096] | Dense block array | 16^3 = 4096 elements |
| dirty | bool | Needs mesh rebuild | Default: true on creation |
| aabb | AABB | World-space bounds | Computed from chunk coord |

**Invariants**:
- Index formula: `local_z * 256 + local_y * 16 + local_x`
- `aabb` must match chunk coord (min = coord * 16, max = min + 16)

**Methods**:
- `new(coord: ChunkCoord) -> Self`: Creates chunk filled with AIR
- `get_block(local: (usize, usize, usize)) -> BlockType`
- `set_block(local: (usize, usize, usize), block: BlockType)`
- `mark_dirty(&mut self)`
- `clear_dirty(&mut self)`
- `is_dirty(&self) -> bool`

---

### AABB

**Location**: `plix-common/src/chunk.rs` (NEW) or reuse existing if present

**Purpose**: Axis-aligned bounding box for culling.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| min | Vec3 | Minimum corner | min < max per axis |
| max | Vec3 | Maximum corner | max > min per axis |

**Methods**:
- `from_chunk_coord(coord: ChunkCoord) -> Self`
- `center() -> Vec3`
- `intersects_frustum(planes: &[Plane; 6]) -> bool`

---

### ChunkedWorld

**Location**: `plix-common/src/world.rs` (NEW)

**Purpose**: Container for all chunks in the world.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| chunks | HashMap<ChunkCoord, Chunk> | Loaded chunks | Sparse storage |

**Methods**:
- `new() -> Self`: Creates empty world
- `from_arena(arena: &LoadedArena) -> Self`: Converts flat arena to chunks
- `get_block(&self, pos: BlockPos) -> BlockType`: Returns AIR if chunk not loaded
- `set_block(&mut self, pos: BlockPos, block: BlockType)`: Creates chunk if needed
- `get_chunk(&self, coord: ChunkCoord) -> Option<&Chunk>`
- `get_chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut Chunk>`
- `ensure_chunk(&mut self, coord: ChunkCoord) -> &mut Chunk`: Creates if missing
- `remove_chunk(&mut self, coord: ChunkCoord) -> Option<Chunk>`
- `iter_chunks(&self) -> impl Iterator<Item = (&ChunkCoord, &Chunk)>`
- `dirty_chunks(&self) -> impl Iterator<Item = &ChunkCoord>`: Chunks needing rebuild

---

### ChunkMesh (Client-only)

**Location**: `plix-client/src/chunk_mesher.rs` (NEW)

**Purpose**: GPU resources for rendering a chunk.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| vertex_buffer | wgpu::Buffer | Vertex data | Position + color |
| index_buffer | wgpu::Buffer | Index data | u32 indices |
| num_indices | u32 | Draw count | Must match index buffer |

**Lifecycle**:
1. Created when chunk is meshed
2. Updated when chunk is dirty and rebuilt
3. Destroyed when chunk is unloaded

---

### ChunkManager (Client-only)

**Location**: `plix-client/src/chunk_manager.rs` (NEW)

**Purpose**: Manages chunk streaming and mesh scheduling.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| view_distance | u8 | Chunk radius | Default: 8 |
| mesh_budget | u32 | Rebuilds per frame | Default: 2 |
| loaded | HashSet<ChunkCoord> | Currently loaded | Matches world.chunks.keys() |
| dirty_queue | VecDeque<ChunkCoord> | Pending rebuilds | Deduplicated |
| meshes | HashMap<ChunkCoord, ChunkMesh> | GPU meshes | 1:1 with non-empty chunks |

**Methods**:
- `new(view_distance: u8, mesh_budget: u32) -> Self`
- `update(&mut self, player_pos: Vec3, world: &mut ChunkedWorld, device: &wgpu::Device)`
- `get_mesh(&self, coord: &ChunkCoord) -> Option<&ChunkMesh>`
- `mark_dirty(&mut self, coord: ChunkCoord)`: Adds to dirty queue if not present
- `visible_chunks(&self, frustum: &Frustum) -> Vec<&ChunkCoord>`: Culled iteration

---

## State Transitions

### Chunk Lifecycle

```
                    ┌──────────────────┐
                    │                  │
                    ▼                  │
┌─────────┐    ┌─────────┐    ┌─────────────┐    ┌──────────┐
│ Unloaded │───▶│ Loading │───▶│    Clean    │───▶│ Unloaded │
└─────────┘    └─────────┘    └─────────────┘    └──────────┘
                    │              │  ▲
                    │              │  │ rebuild
                    │              ▼  │
                    │          ┌───────┐
                    └─────────▶│ Dirty │
                    (block edit)└───────┘
```

**States**:
- **Unloaded**: Not in ChunkedWorld, no GPU resources
- **Loading**: Being populated from world data
- **Clean**: Has valid mesh, dirty=false
- **Dirty**: Needs mesh rebuild, dirty=true

**Transitions**:
- `Unloaded → Loading`: Player enters range, chunk created
- `Loading → Clean`: Initial mesh built
- `Clean → Dirty`: Block edit in chunk or at boundary
- `Dirty → Clean`: Mesh rebuilt (within budget)
- `Clean → Unloaded`: Player exits range, chunk destroyed

---

## Validation Rules

### Coordinate Conversion
- `BlockPos(x, y, z).chunk_pos()` must equal `ChunkCoord(floor(x/16), floor(y/16), floor(z/16))`
- `BlockPos(x, y, z).local_pos()` must be in range `[0, 15]` for each axis
- Roundtrip: `chunk_coord * 16 + local_pos == world_pos`

### Chunk Bounds
- `chunk.aabb.min == Vec3(coord.x * 16, coord.y * 16, coord.z * 16)`
- `chunk.aabb.max == chunk.aabb.min + Vec3(16, 16, 16)`

### Block Array Indexing
- Index `i` for local `(x, y, z)`: `i = z * 256 + y * 16 + x`
- Valid index range: `[0, 4095]`

### Dirty Propagation
- Edit at local `(0, _, _)` → mark neighbor at `(coord.x - 1, coord.y, coord.z)` dirty
- Edit at local `(15, _, _)` → mark neighbor at `(coord.x + 1, coord.y, coord.z)` dirty
- Same for Y and Z axes

---

## Data Volume Estimates

| Metric | Value | Notes |
|--------|-------|-------|
| Chunk size | 4,096 bytes | 16^3 blocks × 1 byte |
| Max loaded chunks | ~4,913 | 17^3 for 8-chunk radius |
| Block data memory | ~20 MB | Worst case, all chunks loaded |
| Mesh data per chunk | ~10 KB average | Varies by geometry |
| Mesh memory | ~50 MB | Worst case estimate |
| Total memory | ~70 MB | Block + mesh data |
