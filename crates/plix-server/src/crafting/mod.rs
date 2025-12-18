//! Crafting system for item conversion.
//!
//! This module provides a minimalist server-authoritative crafting system that allows
//! players to convert resource items into useful equipment via simple recipes.
//!
//! Features:
//! - Recipe registry with static recipes
//! - Server-side validation (ingredients, output space, player state)
//! - Atomic craft operations (all-or-nothing)
//! - Per-player cooldown (1 second between successful crafts)
//! - Game mode configuration (enabled/disabled per mode)

pub mod config;
pub mod cooldown;
pub mod errors;
pub mod metrics;
pub mod recipe;
pub mod system;

pub use config::{get_craft_config, CraftConfig};
pub use cooldown::CraftCooldown;
pub use errors::CraftFailReason;
pub use metrics::CraftMetrics;
pub use recipe::{Ingredient, Recipe, RecipeId, RecipeRegistry, RECIPE_REGISTRY};
pub use system::{CraftResult, CraftSystem};
