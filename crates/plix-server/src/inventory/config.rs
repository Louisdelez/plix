//! Inventory configuration settings.

use plix_common::inventory::{Hotbar, ItemStack};
use plix_common::types::ItemId;
use plix_common::GameMode;

/// Configuration for the inventory system.
#[derive(Debug, Clone)]
pub struct InventoryConfig {
    /// Range in blocks for automatic item pickup (default: 1.5)
    pub pickup_range: f32,
    /// Number of hotbar slots (default: 9)
    pub hotbar_slots: usize,
}

/// Get the starting loadout for a game mode.
/// Returns a vec of (slot, ItemStack) pairs.
pub fn get_starting_loadout(mode: GameMode) -> Vec<(u8, ItemStack)> {
    match mode {
        GameMode::Training => {
            // Training: Sword + Bow + Health Packs + Scrap for practice
            vec![
                (0, ItemStack::single(ItemId::SWORD)),
                (1, ItemStack::single(ItemId::BOW)),
                (2, ItemStack::new(ItemId::HEALTH_PACK, 5)),
                (3, ItemStack::new(ItemId::SCRAP, 5)),
            ]
        }
        GameMode::Tdm | GameMode::Ffa | GameMode::Ctf => {
            // Standard modes: Sword + Bow
            vec![
                (0, ItemStack::single(ItemId::SWORD)),
                (1, ItemStack::single(ItemId::BOW)),
            ]
        }
        GameMode::BrLite => {
            // BR Lite: Start empty, find loot on the ground
            vec![]
        }
    }
}

/// Apply a starting loadout to a hotbar based on game mode.
pub fn give_starting_loadout(hotbar: &mut Hotbar, mode: GameMode) {
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

impl Default for InventoryConfig {
    fn default() -> Self {
        Self {
            pickup_range: 1.5,
            hotbar_slots: 9,
        }
    }
}

impl InventoryConfig {
    /// Create a new inventory config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a config with custom hotbar size.
    pub fn with_hotbar_slots(mut self, slots: usize) -> Self {
        self.hotbar_slots = slots;
        self
    }

    /// Create a config with custom pickup range.
    pub fn with_pickup_range(mut self, range: f32) -> Self {
        self.pickup_range = range;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = InventoryConfig::default();
        assert_eq!(config.pickup_range, 1.5);
        assert_eq!(config.hotbar_slots, 9);
    }

    #[test]
    fn test_builder_pattern() {
        let config = InventoryConfig::new()
            .with_hotbar_slots(5)
            .with_pickup_range(2.0);
        assert_eq!(config.hotbar_slots, 5);
        assert_eq!(config.pickup_range, 2.0);
    }

    #[test]
    fn test_training_loadout() {
        let loadout = get_starting_loadout(GameMode::Training);
        assert_eq!(loadout.len(), 4);
        assert_eq!(loadout[0], (0, ItemStack::single(ItemId::SWORD)));
        assert_eq!(loadout[1], (1, ItemStack::single(ItemId::BOW)));
        assert_eq!(loadout[2], (2, ItemStack::new(ItemId::HEALTH_PACK, 5)));
        assert_eq!(loadout[3], (3, ItemStack::new(ItemId::SCRAP, 5)));
    }

    #[test]
    fn test_standard_mode_loadout() {
        for mode in [GameMode::Tdm, GameMode::Ffa, GameMode::Ctf] {
            let loadout = get_starting_loadout(mode);
            assert_eq!(loadout.len(), 2);
            assert_eq!(loadout[0], (0, ItemStack::single(ItemId::SWORD)));
            assert_eq!(loadout[1], (1, ItemStack::single(ItemId::BOW)));
        }
    }

    #[test]
    fn test_br_lite_loadout() {
        let loadout = get_starting_loadout(GameMode::BrLite);
        assert!(loadout.is_empty());
    }

    #[test]
    fn test_give_starting_loadout() {
        let mut hotbar = Hotbar::new(9);
        // Fill with items first
        for i in 0..4 {
            hotbar.set(i, Some(ItemStack::single(ItemId::HEALTH_PACK)));
        }

        give_starting_loadout(&mut hotbar, GameMode::Tdm);

        // Should have sword in slot 0, bow in slot 1
        assert_eq!(hotbar.get(0).map(|s| s.item_id), Some(ItemId::SWORD));
        assert_eq!(hotbar.get(1).map(|s| s.item_id), Some(ItemId::BOW));
        // Other slots should be cleared
        assert!(hotbar.get(2).is_none());
        assert!(hotbar.get(3).is_none());
    }
}
