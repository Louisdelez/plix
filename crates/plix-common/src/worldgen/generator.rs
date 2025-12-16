//! Chunk Generator
//!
//! Core generation logic that produces chunks from seed and coordinates.
//! Designed as a pure function with no side effects or external dependencies.

use std::time::Instant;

use crate::chunk::{Chunk, CHUNK_SIZE};
use crate::types::{BlockType, ChunkCoord};

use super::biome::BiomeModel;
use super::config::{GenerationMetrics, WorldGenConfig};
use super::height::HeightModel;

/// Chunk generator for procedural terrain
///
/// Thread-safe and produces deterministic output:
/// same seed + same coordinates = identical chunk.
///
/// # Thread Safety
///
/// The generator is safe to use from multiple threads. Each call to
/// `generate_chunk` is independent and does not modify shared state
/// (except for optional metrics tracking).
pub struct ChunkGenerator {
    config: WorldGenConfig,
    height_model: HeightModel,
    biome_model: BiomeModel,
    metrics: std::sync::Mutex<GenerationMetrics>,
}

impl ChunkGenerator {
    /// Create a new chunk generator with the given configuration
    pub fn new(config: WorldGenConfig) -> Self {
        let height_model = HeightModel::new(&config);
        let biome_model = BiomeModel::new(config.seed, config.biome_scale);

        Self {
            config,
            height_model,
            biome_model,
            metrics: std::sync::Mutex::new(GenerationMetrics::new()),
        }
    }

    /// Get the seed used by this generator
    pub fn seed(&self) -> u64 {
        self.config.seed
    }

    /// Get the current generation metrics
    pub fn metrics(&self) -> GenerationMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Generate a chunk at the given coordinates
    ///
    /// This is a pure function - the output depends only on the
    /// configuration seed and the chunk coordinates.
    ///
    /// # Arguments
    /// * `coord` - The chunk coordinates to generate
    ///
    /// # Returns
    /// A fully populated Chunk
    pub fn generate_chunk(&self, coord: ChunkCoord) -> Chunk {
        let start = Instant::now();

        let chunk = if coord.y < 0 {
            // Negative Y chunks are void (all AIR)
            self.generate_void_chunk(coord)
        } else {
            self.generate_terrain_chunk(coord)
        };

        // Record metrics
        let duration_us = start.elapsed().as_micros() as u64;
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_generation(duration_us);
        }

        chunk
    }

    /// Generate an empty (all AIR) chunk for void space
    fn generate_void_chunk(&self, coord: ChunkCoord) -> Chunk {
        Chunk::new(coord) // Default is all AIR
    }

    /// Generate a terrain chunk with heightmap-based blocks
    fn generate_terrain_chunk(&self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord);

        // Calculate world origin of this chunk
        let world_x = coord.x * CHUNK_SIZE as i32;
        let world_y = coord.y * CHUNK_SIZE as i32;
        let world_z = coord.z * CHUNK_SIZE as i32;

        // Generate each column
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let wx = world_x + lx as i32;
                let wz = world_z + lz as i32;

                // Determine biome at this column
                let biome = self.biome_model.biome_at(wx as f64, wz as f64);

                // Calculate surface height
                let surface_y = self
                    .height_model
                    .surface_height(wx as f64, wz as f64, biome);

                // Fill column
                for ly in 0..CHUNK_SIZE {
                    let wy = world_y + ly as i32;
                    let block = self.block_at(wy, surface_y, biome);
                    chunk.set_block(lx, ly, lz, block);
                }
            }
        }

        chunk
    }

    /// Determine the block type at a given Y coordinate
    ///
    /// Layer logic:
    /// - y < 0: AIR (handled by void chunk)
    /// - y == 0: BEDROCK
    /// - y < surface_y - subsurface_depth: STONE
    /// - y < surface_y: subsurface block (biome-specific)
    /// - y == surface_y: surface block (biome-specific)
    /// - y > surface_y: AIR
    fn block_at(&self, y: i32, surface_y: i32, biome: super::biome::Biome) -> BlockType {
        if y < 0 {
            BlockType::AIR
        } else if y == 0 {
            BlockType::BEDROCK
        } else if y > surface_y {
            BlockType::AIR
        } else if y == surface_y {
            biome.surface_block()
        } else if y > surface_y - self.config.subsurface_depth as i32 {
            biome.subsurface_block()
        } else {
            BlockType::STONE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === US1: Determinism Tests ===

    #[test]
    fn test_chunk_determinism_same_seed() {
        let config = WorldGenConfig::with_seed(12345);
        let gen1 = ChunkGenerator::new(config.clone());
        let gen2 = ChunkGenerator::new(config);

        let coord = ChunkCoord::new(5, 0, 3);
        let chunk1 = gen1.generate_chunk(coord);
        let chunk2 = gen2.generate_chunk(coord);

        // Block-for-block comparison
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    assert_eq!(
                        chunk1.get_block(x, y, z),
                        chunk2.get_block(x, y, z),
                        "Block mismatch at ({}, {}, {})",
                        x,
                        y,
                        z
                    );
                }
            }
        }
    }

    #[test]
    fn test_chunk_determinism_order_independent() {
        let config = WorldGenConfig::with_seed(12345);
        let generator = ChunkGenerator::new(config);

        // Generate chunks in different orders
        let coord_a = ChunkCoord::new(0, 0, 0);
        let coord_b = ChunkCoord::new(1, 0, 1);

        // Order 1: A then B
        let chunk_a1 = generator.generate_chunk(coord_a);
        let _ = generator.generate_chunk(coord_b);

        // Order 2: B then A (on fresh generator with same seed)
        let config2 = WorldGenConfig::with_seed(12345);
        let generator2 = ChunkGenerator::new(config2);
        let _ = generator2.generate_chunk(coord_b);
        let chunk_a2 = generator2.generate_chunk(coord_a);

        // Chunk A should be identical regardless of generation order
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    assert_eq!(
                        chunk_a1.get_block(x, y, z),
                        chunk_a2.get_block(x, y, z),
                        "Generation order affected chunk content at ({}, {}, {})",
                        x,
                        y,
                        z
                    );
                }
            }
        }
    }

    #[test]
    fn test_seed_edge_cases() {
        // Seed = 0 should work - verify chunk generates without panic
        let gen0 = ChunkGenerator::new(WorldGenConfig::with_seed(0));
        let chunk0 = gen0.generate_chunk(ChunkCoord::new(0, 0, 0));
        // Verify bedrock at y=0 (always present in valid terrain)
        assert_eq!(chunk0.get_block(0, 0, 0), BlockType::BEDROCK);

        // Seed = u64::MAX should work
        let gen_max = ChunkGenerator::new(WorldGenConfig::with_seed(u64::MAX));
        let chunk_max = gen_max.generate_chunk(ChunkCoord::new(0, 0, 0));
        assert_eq!(chunk_max.get_block(0, 0, 0), BlockType::BEDROCK);

        // Different seeds should produce different terrain (with high probability)
        // Test chunks at surface level (y=3-4 chunks, which is height 48-80 blocks)
        // where terrain variation is most visible
        let gen_a = ChunkGenerator::new(WorldGenConfig::with_seed(12345));
        let gen_b = ChunkGenerator::new(WorldGenConfig::with_seed(54321));

        let mut total_differences = 0;
        for cx in 0..5 {
            for cz in 0..5 {
                for cy in 2..6 {
                    // Check chunks at heights 32-96 where surface varies
                    let chunk_a = gen_a.generate_chunk(ChunkCoord::new(cx, cy, cz));
                    let chunk_b = gen_b.generate_chunk(ChunkCoord::new(cx, cy, cz));

                    for x in 0..CHUNK_SIZE {
                        for y in 0..CHUNK_SIZE {
                            for z in 0..CHUNK_SIZE {
                                if chunk_a.get_block(x, y, z) != chunk_b.get_block(x, y, z) {
                                    total_differences += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(
            total_differences > 0,
            "Different seeds should produce different terrain across multiple chunks"
        );
    }

    // === US2: Heightmap Tests ===

    #[test]
    fn test_solid_below_surface_air_above() {
        let config = WorldGenConfig::with_seed(42);
        let generator = ChunkGenerator::new(config);

        // Generate chunks at y=0 (ground level) and y=6 (should have some surface)
        for chunk_y in 0..8 {
            let chunk = generator.generate_chunk(ChunkCoord::new(0, chunk_y, 0));

            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let mut found_surface = false;
                    let mut solid_below = true;
                    let mut air_above = true;

                    for y in 0..CHUNK_SIZE {
                        let world_y = chunk_y * CHUNK_SIZE as i32 + y as i32;
                        let block = chunk.get_block(x, y, z);

                        if world_y == 0 {
                            // Bedrock check
                            assert_eq!(block, BlockType::BEDROCK, "Expected bedrock at y=0");
                        }

                        if !found_surface && !block.is_solid() && world_y > 0 {
                            found_surface = true;
                        } else if !found_surface && world_y > 0 {
                            // Below surface - should be solid
                            if !block.is_solid() {
                                solid_below = false;
                            }
                        } else if found_surface {
                            // Above surface - should be air
                            if block.is_solid() {
                                air_above = false;
                            }
                        }
                    }

                    // Only assert if we found the surface in this chunk
                    if found_surface {
                        assert!(
                            solid_below,
                            "Found non-solid block below surface at chunk ({}, {}, {}), local ({}, {})",
                            0, chunk_y, 0, x, z
                        );
                        assert!(
                            air_above,
                            "Found solid block above surface at chunk ({}, {}, {}), local ({}, {})",
                            0, chunk_y, 0, x, z
                        );
                    }
                }
            }
        }
    }

    // === US4: Layer Tests ===

    #[test]
    fn test_bedrock_at_y0() {
        let generator = ChunkGenerator::new(WorldGenConfig::with_seed(42));
        let chunk = generator.generate_chunk(ChunkCoord::new(0, 0, 0));

        // y=0 in chunk at y=0 should be bedrock
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                assert_eq!(
                    chunk.get_block(x, 0, z),
                    BlockType::BEDROCK,
                    "Expected bedrock at y=0 ({}, 0, {})",
                    x,
                    z
                );
            }
        }
    }

    #[test]
    fn test_negative_y_chunks_all_air() {
        let generator = ChunkGenerator::new(WorldGenConfig::with_seed(42));

        for y in (-5..-1).rev() {
            let chunk = generator.generate_chunk(ChunkCoord::new(0, y, 0));

            for x in 0..CHUNK_SIZE {
                for ly in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        assert_eq!(
                            chunk.get_block(x, ly, z),
                            BlockType::AIR,
                            "Negative Y chunk should be all AIR at ({}, {}, {})",
                            x,
                            ly,
                            z
                        );
                    }
                }
            }
        }
    }

    // === US5: Independence Tests ===

    #[test]
    fn test_chunk_independence_no_neighbors() {
        let generator = ChunkGenerator::new(WorldGenConfig::with_seed(42));

        // Generate chunks at various Y levels to ensure we find both solid and air
        // A ground-level chunk (y=0) should have solid blocks
        // A higher chunk should have some air (above terrain)
        let ground_chunk = generator.generate_chunk(ChunkCoord::new(1000, 0, 1000));
        let high_chunk = generator.generate_chunk(ChunkCoord::new(1000, 6, 1000)); // y=96+, likely above terrain

        // Ground chunk should have solid blocks (at least bedrock at y=0)
        let mut has_solid = false;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if ground_chunk.get_block(x, y, z).is_solid() {
                        has_solid = true;
                        break;
                    }
                }
            }
        }
        assert!(has_solid, "Ground-level chunk should contain solid blocks");

        // High chunk should have air (above terrain)
        let mut has_air = false;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if !high_chunk.get_block(x, y, z).is_solid() {
                        has_air = true;
                        break;
                    }
                }
            }
        }
        assert!(has_air, "High chunk should contain air blocks");
    }

    #[test]
    fn test_chunk_no_side_effects() {
        let config = WorldGenConfig::with_seed(42);
        let generator = ChunkGenerator::new(config.clone());

        // Generate one chunk
        let coord1 = ChunkCoord::new(0, 0, 0);
        let chunk1_first = generator.generate_chunk(coord1);

        // Generate many other chunks
        for x in 1..10 {
            for z in 1..10 {
                generator.generate_chunk(ChunkCoord::new(x, 0, z));
            }
        }

        // Generate the first chunk again - should be identical
        let chunk1_second = generator.generate_chunk(coord1);

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    assert_eq!(
                        chunk1_first.get_block(x, y, z),
                        chunk1_second.get_block(x, y, z),
                        "Chunk content changed after generating other chunks"
                    );
                }
            }
        }
    }

    #[test]
    fn test_parallel_generation_safe() {
        use std::sync::Arc;
        use std::thread;

        let generator = Arc::new(ChunkGenerator::new(WorldGenConfig::with_seed(42)));
        let mut handles = vec![];

        // Spawn multiple threads generating different chunks
        for i in 0..4 {
            let gen = Arc::clone(&generator);
            handles.push(thread::spawn(move || {
                let mut chunks = vec![];
                for j in 0..10 {
                    let coord = ChunkCoord::new(i * 10 + j, 0, 0);
                    chunks.push((coord, gen.generate_chunk(coord)));
                }
                chunks
            }));
        }

        // Collect results
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // Verify determinism - regenerate and compare
        for chunk_list in results {
            for (coord, original_chunk) in chunk_list {
                let regenerated = generator.generate_chunk(coord);
                for x in 0..CHUNK_SIZE {
                    for y in 0..CHUNK_SIZE {
                        for z in 0..CHUNK_SIZE {
                            assert_eq!(
                                original_chunk.get_block(x, y, z),
                                regenerated.get_block(x, y, z),
                                "Parallel generation produced inconsistent results"
                            );
                        }
                    }
                }
            }
        }
    }

    // === US6: Performance Tests ===

    #[test]
    fn test_chunk_generation_under_10ms() {
        let generator = ChunkGenerator::new(WorldGenConfig::with_seed(42));

        let start = std::time::Instant::now();
        let iterations = 100;

        for i in 0..iterations {
            generator.generate_chunk(ChunkCoord::new(i, 0, 0));
        }

        let elapsed = start.elapsed();
        let avg_ms = elapsed.as_millis() as f64 / iterations as f64;

        assert!(
            avg_ms < 10.0,
            "Average chunk generation time {}ms exceeds 10ms target",
            avg_ms
        );
    }

    #[test]
    fn test_metrics_tracking() {
        let generator = ChunkGenerator::new(WorldGenConfig::with_seed(42));

        let initial_metrics = generator.metrics();
        assert_eq!(initial_metrics.chunks_generated, 0);

        generator.generate_chunk(ChunkCoord::new(0, 0, 0));
        generator.generate_chunk(ChunkCoord::new(1, 0, 0));

        let final_metrics = generator.metrics();
        assert_eq!(final_metrics.chunks_generated, 2);
        assert!(final_metrics.total_generation_time_us > 0);
    }

    // === Integration Tests ===

    #[test]
    fn test_server_client_same_chunks() {
        // Simulate server and client with same seed producing identical chunks
        let seed = 42;
        let server_gen = ChunkGenerator::new(WorldGenConfig::with_seed(seed));
        let client_gen = ChunkGenerator::new(WorldGenConfig::with_seed(seed));

        // Generate same chunks on "server" and "client"
        for cx in -5..5 {
            for cz in -5..5 {
                let coord = ChunkCoord::new(cx, 0, cz);
                let server_chunk = server_gen.generate_chunk(coord);
                let client_chunk = client_gen.generate_chunk(coord);

                // Verify block-for-block equality
                for x in 0..CHUNK_SIZE {
                    for y in 0..CHUNK_SIZE {
                        for z in 0..CHUNK_SIZE {
                            assert_eq!(
                                server_chunk.get_block(x, y, z),
                                client_chunk.get_block(x, y, z),
                                "Server/client mismatch at chunk {:?} local ({}, {}, {})",
                                coord,
                                x,
                                y,
                                z
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_integration_with_chunked_world() {
        use crate::world::ChunkedWorld;

        let generator = ChunkGenerator::new(WorldGenConfig::with_seed(12345));
        let mut world = ChunkedWorld::new();

        // Generate and insert chunks using get_or_generate_chunk
        let coords: Vec<ChunkCoord> = (-2..=2)
            .flat_map(|x| (-2..=2).map(move |z| ChunkCoord::new(x, 0, z)))
            .collect();

        for coord in &coords {
            world.get_or_generate_chunk(*coord, |c| generator.generate_chunk(c));
        }

        // Verify all chunks are loaded
        assert_eq!(world.chunk_count(), coords.len());

        // Verify bedrock at y=0
        for coord in &coords {
            let block_pos = crate::types::BlockPos::new(coord.x * 16 + 8, 0, coord.z * 16 + 8);
            assert_eq!(
                world.get_block(block_pos),
                BlockType::BEDROCK,
                "Expected bedrock at y=0 in chunk {:?}",
                coord
            );
        }

        // Verify regenerating same chunk returns identical result (no new generation)
        let test_coord = ChunkCoord::new(0, 0, 0);
        let original_chunk = world.get_chunk(test_coord).unwrap().clone();

        // This should NOT call the generator (chunk already exists)
        world.get_or_generate_chunk(test_coord, |_| {
            panic!("Generator should not be called for existing chunk")
        });

        // Chunk should still be the same
        let after_chunk = world.get_chunk(test_coord).unwrap();
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    assert_eq!(
                        original_chunk.get_block(x, y, z),
                        after_chunk.get_block(x, y, z)
                    );
                }
            }
        }
    }
}
