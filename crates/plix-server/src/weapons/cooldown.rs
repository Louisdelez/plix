//! Per-weapon cooldown tracking.
//!
//! Tracks when each weapon was last used to enforce attack rate limits.

use plix_common::time::Tick;
use plix_common::types::ItemId;
use std::collections::HashMap;

/// Tracks cooldown state for all weapons a player has used.
#[derive(Debug, Clone, Default)]
pub struct CooldownState {
    /// Map of item ID to the tick when cooldown expires
    ready_at: HashMap<ItemId, Tick>,
}

impl CooldownState {
    /// Create a new cooldown tracker with no active cooldowns.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a weapon is ready to use (cooldown expired).
    pub fn is_ready(&self, item_id: ItemId, current_tick: Tick) -> bool {
        match self.ready_at.get(&item_id) {
            Some(&ready_tick) => current_tick.0 >= ready_tick.0,
            None => true, // Never used = ready
        }
    }

    /// Get remaining cooldown ticks (0 if ready).
    pub fn remaining(&self, item_id: ItemId, current_tick: Tick) -> u32 {
        match self.ready_at.get(&item_id) {
            Some(&ready_tick) => {
                if current_tick.0 >= ready_tick.0 {
                    0
                } else {
                    ready_tick.0.saturating_sub(current_tick.0)
                }
            }
            None => 0,
        }
    }

    /// Start cooldown for a weapon after it was used.
    pub fn trigger(&mut self, item_id: ItemId, current_tick: Tick, cooldown_ticks: u32) {
        let ready_at = Tick(current_tick.0.saturating_add(cooldown_ticks));
        self.ready_at.insert(item_id, ready_at);
    }

    /// Clear all cooldowns (e.g., on respawn).
    pub fn clear(&mut self) {
        self.ready_at.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cooldown_initially_ready() {
        let state = CooldownState::new();
        assert!(state.is_ready(ItemId::SWORD, Tick(0)));
        assert!(state.is_ready(ItemId::BOW, Tick(100)));
        assert_eq!(state.remaining(ItemId::SWORD, Tick(0)), 0);
    }

    #[test]
    fn test_cooldown_trigger_and_check() {
        let mut state = CooldownState::new();
        let current = Tick(100);

        // Trigger 36 tick cooldown
        state.trigger(ItemId::SWORD, current, 36);

        // Should not be ready immediately
        assert!(!state.is_ready(ItemId::SWORD, Tick(100)));
        assert!(!state.is_ready(ItemId::SWORD, Tick(135)));
        assert_eq!(state.remaining(ItemId::SWORD, Tick(100)), 36);
        assert_eq!(state.remaining(ItemId::SWORD, Tick(120)), 16);

        // Should be ready at tick 136
        assert!(state.is_ready(ItemId::SWORD, Tick(136)));
        assert_eq!(state.remaining(ItemId::SWORD, Tick(136)), 0);
    }

    #[test]
    fn test_cooldown_independent_per_weapon() {
        let mut state = CooldownState::new();

        state.trigger(ItemId::SWORD, Tick(100), 36);
        state.trigger(ItemId::BOW, Tick(100), 48);

        // Sword ready at 136, bow ready at 148
        assert!(state.is_ready(ItemId::SWORD, Tick(140)));
        assert!(!state.is_ready(ItemId::BOW, Tick(140)));
        assert!(state.is_ready(ItemId::BOW, Tick(150)));
    }

    #[test]
    fn test_cooldown_clear() {
        let mut state = CooldownState::new();
        state.trigger(ItemId::SWORD, Tick(100), 36);
        state.trigger(ItemId::BOW, Tick(100), 48);

        assert!(!state.is_ready(ItemId::SWORD, Tick(100)));
        assert!(!state.is_ready(ItemId::BOW, Tick(100)));

        state.clear();

        assert!(state.is_ready(ItemId::SWORD, Tick(100)));
        assert!(state.is_ready(ItemId::BOW, Tick(100)));
    }

    #[test]
    fn test_cooldown_retrigger() {
        let mut state = CooldownState::new();

        state.trigger(ItemId::SWORD, Tick(100), 36);
        assert!(!state.is_ready(ItemId::SWORD, Tick(130)));

        // Retrigger at tick 130 (mid-cooldown) - should extend
        state.trigger(ItemId::SWORD, Tick(130), 36);
        assert!(!state.is_ready(ItemId::SWORD, Tick(160)));
        assert!(state.is_ready(ItemId::SWORD, Tick(166)));
    }
}
