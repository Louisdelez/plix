//! Per-player crafting cooldown tracking.

use plix_common::time::Tick;

/// Tracks crafting cooldown for a player.
#[derive(Debug, Clone, Default)]
pub struct CraftCooldown {
    /// Tick of last successful craft (None if never crafted).
    last_craft_tick: Option<Tick>,
}

impl CraftCooldown {
    /// Create a new cooldown tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the player is on cooldown.
    ///
    /// Returns true if a craft would be rejected due to cooldown.
    pub fn is_on_cooldown(&self, current_tick: Tick, cooldown_ticks: u32) -> bool {
        match self.last_craft_tick {
            Some(last) => {
                let elapsed = current_tick.0.wrapping_sub(last.0);
                elapsed < cooldown_ticks
            }
            None => false,
        }
    }

    /// Record a successful craft at the given tick.
    pub fn record_craft(&mut self, tick: Tick) {
        self.last_craft_tick = Some(tick);
    }

    /// Get ticks remaining on cooldown (0 if not on cooldown).
    pub fn remaining_ticks(&self, current_tick: Tick, cooldown_ticks: u32) -> u32 {
        match self.last_craft_tick {
            Some(last) => {
                let elapsed = current_tick.0.wrapping_sub(last.0);
                if elapsed < cooldown_ticks {
                    cooldown_ticks - elapsed
                } else {
                    0
                }
            }
            None => 0,
        }
    }

    /// Reset cooldown (e.g., on respawn).
    pub fn reset(&mut self) {
        self.last_craft_tick = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cooldown_not_on_cooldown() {
        let cooldown = CraftCooldown::new();
        assert!(!cooldown.is_on_cooldown(Tick(100), 60));
    }

    #[test]
    fn test_after_craft_on_cooldown() {
        let mut cooldown = CraftCooldown::new();
        cooldown.record_craft(Tick(100));

        // Immediately after - on cooldown
        assert!(cooldown.is_on_cooldown(Tick(100), 60));
        assert!(cooldown.is_on_cooldown(Tick(159), 60)); // 59 ticks later

        // After cooldown expires
        assert!(!cooldown.is_on_cooldown(Tick(160), 60)); // 60 ticks later
        assert!(!cooldown.is_on_cooldown(Tick(200), 60)); // Much later
    }

    #[test]
    fn test_remaining_ticks() {
        let mut cooldown = CraftCooldown::new();
        cooldown.record_craft(Tick(100));

        assert_eq!(cooldown.remaining_ticks(Tick(100), 60), 60);
        assert_eq!(cooldown.remaining_ticks(Tick(130), 60), 30);
        assert_eq!(cooldown.remaining_ticks(Tick(159), 60), 1);
        assert_eq!(cooldown.remaining_ticks(Tick(160), 60), 0);
        assert_eq!(cooldown.remaining_ticks(Tick(200), 60), 0);
    }

    #[test]
    fn test_reset() {
        let mut cooldown = CraftCooldown::new();
        cooldown.record_craft(Tick(100));
        assert!(cooldown.is_on_cooldown(Tick(100), 60));

        cooldown.reset();
        assert!(!cooldown.is_on_cooldown(Tick(100), 60));
    }

    #[test]
    fn test_remaining_ticks_no_craft() {
        let cooldown = CraftCooldown::new();
        assert_eq!(cooldown.remaining_ticks(Tick(100), 60), 0);
    }
}
