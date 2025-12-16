# Quickstart: World Edit Optimization

**Feature**: 012-world-edit-optimization
**Date**: 2025-12-16

## Overview

This guide explains how to use the optimized block edit system for maintaining performance during build fights.

## Prerequisites

- Feature 011 (Chunked World) must be implemented
- `ChunkManager` and `ChunkedWorld` are available

## Basic Usage

### 1. Block Edit with Automatic Dirty Marking

When a block is placed or removed, use `mark_dirty_for_block()` to automatically handle boundary neighbors:

```rust
use plix_common::types::BlockPos;

// In your block edit handler
fn handle_block_edit(
    chunk_manager: &mut ChunkManager,
    world: &mut ChunkedWorld,
    pos: BlockPos,
    new_block: BlockType,
) {
    // Update world state
    world.set_block(pos, new_block);

    // Mark affected chunks as dirty (handles boundaries automatically)
    chunk_manager.mark_dirty_for_block(pos);
}
```

### 2. Processing Mesh Rebuilds

During the render loop, process the dirty queue:

```rust
fn render_frame(
    chunk_manager: &mut ChunkManager,
    world: &ChunkedWorld,
    mesher: &ChunkMesher,
    engine: &mut RenderEngine,
    player_pos: Vec3,
) {
    // Get chunks to rebuild this frame (respects mesh budget)
    let update = chunk_manager.update(player_pos, world);

    // Rebuild meshes for returned chunks
    for coord in update.chunks_to_rebuild {
        match mesher.build_chunk_mesh(coord, world, engine.device()) {
            Ok(mesh) => {
                engine.set_chunk_mesh(coord, mesh);
                chunk_manager.report_rebuild_result(coord, true);
            }
            Err(e) => {
                tracing::warn!(?coord, ?e, "Mesh rebuild failed");
                chunk_manager.report_rebuild_result(coord, false);
            }
        }
    }

    // Remove meshes for unloaded chunks
    for coord in update.chunks_unloaded {
        engine.remove_chunk_mesh(coord);
    }
}
```

### 3. Monitoring Performance

Access metrics for debugging:

```rust
fn render_debug_overlay(chunk_manager: &ChunkManager, ui: &mut DebugUI) {
    let metrics = chunk_manager.metrics();

    ui.label(format!("Dirty Queue: {}", metrics.dirty_queue_depth));
    ui.label(format!("Rebuilds/Frame: {}", metrics.rebuilds_this_frame));
    ui.label(format!("Skipped Chunks: {}", metrics.skipped_chunks_total));
}
```

## Configuration

Customize behavior via `ChunkManagerConfig`:

```rust
let config = ChunkManagerConfig {
    view_distance: 8,           // Chunks around player
    mesh_budget_per_frame: 2,   // Max rebuilds per frame
    max_retries: 3,             // Retries before skipping
};

let chunk_manager = ChunkManager::with_config(config);
```

### Tuning Guidelines

| Setting | Low-End HW | Mid-Range | High-End |
|---------|------------|-----------|----------|
| `mesh_budget_per_frame` | 1 | 2 | 4 |
| `view_distance` | 4 | 8 | 12 |

## Error Handling

### Skipped Chunks

Chunks that fail mesh rebuild 3 times are skipped:

```rust
// Check if a chunk was skipped
if chunk_manager.is_skipped(coord) {
    // Chunk mesh is missing due to repeated failures
    // Will be re-attempted on next block edit in that chunk
}
```

### Re-enabling Skipped Chunks

Skipped chunks are automatically re-enabled when:
1. A new block edit occurs in that chunk
2. The chunk is unloaded and reloaded

Manual clear:
```rust
chunk_manager.clear_skipped(coord);
```

## Integration Points

### Server-Sent Block Updates

```rust
fn handle_server_block_update(
    msg: ServerMessage::BlockUpdate,
    chunk_manager: &mut ChunkManager,
    world: &mut ChunkedWorld,
) {
    let ServerMessage::BlockUpdate { pos, block } = msg;
    world.set_block(pos, block);
    chunk_manager.mark_dirty_for_block(pos);
}
```

### Local Block Placement

```rust
fn handle_local_block_place(
    input: &PlayerInput,
    chunk_manager: &mut ChunkManager,
    world: &mut ChunkedWorld,
) {
    if let Some((pos, block)) = input.pending_block_place() {
        world.set_block(pos, block);
        chunk_manager.mark_dirty_for_block(pos);
        // Send to server for validation...
    }
}
```

## Testing

Run feature tests:

```bash
cargo test -p plix-client chunk_manager
cargo test -p plix-common chunk::tests
```

## Troubleshooting

### High Dirty Queue Depth

If `dirty_queue_depth` grows unbounded:
- Reduce `mesh_budget_per_frame` to catch up
- Check for edit spam (deduplication should prevent this)

### Many Skipped Chunks

If `skipped_chunks_total` is high:
- GPU may be exhausted - reduce graphical load
- Check for mesh generation errors in logs

### Visual Artifacts at Boundaries

If block faces appear/disappear incorrectly:
- Verify `mark_dirty_for_block()` is called (not just `mark_dirty()`)
- Check that boundary neighbors are loaded
