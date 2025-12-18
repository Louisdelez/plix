//! Melee combat system with cone hit detection.
//!
//! Performs hit detection using a cone from the attacker's position and facing direction.

use glam::Vec3;
use plix_common::types::PlayerId;

use super::defs::WeaponDef;

/// Result of a melee attack attempt.
#[derive(Debug, Clone)]
pub struct MeleeHitResult {
    /// Players hit by the attack
    pub hit_players: Vec<PlayerId>,
    /// Damage dealt per hit
    pub damage: u16,
}

/// Target information for hit detection.
#[derive(Debug, Clone, Copy)]
pub struct MeleeTarget {
    /// Target entity ID
    pub id: PlayerId,
    /// Target position (center mass)
    pub position: Vec3,
    /// Target collision radius (capsule radius)
    pub radius: f32,
}

/// Melee combat system.
pub struct MeleeSystem;

impl MeleeSystem {
    /// Check if a target is within the attack cone.
    ///
    /// Uses dot product to check angle and distance for cone hit detection.
    ///
    /// # Arguments
    /// * `attacker_pos` - Position of the attacker (eye level)
    /// * `attacker_forward` - Normalized direction the attacker is facing
    /// * `target_pos` - Position of the target (center mass)
    /// * `target_radius` - Collision radius of the target
    /// * `cone_half_angle_deg` - Half-angle of the attack cone in degrees
    /// * `reach_blocks` - Maximum reach distance in blocks
    pub fn is_in_cone(
        attacker_pos: Vec3,
        attacker_forward: Vec3,
        target_pos: Vec3,
        target_radius: f32,
        cone_half_angle_deg: f32,
        reach_blocks: f32,
    ) -> bool {
        let to_target = target_pos - attacker_pos;
        let distance = to_target.length();

        // Check distance (account for target radius)
        if distance > reach_blocks + target_radius {
            return false;
        }

        // Check angle
        if distance < 0.001 {
            // Target at same position, consider it a hit
            return true;
        }

        let to_target_normalized = to_target / distance;
        let cos_angle = attacker_forward.dot(to_target_normalized);

        // Convert half-angle to cosine threshold
        let cos_threshold = cone_half_angle_deg.to_radians().cos();

        cos_angle >= cos_threshold
    }

    /// Perform melee attack and return hit results.
    ///
    /// # Arguments
    /// * `attacker_id` - ID of the attacking player
    /// * `attacker_pos` - Position of the attacker (eye level)
    /// * `attacker_forward` - Normalized direction the attacker is facing
    /// * `targets` - Iterator of potential targets
    /// * `weapon` - Weapon definition with cone parameters
    pub fn attack<'a, I>(
        attacker_id: PlayerId,
        attacker_pos: Vec3,
        attacker_forward: Vec3,
        targets: I,
        weapon: &WeaponDef,
    ) -> MeleeHitResult
    where
        I: Iterator<Item = MeleeTarget>,
    {
        let mut hit_players = Vec::new();

        for target in targets {
            // Don't hit yourself
            if target.id == attacker_id {
                continue;
            }

            if Self::is_in_cone(
                attacker_pos,
                attacker_forward,
                target.position,
                target.radius,
                weapon.cone_angle_deg,
                weapon.reach_blocks,
            ) {
                hit_players.push(target.id);
            }
        }

        MeleeHitResult {
            hit_players,
            damage: weapon.damage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAYER_RADIUS: f32 = 0.4;

    #[test]
    fn test_cone_hit_directly_ahead() {
        let attacker_pos = Vec3::new(0.0, 1.0, 0.0);
        let forward = Vec3::new(0.0, 0.0, 1.0);
        let target_pos = Vec3::new(0.0, 1.0, 2.0);

        assert!(MeleeSystem::is_in_cone(
            attacker_pos,
            forward,
            target_pos,
            PLAYER_RADIUS,
            30.0,
            2.5
        ));
    }

    #[test]
    fn test_cone_miss_behind() {
        let attacker_pos = Vec3::new(0.0, 1.0, 0.0);
        let forward = Vec3::new(0.0, 0.0, 1.0);
        let target_pos = Vec3::new(0.0, 1.0, -2.0); // Behind

        assert!(!MeleeSystem::is_in_cone(
            attacker_pos,
            forward,
            target_pos,
            PLAYER_RADIUS,
            30.0,
            2.5
        ));
    }

    #[test]
    fn test_cone_miss_too_far() {
        let attacker_pos = Vec3::new(0.0, 1.0, 0.0);
        let forward = Vec3::new(0.0, 0.0, 1.0);
        let target_pos = Vec3::new(0.0, 1.0, 5.0); // Too far

        assert!(!MeleeSystem::is_in_cone(
            attacker_pos,
            forward,
            target_pos,
            PLAYER_RADIUS,
            30.0,
            2.5
        ));
    }

    #[test]
    fn test_cone_hit_at_edge_distance() {
        let attacker_pos = Vec3::new(0.0, 1.0, 0.0);
        let forward = Vec3::new(0.0, 0.0, 1.0);
        // Distance = 2.5, but with radius 0.4, effective distance = 2.9
        let target_pos = Vec3::new(0.0, 1.0, 2.9);

        assert!(MeleeSystem::is_in_cone(
            attacker_pos,
            forward,
            target_pos,
            PLAYER_RADIUS,
            30.0,
            2.5
        ));
    }

    #[test]
    fn test_cone_hit_at_angle_edge() {
        let attacker_pos = Vec3::new(0.0, 1.0, 0.0);
        let forward = Vec3::new(0.0, 0.0, 1.0);
        // Target at 25 degrees should hit (within 30 degree half-angle)
        let angle_rad = 25.0_f32.to_radians();
        let target_pos = Vec3::new(angle_rad.sin() * 2.0, 1.0, angle_rad.cos() * 2.0);

        assert!(MeleeSystem::is_in_cone(
            attacker_pos,
            forward,
            target_pos,
            PLAYER_RADIUS,
            30.0,
            2.5
        ));
    }

    #[test]
    fn test_cone_miss_outside_angle() {
        let attacker_pos = Vec3::new(0.0, 1.0, 0.0);
        let forward = Vec3::new(0.0, 0.0, 1.0);
        // Target at 45 degrees should miss (outside 30 degree half-angle)
        let angle_rad = 45.0_f32.to_radians();
        let target_pos = Vec3::new(angle_rad.sin() * 2.0, 1.0, angle_rad.cos() * 2.0);

        assert!(!MeleeSystem::is_in_cone(
            attacker_pos,
            forward,
            target_pos,
            PLAYER_RADIUS,
            30.0,
            2.5
        ));
    }

    #[test]
    fn test_attack_hits_multiple() {
        use crate::weapons::SWORD_DEF;

        let attacker_pos = Vec3::new(0.0, 1.0, 0.0);
        let forward = Vec3::new(0.0, 0.0, 1.0);

        let targets = vec![
            MeleeTarget {
                id: PlayerId(1),
                position: Vec3::new(0.0, 1.0, 2.0),
                radius: PLAYER_RADIUS,
            },
            MeleeTarget {
                id: PlayerId(2),
                position: Vec3::new(0.5, 1.0, 1.5),
                radius: PLAYER_RADIUS,
            },
            MeleeTarget {
                id: PlayerId(3),
                position: Vec3::new(0.0, 1.0, 10.0), // Too far
                radius: PLAYER_RADIUS,
            },
        ];

        let result = MeleeSystem::attack(
            PlayerId(0),
            attacker_pos,
            forward,
            targets.into_iter(),
            &SWORD_DEF,
        );

        assert_eq!(result.hit_players.len(), 2);
        assert!(result.hit_players.contains(&PlayerId(1)));
        assert!(result.hit_players.contains(&PlayerId(2)));
        assert_eq!(result.damage, 25);
    }

    #[test]
    fn test_attack_no_self_hit() {
        use crate::weapons::SWORD_DEF;

        let attacker_pos = Vec3::new(0.0, 1.0, 0.0);
        let forward = Vec3::new(0.0, 0.0, 1.0);

        let targets = vec![MeleeTarget {
            id: PlayerId(0), // Same as attacker
            position: attacker_pos,
            radius: PLAYER_RADIUS,
        }];

        let result = MeleeSystem::attack(
            PlayerId(0),
            attacker_pos,
            forward,
            targets.into_iter(),
            &SWORD_DEF,
        );

        assert!(result.hit_players.is_empty());
    }
}
