//! Height Model
//!
//! Provides heightmap generation using noise with biome-aware amplitude.

use super::biome::Biome;
use super::config::WorldGenConfig;
use super::noise::NoiseSource;

/// Height model for terrain generation
pub struct HeightModel {
    noise: NoiseSource,
    min_height: i32,
    max_height: i32,
    height_scale: f64,
}

impl HeightModel {
    /// Create a new height model from configuration
    pub fn new(config: &WorldGenConfig) -> Self {
        Self {
            noise: NoiseSource::new(config.seed, config.height_octaves),
            min_height: config.min_height,
            max_height: config.max_height,
            height_scale: config.height_scale,
        }
    }

    /// Calculate the surface height at world coordinates
    ///
    /// # Arguments
    /// * `x` - World X coordinate
    /// * `z` - World Z coordinate
    /// * `biome` - The biome at this location (affects amplitude)
    ///
    /// # Returns
    /// The surface height in blocks
    pub fn surface_height(&self, x: f64, z: f64, biome: Biome) -> i32 {
        // Sample base noise
        let noise_value = self.noise.sample_height(x, z, self.height_scale);

        // Apply biome amplitude
        let amplitude = biome.height_amplitude();
        let adjusted_noise = noise_value * amplitude;

        // Map from [-1, 1] to [min_height, max_height]
        let height_range = (self.max_height - self.min_height) as f64;
        let mid_height = (self.min_height + self.max_height) as f64 / 2.0;

        let height = mid_height + adjusted_noise * height_range / 2.0;

        // Clamp to valid range
        height.round() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heightmap_range() {
        let config = WorldGenConfig::default();
        let height_model = HeightModel::new(&config);

        // Sample many points and verify heights are in range
        for x in -100..100 {
            for z in -100..100 {
                for biome in [Biome::Plains, Biome::Mountains, Biome::Desert] {
                    let height = height_model.surface_height(x as f64, z as f64, biome);

                    // Mountains can exceed normal range due to 2x amplitude
                    let effective_max = if biome == Biome::Mountains {
                        config.max_height + (config.max_height - config.min_height) / 2
                    } else {
                        config.max_height
                    };

                    assert!(
                        height >= config.min_height && height <= effective_max,
                        "Height {} out of range [{}, {}] at ({}, {}) in {:?}",
                        height,
                        config.min_height,
                        effective_max,
                        x,
                        z,
                        biome
                    );
                }
            }
        }
    }

    #[test]
    fn test_heightmap_continuity() {
        let config = WorldGenConfig::default();
        let height_model = HeightModel::new(&config);
        let biome = Biome::Plains;

        // Adjacent heights should be similar (continuous terrain)
        for x in 0..100 {
            for z in 0..100 {
                let h1 = height_model.surface_height(x as f64, z as f64, biome);
                let h2 = height_model.surface_height(x as f64 + 1.0, z as f64, biome);
                let h3 = height_model.surface_height(x as f64, z as f64 + 1.0, biome);

                // Height difference should be reasonable (not more than a few blocks)
                assert!(
                    (h1 - h2).abs() <= 5,
                    "Height discontinuity in X at ({}, {}): {} vs {}",
                    x,
                    z,
                    h1,
                    h2
                );
                assert!(
                    (h1 - h3).abs() <= 5,
                    "Height discontinuity in Z at ({}, {}): {} vs {}",
                    x,
                    z,
                    h1,
                    h3
                );
            }
        }
    }

    #[test]
    fn test_heightmap_determinism() {
        let config = WorldGenConfig::default();
        let model1 = HeightModel::new(&config);
        let model2 = HeightModel::new(&config);

        for x in -50..50 {
            for z in -50..50 {
                let h1 = model1.surface_height(x as f64, z as f64, Biome::Plains);
                let h2 = model2.surface_height(x as f64, z as f64, Biome::Plains);
                assert_eq!(h1, h2, "Height mismatch at ({}, {})", x, z);
            }
        }
    }

    #[test]
    fn test_biome_amplitude_effect() {
        let config = WorldGenConfig::default();
        let height_model = HeightModel::new(&config);

        // Mountains should have more extreme heights
        let mut plains_variance = 0.0;
        let mut mountain_variance = 0.0;
        let mid_height = (config.min_height + config.max_height) as f64 / 2.0;

        for x in 0..100 {
            for z in 0..100 {
                let plains_h = height_model.surface_height(x as f64, z as f64, Biome::Plains);
                let mountain_h = height_model.surface_height(x as f64, z as f64, Biome::Mountains);

                plains_variance += (plains_h as f64 - mid_height).powi(2);
                mountain_variance += (mountain_h as f64 - mid_height).powi(2);
            }
        }

        // Mountains should have higher variance (more extreme heights)
        assert!(
            mountain_variance > plains_variance,
            "Mountains should have more height variation than plains"
        );
    }
}
