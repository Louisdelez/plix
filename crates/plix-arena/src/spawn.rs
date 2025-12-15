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
}

impl SpawnManager {
    /// Create a new spawn manager from spawn points
    pub fn new(spawn_points: Vec<SpawnPoint>) -> Self {
        // Find max team ID
        let max_team = spawn_points.iter().map(|sp| sp.team).max().unwrap_or(0);

        // Initialize spawn lists per team
        let mut team_spawns = vec![Vec::new(); (max_team + 1) as usize];
        let last_spawn = vec![0; (max_team + 1) as usize];

        for spawn in spawn_points {
            team_spawns[spawn.team as usize].push(spawn);
        }

        Self {
            team_spawns,
            last_spawn,
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
    }

    /// Get number of teams with spawns
    pub fn team_count(&self) -> usize {
        self.team_spawns.iter().filter(|s| !s.is_empty()).count()
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
}
