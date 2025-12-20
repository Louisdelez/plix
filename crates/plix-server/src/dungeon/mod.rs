//! Dungeon system for Adventure Mode
//!
//! This module handles:
//! - Dungeon state tracking (boss alive, chest available)
//! - Shared-world boss with respawn timer
//! - One-time reward chest per player per run
//! - Integration with quest DungeonClear steps

pub mod chest;
pub mod state;
pub mod system;

pub use chest::{ChestEntity, ChestInteractResult};
pub use state::{DungeonPhase, DungeonState};
pub use system::{DungeonEvent, DungeonSystem, DungeonSystemConfig};
