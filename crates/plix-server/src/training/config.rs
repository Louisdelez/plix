//! Training mode configuration

use serde::{Deserialize, Serialize};

/// Bot behavior type configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BotBehaviorType {
    /// Stationary bot (target dummy)
    #[default]
    Dummy,
    /// Random slow movement around spawn point
    Roam,
    /// Side-to-side oscillation around spawn point
    Strafe,
}

/// Training mode configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Number of bots to spawn (0-20)
    pub bot_count: u8,
    /// Bot behavior type for all bots
    pub bot_behavior: BotBehaviorType,
    /// Ticks before bot respawns after death (default: 180 = 3s @ 60Hz)
    pub bot_respawn_delay_ticks: u32,
    /// Ticks before player respawns after death (default: 60 = 1s @ 60Hz)
    pub player_respawn_delay_ticks: u32,
    /// Player immune to damage
    pub invincibility_player: bool,
    /// Bots immune to damage (hits still count for stats)
    pub invincibility_bots: bool,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            bot_count: 5,
            bot_behavior: BotBehaviorType::Dummy,
            bot_respawn_delay_ticks: 180,   // 3 seconds at 60Hz
            player_respawn_delay_ticks: 60, // 1 second at 60Hz
            invincibility_player: false,
            invincibility_bots: false,
        }
    }
}

impl TrainingConfig {
    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if self.bot_count > 20 {
            return Err("bot_count must be 0-20".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_training_config_defaults() {
        let config = TrainingConfig::default();
        assert_eq!(config.bot_count, 5);
        assert_eq!(config.bot_behavior, BotBehaviorType::Dummy);
        assert_eq!(config.bot_respawn_delay_ticks, 180);
        assert_eq!(config.player_respawn_delay_ticks, 60);
        assert!(!config.invincibility_player);
        assert!(!config.invincibility_bots);
    }

    #[test]
    fn test_training_config_validation() {
        let mut config = TrainingConfig::default();
        assert!(config.validate().is_ok());

        config.bot_count = 21;
        assert!(config.validate().is_err());

        config.bot_count = 20;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_bot_behavior_type_default() {
        let behavior = BotBehaviorType::default();
        assert_eq!(behavior, BotBehaviorType::Dummy);
    }

    #[test]
    fn test_bot_behavior_types() {
        // Verify all behavior types can be constructed
        let _dummy = BotBehaviorType::Dummy;
        let _roam = BotBehaviorType::Roam;
        let _strafe = BotBehaviorType::Strafe;
    }
}
