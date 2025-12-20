//! Mob system for Adventure Mode
//!
//! This module handles:
//! - Mob spawning from spawn points
//! - Mob AI (FSM: Idle → Aggro → Attack → Return)
//! - Damage tracking for XP/credit distribution
//! - Loot drops on death
//! - Payout calculation with anti-abuse checks

pub mod ai;
pub mod damage;
pub mod instance;
pub mod payout;
pub mod spawn;
pub mod system;

pub use damage::DamageTracker;
pub use instance::MobInstance;
pub use payout::{PayoutCalculator, PayoutResult};
pub use spawn::SpawnManager;
pub use system::{MobSystem, MobSystemConfig};
