//! CTF (Capture The Flag) game mode implementation
//!
//! This module implements the CTF objective-based game mode where two teams
//! compete to capture the enemy flag and return it to their base.
//!
//! # Architecture
//!
//! - `state`: CTF game state (flags, zones, scores)
//! - `rules`: Game rules engine (pickup, drop, capture, return logic)
//! - `coordinator`: Event orchestration and integration with game loop

pub mod coordinator;
pub mod rules;
pub mod state;

pub use coordinator::{CtfCoordinator, CtfEvent, ReturnReason};
pub use rules::CtfRules;
pub use state::CtfState;

use plix_common::types::{FlagZone, FlagZoneType, TeamId};

/// Configuration for CTF game mode
#[derive(Debug, Clone)]
pub struct CtfConfig {
    /// Number of captures to win (default: 3)
    pub capture_limit: u16,
    /// Ticks before dropped flag returns to base (default: 600 = 10s at 60Hz)
    pub flag_return_delay_ticks: u32,
    /// Ticks before respawning after death (default: 300 = 5s at 60Hz)
    pub respawn_delay_ticks: u32,
    /// Match time limit in seconds (default: 600 = 10 minutes)
    pub time_limit_seconds: u32,
    /// End screen duration in ticks (default: 600 = 10s at 60Hz)
    pub end_screen_ticks: u32,
}

impl Default for CtfConfig {
    fn default() -> Self {
        Self {
            capture_limit: 3,
            flag_return_delay_ticks: 600, // 10 seconds at 60Hz
            respawn_delay_ticks: 300,     // 5 seconds at 60Hz
            time_limit_seconds: 600,      // 10 minutes
            end_screen_ticks: 600,        // 10 seconds at 60Hz
        }
    }
}

impl CtfConfig {
    /// Create config from arena settings, using defaults for unspecified values
    pub fn from_arena(
        capture_limit: Option<u16>,
        flag_return_delay: Option<u32>,
        respawn_delay: Option<u32>,
        time_limit: Option<u32>,
    ) -> Self {
        let defaults = Self::default();
        Self {
            capture_limit: capture_limit.unwrap_or(defaults.capture_limit),
            flag_return_delay_ticks: flag_return_delay
                .map(|s| s * 60) // Convert seconds to ticks
                .unwrap_or(defaults.flag_return_delay_ticks),
            respawn_delay_ticks: respawn_delay
                .map(|s| s * 60)
                .unwrap_or(defaults.respawn_delay_ticks),
            time_limit_seconds: time_limit.unwrap_or(defaults.time_limit_seconds),
            end_screen_ticks: defaults.end_screen_ticks,
        }
    }
}

/// Helper to get enemy team
pub fn enemy_team(team: TeamId) -> TeamId {
    if team == TeamId::TEAM_0 {
        TeamId::TEAM_1
    } else {
        TeamId::TEAM_0
    }
}

/// Find flag base zone for a team from zone list
pub fn find_flag_base(zones: &[FlagZone], team: TeamId) -> Option<&FlagZone> {
    zones
        .iter()
        .find(|z| z.team == team && z.zone_type == FlagZoneType::FlagBase)
}

/// Find capture zone for a team from zone list
pub fn find_capture_zone(zones: &[FlagZone], team: TeamId) -> Option<&FlagZone> {
    zones
        .iter()
        .find(|z| z.team == team && z.zone_type == FlagZoneType::CaptureZone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctf_config_default() {
        let config = CtfConfig::default();
        assert_eq!(config.capture_limit, 3);
        assert_eq!(config.flag_return_delay_ticks, 600);
        assert_eq!(config.respawn_delay_ticks, 300);
        assert_eq!(config.time_limit_seconds, 600);
    }

    #[test]
    fn test_ctf_config_from_arena() {
        let config = CtfConfig::from_arena(Some(5), Some(15), None, Some(900));
        assert_eq!(config.capture_limit, 5);
        assert_eq!(config.flag_return_delay_ticks, 900); // 15 * 60
        assert_eq!(config.respawn_delay_ticks, 300); // default
        assert_eq!(config.time_limit_seconds, 900);
    }

    #[test]
    fn test_enemy_team() {
        assert_eq!(enemy_team(TeamId::TEAM_0), TeamId::TEAM_1);
        assert_eq!(enemy_team(TeamId::TEAM_1), TeamId::TEAM_0);
    }
}
