//! World Persistence Module for plix-server.
//!
//! Provides file I/O and auto-save functionality for persistent worlds.
//!
//! # Components
//!
//! - `atomic` - Crash-safe atomic file writes
//! - `world_store` - WorldStore for file I/O operations
//! - `scheduler` - SaveScheduler for auto-save management
//!
//! # Usage
//!
//! ```rust,ignore
//! use plix_server::persist::{WorldStore, SaveScheduler, default_worlds_dir};
//! use plix_common::persist::WorldMetadata;
//!
//! // Create or open a world
//! let base_dir = default_worlds_dir();
//! let metadata = WorldMetadata::new_generated("my_world".into(), "My World".into(), 12345);
//! let store = WorldStore::open_or_create(&base_dir, metadata)?;
//!
//! // Set up auto-save
//! let mut scheduler = SaveScheduler::with_defaults();
//!
//! // In game loop:
//! // 1. Mark dirty chunks when blocks change
//! scheduler.mark_dirty(coord);
//!
//! // 2. Call tick each frame (saves if interval elapsed)
//! scheduler.tick(&world, &store)?;
//!
//! // 3. Flush on shutdown
//! scheduler.flush(&world, &mut store)?;
//! ```

pub mod atomic;
pub mod scheduler;
pub mod world_store;

// Re-exports
pub use scheduler::{SaveMetrics, SaveScheduler, SaveSchedulerConfig};
pub use world_store::{default_worlds_dir, delete_world, list_worlds, WorldStore};
