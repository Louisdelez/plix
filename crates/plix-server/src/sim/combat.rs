//! Combat system

use plix_common::math::Vec3;
use plix_common::time::Tick;
use plix_common::types::PlayerId;

use crate::validation::{ATTACK_COOLDOWN_TICKS, ATTACK_RANGE};

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
        attacker_id: PlayerId,
        attacker_pos: Vec3,
        attacker_forward: Vec3,
        last_attack_tick: Tick,
        current_tick: Tick,
        targets: &[(PlayerId, Vec3, u8)], // (id, pos, health)
    ) -> Option<(PlayerId, HitResult)> {
        // Check cooldown
        let ticks_since_attack = current_tick.0.wrapping_sub(last_attack_tick.0);
        if ticks_since_attack < ATTACK_COOLDOWN_TICKS {
            return None;
        }

        // Find closest target in range and in front
        let mut best_target = None;
        let mut best_dist = ATTACK_RANGE;

        for (target_id, target_pos, health) in targets {
            if *target_id == attacker_id || *health == 0 {
                continue;
            }

            let to_target = *target_pos - attacker_pos;
            let distance = to_target.length();

            if distance > ATTACK_RANGE {
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
                best_target = Some((*target_id, *health));
            }
        }

        if let Some((target_id, health)) = best_target {
            let killed = health <= MELEE_DAMAGE;
            let damage = MELEE_DAMAGE.min(health);

            Some((
                target_id,
                HitResult {
                    attacker: attacker_id,
                    target: target_id,
                    damage,
                    killed,
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

        let attacker_id = PlayerId(1);
        let target_id = PlayerId(2);

        let attacker_pos = Vec3::ZERO;
        let attacker_forward = Vec3::new(0.0, 0.0, 1.0);

        let targets = vec![
            (target_id, Vec3::new(0.0, 0.0, 1.5), 100), // In range
        ];

        let result = system.try_attack(
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

        let targets = vec![
            (PlayerId(2), Vec3::new(0.0, 0.0, 10.0), 100), // Out of range
        ];

        let result = system.try_attack(
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

        let targets = vec![(PlayerId(2), Vec3::new(0.0, 0.0, 1.5), 100)];

        // Attack on cooldown
        let result = system.try_attack(
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

        let targets = vec![
            (PlayerId(2), Vec3::new(0.0, 0.0, 1.5), 15), // Low health
        ];

        let result = system.try_attack(
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
}
