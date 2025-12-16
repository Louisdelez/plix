//! Biome System
//!
//! Defines biomes and provides per-block biome selection using dual-noise sampling.

use crate::types::BlockType;

use super::noise::NoiseSource;

/// Biome types for terrain generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    /// Flat grasslands with grass surface
    Plains,
    /// High elevation rocky terrain
    Mountains,
    /// Dry sandy terrain
    Desert,
}

impl Biome {
    /// Get the surface block type for this biome
    pub fn surface_block(&self) -> BlockType {
        match self {
            Biome::Plains => BlockType::GRASS,
            Biome::Mountains => BlockType::STONE,
            Biome::Desert => BlockType::SAND,
        }
    }

    /// Get the subsurface block type for this biome
    pub fn subsurface_block(&self) -> BlockType {
        match self {
            Biome::Plains => BlockType::DIRT,
            Biome::Mountains => BlockType::STONE,
            Biome::Desert => BlockType::SANDSTONE,
        }
    }

    /// Get the height amplitude multiplier for this biome
    pub fn height_amplitude(&self) -> f64 {
        match self {
            Biome::Plains => 1.0,
            Biome::Mountains => 2.0,
            Biome::Desert => 0.8,
        }
    }
}

/// Biome selection model using dual-noise
pub struct BiomeModel {
    noise: NoiseSource,
    biome_scale: f64,
}

impl BiomeModel {
    /// Create a new biome model
    pub fn new(seed: u64, biome_scale: f64) -> Self {
        Self {
            noise: NoiseSource::new(seed, 2), // Fewer octaves for smoother biome transitions
            biome_scale,
        }
    }

    /// Determine the biome at world coordinates (x, z)
    ///
    /// Uses dual-noise selection:
    /// - Elevation noise determines mountains vs lowlands
    /// - Temperature noise determines desert vs other biomes
    pub fn biome_at(&self, x: f64, z: f64) -> Biome {
        let elevation = self.noise.sample_biome_elevation(x, z, self.biome_scale);
        let temperature = self.noise.sample_temperature(x, z, self.biome_scale);

        // High elevation = Mountains
        if elevation > 0.3 {
            return Biome::Mountains;
        }

        // High temperature + low elevation = Desert
        if temperature > 0.2 && elevation < 0.1 {
            return Biome::Desert;
        }

        // Default = Plains
        Biome::Plains
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_plains_blocks() {
        let biome = Biome::Plains;
        assert_eq!(biome.surface_block(), BlockType::GRASS);
        assert_eq!(biome.subsurface_block(), BlockType::DIRT);
        assert_eq!(biome.height_amplitude(), 1.0);
    }

    #[test]
    fn test_biome_mountains_blocks() {
        let biome = Biome::Mountains;
        assert_eq!(biome.surface_block(), BlockType::STONE);
        assert_eq!(biome.subsurface_block(), BlockType::STONE);
        assert_eq!(biome.height_amplitude(), 2.0);
    }

    #[test]
    fn test_biome_desert_blocks() {
        let biome = Biome::Desert;
        assert_eq!(biome.surface_block(), BlockType::SAND);
        assert_eq!(biome.subsurface_block(), BlockType::SANDSTONE);
        assert_eq!(biome.height_amplitude(), 0.8);
    }

    #[test]
    fn test_biome_model_determinism() {
        let model1 = BiomeModel::new(12345, 0.005);
        let model2 = BiomeModel::new(12345, 0.005);

        // Same seed should produce same biomes
        for x in -50..50 {
            for z in -50..50 {
                let b1 = model1.biome_at(x as f64, z as f64);
                let b2 = model2.biome_at(x as f64, z as f64);
                assert_eq!(b1, b2, "Biome mismatch at ({}, {})", x, z);
            }
        }
    }

    #[test]
    fn test_biome_variety() {
        // With enough samples, we should see all three biomes
        let model = BiomeModel::new(42, 0.01); // Smaller scale for more variation

        let mut found_plains = false;
        let mut found_mountains = false;
        let mut found_desert = false;

        for x in -200..200 {
            for z in -200..200 {
                match model.biome_at(x as f64 * 10.0, z as f64 * 10.0) {
                    Biome::Plains => found_plains = true,
                    Biome::Mountains => found_mountains = true,
                    Biome::Desert => found_desert = true,
                }
                if found_plains && found_mountains && found_desert {
                    break;
                }
            }
            if found_plains && found_mountains && found_desert {
                break;
            }
        }

        assert!(found_plains, "Should find plains biome");
        assert!(found_mountains, "Should find mountains biome");
        assert!(found_desert, "Should find desert biome");
    }

    #[test]
    fn test_biome_continuity_at_boundaries() {
        let model = BiomeModel::new(42, 0.005);

        // Check that biome doesn't change erratically over small distances
        // (noise is continuous, so biomes should be relatively stable locally)
        let mut changes = 0;
        let mut prev_biome = model.biome_at(0.0, 0.0);

        for i in 1..100 {
            let biome = model.biome_at(i as f64 * 0.5, 0.0);
            if biome != prev_biome {
                changes += 1;
                prev_biome = biome;
            }
        }

        // Should not have too many changes over 50 units
        assert!(
            changes < 20,
            "Too many biome changes ({}) - biomes should be somewhat stable",
            changes
        );
    }
}
