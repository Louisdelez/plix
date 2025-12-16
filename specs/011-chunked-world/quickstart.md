# Quickstart: Chunked World Development

**Feature**: 011-chunked-world
**Date**: 2025-12-16

## Prerequisites

- Rust 1.75+ (stable)
- Git
- GPU with Vulkan/Metal/DX12 support (for wgpu)

## Setup

```bash
# Clone and checkout feature branch
git clone <repo-url>
cd plix
git checkout 011-chunked-world

# Build workspace
cargo build

# Run tests
cargo test

# Run lints
cargo clippy --all-targets
cargo fmt --all -- --check
```

## Project Structure

```
crates/
├── plix-common/     # Shared types (ChunkCoord, Chunk, ChunkedWorld)
├── plix-client/     # Client with ChunkManager, ChunkMesher
├── plix-server/     # Server (no changes for MVP)
├── plix-arena/      # Arena loading → ChunkedWorld conversion
├── plix-net/        # Network transport (no changes)
└── plix-tools/      # Testing utilities
```

## Key Files to Modify

### Phase 1: Chunk Types
- `crates/plix-common/src/chunk.rs` (NEW)
- `crates/plix-common/src/lib.rs` (add mod chunk)

### Phase 2: Chunked Storage
- `crates/plix-common/src/world.rs` (NEW)
- `crates/plix-arena/src/format.rs` (add to_chunked_world)

### Phase 3: Meshing
- `crates/plix-client/src/chunk_mesher.rs` (NEW)
- `crates/plix-client/src/render/voxels.rs` (implement)

### Phase 4: Streaming
- `crates/plix-client/src/chunk_manager.rs` (NEW)

### Phase 5: Block Edit Integration
- `crates/plix-client/src/world.rs` (modify apply_edit)

### Phase 6: Culling
- `crates/plix-client/src/render/engine.rs` (add frustum)

## Running the Game

### Server
```bash
cargo run -p plix-server -- --port 7777
```

### Client
```bash
cargo run -p plix-client -- --server 127.0.0.1:7777
```

## Testing

### Unit Tests
```bash
# All tests
cargo test

# Specific crate
cargo test -p plix-common

# Specific test
cargo test chunk_coord_roundtrip
```

### Manual Testing Checklist

1. **Visual Parity**
   - Load test_arena
   - Compare rendering to previous (should be identical)

2. **Streaming**
   - Move around arena
   - Watch for smooth chunk load/unload
   - No freezes or hitches

3. **Block Edits**
   - Place/remove blocks
   - Verify mesh updates within 1-2 frames
   - Test boundary edits (verify neighbor updates)

4. **Culling**
   - Look away from loaded chunks
   - Verify draw calls decrease (if metrics exposed)

## Debugging

### Enable Debug Logging
```bash
RUST_LOG=plix_client::chunk_manager=debug cargo run -p plix-client
```

### Chunk Bounds Visualization (if implemented)
```toml
# In config
[debug]
show_chunk_bounds = true
```

## Configuration Defaults

| Parameter | Default | Description |
|-----------|---------|-------------|
| `CHUNK_SIZE` | 16 | Blocks per chunk dimension |
| `view_distance_chunks` | 8 | Chunk radius around player |
| `mesh_budget_per_frame` | 2 | Max chunk rebuilds per frame |
| `culling_enabled` | true | Enable frustum culling |

## Common Issues

### Mesh Not Updating
- Check `mark_dirty()` is called on block edit
- Check dirty queue is being processed
- Verify `mesh_budget > 0`

### Visual Artifacts at Chunk Boundaries
- Cross-chunk neighbor lookup failing
- Check `WorldView::get_block()` handles missing chunks

### Performance Issues
- Reduce `view_distance_chunks`
- Increase `mesh_budget_per_frame` if CPU-bound
- Verify culling is enabled

## API Quick Reference

```rust
// Coordinate conversion
let pos = BlockPos::new(17, 5, 32);
let chunk_coord = pos.chunk_pos();  // ChunkPos(1, 0, 2)
let local = pos.local_pos();        // (1, 5, 0)

// World access
let world = ChunkedWorld::new();
let block = world.get_block(pos);   // Returns AIR if not loaded
world.set_block(pos, BlockType::STONE);

// Chunk manager
let mut manager = ChunkManager::with_defaults();
manager.update(player_pos, &mut world, &device);
manager.mark_dirty(chunk_coord);

// Rendering
for (coord, mesh) in manager.visible_chunks(&frustum, &world) {
    // Draw mesh
}
```

## Next Steps

After implementation:
1. Run full test suite: `cargo test`
2. Run lints: `cargo clippy && cargo fmt --check`
3. Manual QA per checklist above
4. Commit and push to branch
