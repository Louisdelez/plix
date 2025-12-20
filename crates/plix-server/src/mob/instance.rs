//! Mob instance runtime state
//!
//! MobInstance represents a live mob in the world, tracking its current
//! state (position, HP, AI state) and damage attribution.

use std::time::Instant;

use plix_common::content::ids::{MobDefId, MobInstanceId, SpawnId};
use plix_common::content::mob::{MobBehavior, MobDefinition};
use plix_common::math::Vec3;
use plix_common::types::PlayerId;

use super::ai::MobAiState;
use super::damage::DamageTracker;

/// A live mob instance in the world.
#[derive(Debug)]
pub struct MobInstance {
    /// Unique runtime ID for this instance
    pub id: MobInstanceId,

    /// Reference to the mob's definition (static data)
    pub def_id: MobDefId,

    /// Spawn point that created this mob
    pub spawn_id: SpawnId,

    /// Original spawn position (for leash calculations)
    pub spawn_position: Vec3,

    /// Current world position
    pub position: Vec3,

    /// Current facing direction (normalized)
    pub facing: Vec3,

    /// Current HP
    pub current_hp: u32,

    /// Maximum HP (from definition, may be modified)
    pub max_hp: u32,

    /// AI state machine
    pub ai_state: MobAiState,

    /// Current movement velocity
    pub velocity: Vec3,

    /// Damage tracking for reward distribution
    pub damage_tracker: DamageTracker,

    /// Time when this mob was spawned
    pub spawn_time: Instant,

    /// Last time this mob was updated
    pub last_update: Instant,

    /// Time of last attack (for cooldown)
    pub last_attack: Option<Instant>,

    /// Current target player (if any)
    pub target: Option<PlayerId>,

    /// Boss phase index (0 = initial, increments as HP drops)
    pub boss_phase: u32,

    /// Whether this mob is marked for removal
    pub pending_removal: bool,
}

impl MobInstance {
    /// Create a new mob instance from a definition.
    pub fn new(
        id: MobInstanceId,
        definition: &MobDefinition,
        spawn_id: SpawnId,
        spawn_position: Vec3,
    ) -> Self {
        let max_hp = definition.stats.hp;

        Self {
            id,
            def_id: definition.mob_id.clone(),
            spawn_id,
            spawn_position,
            position: spawn_position,
            facing: Vec3::new(0.0, 0.0, 1.0),
            current_hp: max_hp,
            max_hp,
            ai_state: MobAiState::Idle,
            velocity: Vec3::ZERO,
            damage_tracker: DamageTracker::new(max_hp),
            spawn_time: Instant::now(),
            last_update: Instant::now(),
            last_attack: None,
            target: None,
            boss_phase: 0,
            pending_removal: false,
        }
    }

    /// Apply damage to this mob from a player.
    /// Returns true if the mob died.
    pub fn take_damage(&mut self, player_id: PlayerId, damage: u32) -> bool {
        self.damage_tracker.record_damage(player_id, damage);
        self.current_hp = self.current_hp.saturating_sub(damage);
        self.current_hp == 0
    }

    /// Check if this mob is dead.
    pub fn is_dead(&self) -> bool {
        self.current_hp == 0
    }

    /// Get the HP percentage (0.0 to 1.0).
    pub fn hp_percent(&self) -> f32 {
        self.current_hp as f32 / self.max_hp as f32
    }

    /// Check if the mob can attack (cooldown expired).
    pub fn can_attack(&self, cooldown_secs: f32) -> bool {
        match self.last_attack {
            None => true,
            Some(last) => {
                let elapsed = self.last_update.duration_since(last);
                elapsed.as_secs_f32() >= cooldown_secs
            }
        }
    }

    /// Record an attack (updates cooldown timer).
    pub fn record_attack(&mut self) {
        self.last_attack = Some(Instant::now());
    }

    /// Calculate distance from current position to spawn point.
    pub fn distance_from_spawn(&self) -> f32 {
        (self.position - self.spawn_position).length()
    }

    /// Calculate distance to a target position.
    pub fn distance_to(&self, target: Vec3) -> f32 {
        (self.position - target).length()
    }

    /// Update facing direction to look at a target.
    pub fn look_at(&mut self, target: Vec3) {
        let dir = target - self.position;
        if dir.length_squared() > 0.0001 {
            self.facing = dir.normalize();
        }
    }

    /// Move towards a target at the given speed.
    /// Returns true if the target was reached.
    pub fn move_towards(&mut self, target: Vec3, speed: f32, dt: f32) -> bool {
        let to_target = target - self.position;
        let dist = to_target.length();

        if dist < 0.1 {
            self.velocity = Vec3::ZERO;
            return true;
        }

        let dir = to_target / dist;
        let move_dist = (speed * dt).min(dist);

        self.position = self.position + dir * move_dist;
        self.velocity = dir * speed;
        self.facing = dir;

        dist <= move_dist
    }

    /// Check and update boss phase based on current HP.
    /// Returns the new phase index if it changed.
    pub fn update_boss_phase(&mut self, phases: &[plix_common::content::mob::BossPhase]) -> Option<u32> {
        let hp_percent = self.hp_percent();

        // Find the highest phase index where HP is below threshold
        for (i, phase) in phases.iter().enumerate() {
            if hp_percent <= phase.hp_threshold && self.boss_phase < (i + 1) as u32 {
                self.boss_phase = (i + 1) as u32;
                return Some(self.boss_phase);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::content::ids::LootTableId;
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
            xp_reward: 50,
            credit_reward: 25,
            tags: vec![],
        }
    }

    #[test]
    fn test_new_instance() {
        let def = test_definition();
        let instance = MobInstance::new(
            MobInstanceId::new(1),
            &def,
            SpawnId::new("spawn_1"),
            Vec3::new(10.0, 0.0, 10.0),
        );

        assert_eq!(instance.current_hp, 100);
        assert_eq!(instance.max_hp, 100);
        assert!(!instance.is_dead());
    }

    #[test]
    fn test_take_damage() {
        let def = test_definition();
        let mut instance = MobInstance::new(
            MobInstanceId::new(1),
            &def,
            SpawnId::new("spawn_1"),
            Vec3::ZERO,
        );

        let died = instance.take_damage(PlayerId(1), 30);
        assert!(!died);
        assert_eq!(instance.current_hp, 70);

        let died = instance.take_damage(PlayerId(1), 70);
        assert!(died);
        assert!(instance.is_dead());
    }

    #[test]
    fn test_hp_percent() {
        let def = test_definition();
        let mut instance = MobInstance::new(
            MobInstanceId::new(1),
            &def,
            SpawnId::new("spawn_1"),
            Vec3::ZERO,
        );

        assert_eq!(instance.hp_percent(), 1.0);

        instance.take_damage(PlayerId(1), 50);
        assert_eq!(instance.hp_percent(), 0.5);
    }

    #[test]
    fn test_distance_calculations() {
        let def = test_definition();
        let mut instance = MobInstance::new(
            MobInstanceId::new(1),
            &def,
            SpawnId::new("spawn_1"),
            Vec3::new(0.0, 0.0, 0.0),
        );

        instance.position = Vec3::new(3.0, 0.0, 4.0);
        assert_eq!(instance.distance_from_spawn(), 5.0);
        assert_eq!(instance.distance_to(Vec3::new(3.0, 0.0, 9.0)), 5.0);
    }

    #[test]
    fn test_move_towards() {
        let def = test_definition();
        let mut instance = MobInstance::new(
            MobInstanceId::new(1),
            &def,
            SpawnId::new("spawn_1"),
            Vec3::new(0.0, 0.0, 0.0),
        );

        let target = Vec3::new(10.0, 0.0, 0.0);
        let reached = instance.move_towards(target, 5.0, 1.0);
        assert!(!reached);
        assert_eq!(instance.position.x, 5.0);

        // Move rest of the way
        let reached = instance.move_towards(target, 10.0, 1.0);
        assert!(reached);
    }
}
