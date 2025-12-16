# Implementation Plan: Chunked World

**Branch**: `011-chunked-world` | **Date**: 2025-12-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/011-chunked-world/spec.md`

## Summary

Replace monolithic voxel rendering with a scalable chunk-based world pipeline. The system partitions the world into 16x16x16 block chunks, streams them around the player, applies frustum and distance culling, generates per-chunk meshes with visible faces only, and performs partial rebuilds when blocks change.

**Key insight from research**: The codebase already has `ChunkPos` and `BlockPos.chunk_pos()` / `BlockPos.local_pos()` methods in `plix-common/src/types.rs`. The chunk coordinate math foundation exists—this feature builds the storage, meshing, streaming, and culling layers on top.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: wgpu 23.0 (rendering), glam (math), bincode (serialization), tokio (async)
**Storage**: In-memory chunked HashMap (client-side); arena still loads from TOML server-side
**Testing**: cargo test (unit + integration tests)
**Target Platform**: Linux (primary), cross-platform via wgpu
**Project Type**: Workspace with 6 crates (plix-common, plix-net, plix-server, plix-client, plix-arena, plix-tools)
**Performance Goals**: 60 fps rendering, no hitches >30ms during streaming, mesh rebuild ≤2 frames
**Constraints**: View distance 8 chunks, mesh budget 2 chunks/frame, chunk size 16x16x16
**Scale/Scope**: Arenas up to 64x32x64 blocks (4x2x4 chunks for test arena), extensible to larger worlds

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | ✅ Pass | Client chunks are view-only; server remains authoritative for block state |
| II. Performance (Low Latency) | ✅ Pass | Lazy chunk loading, per-frame mesh budget, frustum culling reduce load |
| III. Architecture (Engine-First) | ✅ Pass | Chunk system is engine primitive; gameplay builds on it |
| IV. Modding | N/A | No mod API changes |
| V. Code Quality | ✅ Pass | Unit tests required for coordinate math, meshing, culling |
| VI. Technical Standards | ✅ Pass | Stable Rust only, deterministic meshing, clippy/fmt compliance |
| VII. Player Experience | ✅ Pass | Streaming prevents load hitches; visual parity maintained |
| VIII. Open Source | ✅ Pass | No proprietary dependencies |
| IX. Scoping & Realism | ✅ Pass | MVP excludes LOD, multi-threading, infinite generation |
| X. Long-Term Vision | ✅ Pass | Chunk system is foundation for future procedural worlds |

**Gate Result**: PASS - No violations. Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/011-chunked-world/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (internal APIs)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-common/src/
│   ├── types.rs          # Existing: BlockPos, ChunkPos, BlockType
│   ├── chunk.rs          # NEW: Chunk data structure, ChunkCoord aliases
│   └── world.rs          # NEW: ChunkedWorld container (shared types)
│
├── plix-client/src/
│   ├── chunk_manager.rs  # NEW: Streaming, dirty queue, load/unload
│   ├── chunk_mesher.rs   # NEW: Per-chunk mesh generation
│   ├── world.rs          # MODIFY: Use ChunkedWorld instead of flat arena
│   └── render/
│       ├── engine.rs     # MODIFY: Render chunk meshes
│       └── voxels.rs     # MODIFY: Implement chunk-based voxel rendering
│
├── plix-arena/src/
│   └── format.rs         # MODIFY: Add LoadedArena::to_chunked_world()
│
└── plix-server/src/
    └── sim/
        └── collision.rs  # VERIFY: Works with chunked block access

tests/
├── chunk_coord_tests.rs  # NEW: Coordinate conversion unit tests
├── chunk_meshing_tests.rs # NEW: Face generation tests
└── chunk_culling_tests.rs # NEW: Frustum/distance culling tests
```

**Structure Decision**: Single workspace, feature adds modules to existing crates. Primary work in `plix-client` (ChunkManager, meshing), with shared types in `plix-common`.

## Complexity Tracking

> No violations requiring justification. Chunk system is minimal viable implementation.

| Decision | Rationale | Alternative Rejected |
|----------|-----------|---------------------|
| HashMap<ChunkCoord, Chunk> | Simple, O(1) lookup, sparse | Flat Vec (dense, wasteful for large worlds) |
| 16x16x16 chunk size | Balanced rebuild cost vs. memory | 32x32x32 (slower rebuilds), 8x8x8 (too many chunks) |
| Dense block array per chunk | Simple indexing, predictable memory | Octree (complexity, overkill for MVP) |

## Architecture Overview

### Phase Breakdown

```
Phase 0: Research → research.md
Phase 1: Design  → data-model.md, contracts/, quickstart.md
Phase 2: Tasks   → tasks.md (via /speckit.tasks)
```

### Key Components

1. **ChunkCoord & Coordinate Math** (plix-common)
   - Already exists: `BlockPos.chunk_pos()`, `BlockPos.local_pos()`, `ChunkPos`
   - Add: `ChunkCoord` type alias, `world_to_chunk()`, `chunk_to_world()` helpers

2. **Chunk Data Structure** (plix-common)
   - `Chunk { blocks: [BlockType; 4096], dirty: bool, aabb: AABB }`
   - Dense array for 16^3 = 4096 blocks

3. **ChunkedWorld Container** (plix-common/plix-client)
   - `HashMap<ChunkCoord, Chunk>` for sparse storage
   - `get_block(world_pos)` / `set_block(world_pos, block)` with auto-chunking

4. **ChunkManager** (plix-client)
   - Tracks player position, computes desired chunk set
   - Loads/unloads chunks based on view distance
   - Maintains dirty queue with deduplication
   - Schedules mesh rebuilds within per-frame budget

5. **ChunkMesher** (plix-client)
   - Generates visible faces per chunk
   - Cross-chunk neighbor lookup via `WorldView` trait
   - Outputs wgpu-compatible vertex/index buffers

6. **Culling** (plix-client)
   - Distance culling: skip chunks outside view radius
   - Frustum culling: AABB-frustum plane tests

7. **Integration Points**
   - Arena loading → `LoadedArena::to_chunked_world()`
   - Block edits → mark chunk dirty + neighbor if boundary
   - Late joiners → initialize ChunkedWorld from server snapshot

## Phases (Implementation Flow)

### Phase 1: Chunk Types & Coordinate Math
- Leverage existing `ChunkPos`, `BlockPos.chunk_pos()`, `BlockPos.local_pos()`
- Add `Chunk` struct with block storage
- Add coordinate conversion tests (roundtrips, negatives, boundaries)

### Phase 2: Chunked World Storage
- Implement `ChunkedWorld` container
- Integrate with arena loading (`LoadedArena::to_chunked_world()`)
- Verify block access works with existing collision/raycast code

### Phase 3: Chunk Meshing
- Implement `ChunkMesher` for visible face generation
- Handle cross-chunk neighbor lookups
- Create per-chunk GPU buffers
- Validate visual parity with existing rendering

### Phase 4: Streaming Manager
- Implement `ChunkManager` with load/unload logic
- View distance configuration
- No-hitch streaming (budget chunk loads per frame)

### Phase 5: Dirty Rebuild & Partial Updates
- Hook block edits to mark chunks dirty
- Boundary neighbor invalidation
- Per-frame rebuild budget (max 2 chunks)

### Phase 6: Culling
- Distance culling (implicit via streaming radius)
- Frustum culling (AABB-frustum test)
- Debug toggle for culling disable

### Phase 7: Validation & Non-Regression
- All automated tests pass
- Manual testing: fly around, block edits, culling behavior
- Headless mode unaffected

## Next Steps

1. Generate `research.md` (Phase 0) - consolidate codebase findings
2. Generate `data-model.md` (Phase 1) - entity definitions
3. Generate `contracts/` (Phase 1) - internal API contracts
4. Generate `quickstart.md` (Phase 1) - dev setup guide
5. Run `/speckit.tasks` for Phase 2 task generation
