//! Spawn point definitions for mobs

use serde::{Deserialize, Serialize};

use super::ids::{MobDefId, RegionId, SpawnId};
use crate::math::Vec3;

/// Definition of a mob spawn point in the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPointDefinition {
    /// Unique stable identifier
    pub spawn_id: SpawnId,

    /// Mob type to spawn
    pub mob_id: MobDefId,

    /// Maximum simultaneous spawns from this point
    pub count: u32,

    /// Respawn delay in seconds (0 = no respawn, e.g., in dungeons)
    pub respawn_secs: f32,

    /// Spawn radius around center position
    pub radius: f32,

    /// Center position in world coordinates
    pub position: Vec3,

    /// Region this spawn belongs to (for limits)
    pub region_id: RegionId,
}

impl SpawnPointDefinition {
    /// Validate the spawn point has sensible values
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.count < 1 {
            return Err("Count must be >= 1");
        }
        if self.respawn_secs < 0.0 {
            return Err("Respawn time must be >= 0");
        }
        if self.radius < 0.0 {
            return Err("Radius must be >= 0");
        }
        Ok(())
    }

    /// Check if this spawn point respawns mobs
    pub fn has_respawn(&self) -> bool {
        self.respawn_secs > 0.0
    }
}

/// A collection of spawn points, typically loaded from a single TOML file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpawnPointCollection {
    /// All spawn points in this collection
    #[serde(default)]
    pub spawns: Vec<SpawnPointDefinition>,

    /// Optional per-region mob limits
    #[serde(default)]
    pub region_limits: std::collections::HashMap<String, u32>,
}

impl SpawnPointCollection {
    /// Get all spawn points for a specific region
    pub fn spawns_in_region(&self, region_id: &RegionId) -> Vec<&SpawnPointDefinition> {
        self.spawns
            .iter()
            .filter(|s| &s.region_id == region_id)
            .collect()
    }

    /// Get the mob limit for a region (None if no limit set)
    pub fn region_limit(&self, region_id: &RegionId) -> Option<u32> {
        self.region_limits.get(region_id.as_str()).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_spawn() -> SpawnPointDefinition {
        SpawnPointDefinition {
            spawn_id: SpawnId::new("mine_rats_1"),
            mob_id: MobDefId::new("cave_rat"),
            count: 3,
            respawn_secs: 60.0,
            radius: 5.0,
            position: Vec3::new(100.0, 10.0, 50.0),
            region_id: RegionId::new("mine_entrance"),
        }
    }

    #[test]
    fn test_spawn_validation() {
        let valid = sample_spawn();
        assert!(valid.validate().is_ok());

        let zero_count = SpawnPointDefinition { count: 0, ..valid };
        assert!(zero_count.validate().is_err());
    }

    #[test]
    fn test_has_respawn() {
        let with_respawn = sample_spawn();
        assert!(with_respawn.has_respawn());

        let no_respawn = SpawnPointDefinition {
            respawn_secs: 0.0,
            ..sample_spawn()
        };
        assert!(!no_respawn.has_respawn());
    }

    #[test]
    fn test_toml_deserialization() {
        let toml_str = r#"
            [[spawns]]
            spawn_id = "mine_rats_1"
            mob_id = "cave_rat"
            count = 3
            respawn_secs = 60.0
            radius = 5.0
            position = [100.0, 10.0, 50.0]
            region_id = "mine_entrance"

            [[spawns]]
            spawn_id = "mine_rats_2"
            mob_id = "cave_rat"
            count = 2
            respawn_secs = 90.0
            radius = 3.0
            position = [110.0, 10.0, 55.0]
            region_id = "mine_entrance"

            [region_limits]
            mine_entrance = 10
            dark_forest = 15
        "#;

        let collection: SpawnPointCollection = toml::from_str(toml_str).unwrap();
        assert_eq!(collection.spawns.len(), 2);
        assert_eq!(
            collection.region_limit(&RegionId::new("mine_entrance")),
            Some(10)
        );
        assert_eq!(
            collection.region_limit(&RegionId::new("unknown")),
            None
        );
    }

    #[test]
    fn test_spawns_in_region() {
        let collection = SpawnPointCollection {
            spawns: vec![
                SpawnPointDefinition {
                    spawn_id: SpawnId::new("sp1"),
                    region_id: RegionId::new("region_a"),
                    ..sample_spawn()
                },
                SpawnPointDefinition {
                    spawn_id: SpawnId::new("sp2"),
                    region_id: RegionId::new("region_a"),
                    ..sample_spawn()
                },
                SpawnPointDefinition {
                    spawn_id: SpawnId::new("sp3"),
                    region_id: RegionId::new("region_b"),
                    ..sample_spawn()
                },
            ],
            region_limits: Default::default(),
        };

        let region_a = collection.spawns_in_region(&RegionId::new("region_a"));
        assert_eq!(region_a.len(), 2);

        let region_b = collection.spawns_in_region(&RegionId::new("region_b"));
        assert_eq!(region_b.len(), 1);
    }
}
