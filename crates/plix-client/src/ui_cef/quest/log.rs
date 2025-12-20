//! Quest Log CEF bridge (Feature 043)
//!
//! Handles communication between the quest_log.html/js page and the Rust client.
//! Routes messages to/from the server for quest operations.

use plix_common::content::ids::QuestId;
use plix_common::protocol::{
    ActiveQuestInfo, ClientMessage, QuestSyncPayload, QuestUpdatePayload,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, warn};

use crate::ui_cef::bridge::messages::{BridgeError, BridgePush, BridgeResponse, PushType};

/// Bridge for Quest Log page communication
pub struct QuestLogBridge {
    /// Last known sync data (cached for UI responsiveness)
    last_sync: Option<QuestSyncPayload>,
    /// Whether the quest log page is currently open
    page_open: bool,
}

impl Default for QuestLogBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl QuestLogBridge {
    /// Create a new quest log bridge
    pub fn new() -> Self {
        Self {
            last_sync: None,
            page_open: false,
        }
    }

    /// Mark the quest log page as open (for targeted updates)
    pub fn set_page_open(&mut self, open: bool) {
        self.page_open = open;
        debug!(open = open, "Quest log page state changed");
    }

    /// Check if quest log page is open
    pub fn is_page_open(&self) -> bool {
        self.page_open
    }

    /// Handle QuestSyncRequest from JS - returns client message to send to server
    pub fn handle_sync_request(&self, id: &str) -> (BridgeResponse, Option<ClientMessage>) {
        debug!("Quest sync requested from UI");

        // Return success response and indicate server should be queried
        let response = BridgeResponse::success_empty(id, "QuestSyncRequest");
        let client_msg = Some(ClientMessage::QuestSyncRequest);

        (response, client_msg)
    }

    /// Handle QuestAccept from JS - returns client message to send to server
    pub fn handle_quest_accept(
        &self,
        id: &str,
        payload: &Value,
    ) -> (BridgeResponse, Option<ClientMessage>) {
        let quest_id = match payload.get("quest_id").and_then(|v| v.as_str()) {
            Some(qid) => qid.to_string(),
            None => {
                return (
                    BridgeResponse::error(
                        id,
                        "QuestAccept",
                        BridgeError::invalid_format("Missing quest_id"),
                    ),
                    None,
                );
            }
        };

        debug!(quest_id = %quest_id, "Quest accept requested");

        let response = BridgeResponse::success_empty(id, "QuestAccept");
        let client_msg = Some(ClientMessage::QuestAccept {
            quest_id: QuestId::new(&quest_id),
        });

        (response, client_msg)
    }

    /// Handle QuestAbandon from JS - returns client message to send to server
    pub fn handle_quest_abandon(
        &self,
        id: &str,
        payload: &Value,
    ) -> (BridgeResponse, Option<ClientMessage>) {
        let quest_id = match payload.get("quest_id").and_then(|v| v.as_str()) {
            Some(qid) => qid.to_string(),
            None => {
                return (
                    BridgeResponse::error(
                        id,
                        "QuestAbandon",
                        BridgeError::invalid_format("Missing quest_id"),
                    ),
                    None,
                );
            }
        };

        debug!(quest_id = %quest_id, "Quest abandon requested");

        let response = BridgeResponse::success_empty(id, "QuestAbandon");
        let client_msg = Some(ClientMessage::QuestAbandon {
            quest_id: QuestId::new(&quest_id),
        });

        (response, client_msg)
    }

    /// Handle QuestPin from JS - returns client message to send to server
    pub fn handle_quest_pin(
        &self,
        id: &str,
        payload: &Value,
    ) -> (BridgeResponse, Option<ClientMessage>) {
        // quest_id can be null to unpin
        let quest_id = payload
            .get("quest_id")
            .and_then(|v| v.as_str())
            .map(|s| QuestId::new(s));

        debug!(quest_id = ?quest_id, "Quest pin requested");

        let response = BridgeResponse::success_empty(id, "QuestPin");
        let client_msg = Some(ClientMessage::QuestPin { quest_id });

        (response, client_msg)
    }

    /// Update cached sync data from server response
    pub fn on_quest_sync(&mut self, sync: QuestSyncPayload) {
        debug!(
            active = sync.active_quests.len(),
            completed = sync.completed_quest_ids.len(),
            "Quest sync received"
        );
        self.last_sync = Some(sync);
    }

    /// Create push event for quest sync
    pub fn push_quest_sync(sync: &QuestSyncPayload) -> BridgePush {
        BridgePush::new(
            PushType::QuestSync.as_str(),
            serde_json::to_value(QuestSyncPushPayload::from(sync)).unwrap_or_default(),
        )
    }

    /// Create push event for quest update
    pub fn push_quest_update(update: &QuestUpdatePayload) -> BridgePush {
        BridgePush::new(
            PushType::QuestUpdate.as_str(),
            serde_json::to_value(QuestUpdatePushPayload::from(update)).unwrap_or_default(),
        )
    }

    /// Get cached sync data (for quick UI responses)
    pub fn cached_sync(&self) -> Option<&QuestSyncPayload> {
        self.last_sync.as_ref()
    }
}

/// Payload for QuestSync push to JS
#[derive(Debug, Serialize)]
struct QuestSyncPushPayload {
    active_quests: Vec<ActiveQuestJs>,
    completed_quest_ids: Vec<String>,
}

impl From<&QuestSyncPayload> for QuestSyncPushPayload {
    fn from(sync: &QuestSyncPayload) -> Self {
        Self {
            active_quests: sync.active_quests.iter().map(ActiveQuestJs::from).collect(),
            completed_quest_ids: sync.completed_quest_ids.iter().map(|id| id.to_string()).collect(),
        }
    }
}

/// Active quest info for JS (camelCase field names)
#[derive(Debug, Serialize)]
struct ActiveQuestJs {
    quest_id: String,
    title: String,
    description: String,
    current_step: u32,
    steps: Vec<QuestStepJs>,
    pinned: bool,
}

impl From<&ActiveQuestInfo> for ActiveQuestJs {
    fn from(quest: &ActiveQuestInfo) -> Self {
        Self {
            quest_id: quest.quest_id.to_string(),
            title: quest.title.clone(),
            description: quest.description.clone(),
            current_step: quest.current_step,
            steps: quest.steps.iter().map(QuestStepJs::from).collect(),
            pinned: quest.pinned,
        }
    }
}

/// Quest step info for JS
#[derive(Debug, Serialize)]
struct QuestStepJs {
    description: String,
    step_type: String,
    progress: Option<StepProgressJs>,
}

impl From<&plix_common::protocol::QuestStepInfo> for QuestStepJs {
    fn from(step: &plix_common::protocol::QuestStepInfo) -> Self {
        Self {
            description: step.description.clone(),
            step_type: step.step_type.clone(),
            progress: step.progress.as_ref().map(StepProgressJs::from),
        }
    }
}

/// Step progress for JS
#[derive(Debug, Serialize)]
struct StepProgressJs {
    current: u32,
    target: u32,
    completed: bool,
}

impl From<&plix_common::protocol::StepProgressInfo> for StepProgressJs {
    fn from(progress: &plix_common::protocol::StepProgressInfo) -> Self {
        Self {
            current: progress.current,
            target: progress.target,
            completed: progress.completed,
        }
    }
}

/// Payload for QuestUpdate push to JS
#[derive(Debug, Serialize)]
struct QuestUpdatePushPayload {
    quest_id: String,
    update_type: String,
    step_index: Option<u32>,
    progress: Option<u32>,
    target: Option<u32>,
}

impl From<&QuestUpdatePayload> for QuestUpdatePushPayload {
    fn from(update: &QuestUpdatePayload) -> Self {
        Self {
            quest_id: update.quest_id.to_string(),
            update_type: format!("{:?}", update.update_type),
            step_index: update.step_index,
            progress: update.progress,
            target: update.target,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::protocol::QuestUpdateType;

    #[test]
    fn test_quest_log_bridge_new() {
        let bridge = QuestLogBridge::new();
        assert!(!bridge.is_page_open());
        assert!(bridge.cached_sync().is_none());
    }

    #[test]
    fn test_quest_log_bridge_page_state() {
        let mut bridge = QuestLogBridge::new();

        bridge.set_page_open(true);
        assert!(bridge.is_page_open());

        bridge.set_page_open(false);
        assert!(!bridge.is_page_open());
    }

    #[test]
    fn test_handle_sync_request() {
        let bridge = QuestLogBridge::new();
        let (response, client_msg) = bridge.handle_sync_request("req-1");

        assert!(response.ok);
        assert!(matches!(client_msg, Some(ClientMessage::QuestSyncRequest)));
    }

    #[test]
    fn test_handle_quest_accept() {
        let bridge = QuestLogBridge::new();
        let payload = serde_json::json!({"quest_id": "test_quest"});
        let (response, client_msg) = bridge.handle_quest_accept("req-1", &payload);

        assert!(response.ok);
        match client_msg {
            Some(ClientMessage::QuestAccept { quest_id }) => {
                assert_eq!(quest_id.to_string(), "test_quest");
            }
            _ => panic!("Expected QuestAccept client message"),
        }
    }

    #[test]
    fn test_handle_quest_accept_missing_id() {
        let bridge = QuestLogBridge::new();
        let payload = serde_json::json!({});
        let (response, client_msg) = bridge.handle_quest_accept("req-1", &payload);

        assert!(!response.ok);
        assert!(client_msg.is_none());
    }

    #[test]
    fn test_handle_quest_abandon() {
        let bridge = QuestLogBridge::new();
        let payload = serde_json::json!({"quest_id": "test_quest"});
        let (response, client_msg) = bridge.handle_quest_abandon("req-1", &payload);

        assert!(response.ok);
        match client_msg {
            Some(ClientMessage::QuestAbandon { quest_id }) => {
                assert_eq!(quest_id.to_string(), "test_quest");
            }
            _ => panic!("Expected QuestAbandon client message"),
        }
    }

    #[test]
    fn test_handle_quest_pin() {
        let bridge = QuestLogBridge::new();

        // Pin a quest
        let payload = serde_json::json!({"quest_id": "test_quest"});
        let (response, client_msg) = bridge.handle_quest_pin("req-1", &payload);
        assert!(response.ok);
        match client_msg {
            Some(ClientMessage::QuestPin { quest_id }) => {
                assert_eq!(quest_id.as_ref().map(|q| q.to_string()), Some("test_quest".to_string()));
            }
            _ => panic!("Expected QuestPin client message"),
        }

        // Unpin (null quest_id)
        let payload = serde_json::json!({"quest_id": null});
        let (response, client_msg) = bridge.handle_quest_pin("req-2", &payload);
        assert!(response.ok);
        match client_msg {
            Some(ClientMessage::QuestPin { quest_id }) => {
                assert!(quest_id.is_none());
            }
            _ => panic!("Expected QuestPin client message with None"),
        }
    }

    #[test]
    fn test_on_quest_sync() {
        let mut bridge = QuestLogBridge::new();

        let sync = QuestSyncPayload {
            active_quests: vec![],
            completed_quest_ids: vec![QuestId::new("completed_quest")],
        };

        bridge.on_quest_sync(sync);

        let cached = bridge.cached_sync().unwrap();
        assert_eq!(cached.completed_quest_ids.len(), 1);
    }

    #[test]
    fn test_push_quest_sync() {
        let sync = QuestSyncPayload {
            active_quests: vec![],
            completed_quest_ids: vec![QuestId::new("quest1")],
        };

        let push = QuestLogBridge::push_quest_sync(&sync);
        assert_eq!(push.push_type, "QuestSync");
    }

    #[test]
    fn test_push_quest_update() {
        let update = QuestUpdatePayload {
            quest_id: QuestId::new("test_quest"),
            update_type: QuestUpdateType::StepProgress,
            step_index: Some(0),
            progress: Some(2),
            target: Some(5),
        };

        let push = QuestLogBridge::push_quest_update(&update);
        assert_eq!(push.push_type, "QuestUpdate");
    }
}
