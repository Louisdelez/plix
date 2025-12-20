//! Content loader for TOML-based game content

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;
use tracing::{debug, info, warn};

use plix_common::content::{
    ChapterDefinition, ChapterId, DungeonDefinition, DungeonId, LootTable, LootTableId,
    MobDefId, MobDefinition, NpcDefinition, NpcId, QuestDefinition, QuestId,
    SpawnPointDefinition, SpawnId,
};
use plix_common::content::spawn::SpawnPointCollection;

/// Errors that can occur during content loading
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("IO error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("TOML parse error in {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Content directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("Duplicate ID '{id}' found in {path1} and {path2}")]
    DuplicateId {
        id: String,
        path1: PathBuf,
        path2: PathBuf,
    },
}

/// Registry containing all loaded content
#[derive(Debug, Default)]
pub struct ContentRegistry {
    /// Chapter definitions keyed by chapter_id
    pub chapters: HashMap<ChapterId, ChapterDefinition>,

    /// Quest definitions keyed by quest_id
    pub quests: HashMap<QuestId, QuestDefinition>,

    /// Mob definitions keyed by mob_id
    pub mobs: HashMap<MobDefId, MobDefinition>,

    /// Loot table definitions keyed by loot_table_id
    pub loot_tables: HashMap<LootTableId, LootTable>,

    /// Spawn point definitions keyed by spawn_id
    pub spawns: HashMap<SpawnId, SpawnPointDefinition>,

    /// Dungeon definitions keyed by dungeon_id
    pub dungeons: HashMap<DungeonId, DungeonDefinition>,

    /// NPC definitions keyed by npc_id
    pub npcs: HashMap<NpcId, NpcDefinition>,
}

impl ContentRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a quest definition by ID
    pub fn get_quest(&self, id: &QuestId) -> Option<&QuestDefinition> {
        self.quests.get(id)
    }

    /// Get a mob definition by ID
    pub fn get_mob(&self, id: &MobDefId) -> Option<&MobDefinition> {
        self.mobs.get(id)
    }

    /// Get a loot table by ID
    pub fn get_loot_table(&self, id: &LootTableId) -> Option<&LootTable> {
        self.loot_tables.get(id)
    }

    /// Get a dungeon definition by ID
    pub fn get_dungeon(&self, id: &DungeonId) -> Option<&DungeonDefinition> {
        self.dungeons.get(id)
    }

    /// Get an NPC definition by ID
    pub fn get_npc(&self, id: &NpcId) -> Option<&NpcDefinition> {
        self.npcs.get(id)
    }

    /// Get a chapter definition by ID
    pub fn get_chapter(&self, id: &ChapterId) -> Option<&ChapterDefinition> {
        self.chapters.get(id)
    }

    /// Get content counts for logging
    pub fn counts(&self) -> ContentCounts {
        ContentCounts {
            chapters: self.chapters.len(),
            quests: self.quests.len(),
            mobs: self.mobs.len(),
            loot_tables: self.loot_tables.len(),
            spawns: self.spawns.len(),
            dungeons: self.dungeons.len(),
            npcs: self.npcs.len(),
        }
    }
}

/// Summary of loaded content counts
#[derive(Debug, Clone)]
pub struct ContentCounts {
    pub chapters: usize,
    pub quests: usize,
    pub mobs: usize,
    pub loot_tables: usize,
    pub spawns: usize,
    pub dungeons: usize,
    pub npcs: usize,
}

/// Loads content from TOML files
pub struct ContentLoader {
    /// Base path for content files
    content_path: PathBuf,
}

impl ContentLoader {
    /// Create a new loader for the given content directory
    pub fn new(content_path: impl Into<PathBuf>) -> Self {
        Self {
            content_path: content_path.into(),
        }
    }

    /// Load all content from the content directory
    pub fn load_all(&self) -> Result<ContentRegistry, LoadError> {
        let base = &self.content_path;

        if !base.exists() {
            return Err(LoadError::DirectoryNotFound(base.clone()));
        }

        info!("Loading content from {:?}", base);

        let mut registry = ContentRegistry::new();

        // Load each content type
        self.load_chapters(&mut registry)?;
        self.load_quests(&mut registry)?;
        self.load_mobs(&mut registry)?;
        self.load_loot_tables(&mut registry)?;
        self.load_spawns(&mut registry)?;
        self.load_dungeons(&mut registry)?;
        self.load_npcs(&mut registry)?;

        let counts = registry.counts();
        info!(
            "Loaded content: {} chapters, {} quests, {} mobs, {} loot tables, {} spawns, {} dungeons, {} npcs",
            counts.chapters, counts.quests, counts.mobs, counts.loot_tables,
            counts.spawns, counts.dungeons, counts.npcs
        );

        Ok(registry)
    }

    /// Load chapter definitions from chapters/
    fn load_chapters(&self, registry: &mut ContentRegistry) -> Result<(), LoadError> {
        let dir = self.content_path.join("chapters");
        if !dir.exists() {
            debug!("No chapters directory found, skipping");
            return Ok(());
        }

        for entry in self.read_toml_files(&dir)? {
            let chapter: ChapterDefinition = self.parse_file(&entry)?;
            let id = chapter.chapter_id.clone();
            debug!("Loaded chapter: {}", id);
            registry.chapters.insert(id, chapter);
        }

        Ok(())
    }

    /// Load quest definitions from quests/
    fn load_quests(&self, registry: &mut ContentRegistry) -> Result<(), LoadError> {
        let dir = self.content_path.join("quests");
        if !dir.exists() {
            debug!("No quests directory found, skipping");
            return Ok(());
        }

        for entry in self.read_toml_files(&dir)? {
            let quest: QuestDefinition = self.parse_file(&entry)?;
            let id = quest.quest_id.clone();
            debug!("Loaded quest: {}", id);
            registry.quests.insert(id, quest);
        }

        Ok(())
    }

    /// Load mob definitions from mobs/
    fn load_mobs(&self, registry: &mut ContentRegistry) -> Result<(), LoadError> {
        let dir = self.content_path.join("mobs");
        if !dir.exists() {
            debug!("No mobs directory found, skipping");
            return Ok(());
        }

        for entry in self.read_toml_files(&dir)? {
            let mob: MobDefinition = self.parse_file(&entry)?;
            let id = mob.mob_id.clone();
            debug!("Loaded mob: {}", id);
            registry.mobs.insert(id, mob);
        }

        Ok(())
    }

    /// Load loot table definitions from loot_tables/
    fn load_loot_tables(&self, registry: &mut ContentRegistry) -> Result<(), LoadError> {
        let dir = self.content_path.join("loot_tables");
        if !dir.exists() {
            debug!("No loot_tables directory found, skipping");
            return Ok(());
        }

        for entry in self.read_toml_files(&dir)? {
            let loot: LootTable = self.parse_file(&entry)?;
            let id = loot.loot_table_id.clone();
            debug!("Loaded loot table: {}", id);
            registry.loot_tables.insert(id, loot);
        }

        Ok(())
    }

    /// Load spawn point definitions from spawns/
    fn load_spawns(&self, registry: &mut ContentRegistry) -> Result<(), LoadError> {
        let dir = self.content_path.join("spawns");
        if !dir.exists() {
            debug!("No spawns directory found, skipping");
            return Ok(());
        }

        for entry in self.read_toml_files(&dir)? {
            let collection: SpawnPointCollection = self.parse_file(&entry)?;
            for spawn in collection.spawns {
                let id = spawn.spawn_id.clone();
                debug!("Loaded spawn: {}", id);
                registry.spawns.insert(id, spawn);
            }
        }

        Ok(())
    }

    /// Load dungeon definitions from dungeons/
    fn load_dungeons(&self, registry: &mut ContentRegistry) -> Result<(), LoadError> {
        let dir = self.content_path.join("dungeons");
        if !dir.exists() {
            debug!("No dungeons directory found, skipping");
            return Ok(());
        }

        for entry in self.read_toml_files(&dir)? {
            let dungeon: DungeonDefinition = self.parse_file(&entry)?;
            let id = dungeon.dungeon_id.clone();
            debug!("Loaded dungeon: {}", id);
            registry.dungeons.insert(id, dungeon);
        }

        Ok(())
    }

    /// Load NPC definitions from npcs/
    fn load_npcs(&self, registry: &mut ContentRegistry) -> Result<(), LoadError> {
        let dir = self.content_path.join("npcs");
        if !dir.exists() {
            debug!("No npcs directory found, skipping");
            return Ok(());
        }

        for entry in self.read_toml_files(&dir)? {
            let npc: NpcDefinition = self.parse_file(&entry)?;
            let id = npc.npc_id.clone();
            debug!("Loaded npc: {}", id);
            registry.npcs.insert(id, npc);
        }

        Ok(())
    }

    /// Read all .toml files from a directory
    fn read_toml_files(&self, dir: &Path) -> Result<Vec<PathBuf>, LoadError> {
        let mut files = Vec::new();

        let entries = fs::read_dir(dir).map_err(|e| LoadError::Io {
            path: dir.to_path_buf(),
            source: e,
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| LoadError::Io {
                path: dir.to_path_buf(),
                source: e,
            })?;

            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "toml") {
                files.push(path);
            }
        }

        // Sort for deterministic loading order
        files.sort();
        Ok(files)
    }

    /// Parse a TOML file into the given type
    fn parse_file<T: serde::de::DeserializeOwned>(&self, path: &Path) -> Result<T, LoadError> {
        let content = fs::read_to_string(path).map_err(|e| LoadError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        toml::from_str(&content).map_err(|e| LoadError::Parse {
            path: path.to_path_buf(),
            source: e,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_content(dir: &Path) {
        // Create subdirectories
        fs::create_dir_all(dir.join("quests")).unwrap();
        fs::create_dir_all(dir.join("mobs")).unwrap();

        // Create a quest file
        let mut quest_file = fs::File::create(dir.join("quests/test_quest.toml")).unwrap();
        write!(
            quest_file,
            r#"
quest_id = "test_quest"
title = "Test Quest"
description_short = "A test"
description_long = "A longer test"

[[steps]]
type = "KillMob"
mob_id = "test_mob"
count = 5
description = "Kill 5 test mobs"

[rewards]
xp = 100
"#
        )
        .unwrap();

        // Create a mob file
        let mut mob_file = fs::File::create(dir.join("mobs/test_mob.toml")).unwrap();
        write!(
            mob_file,
            r#"
mob_id = "test_mob"
display_name = "Test Mob"
loot_table = "test_drops"
xp_reward = 10

[stats]
hp = 50
damage = 5
speed = 3.0
aggro_radius = 8.0
leash_radius = 20.0
attack_range = 2.0
attack_cooldown = 1.0

[behavior]
type = "Aggro"
"#
        )
        .unwrap();
    }

    #[test]
    fn test_load_content() {
        let temp_dir = TempDir::new().unwrap();
        create_test_content(temp_dir.path());

        let loader = ContentLoader::new(temp_dir.path());
        let registry = loader.load_all().unwrap();

        assert_eq!(registry.quests.len(), 1);
        assert_eq!(registry.mobs.len(), 1);

        let quest = registry.get_quest(&QuestId::new("test_quest"));
        assert!(quest.is_some());
        assert_eq!(quest.unwrap().title, "Test Quest");

        let mob = registry.get_mob(&MobDefId::new("test_mob"));
        assert!(mob.is_some());
        assert_eq!(mob.unwrap().stats.hp, 50);
    }

    #[test]
    fn test_missing_directory() {
        let loader = ContentLoader::new("/nonexistent/path");
        let result = loader.load_all();
        assert!(matches!(result, Err(LoadError::DirectoryNotFound(_))));
    }

    #[test]
    fn test_empty_content_dir() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ContentLoader::new(temp_dir.path());
        let registry = loader.load_all().unwrap();

        assert_eq!(registry.quests.len(), 0);
        assert_eq!(registry.mobs.len(), 0);
    }
}
