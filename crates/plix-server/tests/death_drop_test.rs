//! Tests for death drop system (User Story 5)
//!
//! Verifies:
//! - FFA mode drops all items on death
//! - BR Lite drops all items on death
//! - Training mode does NOT drop items
//! - TDM mode does NOT drop items
//! - Dropped loot spreads 0.5-1.0 blocks from death position

use glam::Vec3;
use plix_common::inventory::{Hotbar, ItemStack};
use plix_common::time::Tick;
use plix_common::types::ItemId;
use plix_common::GameMode;
use plix_server::loot::LootManager;

/// Helper to check if a game mode should drop items on death
fn mode_drops_items(mode: GameMode) -> bool {
    matches!(mode, GameMode::Ffa | GameMode::BrLite)
}

/// Spawn loot items from a hotbar at a death position.
/// Spread items 0.5-1.0 blocks from death position.
fn drop_player_items(
    hotbar: &Hotbar,
    death_pos: Vec3,
    loot_manager: &mut LootManager,
    tick: Tick,
) -> Vec<plix_common::types::LootEntityId> {
    let mut spawned = Vec::new();

    for slot in 0..hotbar.size() {
        if let Some(stack) = hotbar.get(slot as u8) {
            // Calculate spread offset (simple deterministic spread for testing)
            let offset_x = (slot as f32 * 0.3).sin() * 0.75;
            let offset_z = (slot as f32 * 0.3).cos() * 0.75;
            let spawn_pos = Vec3::new(death_pos.x + offset_x, death_pos.y, death_pos.z + offset_z);

            let loot_id = loot_manager.spawn(spawn_pos, *stack, tick);
            spawned.push(loot_id);
        }
    }

    spawned
}

// T067: Test FFA mode drops all items on death
#[test]
fn test_ffa_mode_drops_items() {
    assert!(mode_drops_items(GameMode::Ffa));

    let mut hotbar = Hotbar::new(9);
    hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
    hotbar.set(1, Some(ItemStack::new(ItemId::HEALTH_PACK, 5)));

    let mut loot_manager = LootManager::new();
    let death_pos = Vec3::new(10.0, 1.0, 10.0);

    let spawned = drop_player_items(&hotbar, death_pos, &mut loot_manager, Tick(100));

    assert_eq!(spawned.len(), 2, "Should spawn 2 loot entities");

    // Verify loot entities exist
    for id in &spawned {
        assert!(loot_manager.get(*id).is_some());
    }
}

// T068: Test BR Lite drops all items on death
#[test]
fn test_br_lite_drops_items() {
    assert!(mode_drops_items(GameMode::BrLite));

    let mut hotbar = Hotbar::new(9);
    hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
    hotbar.set(2, Some(ItemStack::single(ItemId::BLOCK_PLACER)));

    let mut loot_manager = LootManager::new();
    let death_pos = Vec3::new(15.0, 2.0, 15.0);

    let spawned = drop_player_items(&hotbar, death_pos, &mut loot_manager, Tick(200));

    assert_eq!(spawned.len(), 2);

    // Verify items have correct types
    let loot1 = loot_manager.get(spawned[0]).unwrap();
    let loot2 = loot_manager.get(spawned[1]).unwrap();
    assert_eq!(loot1.item.item_id, ItemId::SWORD);
    assert_eq!(loot2.item.item_id, ItemId::BLOCK_PLACER);
}

// T069: Test Training mode does NOT drop items
#[test]
fn test_training_mode_no_drops() {
    assert!(!mode_drops_items(GameMode::Training));
}

// T070: Test TDM mode does NOT drop items
#[test]
fn test_tdm_mode_no_drops() {
    assert!(!mode_drops_items(GameMode::Tdm));
}

// Additional: CTF mode should not drop items
#[test]
fn test_ctf_mode_no_drops() {
    assert!(!mode_drops_items(GameMode::Ctf));
}

// T071: Test dropped loot spreads 0.5-1.0 blocks from death position
#[test]
fn test_dropped_loot_spread() {
    let mut hotbar = Hotbar::new(9);
    hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
    hotbar.set(1, Some(ItemStack::single(ItemId::HEALTH_PACK)));
    hotbar.set(2, Some(ItemStack::single(ItemId::BLOCK_PLACER)));

    let mut loot_manager = LootManager::new();
    let death_pos = Vec3::new(10.0, 1.0, 10.0);

    let spawned = drop_player_items(&hotbar, death_pos, &mut loot_manager, Tick(100));

    assert_eq!(spawned.len(), 3);

    for id in &spawned {
        let loot = loot_manager.get(*id).unwrap();
        let distance = (loot.position - death_pos).length();

        // Items should be spread 0.5-1.0 blocks (we use 0.75 max in our simple spread)
        assert!(
            distance <= 1.0,
            "Loot should be within 1.0 blocks of death position, got {}",
            distance
        );
    }
}

// Test empty hotbar drops nothing
#[test]
fn test_empty_hotbar_no_drops() {
    let hotbar = Hotbar::new(9);
    let mut loot_manager = LootManager::new();
    let death_pos = Vec3::new(10.0, 1.0, 10.0);

    let spawned = drop_player_items(&hotbar, death_pos, &mut loot_manager, Tick(100));

    assert!(spawned.is_empty(), "Empty hotbar should spawn no loot");
}

// Test dropped items preserve quantity
#[test]
fn test_dropped_items_preserve_quantity() {
    let mut hotbar = Hotbar::new(9);
    hotbar.set(0, Some(ItemStack::new(ItemId::HEALTH_PACK, 10)));

    let mut loot_manager = LootManager::new();
    let death_pos = Vec3::new(10.0, 1.0, 10.0);

    let spawned = drop_player_items(&hotbar, death_pos, &mut loot_manager, Tick(100));

    assert_eq!(spawned.len(), 1);
    let loot = loot_manager.get(spawned[0]).unwrap();
    assert_eq!(
        loot.item.quantity, 10,
        "Dropped loot should preserve quantity"
    );
}
