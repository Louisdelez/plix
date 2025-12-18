//! Purchase system for validating and executing shop purchases.

use plix_common::economy::PurchaseRejectReason;
use plix_common::inventory::Hotbar;
use plix_common::ItemId;
use tracing::{debug, info};

use super::config::EconomyConfig;
use super::shop::ShopRegistry;
use super::wallet::PlayerWallet;
use crate::inventory::item_registry::get_max_stack;

/// Result of a purchase attempt.
#[derive(Debug, Clone)]
pub struct PurchaseResult {
    /// Whether purchase succeeded
    pub success: bool,
    /// Offer ID that was attempted
    pub offer_id: String,
    /// Item received (if successful)
    pub item_id: Option<ItemId>,
    /// Quantity received (if successful)
    pub quantity: Option<u8>,
    /// Failure reason (if failed)
    pub fail_reason: Option<PurchaseRejectReason>,
    /// New balance after purchase
    pub new_balance: u32,
}

impl PurchaseResult {
    fn success(offer_id: &str, item_id: ItemId, quantity: u8, new_balance: u32) -> Self {
        Self {
            success: true,
            offer_id: offer_id.to_string(),
            item_id: Some(item_id),
            quantity: Some(quantity),
            fail_reason: None,
            new_balance,
        }
    }

    fn failure(offer_id: &str, reason: PurchaseRejectReason, current_balance: u32) -> Self {
        Self {
            success: false,
            offer_id: offer_id.to_string(),
            item_id: None,
            quantity: None,
            fail_reason: Some(reason),
            new_balance: current_balance,
        }
    }
}

/// Attempt to purchase an item from the shop.
///
/// Performs full validation chain:
/// 1. Check economy enabled
/// 2. Check player alive
/// 3. Look up offer
/// 4. Check balance >= price
/// 5. Check hotbar space
/// 6. Check purchase limit (max_per_match)
///
/// If all checks pass, atomically:
/// - Deduct coins from wallet
/// - Add item to hotbar
/// - Record purchase count
///
/// Returns PurchaseResult with success/failure and details.
pub fn try_purchase(
    offer_id: &str,
    wallet: &mut PlayerWallet,
    hotbar: &mut Hotbar,
    shop: &ShopRegistry,
    config: &EconomyConfig,
    is_alive: bool,
    player_id: u16,
) -> PurchaseResult {
    // 1. Check economy enabled
    if !config.enabled {
        debug!(player_id, offer_id, "Purchase rejected: economy disabled");
        return PurchaseResult::failure(
            offer_id,
            PurchaseRejectReason::EconomyDisabled,
            wallet.get_balance(),
        );
    }

    // 2. Check player alive
    if !is_alive {
        debug!(player_id, offer_id, "Purchase rejected: player dead");
        return PurchaseResult::failure(
            offer_id,
            PurchaseRejectReason::PlayerDead,
            wallet.get_balance(),
        );
    }

    // 3. Look up offer
    let offer = match shop.get(offer_id) {
        Some(o) => o,
        None => {
            debug!(player_id, offer_id, "Purchase rejected: unknown offer");
            return PurchaseResult::failure(
                offer_id,
                PurchaseRejectReason::UnknownOffer,
                wallet.get_balance(),
            );
        }
    };

    // 4. Check balance
    if wallet.get_balance() < offer.price {
        debug!(
            player_id,
            offer_id,
            balance = wallet.get_balance(),
            price = offer.price,
            "Purchase rejected: insufficient balance"
        );
        return PurchaseResult::failure(
            offer_id,
            PurchaseRejectReason::InsufficientBalance,
            wallet.get_balance(),
        );
    }

    // 5. Check hotbar space
    let max_stack = get_max_stack(offer.item_id);
    if !hotbar.can_add(offer.item_id, offer.quantity, max_stack) {
        debug!(player_id, offer_id, "Purchase rejected: hotbar full");
        return PurchaseResult::failure(
            offer_id,
            PurchaseRejectReason::HotbarFull,
            wallet.get_balance(),
        );
    }

    // 6. Check purchase limit
    if let Some(max) = offer.max_per_match {
        if wallet.get_purchase_count(offer_id) >= max {
            debug!(
                player_id,
                offer_id,
                count = wallet.get_purchase_count(offer_id),
                max,
                "Purchase rejected: purchase limit reached"
            );
            return PurchaseResult::failure(
                offer_id,
                PurchaseRejectReason::PurchaseLimitReached,
                wallet.get_balance(),
            );
        }
    }

    // All checks passed - apply atomically
    wallet.try_spend(offer.price);
    hotbar.try_add_item(offer.item_id, offer.quantity, max_stack);
    wallet.record_purchase(offer_id);

    info!(
        player_id,
        offer_id,
        price = offer.price,
        new_balance = wallet.get_balance(),
        "Purchase successful"
    );

    PurchaseResult::success(
        offer_id,
        offer.item_id,
        offer.quantity,
        wallet.get_balance(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::shop::ShopOffer;

    fn test_config() -> EconomyConfig {
        EconomyConfig {
            enabled: true,
            kill_reward: 10,
            ctf_capture_reward: 25,
            br_placement_rewards: [50, 30, 15],
            shop_offers: vec![],
        }
    }

    fn test_registry() -> ShopRegistry {
        ShopRegistry::new(vec![
            ShopOffer {
                offer_id: "health_pack".to_string(),
                item_id: ItemId::HEALTH_PACK,
                quantity: 1,
                price: 20,
                allowed_modes: None,
                max_per_match: Some(3),
            },
            ShopOffer {
                offer_id: "sword".to_string(),
                item_id: ItemId::SWORD,
                quantity: 1,
                price: 50,
                allowed_modes: None,
                max_per_match: None,
            },
        ])
    }

    #[test]
    fn test_successful_purchase() {
        let config = test_config();
        let registry = test_registry();
        let mut wallet = PlayerWallet::new();
        wallet.add_coins(100);
        let mut hotbar = Hotbar::with_default_size();

        let result = try_purchase(
            "health_pack",
            &mut wallet,
            &mut hotbar,
            &registry,
            &config,
            true,
            1,
        );

        assert!(result.success);
        assert_eq!(result.offer_id, "health_pack");
        assert_eq!(result.item_id, Some(ItemId::HEALTH_PACK));
        assert_eq!(result.quantity, Some(1));
        assert_eq!(result.new_balance, 80);
        assert_eq!(wallet.get_balance(), 80);
        assert_eq!(wallet.get_purchase_count("health_pack"), 1);
    }

    #[test]
    fn test_economy_disabled() {
        let mut config = test_config();
        config.enabled = false;
        let registry = test_registry();
        let mut wallet = PlayerWallet::new();
        wallet.add_coins(100);
        let mut hotbar = Hotbar::with_default_size();

        let result = try_purchase(
            "health_pack",
            &mut wallet,
            &mut hotbar,
            &registry,
            &config,
            true,
            1,
        );

        assert!(!result.success);
        assert_eq!(
            result.fail_reason,
            Some(PurchaseRejectReason::EconomyDisabled)
        );
        assert_eq!(wallet.get_balance(), 100); // Unchanged
    }

    #[test]
    fn test_player_dead() {
        let config = test_config();
        let registry = test_registry();
        let mut wallet = PlayerWallet::new();
        wallet.add_coins(100);
        let mut hotbar = Hotbar::with_default_size();

        let result = try_purchase(
            "health_pack",
            &mut wallet,
            &mut hotbar,
            &registry,
            &config,
            false,
            1,
        );

        assert!(!result.success);
        assert_eq!(result.fail_reason, Some(PurchaseRejectReason::PlayerDead));
    }

    #[test]
    fn test_unknown_offer() {
        let config = test_config();
        let registry = test_registry();
        let mut wallet = PlayerWallet::new();
        wallet.add_coins(100);
        let mut hotbar = Hotbar::with_default_size();

        let result = try_purchase(
            "unknown",
            &mut wallet,
            &mut hotbar,
            &registry,
            &config,
            true,
            1,
        );

        assert!(!result.success);
        assert_eq!(result.fail_reason, Some(PurchaseRejectReason::UnknownOffer));
    }

    #[test]
    fn test_insufficient_balance() {
        let config = test_config();
        let registry = test_registry();
        let mut wallet = PlayerWallet::new();
        wallet.add_coins(10); // Not enough for health_pack (20)
        let mut hotbar = Hotbar::with_default_size();

        let result = try_purchase(
            "health_pack",
            &mut wallet,
            &mut hotbar,
            &registry,
            &config,
            true,
            1,
        );

        assert!(!result.success);
        assert_eq!(
            result.fail_reason,
            Some(PurchaseRejectReason::InsufficientBalance)
        );
        assert_eq!(wallet.get_balance(), 10); // Unchanged
    }

    #[test]
    fn test_purchase_limit_reached() {
        let config = test_config();
        let registry = test_registry();
        let mut wallet = PlayerWallet::new();
        wallet.add_coins(1000);
        let mut hotbar = Hotbar::with_default_size();

        // Buy 3 health packs (max_per_match = 3)
        for _ in 0..3 {
            let result = try_purchase(
                "health_pack",
                &mut wallet,
                &mut hotbar,
                &registry,
                &config,
                true,
                1,
            );
            assert!(result.success);
        }

        // 4th should fail
        let result = try_purchase(
            "health_pack",
            &mut wallet,
            &mut hotbar,
            &registry,
            &config,
            true,
            1,
        );
        assert!(!result.success);
        assert_eq!(
            result.fail_reason,
            Some(PurchaseRejectReason::PurchaseLimitReached)
        );
    }

    #[test]
    fn test_hotbar_full() {
        let config = test_config();
        let registry = test_registry();
        let mut wallet = PlayerWallet::new();
        wallet.add_coins(1000);
        let mut hotbar = Hotbar::with_default_size();

        // Fill hotbar completely with swords (no stacking, max_stack=1)
        for _ in 0..9 {
            hotbar.try_add_item(ItemId::SWORD, 1, 1);
        }

        // Next purchase should fail
        let result = try_purchase(
            "sword",
            &mut wallet,
            &mut hotbar,
            &registry,
            &config,
            true,
            1,
        );
        assert!(!result.success);
        assert_eq!(result.fail_reason, Some(PurchaseRejectReason::HotbarFull));
    }

    #[test]
    fn test_atomicity_on_failure() {
        let config = test_config();
        let registry = test_registry();
        let mut wallet = PlayerWallet::new();
        wallet.add_coins(10); // Not enough
        let mut hotbar = Hotbar::with_default_size();

        let initial_balance = wallet.get_balance();
        let result = try_purchase(
            "health_pack",
            &mut wallet,
            &mut hotbar,
            &registry,
            &config,
            true,
            1,
        );

        assert!(!result.success);
        // Verify nothing changed
        assert_eq!(wallet.get_balance(), initial_balance);
        assert_eq!(wallet.get_purchase_count("health_pack"), 0);
    }
}
