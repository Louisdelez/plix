//! Quest system for Adventure Mode
//!
//! This module handles:
//! - Quest progress tracking per player
//! - Step completion (KillMob, CollectItem, VisitLocation, TalkToNpc, DungeonClear)
//! - Prerequisite validation
//! - Reward granting on completion
//! - Integration with mob kills for KillMob steps

pub mod progress;
pub mod system;

pub use progress::{ActiveQuestState, PlayerQuestProgress, StepProgress};
pub use system::{QuestSystem, QuestSystemConfig};
