//! Content definitions for Adventure Mode
//!
//! This module contains data-driven content types that can be loaded from TOML files:
//! - Chapters: Narrative containers grouping quests
//! - Quests: Player objectives with steps and rewards
//! - Mobs: Enemy entity definitions with stats and behaviors
//! - Loot: Drop tables for mobs and chests
//! - Spawns: Mob spawn point definitions
//! - Dungeons: Combat areas with rooms, bosses, and rewards
//! - NPCs: Non-player characters with dialogue

pub mod chapter;
pub mod dungeon;
pub mod ids;
pub mod loot;
pub mod mob;
pub mod npc;
pub mod quest;
pub mod spawn;

// Re-export commonly used types
pub use chapter::ChapterDefinition;
pub use dungeon::{DungeonDefinition, RoomDefinition};
pub use ids::*;
pub use loot::{LootEntry, LootTable};
pub use mob::{BossPhase, BossPhaseModifier, MobBehavior, MobDefinition, MobStats};
pub use npc::{NpcDefinition, QuestDialogue};
pub use quest::{QuestDefinition, QuestPrerequisites, QuestRewards, QuestStep};
pub use spawn::SpawnPointDefinition;
