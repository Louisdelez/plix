//! Crafting error types.

use plix_common::protocol::CraftRejectReason;
use serde::{Deserialize, Serialize};

/// Reason for a craft operation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CraftFailReason {
    /// Recipe ID not found in registry.
    UnknownRecipe,
    /// Crafting is disabled for the current game mode.
    CraftingDisabled,
    /// Recipe is not allowed in the current game mode.
    RecipeDisabled,
    /// Player is on cooldown from a recent successful craft.
    CooldownActive,
    /// Player doesn't have all required ingredients.
    MissingIngredients,
    /// No space in hotbar for the output item.
    HotbarFull,
    /// Player is dead and cannot craft.
    PlayerDead,
}

impl std::fmt::Display for CraftFailReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CraftFailReason::UnknownRecipe => write!(f, "Unknown recipe"),
            CraftFailReason::CraftingDisabled => write!(f, "Crafting not available"),
            CraftFailReason::RecipeDisabled => write!(f, "Recipe not available"),
            CraftFailReason::CooldownActive => write!(f, "Cooldown active"),
            CraftFailReason::MissingIngredients => write!(f, "Not enough materials"),
            CraftFailReason::HotbarFull => write!(f, "Inventory full"),
            CraftFailReason::PlayerDead => write!(f, "Cannot craft while dead"),
        }
    }
}

impl From<CraftFailReason> for CraftRejectReason {
    fn from(reason: CraftFailReason) -> Self {
        match reason {
            CraftFailReason::UnknownRecipe => CraftRejectReason::UnknownRecipe,
            CraftFailReason::CraftingDisabled => CraftRejectReason::CraftingDisabled,
            CraftFailReason::RecipeDisabled => CraftRejectReason::RecipeDisabled,
            CraftFailReason::CooldownActive => CraftRejectReason::CooldownActive,
            CraftFailReason::MissingIngredients => CraftRejectReason::MissingIngredients,
            CraftFailReason::HotbarFull => CraftRejectReason::HotbarFull,
            CraftFailReason::PlayerDead => CraftRejectReason::PlayerDead,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_craft_fail_reason_display() {
        assert_eq!(
            format!("{}", CraftFailReason::UnknownRecipe),
            "Unknown recipe"
        );
        assert_eq!(
            format!("{}", CraftFailReason::CraftingDisabled),
            "Crafting not available"
        );
        assert_eq!(
            format!("{}", CraftFailReason::MissingIngredients),
            "Not enough materials"
        );
        assert_eq!(format!("{}", CraftFailReason::HotbarFull), "Inventory full");
        assert_eq!(
            format!("{}", CraftFailReason::CooldownActive),
            "Cooldown active"
        );
        assert_eq!(
            format!("{}", CraftFailReason::PlayerDead),
            "Cannot craft while dead"
        );
    }

    #[test]
    fn test_craft_fail_reason_serde() {
        let reason = CraftFailReason::MissingIngredients;
        let json = serde_json::to_string(&reason).unwrap();
        let parsed: CraftFailReason = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, reason);
    }
}
