//! Earning rules for awarding coins on match events.

use super::config::EconomyConfig;
use super::wallet::PlayerWallet;

/// Events that can award coins to players.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EarningEvent {
    /// Player eliminated another player
    Kill,
    /// Team captured enemy flag (CTF mode)
    CtfCapture,
    /// BR Lite placement (1st, 2nd, or 3rd)
    BrPlacement(u8),
}

/// Award coins for an earning event.
///
/// Returns the new balance if coins were awarded, None if economy is disabled or reward is 0.
pub fn award_coins(
    event: EarningEvent,
    wallet: &mut PlayerWallet,
    config: &EconomyConfig,
) -> Option<u32> {
    if !config.enabled {
        return None;
    }

    let amount = match event {
        EarningEvent::Kill => config.kill_reward,
        EarningEvent::CtfCapture => config.ctf_capture_reward,
        EarningEvent::BrPlacement(position) => config.get_placement_reward(position),
    };

    if amount > 0 {
        wallet.add_coins(amount);
        Some(wallet.get_balance())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_config() -> EconomyConfig {
        EconomyConfig {
            enabled: true,
            kill_reward: 10,
            ctf_capture_reward: 25,
            br_placement_rewards: [50, 30, 15],
            shop_offers: vec![],
        }
    }

    fn disabled_config() -> EconomyConfig {
        EconomyConfig {
            enabled: false,
            ..enabled_config()
        }
    }

    #[test]
    fn test_kill_reward() {
        let config = enabled_config();
        let mut wallet = PlayerWallet::new();

        let balance = award_coins(EarningEvent::Kill, &mut wallet, &config);
        assert_eq!(balance, Some(10));
        assert_eq!(wallet.get_balance(), 10);
    }

    #[test]
    fn test_ctf_capture_reward() {
        let config = enabled_config();
        let mut wallet = PlayerWallet::new();

        let balance = award_coins(EarningEvent::CtfCapture, &mut wallet, &config);
        assert_eq!(balance, Some(25));
        assert_eq!(wallet.get_balance(), 25);
    }

    #[test]
    fn test_br_placement_rewards() {
        let config = enabled_config();

        let mut wallet = PlayerWallet::new();
        let balance = award_coins(EarningEvent::BrPlacement(1), &mut wallet, &config);
        assert_eq!(balance, Some(50));

        let mut wallet = PlayerWallet::new();
        let balance = award_coins(EarningEvent::BrPlacement(2), &mut wallet, &config);
        assert_eq!(balance, Some(30));

        let mut wallet = PlayerWallet::new();
        let balance = award_coins(EarningEvent::BrPlacement(3), &mut wallet, &config);
        assert_eq!(balance, Some(15));

        // 4th place gets nothing
        let mut wallet = PlayerWallet::new();
        let balance = award_coins(EarningEvent::BrPlacement(4), &mut wallet, &config);
        assert_eq!(balance, None);
    }

    #[test]
    fn test_economy_disabled() {
        let config = disabled_config();
        let mut wallet = PlayerWallet::new();

        let balance = award_coins(EarningEvent::Kill, &mut wallet, &config);
        assert_eq!(balance, None);
        assert_eq!(wallet.get_balance(), 0); // No change
    }

    #[test]
    fn test_zero_reward() {
        let config = EconomyConfig {
            enabled: true,
            kill_reward: 0, // Kills disabled
            ctf_capture_reward: 25,
            br_placement_rewards: [50, 30, 15],
            shop_offers: vec![],
        };
        let mut wallet = PlayerWallet::new();

        let balance = award_coins(EarningEvent::Kill, &mut wallet, &config);
        assert_eq!(balance, None);
        assert_eq!(wallet.get_balance(), 0);
    }

    #[test]
    fn test_cumulative_rewards() {
        let config = enabled_config();
        let mut wallet = PlayerWallet::new();

        award_coins(EarningEvent::Kill, &mut wallet, &config);
        assert_eq!(wallet.get_balance(), 10);

        award_coins(EarningEvent::Kill, &mut wallet, &config);
        assert_eq!(wallet.get_balance(), 20);

        award_coins(EarningEvent::CtfCapture, &mut wallet, &config);
        assert_eq!(wallet.get_balance(), 45);
    }
}
