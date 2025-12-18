//! Economy configuration for per-mode settings.

use plix_common::types::GameMode;
use serde::{Deserialize, Serialize};

use super::shop::{default_shop_offers, ShopOffer};

/// Per-mode economy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyConfig {
    /// Whether economy is active for this mode
    pub enabled: bool,
    /// Coins awarded for player kill
    pub kill_reward: u32,
    /// Coins awarded for CTF flag capture
    pub ctf_capture_reward: u32,
    /// Coins awarded for BR Lite placement [1st, 2nd, 3rd]
    pub br_placement_rewards: [u32; 3],
    /// Available shop offers
    pub shop_offers: Vec<ShopOffer>,
}

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            kill_reward: 10,
            ctf_capture_reward: 25,
            br_placement_rewards: [50, 30, 15],
            shop_offers: default_shop_offers(),
        }
    }
}

impl EconomyConfig {
    /// Get placement reward for a given position (1-indexed).
    ///
    /// Returns 0 for positions outside top 3.
    pub fn get_placement_reward(&self, position: u8) -> u32 {
        match position {
            1 => self.br_placement_rewards[0],
            2 => self.br_placement_rewards[1],
            3 => self.br_placement_rewards[2],
            _ => 0,
        }
    }

    /// Validate configuration.
    ///
    /// Returns true if all values are valid.
    pub fn is_valid(&self) -> bool {
        // Rewards can be 0 (disabled), shop offers are validated separately
        true
    }
}

/// Get economy config for a game mode, applying mode-specific defaults.
///
/// If arena_config is provided, uses its values. Otherwise, applies defaults:
/// - Training, TDM, FFA: economy disabled
/// - CTF, BrLite: economy enabled
pub fn get_economy_config(mode: GameMode, arena_config: Option<&EconomyConfig>) -> EconomyConfig {
    if let Some(config) = arena_config {
        return config.clone();
    }

    // Mode-specific defaults
    let enabled = match mode {
        GameMode::Training | GameMode::Tdm | GameMode::Ffa => false,
        GameMode::Ctf | GameMode::BrLite => true,
    };

    EconomyConfig {
        enabled,
        ..Default::default()
    }
}

/// Convert arena TOML config to runtime EconomyConfig (T055).
///
/// Merges arena-specific overrides with mode defaults:
/// - enabled: arena value if set, otherwise mode default
/// - kill_reward: arena value if set, otherwise 10
/// - ctf_capture_reward: arena value if set, otherwise 25
/// - br_placement_rewards: arena values if set, otherwise [50, 30, 15]
/// - shop_offers: arena offers if non-empty, otherwise defaults
pub fn from_arena_config(
    mode: GameMode,
    arena_config: Option<&plix_arena::format::EconomyArenaConfig>,
) -> EconomyConfig {
    let mode_default = get_economy_config(mode, None);

    let Some(arena) = arena_config else {
        return mode_default;
    };

    // Override with arena values where specified
    let enabled = arena.enabled.unwrap_or(mode_default.enabled);
    let kill_reward = arena.kill_reward.unwrap_or(mode_default.kill_reward);
    let ctf_capture_reward = arena
        .ctf_capture_reward
        .unwrap_or(mode_default.ctf_capture_reward);
    let br_placement_rewards = arena
        .br_placement_rewards
        .unwrap_or(mode_default.br_placement_rewards);

    // Convert shop offers from arena config format
    let shop_offers = if arena.shop_offers.is_empty() {
        mode_default.shop_offers
    } else {
        arena
            .shop_offers
            .iter()
            .filter_map(|ao| {
                // Validate and convert
                if ao.price == 0 || ao.quantity == 0 {
                    tracing::warn!(
                        offer_id = %ao.offer_id,
                        "Skipping invalid shop offer (price=0 or quantity=0)"
                    );
                    return None;
                }
                Some(ShopOffer {
                    offer_id: ao.offer_id.clone(),
                    item_id: plix_common::inventory::ItemId(ao.item_id),
                    quantity: ao.quantity,
                    price: ao.price,
                    allowed_modes: None, // Arena offers are available in current mode
                    max_per_match: ao.max_per_match,
                })
            })
            .collect()
    };

    EconomyConfig {
        enabled,
        kill_reward,
        ctf_capture_reward,
        br_placement_rewards,
        shop_offers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EconomyConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.kill_reward, 10);
        assert_eq!(config.ctf_capture_reward, 25);
        assert_eq!(config.br_placement_rewards, [50, 30, 15]);
    }

    #[test]
    fn test_placement_reward() {
        let config = EconomyConfig::default();
        assert_eq!(config.get_placement_reward(1), 50);
        assert_eq!(config.get_placement_reward(2), 30);
        assert_eq!(config.get_placement_reward(3), 15);
        assert_eq!(config.get_placement_reward(4), 0);
        assert_eq!(config.get_placement_reward(0), 0);
    }

    #[test]
    fn test_mode_defaults() {
        // Training, TDM, FFA: disabled
        let training = get_economy_config(GameMode::Training, None);
        assert!(!training.enabled);

        let tdm = get_economy_config(GameMode::Tdm, None);
        assert!(!tdm.enabled);

        let ffa = get_economy_config(GameMode::Ffa, None);
        assert!(!ffa.enabled);

        // CTF, BrLite: enabled
        let ctf = get_economy_config(GameMode::Ctf, None);
        assert!(ctf.enabled);

        let br = get_economy_config(GameMode::BrLite, None);
        assert!(br.enabled);
    }

    #[test]
    fn test_arena_config_override() {
        let custom = EconomyConfig {
            enabled: true,
            kill_reward: 20,
            ctf_capture_reward: 50,
            br_placement_rewards: [100, 50, 25],
            shop_offers: vec![],
        };

        // Should use arena config even for Training mode
        let config = get_economy_config(GameMode::Training, Some(&custom));
        assert!(config.enabled);
        assert_eq!(config.kill_reward, 20);
    }

    #[test]
    fn test_from_arena_config_none() {
        // When arena config is None, should return mode defaults
        let config = from_arena_config(GameMode::Ctf, None);
        assert!(config.enabled); // CTF enables economy by default
        assert_eq!(config.kill_reward, 10);
    }

    #[test]
    fn test_from_arena_config_partial_override() {
        // Create partial arena config
        let arena = plix_arena::format::EconomyArenaConfig {
            enabled: Some(true),
            kill_reward: Some(50),
            ctf_capture_reward: None, // Not set, should use default
            br_placement_rewards: None,
            shop_offers: vec![],
        };

        let config = from_arena_config(GameMode::Tdm, Some(&arena));
        assert!(config.enabled); // Overridden from default false
        assert_eq!(config.kill_reward, 50); // Overridden
        assert_eq!(config.ctf_capture_reward, 25); // Default
    }

    #[test]
    fn test_from_arena_config_shop_offers() {
        let arena = plix_arena::format::EconomyArenaConfig {
            enabled: None,
            kill_reward: None,
            ctf_capture_reward: None,
            br_placement_rewards: None,
            shop_offers: vec![plix_arena::format::ShopOfferConfig {
                offer_id: "custom_item".to_string(),
                item_id: 1, // HEALTH_PACK
                quantity: 2,
                price: 30,
                max_per_match: Some(5),
            }],
        };

        let config = from_arena_config(GameMode::Ctf, Some(&arena));
        assert_eq!(config.shop_offers.len(), 1);
        assert_eq!(config.shop_offers[0].offer_id, "custom_item");
        assert_eq!(config.shop_offers[0].quantity, 2);
        assert_eq!(config.shop_offers[0].price, 30);
        assert_eq!(config.shop_offers[0].max_per_match, Some(5));
    }
}
