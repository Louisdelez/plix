//! Dungeon reward chest entity
//!
//! Tracks the reward chest that spawns after boss defeat:
//! - One-time loot per player per boss run
//! - Disappears on boss respawn
//! - Uses loot table from dungeon definition

use std::collections::HashSet;

use plix_common::content::ids::{DungeonId, LootTableId};
use plix_common::content::loot::LootTable;
use plix_common::math::Vec3;
use plix_common::types::{ItemId, PlayerId};

/// A reward chest entity that spawns after boss defeat.
#[derive(Debug, Clone)]
pub struct ChestEntity {
    /// Dungeon this chest belongs to
    pub dungeon_id: DungeonId,
    /// Chest position in world coordinates
    pub position: Vec3,
    /// Loot table ID for rolling rewards
    pub loot_table_id: LootTableId,
    /// Players who have already looted this chest
    looted_by: HashSet<PlayerId>,
    /// Whether the chest is currently visible/active
    pub active: bool,
}

impl ChestEntity {
    /// Create a new reward chest.
    pub fn new(dungeon_id: DungeonId, position: Vec3, loot_table_id: LootTableId) -> Self {
        Self {
            dungeon_id,
            position,
            loot_table_id,
            looted_by: HashSet::new(),
            active: false,
        }
    }

    /// Activate the chest (called when boss is killed).
    pub fn activate(&mut self) {
        self.active = true;
        self.looted_by.clear();
    }

    /// Deactivate the chest (called when boss respawns).
    pub fn deactivate(&mut self) {
        self.active = false;
        self.looted_by.clear();
    }

    /// Check if a player can loot this chest.
    pub fn can_loot(&self, player_id: PlayerId) -> bool {
        self.active && !self.looted_by.contains(&player_id)
    }

    /// Attempt to loot the chest.
    /// Returns the loot drops if successful, None if already looted or inactive.
    pub fn try_loot(
        &mut self,
        player_id: PlayerId,
        loot_table: &LootTable,
    ) -> Option<Vec<(ItemId, u32)>> {
        if !self.can_loot(player_id) {
            return None;
        }

        self.looted_by.insert(player_id);
        Some(loot_table.roll(None))
    }

    /// Check if a player has already looted this chest.
    pub fn has_looted(&self, player_id: PlayerId) -> bool {
        self.looted_by.contains(&player_id)
    }

    /// Get the number of players who have looted the chest.
    pub fn loot_count(&self) -> usize {
        self.looted_by.len()
    }

    /// Check if the player is within interaction range of the chest.
    pub fn is_in_range(&self, player_pos: &Vec3, range: f32) -> bool {
        self.position.distance(*player_pos) <= range
    }
}

/// Chest interaction result.
#[derive(Debug, Clone, PartialEq)]
pub enum ChestInteractResult {
    /// Successfully looted, contains items
    Success(Vec<(ItemId, u32)>),
    /// Already looted by this player
    AlreadyLooted,
    /// Chest is not active (boss not killed)
    NotActive,
    /// Player too far from chest
    OutOfRange,
    /// Loot table not found
    LootTableNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::content::loot::{LootEntry, LootTable};

    fn sample_loot_table() -> LootTable {
        LootTable {
            loot_table_id: LootTableId::new("boss_rewards"),
            guaranteed: vec![LootEntry {
                item_id: ItemId::new_raw(100), // boss_trophy
                quantity: (1, 1),
                chance: 1.0,
            }],
            entries: vec![LootEntry {
                item_id: ItemId::new_raw(101), // rare_gem
                quantity: (1, 3),
                chance: 0.5,
            }],
        }
    }

    #[test]
    fn test_chest_creation() {
        let chest = ChestEntity::new(
            DungeonId::new("test_dungeon"),
            Vec3::new(100.0, 0.0, 100.0),
            LootTableId::new("boss_rewards"),
        );

        assert!(!chest.active);
        assert_eq!(chest.loot_count(), 0);
    }

    #[test]
    fn test_chest_activation() {
        let mut chest = ChestEntity::new(
            DungeonId::new("test_dungeon"),
            Vec3::new(100.0, 0.0, 100.0),
            LootTableId::new("boss_rewards"),
        );

        assert!(!chest.active);
        assert!(!chest.can_loot(PlayerId(1)));

        chest.activate();

        assert!(chest.active);
        assert!(chest.can_loot(PlayerId(1)));
    }

    #[test]
    fn test_chest_loot_once_per_player() {
        let mut chest = ChestEntity::new(
            DungeonId::new("test_dungeon"),
            Vec3::new(100.0, 0.0, 100.0),
            LootTableId::new("boss_rewards"),
        );
        let loot_table = sample_loot_table();

        chest.activate();

        // First loot should succeed
        let loot1 = chest.try_loot(PlayerId(1), &loot_table);
        assert!(loot1.is_some());
        assert!(chest.has_looted(PlayerId(1)));

        // Second loot should fail
        let loot2 = chest.try_loot(PlayerId(1), &loot_table);
        assert!(loot2.is_none());

        // Different player should succeed
        let loot3 = chest.try_loot(PlayerId(2), &loot_table);
        assert!(loot3.is_some());
        assert_eq!(chest.loot_count(), 2);
    }

    #[test]
    fn test_chest_deactivation_clears_loot_tracking() {
        let mut chest = ChestEntity::new(
            DungeonId::new("test_dungeon"),
            Vec3::new(100.0, 0.0, 100.0),
            LootTableId::new("boss_rewards"),
        );
        let loot_table = sample_loot_table();

        chest.activate();
        chest.try_loot(PlayerId(1), &loot_table);
        assert!(chest.has_looted(PlayerId(1)));

        // Deactivate (boss respawn)
        chest.deactivate();
        assert!(!chest.active);
        assert!(!chest.has_looted(PlayerId(1)));

        // Reactivate (new boss kill)
        chest.activate();
        assert!(chest.can_loot(PlayerId(1)));
    }

    #[test]
    fn test_chest_range_check() {
        let chest = ChestEntity::new(
            DungeonId::new("test_dungeon"),
            Vec3::new(100.0, 0.0, 100.0),
            LootTableId::new("boss_rewards"),
        );

        // Player at chest position
        assert!(chest.is_in_range(&Vec3::new(100.0, 0.0, 100.0), 5.0));

        // Player within range
        assert!(chest.is_in_range(&Vec3::new(103.0, 0.0, 100.0), 5.0));

        // Player out of range
        assert!(!chest.is_in_range(&Vec3::new(110.0, 0.0, 100.0), 5.0));
    }

    #[test]
    fn test_guaranteed_loot_drops() {
        let mut chest = ChestEntity::new(
            DungeonId::new("test_dungeon"),
            Vec3::new(100.0, 0.0, 100.0),
            LootTableId::new("boss_rewards"),
        );
        let loot_table = sample_loot_table();

        chest.activate();

        let loot = chest.try_loot(PlayerId(1), &loot_table).unwrap();

        // Should always contain the guaranteed item
        assert!(loot.iter().any(|(id, _)| id.raw() == 100));
    }
}
