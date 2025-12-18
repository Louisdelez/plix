//! Tests for game mode starting loadouts (User Story 6)
//!
//! Verifies:
//! - Training mode gives Sword + Health Packs
//! - TDM/FFA/CTF give Sword only
//! - BR Lite starts with empty hotbar

use plix_common::inventory::{Hotbar, ItemStack};
use plix_common::types::ItemId;
use plix_common::GameMode;

/// Get the starting loadout for a game mode.
/// Returns a vec of (slot, ItemStack) pairs.
fn get_starting_loadout(mode: GameMode) -> Vec<(u8, ItemStack)> {
    match mode {
        GameMode::Training => {
            // Training: Sword + Health Packs for practice
            vec![
                (0, ItemStack::single(ItemId::SWORD)),
                (1, ItemStack::new(ItemId::HEALTH_PACK, 5)),
            ]
        }
        GameMode::Tdm | GameMode::Ffa | GameMode::Ctf => {
            // Standard modes: Just a Sword
            vec![(0, ItemStack::single(ItemId::SWORD))]
        }
        GameMode::BrLite => {
            // BR Lite: Start empty, find loot on the ground
            vec![]
        }
    }
}

/// Apply a starting loadout to a hotbar.
fn give_starting_loadout(hotbar: &mut Hotbar, mode: GameMode) {
    // Clear hotbar first
    for slot in 0..hotbar.size() {
        hotbar.set(slot as u8, None);
    }

    // Apply loadout
    let loadout = get_starting_loadout(mode);
    for (slot, stack) in loadout {
        hotbar.set(slot, Some(stack));
    }
}

// T078: Test Training mode gives Sword + Health Packs
#[test]
fn test_training_mode_loadout() {
    let mut hotbar = Hotbar::new(9);
    give_starting_loadout(&mut hotbar, GameMode::Training);

    // Should have Sword in slot 0
    assert_eq!(
        hotbar.get(0).map(|s| s.item_id),
        Some(ItemId::SWORD),
        "Training should have Sword in slot 0"
    );
    assert_eq!(hotbar.get(0).map(|s| s.quantity), Some(1));

    // Should have Health Packs in slot 1
    assert_eq!(
        hotbar.get(1).map(|s| s.item_id),
        Some(ItemId::HEALTH_PACK),
        "Training should have Health Packs in slot 1"
    );
    assert_eq!(hotbar.get(1).map(|s| s.quantity), Some(5));

    // Other slots should be empty
    assert!(hotbar.get(2).is_none());
}

// T079: Test TDM/FFA/CTF give Sword only
#[test]
fn test_tdm_mode_loadout() {
    let mut hotbar = Hotbar::new(9);
    give_starting_loadout(&mut hotbar, GameMode::Tdm);

    // Should have Sword in slot 0
    assert_eq!(hotbar.get(0).map(|s| s.item_id), Some(ItemId::SWORD));
    assert_eq!(hotbar.get(0).map(|s| s.quantity), Some(1));

    // Other slots should be empty
    assert!(hotbar.get(1).is_none());
}

#[test]
fn test_ffa_mode_loadout() {
    let mut hotbar = Hotbar::new(9);
    give_starting_loadout(&mut hotbar, GameMode::Ffa);

    assert_eq!(hotbar.get(0).map(|s| s.item_id), Some(ItemId::SWORD));
    assert!(hotbar.get(1).is_none());
}

#[test]
fn test_ctf_mode_loadout() {
    let mut hotbar = Hotbar::new(9);
    give_starting_loadout(&mut hotbar, GameMode::Ctf);

    assert_eq!(hotbar.get(0).map(|s| s.item_id), Some(ItemId::SWORD));
    assert!(hotbar.get(1).is_none());
}

// T080: Test BR Lite starts with empty hotbar
#[test]
fn test_br_lite_mode_loadout() {
    let mut hotbar = Hotbar::new(9);
    // Pre-fill with some items
    hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));

    give_starting_loadout(&mut hotbar, GameMode::BrLite);

    // Should be completely empty
    for slot in 0..hotbar.size() {
        assert!(
            hotbar.get(slot as u8).is_none(),
            "BR Lite slot {} should be empty",
            slot
        );
    }
}

// Test loadout clears existing items
#[test]
fn test_loadout_clears_existing() {
    let mut hotbar = Hotbar::new(9);
    // Fill all slots
    for i in 0..9 {
        hotbar.set(i, Some(ItemStack::single(ItemId::SWORD)));
    }

    give_starting_loadout(&mut hotbar, GameMode::Tdm);

    // Should have only the new loadout
    assert_eq!(hotbar.get(0).map(|s| s.item_id), Some(ItemId::SWORD));
    for slot in 1..9 {
        assert!(
            hotbar.get(slot).is_none(),
            "Slot {} should be cleared",
            slot
        );
    }
}
