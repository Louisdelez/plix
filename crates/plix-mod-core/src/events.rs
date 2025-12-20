//! Event bus: subscription management and phase-based dispatch
//!
//! Events are collected during the game tick and dispatched at end-of-tick
//! to all subscribed mods in FIFO order.

use crate::capabilities::Capability;
use crate::errors::{err_invalid, err_perm, ModApiError};
use crate::registry::{ModRegistry, ERROR_THRESHOLD};
use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tracing::{debug, error, warn};

/// Event types supported in MVP
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// Server started
    ServerStart,
    /// Server stopping
    ServerStop,
    /// Player joined the game
    PlayerJoin,
    /// Player left the game
    PlayerLeave,
    /// Player sent a chat message (cancellable)
    PlayerChat,
    /// Block was placed (cancellable)
    BlockPlaced,
    /// Block was broken (cancellable)
    BlockBroken,
    /// Entity took damage
    EntityDamaged,
    /// Mod-to-mod message received
    ModMessage,
    /// Mob was spawned
    MobSpawned,
    /// Mob took damage
    MobDamaged,
    /// Mob was killed
    MobKilled,
    /// Loot was dropped from a mob
    MobLootDropped,
    /// XP/credits were distributed for a mob kill
    MobRewardDistributed,
    /// Quest was started
    QuestStarted,
    /// Quest step was completed
    QuestStepCompleted,
    /// Quest was completed
    QuestCompleted,
    /// Quest was abandoned
    QuestAbandoned,
    /// Quest became available (prerequisites met)
    QuestAvailable,
    /// Dungeon boss was killed
    DungeonBossKilled,
    /// Dungeon boss respawned
    DungeonBossRespawned,
    /// Dungeon chest was looted
    DungeonChestLooted,
    /// Player entered a dungeon
    DungeonPlayerEntered,
    /// Player left a dungeon
    DungeonPlayerLeft,
}

impl EventType {
    /// Check if this event type can be cancelled
    pub fn is_cancellable(&self) -> bool {
        matches!(
            self,
            EventType::PlayerChat | EventType::BlockPlaced | EventType::BlockBroken
        )
    }

    /// Get the capability required to cancel this event type
    pub fn cancel_capability(&self) -> Option<Capability> {
        match self {
            EventType::PlayerChat => Some(Capability::EVENT_CANCEL_CHAT),
            EventType::BlockPlaced | EventType::BlockBroken => {
                Some(Capability::EVENT_CANCEL_BLOCKS)
            }
            _ => None,
        }
    }
}

// === Event Payloads ===

/// Server start payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStartPayload {
    /// Server tick number
    pub tick: u64,
}

/// Server stop payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStopPayload {
    /// Shutdown reason
    pub reason: String,
}

/// Player join payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerJoinPayload {
    /// Unique player ID
    pub player_id: u64,
    /// Player display name
    pub name: String,
}

/// Reason why a player left
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeaveReason {
    /// Player disconnected normally
    Disconnect,
    /// Player was kicked
    Kicked,
    /// Player timed out
    Timeout,
    /// Server is shutting down
    ServerShutdown,
}

/// Player leave payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLeavePayload {
    /// Unique player ID
    pub player_id: u64,
    /// Why player left
    pub reason: LeaveReason,
}

/// Player chat payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerChatPayload {
    /// Sender player ID
    pub player_id: u64,
    /// Message content
    pub text: String,
}

/// Block placed payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPlacedPayload {
    /// Placer player ID (None if world-generated)
    pub player_id: Option<u64>,
    /// Block position
    pub pos: IVec3,
    /// Block type ID
    pub block_id: u16,
}

/// Block broken payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockBrokenPayload {
    /// Breaker player ID (None if world-generated)
    pub player_id: Option<u64>,
    /// Block position
    pub pos: IVec3,
    /// Previous block type ID
    pub block_id: u16,
}

/// Damage source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageSource {
    /// Source type (e.g., "player", "fall", "environment")
    pub source_type: String,
    /// Entity that caused the damage (if any)
    pub source_entity: Option<u64>,
    /// Direction damage came from
    pub direction: Option<Vec3>,
}

/// Entity damaged payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDamagedPayload {
    /// Damaged entity ID
    pub entity_id: u64,
    /// Damage amount
    pub amount: f32,
    /// Damage source info
    pub source: Option<DamageSource>,
}

/// Message source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageSource {
    /// Message from server
    Server,
    /// Message from a client
    Client(u64),
}

/// Mod message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMessagePayload {
    /// Channel name (mod:id:name)
    pub channel: String,
    /// Message sender
    pub from: MessageSource,
    /// Message bytes (max 8KB)
    pub payload: Vec<u8>,
}

// === Mob Event Payloads ===

/// Mob spawned payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobSpawnedPayload {
    /// Unique runtime mob instance ID
    pub mob_id: u64,
    /// Mob definition ID (e.g., "cave_rat")
    pub def_id: String,
    /// Spawn position
    pub position: Vec3,
    /// Spawn point ID
    pub spawn_id: String,
}

/// Mob damaged payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobDamagedPayload {
    /// Mob instance ID
    pub mob_id: u64,
    /// Player who dealt the damage
    pub attacker_id: u64,
    /// Damage amount
    pub damage: u32,
    /// HP remaining after damage
    pub remaining_hp: u32,
}

/// Mob killed payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobKilledPayload {
    /// Mob instance ID
    pub mob_id: u64,
    /// Mob definition ID
    pub def_id: String,
    /// Player who got the killing blow (if any)
    pub killer_id: Option<u64>,
    /// Position where mob died
    pub position: Vec3,
}

/// Single loot item info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootItemInfo {
    /// Item ID
    pub item_id: u32,
    /// Quantity dropped
    pub quantity: u32,
}

/// Mob loot dropped payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobLootDroppedPayload {
    /// Mob instance ID that dropped the loot
    pub mob_id: u64,
    /// Position where loot dropped
    pub position: Vec3,
    /// Items dropped
    pub items: Vec<LootItemInfo>,
}

/// Player reward info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRewardInfo {
    /// Player ID
    pub player_id: u64,
    /// XP awarded
    pub xp: u32,
    /// Credits awarded
    pub credits: u32,
    /// Whether this player got the killing blow
    pub is_killer: bool,
}

/// Mob reward distributed payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobRewardDistributedPayload {
    /// Mob instance ID
    pub mob_id: u64,
    /// Mob definition ID
    pub def_id: String,
    /// Rewards distributed to each player
    pub rewards: Vec<PlayerRewardInfo>,
    /// Total XP distributed
    pub total_xp: u32,
    /// Total credits distributed
    pub total_credits: u32,
}

// === Quest Event Payloads ===

/// Quest started payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestStartedPayload {
    /// Player who started the quest
    pub player_id: u64,
    /// Quest ID
    pub quest_id: String,
}

/// Quest step completed payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestStepCompletedPayload {
    /// Player who completed the step
    pub player_id: u64,
    /// Quest ID
    pub quest_id: String,
    /// Step index that was completed
    pub step_index: u32,
    /// Step type (e.g., "KillMob", "TalkToNpc")
    pub step_type: String,
}

/// Quest completed payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestCompletedPayload {
    /// Player who completed the quest
    pub player_id: u64,
    /// Quest ID
    pub quest_id: String,
    /// XP rewarded
    pub xp: u32,
    /// Currency rewarded
    pub currency: u32,
}

/// Quest abandoned payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestAbandonedPayload {
    /// Player who abandoned the quest
    pub player_id: u64,
    /// Quest ID
    pub quest_id: String,
}

/// Quest available payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestAvailablePayload {
    /// Player for whom the quest is now available
    pub player_id: u64,
    /// Quest ID
    pub quest_id: String,
}

// === Dungeon Event Payloads ===

/// Dungeon boss killed payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonBossKilledPayload {
    /// Dungeon ID
    pub dungeon_id: String,
    /// Player who got the killing blow (if any)
    pub killer_id: Option<u64>,
    /// Time until boss respawns (seconds)
    pub respawn_secs: f32,
}

/// Dungeon boss respawned payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonBossRespawnedPayload {
    /// Dungeon ID
    pub dungeon_id: String,
}

/// Dungeon chest looted payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonChestLootedPayload {
    /// Dungeon ID
    pub dungeon_id: String,
    /// Player who looted the chest
    pub player_id: u64,
    /// Items received
    pub items: Vec<LootItemInfo>,
}

/// Dungeon player entered payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonPlayerEnteredPayload {
    /// Dungeon ID
    pub dungeon_id: String,
    /// Player who entered
    pub player_id: u64,
    /// Current player count in dungeon
    pub player_count: u32,
}

/// Dungeon player left payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonPlayerLeftPayload {
    /// Dungeon ID
    pub dungeon_id: String,
    /// Player who left
    pub player_id: u64,
    /// Remaining player count in dungeon
    pub player_count: u32,
}

/// Union of all event payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayload {
    ServerStart(ServerStartPayload),
    ServerStop(ServerStopPayload),
    PlayerJoin(PlayerJoinPayload),
    PlayerLeave(PlayerLeavePayload),
    PlayerChat(PlayerChatPayload),
    BlockPlaced(BlockPlacedPayload),
    BlockBroken(BlockBrokenPayload),
    EntityDamaged(EntityDamagedPayload),
    ModMessage(ModMessagePayload),
    MobSpawned(MobSpawnedPayload),
    MobDamaged(MobDamagedPayload),
    MobKilled(MobKilledPayload),
    MobLootDropped(MobLootDroppedPayload),
    MobRewardDistributed(MobRewardDistributedPayload),
    QuestStarted(QuestStartedPayload),
    QuestStepCompleted(QuestStepCompletedPayload),
    QuestCompleted(QuestCompletedPayload),
    QuestAbandoned(QuestAbandonedPayload),
    QuestAvailable(QuestAvailablePayload),
    DungeonBossKilled(DungeonBossKilledPayload),
    DungeonBossRespawned(DungeonBossRespawnedPayload),
    DungeonChestLooted(DungeonChestLootedPayload),
    DungeonPlayerEntered(DungeonPlayerEnteredPayload),
    DungeonPlayerLeft(DungeonPlayerLeftPayload),
}

/// A game event emitted by the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    /// Event type
    pub event_type: EventType,
    /// Type-specific payload
    pub payload: EventPayload,
    /// Server tick when emitted
    pub timestamp: u64,
    /// Whether this event can be cancelled
    pub cancellable: bool,
    /// Set to true by handler to cancel the event
    pub cancelled: bool,
}

impl GameEvent {
    /// Create a new event
    pub fn new(event_type: EventType, payload: EventPayload, timestamp: u64) -> Self {
        Self {
            cancellable: event_type.is_cancellable(),
            event_type,
            payload,
            timestamp,
            cancelled: false,
        }
    }

    /// Create a ServerStart event
    pub fn server_start(tick: u64) -> Self {
        Self::new(
            EventType::ServerStart,
            EventPayload::ServerStart(ServerStartPayload { tick }),
            tick,
        )
    }

    /// Create a ServerStop event
    pub fn server_stop(reason: impl Into<String>, tick: u64) -> Self {
        Self::new(
            EventType::ServerStop,
            EventPayload::ServerStop(ServerStopPayload {
                reason: reason.into(),
            }),
            tick,
        )
    }

    /// Create a PlayerJoin event
    pub fn player_join(player_id: u64, name: impl Into<String>, tick: u64) -> Self {
        Self::new(
            EventType::PlayerJoin,
            EventPayload::PlayerJoin(PlayerJoinPayload {
                player_id,
                name: name.into(),
            }),
            tick,
        )
    }

    /// Create a PlayerLeave event
    pub fn player_leave(player_id: u64, reason: LeaveReason, tick: u64) -> Self {
        Self::new(
            EventType::PlayerLeave,
            EventPayload::PlayerLeave(PlayerLeavePayload { player_id, reason }),
            tick,
        )
    }

    /// Create a PlayerChat event
    pub fn player_chat(player_id: u64, text: impl Into<String>, tick: u64) -> Self {
        Self::new(
            EventType::PlayerChat,
            EventPayload::PlayerChat(PlayerChatPayload {
                player_id,
                text: text.into(),
            }),
            tick,
        )
    }

    /// Create a BlockPlaced event
    pub fn block_placed(player_id: Option<u64>, pos: IVec3, block_id: u16, tick: u64) -> Self {
        Self::new(
            EventType::BlockPlaced,
            EventPayload::BlockPlaced(BlockPlacedPayload {
                player_id,
                pos,
                block_id,
            }),
            tick,
        )
    }

    /// Create a BlockBroken event
    pub fn block_broken(player_id: Option<u64>, pos: IVec3, block_id: u16, tick: u64) -> Self {
        Self::new(
            EventType::BlockBroken,
            EventPayload::BlockBroken(BlockBrokenPayload {
                player_id,
                pos,
                block_id,
            }),
            tick,
        )
    }

    /// Create an EntityDamaged event
    pub fn entity_damaged(
        entity_id: u64,
        amount: f32,
        source: Option<DamageSource>,
        tick: u64,
    ) -> Self {
        Self::new(
            EventType::EntityDamaged,
            EventPayload::EntityDamaged(EntityDamagedPayload {
                entity_id,
                amount,
                source,
            }),
            tick,
        )
    }

    /// Create a ModMessage event
    pub fn mod_message(
        channel: impl Into<String>,
        from: MessageSource,
        payload: Vec<u8>,
        tick: u64,
    ) -> Self {
        Self::new(
            EventType::ModMessage,
            EventPayload::ModMessage(ModMessagePayload {
                channel: channel.into(),
                from,
                payload,
            }),
            tick,
        )
    }

    // === Mob Events ===

    /// Create a MobSpawned event
    pub fn mob_spawned(
        mob_id: u64,
        def_id: impl Into<String>,
        position: Vec3,
        spawn_id: impl Into<String>,
        tick: u64,
    ) -> Self {
        Self::new(
            EventType::MobSpawned,
            EventPayload::MobSpawned(MobSpawnedPayload {
                mob_id,
                def_id: def_id.into(),
                position,
                spawn_id: spawn_id.into(),
            }),
            tick,
        )
    }

    /// Create a MobDamaged event
    pub fn mob_damaged(
        mob_id: u64,
        attacker_id: u64,
        damage: u32,
        remaining_hp: u32,
        tick: u64,
    ) -> Self {
        Self::new(
            EventType::MobDamaged,
            EventPayload::MobDamaged(MobDamagedPayload {
                mob_id,
                attacker_id,
                damage,
                remaining_hp,
            }),
            tick,
        )
    }

    /// Create a MobKilled event
    pub fn mob_killed(
        mob_id: u64,
        def_id: impl Into<String>,
        killer_id: Option<u64>,
        position: Vec3,
        tick: u64,
    ) -> Self {
        Self::new(
            EventType::MobKilled,
            EventPayload::MobKilled(MobKilledPayload {
                mob_id,
                def_id: def_id.into(),
                killer_id,
                position,
            }),
            tick,
        )
    }

    /// Create a MobLootDropped event
    pub fn mob_loot_dropped(
        mob_id: u64,
        position: Vec3,
        items: Vec<LootItemInfo>,
        tick: u64,
    ) -> Self {
        Self::new(
            EventType::MobLootDropped,
            EventPayload::MobLootDropped(MobLootDroppedPayload {
                mob_id,
                position,
                items,
            }),
            tick,
        )
    }

    /// Create a MobRewardDistributed event
    pub fn mob_reward_distributed(
        mob_id: u64,
        def_id: impl Into<String>,
        rewards: Vec<PlayerRewardInfo>,
        total_xp: u32,
        total_credits: u32,
        tick: u64,
    ) -> Self {
        Self::new(
            EventType::MobRewardDistributed,
            EventPayload::MobRewardDistributed(MobRewardDistributedPayload {
                mob_id,
                def_id: def_id.into(),
                rewards,
                total_xp,
                total_credits,
            }),
            tick,
        )
    }

    // === Quest Events ===

    /// Create a QuestStarted event
    pub fn quest_started(player_id: u64, quest_id: impl Into<String>, tick: u64) -> Self {
        Self::new(
            EventType::QuestStarted,
            EventPayload::QuestStarted(QuestStartedPayload {
                player_id,
                quest_id: quest_id.into(),
            }),
            tick,
        )
    }

    /// Create a QuestStepCompleted event
    pub fn quest_step_completed(
        player_id: u64,
        quest_id: impl Into<String>,
        step_index: u32,
        step_type: impl Into<String>,
        tick: u64,
    ) -> Self {
        Self::new(
            EventType::QuestStepCompleted,
            EventPayload::QuestStepCompleted(QuestStepCompletedPayload {
                player_id,
                quest_id: quest_id.into(),
                step_index,
                step_type: step_type.into(),
            }),
            tick,
        )
    }

    /// Create a QuestCompleted event
    pub fn quest_completed(
        player_id: u64,
        quest_id: impl Into<String>,
        xp: u32,
        currency: u32,
        tick: u64,
    ) -> Self {
        Self::new(
            EventType::QuestCompleted,
            EventPayload::QuestCompleted(QuestCompletedPayload {
                player_id,
                quest_id: quest_id.into(),
                xp,
                currency,
            }),
            tick,
        )
    }

    /// Create a QuestAbandoned event
    pub fn quest_abandoned(player_id: u64, quest_id: impl Into<String>, tick: u64) -> Self {
        Self::new(
            EventType::QuestAbandoned,
            EventPayload::QuestAbandoned(QuestAbandonedPayload {
                player_id,
                quest_id: quest_id.into(),
            }),
            tick,
        )
    }

    /// Create a QuestAvailable event
    pub fn quest_available(player_id: u64, quest_id: impl Into<String>, tick: u64) -> Self {
        Self::new(
            EventType::QuestAvailable,
            EventPayload::QuestAvailable(QuestAvailablePayload {
                player_id,
                quest_id: quest_id.into(),
            }),
            tick,
        )
    }

    // === Dungeon Events ===

    /// Create a DungeonBossKilled event
    pub fn dungeon_boss_killed(
        dungeon_id: impl Into<String>,
        killer_id: Option<u64>,
        respawn_secs: f32,
        tick: u64,
    ) -> Self {
        Self::new(
            EventType::DungeonBossKilled,
            EventPayload::DungeonBossKilled(DungeonBossKilledPayload {
                dungeon_id: dungeon_id.into(),
                killer_id,
                respawn_secs,
            }),
            tick,
        )
    }

    /// Create a DungeonBossRespawned event
    pub fn dungeon_boss_respawned(dungeon_id: impl Into<String>, tick: u64) -> Self {
        Self::new(
            EventType::DungeonBossRespawned,
            EventPayload::DungeonBossRespawned(DungeonBossRespawnedPayload {
                dungeon_id: dungeon_id.into(),
            }),
            tick,
        )
    }

    /// Create a DungeonChestLooted event
    pub fn dungeon_chest_looted(
        dungeon_id: impl Into<String>,
        player_id: u64,
        items: Vec<LootItemInfo>,
        tick: u64,
    ) -> Self {
        Self::new(
            EventType::DungeonChestLooted,
            EventPayload::DungeonChestLooted(DungeonChestLootedPayload {
                dungeon_id: dungeon_id.into(),
                player_id,
                items,
            }),
            tick,
        )
    }

    /// Create a DungeonPlayerEntered event
    pub fn dungeon_player_entered(
        dungeon_id: impl Into<String>,
        player_id: u64,
        player_count: u32,
        tick: u64,
    ) -> Self {
        Self::new(
            EventType::DungeonPlayerEntered,
            EventPayload::DungeonPlayerEntered(DungeonPlayerEnteredPayload {
                dungeon_id: dungeon_id.into(),
                player_id,
                player_count,
            }),
            tick,
        )
    }

    /// Create a DungeonPlayerLeft event
    pub fn dungeon_player_left(
        dungeon_id: impl Into<String>,
        player_id: u64,
        player_count: u32,
        tick: u64,
    ) -> Self {
        Self::new(
            EventType::DungeonPlayerLeft,
            EventPayload::DungeonPlayerLeft(DungeonPlayerLeftPayload {
                dungeon_id: dungeon_id.into(),
                player_id,
                player_count,
            }),
            tick,
        )
    }
}

/// Handler result from mod event processing
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Event handler function type
pub type EventHandler = Box<dyn Fn(&str, &GameEvent) -> HandlerResult + Send + Sync>;

/// Event bus for collecting and dispatching events
#[derive(Default)]
pub struct EventBus {
    /// Pending events to dispatch
    queue: VecDeque<GameEvent>,
    /// Currently processing (prevents re-entrancy)
    dispatching: bool,
    /// Event handler (called for each subscribed mod)
    handler: Option<EventHandler>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the event handler
    pub fn set_handler(&mut self, handler: EventHandler) {
        self.handler = Some(handler);
    }

    /// Queue an event for dispatch at end-of-tick
    pub fn emit(&mut self, event: GameEvent) {
        debug!(event_type = ?event.event_type, tick = event.timestamp, "Event emitted");
        self.queue.push_back(event);
    }

    /// Check if we're currently dispatching (prevents re-entrancy)
    pub fn is_dispatching(&self) -> bool {
        self.dispatching
    }

    /// Get the number of pending events
    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }

    /// Dispatch all pending events to subscribed mods
    ///
    /// Returns the number of events that were cancelled.
    pub fn dispatch(&mut self, registry: &mut ModRegistry) -> usize {
        if self.dispatching {
            warn!("Attempted re-entrant dispatch - ignoring");
            return 0;
        }

        self.dispatching = true;
        let mut cancelled_count = 0;

        while let Some(event) = self.queue.pop_front() {
            // Get all active mods subscribed to this event type
            let subscribers: Vec<String> = registry
                .iter_active()
                .filter(|(_, ctx)| ctx.is_subscribed(event.event_type))
                .map(|(id, _)| id.to_string())
                .collect();

            for mod_id in subscribers {
                // Skip if event was cancelled and this is a cancellable event
                if event.cancelled && event.cancellable {
                    break;
                }

                // Call the handler
                if let Some(ref handler) = self.handler {
                    match handler(&mod_id, &event) {
                        Ok(()) => {
                            // Success - reset error counter
                            if let Some(ctx) = registry.get_mut(&mod_id) {
                                ctx.record_success();
                            }
                        }
                        Err(e) => {
                            error!(mod_id = %mod_id, error = %e, "Event handler error");

                            // Record error and check for auto-disable
                            if let Some(ctx) = registry.get_mut(&mod_id) {
                                if ctx.record_error() {
                                    let reason = format!(
                                        "Auto-disabled after {} consecutive errors",
                                        ERROR_THRESHOLD
                                    );
                                    warn!(mod_id = %mod_id, "Mod auto-disabled");
                                    if let Err(e) = registry.disable_mod(&mod_id, reason) {
                                        error!(error = %e, "Failed to disable mod");
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if event.cancelled {
                cancelled_count += 1;
            }
        }

        self.dispatching = false;
        cancelled_count
    }

    /// Clear all pending events
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

/// Context for the currently processing event (used for cancel_event)
#[derive(Debug, Default)]
pub struct EventContext {
    /// Currently processing event (for cancellation)
    pub current_event: Option<GameEvent>,
    /// Mod ID that is currently handling
    pub current_mod_id: Option<String>,
}

impl EventContext {
    /// Create a new event context
    pub fn new() -> Self {
        Self::default()
    }

    /// Cancel the current event if allowed
    pub fn cancel_event(&mut self, capabilities: Capability) -> Result<(), ModApiError> {
        let event = self
            .current_event
            .as_mut()
            .ok_or_else(|| err_invalid("Not in event handler context"))?;

        if !event.cancellable {
            return Err(err_invalid("This event type cannot be cancelled"));
        }

        // Check capability
        if let Some(required_cap) = event.event_type.cancel_capability() {
            if !capabilities.contains(required_cap) {
                return Err(err_perm(required_cap.as_str()));
            }
        }

        event.cancelled = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ModManifest;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    fn test_manifest(id: &str) -> ModManifest {
        ModManifest {
            id: id.to_string(),
            name: format!("Test Mod {}", id),
            version: "1.0.0".to_string(),
            author: None,
            api_version: 1,
            min_api_version: None,
            max_api_version: None,
            capabilities: Default::default(),
            entrypoints: Default::default(),
            dependencies: vec![],
        }
    }

    #[test]
    fn test_event_type_cancellable() {
        assert!(EventType::PlayerChat.is_cancellable());
        assert!(EventType::BlockPlaced.is_cancellable());
        assert!(EventType::BlockBroken.is_cancellable());
        assert!(!EventType::PlayerJoin.is_cancellable());
        assert!(!EventType::ServerStart.is_cancellable());
    }

    #[test]
    fn test_event_creation() {
        let event = GameEvent::player_join(123, "TestPlayer", 1);
        assert_eq!(event.event_type, EventType::PlayerJoin);
        assert_eq!(event.timestamp, 1);
        assert!(!event.cancellable);

        if let EventPayload::PlayerJoin(payload) = event.payload {
            assert_eq!(payload.player_id, 123);
            assert_eq!(payload.name, "TestPlayer");
        } else {
            panic!("Wrong payload type");
        }
    }

    #[test]
    fn test_event_bus_emit() {
        let mut bus = EventBus::new();
        assert_eq!(bus.pending_count(), 0);

        bus.emit(GameEvent::server_start(1));
        bus.emit(GameEvent::player_join(1, "Player1", 2));

        assert_eq!(bus.pending_count(), 2);
    }

    #[test]
    fn test_event_bus_dispatch_fifo() {
        let mut bus = EventBus::new();
        let mut registry = ModRegistry::new();

        let manifest = test_manifest("test-mod");
        registry.load(manifest).unwrap();
        registry
            .get_mut("test-mod")
            .unwrap()
            .subscribe(EventType::PlayerJoin);

        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let order_clone = order.clone();

        bus.set_handler(Box::new(move |mod_id, event| {
            order_clone
                .lock()
                .unwrap()
                .push((mod_id.to_string(), event.timestamp));
            Ok(())
        }));

        bus.emit(GameEvent::player_join(1, "P1", 1));
        bus.emit(GameEvent::player_join(2, "P2", 2));
        bus.emit(GameEvent::player_join(3, "P3", 3));

        bus.dispatch(&mut registry);

        let order = order.lock().unwrap();
        assert_eq!(order.len(), 3);
        assert_eq!(order[0].1, 1);
        assert_eq!(order[1].1, 2);
        assert_eq!(order[2].1, 3);
    }

    #[test]
    fn test_event_bus_error_isolation() {
        let mut bus = EventBus::new();
        let mut registry = ModRegistry::new();

        let manifest = test_manifest("failing-mod");
        registry.load(manifest).unwrap();
        registry
            .get_mut("failing-mod")
            .unwrap()
            .subscribe(EventType::PlayerJoin);

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        bus.set_handler(Box::new(move |_, _| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            Err("Handler error".into())
        }));

        // Emit events - handler always fails but should continue
        for i in 0..3 {
            bus.emit(GameEvent::player_join(i, format!("P{}", i), i));
        }

        bus.dispatch(&mut registry);

        // All 3 handlers were called despite errors
        assert_eq!(call_count.load(Ordering::SeqCst), 3);

        // Mod should have 3 consecutive errors
        assert_eq!(registry.get("failing-mod").unwrap().error_count, 3);
    }

    #[test]
    fn test_event_bus_auto_disable() {
        let mut bus = EventBus::new();
        let mut registry = ModRegistry::new();

        let manifest = test_manifest("failing-mod");
        registry.load(manifest).unwrap();
        registry
            .get_mut("failing-mod")
            .unwrap()
            .subscribe(EventType::PlayerJoin);

        bus.set_handler(Box::new(|_, _| Err("Handler error".into())));

        // Emit 5 events to trigger auto-disable
        for i in 0..5 {
            bus.emit(GameEvent::player_join(i, format!("P{}", i), i));
        }

        bus.dispatch(&mut registry);

        // Mod should be disabled
        assert!(!registry.is_enabled("failing-mod"));
        assert!(registry
            .get("failing-mod")
            .unwrap()
            .disable_reason
            .as_ref()
            .unwrap()
            .contains("5 consecutive errors"));
    }

    #[test]
    fn test_event_context_cancel() {
        let mut ctx = EventContext::new();
        ctx.current_event = Some(GameEvent::player_chat(1, "test", 1));
        ctx.current_mod_id = Some("test-mod".to_string());

        // Cancel with required capability
        let caps = Capability::EVENT_CANCEL_CHAT;
        assert!(ctx.cancel_event(caps).is_ok());
        assert!(ctx.current_event.as_ref().unwrap().cancelled);
    }

    #[test]
    fn test_event_context_cancel_without_capability() {
        let mut ctx = EventContext::new();
        ctx.current_event = Some(GameEvent::player_chat(1, "test", 1));
        ctx.current_mod_id = Some("test-mod".to_string());

        // Try to cancel without capability
        let caps = Capability::empty();
        let result = ctx.cancel_event(caps);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::PermissionDenied
        );
    }

    #[test]
    fn test_event_context_cancel_non_cancellable() {
        let mut ctx = EventContext::new();
        ctx.current_event = Some(GameEvent::player_join(1, "test", 1));
        ctx.current_mod_id = Some("test-mod".to_string());

        let caps = Capability::all();
        let result = ctx.cancel_event(caps);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("cannot be cancelled"));
    }

    #[test]
    fn test_subscription_filtering() {
        let mut registry = ModRegistry::new();

        let manifest1 = test_manifest("mod1");
        let manifest2 = test_manifest("mod2");

        registry.load(manifest1).unwrap();
        registry.load(manifest2).unwrap();

        registry
            .get_mut("mod1")
            .unwrap()
            .subscribe(EventType::PlayerJoin);
        registry
            .get_mut("mod2")
            .unwrap()
            .subscribe(EventType::PlayerChat);

        // Only mod1 should receive PlayerJoin
        let join_subscribers: Vec<_> = registry
            .iter_active()
            .filter(|(_, ctx)| ctx.is_subscribed(EventType::PlayerJoin))
            .map(|(id, _)| id)
            .collect();

        assert_eq!(join_subscribers.len(), 1);
        assert_eq!(join_subscribers[0], "mod1");

        // Only mod2 should receive PlayerChat
        let chat_subscribers: Vec<_> = registry
            .iter_active()
            .filter(|(_, ctx)| ctx.is_subscribed(EventType::PlayerChat))
            .map(|(id, _)| id)
            .collect();

        assert_eq!(chat_subscribers.len(), 1);
        assert_eq!(chat_subscribers[0], "mod2");
    }
}
