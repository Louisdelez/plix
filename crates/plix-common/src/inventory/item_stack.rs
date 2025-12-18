//! Item stack representation for inventory slots.

use serde::{Deserialize, Serialize};

use crate::types::ItemId;

/// A stack of items in an inventory slot.
///
/// Represents a quantity of a single item type. Weapons and tools
/// have quantity 1, while consumables can stack up to max_stack (16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemStack {
    /// The item type
    pub item_id: ItemId,
    /// Current quantity (1 for non-stackable items)
    pub quantity: u8,
}

impl ItemStack {
    /// Maximum stack size for consumables.
    pub const MAX_STACK_SIZE: u8 = 16;

    /// Create a new item stack.
    pub const fn new(item_id: ItemId, quantity: u8) -> Self {
        Self { item_id, quantity }
    }

    /// Create a single-item stack (for weapons/tools).
    pub const fn single(item_id: ItemId) -> Self {
        Self {
            item_id,
            quantity: 1,
        }
    }

    /// Check if this stack is empty (quantity 0 or invalid item).
    pub fn is_empty(&self) -> bool {
        self.quantity == 0 || !self.item_id.is_valid()
    }

    /// Check if this stack can accept more of the same item.
    pub fn can_add(&self, item_id: ItemId, max_stack: u8) -> bool {
        if self.item_id != item_id {
            return false;
        }
        self.quantity < max_stack
    }

    /// Try to add quantity to this stack, returns amount that couldn't be added.
    pub fn try_add(&mut self, amount: u8, max_stack: u8) -> u8 {
        let space = max_stack.saturating_sub(self.quantity);
        let to_add = amount.min(space);
        self.quantity += to_add;
        amount - to_add
    }

    /// Remove one item from the stack, returns true if successful.
    pub fn decrement(&mut self) -> bool {
        if self.quantity > 0 {
            self.quantity -= 1;
            true
        } else {
            false
        }
    }

    /// Remove a specific amount from the stack, returns amount actually removed.
    pub fn remove(&mut self, amount: u8) -> u8 {
        let to_remove = amount.min(self.quantity);
        self.quantity -= to_remove;
        to_remove
    }
}

impl Default for ItemStack {
    fn default() -> Self {
        Self {
            item_id: ItemId::NONE,
            quantity: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_stack_new() {
        let stack = ItemStack::new(ItemId::HEALTH_PACK, 5);
        assert_eq!(stack.item_id, ItemId::HEALTH_PACK);
        assert_eq!(stack.quantity, 5);
    }

    #[test]
    fn test_item_stack_single() {
        let stack = ItemStack::single(ItemId::SWORD);
        assert_eq!(stack.item_id, ItemId::SWORD);
        assert_eq!(stack.quantity, 1);
    }

    #[test]
    fn test_item_stack_default_is_empty() {
        let stack = ItemStack::default();
        assert!(stack.is_empty());
    }

    #[test]
    fn test_item_stack_is_empty_zero_quantity() {
        let stack = ItemStack::new(ItemId::SWORD, 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_item_stack_is_empty_invalid_id() {
        let stack = ItemStack::new(ItemId::NONE, 5);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_item_stack_can_add_same_item() {
        let stack = ItemStack::new(ItemId::HEALTH_PACK, 5);
        assert!(stack.can_add(ItemId::HEALTH_PACK, 16));
    }

    #[test]
    fn test_item_stack_can_add_different_item() {
        let stack = ItemStack::new(ItemId::HEALTH_PACK, 5);
        assert!(!stack.can_add(ItemId::SWORD, 16));
    }

    #[test]
    fn test_item_stack_can_add_full() {
        let stack = ItemStack::new(ItemId::HEALTH_PACK, 16);
        assert!(!stack.can_add(ItemId::HEALTH_PACK, 16));
    }

    #[test]
    fn test_item_stack_try_add() {
        let mut stack = ItemStack::new(ItemId::HEALTH_PACK, 10);
        let remaining = stack.try_add(4, 16);
        assert_eq!(stack.quantity, 14);
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_item_stack_try_add_overflow() {
        let mut stack = ItemStack::new(ItemId::HEALTH_PACK, 10);
        let remaining = stack.try_add(10, 16);
        assert_eq!(stack.quantity, 16);
        assert_eq!(remaining, 4);
    }

    #[test]
    fn test_item_stack_decrement() {
        let mut stack = ItemStack::new(ItemId::HEALTH_PACK, 3);
        assert!(stack.decrement());
        assert_eq!(stack.quantity, 2);
        assert!(stack.decrement());
        assert_eq!(stack.quantity, 1);
        assert!(stack.decrement());
        assert_eq!(stack.quantity, 0);
        assert!(!stack.decrement()); // Can't go below 0
        assert_eq!(stack.quantity, 0);
    }

    #[test]
    fn test_item_stack_remove() {
        let mut stack = ItemStack::new(ItemId::HEALTH_PACK, 10);
        let removed = stack.remove(3);
        assert_eq!(removed, 3);
        assert_eq!(stack.quantity, 7);
    }

    #[test]
    fn test_item_stack_remove_more_than_available() {
        let mut stack = ItemStack::new(ItemId::HEALTH_PACK, 5);
        let removed = stack.remove(10);
        assert_eq!(removed, 5);
        assert_eq!(stack.quantity, 0);
    }

    #[test]
    fn test_item_stack_serde() {
        let stack = ItemStack::new(ItemId::SWORD, 1);
        let json = serde_json::to_string(&stack).unwrap();
        let parsed: ItemStack = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, stack);
    }
}
