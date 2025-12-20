//! NPC definitions with dialogue and quest-giving

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ids::{NpcId, QuestId};
use crate::math::Vec3;

/// A non-player character with dialogue and quest-giving capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcDefinition {
    /// Unique stable identifier
    pub npc_id: NpcId,

    /// Display name shown to players
    pub name: String,

    /// Position in world coordinates
    pub position: Vec3,

    /// Rotation (facing direction in radians, 0 = +Z)
    #[serde(default)]
    pub rotation: f32,

    /// Quests this NPC can give
    #[serde(default)]
    pub quest_giver: Vec<QuestId>,

    /// Default dialogue lines (when no quest available)
    #[serde(default)]
    pub idle_dialogue: Vec<String>,

    /// Dialogue for offering quests (keyed by quest_id)
    #[serde(default)]
    pub quest_offer_dialogue: HashMap<String, QuestDialogue>,

    /// Dialogue for completing quests (keyed by quest_id)
    #[serde(default)]
    pub quest_complete_dialogue: HashMap<String, Vec<String>>,
}

impl NpcDefinition {
    /// Check if this NPC can give a specific quest
    pub fn can_give_quest(&self, quest_id: &QuestId) -> bool {
        self.quest_giver.contains(quest_id)
    }

    /// Get dialogue for offering a quest
    pub fn get_quest_offer(&self, quest_id: &QuestId) -> Option<&QuestDialogue> {
        self.quest_offer_dialogue.get(quest_id.as_str())
    }

    /// Get dialogue for completing a quest
    pub fn get_quest_complete(&self, quest_id: &QuestId) -> Option<&Vec<String>> {
        self.quest_complete_dialogue.get(quest_id.as_str())
    }

    /// Get a random idle dialogue line
    pub fn random_idle_line(&self) -> Option<&str> {
        if self.idle_dialogue.is_empty() {
            None
        } else {
            // Simple deterministic selection based on length
            let idx = self.idle_dialogue.len() % self.idle_dialogue.len();
            Some(&self.idle_dialogue[idx])
        }
    }
}

/// Dialogue shown when offering a quest to a player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestDialogue {
    /// Lines to show when offering quest
    pub offer_lines: Vec<String>,

    /// Text for accept button
    #[serde(default = "default_accept_text")]
    pub accept_text: String,

    /// Text for decline button
    #[serde(default = "default_decline_text")]
    pub decline_text: String,
}

fn default_accept_text() -> String {
    "Accept".to_string()
}

fn default_decline_text() -> String {
    "Decline".to_string()
}

impl QuestDialogue {
    /// Create a simple quest dialogue with default button text
    pub fn simple(lines: Vec<String>) -> Self {
        Self {
            offer_lines: lines,
            accept_text: default_accept_text(),
            decline_text: default_decline_text(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_npc() -> NpcDefinition {
        let mut quest_offer = HashMap::new();
        quest_offer.insert(
            "clear_rats".to_string(),
            QuestDialogue {
                offer_lines: vec![
                    "The mine is overrun with rats!".to_string(),
                    "Will you help clear them out?".to_string(),
                ],
                accept_text: "I'll handle it".to_string(),
                decline_text: "Not now".to_string(),
            },
        );

        let mut quest_complete = HashMap::new();
        quest_complete.insert(
            "clear_rats".to_string(),
            vec![
                "Excellent work!".to_string(),
                "The miners can return to work.".to_string(),
            ],
        );

        NpcDefinition {
            npc_id: NpcId::new("village_elder"),
            name: "Elder Theron".to_string(),
            position: Vec3::new(50.0, 10.0, 50.0),
            rotation: 0.0,
            quest_giver: vec![QuestId::new("clear_rats"), QuestId::new("defeat_warden")],
            idle_dialogue: vec![
                "Greetings, traveler.".to_string(),
                "The times grow dark...".to_string(),
            ],
            quest_offer_dialogue: quest_offer,
            quest_complete_dialogue: quest_complete,
        }
    }

    #[test]
    fn test_can_give_quest() {
        let npc = sample_npc();
        assert!(npc.can_give_quest(&QuestId::new("clear_rats")));
        assert!(!npc.can_give_quest(&QuestId::new("unknown_quest")));
    }

    #[test]
    fn test_get_quest_offer() {
        let npc = sample_npc();
        let offer = npc.get_quest_offer(&QuestId::new("clear_rats"));
        assert!(offer.is_some());
        assert_eq!(offer.unwrap().offer_lines.len(), 2);
    }

    #[test]
    fn test_get_quest_complete() {
        let npc = sample_npc();
        let complete = npc.get_quest_complete(&QuestId::new("clear_rats"));
        assert!(complete.is_some());
        assert_eq!(complete.unwrap().len(), 2);
    }

    #[test]
    fn test_random_idle_line() {
        let npc = sample_npc();
        let line = npc.random_idle_line();
        assert!(line.is_some());

        let empty_npc = NpcDefinition {
            idle_dialogue: vec![],
            ..sample_npc()
        };
        assert!(empty_npc.random_idle_line().is_none());
    }

    #[test]
    fn test_toml_deserialization() {
        let toml_str = r#"
            npc_id = "village_elder"
            name = "Elder Theron"
            position = [50.0, 10.0, 50.0]
            rotation = 0.0
            quest_giver = ["clear_rats", "defeat_warden"]
            idle_dialogue = ["Greetings!", "Dark times..."]

            [quest_offer_dialogue.clear_rats]
            offer_lines = ["The mine is overrun!", "Help?"]
            accept_text = "Sure"
            decline_text = "No"

            [quest_complete_dialogue]
            clear_rats = ["Well done!", "Here's your reward."]
        "#;

        let npc: NpcDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(npc.npc_id.as_str(), "village_elder");
        assert_eq!(npc.quest_giver.len(), 2);
        assert!(npc.quest_offer_dialogue.contains_key("clear_rats"));
    }

    #[test]
    fn test_default_button_text() {
        let dialogue = QuestDialogue::simple(vec!["Hello!".to_string()]);
        assert_eq!(dialogue.accept_text, "Accept");
        assert_eq!(dialogue.decline_text, "Decline");
    }
}
