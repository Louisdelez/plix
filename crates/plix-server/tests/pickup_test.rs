//! Tests for item pickup system (User Story 2)
//!
//! Verifies:
//! - Pickup within 1.5 blocks adds to first empty slot
//! - Pickup fails when hotbar full (non-stackable)
//! - Consumable stacks to max 16
//! - Pickup sends appropriate events

use glam::Vec3;
use plix_common::inventory::{Hotbar, ItemStack};
use plix_common::time::Tick;
use plix_common::types::ItemId;
use plix_server::inventory::{get_max_stack, InventoryConfig};
use plix_server::loot::LootManager;

// T032: Test pickup within range adds to first empty slot
#[test]
fn test_pickup_within_range_adds_to_first_empty_slot() {
    let mut hotbar = Hotbar::new(9);
    let mut loot_manager = LootManager::new();
    let config = InventoryConfig::default();

    // Spawn loot at (10, 1, 10)
    let loot_id = loot_manager.spawn(
        Vec3::new(10.0, 1.0, 10.0),
        ItemStack::single(ItemId::SWORD),
        Tick(0),
    );

    // Player at (10, 1, 11) - 1 block away, within pickup range
    let player_pos = Vec3::new(10.0, 1.0, 11.0);

    // Find loot within range
    let found = loot_manager.find_near(player_pos, config.pickup_range);
    assert_eq!(found, Some(loot_id));

    // Pick up the loot
    if let Some(loot_id) = found {
        if let Some(loot) = loot_manager.remove(loot_id) {
            // Add to hotbar
            let max_stack = get_max_stack(loot.item.item_id);
            let remaining = hotbar.try_add_item(loot.item.item_id, loot.item.quantity, max_stack);
            assert_eq!(remaining, 0); // All items added
        }
    }

    // Verify item is in hotbar
    assert_eq!(hotbar.get(0).map(|s| s.item_id), Some(ItemId::SWORD));
    assert_eq!(hotbar.get(0).map(|s| s.quantity), Some(1));
}

// T033: Test pickup fails when hotbar full (non-stackable)
#[test]
fn test_pickup_fails_when_hotbar_full_non_stackable() {
    let mut hotbar = Hotbar::new(3); // Small hotbar for test
    let mut loot_manager = LootManager::new();

    // Fill hotbar with swords (non-stackable)
    hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
    hotbar.set(1, Some(ItemStack::single(ItemId::SWORD)));
    hotbar.set(2, Some(ItemStack::single(ItemId::SWORD)));
    assert!(hotbar.is_full());

    // Spawn another sword
    let loot_id = loot_manager.spawn(
        Vec3::new(10.0, 1.0, 10.0),
        ItemStack::single(ItemId::SWORD),
        Tick(0),
    );

    // Try to pick up
    let loot = loot_manager.get(loot_id).unwrap();
    let max_stack = get_max_stack(loot.item.item_id);
    let remaining = hotbar.try_add_item(loot.item.item_id, loot.item.quantity, max_stack);

    // Should fail - all items remain
    assert_eq!(remaining, 1);

    // Loot should still exist in manager (not removed since pickup failed)
    assert!(loot_manager.get(loot_id).is_some());
}

// T034: Test consumable stacks to max 16
#[test]
fn test_consumable_stacks_to_max_16() {
    let mut hotbar = Hotbar::new(9);

    // Add initial health pack
    let max_stack = get_max_stack(ItemId::HEALTH_PACK);
    assert_eq!(max_stack, 16);

    hotbar.set(0, Some(ItemStack::new(ItemId::HEALTH_PACK, 10)));

    // Try to add 10 more:
    // - First fills existing stack: 10 + 6 = 16 (first slot full)
    // - Then uses empty slot for overflow: 4 goes to slot 1
    // - Total: 0 remaining
    let remaining = hotbar.try_add_item(ItemId::HEALTH_PACK, 10, max_stack);
    assert_eq!(remaining, 0); // All items placed

    // First slot should be full (16)
    assert_eq!(hotbar.get(0).map(|s| s.quantity), Some(16));

    // Remaining 4 should go to next slot
    assert_eq!(hotbar.get(1).map(|s| s.item_id), Some(ItemId::HEALTH_PACK));
    assert_eq!(hotbar.get(1).map(|s| s.quantity), Some(4));
}

#[test]
fn test_consumable_fills_partial_stacks_first() {
    let mut hotbar = Hotbar::new(9);

    // Add partial stacks in different slots
    hotbar.set(0, Some(ItemStack::new(ItemId::HEALTH_PACK, 5)));
    hotbar.set(2, Some(ItemStack::new(ItemId::HEALTH_PACK, 8)));

    // Add 10 more health packs
    let max_stack = get_max_stack(ItemId::HEALTH_PACK);
    let remaining = hotbar.try_add_item(ItemId::HEALTH_PACK, 10, max_stack);
    assert_eq!(remaining, 0);

    // First slot: 5 + (16-5=11) = but we're adding 10, so 5 + 10 = 15? No wait...
    // Actually try_add_item fills existing stacks first
    // Slot 0: 5 -> can add 11 more -> add 10 -> becomes 15
    // That's one possible interpretation. Let's verify actual behavior.

    // After adding 10:
    // - Slot 0 (5): add up to 11 -> add 10 -> 15
    // - Slot 2 (8): unchanged (no overflow)
    assert_eq!(hotbar.get(0).map(|s| s.quantity), Some(15));
    assert_eq!(hotbar.get(2).map(|s| s.quantity), Some(8));
}

// T035: Test pickup distance validation
#[test]
fn test_pickup_outside_range_fails() {
    let mut loot_manager = LootManager::new();
    let config = InventoryConfig::default();

    // Spawn loot at (10, 1, 10)
    loot_manager.spawn(
        Vec3::new(10.0, 1.0, 10.0),
        ItemStack::single(ItemId::SWORD),
        Tick(0),
    );

    // Player at (10, 1, 15) - 5 blocks away, outside pickup range (1.5)
    let player_pos = Vec3::new(10.0, 1.0, 15.0);

    // Should not find loot
    let found = loot_manager.find_near(player_pos, config.pickup_range);
    assert_eq!(found, None);
}

#[test]
fn test_pickup_at_exact_range() {
    let mut loot_manager = LootManager::new();
    let config = InventoryConfig::default();

    // Spawn loot at origin
    let loot_id = loot_manager.spawn(
        Vec3::new(0.0, 0.0, 0.0),
        ItemStack::single(ItemId::SWORD),
        Tick(0),
    );

    // Player exactly at range limit (1.5 blocks away)
    let player_pos = Vec3::new(1.5, 0.0, 0.0);

    // Should find loot (range is inclusive)
    let found = loot_manager.find_near(player_pos, config.pickup_range);
    assert_eq!(found, Some(loot_id));
}

#[test]
fn test_pickup_finds_nearest() {
    let mut loot_manager = LootManager::new();

    // Spawn two loots
    let near_id = loot_manager.spawn(
        Vec3::new(10.0, 1.0, 10.0),
        ItemStack::single(ItemId::SWORD),
        Tick(0),
    );
    let _far_id = loot_manager.spawn(
        Vec3::new(10.5, 1.0, 10.0),
        ItemStack::single(ItemId::HEALTH_PACK),
        Tick(0),
    );

    // Player closer to first loot
    let player_pos = Vec3::new(10.0, 1.0, 11.0);

    // Should find nearest
    let found = loot_manager.find_near(player_pos, 2.0);
    assert_eq!(found, Some(near_id));
}

#[test]
fn test_loot_manager_clear() {
    let mut loot_manager = LootManager::new();

    loot_manager.spawn(Vec3::ZERO, ItemStack::single(ItemId::SWORD), Tick(0));
    loot_manager.spawn(Vec3::ONE, ItemStack::single(ItemId::SWORD), Tick(0));
    assert_eq!(loot_manager.entities().len(), 2);

    loot_manager.clear();
    assert_eq!(loot_manager.entities().len(), 0);
}
