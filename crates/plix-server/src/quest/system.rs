//! Quest system managing player progress and quest events
//!
//! Handles:
//! - Quest starting with prerequisite validation
//! - Step progress updates from game events
//! - Quest completion with reward granting
//! - Protocol message handling

use std::collections::HashMap;
use std::time::Instant;

use tracing::{debug, info, warn};

use plix_common::content::ids::{DungeonId, MobDefId, NpcId, QuestId, RegionId};
use plix_common::content::quest::{QuestDefinition, QuestStep};
use plix_common::protocol::{
    ActiveQuestInfo, QuestNotificationPayload, QuestNotificationType, QuestRewardsInfo,
    QuestStepInfo, QuestSyncPayload, QuestTrackerPayload, QuestUpdatePayload, QuestUpdateType,
    ServerMessage, StepProgressInfo,
};
use plix_common::types::{ItemId, PlayerId};

use super::progress::{ActiveQuestState, PlayerQuestProgress, StepProgress};

/// Configuration for the quest system.
#[derive(Debug, Clone)]
pub struct QuestSystemConfig {
    /// Maximum number of active quests per player
    pub max_active_quests: usize,
}

impl Default for QuestSystemConfig {
    fn default() -> Self {
        Self {
            max_active_quests: 10,
        }
    }
}

/// Event emitted by the quest system.
#[derive(Debug, Clone)]
pub enum QuestEvent {
    /// Quest was started
    Started {
        player_id: PlayerId,
        quest_id: QuestId,
    },
    /// Quest step was completed
    StepCompleted {
        player_id: PlayerId,
        quest_id: QuestId,
        step_index: usize,
    },
    /// Quest was completed
    Completed {
        player_id: PlayerId,
        quest_id: QuestId,
        xp: u32,
        currency: u32,
    },
    /// Quest was abandoned
    Abandoned {
        player_id: PlayerId,
        quest_id: QuestId,
    },
    /// Quest became available (prerequisites met)
    Available {
        player_id: PlayerId,
        quest_id: QuestId,
    },
}

/// The main quest system.
pub struct QuestSystem {
    /// Configuration
    config: QuestSystemConfig,
    /// Quest definitions keyed by quest_id
    quest_defs: HashMap<QuestId, QuestDefinition>,
    /// Player quest progress
    player_progress: HashMap<PlayerId, PlayerQuestProgress>,
    /// Pending events (consumed by caller)
    pending_events: Vec<QuestEvent>,
    /// Pending messages to send
    pending_messages: Vec<(PlayerId, ServerMessage)>,
}

impl QuestSystem {
    /// Create a new quest system.
    pub fn new(config: QuestSystemConfig) -> Self {
        Self {
            config,
            quest_defs: HashMap::new(),
            player_progress: HashMap::new(),
            pending_events: Vec::new(),
            pending_messages: Vec::new(),
        }
    }

    /// Load quest definitions from content registry.
    pub fn load_content(&mut self, registry: &plix_common::content::ContentRegistry) {
        for (id, def) in &registry.quests {
            self.quest_defs.insert(id.clone(), def.clone());
        }
        info!(quests = self.quest_defs.len(), "Quest system loaded content");
    }

    /// Register a quest definition manually.
    pub fn register_quest(&mut self, definition: QuestDefinition) {
        self.quest_defs.insert(definition.quest_id.clone(), definition);
    }

    /// Get or create player progress.
    fn get_or_create_progress(&mut self, player_id: PlayerId) -> &mut PlayerQuestProgress {
        self.player_progress
            .entry(player_id)
            .or_insert_with(|| PlayerQuestProgress::new(player_id))
    }

    /// Get player progress (read-only).
    pub fn get_progress(&self, player_id: PlayerId) -> Option<&PlayerQuestProgress> {
        self.player_progress.get(&player_id)
    }

    /// Check if a player meets prerequisites for a quest.
    pub fn check_prerequisites(&self, player_id: PlayerId, quest_id: &QuestId) -> Result<(), String> {
        let definition = self
            .quest_defs
            .get(quest_id)
            .ok_or_else(|| format!("Unknown quest: {}", quest_id))?;

        let progress = self.player_progress.get(&player_id);

        // Check completed quest prerequisites
        for required_quest in &definition.prerequisites.completed_quests {
            let completed = progress
                .map(|p| p.has_completed(required_quest))
                .unwrap_or(false);
            if !completed {
                return Err(format!("Requires completion of: {}", required_quest));
            }
        }

        // Check min level (if implemented)
        // For now, skip level checks as leveling isn't implemented

        // Check required items (would need inventory access)
        // For now, skip item checks

        Ok(())
    }

    /// Start a quest for a player.
    pub fn start_quest(&mut self, player_id: PlayerId, quest_id: &QuestId) -> Result<(), String> {
        // Validate quest exists
        let definition = self
            .quest_defs
            .get(quest_id)
            .ok_or_else(|| format!("Unknown quest: {}", quest_id))?
            .clone();

        // Check prerequisites
        self.check_prerequisites(player_id, quest_id)?;

        // Get or create progress
        let progress = self.get_or_create_progress(player_id);

        // Check max active quests
        if progress.active_count() >= self.config.max_active_quests {
            return Err(format!(
                "Maximum active quests ({}) reached",
                self.config.max_active_quests
            ));
        }

        // Start the quest
        progress.start_quest(&definition).map_err(|e| e.to_string())?;

        debug!(
            player_id = ?player_id,
            quest_id = %quest_id,
            "Player started quest"
        );

        // Emit event
        self.pending_events.push(QuestEvent::Started {
            player_id,
            quest_id: quest_id.clone(),
        });

        // Send update to player
        self.send_quest_update(player_id, quest_id, QuestUpdateType::Started);
        self.send_notification(player_id, quest_id, QuestNotificationType::Started);

        Ok(())
    }

    /// Abandon a quest for a player.
    pub fn abandon_quest(&mut self, player_id: PlayerId, quest_id: &QuestId) -> Result<(), String> {
        let progress = self
            .player_progress
            .get_mut(&player_id)
            .ok_or("No progress for player")?;

        if !progress.has_active(quest_id) {
            return Err("Quest not active".to_string());
        }

        progress.abandon_quest(quest_id);

        debug!(
            player_id = ?player_id,
            quest_id = %quest_id,
            "Player abandoned quest"
        );

        // Emit event
        self.pending_events.push(QuestEvent::Abandoned {
            player_id,
            quest_id: quest_id.clone(),
        });

        // Send update to player
        self.send_quest_update(player_id, quest_id, QuestUpdateType::Abandoned);
        self.send_notification(player_id, quest_id, QuestNotificationType::Abandoned);

        Ok(())
    }

    /// Pin/unpin a quest for the HUD tracker.
    pub fn pin_quest(&mut self, player_id: PlayerId, quest_id: Option<&QuestId>) {
        if let Some(progress) = self.player_progress.get_mut(&player_id) {
            progress.set_pinned(quest_id);

            // Send tracker update
            if let Some(qid) = quest_id {
                self.send_tracker_update(player_id, qid);
            }
        }
    }

    /// Handle a mob kill event for KillMob step progression.
    /// Only the killer (last-hit) gets credit.
    pub fn on_mob_killed(&mut self, killer_id: PlayerId, mob_def_id: &MobDefId) {
        let progress = match self.player_progress.get_mut(&killer_id) {
            Some(p) => p,
            None => return,
        };

        // Find all active quests with KillMob steps matching this mob
        let matching_quests: Vec<QuestId> = progress
            .active_quests
            .keys()
            .cloned()
            .collect();

        for quest_id in matching_quests {
            let definition = match self.quest_defs.get(&quest_id) {
                Some(d) => d,
                None => continue,
            };

            let state = match progress.active_quests.get_mut(&quest_id) {
                Some(s) => s,
                None => continue,
            };

            let current_step = state.current_step;
            let step = match definition.steps.get(current_step) {
                Some(s) => s,
                None => continue,
            };

            // Check if current step is KillMob for this mob
            if let QuestStep::KillMob { mob_id, count, .. } = step {
                if mob_id == mob_def_id {
                    if let Some(step_progress) = state.step_progress.get_mut(current_step) {
                        let just_completed = step_progress.add_progress(1);

                        debug!(
                            player_id = ?killer_id,
                            quest_id = %quest_id,
                            mob_id = %mob_def_id,
                            progress = step_progress.current,
                            target = step_progress.target,
                            "Kill counted for quest"
                        );

                        // Send progress update
                        self.pending_messages.push((
                            killer_id,
                            ServerMessage::QuestUpdate(QuestUpdatePayload {
                                quest_id: quest_id.clone(),
                                update_type: QuestUpdateType::StepProgress,
                                step_index: Some(current_step as u32),
                                progress: Some(step_progress.current),
                                target: Some(step_progress.target),
                            }),
                        ));

                        if just_completed {
                            self.handle_step_completed(killer_id, &quest_id, current_step);
                        }
                    }
                }
            }
        }
    }

    /// Handle a region visit event for VisitLocation step progression.
    pub fn on_region_visited(&mut self, player_id: PlayerId, region_id: &RegionId) {
        self.check_step_progress(player_id, |step| {
            matches!(step, QuestStep::VisitLocation { region_id: rid, .. } if rid == region_id)
        });
    }

    /// Handle an NPC interaction for TalkToNpc step progression.
    pub fn on_npc_talked(&mut self, player_id: PlayerId, npc_id: &NpcId) {
        self.check_step_progress(player_id, |step| {
            matches!(step, QuestStep::TalkToNpc { npc_id: nid, .. } if nid == npc_id)
        });
    }

    /// Handle a dungeon clear event for DungeonClear step progression.
    pub fn on_dungeon_cleared(&mut self, player_id: PlayerId, dungeon_id: &DungeonId) {
        self.check_step_progress(player_id, |step| {
            matches!(step, QuestStep::DungeonClear { dungeon_id: did, .. } if did == dungeon_id)
        });
    }

    /// Handle an item collection event for CollectItem step progression.
    pub fn on_item_collected(&mut self, player_id: PlayerId, item_id: &ItemId, count: u32) {
        let progress = match self.player_progress.get_mut(&player_id) {
            Some(p) => p,
            None => return,
        };

        let matching_quests: Vec<QuestId> = progress
            .active_quests
            .keys()
            .cloned()
            .collect();

        for quest_id in matching_quests {
            let definition = match self.quest_defs.get(&quest_id) {
                Some(d) => d,
                None => continue,
            };

            let state = match progress.active_quests.get_mut(&quest_id) {
                Some(s) => s,
                None => continue,
            };

            let current_step = state.current_step;
            let step = match definition.steps.get(current_step) {
                Some(s) => s,
                None => continue,
            };

            if let QuestStep::CollectItem { item_id: iid, .. } = step {
                if iid == item_id {
                    if let Some(step_progress) = state.step_progress.get_mut(current_step) {
                        let just_completed = step_progress.add_progress(count);

                        self.pending_messages.push((
                            player_id,
                            ServerMessage::QuestUpdate(QuestUpdatePayload {
                                quest_id: quest_id.clone(),
                                update_type: QuestUpdateType::StepProgress,
                                step_index: Some(current_step as u32),
                                progress: Some(step_progress.current),
                                target: Some(step_progress.target),
                            }),
                        ));

                        if just_completed {
                            self.handle_step_completed(player_id, &quest_id, current_step);
                        }
                    }
                }
            }
        }
    }

    /// Generic step progress checker for single-completion steps.
    fn check_step_progress<F>(&mut self, player_id: PlayerId, matches: F)
    where
        F: Fn(&QuestStep) -> bool,
    {
        let progress = match self.player_progress.get_mut(&player_id) {
            Some(p) => p,
            None => return,
        };

        let matching_quests: Vec<QuestId> = progress
            .active_quests
            .keys()
            .cloned()
            .collect();

        for quest_id in matching_quests {
            let definition = match self.quest_defs.get(&quest_id) {
                Some(d) => d,
                None => continue,
            };

            let state = match progress.active_quests.get_mut(&quest_id) {
                Some(s) => s,
                None => continue,
            };

            let current_step = state.current_step;
            let step = match definition.steps.get(current_step) {
                Some(s) => s,
                None => continue,
            };

            if matches(step) {
                if let Some(step_progress) = state.step_progress.get_mut(current_step) {
                    if !step_progress.completed {
                        step_progress.mark_complete();
                        self.handle_step_completed(player_id, &quest_id, current_step);
                    }
                }
            }
        }
    }

    /// Handle a step being completed.
    fn handle_step_completed(&mut self, player_id: PlayerId, quest_id: &QuestId, step_index: usize) {
        debug!(
            player_id = ?player_id,
            quest_id = %quest_id,
            step_index = step_index,
            "Quest step completed"
        );

        // Emit step completed event
        self.pending_events.push(QuestEvent::StepCompleted {
            player_id,
            quest_id: quest_id.clone(),
            step_index,
        });

        // Send step completed notification
        self.send_notification(player_id, quest_id, QuestNotificationType::StepCompleted);

        // Check if quest is complete
        let progress = match self.player_progress.get_mut(&player_id) {
            Some(p) => p,
            None => return,
        };

        let state = match progress.active_quests.get_mut(quest_id) {
            Some(s) => s,
            None => return,
        };

        // Try to advance to next step
        if state.try_advance_step() {
            // Send update for new current step
            self.pending_messages.push((
                player_id,
                ServerMessage::QuestUpdate(QuestUpdatePayload {
                    quest_id: quest_id.clone(),
                    update_type: QuestUpdateType::StepAdvanced,
                    step_index: Some(state.current_step as u32),
                    progress: None,
                    target: None,
                }),
            ));
        }

        // Check if all steps are complete
        if state.is_complete() {
            // Complete the quest (need to borrow quest_id clone before borrowing progress again)
            let quest_id_clone = quest_id.clone();
            self.complete_quest(player_id, &quest_id_clone);
        }
    }

    /// Complete a quest and grant rewards.
    fn complete_quest(&mut self, player_id: PlayerId, quest_id: &QuestId) {
        let definition = match self.quest_defs.get(quest_id) {
            Some(d) => d.clone(),
            None => return,
        };

        let progress = match self.player_progress.get_mut(&player_id) {
            Some(p) => p,
            None => return,
        };

        // Remove from active and add to completed
        progress.complete_quest(quest_id);

        // Add rewards to totals
        progress.add_rewards(definition.rewards.xp, definition.rewards.currency);

        info!(
            player_id = ?player_id,
            quest_id = %quest_id,
            xp = definition.rewards.xp,
            currency = definition.rewards.currency,
            "Quest completed"
        );

        // Emit completed event
        self.pending_events.push(QuestEvent::Completed {
            player_id,
            quest_id: quest_id.clone(),
            xp: definition.rewards.xp,
            currency: definition.rewards.currency,
        });

        // Send completion message with rewards
        self.pending_messages.push((
            player_id,
            ServerMessage::QuestUpdate(QuestUpdatePayload {
                quest_id: quest_id.clone(),
                update_type: QuestUpdateType::Completed,
                step_index: None,
                progress: None,
                target: None,
            }),
        ));

        self.pending_messages.push((
            player_id,
            ServerMessage::QuestNotification(QuestNotificationPayload {
                quest_id: quest_id.clone(),
                notification_type: QuestNotificationType::Completed,
                rewards: Some(QuestRewardsInfo {
                    xp: definition.rewards.xp,
                    currency: definition.rewards.currency,
                    items: definition.rewards.items.iter()
                        .map(|(id, qty)| (id.clone(), *qty))
                        .collect(),
                }),
            }),
        ));
    }

    /// Send quest sync to a player (on connect).
    pub fn send_sync(&mut self, player_id: PlayerId) {
        let progress = match self.player_progress.get(&player_id) {
            Some(p) => p,
            None => {
                // No progress yet, send empty sync
                self.pending_messages.push((
                    player_id,
                    ServerMessage::QuestSync(QuestSyncPayload {
                        active_quests: vec![],
                        completed_quest_ids: vec![],
                    }),
                ));
                return;
            }
        };

        let active_quests: Vec<ActiveQuestInfo> = progress
            .active_quests
            .iter()
            .filter_map(|(quest_id, state)| {
                let definition = self.quest_defs.get(quest_id)?;
                Some(self.build_active_quest_info(definition, state))
            })
            .collect();

        let completed_quest_ids: Vec<QuestId> = progress.completed_quests.iter().cloned().collect();

        self.pending_messages.push((
            player_id,
            ServerMessage::QuestSync(QuestSyncPayload {
                active_quests,
                completed_quest_ids,
            }),
        ));
    }

    /// Build ActiveQuestInfo for protocol messages.
    fn build_active_quest_info(
        &self,
        definition: &QuestDefinition,
        state: &ActiveQuestState,
    ) -> ActiveQuestInfo {
        let steps: Vec<QuestStepInfo> = definition
            .steps
            .iter()
            .enumerate()
            .map(|(i, step)| {
                let progress = state.step_progress.get(i);
                QuestStepInfo {
                    description: step.description().to_string(),
                    step_type: step.step_type().to_string(),
                    progress: progress.map(|p| StepProgressInfo {
                        current: p.current,
                        target: p.target,
                        completed: p.completed,
                    }),
                }
            })
            .collect();

        ActiveQuestInfo {
            quest_id: definition.quest_id.clone(),
            title: definition.title.clone(),
            description: definition.description_short.clone(),
            current_step: state.current_step as u32,
            steps,
            pinned: state.pinned,
        }
    }

    /// Send a quest update message.
    fn send_quest_update(&mut self, player_id: PlayerId, quest_id: &QuestId, update_type: QuestUpdateType) {
        self.pending_messages.push((
            player_id,
            ServerMessage::QuestUpdate(QuestUpdatePayload {
                quest_id: quest_id.clone(),
                update_type,
                step_index: None,
                progress: None,
                target: None,
            }),
        ));
    }

    /// Send a quest notification message.
    fn send_notification(&mut self, player_id: PlayerId, quest_id: &QuestId, notification_type: QuestNotificationType) {
        self.pending_messages.push((
            player_id,
            ServerMessage::QuestNotification(QuestNotificationPayload {
                quest_id: quest_id.clone(),
                notification_type,
                rewards: None,
            }),
        ));
    }

    /// Send quest tracker update for pinned quest.
    fn send_tracker_update(&mut self, player_id: PlayerId, quest_id: &QuestId) {
        let progress = match self.player_progress.get(&player_id) {
            Some(p) => p,
            None => return,
        };

        let state = match progress.get_active(quest_id) {
            Some(s) => s,
            None => return,
        };

        let definition = match self.quest_defs.get(quest_id) {
            Some(d) => d,
            None => return,
        };

        let current_step_progress = state.current_step_progress();

        self.pending_messages.push((
            player_id,
            ServerMessage::QuestTrackerUpdate(QuestTrackerPayload {
                quest_id: quest_id.clone(),
                title: definition.title.clone(),
                current_step: state.current_step as u32,
                step_description: definition
                    .steps
                    .get(state.current_step)
                    .map(|s| s.description().to_string())
                    .unwrap_or_default(),
                progress: current_step_progress.map(|p| p.current).unwrap_or(0),
                target: current_step_progress.map(|p| p.target).unwrap_or(1),
            }),
        ));
    }

    /// Take pending events (clears the internal buffer).
    pub fn take_events(&mut self) -> Vec<QuestEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Take pending messages (clears the internal buffer).
    pub fn take_messages(&mut self) -> Vec<(PlayerId, ServerMessage)> {
        std::mem::take(&mut self.pending_messages)
    }

    /// Handle a player disconnect (cleanup if needed).
    pub fn on_player_disconnect(&mut self, player_id: PlayerId) {
        // Quest progress is preserved across sessions in a real implementation
        // For now, we just keep it in memory
        debug!(player_id = ?player_id, "Player disconnected, quest progress preserved");
    }

    /// Get the number of registered quests.
    pub fn quest_count(&self) -> usize {
        self.quest_defs.len()
    }

    /// Get a quest definition by ID.
    pub fn get_quest(&self, quest_id: &QuestId) -> Option<&QuestDefinition> {
        self.quest_defs.get(quest_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::content::quest::{QuestPrerequisites, QuestRewards};

    fn test_quest() -> QuestDefinition {
        QuestDefinition {
            quest_id: QuestId::new("test_quest"),
            title: "Test Quest".to_string(),
            description_short: "A test".to_string(),
            description_long: "A longer test".to_string(),
            chapter_id: None,
            prerequisites: QuestPrerequisites::default(),
            steps: vec![
                QuestStep::KillMob {
                    mob_id: MobDefId::new("cave_rat"),
                    count: 3,
                    description: "Kill 3 rats".to_string(),
                },
                QuestStep::TalkToNpc {
                    npc_id: NpcId::new("elder"),
                    description: "Return to elder".to_string(),
                },
            ],
            rewards: QuestRewards {
                xp: 100,
                currency: 50,
                items: vec![],
                unlocks: vec![],
            },
            repeatable: false,
        }
    }

    #[test]
    fn test_start_quest() {
        let mut system = QuestSystem::new(QuestSystemConfig::default());
        system.register_quest(test_quest());

        let player_id = PlayerId(1);
        let quest_id = QuestId::new("test_quest");

        assert!(system.start_quest(player_id, &quest_id).is_ok());
        assert!(system.get_progress(player_id).unwrap().has_active(&quest_id));

        // Check events
        let events = system.take_events();
        assert!(events.iter().any(|e| matches!(e, QuestEvent::Started { .. })));
    }

    #[test]
    fn test_mob_kill_progress() {
        let mut system = QuestSystem::new(QuestSystemConfig::default());
        system.register_quest(test_quest());

        let player_id = PlayerId(1);
        let quest_id = QuestId::new("test_quest");

        system.start_quest(player_id, &quest_id).unwrap();
        system.take_events(); // Clear start events

        // Kill 3 rats
        let mob_id = MobDefId::new("cave_rat");
        system.on_mob_killed(player_id, &mob_id);
        system.on_mob_killed(player_id, &mob_id);
        system.on_mob_killed(player_id, &mob_id);

        // Check step completed event
        let events = system.take_events();
        assert!(events.iter().any(|e| matches!(e, QuestEvent::StepCompleted { step_index: 0, .. })));

        // Check current step advanced
        let progress = system.get_progress(player_id).unwrap();
        let state = progress.get_active(&quest_id).unwrap();
        assert_eq!(state.current_step, 1);
    }

    #[test]
    fn test_quest_completion() {
        let mut system = QuestSystem::new(QuestSystemConfig::default());
        system.register_quest(test_quest());

        let player_id = PlayerId(1);
        let quest_id = QuestId::new("test_quest");

        system.start_quest(player_id, &quest_id).unwrap();

        // Complete step 1: Kill 3 rats
        let mob_id = MobDefId::new("cave_rat");
        for _ in 0..3 {
            system.on_mob_killed(player_id, &mob_id);
        }

        // Complete step 2: Talk to elder
        let npc_id = NpcId::new("elder");
        system.on_npc_talked(player_id, &npc_id);

        // Check quest completed
        let events = system.take_events();
        assert!(events.iter().any(|e| matches!(e, QuestEvent::Completed { xp: 100, currency: 50, .. })));

        // Quest should be in completed set
        let progress = system.get_progress(player_id).unwrap();
        assert!(progress.has_completed(&quest_id));
        assert!(!progress.has_active(&quest_id));
    }

    #[test]
    fn test_prerequisites() {
        let prereq_quest = QuestDefinition {
            quest_id: QuestId::new("prereq_quest"),
            title: "Prereq".to_string(),
            description_short: "Prereq".to_string(),
            description_long: "Prereq".to_string(),
            chapter_id: None,
            prerequisites: QuestPrerequisites::default(),
            steps: vec![QuestStep::TalkToNpc {
                npc_id: NpcId::new("npc"),
                description: "Talk".to_string(),
            }],
            rewards: QuestRewards::default(),
            repeatable: false,
        };

        let main_quest = QuestDefinition {
            quest_id: QuestId::new("main_quest"),
            title: "Main".to_string(),
            description_short: "Main".to_string(),
            description_long: "Main".to_string(),
            chapter_id: None,
            prerequisites: QuestPrerequisites {
                completed_quests: vec![QuestId::new("prereq_quest")],
                ..Default::default()
            },
            steps: vec![QuestStep::TalkToNpc {
                npc_id: NpcId::new("npc"),
                description: "Talk".to_string(),
            }],
            rewards: QuestRewards::default(),
            repeatable: false,
        };

        let mut system = QuestSystem::new(QuestSystemConfig::default());
        system.register_quest(prereq_quest);
        system.register_quest(main_quest);

        let player_id = PlayerId(1);

        // Should fail - prereq not complete
        assert!(system.start_quest(player_id, &QuestId::new("main_quest")).is_err());

        // Complete prereq
        system.start_quest(player_id, &QuestId::new("prereq_quest")).unwrap();
        system.on_npc_talked(player_id, &NpcId::new("npc"));

        // Now should succeed
        assert!(system.start_quest(player_id, &QuestId::new("main_quest")).is_ok());
    }
}
