//! Dialogue CEF Bridge (Feature 043, T087)
//!
//! Handles NPC dialogue protocol messages and sends them to the CEF overlay
//! for display in the dialogue panel.

use plix_common::protocol::{DialogueShowPayload, ServerMessage};
use serde::Serialize;
use tracing::debug;

/// Dialogue bridge state
pub struct DialogueBridge {
    /// Currently in a dialogue
    in_dialogue: bool,
    /// Current NPC ID
    npc_id: Option<String>,
}

impl Default for DialogueBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogueBridge {
    /// Create a new dialogue bridge.
    pub fn new() -> Self {
        Self {
            in_dialogue: false,
            npc_id: None,
        }
    }

    /// Handle incoming server message.
    /// Returns a CEF message to send if applicable.
    pub fn handle_message(&mut self, msg: &ServerMessage) -> Option<CefDialogueMessage> {
        match msg {
            ServerMessage::DialogueShow(payload) => {
                self.on_dialogue_show(payload);
                Some(CefDialogueMessage::DialogueShow(payload.clone().into()))
            }
            _ => None,
        }
    }

    /// Handle showing a dialogue.
    fn on_dialogue_show(&mut self, payload: &DialogueShowPayload) {
        debug!(
            npc_id = %payload.npc_id,
            npc_name = %payload.npc_name,
            lines = payload.lines.len(),
            options = payload.options.len(),
            "Dialogue started"
        );
        self.in_dialogue = true;
        self.npc_id = Some(payload.npc_id.clone());
    }

    /// Close the current dialogue.
    pub fn close_dialogue(&mut self) {
        if self.in_dialogue {
            debug!(npc_id = ?self.npc_id, "Dialogue closed");
        }
        self.in_dialogue = false;
        self.npc_id = None;
    }

    /// Check if currently in a dialogue.
    pub fn is_in_dialogue(&self) -> bool {
        self.in_dialogue
    }

    /// Get the current NPC ID.
    pub fn current_npc_id(&self) -> Option<&str> {
        self.npc_id.as_deref()
    }
}

/// CEF message for dialogue updates
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum CefDialogueMessage {
    /// Show dialogue
    DialogueShow(CefDialogueShow),
}

/// CEF payload for dialogue show
#[derive(Debug, Clone, Serialize)]
pub struct CefDialogueShow {
    pub npc_id: String,
    pub npc_name: String,
    pub lines: Vec<String>,
    pub options: Vec<CefDialogueOption>,
}

/// CEF payload for a dialogue option
#[derive(Debug, Clone, Serialize)]
pub struct CefDialogueOption {
    pub option_id: String,
    pub text: String,
    pub option_type: String,
}

impl From<DialogueShowPayload> for CefDialogueShow {
    fn from(p: DialogueShowPayload) -> Self {
        Self {
            npc_id: p.npc_id,
            npc_name: p.npc_name,
            lines: p.lines,
            options: p
                .options
                .into_iter()
                .map(|o| CefDialogueOption {
                    option_id: o.option_id,
                    text: o.text,
                    option_type: format!("{:?}", o.option_type),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::protocol::{DialogueOption, DialogueOptionType};

    #[test]
    fn test_dialogue_open_close() {
        let mut bridge = DialogueBridge::new();

        assert!(!bridge.is_in_dialogue());
        assert!(bridge.current_npc_id().is_none());

        // Show dialogue
        let payload = DialogueShowPayload {
            npc_id: "elder".to_string(),
            npc_name: "Elder".to_string(),
            lines: vec!["Greetings, traveler!".to_string()],
            options: vec![DialogueOption {
                option_id: "close".to_string(),
                text: "Goodbye".to_string(),
                option_type: DialogueOptionType::Close,
            }],
        };
        let msg = ServerMessage::DialogueShow(payload);
        let result = bridge.handle_message(&msg);

        assert!(result.is_some());
        assert!(bridge.is_in_dialogue());
        assert_eq!(bridge.current_npc_id(), Some("elder"));

        // Close dialogue
        bridge.close_dialogue();
        assert!(!bridge.is_in_dialogue());
        assert!(bridge.current_npc_id().is_none());
    }

    #[test]
    fn test_cef_payload_conversion() {
        let payload = DialogueShowPayload {
            npc_id: "elder".to_string(),
            npc_name: "Village Elder".to_string(),
            lines: vec![
                "Welcome to our village!".to_string(),
                "We need your help.".to_string(),
            ],
            options: vec![
                DialogueOption {
                    option_id: "accept".to_string(),
                    text: "I'll help you".to_string(),
                    option_type: DialogueOptionType::AcceptQuest,
                },
                DialogueOption {
                    option_id: "decline".to_string(),
                    text: "Not interested".to_string(),
                    option_type: DialogueOptionType::DeclineQuest,
                },
            ],
        };

        let cef: CefDialogueShow = payload.into();

        assert_eq!(cef.npc_id, "elder");
        assert_eq!(cef.npc_name, "Village Elder");
        assert_eq!(cef.lines.len(), 2);
        assert_eq!(cef.options.len(), 2);
        assert_eq!(cef.options[0].option_id, "accept");
        assert_eq!(cef.options[0].option_type, "AcceptQuest");
        assert_eq!(cef.options[1].option_id, "decline");
        assert_eq!(cef.options[1].option_type, "DeclineQuest");
    }
}
