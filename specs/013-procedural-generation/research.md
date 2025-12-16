# Research: Procedural Generation v1

**Feature**: 013-procedural-generation
**Date**: 2025-12-16

## Research Questions

### RQ-001: Noise Library Selection

**Question**: Which noise library best fits deterministic voxel world generation in Rust?

**Decision**: Use `noise` crate (noise-rs) version 0.9.0

**Rationale**:
- Mature, actively maintained Rust library for procedural noise
- Deterministic seeding via `Seedable` trait - `Perlin::new(seed)` produces identical output
- Uses XorShiftRng internally for seed table generation (not crypto, but deterministic)
- Supports Perlin, Simplex, OpenSimplex, and combinators (fBm, RidgedMulti)
- Works on stable Rust, no nightly features required
- Well-tested in game development contexts

**Alternatives Considered**:
- `fastnoise-lite`: Faster but less mature Rust bindings, smaller community
- Hand-rolled implementation: More control but higher risk of bugs, significant effort
- `libnoise`: C++ library, FFI complexity not justified for this use case

**API Usage**:
```rust
use noise::{NoiseFn, Perlin, Seedable};

let perlin = Perlin::new(seed as u32);
let height = perlin.get([x as f64 / scale, z as f64 / scale]);
// Returns f64 in [-1, 1] range
```

**Sources**:
- [noise-rs GitHub](https://github.com/Razaekel/noise-rs)
- [noise crate docs](https://docs.rs/noise/latest/noise/)

---

### RQ-002: Seed Derivation Strategy

**Question**: How to derive multiple noise sources from a single world seed?

**Decision**: Use seed + fixed offset constants for sub-seeds

**Rationale**:
- Simple, deterministic, no complex hashing needed
- Each noise layer gets `base_seed + OFFSET` where offsets are compile-time constants
- Avoids correlation between noise layers (height vs biome)
- Standard practice in voxel games (Minecraft uses similar approach)

**Implementation**:
```rust
const HEIGHT_SEED_OFFSET: u32 = 0;
const BIOME_SEED_OFFSET: u32 = 1000;
const TEMPERATURE_SEED_OFFSET: u32 = 2000;

fn derive_seed(base: u64, offset: u32) -> u32 {
    // noise-rs uses u32 seeds, so we combine high/low bits
    ((base ^ (base >> 32)) as u32).wrapping_add(offset)
}
```

**Alternatives Considered**:
- Hash-based derivation: Overkill for this use case, adds complexity
- Single noise with different frequencies: Can cause visual correlation artifacts

---

### RQ-003: Heightmap Noise Configuration

**Question**: What noise parameters produce natural-looking terrain?

**Decision**: Use Perlin with fBm (fractal Brownian motion) at 2-3 octaves

**Rationale**:
- fBm adds detail at multiple scales (large hills + small bumps)
- 2-3 octaves balances visual quality vs performance
- Perlin preferred over Simplex for heightmaps (smoother gradients)
- Scale factor ~0.01 for chunk-scale variation (100 blocks = 1 noise unit)

**Parameters**:
```rust
// Base terrain configuration
const HEIGHT_SCALE: f64 = 0.01;      // 1 noise unit = 100 blocks
const HEIGHT_OCTAVES: usize = 3;      // Detail levels
const HEIGHT_PERSISTENCE: f64 = 0.5;  // Amplitude reduction per octave
const HEIGHT_LACUNARITY: f64 = 2.0;   // Frequency increase per octave

// Height range mapping
const MIN_HEIGHT: i32 = 32;
const MAX_HEIGHT: i32 = 96;
const HEIGHT_RANGE: i32 = 64;  // MAX - MIN
```

**Alternatives Considered**:
- Simple Perlin (no fBm): Too smooth, lacks interesting detail
- More octaves (4+): Diminishing returns, performance cost
- Simplex: Slightly faster but visually similar for heightmaps

---

### RQ-004: Biome Selection Algorithm

**Question**: How to select biomes smoothly across the world?

**Decision**: Dual-noise approach (elevation noise + temperature noise) with per-block sampling

**Rationale**:
- Two independent noise values create natural biome distribution
- Avoids artificial "stripe" patterns from single noise
- Per-block sampling ensures smooth transitions at all scales
- Simple threshold-based selection (not Voronoi or complex schemes)

**Biome Selection Logic**:
```rust
fn select_biome(elevation_noise: f64, temperature_noise: f64) -> Biome {
    // elevation_noise and temperature_noise are in [-1, 1]
    if elevation_noise > 0.3 {
        Biome::Mountains  // High elevation = mountains
    } else if temperature_noise > 0.2 {
        Biome::Desert     // Hot + not mountains = desert
    } else {
        Biome::Plains     // Default
    }
}
```

**Biome Effects on Height**:
| Biome | Height Amplitude Multiplier | Surface | Subsurface |
|-------|----------------------------|---------|------------|
| Plains | 1.0 (base) | GRASS | DIRT |
| Mountains | 2.0 (doubled) | STONE | STONE |
| Desert | 0.8 (flatter) | SAND | SANDSTONE |

**Alternatives Considered**:
- Chunk-level biome: Creates visible 16-block boundaries (rejected in clarification)
- Voronoi-based biomes: More complex, better for future v2
- Single noise: Creates correlated stripes

---

### RQ-005: Layer Placement Rules

**Question**: What layer thicknesses and rules produce realistic terrain?

**Decision**: Fixed layer depths from surface

**Rationale**:
- Simple, predictable, easy to test
- Bedrock at y=0 is absolute (not relative to surface)
- Surface layer = 1 block
- Subsurface layer = 3 blocks
- Remaining fill = STONE

**Layer Algorithm**:
```rust
fn block_at(x: i32, y: i32, z: i32, surface_height: i32, biome: Biome) -> BlockType {
    if y < 0 {
        BlockType::AIR  // Void below world
    } else if y == 0 {
        BlockType::BEDROCK
    } else if y > surface_height {
        BlockType::AIR
    } else if y == surface_height {
        biome.surface_block()  // GRASS, STONE, or SAND
    } else if y > surface_height - 3 {
        biome.subsurface_block()  // DIRT, STONE, or SANDSTONE
    } else {
        BlockType::STONE
    }
}
```

**Alternatives Considered**:
- Variable subsurface depth: Adds complexity without clear benefit for v1
- Noise-based layer variation: Good for v2, not needed for MVP

---

### RQ-006: Performance Optimization

**Question**: How to meet <10ms per chunk target?

**Decision**: Cache noise instances, sample in optimal order, avoid allocations

**Rationale**:
- noise-rs Perlin/fBm creation is cheap, can be done once per generator
- Iterate in memory-friendly order (x, then z, then y for cache locality)
- Pre-allocate chunk block array
- Height sampling is 2D (256 samples per chunk), not 3D (4096)

**Estimated Costs**:
- Height sampling: 256 noise calls at ~1μs each = ~0.25ms
- Biome sampling: 256 noise calls (can share height x,z) = ~0.25ms
- Block filling: 4096 comparisons = ~0.1ms
- Total: ~0.6ms per chunk (well under 10ms budget)

**Optimization Notes**:
- If needed: SIMD via rayon for parallel height sampling
- If needed: Lookup tables for biome thresholds
- Not needed for v1 given comfortable margin

---

### RQ-007: Thread Safety

**Question**: How to ensure thread-safe chunk generation?

**Decision**: Pure function design - no shared mutable state

**Rationale**:
- `ChunkGenerator` is stateless except for configuration
- Each call to `generate_chunk(coord)` creates local noise instances
- No global RNG, no mutexes needed
- Multiple threads can generate different chunks simultaneously

**API Design**:
```rust
pub struct ChunkGenerator {
    config: WorldGenConfig,  // Immutable after construction
}

impl ChunkGenerator {
    // Pure function: same inputs always produce same output
    pub fn generate_chunk(&self, coord: ChunkCoord) -> Chunk {
        // Creates local noise instances seeded from config.seed
        // No shared state modified
    }
}
```

---

## Dependency Addition

Add to `crates/plix-common/Cargo.toml`:
```toml
[dependencies]
noise = "0.9"
```

No other new dependencies required. The crate uses standard Rust types.

---

## Summary

All research questions resolved. Key decisions:
1. **noise-rs 0.9**: Mature, deterministic, stable Rust
2. **Offset-based sub-seeds**: Simple derivation from world seed
3. **Perlin + fBm**: 3 octaves for terrain heightmaps
4. **Dual-noise biomes**: Elevation + temperature with per-block sampling
5. **Fixed layers**: Surface (1), subsurface (3), stone fill, bedrock at y=0
6. **Pure functions**: Thread-safe by design, no shared state

Proceed to Phase 1 (data-model.md, quickstart.md).
