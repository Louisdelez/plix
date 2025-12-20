//! Quest definitions and step types

use serde::{Deserialize, Serialize};

use super::ids::{ChapterId, DungeonId, MobDefId, NpcId, QuestId, RegionId};
use crate::types::ItemId;

/// A player objective with ordered steps, prerequisites, and rewards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestDefinition {
    /// Unique stable identifier (e.g., "clear_rats")
    pub quest_id: QuestId,

    /// Display title
    pub title: String,

    /// Short description for list view
    pub description_short: String,

    /// Long description with lore
    pub description_long: String,

    /// Associated chapter (optional)
    #[serde(default)]
    pub chapter_id: Option<ChapterId>,

    /// Prerequisites to accept this quest
    #[serde(default)]
    pub prerequisites: QuestPrerequisites,

    /// Ordered steps to complete
    pub steps: Vec<QuestStep>,

    /// Rewards on completion
    pub rewards: QuestRewards,

    /// Whether quest can be repeated after completion
    #[serde(default)]
    pub repeatable: bool,
}

impl QuestDefinition {
    /// Get the total number of steps
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Get a step by index
    pub fn get_step(&self, index: usize) -> Option<&QuestStep> {
        self.steps.get(index)
    }
}

/// Prerequisites required to accept a quest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestPrerequisites {
    /// Quests that must be completed first
    #[serde(default)]
    pub completed_quests: Vec<QuestId>,

    /// Minimum player level (if leveling exists)
    #[serde(default)]
    pub min_level: Option<u32>,

    /// Required items in inventory
    #[serde(default)]
    pub required_items: Vec<ItemId>,
}

impl QuestPrerequisites {
    /// Check if there are any prerequisites
    pub fn has_any(&self) -> bool {
        !self.completed_quests.is_empty()
            || self.min_level.is_some()
            || !self.required_items.is_empty()
    }
}

/// A single objective within a quest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum QuestStep {
    /// Collect X of item_id (individual - collecting player only)
    CollectItem {
        item_id: ItemId,
        count: u32,
        description: String,
    },

    /// Kill X of mob_id (last-hit credit only)
    KillMob {
        mob_id: MobDefId,
        count: u32,
        description: String,
    },

    /// Enter a specific region/zone
    VisitLocation {
        region_id: RegionId,
        description: String,
    },

    /// Interact with an NPC
    TalkToNpc {
        npc_id: NpcId,
        description: String,
    },

    /// Complete a dungeon (boss killed + chest looted)
    DungeonClear {
        dungeon_id: DungeonId,
        description: String,
    },
}

impl QuestStep {
    /// Get the human-readable description of this step
    pub fn description(&self) -> &str {
        match self {
            Self::CollectItem { description, .. } => description,
            Self::KillMob { description, .. } => description,
            Self::VisitLocation { description, .. } => description,
            Self::TalkToNpc { description, .. } => description,
            Self::DungeonClear { description, .. } => description,
        }
    }

    /// Get the step type as a string for mod events
    pub fn step_type(&self) -> &'static str {
        match self {
            Self::CollectItem { .. } => "CollectItem",
            Self::KillMob { .. } => "KillMob",
            Self::VisitLocation { .. } => "VisitLocation",
            Self::TalkToNpc { .. } => "TalkToNpc",
            Self::DungeonClear { .. } => "DungeonClear",
        }
    }

    /// Get the target count for countable steps (CollectItem, KillMob)
    pub fn target_count(&self) -> Option<u32> {
        match self {
            Self::CollectItem { count, .. } => Some(*count),
            Self::KillMob { count, .. } => Some(*count),
            _ => None,
        }
    }
}

/// Rewards granted on quest completion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestRewards {
    /// XP awarded
    #[serde(default)]
    pub xp: u32,

    /// Currency awarded
    #[serde(default)]
    pub currency: u32,

    /// Items awarded (item_id, count)
    #[serde(default)]
    pub items: Vec<(ItemId, u32)>,

    /// Unlocks (e.g., new areas, features) - future use
    #[serde(default)]
    pub unlocks: Vec<String>,
}

impl QuestRewards {
    /// Check if there are any rewards
    pub fn has_any(&self) -> bool {
        self.xp > 0 || self.currency > 0 || !self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_step_type() {
        let step = QuestStep::KillMob {
            mob_id: MobDefId::new("cave_rat"),
            count: 5,
            description: "Kill 5 cave rats".to_string(),
        };
        assert_eq!(step.step_type(), "KillMob");
        assert_eq!(step.target_count(), Some(5));
    }

    #[test]
    fn test_quest_step_visit_location() {
        let step = QuestStep::VisitLocation {
            region_id: RegionId::new("mine_entrance"),
            description: "Enter the mine".to_string(),
        };
        assert_eq!(step.step_type(), "VisitLocation");
        assert_eq!(step.target_count(), None);
    }

    #[test]
    fn test_toml_deserialization() {
        let toml_str = r#"
            quest_id = "clear_rats"
            title = "Pest Control"
            description_short = "Clear the rats"
            description_long = "Long description here"
            repeatable = false

            [prerequisites]
            completed_quests = ["tutorial"]

            [[steps]]
            type = "KillMob"
            mob_id = { raw = 100 }
            count = 5
            description = "Kill 5 rats"

            [[steps]]
            type = "TalkToNpc"
            npc_id = "village_elder"
            description = "Return to elder"

            [rewards]
            xp = 100
            currency = 50
        "#;

        let quest: QuestDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(quest.quest_id.as_str(), "clear_rats");
        assert_eq!(quest.steps.len(), 2);
        assert_eq!(quest.rewards.xp, 100);
    }

    #[test]
    fn test_prerequisites_has_any() {
        let empty = QuestPrerequisites::default();
        assert!(!empty.has_any());

        let with_quest = QuestPrerequisites {
            completed_quests: vec![QuestId::new("tutorial")],
            ..Default::default()
        };
        assert!(with_quest.has_any());
    }
}
