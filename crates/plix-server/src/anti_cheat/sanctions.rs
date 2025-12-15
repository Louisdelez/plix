//! Sanction management for anti-cheat system.
//!
//! Implements progressive sanctions: warning -> kick -> ban
//! based on accumulated infraction strikes.

use plix_common::time::Tick;

use super::config::AntiCheatConfig;
use super::AntiCheatState;

/// Rate limit warnings to once every 5 seconds (300 ticks at 60Hz)
const WARNING_RATE_LIMIT_TICKS: u32 = 300;

/// Types of sanctions that can be applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SanctionType {
    /// Send warning message to player (rate-limited)
    Warning,
    /// Kick player from server
    Kick,
    /// Temporarily ban player IP
    TempBan,
}

/// Evaluate current strike count and return appropriate sanction.
/// Returns None if no sanction should be applied.
pub fn evaluate(strikes: u32, config: &AntiCheatConfig) -> Option<SanctionType> {
    if strikes >= config.ban_threshold {
        Some(SanctionType::TempBan)
    } else if strikes >= config.kick_threshold {
        Some(SanctionType::Kick)
    } else if strikes >= config.warning_threshold {
        Some(SanctionType::Warning)
    } else {
        None
    }
}

/// Check if a warning should be sent (rate limited).
/// Returns true if enough time has passed since last warning.
pub fn should_warn(state: &AntiCheatState, current_tick: Tick) -> bool {
    match state.last_warning_tick {
        Some(last_tick) => {
            let elapsed = current_tick.0.wrapping_sub(last_tick.0);
            elapsed >= WARNING_RATE_LIMIT_TICKS
        }
        None => true, // Never warned before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_no_sanction() {
        let config = AntiCheatConfig::default();
        assert!(evaluate(0, &config).is_none());
        assert!(evaluate(config.warning_threshold - 1, &config).is_none());
    }

    #[test]
    fn test_evaluate_warning() {
        let config = AntiCheatConfig::default();
        assert_eq!(
            evaluate(config.warning_threshold, &config),
            Some(SanctionType::Warning)
        );
        assert_eq!(
            evaluate(config.warning_threshold + 1, &config),
            Some(SanctionType::Warning)
        );
    }

    #[test]
    fn test_evaluate_kick() {
        let config = AntiCheatConfig::default();
        assert_eq!(
            evaluate(config.kick_threshold, &config),
            Some(SanctionType::Kick)
        );
        assert_eq!(
            evaluate(config.kick_threshold + 1, &config),
            Some(SanctionType::Kick)
        );
    }

    #[test]
    fn test_evaluate_ban() {
        let config = AntiCheatConfig::default();
        assert_eq!(
            evaluate(config.ban_threshold, &config),
            Some(SanctionType::TempBan)
        );
        assert_eq!(
            evaluate(config.ban_threshold + 5, &config),
            Some(SanctionType::TempBan)
        );
    }

    #[test]
    fn test_should_warn_first_time() {
        let state = AntiCheatState::new(Tick(0));
        assert!(should_warn(&state, Tick(0)));
    }

    #[test]
    fn test_should_warn_rate_limited() {
        let mut state = AntiCheatState::new(Tick(0));
        state.last_warning_tick = Some(Tick(100));

        // Too soon (only 50 ticks later)
        assert!(!should_warn(&state, Tick(150)));

        // Enough time passed (300 ticks later)
        assert!(should_warn(&state, Tick(400)));
    }
}
