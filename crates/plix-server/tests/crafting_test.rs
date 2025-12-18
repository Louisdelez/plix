//! Integration tests for the crafting system (Feature 023).
//!
//! Tests cover:
//! - T083: Training mode full craft flow
//! - T084: BR Lite craft with looted SCRAP
//! - T085: TDM mode crafting disabled
//! - T086: Cooldown enforcement
//! - T087: All 3 recipes work correctly

use plix_common::inventory::{Hotbar, ItemStack};
use plix_common::time::Tick;
use plix_common::types::ItemId;
use plix_common::GameMode;

use plix_server::crafting::{
    get_craft_config, CraftCooldown, CraftMetrics, CraftSystem, RECIPE_REGISTRY,
};

// ============================================================================
// T083: Integration test - Training mode full craft flow
// ============================================================================

#[test]
fn test_training_mode_full_craft_flow() {
    let config = get_craft_config(GameMode::Training);
    assert!(
        config.enabled,
        "Crafting should be enabled in Training mode"
    );

    let system = CraftSystem::new();
    let mut hotbar = Hotbar::new(9);

    // Simulate Training loadout: give player 5 SCRAP
    hotbar.set(3, Some(ItemStack::new(ItemId::SCRAP, 5)));

    // Verify starting state
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 5);
    assert_eq!(hotbar.count_item(ItemId::HEALTH_PACK), 0);

    // Craft health_pack (costs 2 SCRAP)
    let result = system.try_craft("health_pack", &mut hotbar, true, config.enabled);
    assert!(result.success, "Craft should succeed in Training mode");
    assert_eq!(result.output_item, Some(ItemId::HEALTH_PACK));
    assert_eq!(result.output_quantity, Some(1));

    // Verify hotbar updated
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 3);
    assert_eq!(hotbar.count_item(ItemId::HEALTH_PACK), 1);

    // Craft another health_pack
    let result = system.try_craft("health_pack", &mut hotbar, true, config.enabled);
    assert!(result.success);
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 1);
    assert_eq!(hotbar.count_item(ItemId::HEALTH_PACK), 2);

    // Try to craft with insufficient SCRAP
    let result = system.try_craft("health_pack", &mut hotbar, true, config.enabled);
    assert!(!result.success);
    assert_eq!(
        result.fail_reason,
        Some(plix_server::crafting::CraftFailReason::MissingIngredients)
    );
    // Hotbar unchanged
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 1);
    assert_eq!(hotbar.count_item(ItemId::HEALTH_PACK), 2);
}

// ============================================================================
// T084: Integration test - BR Lite craft with looted SCRAP
// ============================================================================

#[test]
fn test_br_lite_craft_with_looted_scrap() {
    let config = get_craft_config(GameMode::BrLite);
    assert!(config.enabled, "Crafting should be enabled in BR Lite mode");

    let system = CraftSystem::new();
    let mut hotbar = Hotbar::new(9);

    // BR Lite starts empty - simulate picking up SCRAP from loot
    assert!(hotbar.get(0).is_none(), "BR Lite starts with empty hotbar");

    // Simulate looting: add 10 SCRAP (as if picked up from loot drops)
    hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 10)));

    // Craft a sword (costs 3 SCRAP)
    let result = system.try_craft("sword", &mut hotbar, true, config.enabled);
    assert!(result.success);
    assert_eq!(result.output_item, Some(ItemId::SWORD));
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 7);
    assert_eq!(hotbar.count_item(ItemId::SWORD), 1);

    // Craft a bow (costs 4 SCRAP)
    let result = system.try_craft("bow", &mut hotbar, true, config.enabled);
    assert!(result.success);
    assert_eq!(result.output_item, Some(ItemId::BOW));
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 3);
    assert_eq!(hotbar.count_item(ItemId::BOW), 1);

    // Craft a health_pack (costs 2 SCRAP)
    let result = system.try_craft("health_pack", &mut hotbar, true, config.enabled);
    assert!(result.success);
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 1);
    assert_eq!(hotbar.count_item(ItemId::HEALTH_PACK), 1);
}

// ============================================================================
// T085: Integration test - TDM mode crafting disabled
// ============================================================================

#[test]
fn test_tdm_mode_crafting_disabled() {
    let config = get_craft_config(GameMode::Tdm);
    assert!(!config.enabled, "Crafting should be disabled in TDM mode");

    let system = CraftSystem::new();
    let mut hotbar = Hotbar::new(9);

    // Give player plenty of SCRAP
    hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 20)));

    // Try all recipes - all should fail with CraftingDisabled
    for recipe_id in ["health_pack", "sword", "bow"] {
        let result = system.try_craft(recipe_id, &mut hotbar, true, config.enabled);
        assert!(
            !result.success,
            "Craft {} should fail in TDM mode",
            recipe_id
        );
        assert_eq!(
            result.fail_reason,
            Some(plix_server::crafting::CraftFailReason::CraftingDisabled),
            "Craft {} should fail with CraftingDisabled",
            recipe_id
        );
    }

    // Verify hotbar unchanged
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 20);
    assert_eq!(hotbar.count_item(ItemId::HEALTH_PACK), 0);
    assert_eq!(hotbar.count_item(ItemId::SWORD), 0);
    assert_eq!(hotbar.count_item(ItemId::BOW), 0);
}

#[test]
fn test_ffa_mode_crafting_disabled() {
    let config = get_craft_config(GameMode::Ffa);
    assert!(!config.enabled, "Crafting should be disabled in FFA mode");
}

#[test]
fn test_ctf_mode_crafting_disabled() {
    let config = get_craft_config(GameMode::Ctf);
    assert!(!config.enabled, "Crafting should be disabled in CTF mode");
}

// ============================================================================
// T086: Integration test - cooldown enforcement
// ============================================================================

#[test]
fn test_cooldown_enforcement() {
    let config = get_craft_config(GameMode::Training);
    let cooldown_ticks = config.cooldown_ticks;
    assert_eq!(cooldown_ticks, 60, "Cooldown should be 60 ticks (1 second)");

    let mut cooldown = CraftCooldown::new();

    // Initially not on cooldown
    assert!(
        !cooldown.is_on_cooldown(Tick(0), cooldown_ticks),
        "Should not be on cooldown initially"
    );

    // Record a craft at tick 100
    cooldown.record_craft(Tick(100));

    // Should be on cooldown immediately after
    assert!(
        cooldown.is_on_cooldown(Tick(100), cooldown_ticks),
        "Should be on cooldown at tick 100"
    );
    assert!(
        cooldown.is_on_cooldown(Tick(159), cooldown_ticks),
        "Should be on cooldown at tick 159"
    );

    // Should be off cooldown at tick 160 (100 + 60)
    assert!(
        !cooldown.is_on_cooldown(Tick(160), cooldown_ticks),
        "Should be off cooldown at tick 160"
    );

    // Test remaining ticks
    assert_eq!(cooldown.remaining_ticks(Tick(100), cooldown_ticks), 60);
    assert_eq!(cooldown.remaining_ticks(Tick(130), cooldown_ticks), 30);
    assert_eq!(cooldown.remaining_ticks(Tick(160), cooldown_ticks), 0);
}

#[test]
fn test_cooldown_only_on_successful_craft() {
    let system = CraftSystem::new();
    let mut cooldown = CraftCooldown::new();
    let config = get_craft_config(GameMode::Training);

    let mut hotbar = Hotbar::new(9);
    hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 1))); // Not enough for health_pack

    // Try craft (should fail due to insufficient ingredients)
    let result = system.try_craft("health_pack", &mut hotbar, true, config.enabled);
    assert!(!result.success);

    // Don't record cooldown for failed craft
    // (in real server, this is handled by checking result.success)
    assert!(
        !cooldown.is_on_cooldown(Tick(0), config.cooldown_ticks),
        "Failed craft should not trigger cooldown"
    );

    // Now give enough SCRAP and craft successfully
    hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 5)));
    let result = system.try_craft("health_pack", &mut hotbar, true, config.enabled);
    assert!(result.success);

    // Record cooldown for successful craft
    cooldown.record_craft(Tick(100));
    assert!(
        cooldown.is_on_cooldown(Tick(100), config.cooldown_ticks),
        "Successful craft should trigger cooldown"
    );
}

// ============================================================================
// T087: Integration test - all 3 recipes work correctly
// ============================================================================

#[test]
fn test_all_recipes_work() {
    // Verify all 3 recipes exist in registry
    assert!(
        RECIPE_REGISTRY.get("health_pack").is_some(),
        "health_pack recipe should exist"
    );
    assert!(
        RECIPE_REGISTRY.get("sword").is_some(),
        "sword recipe should exist"
    );
    assert!(
        RECIPE_REGISTRY.get("bow").is_some(),
        "bow recipe should exist"
    );
}

#[test]
fn test_health_pack_recipe() {
    let system = CraftSystem::new();
    let mut hotbar = Hotbar::new(9);
    hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 2)));

    let result = system.try_craft("health_pack", &mut hotbar, true, true);

    assert!(result.success);
    assert_eq!(result.recipe_id, "health_pack");
    assert_eq!(result.output_item, Some(ItemId::HEALTH_PACK));
    assert_eq!(result.output_quantity, Some(1));
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 0);
    assert_eq!(hotbar.count_item(ItemId::HEALTH_PACK), 1);
}

#[test]
fn test_sword_recipe() {
    let system = CraftSystem::new();
    let mut hotbar = Hotbar::new(9);
    hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 3)));

    let result = system.try_craft("sword", &mut hotbar, true, true);

    assert!(result.success);
    assert_eq!(result.recipe_id, "sword");
    assert_eq!(result.output_item, Some(ItemId::SWORD));
    assert_eq!(result.output_quantity, Some(1));
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 0);
    assert_eq!(hotbar.count_item(ItemId::SWORD), 1);
}

#[test]
fn test_bow_recipe() {
    let system = CraftSystem::new();
    let mut hotbar = Hotbar::new(9);
    hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 4)));

    let result = system.try_craft("bow", &mut hotbar, true, true);

    assert!(result.success);
    assert_eq!(result.recipe_id, "bow");
    assert_eq!(result.output_item, Some(ItemId::BOW));
    assert_eq!(result.output_quantity, Some(1));
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 0);
    assert_eq!(hotbar.count_item(ItemId::BOW), 1);
}

// ============================================================================
// Additional integration tests
// ============================================================================

#[test]
fn test_metrics_tracking() {
    let mut metrics = CraftMetrics::new();

    // Record some crafts
    metrics.record_success("health_pack");
    metrics.record_success("health_pack");
    metrics.record_success("sword");
    metrics.record_failure("bow", "MissingIngredients");

    assert_eq!(metrics.total_attempts, 4);
    assert_eq!(metrics.successes, 3);
    assert!((metrics.success_rate() - 0.75).abs() < 0.001);
    assert_eq!(metrics.crafts_per_recipe.get("health_pack"), Some(&2));
    assert_eq!(metrics.crafts_per_recipe.get("sword"), Some(&1));
    assert_eq!(
        metrics.failures_by_reason.get("MissingIngredients"),
        Some(&1)
    );
}

#[test]
fn test_dead_player_cannot_craft() {
    let system = CraftSystem::new();
    let mut hotbar = Hotbar::new(9);
    hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 10)));

    // Try craft while dead
    let result = system.try_craft("health_pack", &mut hotbar, false, true);

    assert!(!result.success);
    assert_eq!(
        result.fail_reason,
        Some(plix_server::crafting::CraftFailReason::PlayerDead)
    );
    // Hotbar unchanged
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 10);
}

#[test]
fn test_unknown_recipe() {
    let system = CraftSystem::new();
    let mut hotbar = Hotbar::new(9);
    hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 10)));

    let result = system.try_craft("nonexistent_recipe", &mut hotbar, true, true);

    assert!(!result.success);
    assert_eq!(
        result.fail_reason,
        Some(plix_server::crafting::CraftFailReason::UnknownRecipe)
    );
    // Hotbar unchanged
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 10);
}

#[test]
fn test_multi_slot_ingredient_consumption() {
    let system = CraftSystem::new();
    let mut hotbar = Hotbar::new(9);

    // Split 4 SCRAP across multiple slots
    hotbar.set(0, Some(ItemStack::new(ItemId::SCRAP, 1)));
    hotbar.set(3, Some(ItemStack::new(ItemId::SCRAP, 2)));
    hotbar.set(7, Some(ItemStack::new(ItemId::SCRAP, 1)));

    // Craft bow (requires 4 SCRAP)
    let result = system.try_craft("bow", &mut hotbar, true, true);

    assert!(result.success);
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 0);
    assert_eq!(hotbar.count_item(ItemId::BOW), 1);
}

#[test]
fn test_output_stacking() {
    let system = CraftSystem::new();
    let mut hotbar = Hotbar::new(9);

    // Start with some health packs and SCRAP
    hotbar.set(0, Some(ItemStack::new(ItemId::HEALTH_PACK, 5)));
    hotbar.set(1, Some(ItemStack::new(ItemId::SCRAP, 10)));

    // Craft more health packs
    let result = system.try_craft("health_pack", &mut hotbar, true, true);
    assert!(result.success);

    // Should stack with existing
    assert_eq!(hotbar.count_item(ItemId::HEALTH_PACK), 6);
    assert_eq!(hotbar.count_item(ItemId::SCRAP), 8);
}
