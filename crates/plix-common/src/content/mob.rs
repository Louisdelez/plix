//! Mob definitions with stats and behaviors

use serde::{Deserialize, Serialize};

use super::ids::{LootTableId, MobDefId};
use crate::math::Vec3;

/// Enemy entity definition with stats, behavior, and loot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobDefinition {
    /// Unique stable identifier (e.g., "cave_rat")
    pub mob_id: MobDefId,

    /// Display name shown to players
    pub display_name: String,

    /// Combat stats
    pub stats: MobStats,

    /// AI behavior type
    pub behavior: MobBehavior,

    /// Loot table ID for drops
    pub loot_table: LootTableId,

    /// Base XP reward (distributed proportionally among contributors)
    pub xp_reward: u32,

    /// Base credit reward (distributed proportionally among contributors)
    #[serde(default)]
    pub credit_reward: u32,

    /// Tags for filtering/grouping (e.g., "beast", "undead", "boss")
    #[serde(default)]
    pub tags: Vec<String>,
}

impl MobDefinition {
    /// Check if mob has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Check if this is a boss mob
    pub fn is_boss(&self) -> bool {
        matches!(self.behavior, MobBehavior::Boss { .. }) || self.has_tag("boss")
    }
}

/// Combat and movement stats for a mob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobStats {
    /// Maximum health points
    pub hp: u32,

    /// Damage per attack
    pub damage: u32,

    /// Movement speed in blocks per second
    pub speed: f32,

    /// Armor value (damage reduction)
    #[serde(default)]
    pub armor: u32,

    /// Detection radius for aggro (blocks)
    pub aggro_radius: f32,

    /// Maximum distance from spawn before returning (blocks)
    pub leash_radius: f32,

    /// Melee attack range (blocks)
    pub attack_range: f32,

    /// Cooldown between attacks (seconds)
    pub attack_cooldown: f32,
}

impl MobStats {
    /// Validate stats have sensible values
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.hp == 0 {
            return Err("HP must be greater than 0");
        }
        if self.speed <= 0.0 {
            return Err("Speed must be greater than 0");
        }
        if self.aggro_radius <= 0.0 {
            return Err("Aggro radius must be greater than 0");
        }
        if self.leash_radius < self.aggro_radius {
            return Err("Leash radius must be >= aggro radius");
        }
        if self.attack_range <= 0.0 {
            return Err("Attack range must be greater than 0");
        }
        if self.attack_cooldown <= 0.0 {
            return Err("Attack cooldown must be greater than 0");
        }
        Ok(())
    }
}

/// AI behavior types for mobs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MobBehavior {
    /// Basic melee aggro - target closest player in range
    Aggro,

    /// Patrol between waypoints, aggro on proximity
    Patrol {
        /// Ordered list of world positions to patrol between
        waypoints: Vec<Vec3>,
    },

    /// Ranged attacks, maintain preferred distance
    Ranged {
        /// Preferred combat distance (blocks)
        preferred_range: f32,
    },

    /// Boss with HP-based phase transitions
    Boss {
        /// Ordered phases (checked in order, highest HP threshold first)
        phases: Vec<BossPhase>,
    },
}

/// A boss phase that activates below a certain HP threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossPhase {
    /// HP percentage threshold to enter this phase (0.0 to 1.0, e.g., 0.5 for 50%)
    pub hp_threshold: f32,

    /// Modified behavior in this phase
    pub behavior_modifier: BossPhaseModifier,
}

/// Behavior modifications applied during a boss phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BossPhaseModifier {
    /// Increase attack speed
    Enrage {
        /// Multiplier for attack speed (e.g., 1.5 for 50% faster)
        attack_speed_multiplier: f32,
    },

    /// Spawn additional mobs
    SummonMinions {
        /// Mob type to summon
        mob_id: MobDefId,
        /// Number of minions to spawn
        count: u32,
    },

    /// Area damage attack
    AreaAttack {
        /// Radius of the area attack
        radius: f32,
        /// Damage dealt
        damage: u32,
        /// Cooldown between area attacks
        cooldown: f32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mob() -> MobDefinition {
        MobDefinition {
            mob_id: MobDefId::new("cave_rat"),
            display_name: "Cave Rat".to_string(),
            stats: MobStats {
                hp: 20,
                damage: 5,
                speed: 4.0,
                armor: 0,
                aggro_radius: 8.0,
                leash_radius: 20.0,
                attack_range: 1.5,
                attack_cooldown: 1.0,
            },
            behavior: MobBehavior::Aggro,
            loot_table: LootTableId::new("cave_rat_drops"),
            xp_reward: 10,
            credit_reward: 5,
            tags: vec!["beast".to_string(), "trash".to_string()],
        }
    }

    #[test]
    fn test_has_tag() {
        let mob = sample_mob();
        assert!(mob.has_tag("beast"));
        assert!(!mob.has_tag("undead"));
    }

    #[test]
    fn test_is_boss() {
        let rat = sample_mob();
        assert!(!rat.is_boss());

        let boss = MobDefinition {
            behavior: MobBehavior::Boss { phases: vec![] },
            ..sample_mob()
        };
        assert!(boss.is_boss());
    }

    #[test]
    fn test_stats_validation() {
        let valid = MobStats {
            hp: 100,
            damage: 10,
            speed: 5.0,
            armor: 5,
            aggro_radius: 10.0,
            leash_radius: 25.0,
            attack_range: 2.0,
            attack_cooldown: 1.0,
        };
        assert!(valid.validate().is_ok());

        let zero_hp = MobStats { hp: 0, ..valid };
        assert!(zero_hp.validate().is_err());

        let bad_leash = MobStats {
            leash_radius: 5.0, // Less than aggro_radius
            ..valid
        };
        assert!(bad_leash.validate().is_err());
    }

    #[test]
    fn test_toml_deserialization() {
        let toml_str = r#"
            mob_id = "cave_rat"
            display_name = "Cave Rat"
            loot_table = "cave_rat_drops"
            xp_reward = 10
            credit_reward = 5
            tags = ["beast", "trash"]

            [stats]
            hp = 20
            damage = 5
            speed = 4.0
            armor = 0
            aggro_radius = 8.0
            leash_radius = 20.0
            attack_range = 1.5
            attack_cooldown = 1.0

            [behavior]
            type = "Aggro"
        "#;

        let mob: MobDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(mob.mob_id.as_str(), "cave_rat");
        assert_eq!(mob.stats.hp, 20);
    }

    #[test]
    fn test_boss_phases_toml() {
        let toml_str = r#"
            mob_id = "gate_warden"
            display_name = "Gate Warden"
            loot_table = "gate_warden_drops"
            xp_reward = 500
            tags = ["boss", "undead"]

            [stats]
            hp = 500
            damage = 25
            speed = 3.0
            aggro_radius = 15.0
            leash_radius = 30.0
            attack_range = 2.5
            attack_cooldown = 2.0

            [behavior]
            type = "Boss"

            [[behavior.phases]]
            hp_threshold = 0.5

            [behavior.phases.behavior_modifier]
            type = "Enrage"
            attack_speed_multiplier = 1.5

            [[behavior.phases]]
            hp_threshold = 0.25

            [behavior.phases.behavior_modifier]
            type = "SummonMinions"
            mob_id = "skeleton"
            count = 3
        "#;

        let mob: MobDefinition = toml::from_str(toml_str).unwrap();
        assert!(mob.is_boss());

        if let MobBehavior::Boss { phases } = &mob.behavior {
            assert_eq!(phases.len(), 2);
            assert_eq!(phases[0].hp_threshold, 0.5);
        } else {
            panic!("Expected Boss behavior");
        }
    }
}
