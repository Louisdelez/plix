//! Block physics configuration
//!
//! Configuration for the block physics system (gravity, liquids).

use serde::{Deserialize, Serialize};

/// Configuration for the block physics system.
/// Stored per world/server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockPhysicsConfig {
    /// Enable gravity physics for affected blocks (e.g., sand)
    gravity_enabled: bool,
    /// Enable liquid spreading (optional feature)
    liquids_enabled: bool,
    /// Maximum physics events processed per tick
    max_events_per_tick: u32,
    /// Maximum horizontal spread distance for liquids
    max_liquid_spread_distance: u8,
}

impl Default for BlockPhysicsConfig {
    fn default() -> Self {
        Self {
            gravity_enabled: true,
            liquids_enabled: false,
            max_events_per_tick: 100,
            max_liquid_spread_distance: 7,
        }
    }
}

impl BlockPhysicsConfig {
    /// Create default physics config (gravity on, liquids off)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with all physics disabled
    pub fn disabled() -> Self {
        Self {
            gravity_enabled: false,
            liquids_enabled: false,
            max_events_per_tick: 0,
            max_liquid_spread_distance: 0,
        }
    }

    /// Create with custom settings
    pub fn with_settings(
        gravity_enabled: bool,
        liquids_enabled: bool,
        max_events_per_tick: u32,
        max_liquid_spread_distance: u8,
    ) -> Self {
        Self {
            gravity_enabled,
            liquids_enabled,
            max_events_per_tick: max_events_per_tick.max(1),
            max_liquid_spread_distance: max_liquid_spread_distance.clamp(1, 15),
        }
    }

    /// Check if gravity is enabled
    pub fn gravity_enabled(&self) -> bool {
        self.gravity_enabled
    }

    /// Check if liquids are enabled
    pub fn liquids_enabled(&self) -> bool {
        self.liquids_enabled
    }

    /// Get maximum events per tick budget
    pub fn max_events_per_tick(&self) -> u32 {
        self.max_events_per_tick
    }

    /// Get maximum liquid spread distance
    pub fn max_liquid_spread_distance(&self) -> u8 {
        self.max_liquid_spread_distance
    }

    /// Set gravity enabled state
    pub fn set_gravity_enabled(&mut self, enabled: bool) {
        self.gravity_enabled = enabled;
    }

    /// Set liquids enabled state
    pub fn set_liquids_enabled(&mut self, enabled: bool) {
        self.liquids_enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BlockPhysicsConfig::default();
        assert!(config.gravity_enabled());
        assert!(!config.liquids_enabled());
        assert_eq!(config.max_events_per_tick(), 100);
        assert_eq!(config.max_liquid_spread_distance(), 7);
    }

    #[test]
    fn test_disabled_config() {
        let config = BlockPhysicsConfig::disabled();
        assert!(!config.gravity_enabled());
        assert!(!config.liquids_enabled());
        assert_eq!(config.max_events_per_tick(), 0);
    }

    #[test]
    fn test_custom_config() {
        let config = BlockPhysicsConfig::with_settings(true, true, 200, 10);
        assert!(config.gravity_enabled());
        assert!(config.liquids_enabled());
        assert_eq!(config.max_events_per_tick(), 200);
        assert_eq!(config.max_liquid_spread_distance(), 10);
    }

    #[test]
    fn test_config_validation() {
        // max_events_per_tick should be at least 1
        let config = BlockPhysicsConfig::with_settings(true, false, 0, 7);
        assert_eq!(config.max_events_per_tick(), 1);

        // max_liquid_spread_distance should be clamped to 1-15
        let config = BlockPhysicsConfig::with_settings(true, true, 100, 0);
        assert_eq!(config.max_liquid_spread_distance(), 1);

        let config = BlockPhysicsConfig::with_settings(true, true, 100, 20);
        assert_eq!(config.max_liquid_spread_distance(), 15);
    }

    #[test]
    fn test_config_setters() {
        let mut config = BlockPhysicsConfig::default();
        assert!(config.gravity_enabled());

        config.set_gravity_enabled(false);
        assert!(!config.gravity_enabled());

        config.set_liquids_enabled(true);
        assert!(config.liquids_enabled());
    }
}
