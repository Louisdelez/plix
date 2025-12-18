//! Player wallet for per-match coin balance tracking.

use std::collections::HashMap;

/// Per-player transient economy state for current match.
///
/// Tracks coin balance and purchase counts for the duration of a match.
/// Reset to zero at match start.
#[derive(Debug, Clone, Default)]
pub struct PlayerWallet {
    /// Current coin balance (0 to u32::MAX, saturating)
    balance: u32,
    /// Count of purchases per offer_id this match
    purchases: HashMap<String, u8>,
}

impl PlayerWallet {
    /// Create a new empty wallet with 0 balance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get current coin balance.
    #[inline]
    pub fn get_balance(&self) -> u32 {
        self.balance
    }

    /// Add coins to balance (saturating at u32::MAX).
    #[inline]
    pub fn add_coins(&mut self, amount: u32) {
        self.balance = self.balance.saturating_add(amount);
    }

    /// Attempt to spend coins. Returns true if successful, false if insufficient balance.
    ///
    /// If successful, balance is reduced by amount. If insufficient, balance is unchanged.
    #[inline]
    pub fn try_spend(&mut self, amount: u32) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            true
        } else {
            false
        }
    }

    /// Get purchase count for a specific offer_id this match.
    #[inline]
    pub fn get_purchase_count(&self, offer_id: &str) -> u8 {
        *self.purchases.get(offer_id).unwrap_or(&0)
    }

    /// Record a purchase for an offer_id (increment counter).
    #[inline]
    pub fn record_purchase(&mut self, offer_id: &str) {
        *self.purchases.entry(offer_id.to_string()).or_insert(0) += 1;
    }

    /// Reset wallet to initial state (balance = 0, clear purchases).
    ///
    /// Called at match start to ensure fair play.
    pub fn reset(&mut self) {
        self.balance = 0;
        self.purchases.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wallet_has_zero_balance() {
        let wallet = PlayerWallet::new();
        assert_eq!(wallet.get_balance(), 0);
    }

    #[test]
    fn test_add_coins() {
        let mut wallet = PlayerWallet::new();
        wallet.add_coins(100);
        assert_eq!(wallet.get_balance(), 100);
        wallet.add_coins(50);
        assert_eq!(wallet.get_balance(), 150);
    }

    #[test]
    fn test_add_coins_saturates() {
        let mut wallet = PlayerWallet::new();
        wallet.add_coins(u32::MAX);
        wallet.add_coins(1);
        assert_eq!(wallet.get_balance(), u32::MAX);
    }

    #[test]
    fn test_try_spend_success() {
        let mut wallet = PlayerWallet::new();
        wallet.add_coins(100);
        assert!(wallet.try_spend(50));
        assert_eq!(wallet.get_balance(), 50);
    }

    #[test]
    fn test_try_spend_insufficient() {
        let mut wallet = PlayerWallet::new();
        wallet.add_coins(30);
        assert!(!wallet.try_spend(50));
        assert_eq!(wallet.get_balance(), 30); // Unchanged
    }

    #[test]
    fn test_try_spend_exact() {
        let mut wallet = PlayerWallet::new();
        wallet.add_coins(50);
        assert!(wallet.try_spend(50));
        assert_eq!(wallet.get_balance(), 0);
    }

    #[test]
    fn test_purchase_count() {
        let mut wallet = PlayerWallet::new();
        assert_eq!(wallet.get_purchase_count("health_pack"), 0);
        wallet.record_purchase("health_pack");
        assert_eq!(wallet.get_purchase_count("health_pack"), 1);
        wallet.record_purchase("health_pack");
        assert_eq!(wallet.get_purchase_count("health_pack"), 2);
        assert_eq!(wallet.get_purchase_count("sword"), 0);
    }

    #[test]
    fn test_reset() {
        let mut wallet = PlayerWallet::new();
        wallet.add_coins(100);
        wallet.record_purchase("health_pack");
        wallet.record_purchase("sword");

        wallet.reset();

        assert_eq!(wallet.get_balance(), 0);
        assert_eq!(wallet.get_purchase_count("health_pack"), 0);
        assert_eq!(wallet.get_purchase_count("sword"), 0);
    }
}
