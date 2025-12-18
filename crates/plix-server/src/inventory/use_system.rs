//! Item usage system for processing active item effects.

use plix_common::inventory::{Hotbar, ItemKind};
use plix_common::types::ItemId;

use super::item_registry::{get_heal_amount, get_item_def, get_weapon_damage};

/// Result of using an active item
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UseResult {
    /// No item equipped - use default action (melee, etc.)
    NoItem,
    /// Weapon was used - attack with specified damage
    WeaponAttack { item_id: ItemId, damage: u16 },
    /// Consumable was used - apply effect
    ConsumableUsed {
        item_id: ItemId,
        heal_amount: u16,
        /// If true, the stack is now empty and should be removed
        stack_depleted: bool,
    },
    /// Tool was used - place block
    ToolUsed { item_id: ItemId },
    /// Cannot use this item (unknown type or invalid state)
    CannotUse,
}

/// Try to use the active item in the hotbar.
///
/// Returns the result describing what effect should be applied.
/// The caller is responsible for actually applying the effect (damage, healing, etc.)
/// and updating the hotbar (removing depleted consumables).
pub fn use_active_item(hotbar: &Hotbar) -> UseResult {
    let active_item = match hotbar.active_item() {
        Some(item) => item,
        None => return UseResult::NoItem,
    };

    let def = match get_item_def(active_item.item_id) {
        Some(d) => d,
        None => return UseResult::CannotUse,
    };

    match def.kind {
        ItemKind::Weapon => {
            let damage = get_weapon_damage(active_item.item_id).unwrap_or(0);
            UseResult::WeaponAttack {
                item_id: active_item.item_id,
                damage,
            }
        }
        ItemKind::Consumable => {
            let heal_amount = get_heal_amount(active_item.item_id).unwrap_or(0);
            let stack_depleted = active_item.quantity <= 1;
            UseResult::ConsumableUsed {
                item_id: active_item.item_id,
                heal_amount,
                stack_depleted,
            }
        }
        ItemKind::Tool => UseResult::ToolUsed {
            item_id: active_item.item_id,
        },
        ItemKind::Resource => {
            // Resources cannot be "used" directly - they are crafting ingredients
            UseResult::CannotUse
        }
    }
}

/// Decrement the quantity of the active consumable item.
/// Returns true if the stack was depleted and should be removed.
pub fn decrement_active_consumable(hotbar: &mut Hotbar) -> bool {
    let active_slot = hotbar.active_slot();

    if let Some(stack) = hotbar.get(active_slot) {
        if stack.quantity <= 1 {
            // Stack depleted - remove it
            hotbar.set(active_slot, None);
            true
        } else {
            // Decrement quantity
            let new_quantity = stack.quantity - 1;
            let item_id = stack.item_id;
            hotbar.set(
                active_slot,
                Some(plix_common::inventory::ItemStack::new(
                    item_id,
                    new_quantity,
                )),
            );
            false
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::inventory::{Hotbar, ItemStack};

    #[test]
    fn test_use_empty_slot() {
        let hotbar = Hotbar::new(9);
        let result = use_active_item(&hotbar);
        assert_eq!(result, UseResult::NoItem);
    }

    #[test]
    fn test_use_weapon() {
        let mut hotbar = Hotbar::new(9);
        hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
        hotbar.set_active_slot(0);

        let result = use_active_item(&hotbar);
        assert_eq!(
            result,
            UseResult::WeaponAttack {
                item_id: ItemId::SWORD,
                damage: 25,
            }
        );
    }

    #[test]
    fn test_use_consumable() {
        let mut hotbar = Hotbar::new(9);
        hotbar.set(0, Some(ItemStack::new(ItemId::HEALTH_PACK, 3)));
        hotbar.set_active_slot(0);

        let result = use_active_item(&hotbar);
        assert_eq!(
            result,
            UseResult::ConsumableUsed {
                item_id: ItemId::HEALTH_PACK,
                heal_amount: 50,
                stack_depleted: false,
            }
        );
    }

    #[test]
    fn test_use_last_consumable() {
        let mut hotbar = Hotbar::new(9);
        hotbar.set(0, Some(ItemStack::single(ItemId::HEALTH_PACK)));
        hotbar.set_active_slot(0);

        let result = use_active_item(&hotbar);
        assert_eq!(
            result,
            UseResult::ConsumableUsed {
                item_id: ItemId::HEALTH_PACK,
                heal_amount: 50,
                stack_depleted: true,
            }
        );
    }

    #[test]
    fn test_use_tool() {
        let mut hotbar = Hotbar::new(9);
        hotbar.set(0, Some(ItemStack::single(ItemId::BLOCK_PLACER)));
        hotbar.set_active_slot(0);

        let result = use_active_item(&hotbar);
        assert_eq!(
            result,
            UseResult::ToolUsed {
                item_id: ItemId::BLOCK_PLACER,
            }
        );
    }

    #[test]
    fn test_decrement_consumable() {
        let mut hotbar = Hotbar::new(9);
        hotbar.set(0, Some(ItemStack::new(ItemId::HEALTH_PACK, 3)));
        hotbar.set_active_slot(0);

        let depleted = decrement_active_consumable(&mut hotbar);
        assert!(!depleted);
        assert_eq!(hotbar.get(0).map(|s| s.quantity), Some(2));

        let depleted = decrement_active_consumable(&mut hotbar);
        assert!(!depleted);
        assert_eq!(hotbar.get(0).map(|s| s.quantity), Some(1));

        let depleted = decrement_active_consumable(&mut hotbar);
        assert!(depleted);
        assert!(hotbar.get(0).is_none());
    }
}
