//! Crafting configuration per game mode.

use plix_common::types::GameMode;

/// Crafting configuration for a game mode.
#[derive(Debug, Clone, Copy)]
pub struct CraftConfig {
    /// Whether crafting is enabled in this game mode.
    pub enabled: bool,
    /// Cooldown between successful crafts (in ticks).
    pub cooldown_ticks: u32,
}

impl Default for CraftConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cooldown_ticks: 60, // 1 second at 60 Hz
        }
    }
}

/// Get the crafting configuration for a game mode.
///
/// Per spec:
/// - Training: enabled (practice crafting)
/// - BR Lite: enabled (core gameplay loop)
/// - TDM/FFA/CTF: disabled (spawn loadout based)
pub fn get_craft_config(mode: GameMode) -> CraftConfig {
    match mode {
        GameMode::Training => CraftConfig {
            enabled: true,
            cooldown_ticks: 60,
        },
        GameMode::BrLite => CraftConfig {
            enabled: true,
            cooldown_ticks: 60,
        },
        GameMode::Tdm | GameMode::Ffa | GameMode::Ctf => CraftConfig {
            enabled: false,
            cooldown_ticks: 60,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_training_mode_enabled() {
        let config = get_craft_config(GameMode::Training);
        assert!(config.enabled);
        assert_eq!(config.cooldown_ticks, 60);
    }

    #[test]
    fn test_br_lite_enabled() {
        let config = get_craft_config(GameMode::BrLite);
        assert!(config.enabled);
    }

    #[test]
    fn test_tdm_disabled() {
        let config = get_craft_config(GameMode::Tdm);
        assert!(!config.enabled);
    }

    #[test]
    fn test_ffa_disabled() {
        let config = get_craft_config(GameMode::Ffa);
        assert!(!config.enabled);
    }

    #[test]
    fn test_ctf_disabled() {
        let config = get_craft_config(GameMode::Ctf);
        assert!(!config.enabled);
    }
}
