//! Damage tracking for mobs
//!
//! Tracks damage attribution for XP/credit distribution when a mob dies.
//! Implements anti-abuse checks: must have dealt ≥5% of total damage OR
//! hit within last 10 seconds to be eligible for rewards.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use plix_common::types::PlayerId;

/// Minimum damage percentage to be eligible for rewards
const MIN_DAMAGE_PERCENT: f32 = 0.05;

/// Maximum time since last hit to be eligible (even with low damage)
const MAX_HIT_RECENCY: Duration = Duration::from_secs(10);

/// Tracks damage dealt to a mob by each player.
#[derive(Debug, Clone)]
pub struct DamageTracker {
    /// Damage dealt by each player
    damage_by_player: HashMap<PlayerId, DamageRecord>,
    /// Total damage dealt (may exceed max HP due to overkill)
    total_damage: u32,
    /// Max HP of the mob (for percentage calculations)
    max_hp: u32,
}

/// Record of damage dealt by a single player.
#[derive(Debug, Clone)]
pub struct DamageRecord {
    /// Total damage dealt
    pub damage: u32,
    /// Last time this player hit the mob
    pub last_hit: Instant,
    /// Number of hits
    pub hit_count: u32,
}

impl DamageTracker {
    /// Create a new damage tracker for a mob with the given max HP.
    pub fn new(max_hp: u32) -> Self {
        Self {
            damage_by_player: HashMap::new(),
            total_damage: 0,
            max_hp,
        }
    }

    /// Record damage dealt by a player.
    pub fn record_damage(&mut self, player_id: PlayerId, damage: u32) {
        self.total_damage = self.total_damage.saturating_add(damage);

        let record = self.damage_by_player.entry(player_id).or_insert(DamageRecord {
            damage: 0,
            last_hit: Instant::now(),
            hit_count: 0,
        });

        record.damage = record.damage.saturating_add(damage);
        record.last_hit = Instant::now();
        record.hit_count += 1;
    }

    /// Get the player who dealt the killing blow (most recent hit).
    /// Returns None if no damage was recorded.
    pub fn killer(&self) -> Option<PlayerId> {
        self.damage_by_player
            .iter()
            .max_by_key(|(_, record)| record.last_hit)
            .map(|(id, _)| *id)
    }

    /// Get all players eligible for rewards based on anti-abuse rules.
    /// Returns (player_id, damage_dealt) for eligible players.
    pub fn eligible_players(&self, now: Instant) -> Vec<(PlayerId, u32)> {
        self.damage_by_player
            .iter()
            .filter(|(_, record)| self.is_eligible(record, now))
            .map(|(id, record)| (*id, record.damage))
            .collect()
    }

    /// Check if a player's contribution meets eligibility criteria.
    fn is_eligible(&self, record: &DamageRecord, now: Instant) -> bool {
        // Condition 1: Dealt at least 5% of max HP
        let damage_percent = record.damage as f32 / self.max_hp as f32;
        if damage_percent >= MIN_DAMAGE_PERCENT {
            return true;
        }

        // Condition 2: Hit within last 10 seconds
        let time_since_hit = now.duration_since(record.last_hit);
        if time_since_hit <= MAX_HIT_RECENCY {
            return true;
        }

        false
    }

    /// Get the total damage dealt by all players.
    pub fn total_damage(&self) -> u32 {
        self.total_damage
    }

    /// Get the damage record for a specific player.
    pub fn get_player_damage(&self, player_id: PlayerId) -> Option<&DamageRecord> {
        self.damage_by_player.get(&player_id)
    }

    /// Get damage percentage for a player (capped at 100%).
    pub fn damage_percent(&self, player_id: PlayerId) -> f32 {
        self.damage_by_player
            .get(&player_id)
            .map(|record| (record.damage as f32 / self.max_hp as f32).min(1.0))
            .unwrap_or(0.0)
    }

    /// Get iterator over all damage records.
    pub fn iter(&self) -> impl Iterator<Item = (&PlayerId, &DamageRecord)> {
        self.damage_by_player.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(id: u64) -> PlayerId {
        PlayerId(id)
    }

    #[test]
    fn test_record_damage() {
        let mut tracker = DamageTracker::new(100);
        tracker.record_damage(player(1), 30);
        tracker.record_damage(player(2), 20);
        tracker.record_damage(player(1), 10); // Player 1 hits again

        assert_eq!(tracker.total_damage(), 60);
        assert_eq!(tracker.get_player_damage(player(1)).unwrap().damage, 40);
        assert_eq!(tracker.get_player_damage(player(2)).unwrap().damage, 20);
    }

    #[test]
    fn test_killer_is_most_recent() {
        let mut tracker = DamageTracker::new(100);
        tracker.record_damage(player(1), 90); // First hit
        tracker.record_damage(player(2), 10); // Last hit

        assert_eq!(tracker.killer(), Some(player(2)));
    }

    #[test]
    fn test_eligibility_by_damage_percent() {
        let mut tracker = DamageTracker::new(100);
        tracker.record_damage(player(1), 10); // 10% >= 5%
        tracker.record_damage(player(2), 4);  // 4% < 5%

        // Wait for recency window to expire
        let future = Instant::now() + Duration::from_secs(15);
        let eligible = tracker.eligible_players(future);

        assert_eq!(eligible.len(), 1);
        assert_eq!(eligible[0].0, player(1));
    }

    #[test]
    fn test_eligibility_by_recency() {
        let mut tracker = DamageTracker::new(100);
        tracker.record_damage(player(1), 2); // Only 2% damage

        // Within recency window
        let now = Instant::now();
        let eligible = tracker.eligible_players(now);

        assert_eq!(eligible.len(), 1);
        assert_eq!(eligible[0].0, player(1));
    }

    #[test]
    fn test_damage_percent_capped() {
        let mut tracker = DamageTracker::new(100);
        tracker.record_damage(player(1), 150); // 150% overkill

        assert_eq!(tracker.damage_percent(player(1)), 1.0);
    }

    #[test]
    fn test_hit_count() {
        let mut tracker = DamageTracker::new(100);
        tracker.record_damage(player(1), 10);
        tracker.record_damage(player(1), 10);
        tracker.record_damage(player(1), 10);

        assert_eq!(tracker.get_player_damage(player(1)).unwrap().hit_count, 3);
    }
}
