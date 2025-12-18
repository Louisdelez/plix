//! Weapon definition constants.
//!
//! Static weapon parameters used by the combat system.

use plix_common::inventory::WeaponType;

/// Static definition of weapon behavior and parameters.
#[derive(Debug, Clone, Copy)]
pub struct WeaponDef {
    /// Weapon category (melee or ranged)
    pub weapon_type: WeaponType,
    /// Damage dealt per hit
    pub damage: u16,
    /// Cooldown duration in ticks (60 TPS)
    pub cooldown_ticks: u32,
    /// Melee: cone half-angle in degrees
    pub cone_angle_deg: f32,
    /// Melee: maximum reach distance in blocks
    pub reach_blocks: f32,
    /// Ranged: projectile speed in blocks per second
    pub projectile_speed: f32,
    /// Ranged: projectile lifetime in ticks
    pub projectile_ttl_ticks: u32,
    /// Ranged: base spread angle in degrees
    pub base_spread_deg: f32,
    /// Ranged: additional spread when moving (multiplier)
    pub movement_spread_mult: f32,
    /// Ranged: recoil added per shot in degrees
    pub recoil_per_shot_deg: f32,
    /// Ranged: maximum recoil accumulation in degrees
    pub max_recoil_deg: f32,
    /// Ranged: recoil recovery window in ticks
    pub recoil_recovery_ticks: u32,
}

/// Sword weapon definition.
/// - 25 damage per hit
/// - 0.6s cooldown (36 ticks at 60 TPS)
/// - 60 degree cone (30 degree half-angle)
/// - 2.5 block reach
pub static SWORD_DEF: WeaponDef = WeaponDef {
    weapon_type: WeaponType::Melee,
    damage: 25,
    cooldown_ticks: 36,
    cone_angle_deg: 30.0, // Half-angle
    reach_blocks: 2.5,
    projectile_speed: 0.0,
    projectile_ttl_ticks: 0,
    base_spread_deg: 0.0,
    movement_spread_mult: 0.0,
    recoil_per_shot_deg: 0.0,
    max_recoil_deg: 0.0,
    recoil_recovery_ticks: 0,
};

/// Fist (default melee) definition.
/// - 5 damage per hit
/// - 0.3s cooldown (18 ticks at 60 TPS)
/// - 60 degree cone (30 degree half-angle)
/// - 1.5 block reach
pub static FIST_DEF: WeaponDef = WeaponDef {
    weapon_type: WeaponType::Melee,
    damage: 5,
    cooldown_ticks: 18,
    cone_angle_deg: 30.0, // Half-angle
    reach_blocks: 1.5,
    projectile_speed: 0.0,
    projectile_ttl_ticks: 0,
    base_spread_deg: 0.0,
    movement_spread_mult: 0.0,
    recoil_per_shot_deg: 0.0,
    max_recoil_deg: 0.0,
    recoil_recovery_ticks: 0,
};

/// Bow weapon definition.
/// - 15 damage per hit
/// - 0.8s cooldown (48 ticks at 60 TPS)
/// - 30 blocks/second projectile speed
/// - 3 second lifetime (180 ticks)
/// - ±2 degree base spread
/// - +50% spread when moving
/// - +1 degree recoil per shot
/// - +5 degree max recoil
/// - 0.5s recovery window (30 ticks)
pub static BOW_DEF: WeaponDef = WeaponDef {
    weapon_type: WeaponType::Ranged,
    damage: 15,
    cooldown_ticks: 48,
    cone_angle_deg: 0.0,
    reach_blocks: 0.0,
    projectile_speed: 30.0,
    projectile_ttl_ticks: 180,
    base_spread_deg: 2.0,
    movement_spread_mult: 1.5, // +50% = 1.5x multiplier
    recoil_per_shot_deg: 1.0,
    max_recoil_deg: 5.0,
    recoil_recovery_ticks: 30,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sword_def_values() {
        assert_eq!(SWORD_DEF.damage, 25);
        assert_eq!(SWORD_DEF.cooldown_ticks, 36);
        assert_eq!(SWORD_DEF.cone_angle_deg, 30.0);
        assert_eq!(SWORD_DEF.reach_blocks, 2.5);
        assert_eq!(SWORD_DEF.weapon_type, WeaponType::Melee);
    }

    #[test]
    fn test_bow_def_values() {
        assert_eq!(BOW_DEF.damage, 15);
        assert_eq!(BOW_DEF.cooldown_ticks, 48);
        assert_eq!(BOW_DEF.projectile_speed, 30.0);
        assert_eq!(BOW_DEF.projectile_ttl_ticks, 180);
        assert_eq!(BOW_DEF.base_spread_deg, 2.0);
        assert_eq!(BOW_DEF.movement_spread_mult, 1.5);
        assert_eq!(BOW_DEF.recoil_per_shot_deg, 1.0);
        assert_eq!(BOW_DEF.max_recoil_deg, 5.0);
        assert_eq!(BOW_DEF.recoil_recovery_ticks, 30);
        assert_eq!(BOW_DEF.weapon_type, WeaponType::Ranged);
    }

    #[test]
    fn test_fist_def_values() {
        assert_eq!(FIST_DEF.damage, 5);
        assert_eq!(FIST_DEF.cooldown_ticks, 18);
        assert_eq!(FIST_DEF.reach_blocks, 1.5);
        assert_eq!(FIST_DEF.weapon_type, WeaponType::Melee);
    }
}
