# Data Model: Procedural Generation v1

**Feature**: 013-procedural-generation
**Date**: 2025-12-16

## Entities

### BlockType (Extended)

**Location**: `crates/plix-common/src/types.rs`

Extends existing `BlockType` with new variants for terrain generation.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct BlockType(pub u8);

impl BlockType {
    // Existing
    pub const AIR: Self = Self(0);
    pub const STONE: Self = Self(1);
    pub const BRICK: Self = Self(2);
    pub const METAL: Self = Self(3);

    // New for terrain generation
    pub const GRASS: Self = Self(4);      // Plains surface
    pub const DIRT: Self = Self(5);       // Plains subsurface
    pub const SAND: Self = Self(6);       // Desert surface
    pub const SANDSTONE: Self = Self(7);  // Desert subsurface
    pub const BEDROCK: Self = Self(8);    // World floor (indestructible)
}
```

**Validation Rules**:
- Values 0-8 are valid block types
- BEDROCK should not be breakable (enforced by gameplay, not this feature)

---

### Biome

**Location**: `crates/plix-common/src/worldgen/biome.rs`

Represents terrain biome types and their properties.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biome {
    Plains,
    Mountains,
    Desert,
}

impl Biome {
    /// Block type for terrain surface (1 layer)
    pub fn surface_block(&self) -> BlockType {
        match self {
            Biome::Plains => BlockType::GRASS,
            Biome::Mountains => BlockType::STONE,
            Biome::Desert => BlockType::SAND,
        }
    }

    /// Block type for subsurface (3 layers below surface)
    pub fn subsurface_block(&self) -> BlockType {
        match self {
            Biome::Plains => BlockType::DIRT,
            Biome::Mountains => BlockType::STONE,
            Biome::Desert => BlockType::SANDSTONE,
        }
    }

    /// Height amplitude multiplier for this biome
    pub fn height_amplitude(&self) -> f64 {
        match self {
            Biome::Plains => 1.0,
            Biome::Mountains => 2.0,
            Biome::Desert => 0.8,
        }
    }
}
```

**State Transitions**: N/A (enum is stateless)

---

### WorldGenConfig

**Location**: `crates/plix-common/src/worldgen/config.rs`

Immutable configuration for world generation.

```rust
#[derive(Debug, Clone)]
pub struct WorldGenConfig {
    /// Primary world seed (u64 for large seed space)
    pub seed: u64,

    /// Minimum terrain height (blocks)
    pub min_height: i32,

    /// Maximum terrain height (blocks)
    pub max_height: i32,

    /// Noise scale for heightmap (smaller = larger features)
    pub height_scale: f64,

    /// Number of octaves for fractal noise
    pub height_octaves: u32,

    /// Noise scale for biome selection
    pub biome_scale: f64,

    /// Subsurface layer depth (blocks below surface)
    pub subsurface_depth: u32,
}

impl Default for WorldGenConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            min_height: 32,
            max_height: 96,
            height_scale: 0.01,
            height_octaves: 3,
            biome_scale: 0.005,  // Larger biome regions
            subsurface_depth: 3,
        }
    }
}
```

**Validation Rules**:
- `min_height < max_height`
- `height_scale > 0.0`
- `height_octaves >= 1`
- `biome_scale > 0.0`
- `subsurface_depth >= 1`

---

### NoiseSource

**Location**: `crates/plix-common/src/worldgen/noise.rs`

Wrapper around noise-rs providing seeded noise functions.

```rust
use noise::{NoiseFn, Perlin, Fbm};

/// Seed offsets for different noise layers (prevents correlation)
const HEIGHT_SEED_OFFSET: u32 = 0;
const BIOME_SEED_OFFSET: u32 = 1000;
const TEMPERATURE_SEED_OFFSET: u32 = 2000;

pub struct NoiseSource {
    height_noise: Fbm<Perlin>,
    biome_noise: Perlin,
    temperature_noise: Perlin,
}

impl NoiseSource {
    /// Create noise sources from world seed
    pub fn new(seed: u64, octaves: u32) -> Self {
        let base_seed = Self::derive_seed(seed, 0);

        Self {
            height_noise: Fbm::<Perlin>::new(Self::derive_seed(seed, HEIGHT_SEED_OFFSET))
                .set_octaves(octaves as usize),
            biome_noise: Perlin::new(Self::derive_seed(seed, BIOME_SEED_OFFSET)),
            temperature_noise: Perlin::new(Self::derive_seed(seed, TEMPERATURE_SEED_OFFSET)),
        }
    }

    /// Sample height noise at world position (returns [-1, 1])
    pub fn sample_height(&self, x: f64, z: f64, scale: f64) -> f64 {
        self.height_noise.get([x * scale, z * scale])
    }

    /// Sample biome elevation noise (returns [-1, 1])
    pub fn sample_biome_elevation(&self, x: f64, z: f64, scale: f64) -> f64 {
        self.biome_noise.get([x * scale, z * scale])
    }

    /// Sample temperature noise (returns [-1, 1])
    pub fn sample_temperature(&self, x: f64, z: f64, scale: f64) -> f64 {
        self.temperature_noise.get([x * scale, z * scale])
    }

    /// Derive u32 seed from u64 seed + offset
    fn derive_seed(seed: u64, offset: u32) -> u32 {
        ((seed ^ (seed >> 32)) as u32).wrapping_add(offset)
    }
}
```

**Invariants**:
- Same seed always produces same noise values
- Different offsets produce uncorrelated noise

---

### HeightModel

**Location**: `crates/plix-common/src/worldgen/height.rs`

Computes terrain surface height at any world position.

```rust
pub struct HeightModel {
    config: WorldGenConfig,
    noise: NoiseSource,
}

impl HeightModel {
    pub fn new(config: WorldGenConfig) -> Self {
        let noise = NoiseSource::new(config.seed, config.height_octaves);
        Self { config, noise }
    }

    /// Get surface height at world position (x, z)
    /// Returns the Y coordinate of the topmost solid block
    pub fn surface_height(&self, x: i32, z: i32, biome: Biome) -> i32 {
        let raw_noise = self.noise.sample_height(
            x as f64,
            z as f64,
            self.config.height_scale,
        );

        // Apply biome amplitude multiplier
        let amplitude = biome.height_amplitude();
        let scaled_noise = raw_noise * amplitude;

        // Map [-1, 1] to [min_height, max_height]
        let height_range = self.config.max_height - self.config.min_height;
        let base_height = (self.config.min_height + self.config.max_height) / 2;

        base_height + (scaled_noise * (height_range as f64 / 2.0)) as i32
    }
}
```

---

### BiomeModel

**Location**: `crates/plix-common/src/worldgen/biome.rs`

Determines biome at any world position using per-block sampling.

```rust
pub struct BiomeModel {
    noise: NoiseSource,
    scale: f64,
}

impl BiomeModel {
    pub fn new(seed: u64, scale: f64) -> Self {
        Self {
            noise: NoiseSource::new(seed, 1),  // Single octave for biomes
            scale,
        }
    }

    /// Get biome at world position (x, z)
    /// Per-block sampling ensures smooth transitions
    pub fn biome_at(&self, x: i32, z: i32) -> Biome {
        let elevation = self.noise.sample_biome_elevation(
            x as f64,
            z as f64,
            self.scale,
        );
        let temperature = self.noise.sample_temperature(
            x as f64,
            z as f64,
            self.scale,
        );

        // Selection thresholds
        if elevation > 0.3 {
            Biome::Mountains
        } else if temperature > 0.2 {
            Biome::Desert
        } else {
            Biome::Plains
        }
    }
}
```

---

### ChunkGenerator

**Location**: `crates/plix-common/src/worldgen/generator.rs`

Main entry point for chunk generation.

```rust
use crate::chunk::{Chunk, ChunkCoord, CHUNK_SIZE};
use crate::types::BlockType;

pub struct ChunkGenerator {
    config: WorldGenConfig,
    height_model: HeightModel,
    biome_model: BiomeModel,
}

impl ChunkGenerator {
    pub fn new(config: WorldGenConfig) -> Self {
        let height_model = HeightModel::new(config.clone());
        let biome_model = BiomeModel::new(config.seed, config.biome_scale);

        Self {
            config,
            height_model,
            biome_model,
        }
    }

    /// Generate a complete chunk at the given coordinate
    /// Pure function: same coord always produces identical chunk
    pub fn generate_chunk(&self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord);

        // World-space base position
        let base_x = coord.x * CHUNK_SIZE as i32;
        let base_y = coord.y * CHUNK_SIZE as i32;
        let base_z = coord.z * CHUNK_SIZE as i32;

        // Generate column by column (x, z), then fill y
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let world_x = base_x + lx as i32;
                let world_z = base_z + lz as i32;

                // Per-block biome sampling
                let biome = self.biome_model.biome_at(world_x, world_z);
                let surface_y = self.height_model.surface_height(world_x, world_z, biome);

                // Fill column
                for ly in 0..CHUNK_SIZE {
                    let world_y = base_y + ly as i32;
                    let block = self.block_at(world_y, surface_y, biome);
                    chunk.set_block(lx, ly, lz, block);
                }
            }
        }

        chunk
    }

    /// Determine block type at world Y given surface height and biome
    fn block_at(&self, world_y: i32, surface_y: i32, biome: Biome) -> BlockType {
        if world_y < 0 {
            // Void below world
            BlockType::AIR
        } else if world_y == 0 {
            // Bedrock floor
            BlockType::BEDROCK
        } else if world_y > surface_y {
            // Above surface
            BlockType::AIR
        } else if world_y == surface_y {
            // Surface layer
            biome.surface_block()
        } else if world_y > surface_y - self.config.subsurface_depth as i32 {
            // Subsurface layer
            biome.subsurface_block()
        } else {
            // Deep underground
            BlockType::STONE
        }
    }

    /// Get the world seed
    pub fn seed(&self) -> u64 {
        self.config.seed
    }
}
```

---

## Relationships

```
WorldGenConfig
    └── used by ──> ChunkGenerator
                        ├── owns ──> HeightModel
                        │               └── owns ──> NoiseSource
                        └── owns ──> BiomeModel
                                        └── owns ──> NoiseSource

ChunkGenerator::generate_chunk(ChunkCoord) -> Chunk
    └── queries ──> BiomeModel::biome_at(x, z) -> Biome
    └── queries ──> HeightModel::surface_height(x, z, biome) -> i32
    └── produces ──> Chunk (existing type from plix-common)
```

---

## Integration Points

### With ChunkedWorld (Feature 011)

```rust
// In plix-common/src/world.rs or integration code
impl ChunkedWorld {
    /// Generate and insert a chunk if not already present
    pub fn get_or_generate_chunk(
        &mut self,
        coord: ChunkCoord,
        generator: &ChunkGenerator,
    ) -> &Chunk {
        if !self.has_chunk(coord) {
            let chunk = generator.generate_chunk(coord);
            self.insert_chunk(coord, chunk);
        }
        self.get_chunk(coord).unwrap()
    }
}
```

### With ChunkManager (Feature 012)

```rust
// In plix-client/src/chunk_manager.rs
impl ChunkManager {
    /// Load chunks around player, generating if needed
    pub fn update_with_generation(
        &mut self,
        player_pos: Vec3,
        world: &mut ChunkedWorld,
        generator: &ChunkGenerator,
    ) -> ChunkManagerUpdate {
        // Existing update logic, but generate missing chunks
        for coord in self.chunks_to_load(player_pos) {
            if !world.has_chunk(coord) {
                let chunk = generator.generate_chunk(coord);
                world.insert_chunk(coord, chunk);
                self.mark_dirty(coord);
            }
        }
        // ... rest of update
    }
}
```

---

## Metrics (Feature 010 Integration)

```rust
/// Generation metrics for observability
#[derive(Debug, Clone, Default)]
pub struct GenerationMetrics {
    /// Total chunks generated this session
    pub chunks_generated_total: u64,
    /// Chunks generated this frame/tick
    pub chunks_generated_this_frame: u32,
    /// Generation time exceeded budget count
    pub budget_exceeded_count: u64,
}
```

These can be exposed via existing metrics infrastructure from Feature 010.
