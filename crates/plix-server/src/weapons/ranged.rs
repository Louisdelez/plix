//! Ranged combat system with spread calculation.
//!
//! Handles spread application to projectile direction based on accuracy modifiers.

use glam::Vec3;
use plix_common::time::Tick;

use super::defs::WeaponDef;
use super::recoil::RecoilState;

/// Ranged combat system.
pub struct RangedSystem;

impl RangedSystem {
    /// Apply spread to a direction vector.
    ///
    /// Uses a deterministic random offset based on tick for reproducibility.
    ///
    /// # Arguments
    /// * `direction` - Base direction (normalized)
    /// * `spread_deg` - Spread angle in degrees
    /// * `seed` - Random seed (typically current tick)
    pub fn apply_spread(direction: Vec3, spread_deg: f32, seed: u32) -> Vec3 {
        if spread_deg <= 0.0 {
            return direction;
        }

        // Generate pseudo-random angles using simple hash
        let hash1 = Self::simple_hash(seed);
        let hash2 = Self::simple_hash(seed.wrapping_add(12345));

        // Random angle around direction (0 to 2π)
        let rotation_angle = (hash1 as f32 / u32::MAX as f32) * std::f32::consts::TAU;

        // Random offset within spread cone (0 to spread_deg)
        let offset_deg = (hash2 as f32 / u32::MAX as f32) * spread_deg;
        let offset_rad = offset_deg.to_radians();

        // Find perpendicular vectors to direction
        let (right, up) = Self::perpendicular_vectors(direction);

        // Calculate offset vector
        let offset = right * rotation_angle.cos() * offset_rad.sin()
            + up * rotation_angle.sin() * offset_rad.sin()
            + direction * offset_rad.cos();

        offset.normalize()
    }

    /// Calculate the final spread amount including all modifiers.
    pub fn calculate_spread(
        weapon: &WeaponDef,
        recoil: &RecoilState,
        is_moving: bool,
        current_tick: Tick,
    ) -> f32 {
        recoil.get_effective_spread(
            weapon.base_spread_deg,
            is_moving,
            weapon.movement_spread_mult,
            current_tick,
        )
    }

    /// Calculate initial projectile velocity.
    pub fn calculate_velocity(direction: Vec3, speed_blocks_per_sec: f32) -> Vec3 {
        // Convert to blocks per tick (60 TPS)
        let speed_per_tick = speed_blocks_per_sec / 60.0;
        direction * speed_per_tick
    }

    /// Simple deterministic hash function for spread randomization.
    fn simple_hash(seed: u32) -> u32 {
        let mut x = seed;
        x = x.wrapping_mul(0x45d9f3b);
        x = (x >> 16) ^ x;
        x = x.wrapping_mul(0x45d9f3b);
        x = (x >> 16) ^ x;
        x
    }

    /// Get two perpendicular vectors to the given direction.
    fn perpendicular_vectors(direction: Vec3) -> (Vec3, Vec3) {
        // Choose a vector that isn't parallel to direction
        let arbitrary = if direction.y.abs() < 0.9 {
            Vec3::Y
        } else {
            Vec3::X
        };

        let right = direction.cross(arbitrary).normalize();
        let up = right.cross(direction).normalize();

        (right, up)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_spread_zero() {
        let dir = Vec3::new(0.0, 0.0, 1.0);
        let result = RangedSystem::apply_spread(dir, 0.0, 100);
        assert!((result - dir).length() < 0.001);
    }

    #[test]
    fn test_apply_spread_deterministic() {
        let dir = Vec3::new(0.0, 0.0, 1.0);
        let result1 = RangedSystem::apply_spread(dir, 5.0, 100);
        let result2 = RangedSystem::apply_spread(dir, 5.0, 100);
        assert!((result1 - result2).length() < 0.001);
    }

    #[test]
    fn test_apply_spread_different_seeds() {
        let dir = Vec3::new(0.0, 0.0, 1.0);
        let result1 = RangedSystem::apply_spread(dir, 5.0, 100);
        let result2 = RangedSystem::apply_spread(dir, 5.0, 200);
        // Should be different (with high probability)
        assert!((result1 - result2).length() > 0.001);
    }

    #[test]
    fn test_apply_spread_within_cone() {
        let dir = Vec3::new(0.0, 0.0, 1.0);
        let spread_deg = 10.0;

        // Test multiple seeds, all should be within spread cone
        for seed in 0..100 {
            let result = RangedSystem::apply_spread(dir, spread_deg, seed);

            // Resulting direction should be within spread angle of original
            let angle = dir.dot(result).acos().to_degrees();
            assert!(
                angle <= spread_deg,
                "Spread produced angle {} > {} degrees",
                angle,
                spread_deg
            );
        }
    }

    #[test]
    fn test_calculate_velocity() {
        let dir = Vec3::new(0.0, 0.0, 1.0);
        let speed = 30.0; // 30 blocks/sec

        let velocity = RangedSystem::calculate_velocity(dir, speed);

        // At 60 TPS, velocity should be 0.5 blocks/tick
        assert!((velocity.z - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_calculate_spread_standing() {
        use crate::weapons::BOW_DEF;

        let recoil = RecoilState::default();
        let spread = RangedSystem::calculate_spread(&BOW_DEF, &recoil, false, Tick(0));

        // Should be base spread only
        assert_eq!(spread, 2.0);
    }

    #[test]
    fn test_calculate_spread_moving() {
        use crate::weapons::BOW_DEF;

        let recoil = RecoilState::default();
        let spread = RangedSystem::calculate_spread(&BOW_DEF, &recoil, true, Tick(0));

        // Should be base spread * movement multiplier
        assert_eq!(spread, 3.0); // 2.0 * 1.5
    }

    #[test]
    fn test_perpendicular_vectors() {
        let dir = Vec3::new(0.0, 0.0, 1.0);
        let (right, up) = RangedSystem::perpendicular_vectors(dir);

        // All should be unit vectors
        assert!((right.length() - 1.0).abs() < 0.001);
        assert!((up.length() - 1.0).abs() < 0.001);

        // All should be perpendicular
        assert!(dir.dot(right).abs() < 0.001);
        assert!(dir.dot(up).abs() < 0.001);
        assert!(right.dot(up).abs() < 0.001);
    }
}
