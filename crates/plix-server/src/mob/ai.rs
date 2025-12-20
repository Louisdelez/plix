//! Mob AI state machine
//!
//! Implements a simple FSM for mob behavior:
//! - Idle: Standing at spawn, no target
//! - Aggro: Moving towards target player
//! - Attack: In range, attacking target
//! - Return: Returning to spawn (leash exceeded or target lost)

use plix_common::content::mob::{MobBehavior, MobStats};
use plix_common::math::Vec3;
use plix_common::types::PlayerId;

/// AI state for a mob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobAiState {
    /// Idle at spawn, looking for targets
    Idle,
    /// Moving towards target player
    Aggro,
    /// In attack range, attacking
    Attack,
    /// Returning to spawn point
    Return,
}

/// Result of an AI tick update.
#[derive(Debug, Clone)]
pub enum AiAction {
    /// No action needed
    None,
    /// Move towards a position
    MoveTo(Vec3),
    /// Attack the current target
    Attack(PlayerId),
    /// Target acquired (for initial aggro)
    AcquireTarget(PlayerId),
    /// Target lost
    LoseTarget,
    /// Arrived back at spawn
    ReturnComplete,
}

/// Player position and distance info for targeting.
#[derive(Debug, Clone, Copy)]
pub struct PlayerInfo {
    pub id: PlayerId,
    pub position: Vec3,
    pub distance: f32,
}

/// Update mob AI state and return the action to take.
pub fn update_ai(
    current_state: MobAiState,
    mob_position: Vec3,
    spawn_position: Vec3,
    current_target: Option<PlayerId>,
    nearby_players: &[PlayerInfo],
    stats: &MobStats,
    _behavior: &MobBehavior,
    can_attack: bool,
) -> (MobAiState, AiAction) {
    let distance_from_spawn = (mob_position - spawn_position).length();

    match current_state {
        MobAiState::Idle => {
            // Look for closest player within aggro radius
            if let Some(closest) = find_closest_player(nearby_players, stats.aggro_radius) {
                return (MobAiState::Aggro, AiAction::AcquireTarget(closest.id));
            }
            (MobAiState::Idle, AiAction::None)
        }

        MobAiState::Aggro => {
            // Check leash distance
            if distance_from_spawn > stats.leash_radius {
                return (MobAiState::Return, AiAction::LoseTarget);
            }

            // Find current target or acquire new one
            let target = match current_target {
                Some(target_id) => nearby_players.iter().find(|p| p.id == target_id),
                None => find_closest_player(nearby_players, stats.aggro_radius),
            };

            match target {
                Some(player) => {
                    // Check if in attack range
                    if player.distance <= stats.attack_range {
                        return (MobAiState::Attack, AiAction::MoveTo(player.position));
                    }
                    // Move towards target
                    (MobAiState::Aggro, AiAction::MoveTo(player.position))
                }
                None => {
                    // Target lost, return to spawn
                    (MobAiState::Return, AiAction::LoseTarget)
                }
            }
        }

        MobAiState::Attack => {
            // Check leash distance
            if distance_from_spawn > stats.leash_radius {
                return (MobAiState::Return, AiAction::LoseTarget);
            }

            // Find current target
            let target = match current_target {
                Some(target_id) => nearby_players.iter().find(|p| p.id == target_id),
                None => None,
            };

            match target {
                Some(player) => {
                    // Check if still in attack range
                    if player.distance > stats.attack_range {
                        // Chase target
                        return (MobAiState::Aggro, AiAction::MoveTo(player.position));
                    }

                    // Attack if cooldown ready
                    if can_attack {
                        (MobAiState::Attack, AiAction::Attack(player.id))
                    } else {
                        (MobAiState::Attack, AiAction::None)
                    }
                }
                None => {
                    // Target lost, return to spawn
                    (MobAiState::Return, AiAction::LoseTarget)
                }
            }
        }

        MobAiState::Return => {
            // Check if we've reached spawn
            let dist_to_spawn = (mob_position - spawn_position).length();
            if dist_to_spawn < 1.0 {
                return (MobAiState::Idle, AiAction::ReturnComplete);
            }

            // Continue moving to spawn
            (MobAiState::Return, AiAction::MoveTo(spawn_position))
        }
    }
}

/// Find the closest player within the given radius.
fn find_closest_player(players: &[PlayerInfo], max_distance: f32) -> Option<&PlayerInfo> {
    players
        .iter()
        .filter(|p| p.distance <= max_distance)
        .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal))
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::content::mob::MobBehavior;

    fn test_stats() -> MobStats {
        MobStats {
            hp: 100,
            damage: 10,
            speed: 5.0,
            armor: 0,
            aggro_radius: 10.0,
            leash_radius: 25.0,
            attack_range: 2.0,
            attack_cooldown: 1.0,
        }
    }

    #[test]
    fn test_idle_to_aggro() {
        let stats = test_stats();
        let players = vec![PlayerInfo {
            id: PlayerId(1),
            position: Vec3::new(5.0, 0.0, 0.0),
            distance: 5.0,
        }];

        let (new_state, action) = update_ai(
            MobAiState::Idle,
            Vec3::ZERO,
            Vec3::ZERO,
            None,
            &players,
            &stats,
            &MobBehavior::Aggro,
            true,
        );

        assert_eq!(new_state, MobAiState::Aggro);
        assert!(matches!(action, AiAction::AcquireTarget(PlayerId(1))));
    }

    #[test]
    fn test_idle_no_target() {
        let stats = test_stats();
        let players = vec![PlayerInfo {
            id: PlayerId(1),
            position: Vec3::new(15.0, 0.0, 0.0),
            distance: 15.0, // Outside aggro radius
        }];

        let (new_state, action) = update_ai(
            MobAiState::Idle,
            Vec3::ZERO,
            Vec3::ZERO,
            None,
            &players,
            &stats,
            &MobBehavior::Aggro,
            true,
        );

        assert_eq!(new_state, MobAiState::Idle);
        assert!(matches!(action, AiAction::None));
    }

    #[test]
    fn test_aggro_to_attack() {
        let stats = test_stats();
        let players = vec![PlayerInfo {
            id: PlayerId(1),
            position: Vec3::new(1.5, 0.0, 0.0),
            distance: 1.5, // Within attack range
        }];

        let (new_state, action) = update_ai(
            MobAiState::Aggro,
            Vec3::ZERO,
            Vec3::ZERO,
            Some(PlayerId(1)),
            &players,
            &stats,
            &MobBehavior::Aggro,
            true,
        );

        assert_eq!(new_state, MobAiState::Attack);
    }

    #[test]
    fn test_leash_return() {
        let stats = test_stats();
        let players = vec![PlayerInfo {
            id: PlayerId(1),
            position: Vec3::new(30.0, 0.0, 0.0),
            distance: 5.0, // Distance from mob, not spawn
        }];

        let (new_state, action) = update_ai(
            MobAiState::Aggro,
            Vec3::new(30.0, 0.0, 0.0), // Mob is far from spawn
            Vec3::ZERO,                // Spawn at origin
            Some(PlayerId(1)),
            &players,
            &stats,
            &MobBehavior::Aggro,
            true,
        );

        assert_eq!(new_state, MobAiState::Return);
        assert!(matches!(action, AiAction::LoseTarget));
    }

    #[test]
    fn test_attack_with_cooldown() {
        let stats = test_stats();
        let players = vec![PlayerInfo {
            id: PlayerId(1),
            position: Vec3::new(1.0, 0.0, 0.0),
            distance: 1.0,
        }];

        // Can attack
        let (_, action) = update_ai(
            MobAiState::Attack,
            Vec3::ZERO,
            Vec3::ZERO,
            Some(PlayerId(1)),
            &players,
            &stats,
            &MobBehavior::Aggro,
            true,
        );
        assert!(matches!(action, AiAction::Attack(_)));

        // Cooldown active
        let (_, action) = update_ai(
            MobAiState::Attack,
            Vec3::ZERO,
            Vec3::ZERO,
            Some(PlayerId(1)),
            &players,
            &stats,
            &MobBehavior::Aggro,
            false,
        );
        assert!(matches!(action, AiAction::None));
    }

    #[test]
    fn test_return_complete() {
        let stats = test_stats();
        let players = vec![];

        let (new_state, action) = update_ai(
            MobAiState::Return,
            Vec3::new(0.5, 0.0, 0.0), // Almost at spawn
            Vec3::ZERO,
            None,
            &players,
            &stats,
            &MobBehavior::Aggro,
            true,
        );

        assert_eq!(new_state, MobAiState::Idle);
        assert!(matches!(action, AiAction::ReturnComplete));
    }
}
