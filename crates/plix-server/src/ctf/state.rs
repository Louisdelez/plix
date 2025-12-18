//! CTF game state management
//!
//! Manages the complete state of a CTF match including flags, zones, and capture scores.

use plix_common::math::Vec3;
use plix_common::types::{Flag, FlagState, FlagZone, FlagZoneType, TeamId};

use super::CtfConfig;

/// Complete CTF game state
#[derive(Debug, Clone)]
pub struct CtfState {
    /// Flags for each team (indexed by TeamId: 0 = Team 0's flag, 1 = Team 1's flag)
    pub flags: [Flag; 2],
    /// Capture scores for each team
    pub capture_scores: [u32; 2],
    /// Flag zones loaded from arena
    pub zones: Vec<FlagZone>,
    /// Configuration
    pub config: CtfConfig,
}

impl CtfState {
    /// Create new CTF state from arena zones and config
    pub fn new(zones: Vec<FlagZone>, config: CtfConfig) -> Self {
        // Find flag base positions from zones
        let team0_base = zones
            .iter()
            .find(|z| z.team == TeamId::TEAM_0 && z.zone_type == FlagZoneType::FlagBase)
            .map(|z| z.center())
            .unwrap_or(Vec3::ZERO);

        let team1_base = zones
            .iter()
            .find(|z| z.team == TeamId::TEAM_1 && z.zone_type == FlagZoneType::FlagBase)
            .map(|z| z.center())
            .unwrap_or(Vec3::ZERO);

        Self {
            flags: [
                Flag::new(TeamId::TEAM_0, team0_base),
                Flag::new(TeamId::TEAM_1, team1_base),
            ],
            capture_scores: [0, 0],
            zones,
            config,
        }
    }

    /// Get flag reference for a team
    pub fn flag(&self, team: TeamId) -> &Flag {
        &self.flags[team.0 as usize]
    }

    /// Get mutable flag reference for a team
    pub fn flag_mut(&mut self, team: TeamId) -> &mut Flag {
        &mut self.flags[team.0 as usize]
    }

    /// Get capture score for a team
    pub fn score(&self, team: TeamId) -> u32 {
        self.capture_scores[team.0 as usize]
    }

    /// Increment capture score for a team
    pub fn increment_score(&mut self, team: TeamId) {
        self.capture_scores[team.0 as usize] += 1;
    }

    /// Get flag base zone for a team
    pub fn flag_base(&self, team: TeamId) -> Option<&FlagZone> {
        self.zones
            .iter()
            .find(|z| z.team == team && z.zone_type == FlagZoneType::FlagBase)
    }

    /// Get capture zone for a team
    pub fn capture_zone(&self, team: TeamId) -> Option<&FlagZone> {
        self.zones
            .iter()
            .find(|z| z.team == team && z.zone_type == FlagZoneType::CaptureZone)
    }

    /// Reset state for new match (flags to base, scores to 0)
    pub fn reset(&mut self) {
        for flag in &mut self.flags {
            flag.state = FlagState::AtBase;
        }
        self.capture_scores = [0, 0];
    }

    /// Check if a player is carrying any flag
    pub fn is_carrying_flag(&self, player_id: plix_common::types::PlayerId) -> bool {
        self.flags.iter().any(|f| f.carrier() == Some(player_id))
    }

    /// Get the flag being carried by a player, if any
    pub fn flag_carried_by(&self, player_id: plix_common::types::PlayerId) -> Option<TeamId> {
        for (idx, flag) in self.flags.iter().enumerate() {
            if flag.carrier() == Some(player_id) {
                return Some(if idx == 0 {
                    TeamId::TEAM_0
                } else {
                    TeamId::TEAM_1
                });
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::types::PlayerId;

    fn test_zones() -> Vec<FlagZone> {
        vec![
            FlagZone::new(
                TeamId::TEAM_0,
                FlagZoneType::FlagBase,
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(10.0, 4.0, 10.0),
            ),
            FlagZone::new(
                TeamId::TEAM_1,
                FlagZoneType::FlagBase,
                Vec3::new(50.0, 0.0, 50.0),
                Vec3::new(60.0, 4.0, 60.0),
            ),
            FlagZone::new(
                TeamId::TEAM_0,
                FlagZoneType::CaptureZone,
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(12.0, 4.0, 12.0),
            ),
            FlagZone::new(
                TeamId::TEAM_1,
                FlagZoneType::CaptureZone,
                Vec3::new(48.0, 0.0, 48.0),
                Vec3::new(60.0, 4.0, 60.0),
            ),
        ]
    }

    #[test]
    fn test_ctf_state_new() {
        let state = CtfState::new(test_zones(), CtfConfig::default());

        // Flags should be at base
        assert!(state.flag(TeamId::TEAM_0).is_at_base());
        assert!(state.flag(TeamId::TEAM_1).is_at_base());

        // Scores should be zero
        assert_eq!(state.score(TeamId::TEAM_0), 0);
        assert_eq!(state.score(TeamId::TEAM_1), 0);

        // Flag positions should be at zone centers
        assert_eq!(
            state.flag(TeamId::TEAM_0).base_position,
            Vec3::new(5.0, 2.0, 5.0)
        );
        assert_eq!(
            state.flag(TeamId::TEAM_1).base_position,
            Vec3::new(55.0, 2.0, 55.0)
        );
    }

    #[test]
    fn test_ctf_state_reset() {
        let mut state = CtfState::new(test_zones(), CtfConfig::default());

        // Modify state
        state.flags[0].state = FlagState::Carried {
            carrier: PlayerId(1),
        };
        state.capture_scores = [2, 1];

        // Reset
        state.reset();

        // Verify reset
        assert!(state.flag(TeamId::TEAM_0).is_at_base());
        assert!(state.flag(TeamId::TEAM_1).is_at_base());
        assert_eq!(state.score(TeamId::TEAM_0), 0);
        assert_eq!(state.score(TeamId::TEAM_1), 0);
    }

    #[test]
    fn test_is_carrying_flag() {
        let mut state = CtfState::new(test_zones(), CtfConfig::default());

        let player = PlayerId(5);
        assert!(!state.is_carrying_flag(player));

        state.flags[1].state = FlagState::Carried { carrier: player };
        assert!(state.is_carrying_flag(player));
    }

    #[test]
    fn test_flag_carried_by() {
        let mut state = CtfState::new(test_zones(), CtfConfig::default());

        let player = PlayerId(7);
        assert_eq!(state.flag_carried_by(player), None);

        state.flags[0].state = FlagState::Carried { carrier: player };
        assert_eq!(state.flag_carried_by(player), Some(TeamId::TEAM_0));
    }

    #[test]
    fn test_zone_lookup() {
        let state = CtfState::new(test_zones(), CtfConfig::default());

        let base = state.flag_base(TeamId::TEAM_0).unwrap();
        assert_eq!(base.team, TeamId::TEAM_0);
        assert_eq!(base.zone_type, FlagZoneType::FlagBase);

        let capture = state.capture_zone(TeamId::TEAM_1).unwrap();
        assert_eq!(capture.team, TeamId::TEAM_1);
        assert_eq!(capture.zone_type, FlagZoneType::CaptureZone);
    }
}
