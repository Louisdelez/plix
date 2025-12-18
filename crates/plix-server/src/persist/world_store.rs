//! WorldStore - File I/O operations for world persistence.
//!
//! Manages world directories with metadata and chunk files.

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use plix_common::chunk::{Chunk, ChunkCoord};
use plix_common::persist::{
    chunk_filename, parse_chunk_filename, ChunkCodec, PersistError, WorldMetadata,
};

use super::atomic::atomic_write;

/// Default worlds directory based on platform.
///
/// - Linux: `~/.local/share/plix/worlds/`
/// - macOS: `~/Library/Application Support/plix/worlds/`
/// - Windows: `%APPDATA%/plix/worlds/`
pub fn default_worlds_dir() -> PathBuf {
    let base = if cfg!(target_os = "macos") {
        dirs_next::data_dir()
            .or_else(dirs_next::home_dir)
            .map(|p| p.join("Library/Application Support"))
    } else if cfg!(target_os = "windows") {
        dirs_next::data_dir()
    } else {
        // Linux and others: XDG_DATA_HOME or ~/.local/share
        dirs_next::data_dir().or_else(|| dirs_next::home_dir().map(|p| p.join(".local/share")))
    };

    base.unwrap_or_else(|| PathBuf::from("."))
        .join("plix")
        .join("worlds")
}

/// Handle to a world directory for I/O operations.
pub struct WorldStore {
    /// Path to world directory.
    path: PathBuf,
    /// Cached metadata (updated on save).
    metadata: WorldMetadata,
}

impl WorldStore {
    // =========================================================================
    // LIFECYCLE
    // =========================================================================

    /// Create a new world with the given metadata.
    ///
    /// # Arguments
    /// * `base_dir` - Base directory for worlds (e.g., `~/.local/share/plix/worlds/`).
    /// * `metadata` - World metadata to store.
    ///
    /// # Returns
    /// * `Ok(WorldStore)` - Handle to the new world.
    /// * `Err(PersistError)` - If creation fails.
    ///
    /// # Creates
    /// ```text
    /// <base_dir>/<world_id>/
    /// ├── meta.bin
    /// └── chunks/
    /// ```
    pub fn create_world(base_dir: &Path, metadata: WorldMetadata) -> Result<Self, PersistError> {
        // Validate metadata
        metadata
            .validate()
            .map_err(|e| PersistError::MetadataCorrupted(e))?;

        let world_path = base_dir.join(&metadata.world_id);

        // Check if world already exists
        if world_path.exists() {
            return Err(PersistError::Io(format!(
                "world already exists: {}",
                metadata.world_id
            )));
        }

        // Create directory structure
        fs::create_dir_all(&world_path)?;
        fs::create_dir_all(world_path.join("chunks"))?;

        // Save metadata
        let meta_path = world_path.join("meta.bin");
        let meta_bytes =
            bincode::serialize(&metadata).map_err(|e| PersistError::Codec(e.to_string()))?;
        atomic_write(&meta_path, &meta_bytes)?;

        tracing::info!(
            world_id = %metadata.world_id,
            name = %metadata.name,
            "Created new world"
        );

        Ok(Self {
            path: world_path,
            metadata,
        })
    }

    /// Open an existing world.
    ///
    /// # Arguments
    /// * `base_dir` - Base directory for worlds.
    /// * `world_id` - ID of the world to open.
    ///
    /// # Returns
    /// * `Ok(WorldStore)` - Handle to the world.
    /// * `Err(PersistError::WorldNotFound)` - If world doesn't exist.
    /// * `Err(PersistError::VersionMismatch)` - If version incompatible.
    pub fn open_world(base_dir: &Path, world_id: &str) -> Result<Self, PersistError> {
        let world_path = base_dir.join(world_id);

        if !world_path.exists() {
            return Err(PersistError::WorldNotFound(world_id.to_string()));
        }

        // Load metadata
        let meta_path = world_path.join("meta.bin");
        let metadata = Self::load_meta_from_path(&meta_path)?;

        // Verify world_id matches directory name
        if metadata.world_id != world_id {
            return Err(PersistError::MetadataCorrupted(format!(
                "world_id mismatch: expected {}, got {}",
                world_id, metadata.world_id
            )));
        }

        // Ensure chunks directory exists
        let chunks_dir = world_path.join("chunks");
        if !chunks_dir.exists() {
            fs::create_dir_all(&chunks_dir)?;
        }

        tracing::info!(
            world_id = %metadata.world_id,
            name = %metadata.name,
            "Opened world"
        );

        Ok(Self {
            path: world_path,
            metadata,
        })
    }

    /// Open or create a world.
    ///
    /// Opens if exists, creates if not.
    pub fn open_or_create(base_dir: &Path, metadata: WorldMetadata) -> Result<Self, PersistError> {
        let world_path = base_dir.join(&metadata.world_id);

        if world_path.exists() {
            Self::open_world(base_dir, &metadata.world_id)
        } else {
            Self::create_world(base_dir, metadata)
        }
    }

    // =========================================================================
    // METADATA
    // =========================================================================

    /// Get the world metadata.
    pub fn metadata(&self) -> &WorldMetadata {
        &self.metadata
    }

    /// Get the world ID.
    pub fn world_id(&self) -> &str {
        &self.metadata.world_id
    }

    /// Get the world path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Save updated metadata.
    pub fn save_meta(&mut self) -> Result<(), PersistError> {
        self.metadata.touch();
        let meta_path = self.path.join("meta.bin");
        let meta_bytes =
            bincode::serialize(&self.metadata).map_err(|e| PersistError::Codec(e.to_string()))?;
        atomic_write(&meta_path, &meta_bytes)?;

        tracing::debug!(world_id = %self.metadata.world_id, "Saved world metadata");
        Ok(())
    }

    /// Load metadata from a file path.
    fn load_meta_from_path(path: &Path) -> Result<WorldMetadata, PersistError> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let metadata: WorldMetadata =
            bincode::deserialize(&bytes).map_err(|e| PersistError::Codec(e.to_string()))?;

        // Check version compatibility
        let check = plix_common::persist::check_version(metadata.format_version);
        if !plix_common::persist::is_version_supported(metadata.format_version) {
            return Err(PersistError::VersionMismatch {
                world_version: metadata.format_version,
                reason: check,
            });
        }

        Ok(metadata)
    }

    // =========================================================================
    // CHUNK I/O
    // =========================================================================

    /// Save a chunk to disk.
    ///
    /// # Arguments
    /// * `coord` - Chunk coordinates.
    /// * `chunk` - Chunk data to save.
    ///
    /// # Returns
    /// * `Ok(())` on success.
    /// * `Err(PersistError)` on failure.
    pub fn save_chunk(&self, coord: ChunkCoord, chunk: &Chunk) -> Result<(), PersistError> {
        let filename = chunk_filename(coord);
        let chunk_path = self.path.join("chunks").join(&filename);

        let bytes = ChunkCodec::encode(chunk)?;
        atomic_write(&chunk_path, &bytes)?;

        tracing::trace!(coord = ?coord, "Saved chunk");
        Ok(())
    }

    /// Load a chunk from disk.
    ///
    /// # Arguments
    /// * `coord` - Chunk coordinates.
    ///
    /// # Returns
    /// * `Ok(Some(Chunk))` if chunk exists.
    /// * `Ok(None)` if chunk file not found.
    /// * `Err(PersistError)` on I/O or decode error.
    pub fn load_chunk(&self, coord: ChunkCoord) -> Result<Option<Chunk>, PersistError> {
        let filename = chunk_filename(coord);
        let chunk_path = self.path.join("chunks").join(&filename);

        if !chunk_path.exists() {
            return Ok(None);
        }

        let mut file = File::open(&chunk_path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let chunk = ChunkCodec::decode(&bytes)?;

        // Verify coordinates match filename
        if chunk.coord() != coord {
            return Err(PersistError::ChunkCorrupted {
                coord,
                reason: format!(
                    "coordinate mismatch: file says {:?}, expected {:?}",
                    chunk.coord(),
                    coord
                ),
            });
        }

        tracing::trace!(coord = ?coord, "Loaded chunk");
        Ok(Some(chunk))
    }

    /// Check if a chunk file exists.
    pub fn chunk_exists(&self, coord: ChunkCoord) -> bool {
        let filename = chunk_filename(coord);
        let chunk_path = self.path.join("chunks").join(&filename);
        chunk_path.exists()
    }

    /// List all saved chunk coordinates.
    pub fn list_saved_chunks(&self) -> Result<Vec<ChunkCoord>, PersistError> {
        let chunks_dir = self.path.join("chunks");

        if !chunks_dir.exists() {
            return Ok(Vec::new());
        }

        let mut coords = Vec::new();

        for entry in fs::read_dir(&chunks_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if let Some(coord) = parse_chunk_filename(filename) {
                        coords.push(coord);
                    }
                }
            }
        }

        Ok(coords)
    }

    /// Delete a chunk file.
    pub fn delete_chunk(&self, coord: ChunkCoord) -> Result<bool, PersistError> {
        let filename = chunk_filename(coord);
        let chunk_path = self.path.join("chunks").join(&filename);

        if chunk_path.exists() {
            fs::remove_file(&chunk_path)?;
            tracing::trace!(coord = ?coord, "Deleted chunk");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // =========================================================================
    // BULK OPERATIONS
    // =========================================================================

    /// Save multiple chunks.
    ///
    /// # Returns
    /// Number of chunks successfully saved.
    pub fn save_chunks<'a>(
        &self,
        chunks: impl Iterator<Item = (ChunkCoord, &'a Chunk)>,
    ) -> Result<usize, PersistError> {
        let mut count = 0;
        for (coord, chunk) in chunks {
            self.save_chunk(coord, chunk)?;
            count += 1;
        }
        Ok(count)
    }

    /// Load all saved chunks.
    pub fn load_all_chunks(&self) -> Result<Vec<Chunk>, PersistError> {
        let coords = self.list_saved_chunks()?;
        let mut chunks = Vec::with_capacity(coords.len());

        for coord in coords {
            if let Some(chunk) = self.load_chunk(coord)? {
                chunks.push(chunk);
            }
        }

        Ok(chunks)
    }

    /// Get the total size of saved chunk files in bytes.
    pub fn chunks_size_bytes(&self) -> Result<u64, PersistError> {
        let chunks_dir = self.path.join("chunks");

        if !chunks_dir.exists() {
            return Ok(0);
        }

        let mut total = 0u64;
        for entry in fs::read_dir(&chunks_dir)? {
            let entry = entry?;
            if entry.path().is_file() {
                total += entry.metadata()?.len();
            }
        }

        Ok(total)
    }
}

// =========================================================================
// WORLD LISTING
// =========================================================================

/// List all worlds in the base directory.
///
/// # Arguments
/// * `base_dir` - Base directory for worlds.
///
/// # Returns
/// List of (world_id, metadata) pairs. Corrupted worlds are skipped with a warning.
pub fn list_worlds(base_dir: &Path) -> Result<Vec<(String, WorldMetadata)>, PersistError> {
    if !base_dir.exists() {
        return Ok(Vec::new());
    }

    let mut worlds = Vec::new();

    for entry in fs::read_dir(base_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let meta_path = path.join("meta.bin");
            if meta_path.exists() {
                match WorldStore::load_meta_from_path(&meta_path) {
                    Ok(metadata) => {
                        let world_id = path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        worlds.push((world_id, metadata));
                    }
                    Err(e) => {
                        tracing::warn!(
                            path = ?path,
                            error = %e,
                            "Skipping corrupted world"
                        );
                    }
                }
            }
        }
    }

    // Sort by last_saved descending (most recent first)
    worlds.sort_by(|a, b| b.1.last_saved.cmp(&a.1.last_saved));

    Ok(worlds)
}

/// Delete a world and all its data.
///
/// # Arguments
/// * `base_dir` - Base directory for worlds.
/// * `world_id` - ID of the world to delete.
///
/// # Warning
/// This permanently deletes all world data!
pub fn delete_world(base_dir: &Path, world_id: &str) -> Result<(), PersistError> {
    let world_path = base_dir.join(world_id);

    if !world_path.exists() {
        return Err(PersistError::WorldNotFound(world_id.to_string()));
    }

    fs::remove_dir_all(&world_path)?;

    tracing::info!(world_id = %world_id, "Deleted world");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::types::BlockType;
    use tempfile::TempDir;

    fn test_metadata() -> WorldMetadata {
        WorldMetadata::new_generated("test_world".to_string(), "Test World".to_string(), 12345)
    }

    #[test]
    fn test_create_world() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = test_metadata();

        let store = WorldStore::create_world(temp_dir.path(), metadata.clone()).unwrap();

        assert_eq!(store.world_id(), "test_world");
        assert!(store.path().join("meta.bin").exists());
        assert!(store.path().join("chunks").exists());
    }

    #[test]
    fn test_create_world_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = test_metadata();

        WorldStore::create_world(temp_dir.path(), metadata.clone()).unwrap();

        // Second creation should fail
        let result = WorldStore::create_world(temp_dir.path(), metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_open_world() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = test_metadata();

        WorldStore::create_world(temp_dir.path(), metadata).unwrap();

        let store = WorldStore::open_world(temp_dir.path(), "test_world").unwrap();
        assert_eq!(store.world_id(), "test_world");
        assert_eq!(store.metadata().name, "Test World");
    }

    #[test]
    fn test_open_world_not_found() {
        let temp_dir = TempDir::new().unwrap();

        let result = WorldStore::open_world(temp_dir.path(), "nonexistent");
        assert!(matches!(result, Err(PersistError::WorldNotFound(_))));
    }

    #[test]
    fn test_save_and_load_chunk() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = test_metadata();
        let store = WorldStore::create_world(temp_dir.path(), metadata).unwrap();

        // Create chunk with some blocks
        let coord = ChunkCoord::new(5, -3, 10);
        let mut chunk = Chunk::new(coord);
        chunk.set_block(0, 0, 0, BlockType::STONE);
        chunk.set_block(15, 15, 15, BlockType::BRICK);

        // Save
        store.save_chunk(coord, &chunk).unwrap();
        assert!(store.chunk_exists(coord));

        // Load
        let loaded = store.load_chunk(coord).unwrap().unwrap();
        assert_eq!(loaded.coord(), coord);
        assert_eq!(loaded.get_block(0, 0, 0), BlockType::STONE);
        assert_eq!(loaded.get_block(15, 15, 15), BlockType::BRICK);
    }

    #[test]
    fn test_load_nonexistent_chunk() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = test_metadata();
        let store = WorldStore::create_world(temp_dir.path(), metadata).unwrap();

        let result = store.load_chunk(ChunkCoord::new(0, 0, 0)).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_saved_chunks() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = test_metadata();
        let store = WorldStore::create_world(temp_dir.path(), metadata).unwrap();

        // Save multiple chunks
        let coords = [
            ChunkCoord::new(0, 0, 0),
            ChunkCoord::new(1, 0, 0),
            ChunkCoord::new(-1, 5, 3),
        ];

        for coord in &coords {
            let chunk = Chunk::new(*coord);
            store.save_chunk(*coord, &chunk).unwrap();
        }

        let saved = store.list_saved_chunks().unwrap();
        assert_eq!(saved.len(), 3);
        for coord in &coords {
            assert!(saved.contains(coord));
        }
    }

    #[test]
    fn test_delete_chunk() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = test_metadata();
        let store = WorldStore::create_world(temp_dir.path(), metadata).unwrap();

        let coord = ChunkCoord::new(0, 0, 0);
        let chunk = Chunk::new(coord);
        store.save_chunk(coord, &chunk).unwrap();

        assert!(store.chunk_exists(coord));
        store.delete_chunk(coord).unwrap();
        assert!(!store.chunk_exists(coord));
    }

    #[test]
    fn test_list_worlds() {
        let temp_dir = TempDir::new().unwrap();

        // Create multiple worlds
        WorldStore::create_world(
            temp_dir.path(),
            WorldMetadata::new_generated("world1".to_string(), "World 1".to_string(), 1),
        )
        .unwrap();

        WorldStore::create_world(
            temp_dir.path(),
            WorldMetadata::new_custom("world2".to_string(), "World 2".to_string()),
        )
        .unwrap();

        let worlds = list_worlds(temp_dir.path()).unwrap();
        assert_eq!(worlds.len(), 2);
    }

    #[test]
    fn test_delete_world() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = test_metadata();

        WorldStore::create_world(temp_dir.path(), metadata).unwrap();

        delete_world(temp_dir.path(), "test_world").unwrap();

        assert!(!temp_dir.path().join("test_world").exists());
    }

    #[test]
    fn test_save_meta_updates_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = test_metadata();
        let mut store = WorldStore::create_world(temp_dir.path(), metadata).unwrap();

        let original_timestamp = store.metadata().last_saved;
        std::thread::sleep(std::time::Duration::from_millis(10));

        store.save_meta().unwrap();

        assert!(store.metadata().last_saved >= original_timestamp);
    }

    #[test]
    fn test_chunk_roundtrip_with_negative_coords() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = test_metadata();
        let store = WorldStore::create_world(temp_dir.path(), metadata).unwrap();

        // Test negative coordinates
        let coords = [
            ChunkCoord::new(-100, -50, -25),
            ChunkCoord::new(-1, 0, 0),
            ChunkCoord::new(0, -1, 0),
        ];

        for coord in &coords {
            let mut chunk = Chunk::new(*coord);
            chunk.set_block(7, 7, 7, BlockType::METAL);

            store.save_chunk(*coord, &chunk).unwrap();
            let loaded = store.load_chunk(*coord).unwrap().unwrap();

            assert_eq!(loaded.coord(), *coord);
            assert_eq!(loaded.get_block(7, 7, 7), BlockType::METAL);
        }
    }
}
