//! Plix Arena - Arena loading and management
//!
//! This crate handles arena data:
//! - Arena format definitions
//! - Loading arenas from TOML files
//! - Arena validation
//! - Spawn point management

pub mod format;
pub mod loader;
pub mod spawn;
pub mod validate;

pub use format::{Arena, ArenaMetadata, SpawnPoint};
pub use loader::load_arena;
pub use spawn::SpawnManager;
pub use validate::validate_arena;
