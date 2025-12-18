//! Spawn point management

use plix_common::math::Vec3;
use plix_common::types::TeamId;

use crate::format::SpawnPoint;

/// Manages spawn point selection
#[derive(Debug)]
pub struct SpawnManager {
    /// Spawn points indexed by team
    team_spawns: Vec<Vec<SpawnPoint>>,
    /// Last used spawn index per team
    last_spawn: Vec<usize>,
    /// All spawn points for FFA mode (T022)
    all_spawns: Vec<SpawnPoint>,
    /// Last used FFA spawn index (T023)
    last_ffa_spawn: usize,
}

impl SpawnManager {
    /// Create a new spawn manager from spawn points
    pub fn new(spawn_points: Vec<SpawnPoint>) -> Self {
        // Find max team ID
        let max_team = spawn_points.iter().map(|sp| sp.team).max().unwrap_or(0);

        // Initialize spawn lists per team
        let mut team_spawns = vec![Vec::new(); (max_team + 1) as usize];
        let last_spawn = vec![0; (max_team + 1) as usize];

        // Store all spawns for FFA mode (T022)
        let all_spawns = spawn_points.clone();

        for spawn in spawn_points {
            team_spawns[spawn.team as usize].push(spawn);
        }

        Self {
            team_spawns,
            last_spawn,
            all_spawns,
            last_ffa_spawn: 0,
        }
    }

    /// Get next spawn point for a team (round-robin)
    pub fn get_spawn_point(&mut self, team: TeamId) -> Option<&SpawnPoint> {
        let team_idx = team.0 as usize;

        if team_idx >= self.team_spawns.len() {
            return None;
        }

        let spawns = &self.team_spawns[team_idx];
        if spawns.is_empty() {
            return None;
        }

        let spawn_idx = self.last_spawn[team_idx] % spawns.len();
        self.last_spawn[team_idx] = spawn_idx + 1;

        Some(&spawns[spawn_idx])
    }

    /// Get next spawn point as position and rotation
    pub fn get_spawn(&mut self, team: TeamId) -> Option<(Vec3, f32)> {
        self.get_spawn_point(team)
            .map(|spawn| (spawn.position_vec3(), spawn.rotation_radians()))
    }

    /// Get all spawn points for a team
    pub fn get_team_spawns(&self, team: TeamId) -> &[SpawnPoint] {
        let team_idx = team.0 as usize;
        if team_idx >= self.team_spawns.len() {
            return &[];
        }
        &self.team_spawns[team_idx]
    }

    /// Reset spawn rotation (call at round start)
    pub fn reset(&mut self) {
        for idx in &mut self.last_spawn {
            *idx = 0;
        }
        self.last_ffa_spawn = 0;
    }

    /// Get number of teams with spawns
    pub fn team_count(&self) -> usize {
        self.team_spawns.iter().filter(|s| !s.is_empty()).count()
    }

    // =========================================================================
    // FFA spawn methods (T022, T023)
    // =========================================================================

    /// Get next FFA spawn point (round-robin from all spawns, ignoring team)
    ///
    /// In FFA mode, all spawn points are treated as neutral. This method
    /// cycles through all available spawns regardless of their team field.
    pub fn get_ffa_spawn_point(&mut self) -> Option<&SpawnPoint> {
        if self.all_spawns.is_empty() {
            return None;
        }

        let spawn_idx = self.last_ffa_spawn % self.all_spawns.len();
        self.last_ffa_spawn = spawn_idx + 1;

        Some(&self.all_spawns[spawn_idx])
    }

    /// Get next FFA spawn as position and rotation (T023)
    ///
    /// Returns the spawn position and rotation for FFA mode, cycling through
    /// all available spawns regardless of team assignment.
    pub fn get_ffa_spawn(&mut self) -> Option<(Vec3, f32)> {
        self.get_ffa_spawn_point()
            .map(|spawn| (spawn.position_vec3(), spawn.rotation_radians()))
    }

    /// Get all spawn points (for FFA mode)
    pub fn get_all_spawns(&self) -> &[SpawnPoint] {
        &self.all_spawns
    }

    /// Get total number of spawn points
    pub fn spawn_count(&self) -> usize {
        self.all_spawns.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_spawns() -> Vec<SpawnPoint> {
        vec![
            SpawnPoint {
                team: 0,
                position: [4.0, 1.0, 4.0],
                rotation: 0.0,
            },
            SpawnPoint {
                team: 0,
                position: [4.0, 1.0, 8.0],
                rotation: 0.0,
            },
            SpawnPoint {
                team: 1,
                position: [12.0, 1.0, 12.0],
                rotation: 180.0,
            },
        ]
    }

    #[test]
    fn test_round_robin_spawn() {
        let mut manager = SpawnManager::new(make_test_spawns());

        // Team 0 has 2 spawns, should rotate
        let (pos1, _) = manager.get_spawn(TeamId::TEAM_0).unwrap();
        let (pos2, _) = manager.get_spawn(TeamId::TEAM_0).unwrap();
        let (pos3, _) = manager.get_spawn(TeamId::TEAM_0).unwrap();

        assert_eq!(pos1, Vec3::new(4.0, 1.0, 4.0));
        assert_eq!(pos2, Vec3::new(4.0, 1.0, 8.0));
        assert_eq!(pos3, Vec3::new(4.0, 1.0, 4.0)); // Wraps around
    }

    #[test]
    fn test_reset() {
        let mut manager = SpawnManager::new(make_test_spawns());

        manager.get_spawn(TeamId::TEAM_0);
        manager.reset();

        let (pos, _) = manager.get_spawn(TeamId::TEAM_0).unwrap();
        assert_eq!(pos, Vec3::new(4.0, 1.0, 4.0)); // Back to first spawn
    }

    #[test]
    fn test_team_count() {
        let manager = SpawnManager::new(make_test_spawns());
        assert_eq!(manager.team_count(), 2);
    }

    // =========================================================================
    // FFA spawn tests (T027)
    // =========================================================================

    #[test]
    fn test_ffa_spawn_returns_valid_spawn() {
        // T027: FFA spawn selection returns valid spawn
        let mut manager = SpawnManager::new(make_test_spawns());

        let spawn = manager.get_ffa_spawn();
        assert!(spawn.is_some());

        let (pos, _rot) = spawn.unwrap();
        // Should be one of the three spawns
        assert!(
            pos == Vec3::new(4.0, 1.0, 4.0)
                || pos == Vec3::new(4.0, 1.0, 8.0)
                || pos == Vec3::new(12.0, 1.0, 12.0)
        );
    }

    #[test]
    fn test_ffa_spawn_cycles_all_spawns() {
        // FFA spawn should cycle through ALL spawns regardless of team
        let mut manager = SpawnManager::new(make_test_spawns());

        // Get 3 spawns - should cycle through all
        let (pos1, _) = manager.get_ffa_spawn().unwrap();
        let (pos2, _) = manager.get_ffa_spawn().unwrap();
        let (pos3, _) = manager.get_ffa_spawn().unwrap();

        // Should get all three unique positions (in order)
        assert_eq!(pos1, Vec3::new(4.0, 1.0, 4.0)); // First spawn (team 0)
        assert_eq!(pos2, Vec3::new(4.0, 1.0, 8.0)); // Second spawn (team 0)
        assert_eq!(pos3, Vec3::new(12.0, 1.0, 12.0)); // Third spawn (team 1)

        // Fourth should wrap around
        let (pos4, _) = manager.get_ffa_spawn().unwrap();
        assert_eq!(pos4, Vec3::new(4.0, 1.0, 4.0));
    }

    #[test]
    fn test_ffa_spawn_reset() {
        // FFA spawn index should reset with regular reset
        let mut manager = SpawnManager::new(make_test_spawns());

        // Get a spawn
        manager.get_ffa_spawn();
        manager.get_ffa_spawn();

        // Reset
        manager.reset();

        // Should be back to first spawn
        let (pos, _) = manager.get_ffa_spawn().unwrap();
        assert_eq!(pos, Vec3::new(4.0, 1.0, 4.0));
    }

    #[test]
    fn test_ffa_spawn_count() {
        let manager = SpawnManager::new(make_test_spawns());
        assert_eq!(manager.spawn_count(), 3);
    }

    #[test]
    fn test_ffa_spawn_empty() {
        // Empty spawn list should return None
        let mut manager = SpawnManager::new(vec![]);
        assert!(manager.get_ffa_spawn().is_none());
    }

    #[test]
    fn test_get_all_spawns() {
        let spawns = make_test_spawns();
        let manager = SpawnManager::new(spawns.clone());

        assert_eq!(manager.get_all_spawns().len(), 3);
    }
}
