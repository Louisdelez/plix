# Quickstart: Procedural Generation v1

**Feature**: 013-procedural-generation
**Date**: 2025-12-16

## Overview

This guide explains how to use the procedural world generation system to create terrain from a seed.

## Prerequisites

- Feature 011 (Chunked World) implemented
- Feature 012 (World Edit Optimization) implemented
- `noise` crate added to plix-common dependencies

## Basic Usage

### 1. Create a World Generator

```rust
use plix_common::worldgen::{ChunkGenerator, WorldGenConfig};

// Create with default configuration
let generator = ChunkGenerator::new(WorldGenConfig {
    seed: 12345,
    ..Default::default()
});

// Or customize parameters
let config = WorldGenConfig {
    seed: 42,
    min_height: 32,
    max_height: 96,
    height_scale: 0.01,
    height_octaves: 3,
    biome_scale: 0.005,
    subsurface_depth: 3,
};
let generator = ChunkGenerator::new(config);
```

### 2. Generate Individual Chunks

```rust
use plix_common::chunk::ChunkCoord;

// Generate a single chunk
let coord = ChunkCoord::new(0, 0, 0);
let chunk = generator.generate_chunk(coord);

// Chunks are generated independently - order doesn't matter
let chunk_a = generator.generate_chunk(ChunkCoord::new(5, 0, 3));
let chunk_b = generator.generate_chunk(ChunkCoord::new(-1, 2, 0));
```

### 3. Integrate with ChunkedWorld

```rust
use plix_common::{ChunkedWorld, chunk::ChunkCoord};

let mut world = ChunkedWorld::new();
let generator = ChunkGenerator::new(WorldGenConfig::default());

// Generate and insert chunks as needed
fn ensure_chunk_exists(
    world: &mut ChunkedWorld,
    coord: ChunkCoord,
    generator: &ChunkGenerator,
) {
    if !world.has_chunk(coord) {
        let chunk = generator.generate_chunk(coord);
        world.insert_chunk(coord, chunk);
    }
}
```

### 4. Server-Side Generation

```rust
// In server world initialization
fn create_generated_world(seed: u64) -> (ChunkedWorld, ChunkGenerator) {
    let config = WorldGenConfig {
        seed,
        ..Default::default()
    };
    let generator = ChunkGenerator::new(config);
    let world = ChunkedWorld::new();

    (world, generator)
}

// Generate chunks on demand when players explore
fn handle_chunk_request(
    world: &mut ChunkedWorld,
    generator: &ChunkGenerator,
    coord: ChunkCoord,
) -> &Chunk {
    if !world.has_chunk(coord) {
        let chunk = generator.generate_chunk(coord);
        world.insert_chunk(coord, chunk);
    }
    world.get_chunk(coord).unwrap()
}
```

### 5. Client-Side Generation (Prediction)

```rust
use plix_client::chunk_manager::{ChunkManager, ChunkManagerConfig};

// Create chunk manager with generator
let mut chunk_manager = ChunkManager::with_config(ChunkManagerConfig {
    view_distance: 8,
    mesh_budget_per_frame: 2,
    max_retries: 3,
});

let generator = ChunkGenerator::new(WorldGenConfig { seed: server_seed, ..Default::default() });

// In render loop - generate chunks as player moves
fn update_world(
    chunk_manager: &mut ChunkManager,
    world: &mut ChunkedWorld,
    generator: &ChunkGenerator,
    player_pos: Vec3,
) -> ChunkManagerUpdate {
    // Get chunks that need to be loaded
    let update = chunk_manager.update(player_pos, world);

    // Generate any missing chunks within view distance
    for coord in chunk_manager.chunks_in_view(player_pos) {
        if !world.has_chunk(coord) {
            let chunk = generator.generate_chunk(coord);
            world.insert_chunk(coord, chunk);
            chunk_manager.mark_dirty(coord);
        }
    }

    update
}
```

## Configuration

### WorldGenConfig Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `seed` | 0 | World seed (u64) - same seed = same world |
| `min_height` | 32 | Minimum terrain height in blocks |
| `max_height` | 96 | Maximum terrain height in blocks |
| `height_scale` | 0.01 | Noise scale (smaller = larger hills) |
| `height_octaves` | 3 | Fractal detail levels |
| `biome_scale` | 0.005 | Biome region size (smaller = larger biomes) |
| `subsurface_depth` | 3 | Layers of subsurface blocks |

### Biome Properties

| Biome | Height Amplitude | Surface Block | Subsurface Block |
|-------|-----------------|---------------|------------------|
| Plains | 1.0x (base) | GRASS | DIRT |
| Mountains | 2.0x (doubled) | STONE | STONE |
| Desert | 0.8x (flatter) | SAND | SANDSTONE |

## Block Types

New block types added for terrain:

```rust
BlockType::GRASS     // id 4 - Plains surface
BlockType::DIRT      // id 5 - Plains subsurface
BlockType::SAND      // id 6 - Desert surface
BlockType::SANDSTONE // id 7 - Desert subsurface
BlockType::BEDROCK   // id 8 - World floor (y=0)
```

## Testing

Run feature tests:

```bash
# All worldgen tests
cargo test -p plix-common worldgen

# Specific test categories
cargo test -p plix-common worldgen::tests::determinism
cargo test -p plix-common worldgen::tests::layers
cargo test -p plix-common worldgen::tests::biomes
```

## Determinism Verification

```rust
// Same seed + coord = identical chunk
let gen1 = ChunkGenerator::new(WorldGenConfig { seed: 42, ..Default::default() });
let gen2 = ChunkGenerator::new(WorldGenConfig { seed: 42, ..Default::default() });

let coord = ChunkCoord::new(10, 0, 5);
let chunk1 = gen1.generate_chunk(coord);
let chunk2 = gen2.generate_chunk(coord);

assert_eq!(chunk1.blocks(), chunk2.blocks()); // Guaranteed identical
```

## Edge Cases

### Negative Y Chunks (Below Bedrock)

Chunks with `coord.y < 0` generate as entirely AIR (void):

```rust
let void_chunk = generator.generate_chunk(ChunkCoord::new(0, -1, 0));
// All 4096 blocks are AIR
```

### Extreme Coordinates

Generation works correctly at world edges:

```rust
// These all produce valid terrain
generator.generate_chunk(ChunkCoord::new(i32::MAX / 16, 0, 0));
generator.generate_chunk(ChunkCoord::new(i32::MIN / 16, 0, 0));
```

### Seed Extremes

Both edge seeds produce valid (different) terrain:

```rust
let gen_zero = ChunkGenerator::new(WorldGenConfig { seed: 0, ..Default::default() });
let gen_max = ChunkGenerator::new(WorldGenConfig { seed: u64::MAX, ..Default::default() });
```

## Performance

Expected performance on typical hardware:

| Operation | Target | Notes |
|-----------|--------|-------|
| Single chunk | <10ms | Well under budget (~1ms typical) |
| 512 chunks (spawn area) | <5s | ~10ms budget per chunk |
| Parallel generation | Linear scaling | Thread-safe, no shared state |

## Troubleshooting

### Terrain Seams at Chunk Boundaries

This should not happen - noise is continuous. If you see seams:
- Verify you're using world coordinates, not local chunk coordinates
- Check that biome sampling uses world (x, z), not local (lx, lz)

### Different Results on Client/Server

Ensure both use:
- Same `seed` value
- Same `WorldGenConfig` parameters
- Same version of noise-rs crate

### Performance Issues

If generation is slow:
- Reduce `height_octaves` (3 → 2)
- Increase `height_scale` (larger features = fewer samples matter)
- Use generation budget to spread work across frames
