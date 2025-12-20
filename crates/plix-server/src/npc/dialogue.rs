//! NPC Dialogue System (T080-T088)
//!
//! Handles NPC interactions, dialogue sessions, and quest integration.

use std::collections::HashMap;

use plix_common::content::ids::{NpcId, QuestId};
use plix_common::content::npc::NpcDefinition;
use plix_common::protocol::{DialogueShowPayload, DialogueOption, DialogueOptionType, ServerMessage};
use plix_common::types::PlayerId;
use tracing::{debug, info, warn};

/// Dialogue response options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogueResponse {
    /// Accept offered quest
    Accept,
    /// Decline offered quest
    Decline,
    /// Continue dialogue
    Continue,
    /// Close dialogue
    Close,
}

/// Current dialogue state for a player
#[derive(Debug, Clone)]
pub struct DialogueSession {
    /// NPC being talked to
    pub npc_id: NpcId,
    /// Current dialogue type
    pub dialogue_type: DialogueType,
    /// Quest being offered/completed (if any)
    pub quest_id: Option<QuestId>,
    /// Current line index
    pub current_line: usize,
    /// Total lines in current dialogue
    pub total_lines: usize,
}

/// Type of dialogue being shown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogueType {
    /// Idle/ambient dialogue
    Idle,
    /// Offering a quest
    QuestOffer,
    /// Quest completion
    QuestComplete,
}

/// Dialogue handler manages active sessions and generates protocol messages
pub struct DialogueHandler {
    /// Active dialogue sessions by player ID
    sessions: HashMap<PlayerId, DialogueSession>,
    /// Pending protocol messages
    pending_messages: Vec<(PlayerId, ServerMessage)>,
}

impl Default for DialogueHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogueHandler {
    /// Create a new dialogue handler.
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            pending_messages: Vec::new(),
        }
    }

    /// Start an interaction with an NPC (T080, T081).
    ///
    /// Determines the appropriate dialogue based on player's quest state.
    pub fn start_interaction(
        &mut self,
        player_id: PlayerId,
        npc: &NpcDefinition,
        player_active_quests: &[QuestId],
        player_completed_quests: &[QuestId],
        player_available_quests: &[QuestId],
    ) {
        // Check if already in dialogue
        if self.sessions.contains_key(&player_id) {
            warn!(player_id = ?player_id, "Player already in dialogue");
            return;
        }

        // Determine dialogue type
        let (dialogue_type, quest_id, lines, options) = self.determine_dialogue(
            npc,
            player_active_quests,
            player_completed_quests,
            player_available_quests,
        );

        if lines.is_empty() {
            // No dialogue available
            debug!(
                player_id = ?player_id,
                npc_id = %npc.npc_id,
                "No dialogue available for NPC"
            );
            return;
        }

        // Create session
        let session = DialogueSession {
            npc_id: npc.npc_id.clone(),
            dialogue_type,
            quest_id: quest_id.clone(),
            current_line: 0,
            total_lines: lines.len(),
        };
        self.sessions.insert(player_id, session);

        // Send dialogue message
        self.send_dialogue(player_id, npc, lines, options, quest_id);
    }

    /// Determine which dialogue to show based on quest state (T081).
    fn determine_dialogue(
        &self,
        npc: &NpcDefinition,
        player_active: &[QuestId],
        player_completed: &[QuestId],
        player_available: &[QuestId],
    ) -> (DialogueType, Option<QuestId>, Vec<String>, Vec<DialogueOption>) {
        // Priority 1: Quest completion (player has completed quest steps, needs to turn in)
        for quest_id in npc.quest_giver.iter() {
            if player_active.contains(quest_id) {
                // Check if quest is ready for turn-in (would need more state)
                // For now, check completion dialogue exists
                if let Some(lines) = npc.get_quest_complete(quest_id) {
                    return (
                        DialogueType::QuestComplete,
                        Some(quest_id.clone()),
                        lines.clone(),
                        vec![DialogueOption {
                            option_id: "complete".to_string(),
                            text: "Thank you!".to_string(),
                            option_type: DialogueOptionType::Close,
                        }],
                    );
                }
            }
        }

        // Priority 2: Quest offer (player can accept a new quest)
        for quest_id in npc.quest_giver.iter() {
            // Skip if already active or completed
            if player_active.contains(quest_id) || player_completed.contains(quest_id) {
                continue;
            }

            // Check if available (prerequisites met)
            if !player_available.contains(quest_id) {
                continue;
            }

            if let Some(offer) = npc.get_quest_offer(quest_id) {
                return (
                    DialogueType::QuestOffer,
                    Some(quest_id.clone()),
                    offer.offer_lines.clone(),
                    vec![
                        DialogueOption {
                            option_id: "accept".to_string(),
                            text: offer.accept_text.clone(),
                            option_type: DialogueOptionType::AcceptQuest,
                        },
                        DialogueOption {
                            option_id: "decline".to_string(),
                            text: offer.decline_text.clone(),
                            option_type: DialogueOptionType::DeclineQuest,
                        },
                    ],
                );
            }
        }

        // Priority 3: Idle dialogue
        (
            DialogueType::Idle,
            None,
            npc.idle_dialogue.clone(),
            vec![DialogueOption {
                option_id: "close".to_string(),
                text: "Goodbye".to_string(),
                option_type: DialogueOptionType::Close,
            }],
        )
    }

    /// Send dialogue message to player (T083).
    fn send_dialogue(
        &mut self,
        player_id: PlayerId,
        npc: &NpcDefinition,
        lines: Vec<String>,
        options: Vec<DialogueOption>,
        _quest_id: Option<QuestId>,
    ) {
        self.pending_messages.push((
            player_id,
            ServerMessage::DialogueShow(DialogueShowPayload {
                npc_id: npc.npc_id.as_str().to_string(),
                npc_name: npc.name.clone(),
                lines,
                options,
            }),
        ));

        info!(
            player_id = ?player_id,
            npc_id = %npc.npc_id,
            "Started dialogue session"
        );
    }

    /// Handle player dialogue response (T084).
    pub fn handle_response(
        &mut self,
        player_id: PlayerId,
        response_id: &str,
    ) -> Option<(DialogueType, Option<QuestId>, DialogueResponse)> {
        let session = match self.sessions.remove(&player_id) {
            Some(s) => s,
            None => {
                warn!(player_id = ?player_id, "No active dialogue session");
                return None;
            }
        };

        let response = match response_id {
            "accept" => DialogueResponse::Accept,
            "decline" => DialogueResponse::Decline,
            "continue" => DialogueResponse::Continue,
            "close" | "complete" => DialogueResponse::Close,
            _ => {
                warn!(response_id = %response_id, "Unknown dialogue response");
                DialogueResponse::Close
            }
        };

        debug!(
            player_id = ?player_id,
            npc_id = %session.npc_id,
            response = ?response,
            "Dialogue response received"
        );

        Some((session.dialogue_type, session.quest_id, response))
    }

    /// Close a dialogue session.
    pub fn close_session(&mut self, player_id: PlayerId) {
        if self.sessions.remove(&player_id).is_some() {
            debug!(player_id = ?player_id, "Dialogue session closed");
        }
    }

    /// Check if player is in a dialogue session.
    pub fn is_in_dialogue(&self, player_id: PlayerId) -> bool {
        self.sessions.contains_key(&player_id)
    }

    /// Get the current session for a player.
    pub fn get_session(&self, player_id: PlayerId) -> Option<&DialogueSession> {
        self.sessions.get(&player_id)
    }

    /// Drain pending protocol messages.
    pub fn drain_messages(&mut self) -> Vec<(PlayerId, ServerMessage)> {
        std::mem::take(&mut self.pending_messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::content::npc::QuestDialogue;

    fn sample_npc() -> NpcDefinition {
        let mut quest_offer = HashMap::new();
        quest_offer.insert(
            "clear_rats".to_string(),
            QuestDialogue {
                offer_lines: vec!["Help me clear the rats!".to_string()],
                accept_text: "I'll do it".to_string(),
                decline_text: "Maybe later".to_string(),
            },
        );

        let mut quest_complete = HashMap::new();
        quest_complete.insert(
            "clear_rats".to_string(),
            vec!["Thank you for your help!".to_string()],
        );

        NpcDefinition {
            npc_id: NpcId::new("elder"),
            name: "Elder".to_string(),
            position: plix_common::math::Vec3::new(0.0, 0.0, 0.0),
            rotation: 0.0,
            quest_giver: vec![QuestId::new("clear_rats")],
            idle_dialogue: vec!["Greetings!".to_string()],
            quest_offer_dialogue: quest_offer,
            quest_complete_dialogue: quest_complete,
        }
    }

    #[test]
    fn test_start_interaction_idle() {
        let mut handler = DialogueHandler::new();
        let npc = sample_npc();

        // No quests available -> idle dialogue
        handler.start_interaction(
            PlayerId(1),
            &npc,
            &[QuestId::new("clear_rats")], // already active
            &[],
            &[],
        );

        assert!(handler.is_in_dialogue(PlayerId(1)));
        let session = handler.get_session(PlayerId(1)).unwrap();
        assert_eq!(session.dialogue_type, DialogueType::Idle);
    }

    #[test]
    fn test_start_interaction_quest_offer() {
        let mut handler = DialogueHandler::new();
        let npc = sample_npc();

        // Quest available -> offer dialogue
        handler.start_interaction(
            PlayerId(1),
            &npc,
            &[], // no active quests
            &[], // no completed quests
            &[QuestId::new("clear_rats")], // quest is available
        );

        assert!(handler.is_in_dialogue(PlayerId(1)));
        let session = handler.get_session(PlayerId(1)).unwrap();
        assert_eq!(session.dialogue_type, DialogueType::QuestOffer);
        assert_eq!(session.quest_id, Some(QuestId::new("clear_rats")));
    }

    #[test]
    fn test_handle_response() {
        let mut handler = DialogueHandler::new();
        let npc = sample_npc();

        handler.start_interaction(
            PlayerId(1),
            &npc,
            &[],
            &[],
            &[QuestId::new("clear_rats")],
        );

        let result = handler.handle_response(PlayerId(1), "accept");
        assert!(result.is_some());

        let (dialogue_type, quest_id, response) = result.unwrap();
        assert_eq!(dialogue_type, DialogueType::QuestOffer);
        assert_eq!(quest_id, Some(QuestId::new("clear_rats")));
        assert_eq!(response, DialogueResponse::Accept);

        // Session should be closed
        assert!(!handler.is_in_dialogue(PlayerId(1)));
    }

    #[test]
    fn test_pending_messages() {
        let mut handler = DialogueHandler::new();
        let npc = sample_npc();

        handler.start_interaction(
            PlayerId(1),
            &npc,
            &[],
            &[],
            &[QuestId::new("clear_rats")],
        );

        let messages = handler.drain_messages();
        assert_eq!(messages.len(), 1);
        assert!(matches!(
            messages[0].1,
            ServerMessage::DialogueShow(_)
        ));
    }
}
