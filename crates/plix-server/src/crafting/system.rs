//! Core crafting system for processing craft requests.
//!
//! Implements server-authoritative crafting with atomic transactions:
//! - Validate all preconditions before modifying state
//! - Apply changes atomically (consume inputs, add output)
//! - Return detailed failure reasons on rejection

use plix_common::inventory::Hotbar;
use plix_common::types::ItemId;

use super::errors::CraftFailReason;
use super::recipe::{Recipe, RECIPE_REGISTRY};
use crate::inventory::item_registry::get_max_stack;

/// Result of a craft attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CraftResult {
    /// Whether the craft succeeded.
    pub success: bool,
    /// Recipe that was attempted.
    pub recipe_id: String,
    /// Output item if successful.
    pub output_item: Option<ItemId>,
    /// Output quantity if successful.
    pub output_quantity: Option<u8>,
    /// Failure reason if unsuccessful.
    pub fail_reason: Option<CraftFailReason>,
}

impl CraftResult {
    /// Create a successful craft result.
    pub fn success(recipe_id: &str, output_item: ItemId, output_quantity: u8) -> Self {
        Self {
            success: true,
            recipe_id: recipe_id.to_string(),
            output_item: Some(output_item),
            output_quantity: Some(output_quantity),
            fail_reason: None,
        }
    }

    /// Create a failed craft result.
    pub fn failure(recipe_id: &str, reason: CraftFailReason) -> Self {
        Self {
            success: false,
            recipe_id: recipe_id.to_string(),
            output_item: None,
            output_quantity: None,
            fail_reason: Some(reason),
        }
    }
}

/// Server-authoritative crafting system.
///
/// Processes craft requests with atomic validation and application.
#[derive(Debug, Default)]
pub struct CraftSystem;

impl CraftSystem {
    /// Create a new craft system.
    pub fn new() -> Self {
        Self
    }

    /// Validate that the player has all required ingredients for a recipe.
    ///
    /// Returns Ok(()) if all ingredients are present, Err(MissingIngredients) otherwise.
    pub fn validate_ingredients(
        &self,
        recipe: &Recipe,
        hotbar: &Hotbar,
    ) -> Result<(), CraftFailReason> {
        for ingredient in &recipe.inputs {
            let available = hotbar.count_item(ingredient.item_id);
            if available < ingredient.quantity as u16 {
                return Err(CraftFailReason::MissingIngredients);
            }
        }
        Ok(())
    }

    /// Validate that there's space in the hotbar for the recipe output.
    ///
    /// Returns Ok(()) if space exists, Err(HotbarFull) otherwise.
    pub fn validate_output_space(
        &self,
        recipe: &Recipe,
        hotbar: &Hotbar,
    ) -> Result<(), CraftFailReason> {
        let max_stack = get_max_stack(recipe.output_item);
        if hotbar.can_add(recipe.output_item, recipe.output_quantity, max_stack) {
            Ok(())
        } else {
            Err(CraftFailReason::HotbarFull)
        }
    }

    /// Apply a craft operation: consume inputs and add output.
    ///
    /// This should only be called after validation passes.
    /// Modifies the hotbar in place.
    ///
    /// # Panics
    /// Panics if consumption fails (should not happen after validation).
    pub fn apply_craft(&self, recipe: &Recipe, hotbar: &mut Hotbar) {
        // Consume all inputs
        for ingredient in &recipe.inputs {
            let consumed = hotbar.consume_items(ingredient.item_id, ingredient.quantity);
            debug_assert!(consumed, "Failed to consume validated ingredient");
        }

        // Add output
        let max_stack = get_max_stack(recipe.output_item);
        let remaining = hotbar.try_add_item(recipe.output_item, recipe.output_quantity, max_stack);
        debug_assert_eq!(remaining, 0, "Failed to add validated output");
    }

    /// Attempt to craft an item using the given recipe.
    ///
    /// Performs all validation, then atomically applies the craft if valid.
    /// Returns a CraftResult with success/failure details.
    ///
    /// # Arguments
    /// * `recipe_id` - The recipe to craft
    /// * `hotbar` - The player's hotbar (modified on success)
    /// * `is_alive` - Whether the player is alive
    /// * `crafting_enabled` - Whether crafting is enabled for this game mode
    pub fn try_craft(
        &self,
        recipe_id: &str,
        hotbar: &mut Hotbar,
        is_alive: bool,
        crafting_enabled: bool,
    ) -> CraftResult {
        // Check if player is alive
        if !is_alive {
            return CraftResult::failure(recipe_id, CraftFailReason::PlayerDead);
        }

        // Check if crafting is enabled
        if !crafting_enabled {
            return CraftResult::failure(recipe_id, CraftFailReason::CraftingDisabled);
        }

        // Look up recipe
        let recipe = match RECIPE_REGISTRY.get(recipe_id) {
            Some(r) => r,
            None => return CraftResult::failure(recipe_id, CraftFailReason::UnknownRecipe),
        };

        // Validate ingredients
        if let Err(reason) = self.validate_ingredients(recipe, hotbar) {
            return CraftResult::failure(recipe_id, reason);
        }

        // Validate output space
        if let Err(reason) = self.validate_output_space(recipe, hotbar) {
            return CraftResult::failure(recipe_id, reason);
        }

        // All validations passed - apply the craft atomically
        self.apply_craft(recipe, hotbar);

        CraftResult::success(recipe_id, recipe.output_item, recipe.output_quantity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::inventory::ItemStack;

    // ========================================================================
    // T018: Unit test - ingredient validation (sufficient/insufficient)
    // ========================================================================

    #[test]
    fn test_validate_ingredients_sufficient() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(9);

        // Add 5 scrap (need 2 for health pack)
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 5)));

        let recipe = RECIPE_REGISTRY.get("health_pack").unwrap();
        assert!(system.validate_ingredients(recipe, &hotbar).is_ok());
    }

    #[test]
    fn test_validate_ingredients_exact() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(9);

        // Add exactly 2 scrap for health pack
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 2)));

        let recipe = RECIPE_REGISTRY.get("health_pack").unwrap();
        assert!(system.validate_ingredients(recipe, &hotbar).is_ok());
    }

    #[test]
    fn test_validate_ingredients_insufficient() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(9);

        // Add only 1 scrap (need 2 for health pack)
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 1)));

        let recipe = RECIPE_REGISTRY.get("health_pack").unwrap();
        assert_eq!(
            system.validate_ingredients(recipe, &hotbar),
            Err(CraftFailReason::MissingIngredients)
        );
    }

    #[test]
    fn test_validate_ingredients_none() {
        let system = CraftSystem::new();
        let hotbar = Hotbar::new(9);

        let recipe = RECIPE_REGISTRY.get("health_pack").unwrap();
        assert_eq!(
            system.validate_ingredients(recipe, &hotbar),
            Err(CraftFailReason::MissingIngredients)
        );
    }

    #[test]
    fn test_validate_ingredients_across_slots() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(9);

        // Split 3 scrap across slots (need 3 for sword)
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 1)));
        hotbar.set(2, Some(ItemStack::new(ItemId::SCRAP, 2)));

        let recipe = RECIPE_REGISTRY.get("sword").unwrap();
        assert!(system.validate_ingredients(recipe, &hotbar).is_ok());
    }

    // ========================================================================
    // T019: Unit test - output space validation
    // ========================================================================

    #[test]
    fn test_validate_output_space_empty_hotbar() {
        let system = CraftSystem::new();
        let hotbar = Hotbar::new(9);

        let recipe = RECIPE_REGISTRY.get("health_pack").unwrap();
        assert!(system.validate_output_space(recipe, &hotbar).is_ok());
    }

    #[test]
    fn test_validate_output_space_partial_stack() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(2);

        // Fill first slot with partial health packs, second with sword
        hotbar.set(0, Some(ItemStack::new(ItemId::HEALTH_PACK, 10)));
        hotbar.set(1, Some(ItemStack::single(ItemId::SWORD)));

        let recipe = RECIPE_REGISTRY.get("health_pack").unwrap();
        // Can stack on existing health packs
        assert!(system.validate_output_space(recipe, &hotbar).is_ok());
    }

    #[test]
    fn test_validate_output_space_full() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(2);

        // Fill both slots completely
        hotbar.set(0, Some(ItemStack::new(ItemId::HEALTH_PACK, 16)));
        hotbar.set(1, Some(ItemStack::single(ItemId::SWORD)));

        let recipe = RECIPE_REGISTRY.get("health_pack").unwrap();
        // No space for more health packs
        assert_eq!(
            system.validate_output_space(recipe, &hotbar),
            Err(CraftFailReason::HotbarFull)
        );
    }

    #[test]
    fn test_validate_output_space_non_stackable() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(2);

        // Fill with swords (non-stackable)
        hotbar.set(0, Some(ItemStack::single(ItemId::SWORD)));
        hotbar.set(1, Some(ItemStack::single(ItemId::SWORD)));

        let recipe = RECIPE_REGISTRY.get("sword").unwrap();
        // No empty slots for another sword
        assert_eq!(
            system.validate_output_space(recipe, &hotbar),
            Err(CraftFailReason::HotbarFull)
        );
    }

    // ========================================================================
    // T020: Unit test - atomic craft success
    // ========================================================================

    #[test]
    fn test_apply_craft_health_pack() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(9);

        // Add 5 scrap
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 5)));

        let recipe = RECIPE_REGISTRY.get("health_pack").unwrap();
        system.apply_craft(recipe, &mut hotbar);

        // Should have consumed 2 scrap, leaving 3
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 3);
        // Should have 1 health pack
        assert_eq!(hotbar.count_item(ItemId::HEALTH_PACK), 1);
    }

    #[test]
    fn test_apply_craft_sword() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(9);

        // Add 4 scrap
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 4)));

        let recipe = RECIPE_REGISTRY.get("sword").unwrap();
        system.apply_craft(recipe, &mut hotbar);

        // Should have consumed 3 scrap, leaving 1
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 1);
        // Should have 1 sword
        assert_eq!(hotbar.count_item(ItemId::SWORD), 1);
    }

    #[test]
    fn test_apply_craft_stacks_output() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(9);

        // Add existing health pack and scrap
        hotbar.set(0, Some(ItemStack::new(ItemId::HEALTH_PACK, 5)));
        hotbar.set(1, Some(ItemStack::new(ItemId::SCRAP, 10)));

        let recipe = RECIPE_REGISTRY.get("health_pack").unwrap();
        system.apply_craft(recipe, &mut hotbar);

        // Should stack with existing
        assert_eq!(hotbar.count_item(ItemId::HEALTH_PACK), 6);
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 8);
    }

    // ========================================================================
    // T021: Unit test - atomic craft failure leaves hotbar unchanged
    // ========================================================================

    #[test]
    fn test_try_craft_failure_unchanged_ingredients() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(9);

        // Add only 1 scrap (need 2 for health pack)
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 1)));

        let result = system.try_craft("health_pack", &mut hotbar, true, true);

        assert!(!result.success);
        assert_eq!(
            result.fail_reason,
            Some(CraftFailReason::MissingIngredients)
        );
        // Hotbar unchanged
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 1);
        assert_eq!(hotbar.count_item(ItemId::HEALTH_PACK), 0);
    }

    #[test]
    fn test_try_craft_failure_unchanged_unknown() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(9);

        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 10)));

        let result = system.try_craft("nonexistent_recipe", &mut hotbar, true, true);

        assert!(!result.success);
        assert_eq!(result.fail_reason, Some(CraftFailReason::UnknownRecipe));
        // Hotbar unchanged
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 10);
    }

    #[test]
    fn test_try_craft_failure_unchanged_dead() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(9);

        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 10)));

        let result = system.try_craft("health_pack", &mut hotbar, false, true);

        assert!(!result.success);
        assert_eq!(result.fail_reason, Some(CraftFailReason::PlayerDead));
        // Hotbar unchanged
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 10);
    }

    #[test]
    fn test_try_craft_failure_unchanged_disabled() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(9);

        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 10)));

        let result = system.try_craft("health_pack", &mut hotbar, true, false);

        assert!(!result.success);
        assert_eq!(result.fail_reason, Some(CraftFailReason::CraftingDisabled));
        // Hotbar unchanged
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 10);
    }

    // ========================================================================
    // Additional try_craft tests
    // ========================================================================

    #[test]
    fn test_try_craft_success() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(9);

        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 5)));

        let result = system.try_craft("health_pack", &mut hotbar, true, true);

        assert!(result.success);
        assert_eq!(result.output_item, Some(ItemId::HEALTH_PACK));
        assert_eq!(result.output_quantity, Some(1));
        assert!(result.fail_reason.is_none());

        // Verify hotbar changed
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 3);
        assert_eq!(hotbar.count_item(ItemId::HEALTH_PACK), 1);
    }

    #[test]
    fn test_try_craft_all_recipes() {
        let system = CraftSystem::new();

        // Test health_pack (2 scrap -> 1 health pack)
        let mut hotbar = Hotbar::new(9);
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 2)));
        let result = system.try_craft("health_pack", &mut hotbar, true, true);
        assert!(result.success);
        assert_eq!(hotbar.count_item(ItemId::HEALTH_PACK), 1);

        // Test sword (3 scrap -> 1 sword)
        let mut hotbar = Hotbar::new(9);
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 3)));
        let result = system.try_craft("sword", &mut hotbar, true, true);
        assert!(result.success);
        assert_eq!(hotbar.count_item(ItemId::SWORD), 1);

        // Test bow (4 scrap -> 1 bow)
        let mut hotbar = Hotbar::new(9);
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 4)));
        let result = system.try_craft("bow", &mut hotbar, true, true);
        assert!(result.success);
        assert_eq!(hotbar.count_item(ItemId::BOW), 1);
    }

    #[test]
    fn test_try_craft_hotbar_full_rejection() {
        let system = CraftSystem::new();
        let mut hotbar = Hotbar::new(2);

        // Fill both slots (one with scrap, one with sword)
        hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 5)));
        hotbar.set(1, Some(ItemStack::single(ItemId::SWORD)));

        // Try to craft health pack - should fail (no space)
        let result = system.try_craft("health_pack", &mut hotbar, true, true);

        assert!(!result.success);
        assert_eq!(result.fail_reason, Some(CraftFailReason::HotbarFull));
        // Hotbar unchanged
        assert_eq!(hotbar.count_item(ItemId::SCRAP), 5);
    }
}
