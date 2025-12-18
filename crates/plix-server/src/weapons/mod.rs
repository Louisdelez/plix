//! Weapons system for combat mechanics.
//!
//! This module provides the server-side weapons implementation including:
//! - Weapon definitions (damage, cooldown, behavior)
//! - Melee combat with cone hit detection
//! - Ranged combat with projectile spawning
//! - Cooldown tracking per-player per-weapon
//! - Recoil and accuracy mechanics
//! - Projectile management with collision detection

pub mod cooldown;
pub mod defs;
pub mod melee;
pub mod projectiles;
pub mod ranged;
pub mod recoil;

pub use cooldown::CooldownState;
pub use defs::{WeaponDef, BOW_DEF, FIST_DEF, SWORD_DEF};
pub use melee::MeleeSystem;
pub use projectiles::ProjectileManager;
pub use ranged::RangedSystem;
pub use recoil::RecoilState;

use plix_common::types::ItemId;

/// Combined weapon state for a player
#[derive(Debug, Clone, Default)]
pub struct PlayerWeaponState {
    /// Per-weapon cooldown tracking
    pub cooldowns: CooldownState,
    /// Recoil accumulation and decay
    pub recoil: RecoilState,
}

/// Result of attempting to use a weapon
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeaponUseResult {
    /// Melee attack succeeded, damage dealt
    MeleeAttack { damage: u16 },
    /// Ranged shot succeeded, projectile spawned
    RangedShot { projectile_spawned: bool },
    /// Attack rejected due to cooldown
    Cooldown { remaining_ticks: u32 },
    /// Ranged shot rejected due to projectile limit
    ProjectileLimitReached,
    /// Item is not a weapon
    NotAWeapon,
}

/// Get the weapon definition for an item, if it is a weapon
pub fn get_weapon_def(item_id: ItemId) -> Option<&'static WeaponDef> {
    match item_id {
        ItemId::SWORD => Some(&SWORD_DEF),
        ItemId::BOW => Some(&BOW_DEF),
        ItemId::NONE => Some(&FIST_DEF), // Empty hand = fist
        _ => None,
    }
}
