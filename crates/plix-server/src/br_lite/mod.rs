//! BR Lite (Battle Royale) game mode implementation
//!
//! This module implements a simplified Battle Royale mode with:
//! - Shrinking circular safe zone
//! - Permanent elimination (no respawn)
//! - Minimal loot (health packs, speed boosts)
//! - Last player standing wins

pub mod config;
pub mod coordinator;
pub mod damage;
pub mod loot;
pub mod state;
pub mod zone;

pub use config::{BrLiteConfig, ZonePhase};
pub use coordinator::{BrLiteCoordinator, BrLiteEvent, MatchPhase};
pub use damage::DamageController;
pub use loot::{ActiveEffect, EffectType, LootItem, LootManager, LootType};
pub use state::{BrLiteState, PhaseMode, PlayerBrState, ZoneState};
pub use zone::ZoneController;
