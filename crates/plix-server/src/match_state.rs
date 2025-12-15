//! Match and round state management

use plix_common::protocol::{MatchPhase, MatchState, TeamScore};
use plix_common::time::Tick;
use plix_common::types::TeamId;

/// Match configuration
#[derive(Debug, Clone)]
pub struct MatchConfig {
    /// Minimum players to start
    pub min_players: u8,
    /// Countdown duration in ticks
    pub countdown_ticks: u32,
    /// Round time limit in seconds
    pub round_time_limit: u32,
    /// Rounds to win match
    pub rounds_to_win: u16,
    /// Respawn delay in ticks
    pub respawn_delay: u32,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            min_players: 2,
            countdown_ticks: 180,  // 3 seconds at 60 Hz
            round_time_limit: 300, // 5 minutes
            rounds_to_win: 5,
            respawn_delay: 180, // 3 seconds at 60 Hz
        }
    }
}

/// Match state machine
#[derive(Debug)]
pub struct MatchStateMachine {
    /// Current state
    state: MatchState,
    /// Configuration
    config: MatchConfig,
    /// Phase start tick
    phase_start_tick: Tick,
}

impl MatchStateMachine {
    /// Create a new match state machine
    pub fn new(config: MatchConfig) -> Self {
        Self {
            state: MatchState::default(),
            config,
            phase_start_tick: Tick::ZERO,
        }
    }

    /// Get current state
    pub fn state(&self) -> &MatchState {
        &self.state
    }

    /// Get current phase
    pub fn phase(&self) -> MatchPhase {
        self.state.phase
    }

    /// Update match state based on current tick and player count
    pub fn update(&mut self, current_tick: Tick, player_count: usize) -> Option<MatchPhase> {
        let old_phase = self.state.phase;

        match self.state.phase {
            MatchPhase::WaitingForPlayers => {
                if player_count >= self.config.min_players as usize {
                    self.transition_to(MatchPhase::Countdown, current_tick);
                }
            }
            MatchPhase::Countdown => {
                if player_count < self.config.min_players as usize {
                    self.transition_to(MatchPhase::WaitingForPlayers, current_tick);
                } else {
                    let elapsed = current_tick.diff(self.phase_start_tick) as u32;
                    if elapsed >= self.config.countdown_ticks {
                        self.start_round(current_tick);
                    }
                }
            }
            MatchPhase::Playing => {
                // Round time check
                let elapsed_secs = (current_tick.diff(self.state.round_start_tick) as f64) / 60.0;
                if elapsed_secs >= self.config.round_time_limit as f64 {
                    self.end_round(None, current_tick);
                }
            }
            MatchPhase::RoundEnd => {
                let elapsed = current_tick.diff(self.phase_start_tick) as u32;
                if elapsed >= 180 {
                    // 3 second delay
                    // Check if match is over
                    if let Some(_winner) = self.check_match_winner() {
                        self.transition_to(MatchPhase::MatchEnd, current_tick);
                    } else {
                        self.transition_to(MatchPhase::Countdown, current_tick);
                    }
                }
            }
            MatchPhase::MatchEnd => {
                // Stay in match end until reset
            }
        }

        if self.state.phase != old_phase {
            Some(self.state.phase)
        } else {
            None
        }
    }

    /// Transition to a new phase
    fn transition_to(&mut self, phase: MatchPhase, tick: Tick) {
        self.state.phase = phase;
        self.phase_start_tick = tick;
    }

    /// Start a new round
    fn start_round(&mut self, tick: Tick) {
        self.state.round_number += 1;
        self.state.round_start_tick = tick;
        self.transition_to(MatchPhase::Playing, tick);
    }

    /// End the current round
    pub fn end_round(&mut self, winner: Option<TeamId>, tick: Tick) {
        if let Some(team) = winner {
            for score in &mut self.state.scores {
                if score.team == team {
                    score.score += 1;
                }
            }
        }
        self.transition_to(MatchPhase::RoundEnd, tick);
    }

    /// Check if a team has won the match
    fn check_match_winner(&self) -> Option<TeamId> {
        for score in &self.state.scores {
            if score.score >= self.config.rounds_to_win as u32 {
                return Some(score.team);
            }
        }
        None
    }

    /// Get team score
    pub fn get_score(&self, team: TeamId) -> u32 {
        self.state
            .scores
            .iter()
            .find(|s| s.team == team)
            .map(|s| s.score)
            .unwrap_or(0)
    }

    /// Get remaining round time in seconds
    pub fn round_time_remaining(&self, current_tick: Tick) -> u32 {
        if self.state.phase != MatchPhase::Playing {
            return self.config.round_time_limit;
        }

        let elapsed_ticks = current_tick.diff(self.state.round_start_tick).max(0) as u32;
        let elapsed_secs = elapsed_ticks / 60;
        self.config.round_time_limit.saturating_sub(elapsed_secs)
    }

    /// Reset match to initial state
    pub fn reset(&mut self) {
        self.state = MatchState {
            phase: MatchPhase::WaitingForPlayers,
            round_number: 0,
            round_start_tick: Tick::ZERO,
            round_time_limit: self.config.round_time_limit,
            scores: vec![
                TeamScore {
                    team: TeamId::TEAM_0,
                    score: 0,
                },
                TeamScore {
                    team: TeamId::TEAM_1,
                    score: 0,
                },
            ],
        };
        self.phase_start_tick = Tick::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waiting_to_countdown() {
        let config = MatchConfig::default();
        let mut state = MatchStateMachine::new(config);

        // Not enough players
        assert!(state.update(Tick(0), 1).is_none());
        assert_eq!(state.phase(), MatchPhase::WaitingForPlayers);

        // Enough players
        let change = state.update(Tick(1), 2);
        assert_eq!(change, Some(MatchPhase::Countdown));
    }

    #[test]
    fn test_countdown_to_playing() {
        let config = MatchConfig {
            countdown_ticks: 10,
            ..Default::default()
        };
        let mut state = MatchStateMachine::new(config);

        state.update(Tick(0), 2); // Go to countdown

        // During countdown
        assert!(state.update(Tick(5), 2).is_none());

        // After countdown
        let change = state.update(Tick(15), 2);
        assert_eq!(change, Some(MatchPhase::Playing));
        assert_eq!(state.state().round_number, 1);
    }
}
