//! World Persistence API Contracts
//!
//! This module defines the public APIs for world persistence.
//! See individual files for detailed documentation.
//!
//! # Module Structure
//!
//! - `world_store.rs` - WorldStore: File I/O operations
//! - `save_scheduler.rs` - SaveScheduler: Auto-save management
//! - `chunk_codec.rs` - ChunkCodec: Chunk serialization
//!
//! # Implementation Locations
//!
//! - Core types: `crates/plix-common/src/persist/`
//! - Server integration: `crates/plix-server/src/persist/`
//! - Client integration: `crates/plix-client/src/persist/`

pub mod chunk_codec;
pub mod save_scheduler;
pub mod world_store;

// Re-exports for convenience
pub use chunk_codec::{ChunkCodec, ChunkData, CHUNK_FORMAT_VERSION};
pub use save_scheduler::{SaveMetrics, SaveScheduler, SaveSchedulerConfig};
pub use world_store::{default_worlds_dir, WorldStore};
