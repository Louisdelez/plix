# Data Model: World Edit Optimization

**Feature**: 012-world-edit-optimization
**Date**: 2025-12-16

## Overview

This feature extends existing data structures rather than introducing new entities. The primary changes are additions to `ChunkManager` and a new `MeshMetrics` struct.

## Entity Changes

### ChunkManagerConfig (Extended)

**Location**: `crates/plix-client/src/chunk_manager.rs`

```rust
#[derive(Debug, Clone)]
pub struct ChunkManagerConfig {
    /// View distance in chunks (radius around player).
    pub view_distance: u8,
    /// Maximum number of chunk meshes to rebuild per frame.
    pub mesh_budget_per_frame: u32,
    /// Maximum retry attempts for failed mesh rebuilds. [NEW]
    pub max_retries: u8,
}

impl Default for ChunkManagerConfig {
    fn default() -> Self {
        Self {
            view_distance: 8,
            mesh_budget_per_frame: 2,
            max_retries: 3,  // [NEW]
        }
    }
}
```

**Validation Rules**:
- `max_retries >= 1` (at least one attempt)
- `max_retries <= 10` (reasonable upper bound)

---

### ChunkManager (Extended)

**Location**: `crates/plix-client/src/chunk_manager.rs`

```rust
pub struct ChunkManager {
    // Existing fields
    config: ChunkManagerConfig,
    loaded: HashSet<ChunkCoord>,
    dirty_queue: VecDeque<ChunkCoord>,
    dirty_set: HashSet<ChunkCoord>,

    // New fields [Feature 012]
    /// Retry counts for chunks that failed mesh rebuild
    retry_counts: HashMap<ChunkCoord, u8>,
    /// Chunks that exceeded max retries and are skipped
    skipped_chunks: HashSet<ChunkCoord>,
    /// Current frame metrics
    metrics: MeshMetrics,
}
```

**State Transitions**:

```
                  ┌──────────────────┐
                  │     CLEAN        │
                  │ (not in queue)   │
                  └────────┬─────────┘
                           │ mark_dirty()
                           │ (if loaded)
                           ▼
                  ┌──────────────────┐
                  │      DIRTY       │◄────────────┐
                  │  (in queue)      │             │
                  └────────┬─────────┘             │
                           │ pop_dirty_batch()    │
                           ▼                       │
                  ┌──────────────────┐             │
                  │   REBUILDING     │             │
                  │ (processing)     │             │
                  └────────┬─────────┘             │
                           │                       │
              ┌────────────┴────────────┐         │
              │ success                 │ failure │
              ▼                         ▼         │
     ┌──────────────┐          ┌──────────────┐   │
     │    CLEAN     │          │ retry < max  │───┘
     │              │          │ (re-queue)   │
     └──────────────┘          └──────────────┘
                                       │
                                       │ retry >= max
                                       ▼
                               ┌──────────────┐
                               │   SKIPPED    │
                               │ (in skipped) │
                               └──────────────┘
```

---

### MeshMetrics (New)

**Location**: `crates/plix-client/src/chunk_manager.rs`

```rust
/// Metrics for mesh rebuild operations.
/// Updated each frame during ChunkManager::update().
#[derive(Debug, Clone, Default)]
pub struct MeshMetrics {
    /// Number of mesh rebuilds attempted this frame
    pub rebuilds_this_frame: u32,
    /// Current depth of the dirty queue
    pub dirty_queue_depth: u32,
    /// Total chunks skipped (exceeded max retries)
    pub skipped_chunks_total: u32,
    /// Successful rebuilds this frame
    pub successful_rebuilds: u32,
    /// Failed rebuilds this frame
    pub failed_rebuilds: u32,
}
```

**Validation Rules**: None (counters are always valid)

**Lifecycle**: Reset per-frame counters at start of `update()`, accumulate during processing.

---

### ChunkManagerUpdate (Extended)

**Location**: `crates/plix-client/src/chunk_manager.rs`

```rust
/// Result of ChunkManager::update()
#[derive(Debug, Default)]
pub struct ChunkManagerUpdate {
    /// Chunks that were newly loaded
    pub chunks_loaded: Vec<ChunkCoord>,
    /// Chunks that were unloaded
    pub chunks_unloaded: Vec<ChunkCoord>,
    /// Chunks that should have their mesh rebuilt this frame
    pub chunks_to_rebuild: Vec<ChunkCoord>,
    /// Current metrics snapshot [NEW]
    pub metrics: MeshMetrics,
}
```

---

## Relationships

```
┌─────────────────────────────────────────────────────────────────┐
│                        ChunkManager                              │
├─────────────────────────────────────────────────────────────────┤
│ loaded: HashSet<ChunkCoord>        │ Which chunks are active    │
│ dirty_queue: VecDeque<ChunkCoord>  │ FIFO rebuild queue         │
│ dirty_set: HashSet<ChunkCoord>     │ O(1) membership check      │
│ retry_counts: HashMap<ChunkCoord,u8>│ Failure tracking          │
│ skipped_chunks: HashSet<ChunkCoord>│ Exceeded retries           │
│ metrics: MeshMetrics               │ Per-frame counters         │
├─────────────────────────────────────────────────────────────────┤
│                         Invariants                               │
├─────────────────────────────────────────────────────────────────┤
│ • dirty_set.len() == dirty_queue.len()                          │
│ • dirty_set ⊆ loaded (dirty chunks must be loaded)              │
│ • skipped_chunks ∩ dirty_set = ∅ (skipped not in queue)         │
│ • retry_counts.keys() ⊆ dirty_set (retry only for dirty)        │
└─────────────────────────────────────────────────────────────────┘
```

---

## API Changes

### New Methods on ChunkManager

```rust
impl ChunkManager {
    /// Mark a chunk dirty along with any affected neighbors.
    /// Automatically detects boundary positions and marks neighbor chunks.
    /// Ignores marking for unloaded chunks.
    pub fn mark_dirty_for_block(&mut self, pos: BlockPos) {
        let (chunk_coord, local) = world_to_chunk(pos.x, pos.y, pos.z);

        // Ignore if chunk not loaded
        if !self.is_loaded(chunk_coord) {
            return;
        }

        // Mark the chunk itself
        self.mark_dirty(chunk_coord);

        // Mark boundary neighbors (only if loaded)
        if is_boundary_local(local) {
            for neighbor in boundary_neighbors(chunk_coord, local) {
                if self.is_loaded(neighbor) {
                    self.mark_dirty(neighbor);
                }
            }
        }
    }

    /// Report the result of a mesh rebuild attempt.
    /// Call this after attempting to rebuild each chunk returned by update().
    pub fn report_rebuild_result(&mut self, coord: ChunkCoord, success: bool) {
        if success {
            // Clear retry counter on success
            self.retry_counts.remove(&coord);
            self.metrics.successful_rebuilds += 1;
        } else {
            // Increment retry counter
            let count = self.retry_counts.entry(coord).or_insert(0);
            *count += 1;
            self.metrics.failed_rebuilds += 1;

            if *count >= self.config.max_retries {
                // Max retries exceeded - skip this chunk
                self.skipped_chunks.insert(coord);
                self.retry_counts.remove(&coord);
                self.metrics.skipped_chunks_total += 1;
            } else {
                // Re-queue for retry
                self.mark_dirty(coord);
            }
        }
    }

    /// Get current metrics.
    pub fn metrics(&self) -> &MeshMetrics {
        &self.metrics
    }

    /// Check if a chunk has been skipped due to repeated failures.
    pub fn is_skipped(&self, coord: ChunkCoord) -> bool {
        self.skipped_chunks.contains(&coord)
    }

    /// Clear skipped status for a chunk (e.g., after new edit).
    pub fn clear_skipped(&mut self, coord: ChunkCoord) {
        self.skipped_chunks.remove(&coord);
    }
}
```

### Modified Methods

```rust
impl ChunkManager {
    /// Mark a chunk as needing mesh rebuild.
    /// Uses deduplication to avoid rebuilding the same chunk multiple times.
    /// [MODIFIED] Now ignores marking for unloaded chunks.
    pub fn mark_dirty(&mut self, coord: ChunkCoord) {
        // [NEW] Ignore if not loaded
        if !self.is_loaded(coord) {
            return;
        }

        // [NEW] Clear skipped status on new edit
        self.skipped_chunks.remove(&coord);

        if self.dirty_set.insert(coord) {
            self.dirty_queue.push_back(coord);
        }
    }

    /// Main update - orchestrates load/unload/rebuild per frame.
    /// [MODIFIED] Now updates metrics.
    pub fn update(
        &mut self,
        player_pos: Vec3,
        world: &ChunkedWorld,
    ) -> ChunkManagerUpdate {
        // [NEW] Reset per-frame metrics
        self.metrics.rebuilds_this_frame = 0;
        self.metrics.successful_rebuilds = 0;
        self.metrics.failed_rebuilds = 0;

        let desired = self.compute_desired_chunks(player_pos);
        let chunks_loaded = self.load_missing_chunks(&desired, world);
        let chunks_unloaded = self.unload_far_chunks(&desired);
        let chunks_to_rebuild = self.pop_dirty_batch();

        // [NEW] Update metrics
        self.metrics.rebuilds_this_frame = chunks_to_rebuild.len() as u32;
        self.metrics.dirty_queue_depth = self.dirty_queue.len() as u32;

        ChunkManagerUpdate {
            chunks_loaded,
            chunks_unloaded,
            chunks_to_rebuild,
            metrics: self.metrics.clone(),  // [NEW]
        }
    }
}
```

---

## Storage

No persistent storage. All state is in-memory and scoped to client session.

---

## Migration

No data migration needed. New fields have sensible defaults:
- `retry_counts`: empty HashMap
- `skipped_chunks`: empty HashSet
- `metrics`: all zeros
- `max_retries`: 3 (default)
