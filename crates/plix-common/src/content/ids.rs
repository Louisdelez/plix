//! Content identifier newtypes for type-safe references

use serde::{Deserialize, Serialize};
use std::fmt;

/// Chapter identifier (e.g., "the_broken_gate")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChapterId(pub String);

impl ChapterId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ChapterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ChapterId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ChapterId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Quest identifier (e.g., "clear_rats")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestId(pub String);

impl QuestId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for QuestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for QuestId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for QuestId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Mob definition identifier (e.g., "cave_rat")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MobDefId(pub String);

impl MobDefId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MobDefId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for MobDefId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for MobDefId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Dungeon identifier (e.g., "crypt_of_the_gate")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DungeonId(pub String);

impl DungeonId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DungeonId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for DungeonId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for DungeonId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// NPC identifier (e.g., "village_elder")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NpcId(pub String);

impl NpcId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NpcId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for NpcId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for NpcId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Spawn point identifier (e.g., "mine_entrance_rats_1")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpawnId(pub String);

impl SpawnId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SpawnId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for SpawnId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for SpawnId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Loot table identifier (e.g., "cave_rat_drops")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LootTableId(pub String);

impl LootTableId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LootTableId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for LootTableId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for LootTableId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Region identifier (e.g., "mine_entrance")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegionId(pub String);

impl RegionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RegionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for RegionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for RegionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Runtime mob instance identifier (unique per spawn)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MobInstanceId(pub u64);

impl MobInstanceId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl fmt::Display for MobInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mob#{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_id_from_str() {
        let id: QuestId = "clear_rats".into();
        assert_eq!(id.as_str(), "clear_rats");
    }

    #[test]
    fn test_mob_def_id_display() {
        let id = MobDefId::new("cave_rat");
        assert_eq!(format!("{}", id), "cave_rat");
    }

    #[test]
    fn test_ids_equality() {
        let id1 = ChapterId::new("chapter_1");
        let id2 = ChapterId::new("chapter_1");
        let id3 = ChapterId::new("chapter_2");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
}
