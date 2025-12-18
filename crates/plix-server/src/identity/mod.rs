//! Server-side identity management
//!
//! This module provides server-side identity functionality:
//! - `NameRegistry`: Manages unique display names with disambiguation
//! - Rate limiting constants for rename operations

mod name_registry;

pub use name_registry::NameRegistry;

/// Rename cooldown in ticks (60 seconds at 60 TPS)
pub const RENAME_COOLDOWN_TICKS: u32 = 3600;
