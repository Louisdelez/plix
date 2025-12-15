//! Combat system

use plix_common::combat::CombatConfig;
use plix_common::math::Vec3;
use plix_common::time::Tick;
use plix_common::types::PlayerId;

// Import unified hitbox dimensions from collision module
// These MUST match the collision capsule for consistent hit detection
use super::collision::{PLAYER_HALF_HEIGHT, PLAYER_HALF_WIDTH};

/// Damage dealt by melee attack
pub const MELEE_DAMAGE: u8 = 20;

/// Combat hit result
#[derive(Debug, Clone)]
pub struct HitResult {
    /// Attacker ID
    pub attacker: PlayerId,
    /// Target ID
    pub target: PlayerId,
    /// Damage dealt
    pub damage: u8,
    /// Whether target died
    pub killed: bool,
    /// Knockback direction (normalized, attacker -> victim)
    pub knockback_dir: Vec3,
}

/// Check if a point is within the player's hitbox capsule
/// This uses the same dimensions as the collision capsule
/// Position is feet position, hitbox extends from feet to feet + PLAYER_HALF_HEIGHT * 2
#[inline]
pub fn point_in_player_hitbox(point: Vec3, player_pos: Vec3) -> bool {
    // Horizontal distance (capsule cylinder check)
    let dx = point.x - player_pos.x;
    let dz = point.z - player_pos.z;
    let horiz_dist_sq = dx * dx + dz * dz;

    if horiz_dist_sq > PLAYER_HALF_WIDTH * PLAYER_HALF_WIDTH {
        return false;
    }

    // Vertical bounds check
    let player_top = player_pos.y + PLAYER_HALF_HEIGHT * 2.0;
    point.y >= player_pos.y && point.y <= player_top
}

/// Calculate the closest distance between a ray and a player capsule
/// Returns the distance to the closest point on the player's hitbox
#[inline]
pub fn distance_to_player_hitbox(from: Vec3, player_pos: Vec3) -> f32 {
    // For melee combat, we use center-to-center distance minus radius
    // Player center is at feet + half_height
    let player_center = Vec3::new(
        player_pos.x,
        player_pos.y + PLAYER_HALF_HEIGHT,
        player_pos.z,
    );

    let to_player = player_center - from;

    // Horizontal distance minus width (edge-to-edge)
    let horiz_dist = (to_player.x * to_player.x + to_player.z * to_player.z).sqrt();
    let edge_horiz = (horiz_dist - PLAYER_HALF_WIDTH).max(0.0);

    // Vertical distance (if above/below player)
    let vert_dist = if from.y < player_pos.y {
        player_pos.y - from.y
    } else if from.y > player_pos.y + PLAYER_HALF_HEIGHT * 2.0 {
        from.y - (player_pos.y + PLAYER_HALF_HEIGHT * 2.0)
    } else {
        0.0
    };

    // Combined 3D distance
    (edge_horiz * edge_horiz + vert_dist * vert_dist).sqrt()
}

/// Combat system for processing attacks
#[derive(Debug, Default)]
pub struct CombatSystem;

impl CombatSystem {
    /// Create a new combat system
    pub fn new() -> Self {
        Self
    }

    /// Try to perform a melee attack
    pub fn try_attack(
        &self,
        config: &CombatConfig,
        attacker_id: PlayerId,
        attacker_pos: Vec3,
        attacker_forward: Vec3,
        last_attack_tick: Tick,
        current_tick: Tick,
        targets: &[(PlayerId, Vec3, u8)], // (id, pos, health)
    ) -> Option<(PlayerId, HitResult)> {
        // Check cooldown using config value
        let ticks_since_attack = current_tick.0.wrapping_sub(last_attack_tick.0);
        if ticks_since_attack < config.attack_cooldown_ticks {
            return None;
        }

        // Use effective range (base range + epsilon for latency tolerance)
        let effective_range = config.effective_range();

        // Find closest target in range and in front
        let mut best_target: Option<(PlayerId, Vec3, u8)> = None;
        let mut best_dist = effective_range;

        for (target_id, target_pos, health) in targets {
            if *target_id == attacker_id || *health == 0 {
                continue;
            }

            let to_target = *target_pos - attacker_pos;
            let distance = to_target.length();

            if distance > effective_range {
                continue;
            }

            // Check if target is roughly in front (within 90 degree cone)
            let to_target_norm = to_target.normalize_or_zero();
            let dot = attacker_forward.dot(to_target_norm);

            if dot < 0.0 {
                // Behind attacker
                continue;
            }

            if distance < best_dist {
                best_dist = distance;
                best_target = Some((*target_id, *target_pos, *health));
            }
        }

        if let Some((target_id, target_pos, health)) = best_target {
            let killed = health <= MELEE_DAMAGE;
            let damage = MELEE_DAMAGE.min(health);

            // Calculate knockback direction (attacker -> victim, normalized)
            let knockback_dir = (target_pos - attacker_pos).normalize_or_zero();

            Some((
                target_id,
                HitResult {
                    attacker: attacker_id,
                    target: target_id,
                    damage,
                    killed,
                    knockback_dir,
                },
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_melee_hit() {
        let system = CombatSystem::new();
        let config = CombatConfig::default();

        let attacker_id = PlayerId(1);
        let target_id = PlayerId(2);

        let attacker_pos = Vec3::ZERO;
        let attacker_forward = Vec3::new(0.0, 0.0, 1.0);

        let targets = vec![
            (target_id, Vec3::new(0.0, 0.0, 1.5), 100), // In range
        ];

        let result = system.try_attack(
            &config,
            attacker_id,
            attacker_pos,
            attacker_forward,
            Tick(0),
            Tick(100),
            &targets,
        );

        assert!(result.is_some());
        let (hit_id, hit) = result.unwrap();
        assert_eq!(hit_id, target_id);
        assert_eq!(hit.damage, MELEE_DAMAGE);
        assert!(!hit.killed);
    }

    #[test]
    fn test_melee_out_of_range() {
        let system = CombatSystem::new();
        let config = CombatConfig::default();

        let targets = vec![
            (PlayerId(2), Vec3::new(0.0, 0.0, 10.0), 100), // Out of range
        ];

        let result = system.try_attack(
            &config,
            PlayerId(1),
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 1.0),
            Tick(0),
            Tick(100),
            &targets,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_melee_cooldown() {
        let system = CombatSystem::new();
        let config = CombatConfig::default();

        let targets = vec![(PlayerId(2), Vec3::new(0.0, 0.0, 1.5), 100)];

        // Attack on cooldown (5 ticks since last attack, need 30)
        let result = system.try_attack(
            &config,
            PlayerId(1),
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 1.0),
            Tick(95), // Recent attack
            Tick(100),
            &targets,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_melee_kill() {
        let system = CombatSystem::new();
        let config = CombatConfig::default();

        let targets = vec![
            (PlayerId(2), Vec3::new(0.0, 0.0, 1.5), 15), // Low health
        ];

        let result = system.try_attack(
            &config,
            PlayerId(1),
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 1.0),
            Tick(0),
            Tick(100),
            &targets,
        );

        assert!(result.is_some());
        let (_, hit) = result.unwrap();
        assert!(hit.killed);
        assert_eq!(hit.damage, 15);
    }

    // ============================================================================
    // US5 - Stable Hitbox Tests
    // ============================================================================

    /// T049, T051 [US5] Hitbox matches collision capsule dimensions
    #[test]
    fn test_hitbox_matches_collision_capsule() {
        // Verify hitbox uses the same constants as collision
        // PLAYER_HALF_WIDTH should be 0.4 (same as movement PLAYER_RADIUS)
        assert_eq!(
            PLAYER_HALF_WIDTH, 0.4,
            "Hitbox width should match collision"
        );
        // PLAYER_HALF_HEIGHT should be 0.9 (1.8m / 2)
        assert_eq!(
            PLAYER_HALF_HEIGHT, 0.9,
            "Hitbox height should match collision"
        );
    }

    /// T051 [US5] Point inside player hitbox is detected
    #[test]
    fn test_point_in_hitbox() {
        let player_pos = Vec3::new(5.0, 1.0, 5.0);

        // Point at player center (should be inside)
        let center = Vec3::new(5.0, 1.0 + PLAYER_HALF_HEIGHT, 5.0);
        assert!(
            point_in_player_hitbox(center, player_pos),
            "Center point should be inside hitbox"
        );

        // Point at player feet (should be inside)
        let feet = Vec3::new(5.0, 1.0, 5.0);
        assert!(
            point_in_player_hitbox(feet, player_pos),
            "Feet point should be inside hitbox"
        );

        // Point at player head (should be inside)
        let head = Vec3::new(5.0, 1.0 + PLAYER_HALF_HEIGHT * 2.0 - 0.1, 5.0);
        assert!(
            point_in_player_hitbox(head, player_pos),
            "Head point should be inside hitbox"
        );
    }

    /// T051 [US5] Point outside player hitbox is not detected
    #[test]
    fn test_point_outside_hitbox() {
        let player_pos = Vec3::new(5.0, 1.0, 5.0);

        // Point well outside (too far horizontally)
        let far = Vec3::new(6.0, 1.5, 5.0);
        assert!(
            !point_in_player_hitbox(far, player_pos),
            "Far point should be outside hitbox"
        );

        // Point below feet
        let below = Vec3::new(5.0, 0.5, 5.0);
        assert!(
            !point_in_player_hitbox(below, player_pos),
            "Below point should be outside hitbox"
        );

        // Point above head
        let above = Vec3::new(5.0, 3.0, 5.0);
        assert!(
            !point_in_player_hitbox(above, player_pos),
            "Above point should be outside hitbox"
        );
    }

    /// T051 [US5] Distance to hitbox is calculated correctly
    #[test]
    fn test_distance_to_hitbox() {
        let player_pos = Vec3::new(5.0, 1.0, 5.0);

        // Attacker at same position - distance should be 0
        let same_pos = Vec3::new(5.0, 1.0 + PLAYER_HALF_HEIGHT, 5.0);
        let dist = distance_to_player_hitbox(same_pos, player_pos);
        assert!(
            dist < 0.1,
            "Distance from center should be ~0, got {}",
            dist
        );

        // Attacker 2m away horizontally - distance should be ~1.6 (2.0 - 0.4 width)
        let far = Vec3::new(7.0, 1.0 + PLAYER_HALF_HEIGHT, 5.0);
        let dist = distance_to_player_hitbox(far, player_pos);
        assert!(
            (dist - 1.6).abs() < 0.1,
            "Horizontal distance should be ~1.6, got {}",
            dist
        );
    }
}
