//! Inventory system types shared between client and server.
//!
//! This module provides the core data structures for the hotbar-based
//! inventory system including items, stacks, and the hotbar container.

pub mod hotbar;
pub mod item;
pub mod item_stack;

pub use hotbar::{Hotbar, DEFAULT_HOTBAR_SIZE};
pub use item::{ItemDef, ItemKind, WeaponType};
pub use item_stack::ItemStack;

// Re-export ItemId from types for convenience
pub use crate::types::ItemId;
