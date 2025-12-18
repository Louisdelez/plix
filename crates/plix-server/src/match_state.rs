//! Match and round state management

use plix_common::protocol::{MatchEndReason, MatchPhase, MatchState, PlayerScore, TeamScore};
use plix_common::time::Tick;
use plix_common::types::{GameMode, PlayerId, TeamId};

/// Match configuration
#[derive(Debug, Clone)]
pub struct MatchConfig {
    /// Minimum players to start countdown
    pub min_players: u8,
    /// Countdown duration in ticks (default: 180 = 3s at 60Hz)
    pub countdown_ticks: u32,
    /// Match time limit in seconds (default: 300 = 5 minutes)
    pub time_limit_seconds: u32,
    /// Score limit to win (default: 5 kills for FFA, 25 for TDM)
    pub score_limit: u16,
    /// End screen duration in ticks (default: 300 = 5s at 60Hz, 900 = 15s for TDM)
    pub end_screen_ticks: u32,
    /// Respawn delay in ticks (default: 180 = 3s at 60Hz)
    pub respawn_delay_ticks: u32,
    /// Arena rotation list (empty = replay same arena)
    pub arena_rotation: Vec<String>,
    /// Maximum players per team (default: 8 for TDM)
    pub team_size: u8,
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
            team_size: 8, // 8 players per team
        }
    }
}

impl MatchConfig {
    /// Create TDM-specific default configuration
    pub fn tdm_default() -> Self {
        Self {
            min_players: 2,
            countdown_ticks: 180,     // 3 seconds at 60 Hz
            time_limit_seconds: 300,  // 5 minutes
            score_limit: 25,          // 25 team kills to win (TDM standard)
            end_screen_ticks: 900,    // 15 seconds at 60 Hz (show scoreboard)
            respawn_delay_ticks: 180, // 3 seconds at 60 Hz
            arena_rotation: Vec::new(),
            team_size: 8, // 8v8 max
        }
    }

    /// Create FFA-specific default configuration
    pub fn ffa_default() -> Self {
        Self {
            min_players: 2,
            countdown_ticks: 180,     // 3 seconds at 60 Hz
            time_limit_seconds: 300,  // 5 minutes
            score_limit: 15,          // 15 kills to win (FFA standard per FR-020)
            end_screen_ticks: 600,    // 10 seconds at 60 Hz (FR-022)
            respawn_delay_ticks: 180, // 3 seconds at 60 Hz (FR-021)
            arena_rotation: Vec::new(),
            team_size: 0, // Not applicable for FFA
        }
    }

    /// Create CTF-specific default configuration
    pub fn ctf_default() -> Self {
        Self {
            min_players: 2,
            countdown_ticks: 180,     // 3 seconds at 60 Hz
            time_limit_seconds: 600,  // 10 minutes (CTF matches are longer)
            score_limit: 3,           // 3 captures to win (CTF standard)
            end_screen_ticks: 600,    // 10 seconds at 60 Hz
            respawn_delay_ticks: 300, // 5 seconds at 60 Hz (longer respawn for CTF)
            arena_rotation: Vec::new(),
            team_size: 8, // 8v8 max
        }
    }

    /// Create BR Lite-specific default configuration
    pub fn br_lite_default() -> Self {
        Self {
            min_players: 4,          // 4 minimum for BR Lite (FR-001)
            countdown_ticks: 180,    // 3 seconds at 60 Hz
            time_limit_seconds: 480, // 8 minutes (~5 zone phases)
            score_limit: 1,          // Last player standing (not used - victory by elimination)
            end_screen_ticks: 600,   // 10 seconds at 60 Hz (post-match screen)
            respawn_delay_ticks: 0,  // No respawn in BR Lite (permanent elimination)
            arena_rotation: Vec::new(),
            team_size: 0, // Not applicable for BR Lite (FFA)
        }
    }

    /// Create Training mode-specific default configuration
    pub fn training_default() -> Self {
        Self {
            min_players: 1,          // Single player mode
            countdown_ticks: 0,      // No countdown - start immediately
            time_limit_seconds: 0,   // No time limit (unlimited session)
            score_limit: 0,          // No score limit (no victory condition)
            end_screen_ticks: 0,     // No end screen
            respawn_delay_ticks: 60, // 1 second at 60 Hz (fast respawn)
            arena_rotation: Vec::new(),
            team_size: 0, // Not applicable for Training
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
    pub fn new(config: MatchConfig, arena_name: String, game_mode: GameMode) -> Self {
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
                team_winner: None,
                game_mode,
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

    /// Get the current game mode
    pub fn game_mode(&self) -> GameMode {
        self.state.game_mode
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
                // NOTE: time_limit_seconds=0 means no time limit (Training mode)
                if self.config.time_limit_seconds > 0 && self.time_ticks_remaining == 0 {
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
    /// NOTE: score_limit=0 means no score limit (Training mode)
    pub fn check_score_limit(
        &mut self,
        player_id: PlayerId,
        kills: u16,
        current_tick: Tick,
    ) -> bool {
        if self.state.phase != MatchPhase::Playing {
            return false;
        }
        // score_limit=0 means no limit (Training mode)
        if self.config.score_limit == 0 {
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
        self.state.team_winner = None;
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

    // =========================================================================
    // TDM-specific methods (T005-T007)
    // =========================================================================

    /// Award a point to a team for a kill (TDM mode)
    ///
    /// Increments the team's score by 1. Only call this during Playing phase
    /// and for enemy kills (not friendly fire).
    pub fn award_team_kill(&mut self, team: TeamId) {
        if self.state.phase != MatchPhase::Playing {
            return;
        }
        if let Some(score) = self.state.scores.iter_mut().find(|s| s.team == team) {
            score.score += 1;
        }
    }

    /// Check if a team has reached the score limit (TDM mode)
    ///
    /// If the team's score >= score_limit, triggers match end and returns true.
    /// Only checks during Playing phase.
    pub fn check_team_score_limit(&mut self, team: TeamId, tick: Tick) -> bool {
        if self.state.phase != MatchPhase::Playing {
            return false;
        }
        if let Some(score) = self.state.scores.iter().find(|s| s.team == team) {
            if score.score >= self.config.score_limit as u32 {
                self.end_match_team_score_limit(team, tick);
                return true;
            }
        }
        false
    }

    /// End match due to team reaching score limit (TDM mode)
    ///
    /// Sets team_winner, transitions to EndScreen phase.
    fn end_match_team_score_limit(&mut self, team: TeamId, tick: Tick) {
        self.state.team_winner = Some(team);
        // For TDM, winner (player) is not set - only team_winner
        self.state.winner = None;
        self.match_end_result = Some(MatchEndResult {
            winner: None, // No individual winner in TDM
            scores: self.state.player_scores.clone(),
            reason: MatchEndReason::ScoreLimit,
        });
        self.end_screen_ticks_remaining = self.config.end_screen_ticks;
        self.transition_to(MatchPhase::EndScreen, tick);
    }

    /// Get team score by team ID
    pub fn get_team_score(&self, team: TeamId) -> u32 {
        self.state
            .scores
            .iter()
            .find(|s| s.team == team)
            .map(|s| s.score)
            .unwrap_or(0)
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
            team_size: 8,
        }
    }

    #[test]
    fn test_initial_state_is_lobby() {
        let state = MatchStateMachine::new(test_config(), "test_arena".to_string(), GameMode::Tdm);
        assert_eq!(state.phase(), MatchPhase::Lobby);
    }

    #[test]
    fn test_lobby_to_countdown() {
        let mut state =
            MatchStateMachine::new(test_config(), "test_arena".to_string(), GameMode::Tdm);
        assert_eq!(state.phase(), MatchPhase::Lobby);

        state.start_countdown(Tick(0));
        assert_eq!(state.phase(), MatchPhase::Countdown);
    }

    #[test]
    fn test_countdown_cancel_returns_to_lobby() {
        let mut state =
            MatchStateMachine::new(test_config(), "test_arena".to_string(), GameMode::Tdm);
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
        let mut state = MatchStateMachine::new(config, "test_arena".to_string(), GameMode::Tdm);
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
        let mut state = MatchStateMachine::new(config, "test_arena".to_string(), GameMode::Tdm);
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
        let mut state = MatchStateMachine::new(config, "test_arena".to_string(), GameMode::Tdm);
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
        let mut state = MatchStateMachine::new(config, "test_arena".to_string(), GameMode::Tdm);
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
        let mut state = MatchStateMachine::new(config, "test_arena".to_string(), GameMode::Tdm);
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
        let mut state = MatchStateMachine::new(config, "test_arena".to_string(), GameMode::Tdm);
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
        let mut state = MatchStateMachine::new(config, "arena1".to_string(), GameMode::Tdm);

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
        let mut state = MatchStateMachine::new(config, "single_arena".to_string(), GameMode::Tdm);

        state.start_countdown(Tick(0));
        state.update(Tick(1));
        state.update_player_score(PlayerId(1), "P1", 1, 0);
        state.check_score_limit(PlayerId(1), 1, Tick(2));
        state.update(Tick(3));

        let next = state.complete_reset(Tick(4));
        assert_eq!(next, None);
        assert_eq!(state.state().arena_name, "single_arena");
    }

    // =========================================================================
    // TDM-specific tests (T009)
    // =========================================================================

    #[test]
    fn test_award_team_kill_increments_score() {
        let config = MatchConfig {
            countdown_ticks: 1,
            ..test_config()
        };
        let mut state = MatchStateMachine::new(config, "test_arena".to_string(), GameMode::Tdm);
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Transition to Playing

        assert_eq!(state.get_team_score(TeamId::TEAM_0), 0);
        assert_eq!(state.get_team_score(TeamId::TEAM_1), 0);

        state.award_team_kill(TeamId::TEAM_0);
        assert_eq!(state.get_team_score(TeamId::TEAM_0), 1);
        assert_eq!(state.get_team_score(TeamId::TEAM_1), 0);

        state.award_team_kill(TeamId::TEAM_0);
        assert_eq!(state.get_team_score(TeamId::TEAM_0), 2);

        state.award_team_kill(TeamId::TEAM_1);
        assert_eq!(state.get_team_score(TeamId::TEAM_1), 1);
    }

    #[test]
    fn test_award_team_kill_only_in_playing_phase() {
        let config = MatchConfig {
            countdown_ticks: 10,
            ..test_config()
        };
        let mut state = MatchStateMachine::new(config, "test_arena".to_string(), GameMode::Tdm);

        // In Lobby phase - should not increment
        state.award_team_kill(TeamId::TEAM_0);
        assert_eq!(state.get_team_score(TeamId::TEAM_0), 0);

        // Start countdown - should not increment
        state.start_countdown(Tick(0));
        state.award_team_kill(TeamId::TEAM_0);
        assert_eq!(state.get_team_score(TeamId::TEAM_0), 0);
    }

    #[test]
    fn test_no_scoring_after_match_end() {
        let config = MatchConfig {
            countdown_ticks: 1,
            score_limit: 2,
            ..test_config()
        };
        let mut state = MatchStateMachine::new(config, "test_arena".to_string(), GameMode::Tdm);
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Playing

        // Award kills to reach limit
        state.award_team_kill(TeamId::TEAM_0);
        state.award_team_kill(TeamId::TEAM_0);
        state.check_team_score_limit(TeamId::TEAM_0, Tick(2));

        // Should now be in EndScreen
        assert_eq!(state.phase(), MatchPhase::EndScreen);

        // Try to award more kills - should NOT increment (T025)
        state.award_team_kill(TeamId::TEAM_0);
        assert_eq!(state.get_team_score(TeamId::TEAM_0), 2); // Still 2, not 3

        state.award_team_kill(TeamId::TEAM_1);
        assert_eq!(state.get_team_score(TeamId::TEAM_1), 0); // Still 0
    }

    #[test]
    fn test_check_team_score_limit_triggers_end() {
        let config = MatchConfig {
            countdown_ticks: 1,
            score_limit: 3, // Low limit for testing
            ..test_config()
        };
        let mut state = MatchStateMachine::new(config, "test_arena".to_string(), GameMode::Tdm);
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Transition to Playing

        // Award kills below limit
        state.award_team_kill(TeamId::TEAM_0);
        state.award_team_kill(TeamId::TEAM_0);
        assert!(!state.check_team_score_limit(TeamId::TEAM_0, Tick(2)));
        assert_eq!(state.phase(), MatchPhase::Playing);

        // Award kill to reach limit
        state.award_team_kill(TeamId::TEAM_0);
        assert!(state.check_team_score_limit(TeamId::TEAM_0, Tick(3)));
        assert_eq!(state.phase(), MatchPhase::EndScreen);
        assert_eq!(state.state().team_winner, Some(TeamId::TEAM_0));
    }

    #[test]
    fn test_check_team_score_limit_not_in_lobby() {
        let config = MatchConfig {
            score_limit: 1,
            ..test_config()
        };
        let mut state = MatchStateMachine::new(config, "test_arena".to_string(), GameMode::Tdm);

        // Manually set team score (bypassing phase check)
        state.state_mut().scores[0].score = 10;

        // Should not trigger in Lobby
        assert!(!state.check_team_score_limit(TeamId::TEAM_0, Tick(1)));
        assert_eq!(state.phase(), MatchPhase::Lobby);
    }

    #[test]
    fn test_team_scores_reset_on_complete_reset() {
        let config = MatchConfig {
            countdown_ticks: 1,
            end_screen_ticks: 1,
            score_limit: 3,
            ..test_config()
        };
        let mut state = MatchStateMachine::new(config, "test_arena".to_string(), GameMode::Tdm);
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Playing

        // Award some team kills
        state.award_team_kill(TeamId::TEAM_0);
        state.award_team_kill(TeamId::TEAM_0);
        state.award_team_kill(TeamId::TEAM_1);
        assert_eq!(state.get_team_score(TeamId::TEAM_0), 2);
        assert_eq!(state.get_team_score(TeamId::TEAM_1), 1);

        // End match
        state.award_team_kill(TeamId::TEAM_0);
        state.check_team_score_limit(TeamId::TEAM_0, Tick(2));
        state.update(Tick(3)); // Resetting

        // Complete reset should clear scores
        state.complete_reset(Tick(4));
        assert_eq!(state.get_team_score(TeamId::TEAM_0), 0);
        assert_eq!(state.get_team_score(TeamId::TEAM_1), 0);
        assert_eq!(state.state().team_winner, None);
    }

    // =========================================================================
    // FFA-specific tests (T019-T021)
    // =========================================================================

    fn ffa_test_config() -> MatchConfig {
        MatchConfig {
            min_players: 2,
            countdown_ticks: 1,
            time_limit_seconds: 60,
            score_limit: 15, // FFA default
            end_screen_ticks: 60,
            respawn_delay_ticks: 60,
            arena_rotation: Vec::new(),
            team_size: 0, // FFA: no teams
        }
    }

    #[test]
    fn test_ffa_kill_increments_attacker_score() {
        // T019: FFA kill increments attacker score by 1
        let config = ffa_test_config();
        let mut state = MatchStateMachine::new(config, "ffa_arena".to_string(), GameMode::Ffa);
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Transition to Playing

        let attacker = PlayerId(1);

        // Initial state - no kills
        assert!(state.state().player_scores.is_empty());

        // Record first kill
        state.update_player_score(attacker, "Attacker", 1, 0);
        let score = state
            .state()
            .player_scores
            .iter()
            .find(|s| s.player_id == attacker);
        assert!(score.is_some());
        assert_eq!(score.unwrap().kills, 1);

        // Record second kill
        state.update_player_score(attacker, "Attacker", 2, 0);
        let score = state
            .state()
            .player_scores
            .iter()
            .find(|s| s.player_id == attacker);
        assert_eq!(score.unwrap().kills, 2);
    }

    #[test]
    fn test_ffa_scoring_only_in_playing_phase() {
        // T021: FFA scoring only occurs in Playing phase
        let config = ffa_test_config();
        let mut state = MatchStateMachine::new(config, "ffa_arena".to_string(), GameMode::Ffa);

        let player = PlayerId(1);

        // Lobby phase - check_score_limit should not trigger match end
        state.update_player_score(player, "Player1", 15, 0);
        assert!(!state.check_score_limit(player, 15, Tick(1)));
        assert_eq!(state.phase(), MatchPhase::Lobby);

        // Start countdown
        state.start_countdown(Tick(2));

        // Countdown phase - check_score_limit should not trigger match end
        assert!(!state.check_score_limit(player, 15, Tick(3)));
        assert_eq!(state.phase(), MatchPhase::Countdown);
    }

    #[test]
    fn test_ffa_score_limit_ends_match_with_winner() {
        // Tests that FFA score limit triggers match end with individual winner
        let config = MatchConfig {
            score_limit: 3, // Low limit for testing
            ..ffa_test_config()
        };
        let mut state = MatchStateMachine::new(config, "ffa_arena".to_string(), GameMode::Ffa);
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Transition to Playing

        let player = PlayerId(1);

        // Below score limit - should not end match
        state.update_player_score(player, "Player1", 2, 0);
        assert!(!state.check_score_limit(player, 2, Tick(2)));
        assert_eq!(state.phase(), MatchPhase::Playing);

        // At score limit - should end match
        state.update_player_score(player, "Player1", 3, 0);
        assert!(state.check_score_limit(player, 3, Tick(3)));
        assert_eq!(state.phase(), MatchPhase::EndScreen);
        assert_eq!(state.state().winner, Some(player));
        // FFA should NOT set team_winner
        assert_eq!(state.state().team_winner, None);
    }

    #[test]
    fn test_ffa_game_mode_stored_correctly() {
        // Verify FFA game mode is stored and retrievable
        let config = ffa_test_config();
        let state = MatchStateMachine::new(config, "ffa_arena".to_string(), GameMode::Ffa);

        assert_eq!(state.game_mode(), GameMode::Ffa);
        assert_eq!(state.state().game_mode, GameMode::Ffa);
    }

    #[test]
    fn test_ffa_defaults_applied() {
        // Verify FFA default config values
        let config = MatchConfig::ffa_default();

        assert_eq!(config.score_limit, 15); // 15 kills to win
        assert_eq!(config.end_screen_ticks, 600); // 10 seconds
        assert_eq!(config.respawn_delay_ticks, 180); // 3 seconds
        assert_eq!(config.team_size, 0); // No teams
    }

    #[test]
    fn test_ffa_multiple_players_highest_score_wins() {
        // Test that player with highest kills wins when time expires
        let config = MatchConfig {
            countdown_ticks: 1,
            time_limit_seconds: 1, // 1 second = 60 ticks
            ..ffa_test_config()
        };
        let mut state = MatchStateMachine::new(config, "ffa_arena".to_string(), GameMode::Ffa);
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Transition to Playing

        // Add player scores - Player 2 has most kills
        state.update_player_score(PlayerId(1), "Player1", 5, 2);
        state.update_player_score(PlayerId(2), "Player2", 8, 3);
        state.update_player_score(PlayerId(3), "Player3", 3, 4);

        // Tick through time limit
        for i in 2..=61 {
            state.update(Tick(i));
        }

        assert_eq!(state.phase(), MatchPhase::EndScreen);
        assert_eq!(state.state().winner, Some(PlayerId(2))); // Player 2 wins with 8 kills
    }

    // =========================================================================
    // FFA Match Cycle Tests (T041, T042)
    // =========================================================================

    #[test]
    fn test_ffa_match_resets_scores_after_endscreen() {
        // T041: FFA match resets scores after EndScreen
        let config = MatchConfig {
            countdown_ticks: 1,
            end_screen_ticks: 1,
            score_limit: 3,
            ..ffa_test_config()
        };
        let mut state = MatchStateMachine::new(config, "ffa_arena".to_string(), GameMode::Ffa);
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Playing

        // Award kills
        state.update_player_score(PlayerId(1), "Player1", 2, 0);
        state.update_player_score(PlayerId(2), "Player2", 1, 1);
        assert_eq!(state.state().player_scores.len(), 2);

        // Reach score limit
        state.update_player_score(PlayerId(1), "Player1", 3, 0);
        state.check_score_limit(PlayerId(1), 3, Tick(2));
        assert_eq!(state.phase(), MatchPhase::EndScreen);
        assert_eq!(state.state().winner, Some(PlayerId(1)));

        // Tick through end screen
        state.update(Tick(3)); // Resetting

        // Complete reset should clear scores
        state.complete_reset(Tick(4));
        assert!(state.state().player_scores.is_empty());
        assert_eq!(state.state().winner, None);
    }

    #[test]
    fn test_ffa_match_returns_to_lobby_after_reset() {
        // T042: FFA match returns to Lobby after reset
        let config = MatchConfig {
            countdown_ticks: 1,
            end_screen_ticks: 1,
            score_limit: 1,
            ..ffa_test_config()
        };
        let mut state = MatchStateMachine::new(config, "ffa_arena".to_string(), GameMode::Ffa);

        // Full cycle: Lobby -> Countdown -> Playing -> EndScreen -> Resetting -> Lobby
        assert_eq!(state.phase(), MatchPhase::Lobby);

        state.start_countdown(Tick(0));
        assert_eq!(state.phase(), MatchPhase::Countdown);

        state.update(Tick(1)); // Playing
        assert_eq!(state.phase(), MatchPhase::Playing);

        // Score to end match
        state.update_player_score(PlayerId(1), "P1", 1, 0);
        state.check_score_limit(PlayerId(1), 1, Tick(2));
        assert_eq!(state.phase(), MatchPhase::EndScreen);

        state.update(Tick(3)); // Resetting
        assert_eq!(state.phase(), MatchPhase::Resetting);

        state.complete_reset(Tick(4));
        assert_eq!(state.phase(), MatchPhase::Lobby);
    }

    #[test]
    fn test_ffa_complete_match_cycle() {
        // Complete FFA match cycle test with multiple players
        let config = MatchConfig {
            countdown_ticks: 1,
            end_screen_ticks: 1,
            score_limit: 5,
            ..ffa_test_config()
        };
        let mut state = MatchStateMachine::new(config, "ffa_arena".to_string(), GameMode::Ffa);

        // Start match
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Playing

        // Multiple players scoring
        state.update_player_score(PlayerId(1), "Alice", 3, 1);
        state.update_player_score(PlayerId(2), "Bob", 2, 2);
        state.update_player_score(PlayerId(3), "Charlie", 1, 3);

        // Alice reaches score limit
        state.update_player_score(PlayerId(1), "Alice", 5, 1);
        let ended = state.check_score_limit(PlayerId(1), 5, Tick(10));
        assert!(ended);
        assert_eq!(state.phase(), MatchPhase::EndScreen);
        assert_eq!(state.state().winner, Some(PlayerId(1)));
        assert_eq!(state.state().team_winner, None); // FFA has no team winner

        // End screen and reset
        state.update(Tick(11)); // Resetting
        state.complete_reset(Tick(12));

        // Back to lobby with clean state
        assert_eq!(state.phase(), MatchPhase::Lobby);
        assert!(state.state().player_scores.is_empty());
        assert_eq!(state.state().winner, None);

        // Game mode should persist
        assert_eq!(state.game_mode(), GameMode::Ffa);
    }

    // =========================================================================
    // TDM Non-Regression Tests (T056-T058)
    // =========================================================================

    #[test]
    fn test_tdm_scoring_uses_team_scoring_not_individual() {
        // T056: TDM scoring still uses team scoring (not FFA individual)
        let config = MatchConfig {
            countdown_ticks: 1,
            score_limit: 3,
            ..MatchConfig::tdm_default()
        };
        let mut state = MatchStateMachine::new(config, "tdm_arena".to_string(), GameMode::Tdm);
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Playing

        // In TDM, team score should be tracked separately from individual score
        assert_eq!(state.get_team_score(TeamId::TEAM_0), 0);
        assert_eq!(state.get_team_score(TeamId::TEAM_1), 0);

        // Award team kills (TDM mechanic)
        state.award_team_kill(TeamId::TEAM_0);
        state.award_team_kill(TeamId::TEAM_0);
        assert_eq!(state.get_team_score(TeamId::TEAM_0), 2);

        // Individual scores don't affect team score
        state.update_player_score(PlayerId(1), "Player1", 5, 0);
        assert_eq!(state.get_team_score(TeamId::TEAM_0), 2); // Still 2, not 5
    }

    #[test]
    fn test_tdm_match_uses_team_winner_not_individual() {
        // T058: TDM match uses team_winner not winner
        let config = MatchConfig {
            countdown_ticks: 1,
            score_limit: 2,
            ..MatchConfig::tdm_default()
        };
        let mut state = MatchStateMachine::new(config, "tdm_arena".to_string(), GameMode::Tdm);
        state.start_countdown(Tick(0));
        state.update(Tick(1)); // Playing

        // Award team kills to reach score limit
        state.award_team_kill(TeamId::TEAM_0);
        state.award_team_kill(TeamId::TEAM_0);
        state.check_team_score_limit(TeamId::TEAM_0, Tick(2));

        assert_eq!(state.phase(), MatchPhase::EndScreen);
        // TDM sets team_winner, not winner
        assert_eq!(state.state().team_winner, Some(TeamId::TEAM_0));
        assert_eq!(state.state().winner, None); // Individual winner is None in TDM
    }

    #[test]
    fn test_tdm_game_mode_stored_correctly() {
        // Verify TDM game mode is stored and retrievable
        let config = MatchConfig::tdm_default();
        let state = MatchStateMachine::new(config, "tdm_arena".to_string(), GameMode::Tdm);

        assert_eq!(state.game_mode(), GameMode::Tdm);
        assert_eq!(state.state().game_mode, GameMode::Tdm);
    }

    #[test]
    fn test_tdm_defaults_applied() {
        // Verify TDM default config values
        let config = MatchConfig::tdm_default();

        assert_eq!(config.score_limit, 25); // 25 team kills to win
        assert_eq!(config.end_screen_ticks, 900); // 15 seconds
        assert_eq!(config.team_size, 8); // 8v8 max
    }
}
