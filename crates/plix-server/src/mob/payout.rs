//! Payout calculation for mob kills
//!
//! Calculates XP and credit distribution based on:
//! - Proportional damage dealt (capped at 100%)
//! - Killer bonus (10-25% of base reward)
//! - Anti-abuse eligibility (≥5% damage OR hit within 10s)

use std::time::Instant;

use plix_common::content::mob::MobDefinition;
use plix_common::types::PlayerId;

use super::damage::DamageTracker;

/// Killer bonus range as percentage of base reward.
const KILLER_BONUS_MIN: f32 = 0.10;
const KILLER_BONUS_MAX: f32 = 0.25;

/// Result of payout calculation for a single player.
#[derive(Debug, Clone, Copy)]
pub struct PlayerPayout {
    /// Player receiving the payout
    pub player_id: PlayerId,
    /// XP awarded
    pub xp: u32,
    /// Credits awarded
    pub credits: u32,
    /// Whether this player got the kill
    pub is_killer: bool,
}

/// Result of calculating payouts for a mob death.
#[derive(Debug, Clone)]
pub struct PayoutResult {
    /// Individual payouts for each eligible player
    pub payouts: Vec<PlayerPayout>,
    /// Total XP distributed
    pub total_xp: u32,
    /// Total credits distributed
    pub total_credits: u32,
}

impl PayoutResult {
    /// Create an empty payout result.
    pub fn empty() -> Self {
        Self {
            payouts: Vec::new(),
            total_xp: 0,
            total_credits: 0,
        }
    }
}

/// Calculator for mob kill payouts.
pub struct PayoutCalculator {
    /// Killer bonus percentage (randomized within range)
    killer_bonus: f32,
}

impl PayoutCalculator {
    /// Create a new payout calculator with the default killer bonus.
    pub fn new() -> Self {
        // Use middle of range for deterministic tests
        Self {
            killer_bonus: (KILLER_BONUS_MIN + KILLER_BONUS_MAX) / 2.0,
        }
    }

    /// Create with a specific killer bonus percentage (for testing).
    pub fn with_killer_bonus(bonus: f32) -> Self {
        Self {
            killer_bonus: bonus.clamp(KILLER_BONUS_MIN, KILLER_BONUS_MAX),
        }
    }

    /// Calculate payouts for a mob death.
    pub fn calculate(
        &self,
        definition: &MobDefinition,
        damage_tracker: &DamageTracker,
        now: Instant,
    ) -> PayoutResult {
        let eligible = damage_tracker.eligible_players(now);
        if eligible.is_empty() {
            return PayoutResult::empty();
        }

        let killer_id = damage_tracker.killer();
        let base_xp = definition.xp_reward;
        let base_credits = definition.credit_reward;

        // Calculate proportional payouts
        let mut payouts = Vec::with_capacity(eligible.len());
        let mut total_xp = 0u32;
        let mut total_credits = 0u32;

        for (player_id, damage) in &eligible {
            let damage_percent = (*damage as f32 / definition.stats.hp as f32).min(1.0);
            let is_killer = Some(*player_id) == killer_id;

            // Base payout proportional to damage
            let mut xp = (base_xp as f32 * damage_percent) as u32;
            let mut credits = (base_credits as f32 * damage_percent) as u32;

            // Add killer bonus
            if is_killer {
                xp += (base_xp as f32 * self.killer_bonus) as u32;
                credits += (base_credits as f32 * self.killer_bonus) as u32;
            }

            // Ensure at least 1 XP if any damage was dealt
            if xp == 0 && damage > &0 {
                xp = 1;
            }

            total_xp += xp;
            total_credits += credits;

            payouts.push(PlayerPayout {
                player_id: *player_id,
                xp,
                credits,
                is_killer,
            });
        }

        PayoutResult {
            payouts,
            total_xp,
            total_credits,
        }
    }
}

impl Default for PayoutCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::content::ids::{LootTableId, MobDefId};
    use plix_common::content::mob::{MobBehavior, MobStats};

    fn test_definition() -> MobDefinition {
        MobDefinition {
            mob_id: MobDefId::new("test_mob"),
            display_name: "Test Mob".to_string(),
            stats: MobStats {
                hp: 100,
                damage: 10,
                speed: 5.0,
                armor: 0,
                aggro_radius: 10.0,
                leash_radius: 25.0,
                attack_range: 2.0,
                attack_cooldown: 1.0,
            },
            behavior: MobBehavior::Aggro,
            loot_table: LootTableId::new("test_drops"),
            xp_reward: 100,
            credit_reward: 50,
            tags: vec![],
        }
    }

    #[test]
    fn test_single_player_full_damage() {
        let def = test_definition();
        let mut tracker = DamageTracker::new(100);
        tracker.record_damage(PlayerId(1), 100);

        let calc = PayoutCalculator::with_killer_bonus(0.20);
        let result = calc.calculate(&def, &tracker, Instant::now());

        assert_eq!(result.payouts.len(), 1);
        let payout = &result.payouts[0];
        assert!(payout.is_killer);
        // 100% damage = 100 XP + 20% killer bonus = 120 XP
        assert_eq!(payout.xp, 120);
        // 100% damage = 50 credits + 20% killer bonus = 60 credits
        assert_eq!(payout.credits, 60);
    }

    #[test]
    fn test_two_players_split() {
        let def = test_definition();
        let mut tracker = DamageTracker::new(100);
        tracker.record_damage(PlayerId(1), 60);
        tracker.record_damage(PlayerId(2), 40);

        let calc = PayoutCalculator::with_killer_bonus(0.20);
        let result = calc.calculate(&def, &tracker, Instant::now());

        assert_eq!(result.payouts.len(), 2);

        // Player 2 got the last hit (killer)
        let p1 = result.payouts.iter().find(|p| p.player_id == PlayerId(1)).unwrap();
        let p2 = result.payouts.iter().find(|p| p.player_id == PlayerId(2)).unwrap();

        assert!(!p1.is_killer);
        assert!(p2.is_killer);

        // P1: 60% damage = 60 XP, no bonus
        assert_eq!(p1.xp, 60);
        // P2: 40% damage = 40 XP + 20% bonus = 60 XP
        assert_eq!(p2.xp, 60);
    }

    #[test]
    fn test_no_eligible_players() {
        let def = test_definition();
        let tracker = DamageTracker::new(100); // No damage recorded

        let calc = PayoutCalculator::new();
        let result = calc.calculate(&def, &tracker, Instant::now());

        assert!(result.payouts.is_empty());
        assert_eq!(result.total_xp, 0);
    }

    #[test]
    fn test_minimum_xp() {
        let def = test_definition();
        let mut tracker = DamageTracker::new(100);
        tracker.record_damage(PlayerId(1), 1); // Tiny damage

        let calc = PayoutCalculator::with_killer_bonus(0.10);
        let result = calc.calculate(&def, &tracker, Instant::now());

        // Should get at least 1 XP
        assert!(result.payouts[0].xp >= 1);
    }

    #[test]
    fn test_overkill_capped() {
        let def = test_definition();
        let mut tracker = DamageTracker::new(100);
        tracker.record_damage(PlayerId(1), 200); // 200% overkill

        let calc = PayoutCalculator::with_killer_bonus(0.20);
        let result = calc.calculate(&def, &tracker, Instant::now());

        // Damage is capped at 100%
        // 100 XP base + 20 bonus = 120
        assert_eq!(result.payouts[0].xp, 120);
    }
}
