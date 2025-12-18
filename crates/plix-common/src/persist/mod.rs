//! World Persistence Module
//!
//! This module provides types and utilities for persisting voxel worlds to disk.
//!
//! # Module Structure
//!
//! - `error` - Error types for persistence operations
//! - `version` - Version constants and compatibility checking
//! - `world_meta` - World metadata (stored in `meta.bin`)
//! - `chunk_codec` - Chunk serialization/deserialization
//!
//! # File Format
//!
//! Worlds are stored as directories with the following structure:
//!
//! ```text
//! ~/.local/share/plix/worlds/<world_id>/
//! ├── meta.bin           # WorldMetadata (bincode)
//! └── chunks/
//!     ├── 0_0_0.bin      # ChunkData for coord (0,0,0)
//!     ├── -1_0_5.bin     # ChunkData for coord (-1,0,5)
//!     └── ...
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use plix_common::persist::{WorldMetadata, ChunkCodec, chunk_filename};
//!
//! // Create metadata for a new world
//! let meta = WorldMetadata::new_generated("my_world".to_string(), "My World".to_string(), 12345);
//!
//! // Encode a chunk
//! let bytes = ChunkCodec::encode(&chunk)?;
//!
//! // Get filename for chunk
//! let filename = chunk_filename(coord);  // e.g., "0_0_0.bin"
//! ```

pub mod chunk_codec;
pub mod error;
pub mod version;
pub mod world_meta;

// Re-export main types
pub use chunk_codec::{
    chunk_filename, parse_chunk_filename, ChunkCodec, ChunkData, CHUNK_DATA_SIZE_APPROX,
    CHUNK_FORMAT_VERSION,
};
pub use error::{PersistError, VersionCheck};
pub use version::{check_version, is_version_supported, CURRENT_VERSION, MIN_SUPPORTED_VERSION};
pub use world_meta::{WorldGenConfig, WorldKind, WorldMetadata};
