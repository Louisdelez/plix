//! Item type definitions and item kind enumeration.

use serde::{Deserialize, Serialize};

use crate::types::ItemId;

/// Category of item determining its behavior when used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    /// Weapon items deal damage on use (e.g., Sword)
    Weapon,
    /// Consumable items are used up and provide an effect (e.g., Health Pack)
    Consumable,
    /// Tool items perform world interactions (e.g., Block Placer)
    Tool,
    /// Resource items are used as crafting ingredients (e.g., Scrap)
    Resource,
}

impl Default for ItemKind {
    fn default() -> Self {
        ItemKind::Weapon
    }
}

/// Type of weapon determining attack behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WeaponType {
    /// Melee weapons use cone hit detection (e.g., Sword, Fist)
    #[default]
    Melee,
    /// Ranged weapons spawn projectiles (e.g., Bow)
    Ranged,
}

/// Static definition of an item type (immutable data).
#[derive(Debug, Clone)]
pub struct ItemDef {
    /// Unique identifier for this item type
    pub id: ItemId,
    /// Display name for the item
    pub name: &'static str,
    /// Category determining behavior
    pub kind: ItemKind,
    /// Maximum stack size (1 for non-stackable, 16 for consumables)
    pub max_stack: u8,
    /// Type-specific parameter:
    /// - Weapon: damage value
    /// - Consumable: effect value (e.g., heal amount)
    /// - Tool: tool-specific value
    pub param: u16,
}

impl ItemDef {
    /// Create a new item definition.
    pub const fn new(
        id: ItemId,
        name: &'static str,
        kind: ItemKind,
        max_stack: u8,
        param: u16,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            max_stack,
            param,
        }
    }

    /// Check if this item can stack.
    pub const fn is_stackable(&self) -> bool {
        self.max_stack > 1
    }

    /// Get damage value (for weapons).
    pub const fn damage(&self) -> u16 {
        self.param
    }

    /// Get heal amount (for consumables).
    pub const fn heal_amount(&self) -> u16 {
        self.param
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_kind_default() {
        assert_eq!(ItemKind::default(), ItemKind::Weapon);
    }

    #[test]
    fn test_item_kind_serde() {
        let weapon = ItemKind::Weapon;
        let json = serde_json::to_string(&weapon).unwrap();
        let parsed: ItemKind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ItemKind::Weapon);

        let consumable = ItemKind::Consumable;
        let json = serde_json::to_string(&consumable).unwrap();
        let parsed: ItemKind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ItemKind::Consumable);

        let tool = ItemKind::Tool;
        let json = serde_json::to_string(&tool).unwrap();
        let parsed: ItemKind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ItemKind::Tool);
    }

    #[test]
    fn test_item_def_weapon() {
        let sword = ItemDef::new(ItemId::SWORD, "Sword", ItemKind::Weapon, 1, 25);
        assert_eq!(sword.id, ItemId::SWORD);
        assert_eq!(sword.name, "Sword");
        assert_eq!(sword.kind, ItemKind::Weapon);
        assert_eq!(sword.max_stack, 1);
        assert_eq!(sword.damage(), 25);
        assert!(!sword.is_stackable());
    }

    #[test]
    fn test_item_def_consumable() {
        let health_pack = ItemDef::new(
            ItemId::HEALTH_PACK,
            "Health Pack",
            ItemKind::Consumable,
            16,
            50,
        );
        assert_eq!(health_pack.id, ItemId::HEALTH_PACK);
        assert_eq!(health_pack.kind, ItemKind::Consumable);
        assert_eq!(health_pack.max_stack, 16);
        assert_eq!(health_pack.heal_amount(), 50);
        assert!(health_pack.is_stackable());
    }

    #[test]
    fn test_item_def_tool() {
        let block_placer = ItemDef::new(ItemId::BLOCK_PLACER, "Block Placer", ItemKind::Tool, 1, 0);
        assert_eq!(block_placer.id, ItemId::BLOCK_PLACER);
        assert_eq!(block_placer.kind, ItemKind::Tool);
        assert_eq!(block_placer.max_stack, 1);
        assert!(!block_placer.is_stackable());
    }
}
