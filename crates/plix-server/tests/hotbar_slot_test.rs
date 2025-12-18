//! Tests for hotbar slot selection (User Story 1)
//!
//! Verifies:
//! - Slot selection validates bounds (0-8 for 9 slots)
//! - Slot selection syncs to client via InventoryUpdate
//! - Active slot is always valid after any operation

use plix_common::inventory::{Hotbar, ItemStack, DEFAULT_HOTBAR_SIZE};
use plix_common::types::ItemId;

// T021: Test slot selection validates bounds
#[test]
fn test_slot_selection_validates_bounds() {
    let mut hotbar = Hotbar::new(9);

    // Valid slot indices (0-8)
    assert!(hotbar.set_active_slot(0));
    assert_eq!(hotbar.active_slot(), 0);

    assert!(hotbar.set_active_slot(4));
    assert_eq!(hotbar.active_slot(), 4);

    assert!(hotbar.set_active_slot(8));
    assert_eq!(hotbar.active_slot(), 8);

    // Invalid slot indices
    assert!(!hotbar.set_active_slot(9));
    assert_eq!(hotbar.active_slot(), 8); // Should remain unchanged

    assert!(!hotbar.set_active_slot(100));
    assert_eq!(hotbar.active_slot(), 8); // Should remain unchanged

    assert!(!hotbar.set_active_slot(255));
    assert_eq!(hotbar.active_slot(), 8); // Should remain unchanged
}

#[test]
fn test_slot_selection_with_custom_size() {
    // Test with 5-slot hotbar (minimum per spec)
    let mut hotbar = Hotbar::new(5);

    assert!(hotbar.set_active_slot(0));
    assert!(hotbar.set_active_slot(4));
    assert!(!hotbar.set_active_slot(5)); // Out of bounds

    assert_eq!(hotbar.active_slot(), 4);
}

// T022: Test slot selection would sync to client (tested here at hotbar level)
#[test]
fn test_slot_selection_produces_snapshot() {
    use plix_common::protocol::{InventorySnapshot, SlotSnapshot};

    let mut hotbar = Hotbar::new(9);
    hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
    hotbar.set(2, Some(ItemStack::new(ItemId::HEALTH_PACK, 5)));
    hotbar.set_active_slot(2);

    // Create snapshot from hotbar (this is how server would replicate)
    let snapshot = create_inventory_snapshot(&hotbar);

    assert_eq!(snapshot.slots.len(), 9);
    assert_eq!(snapshot.active_slot, 2);

    // Slot 0 should have sword
    assert_eq!(snapshot.slots[0].item_id, ItemId::SWORD);
    assert_eq!(snapshot.slots[0].quantity, 1);

    // Slot 1 should be empty
    assert!(snapshot.slots[1].is_empty());

    // Slot 2 should have health packs
    assert_eq!(snapshot.slots[2].item_id, ItemId::HEALTH_PACK);
    assert_eq!(snapshot.slots[2].quantity, 5);
}

// T023: Test active_slot always valid after any operation
#[test]
fn test_active_slot_always_valid() {
    let mut hotbar = Hotbar::new(9);

    // After creation
    assert!(hotbar.active_slot() < hotbar.size() as u8);

    // After setting valid slot
    hotbar.set_active_slot(5);
    assert!(hotbar.active_slot() < hotbar.size() as u8);

    // After failed set (invalid slot)
    hotbar.set_active_slot(100);
    assert!(hotbar.active_slot() < hotbar.size() as u8);
    assert_eq!(hotbar.active_slot(), 5); // Unchanged

    // After adding items
    hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
    assert!(hotbar.active_slot() < hotbar.size() as u8);

    // After removing items
    hotbar.set(0, None);
    assert!(hotbar.active_slot() < hotbar.size() as u8);

    // After clearing
    hotbar.clear();
    assert!(hotbar.active_slot() < hotbar.size() as u8);
}

#[test]
fn test_active_slot_default_is_zero() {
    let hotbar = Hotbar::with_default_size();
    assert_eq!(hotbar.active_slot(), 0);
    assert_eq!(hotbar.size(), DEFAULT_HOTBAR_SIZE);
}

#[test]
fn test_slot_selection_preserves_items() {
    let mut hotbar = Hotbar::new(9);

    // Add items
    hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
    hotbar.set(5, Some(ItemStack::new(ItemId::HEALTH_PACK, 10)));

    // Select different slots
    hotbar.set_active_slot(0);
    assert_eq!(hotbar.active_item().map(|s| s.item_id), Some(ItemId::SWORD));

    hotbar.set_active_slot(5);
    assert_eq!(
        hotbar.active_item().map(|s| s.item_id),
        Some(ItemId::HEALTH_PACK)
    );

    hotbar.set_active_slot(3); // Empty slot
    assert!(hotbar.active_item().is_none());

    // Items should still be in their slots
    assert_eq!(hotbar.get(0).map(|s| s.item_id), Some(ItemId::SWORD));
    assert_eq!(hotbar.get(5).map(|s| s.item_id), Some(ItemId::HEALTH_PACK));
}

// Helper function to create inventory snapshot from hotbar
fn create_inventory_snapshot(hotbar: &Hotbar) -> plix_common::protocol::InventorySnapshot {
    use plix_common::protocol::{InventorySnapshot, SlotSnapshot};

    let slots: Vec<SlotSnapshot> = hotbar
        .slots()
        .iter()
        .map(|slot| match slot {
            Some(stack) => SlotSnapshot::new(stack.item_id, stack.quantity),
            None => SlotSnapshot::empty(),
        })
        .collect();

    InventorySnapshot {
        slots,
        active_slot: hotbar.active_slot(),
    }
}
