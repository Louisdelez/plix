//! Item pickup system for automatic loot collection.

use glam::Vec3;
use plix_common::inventory::Hotbar;
use plix_common::types::LootEntityId;

use super::item_registry::get_max_stack;
use crate::inventory::InventoryConfig;
use crate::loot::LootManager;

/// Result of a pickup attempt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickupResult {
    /// Successfully picked up all items
    Success {
        /// Loot entity that was picked up
        loot_id: LootEntityId,
    },
    /// No loot found within range
    NoLootInRange,
    /// Hotbar is full, cannot pick up
    HotbarFull {
        /// Loot entity that couldn't be picked up
        loot_id: LootEntityId,
    },
    /// Partially picked up (some items couldn't fit)
    Partial {
        /// Loot entity
        loot_id: LootEntityId,
        /// Amount remaining in loot
        remaining: u8,
    },
}

/// Try to pick up the nearest loot within range.
///
/// Returns the result of the pickup attempt and modifies the hotbar
/// and loot manager accordingly.
pub fn try_pickup(
    player_pos: Vec3,
    hotbar: &mut Hotbar,
    loot_manager: &mut LootManager,
    config: &InventoryConfig,
) -> PickupResult {
    // Find nearest loot within range
    let loot_id = match loot_manager.find_near(player_pos, config.pickup_range) {
        Some(id) => id,
        None => return PickupResult::NoLootInRange,
    };

    // Get the loot item info
    let loot = match loot_manager.get(loot_id) {
        Some(loot) => loot,
        None => return PickupResult::NoLootInRange,
    };

    let item_id = loot.item.item_id;
    let quantity = loot.item.quantity;
    let max_stack = get_max_stack(item_id);

    // Try to add to hotbar
    let remaining = hotbar.try_add_item(item_id, quantity, max_stack);

    if remaining == quantity {
        // Couldn't add any items - hotbar full
        PickupResult::HotbarFull { loot_id }
    } else if remaining == 0 {
        // All items added - remove loot
        loot_manager.remove(loot_id);
        PickupResult::Success { loot_id }
    } else {
        // Partial pickup - update loot quantity
        // For simplicity in MVP, we just remove the loot entirely if any was picked up
        // A more complex system would update the loot quantity
        loot_manager.remove(loot_id);
        PickupResult::Partial { loot_id, remaining }
    }
}

/// Check if there is any loot within pickup range.
pub fn has_loot_in_range(
    player_pos: Vec3,
    loot_manager: &LootManager,
    config: &InventoryConfig,
) -> bool {
    loot_manager
        .find_near(player_pos, config.pickup_range)
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::inventory::ItemStack;
    use plix_common::time::Tick;
    use plix_common::types::ItemId;

    #[test]
    fn test_try_pickup_success() {
        let mut hotbar = Hotbar::new(9);
        let mut loot_manager = LootManager::new();
        let config = InventoryConfig::default();

        let loot_id = loot_manager.spawn(
            Vec3::new(10.0, 1.0, 10.0),
            ItemStack::single(ItemId::SWORD),
            Tick(0),
        );

        let player_pos = Vec3::new(10.0, 1.0, 10.5);
        let result = try_pickup(player_pos, &mut hotbar, &mut loot_manager, &config);

        assert_eq!(result, PickupResult::Success { loot_id });
        assert!(loot_manager.get(loot_id).is_none());
        assert_eq!(hotbar.get(0).map(|s| s.item_id), Some(ItemId::SWORD));
    }

    #[test]
    fn test_try_pickup_no_loot() {
        let mut hotbar = Hotbar::new(9);
        let mut loot_manager = LootManager::new();
        let config = InventoryConfig::default();

        let player_pos = Vec3::new(10.0, 1.0, 10.0);
        let result = try_pickup(player_pos, &mut hotbar, &mut loot_manager, &config);

        assert_eq!(result, PickupResult::NoLootInRange);
    }

    #[test]
    fn test_try_pickup_hotbar_full() {
        let mut hotbar = Hotbar::new(1);
        hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));

        let mut loot_manager = LootManager::new();
        let config = InventoryConfig::default();

        let loot_id = loot_manager.spawn(
            Vec3::new(10.0, 1.0, 10.0),
            ItemStack::single(ItemId::BLOCK_PLACER), // Different item
            Tick(0),
        );

        let player_pos = Vec3::new(10.0, 1.0, 10.5);
        let result = try_pickup(player_pos, &mut hotbar, &mut loot_manager, &config);

        assert_eq!(result, PickupResult::HotbarFull { loot_id });
        assert!(loot_manager.get(loot_id).is_some()); // Loot still exists
    }

    #[test]
    fn test_has_loot_in_range() {
        let mut loot_manager = LootManager::new();
        let config = InventoryConfig::default();

        loot_manager.spawn(
            Vec3::new(10.0, 1.0, 10.0),
            ItemStack::single(ItemId::SWORD),
            Tick(0),
        );

        // In range
        assert!(has_loot_in_range(
            Vec3::new(10.0, 1.0, 10.5),
            &loot_manager,
            &config
        ));

        // Out of range
        assert!(!has_loot_in_range(
            Vec3::new(20.0, 1.0, 20.0),
            &loot_manager,
            &config
        ));
    }
}
