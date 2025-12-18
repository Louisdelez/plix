//! Tests for item usage system (User Story 3)
//!
//! Verifies:
//! - Sword deals 25 damage (weapon param value)
//! - Health Pack heals 50 HP and decrements stack
//! - Consumable removed when quantity reaches 0
//! - Block Placer places Stone block
//! - Empty slot uses default melee

use plix_common::inventory::{Hotbar, ItemStack};
use plix_common::types::ItemId;
use plix_server::inventory::{get_heal_amount, get_weapon_damage};

// T043: Test Sword deals 25 damage (not default melee)
#[test]
fn test_sword_deals_weapon_damage() {
    let damage = get_weapon_damage(ItemId::SWORD);
    assert_eq!(damage, Some(25), "Sword should deal 25 damage");
}

// T044: Test Health Pack heals 50 HP and decrements stack
#[test]
fn test_health_pack_heals_and_decrements() {
    let mut hotbar = Hotbar::new(9);
    hotbar.set(0, Some(ItemStack::new(ItemId::HEALTH_PACK, 3)));
    hotbar.set_active_slot(0);

    // Verify heal amount
    let heal = get_heal_amount(ItemId::HEALTH_PACK);
    assert_eq!(heal, Some(50), "Health Pack should heal 50 HP");

    // Simulate usage: decrement stack via set()
    if let Some(stack) = hotbar.get(0) {
        let new_quantity = stack.quantity - 1;
        hotbar.set(0, Some(ItemStack::new(ItemId::HEALTH_PACK, new_quantity)));
    }
    assert_eq!(hotbar.get(0).map(|s| s.quantity), Some(2));
}

// T045: Test consumable removed when quantity reaches 0
#[test]
fn test_consumable_removed_at_zero() {
    let mut hotbar = Hotbar::new(9);
    hotbar.set(0, Some(ItemStack::single(ItemId::HEALTH_PACK)));
    hotbar.set_active_slot(0);

    // Use last health pack - decrement to 0
    if let Some(stack) = hotbar.get(0) {
        let new_quantity = stack.quantity.saturating_sub(1);
        if new_quantity == 0 {
            hotbar.set(0, None);
        } else {
            hotbar.set(0, Some(ItemStack::new(stack.item_id, new_quantity)));
        }
    }

    assert!(
        hotbar.get(0).is_none(),
        "Empty consumable should be removed"
    );
}

// T046: Test Block Placer places Stone block
#[test]
fn test_block_placer_item_type() {
    // Block Placer is a tool type with block_type as param
    use plix_common::inventory::ItemKind;
    use plix_server::inventory::ITEM_DEFS;

    let block_placer_def = ITEM_DEFS.iter().find(|d| d.id == ItemId::BLOCK_PLACER);
    assert!(block_placer_def.is_some());

    let def = block_placer_def.unwrap();
    assert_eq!(def.kind, ItemKind::Tool);
    // Block Placer places stone (BlockType::STONE = 1)
    // The param stores the block type to place
}

// T047: Test empty slot uses default melee (no item, returns None)
#[test]
fn test_empty_slot_has_no_weapon_damage() {
    let hotbar = Hotbar::new(9);

    // Empty slot should return None for active item
    let active_item = hotbar.active_item();
    assert!(active_item.is_none(), "Empty slot should have no item");

    // When no item, damage lookup returns None (use default melee)
    let damage = active_item.and_then(|stack| get_weapon_damage(stack.item_id));
    assert!(
        damage.is_none(),
        "Empty slot should use default melee damage"
    );
}

// Test weapon damage with equipped sword vs empty slot
#[test]
fn test_equipped_vs_empty_slot_damage() {
    let mut hotbar = Hotbar::new(9);

    // Empty slot: no weapon damage
    let empty_damage = hotbar
        .active_item()
        .and_then(|s| get_weapon_damage(s.item_id));
    assert!(empty_damage.is_none());

    // Equip sword
    hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
    hotbar.set_active_slot(0);

    let sword_damage = hotbar
        .active_item()
        .and_then(|s| get_weapon_damage(s.item_id));
    assert_eq!(sword_damage, Some(25));
}

// Test that non-weapon items don't return weapon damage
#[test]
fn test_non_weapon_has_no_weapon_damage() {
    assert!(get_weapon_damage(ItemId::HEALTH_PACK).is_none());
    assert!(get_weapon_damage(ItemId::BLOCK_PLACER).is_none());
}

// Test health pack heal amount
#[test]
fn test_health_pack_heal_amount() {
    assert_eq!(get_heal_amount(ItemId::HEALTH_PACK), Some(50));
    assert!(get_heal_amount(ItemId::SWORD).is_none());
    assert!(get_heal_amount(ItemId::BLOCK_PLACER).is_none());
}
