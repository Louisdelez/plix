//! Tests for inventory validation (User Story 4)
//!
//! Verifies:
//! - Invalid slot index is rejected
//! - Using item not in hotbar is rejected
//! - Pickup of non-existent loot is rejected
//! - Race condition: first pickup wins

use glam::Vec3;
use plix_common::inventory::{Hotbar, ItemStack};
use plix_common::time::Tick;
use plix_common::types::ItemId;
use plix_server::inventory::{
    try_pickup, use_active_item, InventoryConfig, PickupResult, UseResult,
};
use plix_server::loot::LootManager;

// T057: Test invalid slot index rejected
#[test]
fn test_invalid_slot_index_rejected() {
    let mut hotbar = Hotbar::new(9); // 9 slots: 0-8 valid

    // Try to set active slot beyond bounds
    let success = hotbar.set_active_slot(9); // Out of bounds
    assert!(!success, "Setting slot 9 should fail for hotbar size 9");

    let success = hotbar.set_active_slot(255);
    assert!(!success, "Setting slot 255 should fail");

    // Valid slot should succeed
    let success = hotbar.set_active_slot(8);
    assert!(success, "Setting slot 8 should succeed for hotbar size 9");
}

// T058: Test use item not in hotbar rejected
#[test]
fn test_use_item_not_in_hotbar_rejected() {
    let mut hotbar = Hotbar::new(9);
    // All slots empty
    hotbar.set_active_slot(0);

    let result = use_active_item(&hotbar);
    assert_eq!(
        result,
        UseResult::NoItem,
        "Using empty slot should return NoItem"
    );

    // Put item in slot 0, but select slot 1 (empty)
    hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
    hotbar.set_active_slot(1);

    let result = use_active_item(&hotbar);
    assert_eq!(
        result,
        UseResult::NoItem,
        "Using empty slot 1 should return NoItem"
    );
}

// T059: Test pickup non-existent loot rejected
#[test]
fn test_pickup_nonexistent_loot_rejected() {
    let mut hotbar = Hotbar::new(9);
    let mut loot_manager = LootManager::new();
    let config = InventoryConfig::default();

    // No loot spawned
    let player_pos = Vec3::new(10.0, 1.0, 10.0);
    let result = try_pickup(player_pos, &mut hotbar, &mut loot_manager, &config);
    assert_eq!(result, PickupResult::NoLootInRange);
}

// T060: Test race condition - first pickup wins
#[test]
fn test_first_pickup_wins() {
    let mut hotbar1 = Hotbar::new(9);
    let mut hotbar2 = Hotbar::new(9);
    let mut loot_manager = LootManager::new();
    let config = InventoryConfig::default();

    // Spawn single loot
    let loot_id = loot_manager.spawn(
        Vec3::new(10.0, 1.0, 10.0),
        ItemStack::single(ItemId::SWORD),
        Tick(0),
    );

    // Both players at same position
    let player_pos = Vec3::new(10.0, 1.0, 10.5);

    // First player picks up - should succeed
    let result1 = try_pickup(player_pos, &mut hotbar1, &mut loot_manager, &config);
    assert_eq!(result1, PickupResult::Success { loot_id });
    assert_eq!(hotbar1.get(0).map(|s| s.item_id), Some(ItemId::SWORD));

    // Second player tries to pick up same loot - should fail
    let result2 = try_pickup(player_pos, &mut hotbar2, &mut loot_manager, &config);
    assert_eq!(
        result2,
        PickupResult::NoLootInRange,
        "Second pickup should fail - loot already taken"
    );
    assert!(
        hotbar2.get(0).is_none(),
        "Second player should not have item"
    );
}

// Additional validation test: pickup out of range
#[test]
fn test_pickup_out_of_range_rejected() {
    let mut hotbar = Hotbar::new(9);
    let mut loot_manager = LootManager::new();
    let config = InventoryConfig::default();

    // Spawn loot at (10, 1, 10)
    loot_manager.spawn(
        Vec3::new(10.0, 1.0, 10.0),
        ItemStack::single(ItemId::SWORD),
        Tick(0),
    );

    // Player at (20, 1, 20) - way out of range
    let player_pos = Vec3::new(20.0, 1.0, 20.0);
    let result = try_pickup(player_pos, &mut hotbar, &mut loot_manager, &config);
    assert_eq!(result, PickupResult::NoLootInRange);
}

// Test hotbar full rejection
#[test]
fn test_pickup_hotbar_full_rejected() {
    let mut hotbar = Hotbar::new(1); // Only 1 slot
    let mut loot_manager = LootManager::new();
    let config = InventoryConfig::default();

    // Fill the hotbar
    hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));

    // Spawn different loot (won't stack with existing)
    let loot_id = loot_manager.spawn(
        Vec3::new(10.0, 1.0, 10.0),
        ItemStack::single(ItemId::HEALTH_PACK),
        Tick(0),
    );

    // Player within range
    let player_pos = Vec3::new(10.0, 1.0, 10.5);
    let result = try_pickup(player_pos, &mut hotbar, &mut loot_manager, &config);
    assert_eq!(result, PickupResult::HotbarFull { loot_id });

    // Loot should still exist
    assert!(loot_manager.get(loot_id).is_some());
}

// Test that unknown item type returns CannotUse
#[test]
fn test_unknown_item_cannot_use() {
    let mut hotbar = Hotbar::new(9);
    // Set an item with ID that doesn't exist in registry
    hotbar.set(0, Some(ItemStack::single(ItemId(999))));
    hotbar.set_active_slot(0);

    let result = use_active_item(&hotbar);
    assert_eq!(result, UseResult::CannotUse);
}
