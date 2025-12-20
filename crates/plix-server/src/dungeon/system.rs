//! Dungeon system coordinating boss, chest, and player tracking
//!
//! Manages dungeon instances with:
//! - Boss state tracking
//! - Respawn timer management
//! - Reward chest activation
//! - Player entry/exit tracking

use std::collections::HashMap;
use std::time::{Duration, Instant};

use plix_common::content::dungeon::DungeonDefinition;
use plix_common::content::ids::DungeonId;
use plix_common::content::loot::LootTable;
use plix_common::math::Vec3;
use plix_common::protocol::{
    ChestAvailablePayload, DungeonCompletedPayload, DungeonEnteredPayload, DungeonStatePayload,
    ServerMessage,
};
use plix_common::types::{ItemId, PlayerId};
use tracing::{debug, info, warn};

use super::chest::{ChestEntity, ChestInteractResult};
use super::state::{DungeonPhase, DungeonState};

/// Configuration for the dungeon system.
#[derive(Debug, Clone)]
pub struct DungeonSystemConfig {
    /// Chest interaction range in blocks
    pub chest_interact_range: f32,
}

impl Default for DungeonSystemConfig {
    fn default() -> Self {
        Self {
            chest_interact_range: 3.0,
        }
    }
}

/// Events emitted by the dungeon system.
#[derive(Debug, Clone)]
pub enum DungeonEvent {
    /// Boss was killed
    BossKilled {
        dungeon_id: DungeonId,
        killer_id: Option<PlayerId>,
    },
    /// Boss has respawned
    BossRespawned {
        dungeon_id: DungeonId,
    },
    /// Chest was activated
    ChestActivated {
        dungeon_id: DungeonId,
    },
    /// Chest was deactivated
    ChestDeactivated {
        dungeon_id: DungeonId,
    },
    /// Player looted chest
    ChestLooted {
        dungeon_id: DungeonId,
        player_id: PlayerId,
        loot: Vec<(ItemId, u32)>,
    },
    /// Player entered dungeon
    PlayerEntered {
        dungeon_id: DungeonId,
        player_id: PlayerId,
    },
    /// Player left dungeon
    PlayerLeft {
        dungeon_id: DungeonId,
        player_id: PlayerId,
    },
}

/// The dungeon system managing all dungeon instances.
pub struct DungeonSystem {
    /// Configuration
    config: DungeonSystemConfig,
    /// Dungeon definitions indexed by ID
    definitions: HashMap<DungeonId, DungeonDefinition>,
    /// Dungeon runtime states indexed by ID
    states: HashMap<DungeonId, DungeonState>,
    /// Reward chests indexed by dungeon ID
    chests: HashMap<DungeonId, ChestEntity>,
    /// Loot tables indexed by ID
    loot_tables: HashMap<String, LootTable>,
    /// Pending events from this tick
    pending_events: Vec<DungeonEvent>,
    /// Pending protocol messages to send
    pending_messages: Vec<(PlayerId, ServerMessage)>,
    /// Counter for chest entity IDs
    next_chest_id: u64,
}

impl DungeonSystem {
    /// Create a new dungeon system.
    pub fn new(config: DungeonSystemConfig) -> Self {
        Self {
            config,
            definitions: HashMap::new(),
            states: HashMap::new(),
            chests: HashMap::new(),
            loot_tables: HashMap::new(),
            pending_events: Vec::new(),
            pending_messages: Vec::new(),
            next_chest_id: 1,
        }
    }

    /// Register a dungeon definition.
    pub fn register_dungeon(&mut self, definition: DungeonDefinition) {
        let dungeon_id = definition.dungeon_id.clone();

        // Create initial state
        let state = DungeonState::new(dungeon_id.clone(), definition.boss_respawn_secs);

        // Create chest entity (positioned at boss spawn + offset)
        let chest_pos = Vec3::new(
            definition.boss_spawn_position.x + 2.0,
            definition.boss_spawn_position.y,
            definition.boss_spawn_position.z,
        );
        let chest = ChestEntity::new(
            dungeon_id.clone(),
            chest_pos,
            definition.reward_loot_table.clone(),
        );

        info!(
            dungeon_id = %dungeon_id,
            "Registered dungeon: {}",
            definition.display_name
        );

        self.definitions.insert(dungeon_id.clone(), definition);
        self.states.insert(dungeon_id.clone(), state);
        self.chests.insert(dungeon_id, chest);
    }

    /// Register a loot table.
    pub fn register_loot_table(&mut self, loot_table: LootTable) {
        let id = loot_table.loot_table_id.as_str().to_string();
        self.loot_tables.insert(id, loot_table);
    }

    /// Get a dungeon definition by ID.
    pub fn get_definition(&self, dungeon_id: &DungeonId) -> Option<&DungeonDefinition> {
        self.definitions.get(dungeon_id)
    }

    /// Get a dungeon state by ID.
    pub fn get_state(&self, dungeon_id: &DungeonId) -> Option<&DungeonState> {
        self.states.get(dungeon_id)
    }

    /// Get a mutable dungeon state by ID.
    pub fn get_state_mut(&mut self, dungeon_id: &DungeonId) -> Option<&mut DungeonState> {
        self.states.get_mut(dungeon_id)
    }

    /// Get the current phase of a dungeon.
    pub fn get_phase(&self, dungeon_id: &DungeonId) -> Option<DungeonPhase> {
        self.states.get(dungeon_id).map(|s| s.phase())
    }

    /// Handle boss being killed in a dungeon.
    pub fn on_boss_killed(&mut self, dungeon_id: &DungeonId, killer_id: Option<PlayerId>) {
        let now = Instant::now();

        if let Some(state) = self.states.get_mut(dungeon_id) {
            if !state.boss_alive {
                warn!(dungeon_id = %dungeon_id, "Boss killed but was already dead");
                return;
            }

            state.on_boss_killed(now);

            info!(
                dungeon_id = %dungeon_id,
                killer = ?killer_id,
                respawn_secs = state.respawn_duration.as_secs(),
                "Boss killed, chest activated"
            );

            // Activate chest
            if let Some(chest) = self.chests.get_mut(dungeon_id) {
                chest.activate();
            }

            self.pending_events.push(DungeonEvent::BossKilled {
                dungeon_id: dungeon_id.clone(),
                killer_id,
            });
            self.pending_events.push(DungeonEvent::ChestActivated {
                dungeon_id: dungeon_id.clone(),
            });
        }
    }

    /// Update dungeon timers (call each tick).
    pub fn update(&mut self, now: Instant) {
        let mut respawning = Vec::new();

        // Check for boss respawns
        for (dungeon_id, state) in &self.states {
            if state.should_respawn(now) {
                respawning.push(dungeon_id.clone());
            }
        }

        // Process respawns
        for dungeon_id in respawning {
            if let Some(state) = self.states.get_mut(&dungeon_id) {
                state.on_boss_respawn();

                info!(dungeon_id = %dungeon_id, "Boss respawned");

                // Deactivate chest
                if let Some(chest) = self.chests.get_mut(&dungeon_id) {
                    chest.deactivate();
                }

                self.pending_events.push(DungeonEvent::BossRespawned {
                    dungeon_id: dungeon_id.clone(),
                });
                self.pending_events.push(DungeonEvent::ChestDeactivated {
                    dungeon_id,
                });
            }
        }
    }

    /// Handle player attempting to loot a chest.
    pub fn try_loot_chest(
        &mut self,
        dungeon_id: &DungeonId,
        player_id: PlayerId,
        player_pos: &Vec3,
    ) -> ChestInteractResult {
        // Check if chest exists
        let chest = match self.chests.get_mut(dungeon_id) {
            Some(c) => c,
            None => return ChestInteractResult::NotActive,
        };

        // Check if chest is active
        if !chest.active {
            return ChestInteractResult::NotActive;
        }

        // Check range
        if !chest.is_in_range(player_pos, self.config.chest_interact_range) {
            return ChestInteractResult::OutOfRange;
        }

        // Check if already looted
        if chest.has_looted(player_id) {
            return ChestInteractResult::AlreadyLooted;
        }

        // Get loot table
        let loot_table_id = chest.loot_table_id.as_str();
        let loot_table = match self.loot_tables.get(loot_table_id) {
            Some(lt) => lt.clone(),
            None => return ChestInteractResult::LootTableNotFound,
        };

        // Roll loot
        match chest.try_loot(player_id, &loot_table) {
            Some(loot) => {
                debug!(
                    dungeon_id = %dungeon_id,
                    player_id = ?player_id,
                    loot_count = loot.len(),
                    "Player looted chest"
                );

                // Record loot in state for quest tracking
                if let Some(state) = self.states.get_mut(dungeon_id) {
                    state.record_chest_loot(player_id);
                }

                self.pending_events.push(DungeonEvent::ChestLooted {
                    dungeon_id: dungeon_id.clone(),
                    player_id,
                    loot: loot.clone(),
                });

                ChestInteractResult::Success(loot)
            }
            None => ChestInteractResult::AlreadyLooted,
        }
    }

    /// Handle player entering a dungeon.
    pub fn on_player_entered(&mut self, dungeon_id: &DungeonId, player_id: PlayerId) {
        if let Some(state) = self.states.get_mut(dungeon_id) {
            if !state.is_player_inside(player_id) {
                state.player_entered(player_id);

                debug!(
                    dungeon_id = %dungeon_id,
                    player_id = ?player_id,
                    player_count = state.player_count(),
                    "Player entered dungeon"
                );

                self.pending_events.push(DungeonEvent::PlayerEntered {
                    dungeon_id: dungeon_id.clone(),
                    player_id,
                });
            }
        }
    }

    /// Handle player leaving a dungeon.
    pub fn on_player_left(&mut self, dungeon_id: &DungeonId, player_id: PlayerId) {
        if let Some(state) = self.states.get_mut(dungeon_id) {
            if state.is_player_inside(player_id) {
                state.player_left(player_id);

                debug!(
                    dungeon_id = %dungeon_id,
                    player_id = ?player_id,
                    player_count = state.player_count(),
                    "Player left dungeon"
                );

                self.pending_events.push(DungeonEvent::PlayerLeft {
                    dungeon_id: dungeon_id.clone(),
                    player_id,
                });
            }
        }
    }

    /// Check if a player is inside any dungeon.
    pub fn player_in_dungeon(&self, player_id: PlayerId) -> Option<DungeonId> {
        for (dungeon_id, state) in &self.states {
            if state.is_player_inside(player_id) {
                return Some(dungeon_id.clone());
            }
        }
        None
    }

    /// Check if position is inside a dungeon.
    pub fn dungeon_at_position(&self, pos: &Vec3) -> Option<DungeonId> {
        for (dungeon_id, definition) in &self.definitions {
            if definition.contains_position(pos) {
                return Some(dungeon_id.clone());
            }
        }
        None
    }

    /// Get time until boss respawn for a dungeon.
    pub fn time_until_respawn(&self, dungeon_id: &DungeonId) -> Option<Duration> {
        self.states
            .get(dungeon_id)
            .and_then(|s| s.time_until_respawn(Instant::now()))
    }

    /// Check if a player has cleared (looted chest) for a dungeon this run.
    pub fn has_player_cleared(&self, dungeon_id: &DungeonId, player_id: PlayerId) -> bool {
        self.states
            .get(dungeon_id)
            .map(|s| s.has_player_looted(player_id))
            .unwrap_or(false)
    }

    /// Drain pending events.
    pub fn drain_events(&mut self) -> Vec<DungeonEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Drain pending protocol messages.
    pub fn drain_messages(&mut self) -> Vec<(PlayerId, ServerMessage)> {
        std::mem::take(&mut self.pending_messages)
    }

    /// Get all dungeon IDs.
    pub fn dungeon_ids(&self) -> Vec<DungeonId> {
        self.definitions.keys().cloned().collect()
    }

    /// Get dungeon count.
    pub fn dungeon_count(&self) -> usize {
        self.definitions.len()
    }

    // =========================================================================
    // Protocol Message Helpers (T067-T070)
    // =========================================================================

    /// Send DungeonEntered message to a player (T067).
    pub fn send_dungeon_entered(&mut self, dungeon_id: &DungeonId, player_id: PlayerId) {
        let definition = match self.definitions.get(dungeon_id) {
            Some(d) => d,
            None => return,
        };

        let objective = format!("Defeat the {}", definition.display_name);

        self.pending_messages.push((
            player_id,
            ServerMessage::DungeonEntered(DungeonEnteredPayload {
                dungeon_id: dungeon_id.as_str().to_string(),
                display_name: definition.display_name.clone(),
                objective,
            }),
        ));
    }

    /// Send DungeonStateUpdate message to a player (T068).
    pub fn send_state_update(&mut self, dungeon_id: &DungeonId, player_id: PlayerId) {
        let state = match self.states.get(dungeon_id) {
            Some(s) => s,
            None => return,
        };

        self.pending_messages.push((
            player_id,
            ServerMessage::DungeonStateUpdate(DungeonStatePayload {
                dungeon_id: dungeon_id.as_str().to_string(),
                boss_alive: state.boss_alive,
                boss_hp_percent: if state.boss_alive { Some(1.0) } else { None },
                chest_available: state.chest_available,
            }),
        ));
    }

    /// Send DungeonCompleted message to a player (T069).
    pub fn send_dungeon_completed(
        &mut self,
        dungeon_id: &DungeonId,
        player_id: PlayerId,
        loot: &[(ItemId, u32)],
        xp_gained: u32,
    ) {
        let definition = match self.definitions.get(dungeon_id) {
            Some(d) => d,
            None => return,
        };

        self.pending_messages.push((
            player_id,
            ServerMessage::DungeonCompleted(DungeonCompletedPayload {
                dungeon_id: dungeon_id.as_str().to_string(),
                display_name: definition.display_name.clone(),
                rewards: loot
                    .iter()
                    .map(|(id, qty)| (format!("item_{}", id.raw()), *qty))
                    .collect(),
                xp_gained,
            }),
        ));
    }

    /// Send ChestAvailable message to players in dungeon (T070).
    pub fn send_chest_available(&mut self, dungeon_id: &DungeonId) {
        let chest = match self.chests.get(dungeon_id) {
            Some(c) => c,
            None => return,
        };

        let state = match self.states.get(dungeon_id) {
            Some(s) => s,
            None => return,
        };

        // Send to all players inside the dungeon
        for &player_id in state.players_inside() {
            self.pending_messages.push((
                player_id,
                ServerMessage::ChestAvailable(ChestAvailablePayload {
                    chest_id: self.next_chest_id,
                    position: [chest.position.x, chest.position.y, chest.position.z],
                    dungeon_id: dungeon_id.as_str().to_string(),
                }),
            ));
        }
    }

    /// Broadcast dungeon state to all players inside (useful after boss kill/respawn).
    pub fn broadcast_state_to_players(&mut self, dungeon_id: &DungeonId) {
        let player_ids: Vec<PlayerId> = self
            .states
            .get(dungeon_id)
            .map(|s| s.players_inside().to_vec())
            .unwrap_or_default();

        for player_id in player_ids {
            self.send_state_update(dungeon_id, player_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::content::dungeon::RoomDefinition;
    use plix_common::content::ids::{LootTableId, MobDefId};
    use plix_common::content::loot::LootEntry;
    use plix_common::math::AABB;

    fn sample_dungeon() -> DungeonDefinition {
        DungeonDefinition {
            dungeon_id: DungeonId::new("test_dungeon"),
            display_name: "Test Dungeon".to_string(),
            difficulty: 3,
            entry_location: Vec3::new(100.0, 0.0, 100.0),
            rooms: vec![RoomDefinition {
                room_id: "boss_room".to_string(),
                name: "Boss Chamber".to_string(),
                bounds: AABB::new(Vec3::new(100.0, 0.0, 100.0), Vec3::new(150.0, 20.0, 150.0)),
                spawns: vec![],
            }],
            boss_mob_id: MobDefId::new("test_boss"),
            boss_spawn_position: Vec3::new(125.0, 0.0, 125.0),
            reward_loot_table: LootTableId::new("test_loot"),
            boss_respawn_secs: 0.1, // 100ms for testing
        }
    }

    fn sample_loot_table() -> LootTable {
        LootTable {
            loot_table_id: LootTableId::new("test_loot"),
            guaranteed: vec![LootEntry {
                item_id: ItemId::new_raw(1),
                quantity: (1, 1),
                chance: 1.0,
            }],
            entries: vec![],
        }
    }

    #[test]
    fn test_system_creation() {
        let system = DungeonSystem::new(DungeonSystemConfig::default());
        assert_eq!(system.dungeon_count(), 0);
    }

    #[test]
    fn test_register_dungeon() {
        let mut system = DungeonSystem::new(DungeonSystemConfig::default());
        let dungeon = sample_dungeon();
        let dungeon_id = dungeon.dungeon_id.clone();

        system.register_dungeon(dungeon);

        assert_eq!(system.dungeon_count(), 1);
        assert!(system.get_definition(&dungeon_id).is_some());
        assert!(system.get_state(&dungeon_id).is_some());

        let state = system.get_state(&dungeon_id).unwrap();
        assert!(state.boss_alive);
        assert_eq!(state.phase(), DungeonPhase::BossActive);
    }

    #[test]
    fn test_boss_kill_flow() {
        let mut system = DungeonSystem::new(DungeonSystemConfig::default());
        let dungeon = sample_dungeon();
        let dungeon_id = dungeon.dungeon_id.clone();

        system.register_dungeon(dungeon);
        system.register_loot_table(sample_loot_table());

        // Kill boss
        system.on_boss_killed(&dungeon_id, Some(PlayerId(1)));

        let state = system.get_state(&dungeon_id).unwrap();
        assert!(!state.boss_alive);
        assert!(state.chest_available);
        assert_eq!(state.phase(), DungeonPhase::ChestAvailable);

        // Check events
        let events = system.drain_events();
        assert!(events.iter().any(|e| matches!(e, DungeonEvent::BossKilled { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, DungeonEvent::ChestActivated { .. })));
    }

    #[test]
    fn test_chest_loot() {
        let mut system = DungeonSystem::new(DungeonSystemConfig::default());
        let dungeon = sample_dungeon();
        let dungeon_id = dungeon.dungeon_id.clone();

        system.register_dungeon(dungeon);
        system.register_loot_table(sample_loot_table());

        // Can't loot before boss is killed
        let result =
            system.try_loot_chest(&dungeon_id, PlayerId(1), &Vec3::new(127.0, 0.0, 125.0));
        assert_eq!(result, ChestInteractResult::NotActive);

        // Kill boss
        system.on_boss_killed(&dungeon_id, Some(PlayerId(1)));
        system.drain_events();

        // First loot should succeed
        let result =
            system.try_loot_chest(&dungeon_id, PlayerId(1), &Vec3::new(127.0, 0.0, 125.0));
        assert!(matches!(result, ChestInteractResult::Success(_)));

        // Second loot should fail
        let result =
            system.try_loot_chest(&dungeon_id, PlayerId(1), &Vec3::new(127.0, 0.0, 125.0));
        assert_eq!(result, ChestInteractResult::AlreadyLooted);

        // Different player should succeed
        let result =
            system.try_loot_chest(&dungeon_id, PlayerId(2), &Vec3::new(127.0, 0.0, 125.0));
        assert!(matches!(result, ChestInteractResult::Success(_)));
    }

    #[test]
    fn test_chest_range_check() {
        let mut system = DungeonSystem::new(DungeonSystemConfig::default());
        let dungeon = sample_dungeon();
        let dungeon_id = dungeon.dungeon_id.clone();

        system.register_dungeon(dungeon);
        system.register_loot_table(sample_loot_table());
        system.on_boss_killed(&dungeon_id, None);
        system.drain_events();

        // Out of range
        let result =
            system.try_loot_chest(&dungeon_id, PlayerId(1), &Vec3::new(200.0, 0.0, 200.0));
        assert_eq!(result, ChestInteractResult::OutOfRange);
    }

    #[test]
    fn test_boss_respawn() {
        let mut system = DungeonSystem::new(DungeonSystemConfig::default());
        let dungeon = sample_dungeon();
        let dungeon_id = dungeon.dungeon_id.clone();

        system.register_dungeon(dungeon);
        system.register_loot_table(sample_loot_table());

        let now = Instant::now();
        system.on_boss_killed(&dungeon_id, None);
        system.drain_events();

        // Loot before respawn
        let result =
            system.try_loot_chest(&dungeon_id, PlayerId(1), &Vec3::new(127.0, 0.0, 125.0));
        assert!(matches!(result, ChestInteractResult::Success(_)));

        // Wait for respawn
        let later = now + Duration::from_millis(150);
        system.update(later);

        let state = system.get_state(&dungeon_id).unwrap();
        assert!(state.boss_alive);
        assert!(!state.chest_available);

        // Check events
        let events = system.drain_events();
        assert!(events
            .iter()
            .any(|e| matches!(e, DungeonEvent::BossRespawned { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, DungeonEvent::ChestDeactivated { .. })));

        // Chest should be inactive
        let result =
            system.try_loot_chest(&dungeon_id, PlayerId(1), &Vec3::new(127.0, 0.0, 125.0));
        assert_eq!(result, ChestInteractResult::NotActive);
    }

    #[test]
    fn test_player_tracking() {
        let mut system = DungeonSystem::new(DungeonSystemConfig::default());
        let dungeon = sample_dungeon();
        let dungeon_id = dungeon.dungeon_id.clone();

        system.register_dungeon(dungeon);

        // Player enters
        system.on_player_entered(&dungeon_id, PlayerId(1));
        assert_eq!(system.player_in_dungeon(PlayerId(1)), Some(dungeon_id.clone()));

        let state = system.get_state(&dungeon_id).unwrap();
        assert_eq!(state.player_count(), 1);

        // Player leaves
        system.on_player_left(&dungeon_id, PlayerId(1));
        assert!(system.player_in_dungeon(PlayerId(1)).is_none());

        let events = system.drain_events();
        assert!(events
            .iter()
            .any(|e| matches!(e, DungeonEvent::PlayerEntered { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, DungeonEvent::PlayerLeft { .. })));
    }

    #[test]
    fn test_dungeon_at_position() {
        let mut system = DungeonSystem::new(DungeonSystemConfig::default());
        let dungeon = sample_dungeon();
        let dungeon_id = dungeon.dungeon_id.clone();

        system.register_dungeon(dungeon);

        // Inside dungeon bounds
        let pos = Vec3::new(125.0, 10.0, 125.0);
        assert_eq!(system.dungeon_at_position(&pos), Some(dungeon_id));

        // Outside dungeon bounds
        let pos = Vec3::new(0.0, 0.0, 0.0);
        assert!(system.dungeon_at_position(&pos).is_none());
    }

    #[test]
    fn test_has_player_cleared() {
        let mut system = DungeonSystem::new(DungeonSystemConfig::default());
        let dungeon = sample_dungeon();
        let dungeon_id = dungeon.dungeon_id.clone();

        system.register_dungeon(dungeon);
        system.register_loot_table(sample_loot_table());
        system.on_boss_killed(&dungeon_id, None);

        assert!(!system.has_player_cleared(&dungeon_id, PlayerId(1)));

        system.try_loot_chest(&dungeon_id, PlayerId(1), &Vec3::new(127.0, 0.0, 125.0));

        assert!(system.has_player_cleared(&dungeon_id, PlayerId(1)));
    }
}
