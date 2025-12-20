# Quest Events Contract

**Feature**: 043-content-lore-campaign
**Date**: 2025-12-20
**Module**: `plix-server::quest::events`, `plix-common::protocol::messages`

---

## Overview

Quest events flow from server to client for UI updates and are exposed to the mod API for extensibility.

```
Server (QuestSystem)
    │
    ├──> Client Protocol (UI updates)
    │       └── ServerMessage::Quest*
    │
    └──> Mod API (hooks)
            └── ModEvent::Quest*
```

---

## Protocol Messages (Server → Client)

### QuestUpdate

Sent when any quest state changes for a player.

```rust
// crates/plix-common/src/protocol/messages.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    // ... existing variants ...

    /// Full quest state sync (on connect or major changes)
    QuestSync(QuestSyncPayload),

    /// Incremental quest update
    QuestUpdate(QuestUpdatePayload),

    /// Quest notification (toast/popup)
    QuestNotification(QuestNotificationPayload),

    /// Quest tracker HUD update
    QuestTrackerUpdate(QuestTrackerPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestSyncPayload {
    pub active_quests: Vec<ActiveQuestInfo>,
    pub completed_quest_ids: Vec<String>,
    pub pinned_quest_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveQuestInfo {
    pub quest_id: String,
    pub title: String,
    pub description: String,
    pub current_step_index: usize,
    pub steps: Vec<QuestStepInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestStepInfo {
    pub description: String,
    pub progress: StepProgressInfo,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepProgressInfo {
    Counter { current: u32, required: u32 },
    Boolean { done: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestUpdatePayload {
    pub quest_id: String,
    pub update_type: QuestUpdateType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestUpdateType {
    Started { title: String, steps: Vec<QuestStepInfo> },
    StepProgress { step_index: usize, progress: StepProgressInfo },
    StepCompleted { step_index: usize },
    Completed { rewards: QuestRewardsInfo },
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestRewardsInfo {
    pub xp: u32,
    pub currency: u32,
    pub items: Vec<(String, u32)>, // (item_name, count)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestNotificationPayload {
    pub notification_type: QuestNotificationType,
    pub quest_title: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestNotificationType {
    Started,
    StepCompleted,
    QuestCompleted,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestTrackerPayload {
    pub quest_id: Option<String>,
    pub quest_title: Option<String>,
    pub current_step_description: Option<String>,
    pub current_step_progress: Option<StepProgressInfo>,
}
```

---

## Protocol Messages (Client → Server)

### Quest Commands

```rust
// crates/plix-common/src/protocol/messages.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    // ... existing variants ...

    /// Request to accept a quest from NPC
    QuestAccept { quest_id: String },

    /// Request to abandon an active quest
    QuestAbandon { quest_id: String },

    /// Set pinned quest for tracker
    QuestPin { quest_id: Option<String> },

    /// Request full quest state sync
    QuestSyncRequest,
}
```

---

## Mod API Events

### QuestEvent

Emitted by QuestSystem, consumable by mods.

```rust
// crates/plix-mod-core/src/events.rs

#[derive(Debug, Clone)]
pub enum ModEvent {
    // ... existing variants ...

    Quest(QuestModEvent),
}

#[derive(Debug, Clone)]
pub enum QuestModEvent {
    /// Player started a quest
    Started {
        player_id: PlayerId,
        quest_id: String,
    },

    /// Player completed a quest step
    StepCompleted {
        player_id: PlayerId,
        quest_id: String,
        step_index: usize,
        step_type: String, // "KillMob", "CollectItem", etc.
    },

    /// Player completed entire quest
    Completed {
        player_id: PlayerId,
        quest_id: String,
        rewards_granted: Vec<(String, u32)>, // (item_id, count)
        xp_granted: u32,
        currency_granted: u32,
    },

    /// Player abandoned a quest
    Abandoned {
        player_id: PlayerId,
        quest_id: String,
    },

    /// Quest became available to player (prerequisites met)
    Available {
        player_id: PlayerId,
        quest_id: String,
    },
}
```

---

## Event Flow Examples

### Starting a Quest

```
1. Player interacts with NPC → ClientMessage::NpcInteract { npc_id }
2. Server sends dialogue → ServerMessage::DialogueShow { ... }
3. Player accepts quest → ClientMessage::QuestAccept { quest_id }
4. Server validates prerequisites
5. Server adds quest to player progress
6. Server emits ModEvent::Quest(Started { ... })
7. Server sends → ServerMessage::QuestUpdate(Started { ... })
8. Server sends → ServerMessage::QuestNotification(Started, ...)
9. Server sends → ServerMessage::QuestTrackerUpdate { ... }
```

### Killing a Mob (Quest Progress)

```
1. Player kills mob (last hit)
2. MobSystem emits ModEvent::Mob(Killed { ... })
3. QuestSystem receives event
4. QuestSystem checks active quests for KillMob matching mob_id
5. QuestSystem increments progress
6. If step complete:
   a. Server emits ModEvent::Quest(StepCompleted { ... })
   b. Server sends → ServerMessage::QuestUpdate(StepCompleted { ... })
   c. Server sends → ServerMessage::QuestNotification(StepCompleted, ...)
7. If quest complete (all steps done):
   a. Server grants rewards
   b. Server emits ModEvent::Quest(Completed { ... })
   c. Server sends → ServerMessage::QuestUpdate(Completed { ... })
   d. Server sends → ServerMessage::QuestNotification(QuestCompleted, ...)
```

---

## CEF Bridge Messages

### Rust → JavaScript

```javascript
// assets/ui/pages/quest_log.js

window.plix.on('quest_sync', (payload) => {
    // payload: { active_quests: [...], completed_quest_ids: [...], pinned_quest_id: ... }
    renderQuestList(payload);
});

window.plix.on('quest_update', (payload) => {
    // payload: { quest_id: "...", update_type: {...} }
    updateQuestEntry(payload);
});

window.plix.on('quest_notification', (payload) => {
    // payload: { notification_type: "Started"|"StepCompleted"|"QuestCompleted", quest_title: "...", details: "..." }
    showToast(payload);
});

// Quest tracker (separate overlay)
window.plix.on('quest_tracker_update', (payload) => {
    // payload: { quest_id, quest_title, current_step_description, current_step_progress }
    updateTrackerHUD(payload);
});
```

### JavaScript → Rust

```javascript
// Request actions
window.plix.send('quest_abandon', { quest_id: 'clear_rats' });
window.plix.send('quest_pin', { quest_id: 'investigate_cultists' });
window.plix.send('quest_sync_request', {});
```

---

## Debug Commands

Console commands for development/testing:

| Command | Description | Server Response |
|---------|-------------|-----------------|
| `/quest list` | List all active and completed quests | Text output |
| `/quest start <id>` | Force-start a quest (skip prerequisites) | QuestUpdate(Started) |
| `/quest complete <id>` | Force-complete a quest | QuestUpdate(Completed) |
| `/quest reset` | Reset all quest progress | QuestSync (empty) |
| `/quest step <id>` | Complete current step of quest | QuestUpdate(StepCompleted) |
