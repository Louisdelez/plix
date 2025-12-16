//! Noise Generation for World Terrain
//!
//! Wraps the noise-rs crate to provide deterministic noise sampling
//! for heightmap and biome generation.

use noise::{NoiseFn, Perlin};

use super::config::derive_seed;

/// Component identifiers for seed derivation
const COMPONENT_HEIGHT: u32 = 0;
const COMPONENT_BIOME_ELEVATION: u32 = 1;
const COMPONENT_TEMPERATURE: u32 = 2;

/// Noise source for world generation
///
/// Provides multiple noise channels for different terrain features,
/// all derived deterministically from a master seed.
pub struct NoiseSource {
    height_noise: Perlin,
    biome_elevation_noise: Perlin,
    temperature_noise: Perlin,
    octaves: u32,
    /// Seed-based offset to ensure different seeds produce different terrain
    seed_offset: f64,
}

impl NoiseSource {
    /// Create a new noise source from a master seed
    ///
    /// # Arguments
    /// * `seed` - Master seed for the world
    /// * `octaves` - Number of octaves for fractal noise (typically 3-4)
    pub fn new(seed: u64, octaves: u32) -> Self {
        let height_noise = Perlin::new(derive_seed(seed, COMPONENT_HEIGHT));
        let biome_elevation_noise = Perlin::new(derive_seed(seed, COMPONENT_BIOME_ELEVATION));
        let temperature_noise = Perlin::new(derive_seed(seed, COMPONENT_TEMPERATURE));

        // Use seed to create an offset for coordinate sampling
        // This ensures different seeds produce completely different terrain
        // The offset is kept in a reasonable range to avoid floating point issues
        // while still providing good differentiation between seeds
        let seed_offset = (seed as f64) % 1_000_000.0;

        Self {
            height_noise,
            biome_elevation_noise,
            temperature_noise,
            octaves,
            seed_offset,
        }
    }

    /// Sample height noise at world coordinates using fBm
    ///
    /// Returns a value in approximately [-1, 1] range
    pub fn sample_height(&self, x: f64, z: f64, scale: f64) -> f64 {
        self.fbm(
            &self.height_noise,
            (x + self.seed_offset) * scale,
            (z + self.seed_offset) * scale,
        )
    }

    /// Sample biome elevation noise at world coordinates
    ///
    /// Returns a value in approximately [-1, 1] range
    pub fn sample_biome_elevation(&self, x: f64, z: f64, scale: f64) -> f64 {
        // Use fewer octaves for biome noise (smoother transitions)
        self.fbm_octaves(
            &self.biome_elevation_noise,
            (x + self.seed_offset) * scale,
            (z + self.seed_offset) * scale,
            2,
        )
    }

    /// Sample temperature noise at world coordinates
    ///
    /// Returns a value in approximately [-1, 1] range
    pub fn sample_temperature(&self, x: f64, z: f64, scale: f64) -> f64 {
        // Use fewer octaves for temperature (smoother transitions)
        self.fbm_octaves(
            &self.temperature_noise,
            (x + self.seed_offset) * scale,
            (z + self.seed_offset) * scale,
            2,
        )
    }

    /// Fractal Brownian motion (fBm) noise using configured octaves
    fn fbm(&self, noise: &Perlin, x: f64, z: f64) -> f64 {
        self.fbm_octaves(noise, x, z, self.octaves)
    }

    /// fBm with specified number of octaves
    fn fbm_octaves(&self, noise: &Perlin, x: f64, z: f64, octaves: u32) -> f64 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            value += noise.get([x * frequency, z * frequency]) * amplitude;
            max_value += amplitude;
            amplitude *= 0.5; // Persistence
            frequency *= 2.0; // Lacunarity
        }

        // Normalize to [-1, 1]
        value / max_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_determinism() {
        // Same seed should produce same values
        let noise1 = NoiseSource::new(12345, 3);
        let noise2 = NoiseSource::new(12345, 3);

        let scale = 0.01;
        for x in 0..10 {
            for z in 0..10 {
                let h1 = noise1.sample_height(x as f64, z as f64, scale);
                let h2 = noise2.sample_height(x as f64, z as f64, scale);
                assert!(
                    (h1 - h2).abs() < 1e-10,
                    "Height mismatch at ({}, {}): {} vs {}",
                    x,
                    z,
                    h1,
                    h2
                );

                let b1 = noise1.sample_biome_elevation(x as f64, z as f64, scale);
                let b2 = noise2.sample_biome_elevation(x as f64, z as f64, scale);
                assert!(
                    (b1 - b2).abs() < 1e-10,
                    "Biome elevation mismatch at ({}, {})",
                    x,
                    z
                );

                let t1 = noise1.sample_temperature(x as f64, z as f64, scale);
                let t2 = noise2.sample_temperature(x as f64, z as f64, scale);
                assert!(
                    (t1 - t2).abs() < 1e-10,
                    "Temperature mismatch at ({}, {})",
                    x,
                    z
                );
            }
        }
    }

    #[test]
    fn test_noise_different_seeds() {
        let noise1 = NoiseSource::new(12345, 3);
        let noise2 = NoiseSource::new(54321, 3);

        // Different seeds should produce different values at most locations
        // Sample multiple points and count differences
        let mut different_count = 0;
        for x in 0..10 {
            for z in 0..10 {
                let h1 = noise1.sample_height(x as f64 * 100.0, z as f64 * 100.0, 0.01);
                let h2 = noise2.sample_height(x as f64 * 100.0, z as f64 * 100.0, 0.01);
                if (h1 - h2).abs() > 0.01 {
                    different_count += 1;
                }
            }
        }
        assert!(
            different_count > 50,
            "Different seeds should produce different values at most locations (got {} different out of 100)",
            different_count
        );
    }

    #[test]
    fn test_noise_range() {
        let noise = NoiseSource::new(42, 3);
        let scale = 0.01;

        // Sample many points and verify values are in expected range
        for x in -100..100 {
            for z in -100..100 {
                let h = noise.sample_height(x as f64, z as f64, scale);
                assert!(
                    (-1.5..=1.5).contains(&h),
                    "Height {} out of expected range at ({}, {})",
                    h,
                    x,
                    z
                );
            }
        }
    }

    #[test]
    fn test_noise_continuity() {
        let noise = NoiseSource::new(42, 3);
        let scale = 0.01;

        // Check that adjacent samples are similar (continuity)
        for x in 0..10 {
            for z in 0..10 {
                let h1 = noise.sample_height(x as f64, z as f64, scale);
                let h2 = noise.sample_height(x as f64 + 0.1, z as f64, scale);
                let diff = (h1 - h2).abs();
                assert!(
                    diff < 0.5,
                    "Noise should be continuous, but diff={} at ({}, {})",
                    diff,
                    x,
                    z
                );
            }
        }
    }

    #[test]
    fn test_noise_seed_edge_cases() {
        // Test with edge case seeds
        let _ = NoiseSource::new(0, 3);
        let _ = NoiseSource::new(u64::MAX, 3);

        // These should not panic and should produce valid noise
        let noise = NoiseSource::new(0, 3);
        let h = noise.sample_height(0.0, 0.0, 0.01);
        assert!(h.is_finite());

        let noise = NoiseSource::new(u64::MAX, 3);
        let h = noise.sample_height(0.0, 0.0, 0.01);
        assert!(h.is_finite());
    }
}
