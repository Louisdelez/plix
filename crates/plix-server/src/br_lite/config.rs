//! BR Lite configuration types and defaults

use serde::{Deserialize, Serialize};

/// Configuration for a BR Lite match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrLiteConfig {
    /// Minimum players to start a match (default: 4)
    pub min_players: u8,
    /// Duration of PostMatch screen in seconds (default: 10)
    pub post_match_delay_secs: u32,
    /// Duration of temporary effects like speed boost in seconds (default: 10)
    pub bonus_duration_secs: u32,
    /// Zone center (XZ coordinates) - None means arena center
    pub zone_center: Option<[f32; 2]>,
    /// Initial zone radius - None means half of arena size
    pub initial_radius: Option<f32>,
    /// Zone phase configurations
    pub phases: Vec<ZonePhase>,
    /// Damage tick interval in server ticks (default: 60 = 1 second at 60Hz)
    pub damage_tick_interval: u32,
    /// Whether loot is enabled
    pub loot_enabled: bool,
}

impl Default for BrLiteConfig {
    fn default() -> Self {
        Self {
            min_players: 4,
            post_match_delay_secs: 10,
            bonus_duration_secs: 10,
            zone_center: None,
            initial_radius: None,
            phases: default_phases(),
            damage_tick_interval: 60, // 1 second at 60Hz
            loot_enabled: true,
        }
    }
}

impl BrLiteConfig {
    /// Create config with custom min_players
    pub fn with_min_players(mut self, min_players: u8) -> Self {
        self.min_players = min_players;
        self
    }

    /// Create config with custom phases
    pub fn with_phases(mut self, phases: Vec<ZonePhase>) -> Self {
        self.phases = phases;
        self
    }

    /// Validate configuration bounds
    /// Returns Ok(()) if valid, Err with description if invalid
    pub fn validate(&self) -> Result<(), String> {
        if self.min_players < 2 {
            return Err("min_players must be at least 2".to_string());
        }
        if self.min_players > 16 {
            return Err("min_players cannot exceed 16".to_string());
        }
        if self.phases.is_empty() {
            return Err("at least one zone phase required".to_string());
        }
        if self.damage_tick_interval == 0 {
            return Err("damage_tick_interval must be > 0".to_string());
        }

        // Validate phases are shrinking (each end_radius smaller than previous)
        let mut prev_radius: Option<f32> = None;
        for (i, phase) in self.phases.iter().enumerate() {
            if phase.shrink_duration_secs == 0 {
                return Err(format!("phase {} shrink_duration must be > 0", i));
            }
            if phase.end_radius < 0.0 {
                return Err(format!("phase {} end_radius must be >= 0", i));
            }
            if let Some(prev) = prev_radius {
                if phase.end_radius >= prev {
                    return Err(format!(
                        "phase {} end_radius ({}) must be smaller than previous ({})",
                        i, phase.end_radius, prev
                    ));
                }
            }
            prev_radius = Some(phase.end_radius);
        }

        Ok(())
    }

    /// Get total match duration in seconds (sum of all phases)
    pub fn total_duration_secs(&self) -> u32 {
        self.phases
            .iter()
            .map(|p| p.stable_duration_secs + p.shrink_duration_secs)
            .sum()
    }
}

/// Configuration for a single zone phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZonePhase {
    /// Duration the zone stays stable in seconds
    pub stable_duration_secs: u32,
    /// Duration to shrink to target radius in seconds
    pub shrink_duration_secs: u32,
    /// Target radius after shrinking (percentage of initial or absolute)
    pub end_radius: f32,
    /// Damage per second when outside the zone
    pub damage_per_tick: u32,
}

impl ZonePhase {
    /// Create a new zone phase
    pub fn new(
        stable_duration_secs: u32,
        shrink_duration_secs: u32,
        end_radius: f32,
        damage_per_tick: u32,
    ) -> Self {
        Self {
            stable_duration_secs,
            shrink_duration_secs,
            end_radius,
            damage_per_tick,
        }
    }
}

/// Default 5-phase schedule for ~8 minute matches
pub fn default_phases() -> Vec<ZonePhase> {
    vec![
        // Phase 0: 60s stable, 30s shrink to 80% radius, 5 damage/s
        ZonePhase::new(60, 30, 0.80, 5),
        // Phase 1: 45s stable, 25s shrink to 60% radius, 10 damage/s
        ZonePhase::new(45, 25, 0.60, 10),
        // Phase 2: 30s stable, 20s shrink to 40% radius, 15 damage/s
        ZonePhase::new(30, 20, 0.40, 15),
        // Phase 3: 20s stable, 15s shrink to 20% radius, 25 damage/s
        ZonePhase::new(20, 15, 0.20, 25),
        // Phase 4: 15s stable, 10s shrink to 5% radius, 50 damage/s (final)
        ZonePhase::new(15, 10, 0.05, 50),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BrLiteConfig::default();
        assert_eq!(config.min_players, 4);
        assert_eq!(config.bonus_duration_secs, 10);
        assert_eq!(config.phases.len(), 5);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_min_players() {
        let mut config = BrLiteConfig::default();
        config.min_players = 1;
        assert!(config.validate().is_err());

        config.min_players = 17;
        assert!(config.validate().is_err());

        config.min_players = 4;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_empty_phases() {
        let mut config = BrLiteConfig::default();
        config.phases = vec![];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_non_shrinking_phases() {
        let mut config = BrLiteConfig::default();
        config.phases = vec![
            ZonePhase::new(30, 10, 0.8, 5),
            ZonePhase::new(30, 10, 0.9, 10), // Invalid: larger than previous
        ];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_total_duration() {
        let config = BrLiteConfig::default();
        // Phase 0: 60+30 = 90
        // Phase 1: 45+25 = 70
        // Phase 2: 30+20 = 50
        // Phase 3: 20+15 = 35
        // Phase 4: 15+10 = 25
        // Total: 270 seconds
        assert_eq!(config.total_duration_secs(), 270);
    }
}
