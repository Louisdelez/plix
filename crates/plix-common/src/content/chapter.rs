//! Chapter definitions for campaign structure

use serde::{Deserialize, Serialize};

use super::ids::{ChapterId, QuestId};

/// A narrative container grouping related quests into a campaign chapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterDefinition {
    /// Unique stable identifier (e.g., "the_broken_gate")
    pub chapter_id: ChapterId,

    /// Display title shown to players
    pub title: String,

    /// Introduction text shown when chapter begins
    pub intro_text: String,

    /// Outro text shown when all mainline quests complete
    pub outro_text: String,

    /// Ordered list of mainline quest IDs (must complete in sequence)
    pub mainline_quests: Vec<QuestId>,

    /// Optional side quests (can complete in any order)
    #[serde(default)]
    pub side_quests: Vec<QuestId>,
}

impl ChapterDefinition {
    /// Get all quest IDs referenced by this chapter
    pub fn all_quest_ids(&self) -> Vec<&QuestId> {
        self.mainline_quests
            .iter()
            .chain(self.side_quests.iter())
            .collect()
    }

    /// Check if a quest is part of the mainline
    pub fn is_mainline(&self, quest_id: &QuestId) -> bool {
        self.mainline_quests.contains(quest_id)
    }

    /// Get the position of a mainline quest (0-indexed)
    pub fn mainline_position(&self, quest_id: &QuestId) -> Option<usize> {
        self.mainline_quests.iter().position(|q| q == quest_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_chapter() -> ChapterDefinition {
        ChapterDefinition {
            chapter_id: ChapterId::new("the_broken_gate"),
            title: "The Broken Gate".to_string(),
            intro_text: "Something stirs beneath the old mine...".to_string(),
            outro_text: "The Gate Warden is defeated.".to_string(),
            mainline_quests: vec![
                QuestId::new("tutorial_elder"),
                QuestId::new("clear_rats"),
                QuestId::new("defeat_warden"),
            ],
            side_quests: vec![QuestId::new("collect_mushrooms")],
        }
    }

    #[test]
    fn test_all_quest_ids() {
        let chapter = sample_chapter();
        let all_ids = chapter.all_quest_ids();
        assert_eq!(all_ids.len(), 4);
    }

    #[test]
    fn test_is_mainline() {
        let chapter = sample_chapter();
        assert!(chapter.is_mainline(&QuestId::new("clear_rats")));
        assert!(!chapter.is_mainline(&QuestId::new("collect_mushrooms")));
    }

    #[test]
    fn test_mainline_position() {
        let chapter = sample_chapter();
        assert_eq!(
            chapter.mainline_position(&QuestId::new("tutorial_elder")),
            Some(0)
        );
        assert_eq!(
            chapter.mainline_position(&QuestId::new("defeat_warden")),
            Some(2)
        );
        assert_eq!(
            chapter.mainline_position(&QuestId::new("collect_mushrooms")),
            None
        );
    }

    #[test]
    fn test_toml_deserialization() {
        let toml_str = r#"
            chapter_id = "the_broken_gate"
            title = "The Broken Gate"
            intro_text = "Something stirs..."
            outro_text = "Victory!"
            mainline_quests = ["quest_1", "quest_2"]
            side_quests = ["side_1"]
        "#;

        let chapter: ChapterDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(chapter.chapter_id.as_str(), "the_broken_gate");
        assert_eq!(chapter.mainline_quests.len(), 2);
        assert_eq!(chapter.side_quests.len(), 1);
    }
}
