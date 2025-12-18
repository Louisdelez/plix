//! Loot entity management for the server.
//!
//! This module handles loot entities in the world including:
//! - Loot entity storage and lookup
//! - Loot spawning on player death
//! - Loot removal on pickup

pub mod entity;
pub mod spawner;

pub use entity::LootEntity;
pub use spawner::{drop_player_items, mode_drops_items};

use std::collections::HashMap;

use glam::Vec3;
use plix_common::inventory::ItemStack;
use plix_common::time::Tick;
use plix_common::types::LootEntityId;

/// Manages all loot entities in the world.
#[derive(Debug, Default)]
pub struct LootManager {
    /// All active loot entities
    entities: HashMap<LootEntityId, LootEntity>,
    /// Next ID to assign
    next_id: u32,
}

impl LootManager {
    /// Create a new loot manager.
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 1, // Start at 1, 0 is NONE
        }
    }

    /// Spawn a new loot entity in the world.
    pub fn spawn(&mut self, position: Vec3, item: ItemStack, tick: Tick) -> LootEntityId {
        let id = LootEntityId(self.next_id);
        self.next_id += 1;

        let entity = LootEntity::new(id, position, item, tick);
        self.entities.insert(id, entity);

        id
    }

    /// Remove a loot entity from the world.
    pub fn remove(&mut self, id: LootEntityId) -> Option<LootEntity> {
        self.entities.remove(&id)
    }

    /// Get a loot entity by ID.
    pub fn get(&self, id: LootEntityId) -> Option<&LootEntity> {
        self.entities.get(&id)
    }

    /// Find the nearest loot entity within range of a position.
    pub fn find_near(&self, position: Vec3, range: f32) -> Option<LootEntityId> {
        let range_sq = range * range;

        self.entities
            .iter()
            .filter(|(_, e)| (e.position - position).length_squared() <= range_sq)
            .min_by(|(_, a), (_, b)| {
                let dist_a = (a.position - position).length_squared();
                let dist_b = (b.position - position).length_squared();
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .map(|(id, _)| *id)
    }

    /// Get all loot entities (for iteration/replication).
    pub fn entities(&self) -> &HashMap<LootEntityId, LootEntity> {
        &self.entities
    }

    /// Clear all loot entities.
    pub fn clear(&mut self) {
        self.entities.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::types::ItemId;

    #[test]
    fn test_loot_manager_spawn() {
        let mut manager = LootManager::new();
        let item = ItemStack::single(ItemId::SWORD);
        let pos = Vec3::new(10.0, 1.0, 10.0);

        let id = manager.spawn(pos, item, Tick(0));

        assert!(id.is_valid());
        assert!(manager.get(id).is_some());
    }

    #[test]
    fn test_loot_manager_remove() {
        let mut manager = LootManager::new();
        let item = ItemStack::single(ItemId::SWORD);
        let pos = Vec3::new(10.0, 1.0, 10.0);

        let id = manager.spawn(pos, item, Tick(0));
        let removed = manager.remove(id);

        assert!(removed.is_some());
        assert!(manager.get(id).is_none());
    }

    #[test]
    fn test_loot_manager_find_near() {
        let mut manager = LootManager::new();
        let item = ItemStack::single(ItemId::SWORD);

        // Spawn loot at (10, 1, 10)
        let id1 = manager.spawn(Vec3::new(10.0, 1.0, 10.0), item, Tick(0));
        // Spawn loot at (20, 1, 20) - far away
        let _id2 = manager.spawn(Vec3::new(20.0, 1.0, 20.0), item, Tick(0));

        // Player at (10, 1, 11) - 1 block away from first loot
        let found = manager.find_near(Vec3::new(10.0, 1.0, 11.0), 1.5);
        assert_eq!(found, Some(id1));

        // Player at (15, 1, 15) - too far from both
        let found = manager.find_near(Vec3::new(15.0, 1.0, 15.0), 1.5);
        assert_eq!(found, None);
    }
}
