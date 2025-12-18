//! Shop offer and registry definitions.

use plix_common::types::GameMode;
use plix_common::ItemId;
use serde::{Deserialize, Serialize};

/// A purchasable item configuration defined in arena config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopOffer {
    /// Unique identifier for the offer (e.g., "health_pack", "sword")
    pub offer_id: String,
    /// Item to deliver on purchase
    pub item_id: ItemId,
    /// Number of items per purchase
    pub quantity: u8,
    /// Cost in coins
    pub price: u32,
    /// Modes where offer is available (None = all modes)
    pub allowed_modes: Option<Vec<GameMode>>,
    /// Maximum purchases per player per match (None = unlimited)
    pub max_per_match: Option<u8>,
}

impl ShopOffer {
    /// Check if this offer is allowed in the given game mode.
    pub fn is_allowed_in_mode(&self, mode: GameMode) -> bool {
        self.allowed_modes
            .as_ref()
            .map_or(true, |modes| modes.contains(&mode))
    }

    /// Validate offer configuration.
    ///
    /// Returns true if valid, false if invalid (price=0, quantity=0, etc).
    pub fn is_valid(&self) -> bool {
        !self.offer_id.is_empty() && self.price > 0 && self.quantity > 0
    }
}

/// Collection of shop offers with lookup methods.
#[derive(Debug, Clone, Default)]
pub struct ShopRegistry {
    /// All configured shop offers
    offers: Vec<ShopOffer>,
}

impl ShopRegistry {
    /// Create a new registry from a list of offers.
    ///
    /// Filters out invalid offers (price=0, quantity=0, empty offer_id).
    pub fn new(offers: Vec<ShopOffer>) -> Self {
        let valid_offers: Vec<_> = offers.into_iter().filter(|o| o.is_valid()).collect();
        Self {
            offers: valid_offers,
        }
    }

    /// Get an offer by its ID.
    pub fn get(&self, offer_id: &str) -> Option<&ShopOffer> {
        self.offers.iter().find(|o| o.offer_id == offer_id)
    }

    /// List all offers available in the given game mode.
    pub fn list_for_mode(&self, mode: GameMode) -> Vec<&ShopOffer> {
        self.offers
            .iter()
            .filter(|o| o.is_allowed_in_mode(mode))
            .collect()
    }

    /// Check if registry has no offers.
    pub fn is_empty(&self) -> bool {
        self.offers.is_empty()
    }

    /// Get all offers.
    pub fn all(&self) -> &[ShopOffer] {
        &self.offers
    }
}

/// Default shop offers for v1.
pub fn default_shop_offers() -> Vec<ShopOffer> {
    vec![
        ShopOffer {
            offer_id: "health_pack".to_string(),
            item_id: ItemId::HEALTH_PACK,
            quantity: 1,
            price: 20,
            allowed_modes: None,
            max_per_match: Some(5),
        },
        ShopOffer {
            offer_id: "sword".to_string(),
            item_id: ItemId::SWORD,
            quantity: 1,
            price: 50,
            allowed_modes: None,
            max_per_match: None,
        },
        ShopOffer {
            offer_id: "bow".to_string(),
            item_id: ItemId::BOW,
            quantity: 1,
            price: 75,
            allowed_modes: None,
            max_per_match: None,
        },
        ShopOffer {
            offer_id: "scrap".to_string(),
            item_id: ItemId::SCRAP,
            quantity: 5,
            price: 10,
            allowed_modes: None,
            max_per_match: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shop_offer_validation() {
        let valid = ShopOffer {
            offer_id: "test".to_string(),
            item_id: ItemId::HEALTH_PACK,
            quantity: 1,
            price: 10,
            allowed_modes: None,
            max_per_match: None,
        };
        assert!(valid.is_valid());

        let invalid_price = ShopOffer {
            offer_id: "test".to_string(),
            item_id: ItemId::HEALTH_PACK,
            quantity: 1,
            price: 0,
            allowed_modes: None,
            max_per_match: None,
        };
        assert!(!invalid_price.is_valid());

        let invalid_quantity = ShopOffer {
            offer_id: "test".to_string(),
            item_id: ItemId::HEALTH_PACK,
            quantity: 0,
            price: 10,
            allowed_modes: None,
            max_per_match: None,
        };
        assert!(!invalid_quantity.is_valid());

        let empty_id = ShopOffer {
            offer_id: "".to_string(),
            item_id: ItemId::HEALTH_PACK,
            quantity: 1,
            price: 10,
            allowed_modes: None,
            max_per_match: None,
        };
        assert!(!empty_id.is_valid());
    }

    #[test]
    fn test_shop_offer_mode_restriction() {
        let unrestricted = ShopOffer {
            offer_id: "test".to_string(),
            item_id: ItemId::HEALTH_PACK,
            quantity: 1,
            price: 10,
            allowed_modes: None,
            max_per_match: None,
        };
        assert!(unrestricted.is_allowed_in_mode(GameMode::Ctf));
        assert!(unrestricted.is_allowed_in_mode(GameMode::BrLite));
        assert!(unrestricted.is_allowed_in_mode(GameMode::Tdm));

        let ctf_only = ShopOffer {
            offer_id: "test".to_string(),
            item_id: ItemId::HEALTH_PACK,
            quantity: 1,
            price: 10,
            allowed_modes: Some(vec![GameMode::Ctf]),
            max_per_match: None,
        };
        assert!(ctf_only.is_allowed_in_mode(GameMode::Ctf));
        assert!(!ctf_only.is_allowed_in_mode(GameMode::BrLite));
    }

    #[test]
    fn test_shop_registry_lookup() {
        let offers = vec![
            ShopOffer {
                offer_id: "health_pack".to_string(),
                item_id: ItemId::HEALTH_PACK,
                quantity: 1,
                price: 20,
                allowed_modes: None,
                max_per_match: None,
            },
            ShopOffer {
                offer_id: "sword".to_string(),
                item_id: ItemId::SWORD,
                quantity: 1,
                price: 50,
                allowed_modes: None,
                max_per_match: None,
            },
        ];
        let registry = ShopRegistry::new(offers);

        assert!(registry.get("health_pack").is_some());
        assert!(registry.get("sword").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_shop_registry_filters_invalid() {
        let offers = vec![
            ShopOffer {
                offer_id: "valid".to_string(),
                item_id: ItemId::HEALTH_PACK,
                quantity: 1,
                price: 10,
                allowed_modes: None,
                max_per_match: None,
            },
            ShopOffer {
                offer_id: "invalid".to_string(),
                item_id: ItemId::HEALTH_PACK,
                quantity: 0, // Invalid
                price: 10,
                allowed_modes: None,
                max_per_match: None,
            },
        ];
        let registry = ShopRegistry::new(offers);

        assert!(registry.get("valid").is_some());
        assert!(registry.get("invalid").is_none());
        assert_eq!(registry.all().len(), 1);
    }

    #[test]
    fn test_shop_registry_list_for_mode() {
        let offers = vec![
            ShopOffer {
                offer_id: "all".to_string(),
                item_id: ItemId::HEALTH_PACK,
                quantity: 1,
                price: 10,
                allowed_modes: None,
                max_per_match: None,
            },
            ShopOffer {
                offer_id: "ctf_only".to_string(),
                item_id: ItemId::SWORD,
                quantity: 1,
                price: 50,
                allowed_modes: Some(vec![GameMode::Ctf]),
                max_per_match: None,
            },
        ];
        let registry = ShopRegistry::new(offers);

        let ctf_offers = registry.list_for_mode(GameMode::Ctf);
        assert_eq!(ctf_offers.len(), 2);

        let br_offers = registry.list_for_mode(GameMode::BrLite);
        assert_eq!(br_offers.len(), 1);
        assert_eq!(br_offers[0].offer_id, "all");
    }
}
