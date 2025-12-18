//! Anti-cheat configuration with safe defaults.

/// Configuration for all anti-cheat thresholds and limits.
/// All fields are public for easy construction in tests.
#[derive(Debug, Clone)]
pub struct AntiCheatConfig {
    // Rate limits (per second)
    /// Maximum movement inputs per second (default: 120)
    pub max_inputs_per_second: u32,
    /// Maximum attacks per second (default: 4)
    pub max_attacks_per_second: u32,
    /// Maximum block edits per second (default: 10)
    pub max_block_edits_per_second: u32,
    /// Maximum ready toggles per second (default: 5)
    pub max_ready_toggles_per_second: u32,
    /// Maximum training resets per second (default: 1)
    pub max_training_resets_per_second: u32,
    /// Maximum hotbar slot selections per second (default: 20)
    pub max_slot_selects_per_second: u32,
    /// Maximum inventory item uses per second (default: 4)
    pub max_inventory_uses_per_second: u32,
    /// Maximum shop buy requests per second (default: 5)
    pub max_shop_buys_per_second: u32,

    // Physics limits (per tick at 60Hz)
    /// Maximum movement distance per tick in blocks (default: 0.25)
    pub max_speed_per_tick: f32,
    /// Maximum acceleration in blocks/tick^2 (default: 1.5)
    pub max_acceleration: f32,

    // Sanction thresholds
    /// Strike count to trigger warning (default: 3)
    pub warning_threshold: u32,
    /// Strike count to trigger kick (default: 5)
    pub kick_threshold: u32,
    /// Strike count to trigger temp ban (default: 10)
    pub ban_threshold: u32,

    // Ban duration
    /// Ban duration in seconds (default: 3600 = 1 hour)
    pub ban_duration_seconds: u64,
}

impl Default for AntiCheatConfig {
    fn default() -> Self {
        Self {
            // Rate limits - generous to avoid false positives
            max_inputs_per_second: 120, // 2x normal rate for lag compensation
            max_attacks_per_second: 4,  // 2x theoretical max (cooldown-limited)
            max_block_edits_per_second: 10, // ~2.5x theoretical max
            max_ready_toggles_per_second: 5, // No gameplay reason to spam
            max_training_resets_per_second: 1, // Once per second to prevent spam
            max_slot_selects_per_second: 20, // Allows rapid scroll wheel usage
            max_inventory_uses_per_second: 4, // ~2x theoretical max (cooldown-limited)
            max_shop_buys_per_second: 5, // 5 requests per second

            // Physics limits
            max_speed_per_tick: 0.25, // Based on movement system max speed
            max_acceleration: 1.5,    // Based on movement physics

            // Sanction thresholds
            warning_threshold: 3,
            kick_threshold: 5,
            ban_threshold: 10,

            // Ban duration
            ban_duration_seconds: 3600, // 1 hour
        }
    }
}

impl AntiCheatConfig {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AntiCheatConfig::default();
        assert_eq!(config.max_inputs_per_second, 120);
        assert_eq!(config.warning_threshold, 3);
        assert_eq!(config.kick_threshold, 5);
        assert_eq!(config.ban_threshold, 10);
        assert!(config.warning_threshold < config.kick_threshold);
        assert!(config.kick_threshold < config.ban_threshold);
    }
}
