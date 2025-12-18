//! Loot entity representation for world items.

use glam::Vec3;
use plix_common::inventory::ItemStack;
use plix_common::time::Tick;
use plix_common::types::LootEntityId;

/// A loot entity in the world representing a dropped item.
#[derive(Debug, Clone)]
pub struct LootEntity {
    /// Unique identifier for this loot entity
    pub id: LootEntityId,
    /// World position of the loot
    pub position: Vec3,
    /// The item this loot contains
    pub item: ItemStack,
    /// Tick when this loot was spawned
    pub spawn_tick: Tick,
}

impl LootEntity {
    /// Create a new loot entity.
    pub fn new(id: LootEntityId, position: Vec3, item: ItemStack, spawn_tick: Tick) -> Self {
        Self {
            id,
            position,
            item,
            spawn_tick,
        }
    }

    /// Check if this loot is within range of a position.
    pub fn is_within_range(&self, pos: Vec3, range: f32) -> bool {
        (self.position - pos).length_squared() <= range * range
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::types::ItemId;

    #[test]
    fn test_loot_entity_new() {
        let id = LootEntityId(1);
        let pos = Vec3::new(10.0, 1.0, 20.0);
        let item = ItemStack::single(ItemId::SWORD);
        let tick = Tick(100);

        let entity = LootEntity::new(id, pos, item, tick);

        assert_eq!(entity.id, id);
        assert_eq!(entity.position, pos);
        assert_eq!(entity.item, item);
        assert_eq!(entity.spawn_tick, tick);
    }

    #[test]
    fn test_loot_entity_is_within_range() {
        let entity = LootEntity::new(
            LootEntityId(1),
            Vec3::new(10.0, 1.0, 10.0),
            ItemStack::single(ItemId::SWORD),
            Tick(0),
        );

        // Within range (1.0 block away)
        assert!(entity.is_within_range(Vec3::new(11.0, 1.0, 10.0), 1.5));

        // Exactly at range limit
        assert!(entity.is_within_range(Vec3::new(11.5, 1.0, 10.0), 1.5));

        // Outside range
        assert!(!entity.is_within_range(Vec3::new(12.0, 1.0, 10.0), 1.5));
    }
}
