//! CTF event coordinator
//!
//! Orchestrates CTF game flow by handling events, coordinating between rules engine
//! and state, and triggering broadcasts.

use plix_common::math::Vec3;
use plix_common::time::Tick;
use plix_common::types::{PlayerId, TeamId};

use super::rules::CtfRules;
use super::state::CtfState;
use super::CtfConfig;

/// Reason why a flag was returned to base
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnReason {
    /// Teammate touched the dropped flag
    TeammateTouch,
    /// Return timer expired
    Timeout,
    /// Flag was dropped out of bounds
    OutOfBounds,
}

/// Events emitted by coordinator
#[derive(Debug, Clone)]
pub enum CtfEvent {
    /// Player picked up enemy flag
    FlagPickup {
        player_id: PlayerId,
        flag_team: TeamId,
    },
    /// Player dropped flag (death/disconnect)
    FlagDrop {
        player_id: PlayerId,
        flag_team: TeamId,
        position: Vec3,
        return_tick: Tick,
    },
    /// Flag returned to base
    FlagReturn {
        flag_team: TeamId,
        reason: ReturnReason,
    },
    /// Flag captured - point scored
    FlagCapture {
        capturing_team: TeamId,
        capturing_player: PlayerId,
        new_score: u32,
    },
    /// Team won by reaching capture limit
    Victory {
        winning_team: TeamId,
        final_scores: [u32; 2],
    },
}

/// CTF event coordinator
pub struct CtfCoordinator {
    /// CTF game state
    state: CtfState,
}

impl CtfCoordinator {
    /// Create coordinator with initial state
    pub fn new(state: CtfState) -> Self {
        Self { state }
    }

    /// Get immutable state reference
    pub fn state(&self) -> &CtfState {
        &self.state
    }

    /// Get mutable state reference
    pub fn state_mut(&mut self) -> &mut CtfState {
        &mut self.state
    }

    /// Get CTF config
    pub fn config(&self) -> &CtfConfig {
        &self.state.config
    }

    /// Process player position update - checks for pickups/captures/returns
    /// Returns events that occurred
    pub fn on_player_position(
        &mut self,
        player_id: PlayerId,
        player_team: TeamId,
        position: Vec3,
        _current_tick: Tick,
    ) -> Vec<CtfEvent> {
        let mut events = Vec::new();

        // Check if player can return own dropped flag (teammate touch)
        if CtfRules::can_return(player_team, position, &self.state) {
            CtfRules::return_flag(player_team, &mut self.state);
            events.push(CtfEvent::FlagReturn {
                flag_team: player_team,
                reason: ReturnReason::TeammateTouch,
            });
        }

        // Check if player can pick up enemy flag
        if CtfRules::pickup(player_id, player_team, position, &mut self.state) {
            let enemy_team = super::enemy_team(player_team);
            events.push(CtfEvent::FlagPickup {
                player_id,
                flag_team: enemy_team,
            });
        }

        // Check if carrier can capture
        if CtfRules::can_capture(player_id, player_team, position, &self.state) {
            let new_score = CtfRules::capture(player_id, player_team, &mut self.state);
            events.push(CtfEvent::FlagCapture {
                capturing_team: player_team,
                capturing_player: player_id,
                new_score,
            });

            // Check for victory
            if new_score >= self.state.config.capture_limit as u32 {
                events.push(CtfEvent::Victory {
                    winning_team: player_team,
                    final_scores: self.state.capture_scores,
                });
            }
        }

        events
    }

    /// Process player death - drops flag if carrying
    /// Returns events that occurred
    pub fn on_player_death(
        &mut self,
        player_id: PlayerId,
        death_position: Vec3,
        current_tick: Tick,
    ) -> Vec<CtfEvent> {
        let mut events = Vec::new();

        // Extract config values before mutable borrow
        let flag_return_delay_ticks = self.state.config.flag_return_delay_ticks;

        if let Some(flag_team) =
            CtfRules::drop(player_id, death_position, current_tick, &mut self.state)
        {
            let return_tick = Tick(current_tick.0 + flag_return_delay_ticks);
            events.push(CtfEvent::FlagDrop {
                player_id,
                flag_team,
                position: death_position,
                return_tick,
            });
        }

        events
    }

    /// Process player disconnect - drops flag if carrying
    /// Returns events that occurred
    pub fn on_player_disconnect(
        &mut self,
        player_id: PlayerId,
        last_position: Vec3,
        current_tick: Tick,
    ) -> Vec<CtfEvent> {
        // Same behavior as death
        self.on_player_death(player_id, last_position, current_tick)
    }

    /// Tick update - processes timers
    /// Returns events that occurred (auto-returns)
    pub fn tick(&mut self, current_tick: Tick) -> Vec<CtfEvent> {
        let returned = CtfRules::update_return_timers(current_tick, &mut self.state);

        returned
            .into_iter()
            .map(|flag_team| CtfEvent::FlagReturn {
                flag_team,
                reason: ReturnReason::Timeout,
            })
            .collect()
    }

    /// Reset state for new match
    pub fn reset(&mut self) {
        self.state.reset();
    }

    /// Check if capture limit reached
    pub fn is_victory(&self) -> Option<TeamId> {
        let limit = self.state.config.capture_limit as u32;

        if self.state.score(TeamId::TEAM_0) >= limit {
            Some(TeamId::TEAM_0)
        } else if self.state.score(TeamId::TEAM_1) >= limit {
            Some(TeamId::TEAM_1)
        } else {
            None
        }
    }

    /// Determine winner by score (for time limit)
    pub fn winner_by_score(&self) -> Option<TeamId> {
        let score0 = self.state.score(TeamId::TEAM_0);
        let score1 = self.state.score(TeamId::TEAM_1);

        if score0 > score1 {
            Some(TeamId::TEAM_0)
        } else if score1 > score0 {
            Some(TeamId::TEAM_1)
        } else {
            None // Tie
        }
    }

    /// Get current capture scores
    pub fn scores(&self) -> [u32; 2] {
        self.state.capture_scores
    }

    /// Check out of bounds for all dropped flags
    pub fn check_out_of_bounds(&mut self, arena_min: Vec3, arena_max: Vec3) -> Vec<CtfEvent> {
        let mut events = Vec::new();

        for team_idx in 0..2 {
            let team = if team_idx == 0 {
                TeamId::TEAM_0
            } else {
                TeamId::TEAM_1
            };

            if CtfRules::check_out_of_bounds(team, arena_min, arena_max, &mut self.state) {
                events.push(CtfEvent::FlagReturn {
                    flag_team: team,
                    reason: ReturnReason::OutOfBounds,
                });
            }
        }

        events
    }
}
