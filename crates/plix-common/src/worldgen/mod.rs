//! Procedural World Generation
//!
//! This module provides deterministic world generation from a seed.
//! Key features:
//! - Noise-based heightmap terrain
//! - Three biomes: Plains, Mountains, Desert
//! - Layer-based block placement (surface, subsurface, stone, bedrock)
//! - Per-chunk independent generation (no neighbor dependencies)
//!
//! # Example
//!
//! ```ignore
//! use plix_common::worldgen::{ChunkGenerator, WorldGenConfig};
//! use plix_common::chunk::ChunkCoord;
//!
//! let config = WorldGenConfig { seed: 12345, ..Default::default() };
//! let generator = ChunkGenerator::new(config);
//! let chunk = generator.generate_chunk(ChunkCoord::new(0, 0, 0));
//! ```

mod biome;
mod config;
mod generator;
mod height;
mod noise;

pub use biome::{Biome, BiomeModel};
pub use config::{GenerationMetrics, WorldGenConfig};
pub use generator::ChunkGenerator;
pub use height::HeightModel;
pub use noise::NoiseSource;
