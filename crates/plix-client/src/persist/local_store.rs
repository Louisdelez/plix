//! LocalStore - Solo mode world persistence.
//!
//! Wrapper around WorldStore for client-side solo play.

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use plix_common::chunk::{Chunk, ChunkCoord};
use plix_common::persist::{
    chunk_filename, parse_chunk_filename, ChunkCodec, PersistError, WorldMetadata,
};
use plix_common::world::ChunkedWorld;

/// Default worlds directory for solo play.
///
/// - Linux: `~/.local/share/plix/worlds/`
/// - macOS: `~/Library/Application Support/plix/worlds/`
/// - Windows: `%APPDATA%/plix/worlds/`
pub fn default_worlds_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("plix")
        .join("worlds")
}

/// Local world store for solo mode.
///
/// Provides a simplified interface for solo play persistence.
pub struct LocalStore {
    /// Path to world directory.
    path: PathBuf,
    /// World metadata.
    metadata: WorldMetadata,
}

impl LocalStore {
    // =========================================================================
    // LIFECYCLE
    // =========================================================================

    /// Create a new solo world.
    pub fn create(name: &str, seed: u64) -> Result<Self, PersistError> {
        let base_dir = default_worlds_dir();
        let world_id = Self::generate_world_id(name);

        let metadata = WorldMetadata::new_generated(world_id.clone(), name.to_string(), seed);

        let world_path = base_dir.join(&world_id);

        // Check if world already exists
        if world_path.exists() {
            return Err(PersistError::Io(format!(
                "world already exists: {}",
                world_id
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
            seed = seed,
            "Created new solo world"
        );

        Ok(Self {
            path: world_path,
            metadata,
        })
    }

    /// Open an existing solo world by ID.
    pub fn open(world_id: &str) -> Result<Self, PersistError> {
        let base_dir = default_worlds_dir();
        let world_path = base_dir.join(world_id);

        if !world_path.exists() {
            return Err(PersistError::WorldNotFound(world_id.to_string()));
        }

        let meta_path = world_path.join("meta.bin");
        let metadata = Self::load_meta(&meta_path)?;

        tracing::info!(
            world_id = %metadata.world_id,
            name = %metadata.name,
            "Opened solo world"
        );

        Ok(Self {
            path: world_path,
            metadata,
        })
    }

    /// Open or create a solo world.
    pub fn open_or_create(name: &str, seed: u64) -> Result<Self, PersistError> {
        let base_dir = default_worlds_dir();
        let world_id = Self::generate_world_id(name);

        if base_dir.join(&world_id).exists() {
            Self::open(&world_id)
        } else {
            Self::create(name, seed)
        }
    }

    /// Generate a filesystem-safe world ID from a name.
    fn generate_world_id(name: &str) -> String {
        let safe: String = name
            .chars()
            .filter_map(|c| {
                if c.is_alphanumeric() {
                    Some(c.to_ascii_lowercase())
                } else if c == ' ' || c == '-' {
                    Some('_')
                } else {
                    None
                }
            })
            .take(32)
            .collect();

        if safe.is_empty() {
            format!(
                "world_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            )
        } else {
            safe
        }
    }

    /// Load metadata from file.
    fn load_meta(path: &Path) -> Result<WorldMetadata, PersistError> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        bincode::deserialize(&bytes).map_err(|e| PersistError::Codec(e.to_string()))
    }

    // =========================================================================
    // ACCESSORS
    // =========================================================================

    /// Get the world metadata.
    pub fn metadata(&self) -> &WorldMetadata {
        &self.metadata
    }

    /// Get the world ID.
    pub fn world_id(&self) -> &str {
        &self.metadata.world_id
    }

    /// Get the world name.
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Get the seed (if generated world).
    pub fn seed(&self) -> Option<u64> {
        self.metadata.seed()
    }

    // =========================================================================
    // CHUNK I/O
    // =========================================================================

    /// Save a chunk to disk.
    pub fn save_chunk(&self, chunk: &Chunk) -> Result<(), PersistError> {
        let coord = chunk.coord();
        let filename = chunk_filename(coord);
        let chunk_path = self.path.join("chunks").join(&filename);

        let bytes = ChunkCodec::encode(chunk)?;
        atomic_write(&chunk_path, &bytes)?;

        tracing::trace!(coord = ?coord, "Saved chunk");
        Ok(())
    }

    /// Load a chunk from disk.
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
        Ok(Some(chunk))
    }

    /// Check if a chunk is saved.
    pub fn chunk_exists(&self, coord: ChunkCoord) -> bool {
        let filename = chunk_filename(coord);
        self.path.join("chunks").join(&filename).exists()
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
            if let Some(filename) = entry.path().file_name().and_then(|s| s.to_str()) {
                if let Some(coord) = parse_chunk_filename(filename) {
                    coords.push(coord);
                }
            }
        }

        Ok(coords)
    }

    // =========================================================================
    // WORLD SAVE/LOAD
    // =========================================================================

    /// Save the entire world (all dirty chunks).
    pub fn save_world(&mut self, world: &ChunkedWorld) -> Result<usize, PersistError> {
        let mut saved = 0;

        // Save all persistence-dirty chunks
        for coord in world.persistence_dirty_chunks() {
            if let Some(chunk) = world.get_chunk(coord) {
                self.save_chunk(chunk)?;
                saved += 1;
            }
        }

        // Update metadata timestamp
        self.metadata.touch();
        let meta_path = self.path.join("meta.bin");
        let meta_bytes =
            bincode::serialize(&self.metadata).map_err(|e| PersistError::Codec(e.to_string()))?;
        atomic_write(&meta_path, &meta_bytes)?;

        if saved > 0 {
            tracing::info!(chunks_saved = saved, "Saved solo world");
        }

        Ok(saved)
    }

    /// Load all saved chunks into the world.
    pub fn load_all_chunks(&self, world: &mut ChunkedWorld) -> Result<usize, PersistError> {
        let coords = self.list_saved_chunks()?;
        let mut loaded = 0;

        for coord in coords {
            if let Some(chunk) = self.load_chunk(coord)? {
                world.insert_chunk(coord, chunk);
                loaded += 1;
            }
        }

        if loaded > 0 {
            tracing::info!(chunks_loaded = loaded, "Loaded solo world chunks");
        }

        Ok(loaded)
    }
}

// =========================================================================
// ATOMIC WRITE (local copy to avoid server dependency)
// =========================================================================

fn atomic_write(path: &Path, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_path = path.with_extension("tmp");
    let mut file = File::create(&temp_path)?;
    file.write_all(data)?;
    file.sync_all()?;

    fs::rename(&temp_path, path)?;

    if let Some(parent) = path.parent() {
        if let Ok(dir) = File::open(parent) {
            let _ = dir.sync_all();
        }
    }

    Ok(())
}

// =========================================================================
// WORLD LISTING
// =========================================================================

/// List all solo worlds.
pub fn list_worlds() -> Result<Vec<WorldMetadata>, PersistError> {
    let base_dir = default_worlds_dir();

    if !base_dir.exists() {
        return Ok(Vec::new());
    }

    let mut worlds = Vec::new();

    for entry in fs::read_dir(&base_dir)? {
        let entry = entry?;
        let meta_path = entry.path().join("meta.bin");

        if meta_path.exists() {
            match LocalStore::load_meta(&meta_path) {
                Ok(metadata) => worlds.push(metadata),
                Err(e) => {
                    tracing::warn!(path = ?entry.path(), error = %e, "Skipping corrupted world");
                }
            }
        }
    }

    // Sort by last_saved (most recent first)
    worlds.sort_by(|a, b| b.last_saved.cmp(&a.last_saved));

    Ok(worlds)
}

/// Delete a solo world.
pub fn delete_world(world_id: &str) -> Result<(), PersistError> {
    let base_dir = default_worlds_dir();
    let world_path = base_dir.join(world_id);

    if !world_path.exists() {
        return Err(PersistError::WorldNotFound(world_id.to_string()));
    }

    fs::remove_dir_all(&world_path)?;

    tracing::info!(world_id = %world_id, "Deleted solo world");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::types::BlockType;
    use tempfile::TempDir;

    // Override default_worlds_dir for tests
    fn test_worlds_dir(temp_dir: &TempDir) -> PathBuf {
        temp_dir.path().to_path_buf()
    }

    #[test]
    fn test_generate_world_id() {
        assert_eq!(LocalStore::generate_world_id("My World"), "my_world");
        assert_eq!(LocalStore::generate_world_id("Test-World"), "test_world");
        assert_eq!(LocalStore::generate_world_id("123"), "123");
    }

    #[test]
    fn test_generate_world_id_special_chars() {
        let id = LocalStore::generate_world_id("Test!@#$%World");
        assert!(id.chars().all(|c| c.is_alphanumeric() || c == '_'));
    }

    #[test]
    fn test_chunk_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let world_path = temp_dir.path().join("test_world");
        fs::create_dir_all(world_path.join("chunks")).unwrap();

        let metadata =
            WorldMetadata::new_generated("test_world".to_string(), "Test World".to_string(), 12345);

        let store = LocalStore {
            path: world_path,
            metadata,
        };

        // Create and save chunk
        let coord = ChunkCoord::new(1, 2, 3);
        let mut chunk = Chunk::new(coord);
        chunk.set_block(5, 5, 5, BlockType::STONE);

        store.save_chunk(&chunk).unwrap();
        assert!(store.chunk_exists(coord));

        // Load and verify
        let loaded = store.load_chunk(coord).unwrap().unwrap();
        assert_eq!(loaded.coord(), coord);
        assert_eq!(loaded.get_block(5, 5, 5), BlockType::STONE);
    }
}
