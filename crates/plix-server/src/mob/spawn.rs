//! Spawn point management for mobs
//!
//! Handles:
//! - Processing spawn point definitions
//! - Tracking spawn cooldowns
//! - Creating mob instances at spawn points
//! - Respawning mobs after death

use std::collections::HashMap;
use std::time::{Duration, Instant};

use rand::prelude::*;

use plix_common::content::ids::{MobDefId, MobInstanceId, RegionId, SpawnId};
use plix_common::content::mob::MobDefinition;
use plix_common::content::spawn::SpawnPointDefinition;
use plix_common::math::Vec3;

use super::instance::MobInstance;

/// Tracks the state of a spawn point.
#[derive(Debug)]
struct SpawnPointState {
    /// The spawn point definition
    definition: SpawnPointDefinition,
    /// Currently spawned mob IDs from this point
    active_mobs: Vec<MobInstanceId>,
    /// Time when respawn becomes available (per slot)
    respawn_timers: Vec<Option<Instant>>,
}

/// Manages mob spawn points and respawn timers.
pub struct SpawnManager {
    /// Spawn point states keyed by spawn_id
    spawn_points: HashMap<SpawnId, SpawnPointState>,
    /// Region mob counts (for limits)
    region_counts: HashMap<RegionId, u32>,
    /// Per-region mob limits
    region_limits: HashMap<RegionId, u32>,
    /// Next mob instance ID
    next_mob_id: u64,
    /// RNG for position randomization
    rng: StdRng,
}

impl SpawnManager {
    /// Create a new spawn manager.
    pub fn new() -> Self {
        Self {
            spawn_points: HashMap::new(),
            region_counts: HashMap::new(),
            region_limits: HashMap::new(),
            next_mob_id: 1,
            rng: StdRng::from_entropy(),
        }
    }

    /// Create with a specific RNG seed (for testing).
    pub fn with_seed(seed: u64) -> Self {
        Self {
            spawn_points: HashMap::new(),
            region_counts: HashMap::new(),
            region_limits: HashMap::new(),
            next_mob_id: 1,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Register a spawn point.
    pub fn register_spawn_point(&mut self, definition: SpawnPointDefinition) {
        let count = definition.count as usize;
        let spawn_id = definition.spawn_id.clone();

        let state = SpawnPointState {
            definition,
            active_mobs: Vec::with_capacity(count),
            respawn_timers: vec![None; count],
        };

        self.spawn_points.insert(spawn_id, state);
    }

    /// Set the mob limit for a region.
    pub fn set_region_limit(&mut self, region_id: RegionId, limit: u32) {
        self.region_limits.insert(region_id, limit);
    }

    /// Generate a new unique mob instance ID.
    fn next_id(&mut self) -> MobInstanceId {
        let id = MobInstanceId::new(self.next_mob_id);
        self.next_mob_id += 1;
        id
    }

    /// Generate a random position within the spawn radius.
    fn random_position(&mut self, center: Vec3, radius: f32) -> Vec3 {
        if radius <= 0.0 {
            return center;
        }

        let angle = self.rng.gen::<f32>() * std::f32::consts::TAU;
        let dist = self.rng.gen::<f32>().sqrt() * radius;

        Vec3::new(
            center.x + dist * angle.cos(),
            center.y,
            center.z + dist * angle.sin(),
        )
    }

    /// Check if a region has room for more mobs.
    fn region_has_room(&self, region_id: &RegionId) -> bool {
        let limit = self.region_limits.get(region_id).copied().unwrap_or(u32::MAX);
        let count = self.region_counts.get(region_id).copied().unwrap_or(0);
        count < limit
    }

    /// Process all spawn points and return new mobs to spawn.
    ///
    /// Returns a list of (spawn_id, mob_def_id, spawn_position) for each mob to create.
    pub fn process_spawns(&mut self, now: Instant) -> Vec<(SpawnId, MobDefId, Vec3, MobInstanceId)> {
        let mut to_spawn = Vec::new();

        for (spawn_id, state) in &mut self.spawn_points {
            // Check region limit
            if !self.region_has_room(&state.definition.region_id) {
                continue;
            }

            // Find empty slots that are ready to spawn
            for slot in 0..state.definition.count as usize {
                // Skip if slot is occupied
                if slot < state.active_mobs.len() && state.active_mobs.len() > slot {
                    // Check if this slot's mob is still active
                    // (This is handled externally via notify_mob_died)
                    continue;
                }

                // Check respawn timer
                if let Some(timer) = state.respawn_timers.get(slot).copied().flatten() {
                    if now < timer {
                        continue; // Not ready yet
                    }
                }

                // Spawn a new mob
                let mob_id = MobInstanceId::new(self.next_mob_id);
                self.next_mob_id += 1;

                let position = self.random_position(
                    state.definition.position,
                    state.definition.radius,
                );

                to_spawn.push((
                    spawn_id.clone(),
                    state.definition.mob_id.clone(),
                    position,
                    mob_id,
                ));

                // Track the new mob
                if slot < state.active_mobs.len() {
                    state.active_mobs[slot] = mob_id;
                } else {
                    state.active_mobs.push(mob_id);
                }

                // Clear respawn timer
                if slot < state.respawn_timers.len() {
                    state.respawn_timers[slot] = None;
                }

                // Update region count
                *self.region_counts.entry(state.definition.region_id.clone()).or_insert(0) += 1;
            }
        }

        to_spawn
    }

    /// Notify that a mob has died and should be respawned.
    pub fn notify_mob_died(&mut self, spawn_id: &SpawnId, mob_id: MobInstanceId, now: Instant) {
        if let Some(state) = self.spawn_points.get_mut(spawn_id) {
            // Find and remove the mob from active list
            if let Some(pos) = state.active_mobs.iter().position(|&id| id == mob_id) {
                state.active_mobs.remove(pos);

                // Set respawn timer if the spawn point has respawning enabled
                if state.definition.has_respawn() {
                    let respawn_time = now + Duration::from_secs_f32(state.definition.respawn_secs);
                    if pos < state.respawn_timers.len() {
                        state.respawn_timers[pos] = Some(respawn_time);
                    } else {
                        state.respawn_timers.push(Some(respawn_time));
                    }
                }

                // Update region count
                if let Some(count) = self.region_counts.get_mut(&state.definition.region_id) {
                    *count = count.saturating_sub(1);
                }
            }
        }
    }

    /// Get the number of active mobs from a spawn point.
    pub fn active_count(&self, spawn_id: &SpawnId) -> usize {
        self.spawn_points
            .get(spawn_id)
            .map(|s| s.active_mobs.len())
            .unwrap_or(0)
    }

    /// Get the total number of active mobs.
    pub fn total_active(&self) -> usize {
        self.spawn_points.values().map(|s| s.active_mobs.len()).sum()
    }

    /// Get the mob count in a region.
    pub fn region_count(&self, region_id: &RegionId) -> u32 {
        self.region_counts.get(region_id).copied().unwrap_or(0)
    }
}

impl Default for SpawnManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_spawn_point(id: &str, count: u32, respawn_secs: f32) -> SpawnPointDefinition {
        SpawnPointDefinition {
            spawn_id: SpawnId::new(id),
            mob_id: MobDefId::new("test_mob"),
            count,
            respawn_secs,
            radius: 5.0,
            position: Vec3::new(100.0, 0.0, 100.0),
            region_id: RegionId::new("test_region"),
        }
    }

    #[test]
    fn test_initial_spawn() {
        let mut manager = SpawnManager::with_seed(12345);
        manager.register_spawn_point(test_spawn_point("sp1", 3, 60.0));

        let spawns = manager.process_spawns(Instant::now());
        assert_eq!(spawns.len(), 3);
        assert_eq!(manager.active_count(&SpawnId::new("sp1")), 3);
    }

    #[test]
    fn test_respawn_timer() {
        let mut manager = SpawnManager::with_seed(12345);
        manager.register_spawn_point(test_spawn_point("sp1", 1, 10.0));

        let now = Instant::now();
        let spawns = manager.process_spawns(now);
        assert_eq!(spawns.len(), 1);

        let mob_id = spawns[0].3;
        manager.notify_mob_died(&SpawnId::new("sp1"), mob_id, now);
        assert_eq!(manager.active_count(&SpawnId::new("sp1")), 0);

        // Try to spawn immediately - should not spawn yet
        let spawns = manager.process_spawns(now);
        assert_eq!(spawns.len(), 0);

        // After respawn time
        let later = now + Duration::from_secs(15);
        let spawns = manager.process_spawns(later);
        assert_eq!(spawns.len(), 1);
    }

    #[test]
    fn test_no_respawn() {
        let mut manager = SpawnManager::with_seed(12345);
        manager.register_spawn_point(test_spawn_point("sp1", 1, 0.0)); // No respawn

        let now = Instant::now();
        let spawns = manager.process_spawns(now);
        let mob_id = spawns[0].3;

        manager.notify_mob_died(&SpawnId::new("sp1"), mob_id, now);

        // Should not respawn
        let later = now + Duration::from_secs(100);
        let spawns = manager.process_spawns(later);
        assert_eq!(spawns.len(), 0);
    }

    #[test]
    fn test_region_limit() {
        let mut manager = SpawnManager::with_seed(12345);

        let sp1 = SpawnPointDefinition {
            spawn_id: SpawnId::new("sp1"),
            mob_id: MobDefId::new("test_mob"),
            count: 5,
            respawn_secs: 60.0,
            radius: 0.0,
            position: Vec3::ZERO,
            region_id: RegionId::new("limited_region"),
        };

        manager.register_spawn_point(sp1);
        manager.set_region_limit(RegionId::new("limited_region"), 3);

        let spawns = manager.process_spawns(Instant::now());
        // Should only spawn 3 due to region limit
        assert_eq!(spawns.len(), 3);
        assert_eq!(manager.region_count(&RegionId::new("limited_region")), 3);
    }

    #[test]
    fn test_unique_ids() {
        let mut manager = SpawnManager::with_seed(12345);
        manager.register_spawn_point(test_spawn_point("sp1", 3, 60.0));

        let spawns = manager.process_spawns(Instant::now());
        let ids: Vec<_> = spawns.iter().map(|s| s.3).collect();

        // All IDs should be unique
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                assert_ne!(ids[i], ids[j]);
            }
        }
    }
}
