//! BR Lite loot system - pickups and temporary effects

use std::collections::HashMap;

use glam::Vec3;
use plix_common::time::Tick;
use plix_common::types::PlayerId;
use serde::{Deserialize, Serialize};

/// Type of loot item
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LootType {
    /// Instant health restore
    HealthPack { heal_amount: u32 },
    /// Temporary speed boost
    SpeedBoost {
        multiplier: f32,
        duration_ticks: u32,
    },
    /// Scrap resource for crafting
    Scrap { quantity: u8 },
}

impl LootType {
    /// Convert to u8 for protocol
    pub fn to_u8(&self) -> u8 {
        match self {
            LootType::HealthPack { .. } => 0,
            LootType::SpeedBoost { .. } => 1,
            LootType::Scrap { .. } => 2,
        }
    }

    /// Get the param value for protocol (heal_amount, multiplier*100, or quantity)
    pub fn param(&self) -> u16 {
        match self {
            LootType::HealthPack { heal_amount } => *heal_amount as u16,
            LootType::SpeedBoost { multiplier, .. } => (*multiplier * 100.0) as u16,
            LootType::Scrap { quantity } => *quantity as u16,
        }
    }
}

/// A loot item in the arena
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootItem {
    /// Unique identifier
    pub id: u16,
    /// World position
    pub position: Vec3,
    /// Type of loot
    pub loot_type: LootType,
    /// Whether this item has been collected
    pub collected: bool,
}

impl LootItem {
    /// Create a new loot item
    pub fn new(id: u16, position: Vec3, loot_type: LootType) -> Self {
        Self {
            id,
            position,
            loot_type,
            collected: false,
        }
    }
}

/// Type of active effect
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EffectType {
    /// Speed boost with multiplier
    SpeedBoost { multiplier: f32 },
}

/// An active temporary effect on a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEffect {
    /// Type of effect
    pub effect_type: EffectType,
    /// Server tick when effect expires
    pub expires_at: Tick,
}

impl ActiveEffect {
    /// Create a new active effect
    pub fn new(effect_type: EffectType, expires_at: Tick) -> Self {
        Self {
            effect_type,
            expires_at,
        }
    }

    /// Check if effect has expired
    pub fn is_expired(&self, current_tick: Tick) -> bool {
        current_tick.0 >= self.expires_at.0
    }
}

/// Pickup radius for loot items (in blocks)
pub const PICKUP_RADIUS: f32 = 1.0;

/// Loot manager for handling spawns, pickups, and effects
#[derive(Debug)]
pub struct LootManager {
    /// All loot items in the arena
    items: Vec<LootItem>,
    /// Active effects per player
    effects: HashMap<PlayerId, Vec<ActiveEffect>>,
    /// Next available loot ID
    next_id: u16,
    /// Default bonus duration in ticks
    bonus_duration_ticks: u32,
}

impl Default for LootManager {
    fn default() -> Self {
        Self::new(600) // 10 seconds at 60Hz
    }
}

impl LootManager {
    /// Create a new loot manager
    pub fn new(bonus_duration_ticks: u32) -> Self {
        Self {
            items: Vec::new(),
            effects: HashMap::new(),
            next_id: 0,
            bonus_duration_ticks,
        }
    }

    /// Spawn a loot item at the given position
    pub fn spawn_loot(&mut self, position: Vec3, loot_type: LootType) -> u16 {
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(LootItem::new(id, position, loot_type));
        id
    }

    /// Get all loot items
    pub fn items(&self) -> &[LootItem] {
        &self.items
    }

    /// Check if a player can pick up any loot at their position
    /// Returns the loot ID if there's an available pickup
    pub fn check_pickup(&self, player_pos: Vec3) -> Option<u16> {
        for item in &self.items {
            if item.collected {
                continue;
            }
            let dx = player_pos.x - item.position.x;
            let dz = player_pos.z - item.position.z;
            let dist_sq = dx * dx + dz * dz;
            if dist_sq < PICKUP_RADIUS * PICKUP_RADIUS {
                return Some(item.id);
            }
        }
        None
    }

    /// Try to pick up a loot item
    /// Returns the loot type if successful, None if already collected
    pub fn try_pickup(&mut self, loot_id: u16) -> Option<LootType> {
        for item in &mut self.items {
            if item.id == loot_id && !item.collected {
                item.collected = true;
                return Some(item.loot_type);
            }
        }
        None
    }

    /// Apply loot effect to a player
    /// Returns the amount healed (for health packs) or quantity (for scrap) or 0 (for effects)
    pub fn apply_effect(
        &mut self,
        player_id: PlayerId,
        loot_type: LootType,
        current_tick: Tick,
    ) -> u32 {
        match loot_type {
            LootType::HealthPack { heal_amount } => {
                // Health packs are instant, no active effect
                heal_amount
            }
            LootType::SpeedBoost {
                multiplier,
                duration_ticks,
            } => {
                // Add speed boost effect
                let duration = if duration_ticks > 0 {
                    duration_ticks
                } else {
                    self.bonus_duration_ticks
                };
                let effect = ActiveEffect::new(
                    EffectType::SpeedBoost { multiplier },
                    Tick(current_tick.0 + duration),
                );
                self.effects.entry(player_id).or_default().push(effect);
                0
            }
            LootType::Scrap { quantity } => {
                // Scrap is added to player inventory (handled by caller)
                // Return quantity so caller knows how much to add
                let _ = player_id; // Unused here, inventory managed elsewhere
                quantity as u32
            }
        }
    }

    /// Process tick - expire effects
    pub fn tick(&mut self, current_tick: Tick) {
        for effects in self.effects.values_mut() {
            effects.retain(|e| !e.is_expired(current_tick));
        }
    }

    /// Get active speed multiplier for a player
    pub fn get_speed_multiplier(&self, player_id: PlayerId) -> f32 {
        self.effects
            .get(&player_id)
            .and_then(|effects| {
                effects.iter().find_map(|e| match e.effect_type {
                    EffectType::SpeedBoost { multiplier } => Some(multiplier),
                })
            })
            .unwrap_or(1.0)
    }

    /// Clear all effects for a player (on elimination)
    pub fn clear_effects(&mut self, player_id: PlayerId) {
        self.effects.remove(&player_id);
    }

    /// Get all active effects for a player
    pub fn get_effects(&self, player_id: PlayerId) -> Option<&Vec<ActiveEffect>> {
        self.effects.get(&player_id)
    }

    /// Check if player has any active effects
    pub fn has_effects(&self, player_id: PlayerId) -> bool {
        self.effects
            .get(&player_id)
            .map(|e| !e.is_empty())
            .unwrap_or(false)
    }

    /// Get count of uncollected loot
    pub fn remaining_count(&self) -> usize {
        self.items.iter().filter(|i| !i.collected).count()
    }

    /// Reset for new match
    pub fn reset(&mut self) {
        self.items.clear();
        self.effects.clear();
        self.next_id = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loot_item_creation() {
        let item = LootItem::new(
            1,
            Vec3::new(10.0, 2.0, 10.0),
            LootType::HealthPack { heal_amount: 25 },
        );
        assert_eq!(item.id, 1);
        assert!(!item.collected);
    }

    #[test]
    fn test_loot_type_protocol() {
        let health = LootType::HealthPack { heal_amount: 50 };
        assert_eq!(health.to_u8(), 0);
        assert_eq!(health.param(), 50);

        let speed = LootType::SpeedBoost {
            multiplier: 1.5,
            duration_ticks: 600,
        };
        assert_eq!(speed.to_u8(), 1);
        assert_eq!(speed.param(), 150);
    }

    #[test]
    fn test_spawn_and_pickup() {
        let mut manager = LootManager::new(600);

        let id = manager.spawn_loot(
            Vec3::new(10.0, 2.0, 10.0),
            LootType::HealthPack { heal_amount: 25 },
        );

        // Player too far away
        assert!(manager.check_pickup(Vec3::new(20.0, 2.0, 20.0)).is_none());

        // Player in range
        let pickup_id = manager.check_pickup(Vec3::new(10.5, 2.0, 10.0));
        assert_eq!(pickup_id, Some(id));

        // Collect it
        let loot_type = manager.try_pickup(id);
        assert!(loot_type.is_some());

        // Can't pick up again
        assert!(manager.try_pickup(id).is_none());
        assert!(manager.check_pickup(Vec3::new(10.0, 2.0, 10.0)).is_none());
    }

    #[test]
    fn test_health_pack_effect() {
        let mut manager = LootManager::new(600);
        let heal = manager.apply_effect(
            PlayerId(1),
            LootType::HealthPack { heal_amount: 30 },
            Tick(100),
        );
        assert_eq!(heal, 30);
        assert!(!manager.has_effects(PlayerId(1)));
    }

    #[test]
    fn test_speed_boost_effect() {
        let mut manager = LootManager::new(600);
        manager.apply_effect(
            PlayerId(1),
            LootType::SpeedBoost {
                multiplier: 1.5,
                duration_ticks: 600,
            },
            Tick(100),
        );

        assert!(manager.has_effects(PlayerId(1)));
        assert_eq!(manager.get_speed_multiplier(PlayerId(1)), 1.5);

        // Expire effect
        manager.tick(Tick(700));
        assert!(!manager.has_effects(PlayerId(1)));
        assert_eq!(manager.get_speed_multiplier(PlayerId(1)), 1.0);
    }

    #[test]
    fn test_clear_effects() {
        let mut manager = LootManager::new(600);
        manager.apply_effect(
            PlayerId(1),
            LootType::SpeedBoost {
                multiplier: 1.5,
                duration_ticks: 600,
            },
            Tick(100),
        );

        manager.clear_effects(PlayerId(1));
        assert!(!manager.has_effects(PlayerId(1)));
    }

    #[test]
    fn test_reset() {
        let mut manager = LootManager::new(600);
        manager.spawn_loot(
            Vec3::new(10.0, 2.0, 10.0),
            LootType::HealthPack { heal_amount: 25 },
        );
        manager.apply_effect(
            PlayerId(1),
            LootType::SpeedBoost {
                multiplier: 1.5,
                duration_ticks: 600,
            },
            Tick(100),
        );

        manager.reset();

        assert_eq!(manager.remaining_count(), 0);
        assert!(!manager.has_effects(PlayerId(1)));
    }

    #[test]
    fn test_scrap_loot_type() {
        let scrap = LootType::Scrap { quantity: 3 };
        assert_eq!(scrap.to_u8(), 2);
        assert_eq!(scrap.param(), 3);
    }

    #[test]
    fn test_scrap_pickup() {
        let mut manager = LootManager::new(600);

        // Spawn scrap loot
        let id = manager.spawn_loot(Vec3::new(5.0, 1.0, 5.0), LootType::Scrap { quantity: 2 });

        // Player in range
        let pickup_id = manager.check_pickup(Vec3::new(5.0, 1.0, 5.5));
        assert_eq!(pickup_id, Some(id));

        // Collect it
        let loot_type = manager.try_pickup(id);
        assert_eq!(loot_type, Some(LootType::Scrap { quantity: 2 }));

        // Apply effect returns quantity
        let mut manager2 = LootManager::new(600);
        let qty = manager2.apply_effect(PlayerId(1), LootType::Scrap { quantity: 5 }, Tick(100));
        assert_eq!(qty, 5);
        // No active effect for scrap
        assert!(!manager2.has_effects(PlayerId(1)));
    }
}
