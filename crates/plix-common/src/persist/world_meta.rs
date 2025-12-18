//! World metadata types.
//!
//! Defines the metadata structure stored in `meta.bin` for each world.

use serde::{Deserialize, Serialize};

use super::version::CURRENT_VERSION;

/// How the world was created (affects load strategy).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorldKind {
    /// Procedurally generated world (regenerate unmodified chunks).
    Generated {
        /// Seed for deterministic generation.
        seed: u64,
        /// Generation config (optional, for non-default settings).
        config: Option<WorldGenConfig>,
    },

    /// Imported from arena definition.
    Arena {
        /// Original arena file name.
        arena_name: String,
    },

    /// Manually created or imported (all chunks must be saved).
    Custom,
}

impl Default for WorldKind {
    fn default() -> Self {
        WorldKind::Custom
    }
}

/// Configuration for procedural world generation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldGenConfig {
    /// Biome type or generator name.
    pub biome: String,
    /// Sea level height.
    pub sea_level: i32,
    /// Additional parameters as key-value pairs.
    pub params: Vec<(String, String)>,
}

impl Default for WorldGenConfig {
    fn default() -> Self {
        Self {
            biome: "default".to_string(),
            sea_level: 64,
            params: Vec::new(),
        }
    }
}

/// World metadata stored in `meta.bin`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    /// Format version for compatibility checking.
    pub format_version: u32,

    /// Unique identifier (also directory name).
    pub world_id: String,

    /// Human-readable display name.
    pub name: String,

    /// World generation type.
    pub world_kind: WorldKind,

    /// Creation timestamp (Unix seconds).
    pub created_at: u64,

    /// Last save timestamp (Unix seconds).
    pub last_saved: u64,
}

impl WorldMetadata {
    /// Create new metadata for a generated world.
    ///
    /// # Arguments
    /// * `world_id` - Unique identifier (used as directory name).
    /// * `name` - Human-readable display name.
    /// * `seed` - Seed for procedural generation.
    pub fn new_generated(world_id: String, name: String, seed: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            format_version: CURRENT_VERSION,
            world_id,
            name,
            world_kind: WorldKind::Generated { seed, config: None },
            created_at: now,
            last_saved: now,
        }
    }

    /// Create new metadata for an arena world.
    ///
    /// # Arguments
    /// * `world_id` - Unique identifier (used as directory name).
    /// * `name` - Human-readable display name.
    /// * `arena_name` - Original arena file name.
    pub fn new_arena(world_id: String, name: String, arena_name: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            format_version: CURRENT_VERSION,
            world_id,
            name,
            world_kind: WorldKind::Arena { arena_name },
            created_at: now,
            last_saved: now,
        }
    }

    /// Create new metadata for a custom world.
    ///
    /// # Arguments
    /// * `world_id` - Unique identifier (used as directory name).
    /// * `name` - Human-readable display name.
    pub fn new_custom(world_id: String, name: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            format_version: CURRENT_VERSION,
            world_id,
            name,
            world_kind: WorldKind::Custom,
            created_at: now,
            last_saved: now,
        }
    }

    /// Update the last_saved timestamp to now.
    pub fn touch(&mut self) {
        self.last_saved = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(self.last_saved);
    }

    /// Check if this is a procedurally generated world.
    pub fn is_generated(&self) -> bool {
        matches!(self.world_kind, WorldKind::Generated { .. })
    }

    /// Get the seed if this is a generated world.
    pub fn seed(&self) -> Option<u64> {
        match &self.world_kind {
            WorldKind::Generated { seed, .. } => Some(*seed),
            _ => None,
        }
    }

    /// Validate the metadata.
    ///
    /// # Returns
    /// `Ok(())` if valid, `Err(reason)` if invalid.
    pub fn validate(&self) -> Result<(), String> {
        // Check world_id
        if self.world_id.is_empty() {
            return Err("world_id is empty".to_string());
        }
        if !self
            .world_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err("world_id contains invalid characters".to_string());
        }

        // Check name
        if self.name.is_empty() {
            return Err("name is empty".to_string());
        }
        if self.name.len() > 64 {
            return Err("name exceeds 64 characters".to_string());
        }

        // Check timestamps
        if self.created_at > self.last_saved {
            return Err("created_at is after last_saved".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_generated() {
        let meta =
            WorldMetadata::new_generated("test_world".to_string(), "Test World".to_string(), 12345);
        assert_eq!(meta.world_id, "test_world");
        assert_eq!(meta.name, "Test World");
        assert_eq!(meta.format_version, CURRENT_VERSION);
        assert!(meta.is_generated());
        assert_eq!(meta.seed(), Some(12345));
    }

    #[test]
    fn test_new_arena() {
        let meta = WorldMetadata::new_arena(
            "arena_1".to_string(),
            "Arena One".to_string(),
            "arena.toml".to_string(),
        );
        assert_eq!(meta.world_id, "arena_1");
        assert!(!meta.is_generated());
        assert_eq!(meta.seed(), None);
        match &meta.world_kind {
            WorldKind::Arena { arena_name } => assert_eq!(arena_name, "arena.toml"),
            _ => panic!("Expected Arena kind"),
        }
    }

    #[test]
    fn test_new_custom() {
        let meta = WorldMetadata::new_custom("custom_1".to_string(), "Custom World".to_string());
        assert_eq!(meta.world_id, "custom_1");
        assert!(!meta.is_generated());
        assert!(matches!(meta.world_kind, WorldKind::Custom));
    }

    #[test]
    fn test_validate_valid() {
        let meta = WorldMetadata::new_generated("my_world".to_string(), "My World".to_string(), 0);
        assert!(meta.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_world_id() {
        let mut meta = WorldMetadata::new_custom("".to_string(), "Name".to_string());
        meta.world_id = "".to_string();
        assert!(meta.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_world_id_chars() {
        let mut meta = WorldMetadata::new_custom("test".to_string(), "Name".to_string());
        meta.world_id = "test/world".to_string();
        assert!(meta.validate().is_err());

        meta.world_id = "test world".to_string();
        assert!(meta.validate().is_err());
    }

    #[test]
    fn test_validate_empty_name() {
        let mut meta = WorldMetadata::new_custom("test".to_string(), "Name".to_string());
        meta.name = "".to_string();
        assert!(meta.validate().is_err());
    }

    #[test]
    fn test_validate_name_too_long() {
        let mut meta = WorldMetadata::new_custom("test".to_string(), "Name".to_string());
        meta.name = "a".repeat(65);
        assert!(meta.validate().is_err());
    }

    #[test]
    fn test_validate_timestamp_order() {
        let mut meta = WorldMetadata::new_custom("test".to_string(), "Name".to_string());
        meta.created_at = 1000;
        meta.last_saved = 500;
        assert!(meta.validate().is_err());
    }

    #[test]
    fn test_touch() {
        let mut meta = WorldMetadata::new_custom("test".to_string(), "Name".to_string());
        let original = meta.last_saved;
        std::thread::sleep(std::time::Duration::from_millis(10));
        meta.touch();
        // last_saved should be >= original (could be same if resolution is low)
        assert!(meta.last_saved >= original);
    }

    #[test]
    fn test_world_gen_config_default() {
        let config = WorldGenConfig::default();
        assert_eq!(config.biome, "default");
        assert_eq!(config.sea_level, 64);
        assert!(config.params.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let meta = WorldMetadata::new_generated("test".to_string(), "Test".to_string(), 42);
        let bytes = bincode::serialize(&meta).unwrap();
        let decoded: WorldMetadata = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded.world_id, meta.world_id);
        assert_eq!(decoded.name, meta.name);
        assert_eq!(decoded.format_version, meta.format_version);
    }
}
