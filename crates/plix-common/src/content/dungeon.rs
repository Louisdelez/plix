//! Dungeon definitions with rooms, bosses, and rewards

use serde::{Deserialize, Serialize};

use super::ids::{DungeonId, LootTableId, MobDefId};
use super::spawn::SpawnPointDefinition;
use crate::math::{Vec3, AABB};

/// A dungeon definition with rooms, boss, and rewards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonDefinition {
    /// Unique stable identifier
    pub dungeon_id: DungeonId,

    /// Display name shown to players
    pub display_name: String,

    /// Suggested level/difficulty (1-10 scale)
    pub difficulty: u32,

    /// Entry portal location in world coordinates
    pub entry_location: Vec3,

    /// Room definitions (ordered progression)
    pub rooms: Vec<RoomDefinition>,

    /// Boss mob ID
    pub boss_mob_id: MobDefId,

    /// Boss spawn location
    pub boss_spawn_position: Vec3,

    /// Reward chest loot table
    pub reward_loot_table: LootTableId,

    /// Boss respawn timer in seconds (shared world model)
    pub boss_respawn_secs: f32,
}

impl DungeonDefinition {
    /// Get the total number of rooms
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Find which room contains a position (if any)
    pub fn room_at_position(&self, pos: &Vec3) -> Option<&RoomDefinition> {
        self.rooms.iter().find(|room| room.bounds.contains(pos))
    }

    /// Check if a position is inside the dungeon bounds
    pub fn contains_position(&self, pos: &Vec3) -> bool {
        self.rooms.iter().any(|room| room.bounds.contains(pos))
    }

    /// Get all spawn points across all rooms
    pub fn all_spawns(&self) -> Vec<&SpawnPointDefinition> {
        self.rooms.iter().flat_map(|room| &room.spawns).collect()
    }

    /// Validate the dungeon definition
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.rooms.is_empty() {
            return Err("Dungeon must have at least one room");
        }
        if self.difficulty < 1 || self.difficulty > 10 {
            return Err("Difficulty must be between 1 and 10");
        }
        if self.boss_respawn_secs <= 0.0 {
            return Err("Boss respawn time must be > 0");
        }
        Ok(())
    }
}

/// A room within a dungeon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomDefinition {
    /// Room identifier within dungeon
    pub room_id: String,

    /// Display name
    pub name: String,

    /// Bounds of this room (for tracking player location)
    pub bounds: AABB,

    /// Spawn points within this room
    #[serde(default)]
    pub spawns: Vec<SpawnPointDefinition>,
}

impl RoomDefinition {
    /// Check if a position is inside this room
    pub fn contains(&self, pos: &Vec3) -> bool {
        self.bounds.contains(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_dungeon() -> DungeonDefinition {
        DungeonDefinition {
            dungeon_id: DungeonId::new("crypt_of_the_gate"),
            display_name: "Crypt of the Gate".to_string(),
            difficulty: 5,
            entry_location: Vec3::new(300.0, 5.0, 200.0),
            rooms: vec![
                RoomDefinition {
                    room_id: "entrance".to_string(),
                    name: "Entrance Hall".to_string(),
                    bounds: AABB::new(
                        Vec3::new(300.0, 0.0, 200.0),
                        Vec3::new(320.0, 10.0, 220.0),
                    ),
                    spawns: vec![],
                },
                RoomDefinition {
                    room_id: "boss_room".to_string(),
                    name: "The Gate Chamber".to_string(),
                    bounds: AABB::new(
                        Vec3::new(350.0, 0.0, 250.0),
                        Vec3::new(400.0, 20.0, 300.0),
                    ),
                    spawns: vec![],
                },
            ],
            boss_mob_id: MobDefId::new("gate_warden"),
            boss_spawn_position: Vec3::new(375.0, 1.0, 275.0),
            reward_loot_table: LootTableId::new("gate_warden_drops"),
            boss_respawn_secs: 900.0, // 15 minutes
        }
    }

    #[test]
    fn test_room_count() {
        let dungeon = sample_dungeon();
        assert_eq!(dungeon.room_count(), 2);
    }

    #[test]
    fn test_room_at_position() {
        let dungeon = sample_dungeon();

        // Inside entrance hall
        let entrance_pos = Vec3::new(310.0, 5.0, 210.0);
        let room = dungeon.room_at_position(&entrance_pos);
        assert!(room.is_some());
        assert_eq!(room.unwrap().room_id, "entrance");

        // Inside boss room
        let boss_pos = Vec3::new(375.0, 10.0, 275.0);
        let room = dungeon.room_at_position(&boss_pos);
        assert!(room.is_some());
        assert_eq!(room.unwrap().room_id, "boss_room");

        // Outside dungeon
        let outside_pos = Vec3::new(0.0, 0.0, 0.0);
        assert!(dungeon.room_at_position(&outside_pos).is_none());
    }

    #[test]
    fn test_contains_position() {
        let dungeon = sample_dungeon();
        assert!(dungeon.contains_position(&Vec3::new(310.0, 5.0, 210.0)));
        assert!(!dungeon.contains_position(&Vec3::new(0.0, 0.0, 0.0)));
    }

    #[test]
    fn test_validation() {
        let valid = sample_dungeon();
        assert!(valid.validate().is_ok());

        let empty_rooms = DungeonDefinition {
            rooms: vec![],
            ..sample_dungeon()
        };
        assert!(empty_rooms.validate().is_err());

        let bad_difficulty = DungeonDefinition {
            difficulty: 15,
            ..sample_dungeon()
        };
        assert!(bad_difficulty.validate().is_err());
    }

    #[test]
    fn test_toml_deserialization() {
        let toml_str = r#"
            dungeon_id = "crypt_of_the_gate"
            display_name = "Crypt of the Gate"
            difficulty = 5
            entry_location = [300.0, 5.0, 200.0]
            boss_mob_id = "gate_warden"
            boss_spawn_position = [350.0, 0.0, 250.0]
            reward_loot_table = "gate_warden_drops"
            boss_respawn_secs = 900.0

            [[rooms]]
            room_id = "entrance"
            name = "Entrance Hall"

            [rooms.bounds]
            min = [300.0, 0.0, 200.0]
            max = [320.0, 10.0, 220.0]
        "#;

        let dungeon: DungeonDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(dungeon.dungeon_id.as_str(), "crypt_of_the_gate");
        assert_eq!(dungeon.rooms.len(), 1);
    }
}
