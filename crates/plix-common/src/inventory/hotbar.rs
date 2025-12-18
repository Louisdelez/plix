//! Hotbar container for player inventory.

use serde::{Deserialize, Serialize};

use super::item_stack::ItemStack;
use crate::types::ItemId;

/// Default number of hotbar slots.
pub const DEFAULT_HOTBAR_SIZE: usize = 9;

/// Player's hotbar inventory container.
///
/// Contains a fixed number of slots (default 9) that can hold item stacks.
/// One slot is always "active" and represents the currently selected item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotbar {
    /// Slots in the hotbar (None = empty slot)
    slots: Vec<Option<ItemStack>>,
    /// Currently active slot index (0-based)
    active_slot: u8,
}

impl Hotbar {
    /// Create a new hotbar with the given number of slots.
    pub fn new(size: usize) -> Self {
        Self {
            slots: vec![None; size],
            active_slot: 0,
        }
    }

    /// Create a new hotbar with default size (9 slots).
    pub fn with_default_size() -> Self {
        Self::new(DEFAULT_HOTBAR_SIZE)
    }

    /// Get the number of slots in the hotbar.
    pub fn size(&self) -> usize {
        self.slots.len()
    }

    /// Get the currently active slot index.
    pub fn active_slot(&self) -> u8 {
        self.active_slot
    }

    /// Set the active slot index. Returns false if index is out of bounds.
    pub fn set_active_slot(&mut self, index: u8) -> bool {
        if (index as usize) < self.slots.len() {
            self.active_slot = index;
            true
        } else {
            false
        }
    }

    /// Get the item in the active slot.
    pub fn active_item(&self) -> Option<&ItemStack> {
        self.slots
            .get(self.active_slot as usize)
            .and_then(|s| s.as_ref())
    }

    /// Get a mutable reference to the item in the active slot.
    pub fn active_item_mut(&mut self) -> Option<&mut ItemStack> {
        self.slots
            .get_mut(self.active_slot as usize)
            .and_then(|s| s.as_mut())
    }

    /// Get the item in a specific slot.
    pub fn get(&self, index: u8) -> Option<&ItemStack> {
        self.slots.get(index as usize).and_then(|s| s.as_ref())
    }

    /// Get a mutable reference to a specific slot.
    pub fn get_mut(&mut self, index: u8) -> Option<&mut Option<ItemStack>> {
        self.slots.get_mut(index as usize)
    }

    /// Set the item in a specific slot. Returns false if index is out of bounds.
    pub fn set(&mut self, index: u8, stack: Option<ItemStack>) -> bool {
        if let Some(slot) = self.slots.get_mut(index as usize) {
            *slot = stack;
            true
        } else {
            false
        }
    }

    /// Find the first empty slot index, if any.
    pub fn find_empty_slot(&self) -> Option<u8> {
        self.slots.iter().position(|s| s.is_none()).map(|i| i as u8)
    }

    /// Find a slot containing the given item type that can accept more.
    pub fn find_stackable_slot(&self, item_id: ItemId, max_stack: u8) -> Option<u8> {
        self.slots
            .iter()
            .position(|s| {
                s.as_ref()
                    .map(|stack| stack.can_add(item_id, max_stack))
                    .unwrap_or(false)
            })
            .map(|i| i as u8)
    }

    /// Try to add an item to the hotbar. Returns the remaining quantity that couldn't be added.
    pub fn try_add_item(&mut self, item_id: ItemId, quantity: u8, max_stack: u8) -> u8 {
        let mut remaining = quantity;

        // First, try to stack with existing items
        while remaining > 0 {
            if let Some(slot_idx) = self.find_stackable_slot(item_id, max_stack) {
                if let Some(Some(stack)) = self.slots.get_mut(slot_idx as usize) {
                    remaining = stack.try_add(remaining, max_stack);
                }
            } else {
                break;
            }
        }

        // Then, try to fill empty slots
        while remaining > 0 {
            if let Some(slot_idx) = self.find_empty_slot() {
                let to_add = remaining.min(max_stack);
                self.slots[slot_idx as usize] = Some(ItemStack::new(item_id, to_add));
                remaining -= to_add;
            } else {
                break;
            }
        }

        remaining
    }

    /// Check if the hotbar is completely full.
    pub fn is_full(&self) -> bool {
        self.slots.iter().all(|s| s.is_some())
    }

    /// Check if the hotbar is completely empty.
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(|s| s.is_none())
    }

    /// Clear all items from the hotbar.
    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
    }

    /// Get all slots as a slice.
    pub fn slots(&self) -> &[Option<ItemStack>] {
        &self.slots
    }

    /// Remove the item in the active slot if it's empty (quantity 0).
    pub fn cleanup_active_slot(&mut self) {
        if let Some(Some(stack)) = self.slots.get(self.active_slot as usize) {
            if stack.is_empty() {
                self.slots[self.active_slot as usize] = None;
            }
        }
    }

    /// Iterate over all items in the hotbar (for death drops, etc.).
    pub fn items(&self) -> impl Iterator<Item = (u8, &ItemStack)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|stack| (i as u8, stack)))
    }

    /// Count the total quantity of a specific item across all slots.
    ///
    /// Used by crafting to check if player has enough ingredients.
    pub fn count_item(&self, item_id: ItemId) -> u16 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .filter(|stack| stack.item_id == item_id)
            .map(|stack| stack.quantity as u16)
            .sum()
    }

    /// Consume a specific quantity of an item from the hotbar.
    ///
    /// Returns true if the full quantity was consumed, false if insufficient.
    /// Consumes from first matching slot(s) found.
    /// Used by crafting to remove ingredients atomically.
    pub fn consume_items(&mut self, item_id: ItemId, mut quantity: u8) -> bool {
        // First check if we have enough
        if self.count_item(item_id) < quantity as u16 {
            return false;
        }

        // Consume from slots
        for slot in &mut self.slots {
            if quantity == 0 {
                break;
            }
            if let Some(stack) = slot {
                if stack.item_id == item_id {
                    let to_consume = quantity.min(stack.quantity);
                    stack.quantity -= to_consume;
                    quantity -= to_consume;

                    // Remove empty stacks
                    if stack.quantity == 0 {
                        *slot = None;
                    }
                }
            }
        }

        true
    }

    /// Check if the hotbar can accommodate an item of the given type and quantity.
    ///
    /// Returns true if there's space (either empty slot or stackable with existing).
    /// Does not modify the hotbar.
    /// Used by crafting to validate output can be added before consuming inputs.
    pub fn can_add(&self, item_id: ItemId, quantity: u8, max_stack: u8) -> bool {
        let mut remaining = quantity;

        // Check existing stacks for space
        for slot in &self.slots {
            if remaining == 0 {
                return true;
            }
            if let Some(stack) = slot {
                if stack.item_id == item_id && stack.quantity < max_stack {
                    let available = max_stack - stack.quantity;
                    remaining = remaining.saturating_sub(available);
                }
            }
        }

        // Check empty slots
        for slot in &self.slots {
            if remaining == 0 {
                return true;
            }
            if slot.is_none() {
                remaining = remaining.saturating_sub(max_stack);
            }
        }

        remaining == 0
    }
}

impl Default for Hotbar {
    fn default() -> Self {
        Self::with_default_size()
    }
}

impl PartialEq for Hotbar {
    fn eq(&self, other: &Self) -> bool {
        self.slots == other.slots && self.active_slot == other.active_slot
    }
}

impl Eq for Hotbar {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotbar_new() {
        let hotbar = Hotbar::new(5);
        assert_eq!(hotbar.size(), 5);
        assert_eq!(hotbar.active_slot(), 0);
        assert!(hotbar.is_empty());
    }

    #[test]
    fn test_hotbar_default_size() {
        let hotbar = Hotbar::with_default_size();
        assert_eq!(hotbar.size(), DEFAULT_HOTBAR_SIZE);
    }

    #[test]
    fn test_hotbar_set_active_slot() {
        let mut hotbar = Hotbar::new(5);
        assert!(hotbar.set_active_slot(3));
        assert_eq!(hotbar.active_slot(), 3);

        // Out of bounds should fail
        assert!(!hotbar.set_active_slot(5));
        assert_eq!(hotbar.active_slot(), 3); // Unchanged
    }

    #[test]
    fn test_hotbar_set_and_get() {
        let mut hotbar = Hotbar::new(5);
        let sword = ItemStack::single(ItemId::SWORD);

        assert!(hotbar.set(0, Some(sword)));
        assert_eq!(hotbar.get(0), Some(&sword));
        assert_eq!(hotbar.get(1), None);

        // Out of bounds
        assert!(!hotbar.set(10, Some(sword)));
    }

    #[test]
    fn test_hotbar_active_item() {
        let mut hotbar = Hotbar::new(5);
        assert!(hotbar.active_item().is_none());

        let sword = ItemStack::single(ItemId::SWORD);
        hotbar.set(0, Some(sword));
        assert_eq!(hotbar.active_item(), Some(&sword));

        hotbar.set_active_slot(1);
        assert!(hotbar.active_item().is_none());
    }

    #[test]
    fn test_hotbar_find_empty_slot() {
        let mut hotbar = Hotbar::new(3);
        assert_eq!(hotbar.find_empty_slot(), Some(0));

        hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
        assert_eq!(hotbar.find_empty_slot(), Some(1));

        hotbar.set(1, Some(ItemStack::single(ItemId::SWORD)));
        hotbar.set(2, Some(ItemStack::single(ItemId::SWORD)));
        assert_eq!(hotbar.find_empty_slot(), None);
    }

    #[test]
    fn test_hotbar_find_stackable_slot() {
        let mut hotbar = Hotbar::new(3);

        // No stackable slots when empty
        assert!(hotbar
            .find_stackable_slot(ItemId::HEALTH_PACK, 16)
            .is_none());

        // Add a partial stack
        hotbar.set(1, Some(ItemStack::new(ItemId::HEALTH_PACK, 5)));
        assert_eq!(hotbar.find_stackable_slot(ItemId::HEALTH_PACK, 16), Some(1));

        // Different item type
        assert!(hotbar.find_stackable_slot(ItemId::SWORD, 1).is_none());

        // Full stack
        hotbar.set(1, Some(ItemStack::new(ItemId::HEALTH_PACK, 16)));
        assert!(hotbar
            .find_stackable_slot(ItemId::HEALTH_PACK, 16)
            .is_none());
    }

    #[test]
    fn test_hotbar_try_add_item_empty() {
        let mut hotbar = Hotbar::new(3);

        // Add to empty hotbar
        let remaining = hotbar.try_add_item(ItemId::SWORD, 1, 1);
        assert_eq!(remaining, 0);
        assert_eq!(hotbar.get(0), Some(&ItemStack::single(ItemId::SWORD)));
    }

    #[test]
    fn test_hotbar_try_add_item_stacking() {
        let mut hotbar = Hotbar::new(3);

        // Add initial stack
        hotbar.try_add_item(ItemId::HEALTH_PACK, 5, 16);
        assert_eq!(hotbar.get(0).unwrap().quantity, 5);

        // Add more, should stack
        hotbar.try_add_item(ItemId::HEALTH_PACK, 3, 16);
        assert_eq!(hotbar.get(0).unwrap().quantity, 8);
    }

    #[test]
    fn test_hotbar_try_add_item_overflow() {
        let mut hotbar = Hotbar::new(2);

        // Add more than one stack can hold
        hotbar.try_add_item(ItemId::HEALTH_PACK, 20, 16);
        assert_eq!(hotbar.get(0).unwrap().quantity, 16);
        assert_eq!(hotbar.get(1).unwrap().quantity, 4);
    }

    #[test]
    fn test_hotbar_try_add_item_full() {
        let mut hotbar = Hotbar::new(1);

        // Fill the slot
        hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));

        // Try to add different item - should fail
        let remaining = hotbar.try_add_item(ItemId::HEALTH_PACK, 5, 16);
        assert_eq!(remaining, 5);
    }

    #[test]
    fn test_hotbar_is_full() {
        let mut hotbar = Hotbar::new(2);
        assert!(!hotbar.is_full());

        hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
        assert!(!hotbar.is_full());

        hotbar.set(1, Some(ItemStack::single(ItemId::SWORD)));
        assert!(hotbar.is_full());
    }

    #[test]
    fn test_hotbar_clear() {
        let mut hotbar = Hotbar::new(3);
        hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
        hotbar.set(1, Some(ItemStack::new(ItemId::HEALTH_PACK, 5)));

        hotbar.clear();
        assert!(hotbar.is_empty());
    }

    #[test]
    fn test_hotbar_cleanup_active_slot() {
        let mut hotbar = Hotbar::new(3);
        hotbar.set(0, Some(ItemStack::new(ItemId::HEALTH_PACK, 1)));

        // Decrement to 0
        hotbar.active_item_mut().unwrap().decrement();
        assert_eq!(hotbar.active_item().unwrap().quantity, 0);

        // Cleanup should remove the empty stack
        hotbar.cleanup_active_slot();
        assert!(hotbar.active_item().is_none());
    }

    #[test]
    fn test_hotbar_items_iterator() {
        let mut hotbar = Hotbar::new(5);
        hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
        hotbar.set(2, Some(ItemStack::new(ItemId::HEALTH_PACK, 5)));
        hotbar.set(4, Some(ItemStack::single(ItemId::BLOCK_PLACER)));

        let items: Vec<_> = hotbar.items().collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].0, 0);
        assert_eq!(items[0].1.item_id, ItemId::SWORD);
        assert_eq!(items[1].0, 2);
        assert_eq!(items[1].1.item_id, ItemId::HEALTH_PACK);
        assert_eq!(items[2].0, 4);
        assert_eq!(items[2].1.item_id, ItemId::BLOCK_PLACER);
    }

    #[test]
    fn test_hotbar_serde() {
        let mut hotbar = Hotbar::new(5);
        hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
        hotbar.set_active_slot(2);

        let json = serde_json::to_string(&hotbar).unwrap();
        let parsed: Hotbar = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, hotbar);
    }

    #[test]
    fn test_hotbar_active_slot_always_valid() {
        // Per spec: active_slot always valid after any operation
        let mut hotbar = Hotbar::new(5);

        // Initially valid
        assert!(hotbar.active_slot() < hotbar.size() as u8);

        // After set_active_slot
        hotbar.set_active_slot(3);
        assert!(hotbar.active_slot() < hotbar.size() as u8);

        // Invalid set should not change it
        hotbar.set_active_slot(100);
        assert!(hotbar.active_slot() < hotbar.size() as u8);
        assert_eq!(hotbar.active_slot(), 3);
    }

    // ========================================================================
    // Crafting helper method tests (T016)
    // ========================================================================

    #[test]
    fn test_count_item_empty_hotbar() {
        let hotbar = Hotbar::new(5);
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 0);
    }

    #[test]
    fn test_count_item_single_slot() {
        let mut hotbar = Hotbar::new(5);
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 5)));
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 5);
        assert_eq!(hotbar.count_item(ItemId::SWORD), 0); // Different item
    }

    #[test]
    fn test_count_item_multiple_slots() {
        let mut hotbar = Hotbar::new(5);
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 5)));
        hotbar.set(2, Some(ItemStack::new(ItemId::SCRAP, 8)));
        hotbar.set(4, Some(ItemStack::new(ItemId::SCRAP, 3)));
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 16);
    }

    #[test]
    fn test_count_item_mixed_items() {
        let mut hotbar = Hotbar::new(5);
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 5)));
        hotbar.set(1, Some(ItemStack::single(ItemId::SWORD)));
        hotbar.set(2, Some(ItemStack::new(ItemId::SCRAP, 3)));
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 8);
        assert_eq!(hotbar.count_item(ItemId::SWORD), 1);
    }

    #[test]
    fn test_consume_items_exact_amount() {
        let mut hotbar = Hotbar::new(5);
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 5)));

        assert!(hotbar.consume_items(ItemId::SCRAP, 5));
        assert!(hotbar.get(0).is_none()); // Slot cleared
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 0);
    }

    #[test]
    fn test_consume_items_partial_stack() {
        let mut hotbar = Hotbar::new(5);
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 10)));

        assert!(hotbar.consume_items(ItemId::SCRAP, 4));
        assert_eq!(hotbar.get(0).unwrap().quantity, 6);
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 6);
    }

    #[test]
    fn test_consume_items_across_slots() {
        let mut hotbar = Hotbar::new(5);
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 3)));
        hotbar.set(2, Some(ItemStack::new(ItemId::SCRAP, 5)));

        assert!(hotbar.consume_items(ItemId::SCRAP, 5));
        // First slot fully consumed, second partially
        assert!(hotbar.get(0).is_none());
        assert_eq!(hotbar.get(2).unwrap().quantity, 3);
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 3);
    }

    #[test]
    fn test_consume_items_insufficient() {
        let mut hotbar = Hotbar::new(5);
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 3)));

        // Try to consume more than available
        assert!(!hotbar.consume_items(ItemId::SCRAP, 5));
        // Hotbar should be unchanged
        assert_eq!(hotbar.get(0).unwrap().quantity, 3);
    }

    #[test]
    fn test_consume_items_wrong_item() {
        let mut hotbar = Hotbar::new(5);
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 10)));

        // Try to consume wrong item type
        assert!(!hotbar.consume_items(ItemId::SWORD, 1));
        // Hotbar unchanged
        assert_eq!(hotbar.get(0).unwrap().quantity, 10);
    }

    #[test]
    fn test_consume_items_zero_quantity() {
        let mut hotbar = Hotbar::new(5);
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 5)));

        // Consuming 0 should always succeed
        assert!(hotbar.consume_items(ItemId::SCRAP, 0));
        assert_eq!(hotbar.get(0).unwrap().quantity, 5);
    }

    #[test]
    fn test_can_add_empty_hotbar() {
        let hotbar = Hotbar::new(5);
        assert!(hotbar.can_add(ItemId::HEALTH_PACK, 1, 16));
        assert!(hotbar.can_add(ItemId::HEALTH_PACK, 16, 16));
        // Can add up to 5 full stacks
        assert!(hotbar.can_add(ItemId::HEALTH_PACK, 80, 16));
    }

    #[test]
    fn test_can_add_stack_with_existing() {
        let mut hotbar = Hotbar::new(2);
        hotbar.set(0, Some(ItemStack::new(ItemId::HEALTH_PACK, 10)));
        hotbar.set(1, Some(ItemStack::new(ItemId::SWORD, 1)));

        // Can stack 6 more with existing
        assert!(hotbar.can_add(ItemId::HEALTH_PACK, 6, 16));
        // Cannot add 7 (only 6 space left, no empty slots)
        assert!(!hotbar.can_add(ItemId::HEALTH_PACK, 7, 16));
    }

    #[test]
    fn test_can_add_full_slots() {
        let mut hotbar = Hotbar::new(2);
        hotbar.set(0, Some(ItemStack::new(ItemId::HEALTH_PACK, 16)));
        hotbar.set(1, Some(ItemStack::single(ItemId::SWORD)));

        // Cannot add - all slots full or different items
        assert!(!hotbar.can_add(ItemId::HEALTH_PACK, 1, 16));
        assert!(!hotbar.can_add(ItemId::SCRAP, 1, 16));
    }

    #[test]
    fn test_can_add_non_stackable() {
        let mut hotbar = Hotbar::new(3);
        hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));

        // Can add to empty slots
        assert!(hotbar.can_add(ItemId::SWORD, 1, 1));
        assert!(hotbar.can_add(ItemId::SWORD, 2, 1));
        // Cannot add 3 (only 2 empty slots)
        assert!(!hotbar.can_add(ItemId::SWORD, 3, 1));
    }

    #[test]
    fn test_can_add_does_not_modify() {
        let mut hotbar = Hotbar::new(5);
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 5)));

        // Check can_add doesn't modify anything
        assert!(hotbar.can_add(ItemId::SCRAP, 10, 16));
        assert_eq!(hotbar.get(0).unwrap().quantity, 5);
        assert!(hotbar.get(1).is_none());
    }
}
