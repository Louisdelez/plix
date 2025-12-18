//! Recoil and accuracy mechanics.
//!
//! Tracks accumulated recoil from shots and applies spread to projectile direction.

use plix_common::time::Tick;

/// Tracks recoil accumulation for spread calculation.
#[derive(Debug, Clone)]
pub struct RecoilState {
    /// Current accumulated recoil in degrees
    accumulated_deg: f32,
    /// Tick when last shot was fired (for decay calculation)
    last_shot_tick: Option<Tick>,
    /// Recoil recovery window in ticks
    recovery_ticks: u32,
    /// Maximum recoil cap in degrees
    max_recoil_deg: f32,
}

impl Default for RecoilState {
    fn default() -> Self {
        Self {
            accumulated_deg: 0.0,
            last_shot_tick: None,
            recovery_ticks: 30, // 0.5s at 60 TPS
            max_recoil_deg: 5.0,
        }
    }
}

impl RecoilState {
    /// Create a new recoil tracker with custom parameters.
    pub fn new(recovery_ticks: u32, max_recoil_deg: f32) -> Self {
        Self {
            accumulated_deg: 0.0,
            last_shot_tick: None,
            recovery_ticks,
            max_recoil_deg,
        }
    }

    /// Get current recoil amount after applying decay.
    pub fn current_recoil(&self, current_tick: Tick) -> f32 {
        match self.last_shot_tick {
            Some(last_tick) => {
                let ticks_elapsed = current_tick.0.saturating_sub(last_tick.0);
                if ticks_elapsed >= self.recovery_ticks {
                    // Fully recovered
                    0.0
                } else {
                    // Linear decay
                    let decay_ratio = ticks_elapsed as f32 / self.recovery_ticks as f32;
                    self.accumulated_deg * (1.0 - decay_ratio)
                }
            }
            None => 0.0,
        }
    }

    /// Calculate effective spread including base spread, movement penalty, and recoil.
    pub fn get_effective_spread(
        &self,
        base_spread_deg: f32,
        is_moving: bool,
        movement_spread_mult: f32,
        current_tick: Tick,
    ) -> f32 {
        let movement_spread = if is_moving {
            base_spread_deg * movement_spread_mult
        } else {
            base_spread_deg
        };

        movement_spread + self.current_recoil(current_tick)
    }

    /// Add recoil from firing a shot.
    pub fn add_shot(&mut self, recoil_per_shot_deg: f32, current_tick: Tick) {
        // Get current recoil (with decay) and add new recoil
        let current = self.current_recoil(current_tick);
        self.accumulated_deg = (current + recoil_per_shot_deg).min(self.max_recoil_deg);
        self.last_shot_tick = Some(current_tick);
    }

    /// Reset recoil state (e.g., on respawn).
    pub fn reset(&mut self) {
        self.accumulated_deg = 0.0;
        self.last_shot_tick = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recoil_initial_zero() {
        let state = RecoilState::default();
        assert_eq!(state.current_recoil(Tick(0)), 0.0);
        assert_eq!(state.current_recoil(Tick(1000)), 0.0);
    }

    #[test]
    fn test_recoil_add_shot() {
        let mut state = RecoilState::default();

        state.add_shot(1.0, Tick(100));
        assert_eq!(state.current_recoil(Tick(100)), 1.0);

        state.add_shot(1.0, Tick(100));
        assert_eq!(state.current_recoil(Tick(100)), 2.0);
    }

    #[test]
    fn test_recoil_caps_at_max() {
        let mut state = RecoilState::new(30, 5.0);

        // Add more recoil than max
        for _ in 0..10 {
            state.add_shot(1.0, Tick(100));
        }

        assert_eq!(state.current_recoil(Tick(100)), 5.0);
    }

    #[test]
    fn test_recoil_decay() {
        let mut state = RecoilState::new(30, 5.0); // 30 tick recovery

        state.add_shot(3.0, Tick(100));
        assert_eq!(state.current_recoil(Tick(100)), 3.0);

        // Half decay at 15 ticks
        let recoil_at_half = state.current_recoil(Tick(115));
        assert!((recoil_at_half - 1.5).abs() < 0.01);

        // Full decay at 30 ticks
        assert_eq!(state.current_recoil(Tick(130)), 0.0);
    }

    #[test]
    fn test_recoil_effective_spread_standing() {
        let state = RecoilState::default();
        let spread = state.get_effective_spread(2.0, false, 1.5, Tick(0));
        assert_eq!(spread, 2.0); // Base spread only
    }

    #[test]
    fn test_recoil_effective_spread_moving() {
        let state = RecoilState::default();
        let spread = state.get_effective_spread(2.0, true, 1.5, Tick(0));
        assert_eq!(spread, 3.0); // 2.0 * 1.5 = 3.0
    }

    #[test]
    fn test_recoil_effective_spread_with_recoil() {
        let mut state = RecoilState::default();
        state.add_shot(2.0, Tick(100));

        // Standing: base 2.0 + recoil 2.0 = 4.0
        let spread = state.get_effective_spread(2.0, false, 1.5, Tick(100));
        assert_eq!(spread, 4.0);

        // Moving: base 2.0 * 1.5 + recoil 2.0 = 5.0
        let spread = state.get_effective_spread(2.0, true, 1.5, Tick(100));
        assert_eq!(spread, 5.0);
    }

    #[test]
    fn test_recoil_reset() {
        let mut state = RecoilState::default();
        state.add_shot(3.0, Tick(100));
        assert_eq!(state.current_recoil(Tick(100)), 3.0);

        state.reset();
        assert_eq!(state.current_recoil(Tick(100)), 0.0);
    }

    #[test]
    fn test_recoil_stacking_with_decay() {
        let mut state = RecoilState::new(30, 5.0);

        // First shot at tick 100
        state.add_shot(1.0, Tick(100));

        // Second shot at tick 115 (15 ticks later, half decay)
        // Current recoil before shot: 1.0 * 0.5 = 0.5
        // After shot: 0.5 + 1.0 = 1.5
        state.add_shot(1.0, Tick(115));
        assert!((state.current_recoil(Tick(115)) - 1.5).abs() < 0.01);
    }
}
