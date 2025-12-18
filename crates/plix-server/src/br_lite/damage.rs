//! BR Lite damage controller - applies zone damage to players outside safe zone

use std::collections::HashMap;

use glam::Vec3;
use plix_common::time::Tick;
use plix_common::types::PlayerId;
use tracing::debug;

use super::state::ZoneState;
use super::zone::ZoneController;

/// Tracks damage application for players outside the zone
#[derive(Debug)]
pub struct DamageController {
    /// Tick interval between damage applications (default: 60 = 1 second at 60Hz)
    damage_interval: u32,
    /// Last tick each player received zone damage
    last_damage_tick: HashMap<PlayerId, u32>,
}

impl Default for DamageController {
    fn default() -> Self {
        Self::new(60) // 1 second at 60Hz
    }
}

impl DamageController {
    /// Create a new damage controller with specified interval
    pub fn new(damage_interval: u32) -> Self {
        Self {
            damage_interval,
            last_damage_tick: HashMap::new(),
        }
    }

    /// Check if a player should receive damage this tick
    /// Returns the damage amount if damage should be applied, None otherwise
    pub fn check_damage(
        &mut self,
        player_id: PlayerId,
        player_pos: Vec3,
        zone_controller: &ZoneController,
        zone_state: &ZoneState,
        current_tick: Tick,
    ) -> Option<u32> {
        // Check if player is inside the safe zone
        if zone_controller.is_in_zone(player_pos, zone_state) {
            // Player is safe - reset their damage timer
            self.last_damage_tick.remove(&player_id);
            return None;
        }

        // Player is outside zone - check damage interval
        let last_tick = self.last_damage_tick.get(&player_id).copied().unwrap_or(0);
        let ticks_since_damage = current_tick.0.saturating_sub(last_tick);

        if ticks_since_damage >= self.damage_interval {
            // Apply damage
            self.last_damage_tick.insert(player_id, current_tick.0);

            debug!(
                target: "br_lite",
                player_id = ?player_id,
                damage = zone_state.damage_per_tick,
                "zone_damage_applied"
            );

            Some(zone_state.damage_per_tick)
        } else {
            None
        }
    }

    /// Clear damage tracking for a player (on elimination)
    pub fn clear_player(&mut self, player_id: PlayerId) {
        self.last_damage_tick.remove(&player_id);
    }

    /// Reset all damage tracking (new match)
    pub fn reset(&mut self) {
        self.last_damage_tick.clear();
    }

    /// Get the damage interval
    pub fn damage_interval(&self) -> u32 {
        self.damage_interval
    }
}

/// Result of processing zone damage for all players
#[derive(Debug, Default)]
pub struct DamageResult {
    /// Players who received damage this tick: (player_id, damage_amount)
    pub damaged: Vec<(PlayerId, u32)>,
}

impl DamageController {
    /// Process damage for multiple players
    /// Returns list of players who received damage
    pub fn tick_all<'a>(
        &mut self,
        players: impl Iterator<Item = (PlayerId, Vec3)>,
        zone_controller: &ZoneController,
        zone_state: &ZoneState,
        current_tick: Tick,
    ) -> DamageResult {
        let mut result = DamageResult::default();

        for (player_id, pos) in players {
            if let Some(damage) =
                self.check_damage(player_id, pos, zone_controller, zone_state, current_tick)
            {
                result.damaged.push((player_id, damage));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::br_lite::config::{BrLiteConfig, ZonePhase};
    use glam::Vec2;

    fn test_config() -> BrLiteConfig {
        BrLiteConfig {
            phases: vec![ZonePhase::new(10, 5, 0.8, 10)],
            initial_radius: Some(50.0),
            ..Default::default()
        }
    }

    #[test]
    fn test_damage_controller_creation() {
        let controller = DamageController::new(60);
        assert_eq!(controller.damage_interval(), 60);
    }

    #[test]
    fn test_no_damage_inside_zone() {
        let config = test_config();
        let zone_controller = ZoneController::new(&config, Vec2::new(64.0, 64.0));
        let zone_state = zone_controller.init_state(Vec2::new(32.0, 32.0));
        let mut damage_controller = DamageController::new(60);

        // Player at center - inside zone
        let damage = damage_controller.check_damage(
            PlayerId(1),
            Vec3::new(32.0, 5.0, 32.0),
            &zone_controller,
            &zone_state,
            Tick(100),
        );

        assert!(damage.is_none());
    }

    #[test]
    fn test_damage_outside_zone() {
        let config = test_config();
        let zone_controller = ZoneController::new(&config, Vec2::new(64.0, 64.0));
        let zone_state = zone_controller.init_state(Vec2::new(32.0, 32.0));
        let mut damage_controller = DamageController::new(60);

        // Player far outside zone
        let damage = damage_controller.check_damage(
            PlayerId(1),
            Vec3::new(200.0, 5.0, 200.0),
            &zone_controller,
            &zone_state,
            Tick(60),
        );

        assert_eq!(damage, Some(10)); // 10 damage per tick from config
    }

    #[test]
    fn test_damage_interval_respected() {
        let config = test_config();
        let zone_controller = ZoneController::new(&config, Vec2::new(64.0, 64.0));
        let zone_state = zone_controller.init_state(Vec2::new(32.0, 32.0));
        let mut damage_controller = DamageController::new(60);

        let outside_pos = Vec3::new(200.0, 5.0, 200.0);

        // First damage at tick 60
        let damage = damage_controller.check_damage(
            PlayerId(1),
            outside_pos,
            &zone_controller,
            &zone_state,
            Tick(60),
        );
        assert_eq!(damage, Some(10));

        // No damage at tick 61 (too soon)
        let damage = damage_controller.check_damage(
            PlayerId(1),
            outside_pos,
            &zone_controller,
            &zone_state,
            Tick(61),
        );
        assert!(damage.is_none());

        // No damage at tick 119 (still too soon)
        let damage = damage_controller.check_damage(
            PlayerId(1),
            outside_pos,
            &zone_controller,
            &zone_state,
            Tick(119),
        );
        assert!(damage.is_none());

        // Damage at tick 120 (60 ticks later)
        let damage = damage_controller.check_damage(
            PlayerId(1),
            outside_pos,
            &zone_controller,
            &zone_state,
            Tick(120),
        );
        assert_eq!(damage, Some(10));
    }

    #[test]
    fn test_entering_zone_resets_timer() {
        let config = test_config();
        let zone_controller = ZoneController::new(&config, Vec2::new(64.0, 64.0));
        let zone_state = zone_controller.init_state(Vec2::new(32.0, 32.0));
        let mut damage_controller = DamageController::new(60);

        let outside_pos = Vec3::new(200.0, 5.0, 200.0);
        let inside_pos = Vec3::new(32.0, 5.0, 32.0);

        // Take damage outside
        let damage = damage_controller.check_damage(
            PlayerId(1),
            outside_pos,
            &zone_controller,
            &zone_state,
            Tick(60),
        );
        assert_eq!(damage, Some(10));

        // Enter zone - should reset timer
        let damage = damage_controller.check_damage(
            PlayerId(1),
            inside_pos,
            &zone_controller,
            &zone_state,
            Tick(70),
        );
        assert!(damage.is_none());

        // Leave zone again - should take damage immediately (timer was reset)
        let damage = damage_controller.check_damage(
            PlayerId(1),
            outside_pos,
            &zone_controller,
            &zone_state,
            Tick(80),
        );
        assert_eq!(damage, Some(10));
    }

    #[test]
    fn test_clear_player() {
        let mut damage_controller = DamageController::new(60);
        damage_controller.last_damage_tick.insert(PlayerId(1), 100);
        damage_controller.last_damage_tick.insert(PlayerId(2), 100);

        damage_controller.clear_player(PlayerId(1));

        assert!(!damage_controller
            .last_damage_tick
            .contains_key(&PlayerId(1)));
        assert!(damage_controller
            .last_damage_tick
            .contains_key(&PlayerId(2)));
    }

    #[test]
    fn test_reset() {
        let mut damage_controller = DamageController::new(60);
        damage_controller.last_damage_tick.insert(PlayerId(1), 100);
        damage_controller.last_damage_tick.insert(PlayerId(2), 100);

        damage_controller.reset();

        assert!(damage_controller.last_damage_tick.is_empty());
    }

    #[test]
    fn test_tick_all_multiple_players() {
        let config = test_config();
        let zone_controller = ZoneController::new(&config, Vec2::new(64.0, 64.0));
        let zone_state = zone_controller.init_state(Vec2::new(32.0, 32.0));
        let mut damage_controller = DamageController::new(60);

        let players = vec![
            (PlayerId(1), Vec3::new(32.0, 5.0, 32.0)),   // Inside zone
            (PlayerId(2), Vec3::new(200.0, 5.0, 200.0)), // Outside zone
            (PlayerId(3), Vec3::new(150.0, 5.0, 150.0)), // Outside zone
        ];

        let result = damage_controller.tick_all(
            players.into_iter(),
            &zone_controller,
            &zone_state,
            Tick(60),
        );

        assert_eq!(result.damaged.len(), 2);
        assert!(result.damaged.iter().any(|(id, _)| *id == PlayerId(2)));
        assert!(result.damaged.iter().any(|(id, _)| *id == PlayerId(3)));
        assert!(!result.damaged.iter().any(|(id, _)| *id == PlayerId(1)));
    }
}
