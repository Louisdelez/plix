//! Server-side inventory system implementation.
//!
//! This module handles all server-authoritative inventory logic including:
//! - Item definitions and registry
//! - Player hotbar management
//! - Item pickup processing
//! - Item usage effects
//! - Inventory replication to clients

pub mod config;
pub mod item_registry;
pub mod pickup_system;
pub mod use_system;

pub use config::{get_starting_loadout, give_starting_loadout, InventoryConfig};
pub use item_registry::{
    get_heal_amount, get_item_def, get_max_stack, get_weapon_damage, ITEM_DEFS,
};
pub use pickup_system::{has_loot_in_range, try_pickup, PickupResult};
pub use use_system::{decrement_active_consumable, use_active_item, UseResult};
