//! Recipe definitions and registry for the crafting system.

use once_cell::sync::Lazy;
use plix_common::inventory::ItemId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a crafting recipe.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecipeId(pub String);

impl RecipeId {
    /// Create a new recipe ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Recipe ID for health pack crafting.
    pub const HEALTH_PACK: &'static str = "health_pack";
    /// Recipe ID for sword crafting.
    pub const SWORD: &'static str = "sword";
    /// Recipe ID for bow crafting.
    pub const BOW: &'static str = "bow";
}

impl std::fmt::Display for RecipeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for RecipeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for RecipeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A required ingredient for a recipe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ingredient {
    /// Item type required.
    pub item_id: ItemId,
    /// Quantity required.
    pub quantity: u8,
}

impl Ingredient {
    /// Create a new ingredient.
    pub const fn new(item_id: ItemId, quantity: u8) -> Self {
        Self { item_id, quantity }
    }
}

/// A crafting recipe definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Unique identifier for this recipe.
    pub id: RecipeId,
    /// Required ingredients (item_id, quantity pairs).
    pub inputs: Vec<Ingredient>,
    /// Output item type.
    pub output_item: ItemId,
    /// Output quantity.
    pub output_quantity: u8,
}

impl Recipe {
    /// Create a new recipe.
    ///
    /// # Panics
    /// Panics if inputs is empty or output_quantity is 0.
    pub fn new(
        id: impl Into<RecipeId>,
        inputs: Vec<Ingredient>,
        output_item: ItemId,
        output_quantity: u8,
    ) -> Self {
        assert!(!inputs.is_empty(), "Recipe must have at least one input");
        assert!(output_quantity > 0, "Output quantity must be > 0");

        Self {
            id: id.into(),
            inputs,
            output_item,
            output_quantity,
        }
    }

    /// Validate recipe invariants.
    pub fn is_valid(&self) -> bool {
        !self.inputs.is_empty()
            && self.output_quantity > 0
            && self.inputs.iter().all(|i| i.quantity > 0)
    }
}

/// Registry of all available crafting recipes.
#[derive(Debug)]
pub struct RecipeRegistry {
    recipes: HashMap<String, Recipe>,
}

impl RecipeRegistry {
    /// Create a new recipe registry from a list of recipes.
    pub fn new(recipes: Vec<Recipe>) -> Self {
        let mut map = HashMap::new();
        for recipe in recipes {
            map.insert(recipe.id.0.clone(), recipe);
        }
        Self { recipes: map }
    }

    /// Get a recipe by ID.
    pub fn get(&self, id: &str) -> Option<&Recipe> {
        self.recipes.get(id)
    }

    /// Check if a recipe exists.
    pub fn exists(&self, id: &str) -> bool {
        self.recipes.contains_key(id)
    }

    /// Get all recipes.
    pub fn all(&self) -> impl Iterator<Item = &Recipe> {
        self.recipes.values()
    }

    /// Get the number of recipes.
    pub fn len(&self) -> usize {
        self.recipes.len()
    }

    /// Check if registry is empty.
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }
}

/// Static recipe registry with all v1 recipes.
///
/// v1 Recipes:
/// - health_pack: 2x SCRAP → 1x HEALTH_PACK
/// - sword: 3x SCRAP → 1x SWORD
/// - bow: 4x SCRAP → 1x BOW
pub static RECIPE_REGISTRY: Lazy<RecipeRegistry> = Lazy::new(|| {
    RecipeRegistry::new(vec![
        Recipe::new(
            RecipeId::HEALTH_PACK,
            vec![Ingredient::new(ItemId::SCRAP, 2)],
            ItemId::HEALTH_PACK,
            1,
        ),
        Recipe::new(
            RecipeId::SWORD,
            vec![Ingredient::new(ItemId::SCRAP, 3)],
            ItemId::SWORD,
            1,
        ),
        Recipe::new(
            RecipeId::BOW,
            vec![Ingredient::new(ItemId::SCRAP, 4)],
            ItemId::BOW,
            1,
        ),
    ])
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_id_creation() {
        let id = RecipeId::new("test_recipe");
        assert_eq!(id.0, "test_recipe");
    }

    #[test]
    fn test_recipe_id_from_str() {
        let id: RecipeId = "test".into();
        assert_eq!(id.0, "test");
    }

    #[test]
    fn test_ingredient_creation() {
        let ingredient = Ingredient::new(ItemId::SCRAP, 5);
        assert_eq!(ingredient.item_id, ItemId::SCRAP);
        assert_eq!(ingredient.quantity, 5);
    }

    #[test]
    fn test_recipe_creation() {
        let recipe = Recipe::new(
            "test",
            vec![Ingredient::new(ItemId::SCRAP, 2)],
            ItemId::HEALTH_PACK,
            1,
        );
        assert_eq!(recipe.id.0, "test");
        assert_eq!(recipe.inputs.len(), 1);
        assert_eq!(recipe.output_item, ItemId::HEALTH_PACK);
        assert_eq!(recipe.output_quantity, 1);
        assert!(recipe.is_valid());
    }

    #[test]
    #[should_panic(expected = "Recipe must have at least one input")]
    fn test_recipe_empty_inputs_panics() {
        Recipe::new("bad", vec![], ItemId::HEALTH_PACK, 1);
    }

    #[test]
    #[should_panic(expected = "Output quantity must be > 0")]
    fn test_recipe_zero_output_panics() {
        Recipe::new(
            "bad",
            vec![Ingredient::new(ItemId::SCRAP, 1)],
            ItemId::HEALTH_PACK,
            0,
        );
    }

    #[test]
    fn test_registry_get_existing() {
        assert!(RECIPE_REGISTRY.get(RecipeId::HEALTH_PACK).is_some());
        assert!(RECIPE_REGISTRY.get(RecipeId::SWORD).is_some());
        assert!(RECIPE_REGISTRY.get(RecipeId::BOW).is_some());
    }

    #[test]
    fn test_registry_get_nonexistent() {
        assert!(RECIPE_REGISTRY.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_exists() {
        assert!(RECIPE_REGISTRY.exists(RecipeId::HEALTH_PACK));
        assert!(!RECIPE_REGISTRY.exists("nonexistent"));
    }

    #[test]
    fn test_registry_has_3_recipes() {
        assert_eq!(RECIPE_REGISTRY.len(), 3);
    }

    #[test]
    fn test_health_pack_recipe() {
        let recipe = RECIPE_REGISTRY.get(RecipeId::HEALTH_PACK).unwrap();
        assert_eq!(recipe.inputs.len(), 1);
        assert_eq!(recipe.inputs[0].item_id, ItemId::SCRAP);
        assert_eq!(recipe.inputs[0].quantity, 2);
        assert_eq!(recipe.output_item, ItemId::HEALTH_PACK);
        assert_eq!(recipe.output_quantity, 1);
    }

    #[test]
    fn test_sword_recipe() {
        let recipe = RECIPE_REGISTRY.get(RecipeId::SWORD).unwrap();
        assert_eq!(recipe.inputs[0].quantity, 3);
        assert_eq!(recipe.output_item, ItemId::SWORD);
    }

    #[test]
    fn test_bow_recipe() {
        let recipe = RECIPE_REGISTRY.get(RecipeId::BOW).unwrap();
        assert_eq!(recipe.inputs[0].quantity, 4);
        assert_eq!(recipe.output_item, ItemId::BOW);
    }
}
