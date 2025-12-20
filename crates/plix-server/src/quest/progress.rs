//! Quest progress tracking per player
//!
//! Tracks active quests, step completion, and completed quest history.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use plix_common::content::ids::QuestId;
use plix_common::content::quest::{QuestDefinition, QuestStep};
use plix_common::types::PlayerId;

/// Progress for a single step within a quest.
#[derive(Debug, Clone)]
pub struct StepProgress {
    /// Step index within the quest
    pub step_index: usize,
    /// Current progress count (for countable steps like KillMob, CollectItem)
    pub current: u32,
    /// Target count to complete (1 for non-countable steps)
    pub target: u32,
    /// Whether this step is completed
    pub completed: bool,
    /// Time when the step was completed (if completed)
    pub completed_at: Option<Instant>,
}

impl StepProgress {
    /// Create a new step progress tracker.
    pub fn new(step_index: usize, step: &QuestStep) -> Self {
        let target = step.target_count().unwrap_or(1);
        Self {
            step_index,
            current: 0,
            target,
            completed: false,
            completed_at: None,
        }
    }

    /// Add progress to this step.
    /// Returns true if the step was just completed.
    pub fn add_progress(&mut self, amount: u32) -> bool {
        if self.completed {
            return false;
        }

        self.current = (self.current + amount).min(self.target);

        if self.current >= self.target && !self.completed {
            self.completed = true;
            self.completed_at = Some(Instant::now());
            return true;
        }

        false
    }

    /// Set step as completed regardless of current progress.
    pub fn mark_complete(&mut self) {
        if !self.completed {
            self.current = self.target;
            self.completed = true;
            self.completed_at = Some(Instant::now());
        }
    }

    /// Get completion percentage (0.0 to 1.0).
    pub fn progress_percent(&self) -> f32 {
        if self.target == 0 {
            return 1.0;
        }
        (self.current as f32 / self.target as f32).min(1.0)
    }
}

/// State of an active quest for a player.
#[derive(Debug, Clone)]
pub struct ActiveQuestState {
    /// Quest ID
    pub quest_id: QuestId,
    /// When the quest was started
    pub started_at: Instant,
    /// Progress for each step
    pub step_progress: Vec<StepProgress>,
    /// Current active step index (0-based)
    pub current_step: usize,
    /// Whether this quest is pinned to the HUD
    pub pinned: bool,
}

impl ActiveQuestState {
    /// Create a new active quest state from a definition.
    pub fn new(definition: &QuestDefinition) -> Self {
        let step_progress = definition
            .steps
            .iter()
            .enumerate()
            .map(|(i, step)| StepProgress::new(i, step))
            .collect();

        Self {
            quest_id: definition.quest_id.clone(),
            started_at: Instant::now(),
            step_progress,
            current_step: 0,
            pinned: false,
        }
    }

    /// Get the current step's progress.
    pub fn current_step_progress(&self) -> Option<&StepProgress> {
        self.step_progress.get(self.current_step)
    }

    /// Get mutable reference to current step's progress.
    pub fn current_step_progress_mut(&mut self) -> Option<&mut StepProgress> {
        self.step_progress.get_mut(self.current_step)
    }

    /// Check if all steps are completed.
    pub fn is_complete(&self) -> bool {
        self.step_progress.iter().all(|s| s.completed)
    }

    /// Advance to the next step if the current step is complete.
    /// Returns true if advanced.
    pub fn try_advance_step(&mut self) -> bool {
        if let Some(current) = self.step_progress.get(self.current_step) {
            if current.completed && self.current_step + 1 < self.step_progress.len() {
                self.current_step += 1;
                return true;
            }
        }
        false
    }

    /// Get overall completion percentage.
    pub fn overall_progress(&self) -> f32 {
        if self.step_progress.is_empty() {
            return 1.0;
        }

        let completed_steps = self.step_progress.iter().filter(|s| s.completed).count();
        let current_step_progress = self
            .current_step_progress()
            .map(|s| s.progress_percent())
            .unwrap_or(0.0);

        // Weight: completed steps + partial current step
        (completed_steps as f32 + current_step_progress * (1.0 - if self.current_step < self.step_progress.len() && self.step_progress[self.current_step].completed { 1.0 } else { 0.0 }))
            / self.step_progress.len() as f32
    }
}

/// All quest-related progress for a single player.
#[derive(Debug, Clone)]
pub struct PlayerQuestProgress {
    /// Player ID
    pub player_id: PlayerId,
    /// Currently active quests (max limit enforced by QuestSystem)
    pub active_quests: HashMap<QuestId, ActiveQuestState>,
    /// Completed quest IDs (for prerequisite checking)
    pub completed_quests: HashSet<QuestId>,
    /// Total XP earned from quests
    pub total_quest_xp: u32,
    /// Total currency earned from quests
    pub total_quest_currency: u32,
}

impl PlayerQuestProgress {
    /// Create a new empty progress tracker for a player.
    pub fn new(player_id: PlayerId) -> Self {
        Self {
            player_id,
            active_quests: HashMap::new(),
            completed_quests: HashSet::new(),
            total_quest_xp: 0,
            total_quest_currency: 0,
        }
    }

    /// Check if the player has completed a specific quest.
    pub fn has_completed(&self, quest_id: &QuestId) -> bool {
        self.completed_quests.contains(quest_id)
    }

    /// Check if the player has an active instance of a quest.
    pub fn has_active(&self, quest_id: &QuestId) -> bool {
        self.active_quests.contains_key(quest_id)
    }

    /// Get an active quest state.
    pub fn get_active(&self, quest_id: &QuestId) -> Option<&ActiveQuestState> {
        self.active_quests.get(quest_id)
    }

    /// Get mutable reference to an active quest state.
    pub fn get_active_mut(&mut self, quest_id: &QuestId) -> Option<&mut ActiveQuestState> {
        self.active_quests.get_mut(quest_id)
    }

    /// Start a new quest.
    /// Returns Err if already active or (completed and not repeatable).
    pub fn start_quest(&mut self, definition: &QuestDefinition) -> Result<(), &'static str> {
        if self.has_active(&definition.quest_id) {
            return Err("Quest already active");
        }

        if self.has_completed(&definition.quest_id) && !definition.repeatable {
            return Err("Quest already completed and not repeatable");
        }

        let state = ActiveQuestState::new(definition);
        self.active_quests.insert(definition.quest_id.clone(), state);
        Ok(())
    }

    /// Complete and remove an active quest.
    /// Returns the quest state if it was active.
    pub fn complete_quest(&mut self, quest_id: &QuestId) -> Option<ActiveQuestState> {
        let state = self.active_quests.remove(quest_id)?;
        self.completed_quests.insert(quest_id.clone());
        Some(state)
    }

    /// Abandon an active quest.
    /// Returns the quest state if it was active.
    pub fn abandon_quest(&mut self, quest_id: &QuestId) -> Option<ActiveQuestState> {
        self.active_quests.remove(quest_id)
    }

    /// Get the number of active quests.
    pub fn active_count(&self) -> usize {
        self.active_quests.len()
    }

    /// Get the number of completed quests.
    pub fn completed_count(&self) -> usize {
        self.completed_quests.len()
    }

    /// Add quest rewards to totals.
    pub fn add_rewards(&mut self, xp: u32, currency: u32) {
        self.total_quest_xp = self.total_quest_xp.saturating_add(xp);
        self.total_quest_currency = self.total_quest_currency.saturating_add(currency);
    }

    /// Get the currently pinned quest (if any).
    pub fn pinned_quest(&self) -> Option<&QuestId> {
        self.active_quests
            .iter()
            .find(|(_, state)| state.pinned)
            .map(|(id, _)| id)
    }

    /// Set which quest is pinned (unpins others).
    pub fn set_pinned(&mut self, quest_id: Option<&QuestId>) {
        for (id, state) in &mut self.active_quests {
            state.pinned = Some(id) == quest_id;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::content::ids::MobDefId;
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
                    count: 5,
                    description: "Kill 5 rats".to_string(),
                },
                QuestStep::TalkToNpc {
                    npc_id: plix_common::content::ids::NpcId::new("elder"),
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
    fn test_step_progress() {
        let step = QuestStep::KillMob {
            mob_id: MobDefId::new("cave_rat"),
            count: 5,
            description: "Kill 5 rats".to_string(),
        };

        let mut progress = StepProgress::new(0, &step);
        assert_eq!(progress.current, 0);
        assert_eq!(progress.target, 5);
        assert!(!progress.completed);

        // Add some progress
        let completed = progress.add_progress(2);
        assert!(!completed);
        assert_eq!(progress.current, 2);

        // Complete the step
        let completed = progress.add_progress(3);
        assert!(completed);
        assert!(progress.completed);
        assert_eq!(progress.current, 5);
    }

    #[test]
    fn test_active_quest_state() {
        let quest = test_quest();
        let mut state = ActiveQuestState::new(&quest);

        assert_eq!(state.current_step, 0);
        assert!(!state.is_complete());

        // Progress first step
        state.current_step_progress_mut().unwrap().add_progress(5);
        assert!(state.current_step_progress().unwrap().completed);

        // Advance to next step
        assert!(state.try_advance_step());
        assert_eq!(state.current_step, 1);

        // Complete second step
        state.current_step_progress_mut().unwrap().mark_complete();
        assert!(state.is_complete());
    }

    #[test]
    fn test_player_quest_progress() {
        let quest = test_quest();
        let mut progress = PlayerQuestProgress::new(PlayerId(1));

        // Start quest
        assert!(progress.start_quest(&quest).is_ok());
        assert!(progress.has_active(&quest.quest_id));

        // Can't start again
        assert!(progress.start_quest(&quest).is_err());

        // Complete quest
        let state = progress.complete_quest(&quest.quest_id);
        assert!(state.is_some());
        assert!(progress.has_completed(&quest.quest_id));
        assert!(!progress.has_active(&quest.quest_id));

        // Can't start non-repeatable quest again
        assert!(progress.start_quest(&quest).is_err());
    }

    #[test]
    fn test_pinned_quest() {
        let quest = test_quest();
        let mut progress = PlayerQuestProgress::new(PlayerId(1));
        progress.start_quest(&quest).unwrap();

        // Pin quest
        progress.set_pinned(Some(&quest.quest_id));
        assert_eq!(progress.pinned_quest(), Some(&quest.quest_id));

        // Unpin
        progress.set_pinned(None);
        assert_eq!(progress.pinned_quest(), None);
    }
}
