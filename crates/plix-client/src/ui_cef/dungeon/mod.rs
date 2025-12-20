//! Dungeon HUD CEF Bridge (Feature 043, T072)
//!
//! Handles dungeon-related protocol messages and sends them to the CEF overlay
//! for display in the dungeon HUD.

use plix_common::protocol::{
    ChestAvailablePayload, DungeonCompletedPayload, DungeonEnteredPayload, DungeonStatePayload,
    ServerMessage,
};
use serde::Serialize;
use tracing::{debug, warn};

/// Dungeon HUD bridge state
pub struct DungeonHudBridge {
    /// Currently in a dungeon
    in_dungeon: bool,
    /// Current dungeon ID
    dungeon_id: Option<String>,
}

impl Default for DungeonHudBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl DungeonHudBridge {
    /// Create a new dungeon HUD bridge.
    pub fn new() -> Self {
        Self {
            in_dungeon: false,
            dungeon_id: None,
        }
    }

    /// Handle incoming server message.
    /// Returns a CEF message to send if applicable.
    pub fn handle_message(&mut self, msg: &ServerMessage) -> Option<CefDungeonMessage> {
        match msg {
            ServerMessage::DungeonEntered(payload) => {
                self.on_dungeon_entered(payload);
                Some(CefDungeonMessage::DungeonEntered(payload.clone().into()))
            }
            ServerMessage::DungeonStateUpdate(payload) => {
                if self.in_dungeon && self.dungeon_id.as_deref() == Some(&payload.dungeon_id) {
                    Some(CefDungeonMessage::DungeonStateUpdate(payload.clone().into()))
                } else {
                    None
                }
            }
            ServerMessage::ChestAvailable(payload) => {
                if self.in_dungeon && self.dungeon_id.as_deref() == Some(&payload.dungeon_id) {
                    Some(CefDungeonMessage::ChestAvailable(payload.clone().into()))
                } else {
                    None
                }
            }
            ServerMessage::DungeonCompleted(payload) => {
                if self.in_dungeon && self.dungeon_id.as_deref() == Some(&payload.dungeon_id) {
                    self.on_dungeon_completed(payload);
                    Some(CefDungeonMessage::DungeonCompleted(payload.clone().into()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Handle entering a dungeon.
    fn on_dungeon_entered(&mut self, payload: &DungeonEnteredPayload) {
        debug!(
            dungeon_id = %payload.dungeon_id,
            display_name = %payload.display_name,
            "Entered dungeon"
        );
        self.in_dungeon = true;
        self.dungeon_id = Some(payload.dungeon_id.clone());
    }

    /// Handle completing a dungeon.
    fn on_dungeon_completed(&mut self, payload: &DungeonCompletedPayload) {
        debug!(
            dungeon_id = %payload.dungeon_id,
            xp_gained = payload.xp_gained,
            "Dungeon completed"
        );
        // Note: We don't immediately clear state - the HUD will handle the delay
    }

    /// Manually leave the dungeon (e.g., on disconnect).
    pub fn leave_dungeon(&mut self) {
        if self.in_dungeon {
            debug!(dungeon_id = ?self.dungeon_id, "Left dungeon");
        }
        self.in_dungeon = false;
        self.dungeon_id = None;
    }

    /// Check if currently in a dungeon.
    pub fn is_in_dungeon(&self) -> bool {
        self.in_dungeon
    }

    /// Get the current dungeon ID.
    pub fn current_dungeon_id(&self) -> Option<&str> {
        self.dungeon_id.as_deref()
    }
}

/// CEF message for dungeon HUD updates
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum CefDungeonMessage {
    /// Player entered a dungeon
    DungeonEntered(CefDungeonEntered),
    /// Dungeon state update
    DungeonStateUpdate(CefDungeonState),
    /// Reward chest available
    ChestAvailable(CefChestAvailable),
    /// Dungeon completed
    DungeonCompleted(CefDungeonCompleted),
}

/// CEF payload for dungeon entered
#[derive(Debug, Clone, Serialize)]
pub struct CefDungeonEntered {
    pub dungeon_id: String,
    pub display_name: String,
    pub objective: String,
}

impl From<DungeonEnteredPayload> for CefDungeonEntered {
    fn from(p: DungeonEnteredPayload) -> Self {
        Self {
            dungeon_id: p.dungeon_id,
            display_name: p.display_name,
            objective: p.objective,
        }
    }
}

/// CEF payload for dungeon state update
#[derive(Debug, Clone, Serialize)]
pub struct CefDungeonState {
    pub dungeon_id: String,
    pub boss_alive: bool,
    pub boss_hp_percent: Option<f32>,
    pub chest_available: bool,
}

impl From<DungeonStatePayload> for CefDungeonState {
    fn from(p: DungeonStatePayload) -> Self {
        Self {
            dungeon_id: p.dungeon_id,
            boss_alive: p.boss_alive,
            boss_hp_percent: p.boss_hp_percent,
            chest_available: p.chest_available,
        }
    }
}

/// CEF payload for chest available
#[derive(Debug, Clone, Serialize)]
pub struct CefChestAvailable {
    pub chest_id: u64,
    pub position: [f32; 3],
    pub dungeon_id: String,
}

impl From<ChestAvailablePayload> for CefChestAvailable {
    fn from(p: ChestAvailablePayload) -> Self {
        Self {
            chest_id: p.chest_id,
            position: p.position,
            dungeon_id: p.dungeon_id,
        }
    }
}

/// CEF payload for dungeon completed
#[derive(Debug, Clone, Serialize)]
pub struct CefDungeonCompleted {
    pub dungeon_id: String,
    pub display_name: String,
    pub rewards: Vec<(String, u32)>,
    pub xp_gained: u32,
}

impl From<DungeonCompletedPayload> for CefDungeonCompleted {
    fn from(p: DungeonCompletedPayload) -> Self {
        Self {
            dungeon_id: p.dungeon_id,
            display_name: p.display_name,
            rewards: p.rewards,
            xp_gained: p.xp_gained,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dungeon_enter_exit() {
        let mut bridge = DungeonHudBridge::new();

        assert!(!bridge.is_in_dungeon());
        assert!(bridge.current_dungeon_id().is_none());

        // Enter dungeon
        let payload = DungeonEnteredPayload {
            dungeon_id: "crypt_of_gate".to_string(),
            display_name: "Crypt of the Gate".to_string(),
            objective: "Defeat the Gate Warden".to_string(),
        };
        let msg = ServerMessage::DungeonEntered(payload);
        let result = bridge.handle_message(&msg);

        assert!(result.is_some());
        assert!(bridge.is_in_dungeon());
        assert_eq!(bridge.current_dungeon_id(), Some("crypt_of_gate"));

        // Leave dungeon
        bridge.leave_dungeon();
        assert!(!bridge.is_in_dungeon());
        assert!(bridge.current_dungeon_id().is_none());
    }

    #[test]
    fn test_state_update_filtering() {
        let mut bridge = DungeonHudBridge::new();

        // State update without entering should be ignored
        let state_payload = DungeonStatePayload {
            dungeon_id: "crypt_of_gate".to_string(),
            boss_alive: true,
            boss_hp_percent: Some(0.75),
            chest_available: false,
        };
        let msg = ServerMessage::DungeonStateUpdate(state_payload.clone());
        assert!(bridge.handle_message(&msg).is_none());

        // Enter dungeon
        let enter_payload = DungeonEnteredPayload {
            dungeon_id: "crypt_of_gate".to_string(),
            display_name: "Crypt of the Gate".to_string(),
            objective: "Defeat the Gate Warden".to_string(),
        };
        bridge.handle_message(&ServerMessage::DungeonEntered(enter_payload));

        // Now state update should work
        let result = bridge.handle_message(&msg);
        assert!(result.is_some());

        // Wrong dungeon ID should be ignored
        let wrong_state = DungeonStatePayload {
            dungeon_id: "other_dungeon".to_string(),
            boss_alive: true,
            boss_hp_percent: Some(1.0),
            chest_available: false,
        };
        assert!(bridge
            .handle_message(&ServerMessage::DungeonStateUpdate(wrong_state))
            .is_none());
    }
}
