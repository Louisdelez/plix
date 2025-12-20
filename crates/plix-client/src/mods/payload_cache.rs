//! Payload cache for storing synchronized mod data
//!
//! Caches client payloads by SHA-256 hash to avoid redundant downloads.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Metadata stored alongside cached payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadMetadata {
    /// Mod this payload belongs to
    pub mod_id: String,
    /// Mod version at time of download
    pub mod_version: String,
    /// When this was cached (Unix timestamp)
    pub cached_at: u64,
    /// Original size in bytes
    pub size: u64,
}

/// Cache for client mod payloads indexed by SHA-256 hash
pub struct PayloadCache {
    /// Base directory for cache storage
    cache_dir: PathBuf,
    /// In-memory index of cached payloads
    index: HashMap<String, PayloadMetadata>,
}

impl PayloadCache {
    /// Create a new payload cache at the given directory
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            index: HashMap::new(),
        }
    }

    /// Get the default cache directory
    pub fn default_cache_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("plix")
            .join("mods")
            .join("payloads")
    }

    /// Check if a payload with the given hash is cached
    pub fn has_payload(&self, hash: &str) -> bool {
        self.index.contains_key(hash)
    }

    /// Get all cached payload hashes
    pub fn cached_hashes(&self) -> Vec<String> {
        self.index.keys().cloned().collect()
    }

    /// Get the file path for a payload hash
    pub fn payload_path(&self, hash: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.bin", hash))
    }

    /// Get the metadata path for a payload hash
    pub fn metadata_path(&self, hash: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.meta", hash))
    }

    /// Store a payload in the cache
    pub fn store(&mut self, hash: String, data: &[u8], metadata: PayloadMetadata) -> std::io::Result<()> {
        // Ensure cache directory exists
        std::fs::create_dir_all(&self.cache_dir)?;

        // Write payload data
        let payload_path = self.payload_path(&hash);
        std::fs::write(&payload_path, data)?;

        // Write metadata
        let metadata_path = self.metadata_path(&hash);
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&metadata_path, metadata_json)?;

        // Update index
        self.index.insert(hash, metadata);

        Ok(())
    }

    /// Load a payload from the cache
    pub fn load(&self, hash: &str) -> std::io::Result<Vec<u8>> {
        let payload_path = self.payload_path(hash);
        std::fs::read(payload_path)
    }

    /// Load the cache index from disk
    pub fn load_index(&mut self) -> std::io::Result<()> {
        self.index.clear();

        if !self.cache_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(ext) = path.extension() {
                if ext == "meta" {
                    if let Some(stem) = path.file_stem() {
                        let hash = stem.to_string_lossy().to_string();
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(metadata) = serde_json::from_str::<PayloadMetadata>(&content) {
                                self.index.insert(hash, metadata);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Remove a payload from the cache
    pub fn remove(&mut self, hash: &str) -> std::io::Result<()> {
        let payload_path = self.payload_path(hash);
        let metadata_path = self.metadata_path(hash);

        if payload_path.exists() {
            std::fs::remove_file(payload_path)?;
        }
        if metadata_path.exists() {
            std::fs::remove_file(metadata_path)?;
        }

        self.index.remove(hash);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_operations() {
        let temp_dir = std::env::temp_dir().join("plix_test_cache");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let mut cache = PayloadCache::new(temp_dir.clone());

        let hash = "abc123def456".to_string();
        let data = b"test payload data";
        let metadata = PayloadMetadata {
            mod_id: "test-mod".to_string(),
            mod_version: "1.0.0".to_string(),
            cached_at: 1234567890,
            size: data.len() as u64,
        };

        // Store
        cache.store(hash.clone(), data, metadata).unwrap();
        assert!(cache.has_payload(&hash));

        // Load
        let loaded = cache.load(&hash).unwrap();
        assert_eq!(loaded, data);

        // Cached hashes
        let hashes = cache.cached_hashes();
        assert_eq!(hashes.len(), 1);
        assert!(hashes.contains(&hash));

        // Remove
        cache.remove(&hash).unwrap();
        assert!(!cache.has_payload(&hash));

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
