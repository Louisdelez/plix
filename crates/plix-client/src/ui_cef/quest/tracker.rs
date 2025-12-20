//! Quest Tracker HUD bridge (Feature 043)
//!
//! Handles communication between the quest_tracker.js overlay and the Rust client.
//! The tracker displays the currently pinned quest in the game HUD.

use plix_common::content::ids::QuestId;
use plix_common::protocol::{QuestNotificationPayload, QuestTrackerPayload, QuestUpdatePayload};
use serde::Serialize;
use tracing::debug;

use crate::ui_cef::bridge::messages::{BridgePush, PushType};

/// Bridge for Quest Tracker HUD overlay
pub struct QuestTrackerBridge {
    /// Currently tracked quest state
    current_tracker: Option<QuestTrackerPayload>,
    /// Whether the tracker is visible
    visible: bool,
}

impl Default for QuestTrackerBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl QuestTrackerBridge {
    /// Create a new quest tracker bridge
    pub fn new() -> Self {
        Self {
            current_tracker: None,
            visible: false,
        }
    }

    /// Check if tracker is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get current tracked quest ID
    pub fn current_quest_id(&self) -> Option<&QuestId> {
        self.current_tracker.as_ref().map(|t| &t.quest_id)
    }

    /// Update tracker state from server
    pub fn on_tracker_update(&mut self, payload: QuestTrackerPayload) {
        debug!(
            quest_id = %payload.quest_id,
            step = payload.current_step,
            "Quest tracker updated"
        );
        self.current_tracker = Some(payload);
        self.visible = true;
    }

    /// Clear tracker (when quest completed/abandoned or unpin)
    pub fn clear(&mut self) {
        debug!("Quest tracker cleared");
        self.current_tracker = None;
        self.visible = false;
    }

    /// Check if a quest update affects the tracked quest
    pub fn is_tracking(&self, quest_id: &QuestId) -> bool {
        self.current_tracker
            .as_ref()
            .map(|t| &t.quest_id == quest_id)
            .unwrap_or(false)
    }

    /// Create push event for tracker update
    pub fn push_tracker_update(tracker: &QuestTrackerPayload) -> BridgePush {
        BridgePush::new(
            PushType::QuestTrackerUpdate.as_str(),
            serde_json::to_value(TrackerUpdateJs::from(tracker)).unwrap_or_default(),
        )
    }

    /// Create push event for quest notification (toast)
    pub fn push_notification(notification: &QuestNotificationPayload) -> BridgePush {
        BridgePush::new(
            PushType::QuestNotification.as_str(),
            serde_json::to_value(NotificationJs::from(notification)).unwrap_or_default(),
        )
    }

    /// Get current tracker data
    pub fn current(&self) -> Option<&QuestTrackerPayload> {
        self.current_tracker.as_ref()
    }
}

/// Tracker update payload for JS
#[derive(Debug, Serialize)]
struct TrackerUpdateJs {
    quest_id: String,
    title: String,
    current_step: u32,
    step_description: String,
    progress: u32,
    target: u32,
}

impl From<&QuestTrackerPayload> for TrackerUpdateJs {
    fn from(payload: &QuestTrackerPayload) -> Self {
        Self {
            quest_id: payload.quest_id.to_string(),
            title: payload.title.clone(),
            current_step: payload.current_step,
            step_description: payload.step_description.clone(),
            progress: payload.progress,
            target: payload.target,
        }
    }
}

/// Notification payload for JS
#[derive(Debug, Serialize)]
struct NotificationJs {
    quest_id: String,
    notification_type: String,
    rewards: Option<RewardsJs>,
}

impl From<&QuestNotificationPayload> for NotificationJs {
    fn from(payload: &QuestNotificationPayload) -> Self {
        Self {
            quest_id: payload.quest_id.to_string(),
            notification_type: format!("{:?}", payload.notification_type),
            rewards: payload.rewards.as_ref().map(RewardsJs::from),
        }
    }
}

/// Rewards info for JS
#[derive(Debug, Serialize)]
struct RewardsJs {
    xp: u32,
    currency: u32,
    items: Vec<(String, u32)>,
}

impl From<&plix_common::protocol::QuestRewardsInfo> for RewardsJs {
    fn from(rewards: &plix_common::protocol::QuestRewardsInfo) -> Self {
        Self {
            xp: rewards.xp,
            currency: rewards.currency,
            items: rewards
                .items
                .iter()
                .map(|(id, qty)| (id.to_string(), *qty))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::protocol::QuestNotificationType;

    #[test]
    fn test_quest_tracker_bridge_new() {
        let bridge = QuestTrackerBridge::new();
        assert!(!bridge.is_visible());
        assert!(bridge.current_quest_id().is_none());
    }

    #[test]
    fn test_on_tracker_update() {
        let mut bridge = QuestTrackerBridge::new();

        let payload = QuestTrackerPayload {
            quest_id: QuestId::new("test_quest"),
            title: "Test Quest".to_string(),
            current_step: 0,
            step_description: "Kill 3 rats".to_string(),
            progress: 1,
            target: 3,
        };

        bridge.on_tracker_update(payload);

        assert!(bridge.is_visible());
        assert_eq!(
            bridge.current_quest_id().map(|q| q.to_string()),
            Some("test_quest".to_string())
        );
    }

    #[test]
    fn test_clear() {
        let mut bridge = QuestTrackerBridge::new();

        let payload = QuestTrackerPayload {
            quest_id: QuestId::new("test_quest"),
            title: "Test Quest".to_string(),
            current_step: 0,
            step_description: "Kill 3 rats".to_string(),
            progress: 1,
            target: 3,
        };

        bridge.on_tracker_update(payload);
        assert!(bridge.is_visible());

        bridge.clear();
        assert!(!bridge.is_visible());
        assert!(bridge.current_quest_id().is_none());
    }

    #[test]
    fn test_is_tracking() {
        let mut bridge = QuestTrackerBridge::new();

        let payload = QuestTrackerPayload {
            quest_id: QuestId::new("test_quest"),
            title: "Test Quest".to_string(),
            current_step: 0,
            step_description: "Kill 3 rats".to_string(),
            progress: 1,
            target: 3,
        };

        bridge.on_tracker_update(payload);

        assert!(bridge.is_tracking(&QuestId::new("test_quest")));
        assert!(!bridge.is_tracking(&QuestId::new("other_quest")));
    }

    #[test]
    fn test_push_tracker_update() {
        let payload = QuestTrackerPayload {
            quest_id: QuestId::new("test_quest"),
            title: "Test Quest".to_string(),
            current_step: 0,
            step_description: "Kill 3 rats".to_string(),
            progress: 1,
            target: 3,
        };

        let push = QuestTrackerBridge::push_tracker_update(&payload);
        assert_eq!(push.push_type, "QuestTrackerUpdate");
    }

    #[test]
    fn test_push_notification() {
        let notification = QuestNotificationPayload {
            quest_id: QuestId::new("test_quest"),
            notification_type: QuestNotificationType::Started,
            rewards: None,
        };

        let push = QuestTrackerBridge::push_notification(&notification);
        assert_eq!(push.push_type, "QuestNotification");
    }

    #[test]
    fn test_push_notification_with_rewards() {
        let notification = QuestNotificationPayload {
            quest_id: QuestId::new("test_quest"),
            notification_type: QuestNotificationType::Completed,
            rewards: Some(plix_common::protocol::QuestRewardsInfo {
                xp: 100,
                currency: 50,
                items: vec![],
            }),
        };

        let push = QuestTrackerBridge::push_notification(&notification);
        assert_eq!(push.push_type, "QuestNotification");

        let payload = push.payload;
        assert!(payload.get("rewards").is_some());
        assert_eq!(payload["rewards"]["xp"], 100);
        assert_eq!(payload["rewards"]["currency"], 50);
    }
}
