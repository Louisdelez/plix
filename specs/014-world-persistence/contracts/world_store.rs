//! WorldStore API Contract
//!
//! This file defines the public API for world persistence operations.
//! Implementation in: crates/plix-server/src/persist/world_store.rs

use std::path::{Path, PathBuf};

// Re-exports from plix-common
use crate::persist::{ChunkData, PersistError, WorldMetadata};
use crate::chunk::{Chunk, ChunkCoord};

/// Handle to an open world directory for I/O operations.
///
/// # Thread Safety
/// WorldStore is NOT thread-safe. For concurrent access, wrap in Arc<Mutex<>>
/// or use separate instances per thread with external synchronization.
///
/// # Atomic Writes
/// All write operations use atomic write-to-temp-then-rename pattern
/// to prevent corruption from crashes.
pub struct WorldStore {
    // Private fields - implementation detail
}

impl WorldStore {
    // =========================================================================
    // WORLD LIFECYCLE
    // =========================================================================

    /// Create a new world with the given metadata.
    ///
    /// # Arguments
    /// * `base_path` - Parent directory for worlds (e.g., ~/.local/share/plix/worlds)
    /// * `metadata` - World metadata (world_id used as directory name)
    ///
    /// # Returns
    /// * `Ok(WorldStore)` - Handle to the new world
    /// * `Err(PersistError::Io)` - If directory creation fails
    /// * `Err(PersistError::PermissionDenied)` - If base_path not writable
    ///
    /// # Postconditions
    /// * Directory `base_path/<world_id>/` created
    /// * Directory `base_path/<world_id>/chunks/` created
    /// * File `base_path/<world_id>/meta.bin` written with metadata
    pub fn create_world(base_path: &Path, metadata: WorldMetadata) -> Result<Self, PersistError>;

    /// Open an existing world by ID.
    ///
    /// # Arguments
    /// * `base_path` - Parent directory for worlds
    /// * `world_id` - World identifier (directory name)
    ///
    /// # Returns
    /// * `Ok(WorldStore)` - Handle to the world
    /// * `Err(PersistError::WorldNotFound)` - If world directory doesn't exist
    /// * `Err(PersistError::MetadataCorrupted)` - If meta.bin invalid
    /// * `Err(PersistError::VersionMismatch)` - If version incompatible
    ///
    /// # Version Handling
    /// * Version == CURRENT: Load normally
    /// * MIN_SUPPORTED <= Version < CURRENT: Apply migrations
    /// * Version < MIN_SUPPORTED: Return VersionMismatch(TooOld)
    /// * Version > CURRENT: Return VersionMismatch(TooNew)
    pub fn open_world(base_path: &Path, world_id: &str) -> Result<Self, PersistError>;

    /// List all available worlds in the base directory.
    ///
    /// # Arguments
    /// * `base_path` - Parent directory for worlds
    ///
    /// # Returns
    /// * List of (world_id, Result<WorldMetadata, PersistError>)
    /// * Corrupted worlds included with error, not silently skipped
    ///
    /// # Performance
    /// * O(n) where n = number of world directories
    /// * Only reads meta.bin, not chunk data
    pub fn list_worlds(base_path: &Path) -> Vec<(String, Result<WorldMetadata, PersistError>)>;

    // =========================================================================
    // METADATA OPERATIONS
    // =========================================================================

    /// Get cached metadata for this world.
    ///
    /// # Returns
    /// Reference to WorldMetadata loaded on open/create.
    /// Updated after save_metadata() calls.
    pub fn metadata(&self) -> &WorldMetadata;

    /// Save updated metadata to disk.
    ///
    /// # Arguments
    /// * `metadata` - New metadata (world_id must match)
    ///
    /// # Returns
    /// * `Ok(())` - Metadata saved successfully
    /// * `Err(PersistError::Io)` - If write fails
    ///
    /// # Atomicity
    /// Uses atomic write pattern - meta.bin either fully old or fully new.
    pub fn save_metadata(&mut self, metadata: WorldMetadata) -> Result<(), PersistError>;

    // =========================================================================
    // CHUNK OPERATIONS
    // =========================================================================

    /// Load a chunk from disk.
    ///
    /// # Arguments
    /// * `coord` - Chunk coordinates
    ///
    /// # Returns
    /// * `Ok(Some(chunk))` - Chunk loaded successfully
    /// * `Ok(None)` - Chunk file doesn't exist (not an error)
    /// * `Err(PersistError::ChunkCorrupted)` - If file exists but invalid
    /// * `Err(PersistError::Io)` - If read fails
    ///
    /// # For Procedural Worlds
    /// Caller should regenerate chunk if Ok(None) returned and world is Generated.
    pub fn load_chunk(&self, coord: ChunkCoord) -> Result<Option<Chunk>, PersistError>;

    /// Save a chunk to disk.
    ///
    /// # Arguments
    /// * `coord` - Chunk coordinates
    /// * `chunk` - Chunk data to save
    ///
    /// # Returns
    /// * `Ok(())` - Chunk saved successfully
    /// * `Err(PersistError::Io)` - If write fails
    /// * `Err(PersistError::DiskFull)` - If disk space exhausted
    ///
    /// # Atomicity
    /// Uses atomic write pattern - chunk file either fully old or fully new.
    ///
    /// # File Naming
    /// Saved to `chunks/<x>_<y>_<z>.bin`
    pub fn save_chunk(&self, coord: ChunkCoord, chunk: &Chunk) -> Result<(), PersistError>;

    /// Delete a chunk file.
    ///
    /// # Arguments
    /// * `coord` - Chunk coordinates
    ///
    /// # Returns
    /// * `Ok(())` - Chunk deleted or didn't exist
    /// * `Err(PersistError::Io)` - If deletion fails
    ///
    /// # Note
    /// Not an error if chunk doesn't exist (idempotent).
    pub fn delete_chunk(&self, coord: ChunkCoord) -> Result<(), PersistError>;

    /// Check if a chunk file exists.
    ///
    /// # Arguments
    /// * `coord` - Chunk coordinates
    ///
    /// # Returns
    /// * `true` if chunk file exists
    /// * `false` if not (doesn't check validity)
    ///
    /// # Performance
    /// Fast filesystem stat operation, doesn't read file.
    pub fn chunk_exists(&self, coord: ChunkCoord) -> bool;

    /// List all saved chunk coordinates.
    ///
    /// # Returns
    /// * `Ok(Vec<ChunkCoord>)` - All chunk coordinates with saved files
    /// * `Err(PersistError::Io)` - If directory read fails
    ///
    /// # Performance
    /// O(n) where n = number of chunk files.
    pub fn list_saved_chunks(&self) -> Result<Vec<ChunkCoord>, PersistError>;

    // =========================================================================
    // PATH ACCESSORS
    // =========================================================================

    /// Get the world directory path.
    pub fn world_path(&self) -> &Path;

    /// Get the chunks directory path.
    pub fn chunks_path(&self) -> PathBuf;

    /// Get path for a specific chunk file.
    pub fn chunk_path(&self, coord: ChunkCoord) -> PathBuf;
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Get the default worlds directory for the current platform.
///
/// # Returns
/// * Linux: ~/.local/share/plix/worlds/
/// * macOS: ~/Library/Application Support/plix/worlds/
/// * Windows: %APPDATA%\plix\worlds\
///
/// # Fallback
/// If platform directory not available, returns ./worlds/
pub fn default_worlds_dir() -> PathBuf;

/// Validate a world ID for filesystem safety.
///
/// # Rules
/// * Non-empty
/// * Max 64 characters
/// * Alphanumeric, underscore, hyphen only
/// * No path separators
///
/// # Returns
/// * `Ok(())` if valid
/// * `Err(reason)` if invalid
pub fn validate_world_id(id: &str) -> Result<(), String>;

/// Generate a unique world ID from a display name.
///
/// # Arguments
/// * `name` - Display name
///
/// # Returns
/// Sanitized ID with timestamp suffix for uniqueness.
///
/// # Example
/// "My World!" → "my_world_20251216_143052"
pub fn generate_world_id(name: &str) -> String;
