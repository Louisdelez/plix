//! World Generation Configuration
//!
//! Defines all parameters for procedural terrain generation.

use serde::{Deserialize, Serialize};

/// Configuration for procedural world generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldGenConfig {
    /// Master seed for world generation (u64)
    pub seed: u64,
    /// Minimum terrain height in blocks
    pub min_height: i32,
    /// Maximum terrain height in blocks
    pub max_height: i32,
    /// Noise scale for height sampling (smaller = larger features)
    pub height_scale: f64,
    /// Number of octaves for fractal noise
    pub height_octaves: u32,
    /// Noise scale for biome sampling
    pub biome_scale: f64,
    /// Number of layers below surface (subsurface depth)
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
            biome_scale: 0.005,
            subsurface_depth: 3,
        }
    }
}

impl WorldGenConfig {
    /// Create a new configuration with the given seed
    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed,
            ..Default::default()
        }
    }
}

/// Derive a component-specific seed from the master seed
///
/// Uses fixed offsets to ensure different noise sources produce different patterns
/// while remaining deterministic.
///
/// # Arguments
/// * `master_seed` - The world's master seed
/// * `component` - A unique component identifier (e.g., 0 for height, 1 for biome)
///
/// # Returns
/// A deterministic u32 seed for the specific component
pub fn derive_seed(master_seed: u64, component: u32) -> u32 {
    // Use different parts of the master seed combined with component
    // This ensures each component gets a unique but deterministic seed
    let high = (master_seed >> 32) as u32;
    let low = master_seed as u32;
    // XOR and rotate to mix bits
    high.wrapping_add(low)
        .wrapping_mul(2654435761)
        .wrapping_add(component)
}

/// Metrics for generation performance tracking
#[derive(Debug, Clone, Default)]
pub struct GenerationMetrics {
    /// Total number of chunks generated
    pub chunks_generated: u64,
    /// Total time spent generating (in microseconds)
    pub total_generation_time_us: u64,
}

impl GenerationMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a chunk generation
    pub fn record_generation(&mut self, duration_us: u64) {
        self.chunks_generated += 1;
        self.total_generation_time_us += duration_us;
    }

    /// Get average generation time in microseconds
    pub fn average_generation_time_us(&self) -> f64 {
        if self.chunks_generated == 0 {
            0.0
        } else {
            self.total_generation_time_us as f64 / self.chunks_generated as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WorldGenConfig::default();
        assert_eq!(config.seed, 0);
        assert_eq!(config.min_height, 32);
        assert_eq!(config.max_height, 96);
        assert_eq!(config.height_octaves, 3);
        assert_eq!(config.subsurface_depth, 3);
    }

    #[test]
    fn test_derive_seed_determinism() {
        // Same inputs should produce same output
        let seed1 = derive_seed(12345, 0);
        let seed2 = derive_seed(12345, 0);
        assert_eq!(seed1, seed2);

        // Different components should produce different seeds
        let seed_height = derive_seed(12345, 0);
        let seed_biome = derive_seed(12345, 1);
        assert_ne!(seed_height, seed_biome);

        // Different master seeds should produce different component seeds
        let seed_a = derive_seed(12345, 0);
        let seed_b = derive_seed(54321, 0);
        assert_ne!(seed_a, seed_b);
    }

    #[test]
    fn test_derive_seed_edge_cases() {
        // Test with extreme seed values
        let _ = derive_seed(0, 0);
        let _ = derive_seed(u64::MAX, 0);
        let _ = derive_seed(u64::MAX, u32::MAX);
    }

    #[test]
    fn test_metrics() {
        let mut metrics = GenerationMetrics::new();
        assert_eq!(metrics.chunks_generated, 0);
        assert_eq!(metrics.average_generation_time_us(), 0.0);

        metrics.record_generation(1000);
        metrics.record_generation(2000);
        assert_eq!(metrics.chunks_generated, 2);
        assert_eq!(metrics.average_generation_time_us(), 1500.0);
    }
}
