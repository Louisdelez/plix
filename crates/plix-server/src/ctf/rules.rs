//! CTF game rules engine
//!
//! Implements the game rules for CTF flag interactions: pickup, drop, return, and capture logic.

use plix_common::math::Vec3;
use plix_common::time::Tick;
use plix_common::types::{FlagState, PlayerId, TeamId};

use super::enemy_team;
use super::state::CtfState;

/// Pickup radius for dropped flags (in blocks)
const PICKUP_RADIUS: f32 = 2.0;

/// CTF game rules engine
pub struct CtfRules;

impl CtfRules {
    /// Check if player can pick up a flag
    /// Returns Some(flag_team) if pickup is possible, None otherwise
    pub fn can_pickup(
        player_id: PlayerId,
        player_team: TeamId,
        player_pos: Vec3,
        state: &CtfState,
    ) -> Option<TeamId> {
        // Player can only pick up enemy flag
        let enemy = enemy_team(player_team);
        let enemy_flag = state.flag(enemy);

        // Can't pickup if already carrying a flag
        if state.is_carrying_flag(player_id) {
            return None;
        }

        match &enemy_flag.state {
            FlagState::AtBase => {
                // Check if player is in enemy flag base zone
                if let Some(base_zone) = state.flag_base(enemy) {
                    if base_zone.contains(player_pos) {
                        return Some(enemy);
                    }
                }
                None
            }
            FlagState::Dropped { position, .. } => {
                // Check if player is near dropped flag
                if player_pos.distance(*position) <= PICKUP_RADIUS {
                    return Some(enemy);
                }
                None
            }
            FlagState::Carried { .. } => {
                // Can't pickup a flag that's already being carried
                None
            }
        }
    }

    /// Execute flag pickup - mutates state
    /// Returns true if pickup succeeded
    pub fn pickup(
        player_id: PlayerId,
        player_team: TeamId,
        player_pos: Vec3,
        state: &mut CtfState,
    ) -> bool {
        if let Some(flag_team) = Self::can_pickup(player_id, player_team, player_pos, state) {
            state.flag_mut(flag_team).state = FlagState::Carried { carrier: player_id };
            true
        } else {
            false
        }
    }

    /// Check if carrier can capture (in capture zone, own flag at base)
    pub fn can_capture(
        player_id: PlayerId,
        player_team: TeamId,
        player_pos: Vec3,
        state: &CtfState,
    ) -> bool {
        let enemy = enemy_team(player_team);
        let enemy_flag = state.flag(enemy);
        let own_flag = state.flag(player_team);

        // Must be carrying enemy flag
        if enemy_flag.carrier() != Some(player_id) {
            return false;
        }

        // Must be in own capture zone
        if let Some(capture_zone) = state.capture_zone(player_team) {
            if !capture_zone.contains(player_pos) {
                return false;
            }
        } else {
            return false;
        }

        // Classic rule: own flag must be at base
        own_flag.is_at_base()
    }

    /// Execute capture - awards point, resets flags
    /// Returns new capture score for team (0 if capture failed)
    pub fn capture(player_id: PlayerId, player_team: TeamId, state: &mut CtfState) -> u32 {
        let enemy = enemy_team(player_team);

        // Reset both flags to base
        state.flag_mut(enemy).reset_to_base();
        state.flag_mut(player_team).reset_to_base();

        // Increment score
        state.increment_score(player_team);
        state.score(player_team)
    }

    /// Drop flag at position (called on death/disconnect)
    /// Returns Some(flag_team) if player was carrying a flag
    pub fn drop(
        player_id: PlayerId,
        drop_pos: Vec3,
        current_tick: Tick,
        state: &mut CtfState,
    ) -> Option<TeamId> {
        // Find which flag (if any) this player is carrying
        let flag_team = state.flag_carried_by(player_id)?;

        let return_tick = Tick(current_tick.0 + state.config.flag_return_delay_ticks);
        state.flag_mut(flag_team).state = FlagState::Dropped {
            position: drop_pos,
            return_tick,
        };

        Some(flag_team)
    }

    /// Check if teammate can return a dropped flag
    pub fn can_return(player_team: TeamId, player_pos: Vec3, state: &CtfState) -> bool {
        let own_flag = state.flag(player_team);

        match &own_flag.state {
            FlagState::Dropped { position, .. } => player_pos.distance(*position) <= PICKUP_RADIUS,
            _ => false,
        }
    }

    /// Return dropped flag to base (teammate touch)
    /// Returns true if flag was returned
    pub fn return_flag(player_team: TeamId, state: &mut CtfState) -> bool {
        let own_flag = state.flag(player_team);

        if own_flag.state.is_dropped() {
            state.flag_mut(player_team).reset_to_base();
            true
        } else {
            false
        }
    }

    /// Update dropped flag timers - returns flags that auto-returned
    pub fn update_return_timers(current_tick: Tick, state: &mut CtfState) -> Vec<TeamId> {
        let mut returned = Vec::new();

        for team_idx in 0..2 {
            let team = if team_idx == 0 {
                TeamId::TEAM_0
            } else {
                TeamId::TEAM_1
            };
            let flag = state.flag(team);

            if let FlagState::Dropped { return_tick, .. } = &flag.state {
                if current_tick.0 >= return_tick.0 {
                    state.flag_mut(team).reset_to_base();
                    returned.push(team);
                }
            }
        }

        returned
    }

    /// Check if flag is out of bounds and should auto-return
    pub fn check_out_of_bounds(
        flag_team: TeamId,
        arena_min: Vec3,
        arena_max: Vec3,
        state: &mut CtfState,
    ) -> bool {
        let flag = state.flag(flag_team);

        if let FlagState::Dropped { position, .. } = &flag.state {
            let out_of_bounds = position.x < arena_min.x
                || position.x > arena_max.x
                || position.y < arena_min.y
                || position.y > arena_max.y
                || position.z < arena_min.z
                || position.z > arena_max.z;

            if out_of_bounds {
                state.flag_mut(flag_team).reset_to_base();
                return true;
            }
        }

        false
    }
}
