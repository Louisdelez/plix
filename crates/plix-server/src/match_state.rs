//! Match and round state management

use plix_common::protocol::{MatchEndReason, MatchPhase, MatchState, PlayerScore, TeamScore};
use plix_common::time::Tick;
use plix_common::types::{PlayerId, TeamId};

/// Match configuration
#[derive(Debug, Clone)]
pub struct MatchConfig {
    /// Minimum players to start countdown
    pub min_players: u8,
    /// Countdown duration in ticks (default: 180 = 3s at 60Hz)
    pub countdown_ticks: u32,
    /// Match time limit in seconds (default: 300 = 5 minutes)
    pub time_limit_seconds: u32,
    /// Score limit to win (default: 5 kills)
    pub score_limit: u16,
    /// End screen duration in ticks (default: 300 = 5s at 60Hz)
    pub end_screen_ticks: u32,
    /// Respawn delay in ticks (default: 180 = 3s at 60Hz)
    pub respawn_delay_ticks: u32,
    /// Arena rotation list (empty = replay same arena)
    pub arena_rotation: Vec<String>,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            min_players: 2,
            countdown_ticks: 180,     // 3 seconds at 60 Hz
            time_limit_seconds: 300,  // 5 minutes
            score_limit: 5,           // 5 kills to win
            end_screen_ticks: 300,    // 5 seconds at 60 Hz
            respawn_delay_ticks: 180, // 3 seconds at 60 Hz
            arena_rotation: Vec::new(),
        }
    }
}

/// Match end result
#[derive(Debug, Clone)]
pub struct MatchEndResult {
    /// Winner player ID (None for tie)
    pub winner: Option<PlayerId>,
    /// Final scores
    pub scores: Vec<PlayerScore>,
    /// Reason match ended
    pub reason: MatchEndReason,
}

/// Match state machine
#[derive(Debug)]
pub struct MatchStateMachine {
    /// Current state (broadcast to clients)
    state: MatchState,
    /// Server-only configuration
    config: MatchConfig,
    /// Tick when current phase started
    phase_start_tick: Tick,
    /// Current arena index in rotation
    arena_index: usize,
    /// Countdown ticks remaining (internal tracking at tick resolution)
    countdown_ticks_remaining: u32,
    /// Match time ticks remaining (internal tracking at tick resolution)
    time_ticks_remaining: u32,
    /// End screen ticks remaining
    end_screen_ticks_remaining: u32,
    /// Cached match end result (set when transitioning to EndScreen)
    match_end_result: Option<MatchEndResult>,
}

impl MatchStateMachine {
    /// Create a new match state machine
    pub fn new(config: MatchConfig, arena_name: String) -> Self {
        let time_limit = config.time_limit_seconds;
        let score_limit = config.score_limit;
        Self {
            state: MatchState {
                phase: MatchPhase::Lobby,
                countdown_remaining: 3,
                time_remaining: time_limit,
                score_limit,
                player_scores: Vec::new(),
                winner: None,
                arena_name,
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
            },
            config,
            phase_start_tick: Tick::ZERO,
            arena_index: 0,
            countdown_ticks_remaining: 0,
            time_ticks_remaining: time_limit * 60, // Convert seconds to ticks
            end_screen_ticks_remaining: 0,
            match_end_result: None,
        }
    }

    /// Get current state (for broadcast to clients)
    pub fn state(&self) -> &MatchState {
        &self.state
    }

    /// Get mutable state (for updating player scores)
    pub fn state_mut(&mut self) -> &mut MatchState {
        &mut self.state
    }

    /// Get current phase
    pub fn phase(&self) -> MatchPhase {
        self.state.phase
    }

    /// Get config
    pub fn config(&self) -> &MatchConfig {
        &self.config
    }

    /// Get match end result (only valid in EndScreen phase)
    pub fn match_end_result(&self) -> Option<&MatchEndResult> {
        self.match_end_result.as_ref()
    }

    /// Get current arena index
    pub fn arena_index(&self) -> usize {
        self.arena_index
    }

    /// Start countdown when all players are ready
    pub fn start_countdown(&mut self, current_tick: Tick) {
        if self.state.phase == MatchPhase::Lobby {
            self.countdown_ticks_remaining = self.config.countdown_ticks;
            self.state.countdown_remaining = (self.config.countdown_ticks / 60).max(1) as u8;
            self.transition_to(MatchPhase::Countdown, current_tick);
        }
    }

    /// Cancel countdown (player disconnected or unreadied)
    pub fn cancel_countdown(&mut self, current_tick: Tick) {
        if self.state.phase == MatchPhase::Countdown {
            self.transition_to(MatchPhase::Lobby, current_tick);
        }
    }

    /// Update match state based on current tick
    /// Returns Some(new_phase) if phase changed
    pub fn update(&mut self, current_tick: Tick) -> Option<MatchPhase> {
        let old_phase = self.state.phase;

        match self.state.phase {
            MatchPhase::Lobby => {
                // Lobby stays until start_countdown() is called
            }
            MatchPhase::Countdown => {
                if self.countdown_ticks_remaining > 0 {
                    self.countdown_ticks_remaining -= 1;
                    // Update seconds display (every 60 ticks)
                    self.state.countdown_remaining =
                        ((self.countdown_ticks_remaining + 59) / 60) as u8;
                }
                if self.countdown_ticks_remaining == 0 {
                    self.start_playing(current_tick);
                }
            }
            MatchPhase::Playing => {
                if self.time_ticks_remaining > 0 {
                    self.time_ticks_remaining -= 1;
                    // Update seconds display
                    self.state.time_remaining = self.time_ticks_remaining / 60;
                }
                // Time limit check - if time runs out, end match
                if self.time_ticks_remaining == 0 {
                    self.end_match_time_limit(current_tick);
                }
            }
            MatchPhase::EndScreen => {
                if self.end_screen_ticks_remaining > 0 {
                    self.end_screen_ticks_remaining -= 1;
                }
                if self.end_screen_ticks_remaining == 0 {
                    self.transition_to(MatchPhase::Resetting, current_tick);
                }
            }
            MatchPhase::Resetting => {
                // Resetting phase handled externally (world reset, arena load)
                // Call complete_reset() when done
            }
        }

        if self.state.phase != old_phase {
            Some(self.state.phase)
        } else {
            None
        }
    }

    /// Start playing phase
    fn start_playing(&mut self, tick: Tick) {
        self.time_ticks_remaining = self.config.time_limit_seconds * 60;
        self.state.time_remaining = self.config.time_limit_seconds;
        self.state.player_scores.clear();
        self.state.winner = None;
        self.match_end_result = None;
        self.transition_to(MatchPhase::Playing, tick);
    }

    /// Check if a player reached the score limit
    pub fn check_score_limit(
        &mut self,
        player_id: PlayerId,
        kills: u16,
        current_tick: Tick,
    ) -> bool {
        if self.state.phase != MatchPhase::Playing {
            return false;
        }
        if kills >= self.config.score_limit {
            self.end_match_score_limit(player_id, current_tick);
            true
        } else {
            false
        }
    }

    /// End match due to score limit reached
    fn end_match_score_limit(&mut self, winner: PlayerId, tick: Tick) {
        self.state.winner = Some(winner);
        self.match_end_result = Some(MatchEndResult {
            winner: Some(winner),
            scores: self.state.player_scores.clone(),
            reason: MatchEndReason::ScoreLimit,
        });
        self.end_screen_ticks_remaining = self.config.end_screen_ticks;
        self.transition_to(MatchPhase::EndScreen, tick);
    }

    /// End match due to time limit
    fn end_match_time_limit(&mut self, tick: Tick) {
        // Find winner (highest kills) or tie
        let winner = self.determine_winner();
        self.state.winner = winner;
        self.match_end_result = Some(MatchEndResult {
            winner,
            scores: self.state.player_scores.clone(),
            reason: MatchEndReason::TimeLimit,
        });
        self.end_screen_ticks_remaining = self.config.end_screen_ticks;
        self.transition_to(MatchPhase::EndScreen, tick);
    }

    /// End match due to forfeit (all players disconnected)
    pub fn end_match_forfeit(&mut self, tick: Tick) {
        if self.state.phase != MatchPhase::Playing {
            return;
        }
        self.state.winner = None;
        self.match_end_result = Some(MatchEndResult {
            winner: None,
            scores: self.state.player_scores.clone(),
            reason: MatchEndReason::Forfeit,
        });
        self.end_screen_ticks_remaining = self.config.end_screen_ticks;
        self.transition_to(MatchPhase::EndScreen, tick);
    }

    /// Determine winner based on current scores
    fn determine_winner(&self) -> Option<PlayerId> {
        if self.state.player_scores.is_empty() {
            return None;
        }

        let max_kills = self.state.player_scores.iter().map(|p| p.kills).max()?;
        let leaders: Vec<_> = self
            .state
            .player_scores
            .iter()
            .filter(|p| p.kills == max_kills)
            .collect();

        if leaders.len() == 1 {
            Some(leaders[0].player_id)
        } else {
            None // Tie
        }
    }

    /// Complete the reset phase and return to lobby
    /// Returns the next arena name if rotation is enabled
    pub fn complete_reset(&mut self, tick: Tick) -> Option<String> {
        if self.state.phase != MatchPhase::Resetting {
            return None;
        }

        // Advance arena rotation
        let next_arena = if !self.config.arena_rotation.is_empty() {
            self.arena_index = (self.arena_index + 1) % self.config.arena_rotation.len();
            let arena_name = self.config.arena_rotation[self.arena_index].clone();
            self.state.arena_name = arena_name.clone();
            Some(arena_name)
        } else {
            None
        };

        // Reset match state
        self.state.phase = MatchPhase::Lobby;
        self.state.countdown_remaining = (self.config.countdown_ticks / 60).max(1) as u8;
        self.state.time_remaining = self.config.time_limit_seconds;
        self.state.player_scores.clear();
        self.state.winner = None;
        self.match_end_result = None;
        self.time_ticks_remaining = self.config.time_limit_seconds * 60;
        self.phase_start_tick = tick;

        // Reset team scores
        for score in &mut self.state.scores {
            score.score = 0;
        }

        next_arena
    }

    /// Transition to a new phase
    fn transition_to(&mut self, phase: MatchPhase, tick: Tick) {
        self.state.phase = phase;
        self.phase_start_tick = tick;
    }

    /// Get countdown seconds remaining (for CountdownTick events)
    pub fn countdown_seconds(&self) -> u8 {
        self.state.countdown_remaining
    }

    /// Check if countdown tick should fire (every 60 ticks during countdown)
    pub fn should_broadcast_countdown(&self) -> bool {
        self.state.phase == MatchPhase::Countdown && self.countdown_ticks_remaining % 60 == 0
    }

    /// Update player score in state
    pub fn update_player_score(
        &mut self,
        player_id: PlayerId,
        name: &str,
        kills: u16,
        deaths: u16,
    ) {
        if let Some(score) = self
            .state
            .player_scores
            .iter_mut()
            .find(|s| s.player_id == player_id)
        {
            score.kills = kills;
            score.deaths = deaths;
        } else {
            self.state.player_scores.push(PlayerScore {
                player_id,
                name: name.to_string(),
                kills,
                deaths,
            });
        }
    }

    /// Remove player from scores (on disconnect)
    pub fn remove_player_score(&mut self, player_id: PlayerId) {
        self.state
            .player_scores
            .retain(|s| s.player_id != player_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> MatchConfig {
        MatchConfig {
            min_players: 2,
            countdown_ticks: 180,
            time_limit_seconds: 60,
            score_limit: 5,
            end_screen_ticks: 60,
            respawn_delay_ticks: 60,
            arena_rotation: Vec::new(),
        }
    }

    #[test]
    fn test_initial_state_is_lobby() {
        let state = MatchStateMachine::new(test_config(), "test_arena".to_string());
        assert_eq!(state.phase(), MatchPhase::Lobby);
    }

    #[test]
    fn test_lobby_to_countdown() {
        let mut state = MatchStateMachine::new(test_config(), "test_arena".to_string());
        assert_eq!(state.phase(), MatchPhase::Lobby);

        state.start_countdown(Tick(0));
        assert_eq!(state.phase(), MatchPhase::Countdown);
    }

    #[test]
    fn test_countdown_cancel_returns_to_lobby() {
        let mut state = MatchStateMachine::new(test_config(), "test_arena".to_string());
        state.start_countdown(Tick(0));
        assert_eq!(state.phase(), MatchPhase::Countdown);

        state.cancel_countdown(Tick(10));
        assert_eq!(state.phase(), MatchPhase::Lobby);
    }

    #[test]
    fn test_countdown_to_playing() {
        let config = MatchConfig {
            countdown_ticks: 10,
            ..test_config()
        };
        let mut state = MatchStateMachine::new(config, "test_arena".to_string());
        state.start_countdown(Tick(0));

        // Tick through countdown
        for i in 1..=10 {
            let phase_change = state.update(Tick(i));
            if i == 10 {
                assert_eq!(phase_change, Some(MatchPhase::Playing));
            } else {
                assert!(phase_change.is_none());
            }
        }
        assert_eq!(state.phase(), MatchPhase::Playing);
    }

    #[test]
    fn test_score_limit_ends_match() {
        let config = MatchConfig {
            countdown_ticks: 1,
            score_limit: 3,
            ..test_config()
        };
        let mut state = MatchStateMachine::new(config, "test_arena".to_string());
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Transition to Playing

        let player_id = PlayerId(1);
        state.update_player_score(player_id, "Player1", 2, 0);

        // Score limit not reached yet
        assert!(!state.check_score_limit(player_id, 2, Tick(2)));
        assert_eq!(state.phase(), MatchPhase::Playing);

        // Score limit reached
        state.update_player_score(player_id, "Player1", 3, 0);
        assert!(state.check_score_limit(player_id, 3, Tick(3)));
        assert_eq!(state.phase(), MatchPhase::EndScreen);
        assert_eq!(state.state().winner, Some(player_id));
    }

    #[test]
    fn test_time_limit_ends_match() {
        let config = MatchConfig {
            countdown_ticks: 1,
            time_limit_seconds: 1, // 1 second = 60 ticks
            ..test_config()
        };
        let mut state = MatchStateMachine::new(config, "test_arena".to_string());
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Transition to Playing

        // Add some scores
        state.update_player_score(PlayerId(1), "Player1", 2, 1);
        state.update_player_score(PlayerId(2), "Player2", 1, 2);

        // Tick through time limit
        for i in 2..=61 {
            state.update(Tick(i));
        }

        assert_eq!(state.phase(), MatchPhase::EndScreen);
        assert_eq!(state.state().winner, Some(PlayerId(1))); // Player1 has more kills
    }

    #[test]
    fn test_tie_at_time_limit() {
        let config = MatchConfig {
            countdown_ticks: 1,
            time_limit_seconds: 1,
            ..test_config()
        };
        let mut state = MatchStateMachine::new(config, "test_arena".to_string());
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Transition to Playing

        // Add tied scores
        state.update_player_score(PlayerId(1), "Player1", 2, 1);
        state.update_player_score(PlayerId(2), "Player2", 2, 1);

        // Tick through time limit
        for i in 2..=61 {
            state.update(Tick(i));
        }

        assert_eq!(state.phase(), MatchPhase::EndScreen);
        assert_eq!(state.state().winner, None); // Tie
    }

    #[test]
    fn test_endscreen_to_resetting() {
        let config = MatchConfig {
            countdown_ticks: 1,
            end_screen_ticks: 10,
            score_limit: 1,
            ..test_config()
        };
        let mut state = MatchStateMachine::new(config, "test_arena".to_string());
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Playing

        let player_id = PlayerId(1);
        state.update_player_score(player_id, "Player1", 1, 0);
        state.check_score_limit(player_id, 1, Tick(2)); // EndScreen

        // Tick through end screen
        for i in 3..=12 {
            let phase_change = state.update(Tick(i));
            if i == 12 {
                assert_eq!(phase_change, Some(MatchPhase::Resetting));
            }
        }
        assert_eq!(state.phase(), MatchPhase::Resetting);
    }

    #[test]
    fn test_resetting_to_lobby() {
        let config = MatchConfig {
            countdown_ticks: 1,
            end_screen_ticks: 1,
            score_limit: 1,
            ..test_config()
        };
        let mut state = MatchStateMachine::new(config, "test_arena".to_string());
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Playing

        let player_id = PlayerId(1);
        state.update_player_score(player_id, "Player1", 1, 0);
        state.check_score_limit(player_id, 1, Tick(2)); // EndScreen
        state.update(Tick(3)); // Resetting

        state.complete_reset(Tick(4));
        assert_eq!(state.phase(), MatchPhase::Lobby);
        assert!(state.state().player_scores.is_empty());
        assert_eq!(state.state().winner, None);
    }

    #[test]
    fn test_arena_rotation() {
        let config = MatchConfig {
            countdown_ticks: 1,
            end_screen_ticks: 1,
            score_limit: 1,
            arena_rotation: vec![
                "arena1".to_string(),
                "arena2".to_string(),
                "arena3".to_string(),
            ],
            ..test_config()
        };
        let mut state = MatchStateMachine::new(config, "arena1".to_string());

        // Complete a match cycle
        state.start_countdown(Tick(0));
        state.update(Tick(1));
        state.update_player_score(PlayerId(1), "P1", 1, 0);
        state.check_score_limit(PlayerId(1), 1, Tick(2));
        state.update(Tick(3));

        let next = state.complete_reset(Tick(4));
        assert_eq!(next, Some("arena2".to_string()));
        assert_eq!(state.state().arena_name, "arena2");

        // Another cycle
        state.start_countdown(Tick(5));
        state.update(Tick(6));
        state.update_player_score(PlayerId(1), "P1", 1, 0);
        state.check_score_limit(PlayerId(1), 1, Tick(7));
        state.update(Tick(8));

        let next = state.complete_reset(Tick(9));
        assert_eq!(next, Some("arena3".to_string()));

        // Wrap around
        state.start_countdown(Tick(10));
        state.update(Tick(11));
        state.update_player_score(PlayerId(1), "P1", 1, 0);
        state.check_score_limit(PlayerId(1), 1, Tick(12));
        state.update(Tick(13));

        let next = state.complete_reset(Tick(14));
        assert_eq!(next, Some("arena1".to_string()));
    }

    #[test]
    fn test_no_rotation_replays_same_arena() {
        let config = MatchConfig {
            countdown_ticks: 1,
            end_screen_ticks: 1,
            score_limit: 1,
            arena_rotation: Vec::new(),
            ..test_config()
        };
        let mut state = MatchStateMachine::new(config, "single_arena".to_string());

        state.start_countdown(Tick(0));
        state.update(Tick(1));
        state.update_player_score(PlayerId(1), "P1", 1, 0);
        state.check_score_limit(PlayerId(1), 1, Tick(2));
        state.update(Tick(3));

        let next = state.complete_reset(Tick(4));
        assert_eq!(next, None);
        assert_eq!(state.state().arena_name, "single_arena");
    }
}
