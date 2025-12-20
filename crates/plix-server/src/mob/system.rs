//! Main mob system coordinating spawning, AI, combat, and loot
//!
//! The MobSystem is the central manager for all mob-related logic:
//! - Processes spawn points to create mobs
//! - Updates mob AI and movement
//! - Handles combat damage and death
//! - Manages loot drops and pickup

use std::collections::HashMap;
use std::time::{Duration, Instant};

use tracing::{debug, info, warn};

use plix_common::content::ids::{MobDefId, MobInstanceId, SpawnId};
use plix_common::content::mob::MobDefinition;
use plix_common::content::spawn::SpawnPointDefinition;
use plix_common::content::{ContentRegistry, LootTable};
use plix_common::math::Vec3;
use plix_common::protocol::{
    MobDamagedPayload, MobDiedPayload, MobLootDropPayload, MobSpawnPayload, MobUpdatePayload,
    RewardPayload, RewardType, ServerMessage,
};
use plix_common::types::{ItemId, PlayerId};

use super::ai::{update_ai, AiAction, MobAiState, PlayerInfo};
use super::instance::MobInstance;
use super::payout::{PayoutCalculator, PayoutResult};
use super::spawn::SpawnManager;

/// Configuration for the mob system.
#[derive(Debug, Clone)]
pub struct MobSystemConfig {
    /// Maximum distance to consider a player for targeting
    pub max_player_distance: f32,
    /// How often to process spawn points (seconds)
    pub spawn_check_interval: f32,
}

impl Default for MobSystemConfig {
    fn default() -> Self {
        Self {
            max_player_distance: 100.0,
            spawn_check_interval: 1.0,
        }
    }
}

/// A loot drop entity in the world.
#[derive(Debug, Clone)]
pub struct LootDrop {
    /// Unique ID for this drop
    pub id: u64,
    /// Position in the world
    pub position: Vec3,
    /// Item and quantity
    pub item_id: ItemId,
    pub quantity: u32,
    /// Time when this drop was created
    pub created_at: Instant,
    /// Time when this drop expires
    pub expires_at: Instant,
}

/// Event emitted by the mob system.
#[derive(Debug, Clone)]
pub enum MobEvent {
    /// A mob was spawned
    Spawned {
        mob_id: MobInstanceId,
        def_id: MobDefId,
        position: Vec3,
    },
    /// A mob took damage
    Damaged {
        mob_id: MobInstanceId,
        attacker: PlayerId,
        damage: u32,
        remaining_hp: u32,
    },
    /// A mob died
    Killed {
        mob_id: MobInstanceId,
        def_id: MobDefId,
        killer: Option<PlayerId>,
        position: Vec3,
    },
    /// Loot was dropped
    LootDropped {
        mob_id: MobInstanceId,
        position: Vec3,
        items: Vec<(ItemId, u32)>,
    },
    /// Rewards were distributed
    RewardDistributed {
        mob_id: MobInstanceId,
        payouts: PayoutResult,
    },
}

/// The main mob system.
pub struct MobSystem {
    /// Configuration
    config: MobSystemConfig,
    /// Spawn point manager
    spawn_manager: SpawnManager,
    /// Active mob instances
    mobs: HashMap<MobInstanceId, MobInstance>,
    /// Mob definitions (borrowed from content registry)
    mob_defs: HashMap<MobDefId, MobDefinition>,
    /// Loot tables (borrowed from content registry)
    loot_tables: HashMap<String, LootTable>,
    /// Active loot drops
    loot_drops: HashMap<u64, LootDrop>,
    /// Next loot drop ID
    next_loot_id: u64,
    /// Payout calculator
    payout_calc: PayoutCalculator,
    /// Last spawn check time
    last_spawn_check: Instant,
    /// Events generated this tick (consumed by caller)
    pending_events: Vec<MobEvent>,
    /// Messages to send to clients
    pending_messages: Vec<(Option<PlayerId>, ServerMessage)>,
    /// Loot drop lifetime in seconds
    loot_lifetime: Duration,
}

impl MobSystem {
    /// Create a new mob system.
    pub fn new(config: MobSystemConfig) -> Self {
        Self {
            config,
            spawn_manager: SpawnManager::new(),
            mobs: HashMap::new(),
            mob_defs: HashMap::new(),
            loot_tables: HashMap::new(),
            loot_drops: HashMap::new(),
            next_loot_id: 1,
            payout_calc: PayoutCalculator::new(),
            last_spawn_check: Instant::now(),
            pending_events: Vec::new(),
            pending_messages: Vec::new(),
            loot_lifetime: Duration::from_secs(60),
        }
    }

    /// Load content from a content registry.
    pub fn load_content(&mut self, registry: &plix_common::content::ContentRegistry) {
        // Load mob definitions
        for (id, def) in &registry.mobs {
            self.mob_defs.insert(id.clone(), def.clone());
        }

        // Load loot tables
        for (id, table) in &registry.loot_tables {
            self.loot_tables.insert(id.as_str().to_string(), table.clone());
        }

        // Load spawn points
        for (_, spawn) in &registry.spawns {
            self.spawn_manager.register_spawn_point(spawn.clone());
        }

        info!(
            mobs = self.mob_defs.len(),
            loot_tables = self.loot_tables.len(),
            "Mob system loaded content"
        );
    }

    /// Register a spawn point manually.
    pub fn register_spawn_point(&mut self, definition: SpawnPointDefinition) {
        self.spawn_manager.register_spawn_point(definition);
    }

    /// Register a mob definition.
    pub fn register_mob_def(&mut self, definition: MobDefinition) {
        self.mob_defs.insert(definition.mob_id.clone(), definition);
    }

    /// Register a loot table.
    pub fn register_loot_table(&mut self, table: LootTable) {
        self.loot_tables.insert(table.loot_table_id.as_str().to_string(), table);
    }

    /// Update the mob system for one tick.
    pub fn tick(&mut self, dt: f32, player_positions: &[(PlayerId, Vec3)], now: Instant) {
        // Process spawns periodically
        if now.duration_since(self.last_spawn_check).as_secs_f32() >= self.config.spawn_check_interval {
            self.process_spawns(now);
            self.last_spawn_check = now;
        }

        // Update all mobs
        let mob_ids: Vec<_> = self.mobs.keys().copied().collect();
        for mob_id in mob_ids {
            self.update_mob(mob_id, dt, player_positions, now);
        }

        // Clean up dead mobs
        self.cleanup_dead_mobs(now);

        // Clean up expired loot
        self.cleanup_expired_loot(now);
    }

    /// Process spawn points and create new mobs.
    fn process_spawns(&mut self, now: Instant) {
        let spawns = self.spawn_manager.process_spawns(now);

        for (spawn_id, mob_def_id, position, mob_id) in spawns {
            if let Some(definition) = self.mob_defs.get(&mob_def_id) {
                let instance = MobInstance::new(mob_id, definition, spawn_id.clone(), position);

                debug!(
                    mob_id = %mob_id,
                    def_id = %mob_def_id,
                    position = ?position,
                    "Spawned mob"
                );

                // Emit event
                self.pending_events.push(MobEvent::Spawned {
                    mob_id,
                    def_id: mob_def_id.clone(),
                    position,
                });

                // Broadcast spawn message
                self.pending_messages.push((
                    None,
                    ServerMessage::MobSpawned(MobSpawnPayload {
                        mob_id,
                        def_id: mob_def_id,
                        position,
                        hp: instance.current_hp,
                        max_hp: instance.max_hp,
                    }),
                ));

                self.mobs.insert(mob_id, instance);
            } else {
                warn!(def_id = %mob_def_id, "Unknown mob definition, skipping spawn");
            }
        }
    }

    /// Update a single mob's AI and movement.
    fn update_mob(&mut self, mob_id: MobInstanceId, dt: f32, player_positions: &[(PlayerId, Vec3)], now: Instant) {
        let mob = match self.mobs.get_mut(&mob_id) {
            Some(m) => m,
            None => return,
        };

        if mob.is_dead() || mob.pending_removal {
            return;
        }

        let definition = match self.mob_defs.get(&mob.def_id) {
            Some(d) => d,
            None => return,
        };

        // Build player info list
        let nearby_players: Vec<PlayerInfo> = player_positions
            .iter()
            .map(|(id, pos)| PlayerInfo {
                id: *id,
                position: *pos,
                distance: mob.distance_to(*pos),
            })
            .filter(|p| p.distance <= self.config.max_player_distance)
            .collect();

        // Update AI
        let can_attack = mob.can_attack(definition.stats.attack_cooldown);
        let (new_state, action) = update_ai(
            mob.ai_state,
            mob.position,
            mob.spawn_position,
            mob.target,
            &nearby_players,
            &definition.stats,
            &definition.behavior,
            can_attack,
        );

        mob.ai_state = new_state;
        mob.last_update = now;

        // Process AI action
        match action {
            AiAction::None => {}
            AiAction::MoveTo(target) => {
                mob.move_towards(target, definition.stats.speed, dt);
            }
            AiAction::Attack(target_id) => {
                mob.record_attack();
                // Attack is handled externally via apply_damage
                // Here we just track that an attack occurred
            }
            AiAction::AcquireTarget(player_id) => {
                mob.target = Some(player_id);
            }
            AiAction::LoseTarget => {
                mob.target = None;
            }
            AiAction::ReturnComplete => {
                mob.target = None;
                // Heal to full when returning to spawn
                mob.current_hp = mob.max_hp;
            }
        }

        // Broadcast position update if moving
        if mob.velocity.length_squared() > 0.01 {
            self.pending_messages.push((
                None,
                ServerMessage::MobUpdate(MobUpdatePayload {
                    mob_id,
                    position: mob.position,
                    hp: mob.current_hp,
                    state: format!("{:?}", mob.ai_state),
                    target: mob.target,
                }),
            ));
        }
    }

    /// Apply damage to a mob from a player.
    /// Returns true if the mob died.
    pub fn apply_damage(&mut self, mob_id: MobInstanceId, attacker: PlayerId, damage: u32, now: Instant) -> bool {
        let mob = match self.mobs.get_mut(&mob_id) {
            Some(m) => m,
            None => return false,
        };

        if mob.is_dead() {
            return false;
        }

        let died = mob.take_damage(attacker, damage);

        // Emit damage event
        self.pending_events.push(MobEvent::Damaged {
            mob_id,
            attacker,
            damage,
            remaining_hp: mob.current_hp,
        });

        // Broadcast damage message
        self.pending_messages.push((
            None,
            ServerMessage::MobDamaged(MobDamagedPayload {
                mob_id,
                attacker,
                damage,
                remaining_hp: mob.current_hp,
            }),
        ));

        if died {
            self.handle_mob_death(mob_id, now);
        }

        died
    }

    /// Handle a mob death (loot, payouts, cleanup).
    fn handle_mob_death(&mut self, mob_id: MobInstanceId, now: Instant) {
        let mob = match self.mobs.get(&mob_id) {
            Some(m) => m,
            None => return,
        };

        let definition = match self.mob_defs.get(&mob.def_id) {
            Some(d) => d.clone(),
            None => return,
        };

        let position = mob.position;
        let spawn_id = mob.spawn_id.clone();
        let def_id = mob.def_id.clone();
        let killer = mob.damage_tracker.killer();

        // Emit kill event
        self.pending_events.push(MobEvent::Killed {
            mob_id,
            def_id: def_id.clone(),
            killer,
            position,
        });

        // Calculate and distribute payouts
        let payouts = self.payout_calc.calculate(&definition, &mob.damage_tracker, now);

        for payout in &payouts.payouts {
            self.pending_messages.push((
                Some(payout.player_id),
                ServerMessage::RewardGranted(RewardPayload {
                    reward_type: RewardType::MobKill,
                    xp: payout.xp,
                    credits: payout.credits,
                    source: format!("{}", def_id),
                }),
            ));
        }

        self.pending_events.push(MobEvent::RewardDistributed {
            mob_id,
            payouts: payouts.clone(),
        });

        // Drop loot
        if let Some(loot_table) = self.loot_tables.get(definition.loot_table.as_str()) {
            let drops = loot_table.roll(None);

            if !drops.is_empty() {
                let mut drop_items = Vec::new();

                for (item_id, quantity) in &drops {
                    let loot_id = self.next_loot_id;
                    self.next_loot_id += 1;

                    let drop = LootDrop {
                        id: loot_id,
                        position,
                        item_id: item_id.clone(),
                        quantity: *quantity,
                        created_at: now,
                        expires_at: now + self.loot_lifetime,
                    };

                    drop_items.push((item_id.clone(), *quantity));
                    self.loot_drops.insert(loot_id, drop);
                }

                // Broadcast loot drop
                self.pending_messages.push((
                    None,
                    ServerMessage::MobLootDropped(MobLootDropPayload {
                        mob_id,
                        position,
                        items: drop_items.clone(),
                    }),
                ));

                self.pending_events.push(MobEvent::LootDropped {
                    mob_id,
                    position,
                    items: drop_items,
                });
            }
        }

        // Broadcast death message
        self.pending_messages.push((
            None,
            ServerMessage::MobDied(MobDiedPayload {
                mob_id,
                killer,
                position,
            }),
        ));

        // Notify spawn manager
        self.spawn_manager.notify_mob_died(&spawn_id, mob_id, now);

        // Mark for removal
        if let Some(mob) = self.mobs.get_mut(&mob_id) {
            mob.pending_removal = true;
        }
    }

    /// Try to pick up a loot drop.
    /// Returns Some((item_id, quantity)) if successful.
    pub fn pickup_loot(&mut self, player_id: PlayerId, loot_id: u64, player_pos: Vec3, max_distance: f32) -> Option<(ItemId, u32)> {
        let drop = self.loot_drops.get(&loot_id)?;

        // Check distance
        let dist = (drop.position - player_pos).length();
        if dist > max_distance {
            return None;
        }

        let item = (drop.item_id.clone(), drop.quantity);
        self.loot_drops.remove(&loot_id);

        Some(item)
    }

    /// Clean up dead mobs.
    fn cleanup_dead_mobs(&mut self, _now: Instant) {
        self.mobs.retain(|_, mob| !mob.pending_removal);
    }

    /// Clean up expired loot drops.
    fn cleanup_expired_loot(&mut self, now: Instant) {
        self.loot_drops.retain(|_, drop| now < drop.expires_at);
    }

    /// Take pending events (clears the internal buffer).
    pub fn take_events(&mut self) -> Vec<MobEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Take pending messages (clears the internal buffer).
    pub fn take_messages(&mut self) -> Vec<(Option<PlayerId>, ServerMessage)> {
        std::mem::take(&mut self.pending_messages)
    }

    /// Get a mob instance by ID.
    pub fn get_mob(&self, mob_id: MobInstanceId) -> Option<&MobInstance> {
        self.mobs.get(&mob_id)
    }

    /// Get all active mob instances.
    pub fn all_mobs(&self) -> impl Iterator<Item = (&MobInstanceId, &MobInstance)> {
        self.mobs.iter()
    }

    /// Get the total number of active mobs.
    pub fn active_mob_count(&self) -> usize {
        self.mobs.len()
    }

    /// Get the total number of loot drops.
    pub fn loot_drop_count(&self) -> usize {
        self.loot_drops.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::content::ids::{LootTableId, RegionId};
    use plix_common::content::loot::{LootEntry, LootTable};
    use plix_common::content::mob::{MobBehavior, MobStats};

    fn test_definition() -> MobDefinition {
        MobDefinition {
            mob_id: MobDefId::new("test_mob"),
            display_name: "Test Mob".to_string(),
            stats: MobStats {
                hp: 100,
                damage: 10,
                speed: 5.0,
                armor: 0,
                aggro_radius: 10.0,
                leash_radius: 25.0,
                attack_range: 2.0,
                attack_cooldown: 1.0,
            },
            behavior: MobBehavior::Aggro,
            loot_table: LootTableId::new("test_drops"),
            xp_reward: 100,
            credit_reward: 50,
            tags: vec![],
        }
    }

    fn test_loot_table() -> LootTable {
        LootTable {
            loot_table_id: LootTableId::new("test_drops"),
            guaranteed: vec![LootEntry {
                item_id: ItemId::new_raw(1),
                quantity: (1, 1),
                chance: 1.0,
            }],
            entries: vec![],
        }
    }

    fn test_spawn_point() -> SpawnPointDefinition {
        SpawnPointDefinition {
            spawn_id: SpawnId::new("test_spawn"),
            mob_id: MobDefId::new("test_mob"),
            count: 1,
            respawn_secs: 60.0,
            radius: 0.0,
            position: Vec3::new(100.0, 0.0, 100.0),
            region_id: RegionId::new("test_region"),
        }
    }

    #[test]
    fn test_spawn_mob() {
        let mut system = MobSystem::new(MobSystemConfig::default());
        system.register_mob_def(test_definition());
        system.register_loot_table(test_loot_table());
        system.register_spawn_point(test_spawn_point());

        let now = Instant::now();
        system.tick(0.016, &[], now);

        assert_eq!(system.active_mob_count(), 1);

        let events = system.take_events();
        assert!(events.iter().any(|e| matches!(e, MobEvent::Spawned { .. })));
    }

    #[test]
    fn test_damage_and_kill() {
        let mut system = MobSystem::new(MobSystemConfig::default());
        system.register_mob_def(test_definition());
        system.register_loot_table(test_loot_table());
        system.register_spawn_point(test_spawn_point());

        let now = Instant::now();
        system.tick(0.016, &[], now);

        let mob_id = *system.mobs.keys().next().unwrap();

        // Apply damage
        let died = system.apply_damage(mob_id, PlayerId(1), 50, now);
        assert!(!died);

        let events = system.take_events();
        assert!(events.iter().any(|e| matches!(e, MobEvent::Damaged { damage: 50, .. })));

        // Kill the mob
        let died = system.apply_damage(mob_id, PlayerId(1), 50, now);
        assert!(died);

        let events = system.take_events();
        assert!(events.iter().any(|e| matches!(e, MobEvent::Killed { .. })));
        assert!(events.iter().any(|e| matches!(e, MobEvent::RewardDistributed { .. })));
    }

    #[test]
    fn test_loot_pickup() {
        let mut system = MobSystem::new(MobSystemConfig::default());
        system.register_mob_def(test_definition());
        system.register_loot_table(test_loot_table());
        system.register_spawn_point(test_spawn_point());

        let now = Instant::now();
        system.tick(0.016, &[], now);

        let mob_id = *system.mobs.keys().next().unwrap();
        let mob_pos = system.mobs.get(&mob_id).unwrap().position;

        // Kill the mob
        system.apply_damage(mob_id, PlayerId(1), 100, now);
        system.tick(0.016, &[], now); // Cleanup

        // Should have loot drops
        assert!(system.loot_drop_count() > 0);

        // Pick up loot
        let loot_id = *system.loot_drops.keys().next().unwrap();
        let result = system.pickup_loot(PlayerId(1), loot_id, mob_pos, 5.0);
        assert!(result.is_some());
        assert_eq!(system.loot_drop_count(), 0);
    }
}
