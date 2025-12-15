//! Per-player anti-cheat state tracking.

use plix_common::math::Vec3;
use plix_common::time::Tick;
use plix_common::types::InputSeq;

use super::config::AntiCheatConfig;
use super::{ActionType, InfractionType};

/// Window duration in ticks (60 ticks = 1 second at 60Hz)
const WINDOW_TICKS: u32 = 60;

/// Per-player anti-cheat tracking state.
/// Embedded in ServerPlayer for zero-cost access.
#[derive(Debug, Clone)]
pub struct AntiCheatState {
    // Infraction tracking
    /// Total strike count
    pub strikes: u32,
    /// Tick when last warning was sent (for rate limiting warnings)
    pub last_warning_tick: Option<Tick>,

    // Rate limiting (fixed window counters)
    /// Movement input count this window
    input_count: u32,
    /// Attack count this window
    attack_count: u32,
    /// Block edit count this window
    block_edit_count: u32,
    /// Ready toggle count this window
    ready_toggle_count: u32,
    /// Start tick of current rate limit window
    window_start_tick: Tick,

    // Sequence validation
    /// Last seen input sequence number
    last_input_seq: InputSeq,

    // Movement sanity (for Phase 5 / US3)
    /// Last valid server-calculated position
    pub last_valid_position: Option<Vec3>,
}

impl Default for AntiCheatState {
    fn default() -> Self {
        Self::new(Tick::ZERO)
    }
}

impl AntiCheatState {
    /// Create new state for a player connection
    pub fn new(current_tick: Tick) -> Self {
        Self {
            strikes: 0,
            last_warning_tick: None,
            input_count: 0,
            attack_count: 0,
            block_edit_count: 0,
            ready_toggle_count: 0,
            window_start_tick: current_tick,
            last_input_seq: InputSeq::default(),
            last_valid_position: None,
        }
    }

    /// Record an infraction, returns updated strike count
    pub fn record_infraction(&mut self, _infraction: InfractionType) -> u32 {
        self.strikes = self.strikes.saturating_add(1);
        self.strikes
    }

    /// Get current strike count
    pub fn strike_count(&self) -> u32 {
        self.strikes
    }

    /// Reset strikes (called after kick)
    pub fn reset_strikes(&mut self) {
        self.strikes = 0;
    }

    /// Check and update rate limit for an action.
    /// Returns true if action is allowed, false if rate limited.
    pub fn check_rate_limit(
        &mut self,
        action: ActionType,
        current_tick: Tick,
        config: &AntiCheatConfig,
    ) -> bool {
        // Reset window if needed
        self.maybe_reset_window(current_tick);

        let (count, limit) = match action {
            ActionType::Input => (&mut self.input_count, config.max_inputs_per_second),
            ActionType::Attack => (&mut self.attack_count, config.max_attacks_per_second),
            ActionType::BlockEdit => (
                &mut self.block_edit_count,
                config.max_block_edits_per_second,
            ),
            ActionType::ReadyToggle => (
                &mut self.ready_toggle_count,
                config.max_ready_toggles_per_second,
            ),
        };

        if *count >= limit {
            false
        } else {
            *count += 1;
            true
        }
    }

    /// Reset rate limit window if enough time has passed
    pub fn maybe_reset_window(&mut self, current_tick: Tick) {
        let elapsed = current_tick.0.wrapping_sub(self.window_start_tick.0);
        if elapsed >= WINDOW_TICKS {
            self.input_count = 0;
            self.attack_count = 0;
            self.block_edit_count = 0;
            self.ready_toggle_count = 0;
            self.window_start_tick = current_tick;
        }
    }

    /// Check if input sequence is valid (newer than last seen).
    /// Returns true if sequence is valid and should be accepted.
    /// Updates last_input_seq if valid.
    pub fn check_sequence(&mut self, seq: InputSeq) -> bool {
        if seq.is_newer_than(self.last_input_seq) {
            self.last_input_seq = seq;
            true
        } else {
            // Duplicate or out-of-order
            false
        }
    }

    /// Get the last seen input sequence
    pub fn last_sequence(&self) -> InputSeq {
        self.last_input_seq
    }

    /// Update last valid position after successful movement
    pub fn update_position(&mut self, pos: Vec3) {
        self.last_valid_position = Some(pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let state = AntiCheatState::new(Tick(100));
        assert_eq!(state.strikes, 0);
        assert_eq!(state.input_count, 0);
        assert!(state.last_valid_position.is_none());
    }

    #[test]
    fn test_record_infraction() {
        let mut state = AntiCheatState::new(Tick(0));
        assert_eq!(state.record_infraction(InfractionType::InvalidFloat), 1);
        assert_eq!(state.record_infraction(InfractionType::OutOfBounds), 2);
        assert_eq!(state.strikes, 2);
    }

    #[test]
    fn test_rate_limit_allows_under_limit() {
        let mut state = AntiCheatState::new(Tick(0));
        let config = AntiCheatConfig::default();

        // Should allow up to max_inputs_per_second
        for _ in 0..config.max_inputs_per_second {
            assert!(state.check_rate_limit(ActionType::Input, Tick(0), &config));
        }
    }

    #[test]
    fn test_rate_limit_rejects_over_limit() {
        let mut state = AntiCheatState::new(Tick(0));
        let config = AntiCheatConfig::default();

        // Exhaust the limit
        for _ in 0..config.max_inputs_per_second {
            state.check_rate_limit(ActionType::Input, Tick(0), &config);
        }

        // Should reject
        assert!(!state.check_rate_limit(ActionType::Input, Tick(0), &config));
    }

    #[test]
    fn test_rate_limit_window_reset() {
        let mut state = AntiCheatState::new(Tick(0));
        let config = AntiCheatConfig::default();

        // Exhaust the limit
        for _ in 0..config.max_inputs_per_second {
            state.check_rate_limit(ActionType::Input, Tick(0), &config);
        }

        // Should reject at tick 0
        assert!(!state.check_rate_limit(ActionType::Input, Tick(0), &config));

        // Should allow after window reset (60 ticks later)
        assert!(state.check_rate_limit(ActionType::Input, Tick(60), &config));
    }

    #[test]
    fn test_sequence_validation() {
        let mut state = AntiCheatState::new(Tick(0));

        // First sequence should be accepted
        assert!(state.check_sequence(InputSeq(1)));
        assert_eq!(state.last_sequence(), InputSeq(1));

        // Newer sequence should be accepted
        assert!(state.check_sequence(InputSeq(2)));
        assert_eq!(state.last_sequence(), InputSeq(2));

        // Duplicate should be rejected
        assert!(!state.check_sequence(InputSeq(2)));

        // Older should be rejected
        assert!(!state.check_sequence(InputSeq(1)));
    }
}
