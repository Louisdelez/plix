//! Item registry with static item definitions.
//!
//! Provides lookup for all item types used in the game.

use plix_common::inventory::{ItemDef, ItemId, ItemKind};

/// Static item definitions for all items in the game.
pub static ITEM_DEFS: &[ItemDef] = &[
    // Sword: 25 damage weapon (per spec clarification)
    ItemDef::new(ItemId::SWORD, "Sword", ItemKind::Weapon, 1, 25),
    // Health Pack: heals 50 HP, stacks to 16 (per spec clarification)
    ItemDef::new(
        ItemId::HEALTH_PACK,
        "Health Pack",
        ItemKind::Consumable,
        16,
        50,
    ),
    // Block Placer: places stone blocks
    ItemDef::new(ItemId::BLOCK_PLACER, "Block Placer", ItemKind::Tool, 1, 0),
    // Bow: 15 damage ranged weapon (per spec clarification)
    ItemDef::new(ItemId::BOW, "Bow", ItemKind::Weapon, 1, 15),
    // Scrap: crafting resource, stacks to 16
    ItemDef::new(ItemId::SCRAP, "Scrap", ItemKind::Resource, 16, 0),
];

/// Get the item definition for an item ID.
pub fn get_item_def(id: ItemId) -> Option<&'static ItemDef> {
    ITEM_DEFS.iter().find(|def| def.id == id)
}

/// Get the damage value for a weapon item.
pub fn get_weapon_damage(id: ItemId) -> Option<u16> {
    get_item_def(id).and_then(|def| {
        if def.kind == ItemKind::Weapon {
            Some(def.damage())
        } else {
            None
        }
    })
}

/// Get the heal amount for a consumable item.
pub fn get_heal_amount(id: ItemId) -> Option<u16> {
    get_item_def(id).and_then(|def| {
        if def.kind == ItemKind::Consumable {
            Some(def.heal_amount())
        } else {
            None
        }
    })
}

/// Get the max stack size for an item.
pub fn get_max_stack(id: ItemId) -> u8 {
    get_item_def(id).map(|def| def.max_stack).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sword_definition() {
        let sword = get_item_def(ItemId::SWORD).unwrap();
        assert_eq!(sword.name, "Sword");
        assert_eq!(sword.kind, ItemKind::Weapon);
        assert_eq!(sword.max_stack, 1);
        assert_eq!(sword.damage(), 25);
    }

    #[test]
    fn test_health_pack_definition() {
        let health_pack = get_item_def(ItemId::HEALTH_PACK).unwrap();
        assert_eq!(health_pack.name, "Health Pack");
        assert_eq!(health_pack.kind, ItemKind::Consumable);
        assert_eq!(health_pack.max_stack, 16);
        assert_eq!(health_pack.heal_amount(), 50);
    }

    #[test]
    fn test_block_placer_definition() {
        let block_placer = get_item_def(ItemId::BLOCK_PLACER).unwrap();
        assert_eq!(block_placer.name, "Block Placer");
        assert_eq!(block_placer.kind, ItemKind::Tool);
        assert_eq!(block_placer.max_stack, 1);
    }

    #[test]
    fn test_bow_definition() {
        let bow = get_item_def(ItemId::BOW).unwrap();
        assert_eq!(bow.name, "Bow");
        assert_eq!(bow.kind, ItemKind::Weapon);
        assert_eq!(bow.max_stack, 1);
        assert_eq!(bow.damage(), 15);
    }

    #[test]
    fn test_get_weapon_damage() {
        assert_eq!(get_weapon_damage(ItemId::SWORD), Some(25));
        assert_eq!(get_weapon_damage(ItemId::BOW), Some(15));
        assert_eq!(get_weapon_damage(ItemId::HEALTH_PACK), None);
        assert_eq!(get_weapon_damage(ItemId::BLOCK_PLACER), None);
    }

    #[test]
    fn test_get_heal_amount() {
        assert_eq!(get_heal_amount(ItemId::HEALTH_PACK), Some(50));
        assert_eq!(get_heal_amount(ItemId::SWORD), None);
        assert_eq!(get_heal_amount(ItemId::BLOCK_PLACER), None);
    }

    #[test]
    fn test_get_max_stack() {
        assert_eq!(get_max_stack(ItemId::SWORD), 1);
        assert_eq!(get_max_stack(ItemId::HEALTH_PACK), 16);
        assert_eq!(get_max_stack(ItemId::BLOCK_PLACER), 1);
        assert_eq!(get_max_stack(ItemId::NONE), 1); // Unknown item defaults to 1
    }

    #[test]
    fn test_invalid_item_id() {
        assert!(get_item_def(ItemId::NONE).is_none());
        assert!(get_item_def(ItemId(999)).is_none());
    }
}
