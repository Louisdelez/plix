//! Projectile management and collision detection.
//!
//! Manages active projectiles, movement simulation, and collision detection.

use glam::Vec3;
use plix_common::time::Tick;
use plix_common::types::{BlockPos, PlayerId, ProjectileId};

use super::defs::WeaponDef;

/// Maximum number of active projectiles (server limit).
pub const MAX_PROJECTILES: usize = 128;

/// Active projectile state.
#[derive(Debug, Clone)]
pub struct Projectile {
    /// Unique identifier
    pub id: ProjectileId,
    /// Player who fired this projectile
    pub owner: PlayerId,
    /// Current position in world space
    pub position: Vec3,
    /// Velocity in blocks per tick
    pub velocity: Vec3,
    /// Damage dealt on impact
    pub damage: u16,
    /// Tick when projectile expires
    pub expires_at: Tick,
    /// Whether projectile is still active
    pub active: bool,
}

/// What a projectile hit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectileImpactType {
    /// Hit a player
    Player(PlayerId),
    /// Hit a solid block
    Block(BlockPos),
}

/// Result of projectile impact.
#[derive(Debug, Clone)]
pub struct ProjectileImpact {
    /// Projectile that impacted
    pub id: ProjectileId,
    /// Player who fired the projectile
    pub owner: PlayerId,
    /// Position of impact
    pub position: Vec3,
    /// What was hit
    pub impact_type: ProjectileImpactType,
    /// Damage dealt
    pub damage: u16,
}

/// Reason a projectile was removed without impact.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectileDespawnReason {
    /// Lifetime expired
    Timeout,
    /// Server projectile limit exceeded
    LimitPurge,
    /// Owner disconnected
    OwnerLeft,
}

/// Target information for projectile collision.
#[derive(Debug, Clone, Copy)]
pub struct ProjectileTarget {
    /// Target player ID
    pub id: PlayerId,
    /// Target position (center mass)
    pub position: Vec3,
    /// Target capsule radius
    pub radius: f32,
    /// Target capsule half-height (from center)
    pub half_height: f32,
}

/// Callback trait for world block collision checks.
pub trait WorldCollision {
    /// Check if a block position is solid.
    fn is_solid(&self, pos: BlockPos) -> bool;
}

/// Manages all active projectiles.
#[derive(Debug)]
pub struct ProjectileManager {
    /// Pool of projectile slots
    projectiles: Vec<Option<Projectile>>,
    /// Generation counter for slot reuse
    generations: Vec<u16>,
    /// Number of currently active projectiles
    active_count: usize,
}

impl Default for ProjectileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectileManager {
    /// Create a new projectile manager.
    pub fn new() -> Self {
        Self {
            projectiles: vec![None; MAX_PROJECTILES],
            generations: vec![0; MAX_PROJECTILES],
            active_count: 0,
        }
    }

    /// Check if the projectile pool is full.
    pub fn is_full(&self) -> bool {
        self.active_count >= MAX_PROJECTILES
    }

    /// Get the current number of active projectiles.
    pub fn active_count(&self) -> usize {
        self.active_count
    }

    /// Spawn a new projectile.
    ///
    /// Returns the projectile ID if successful, None if pool is full.
    pub fn spawn(
        &mut self,
        owner: PlayerId,
        position: Vec3,
        velocity: Vec3,
        damage: u16,
        ttl_ticks: u32,
        current_tick: Tick,
    ) -> Option<ProjectileId> {
        if self.is_full() {
            return None;
        }

        // Find a free slot
        let slot_index = self.projectiles.iter().position(|p| p.is_none())?;

        // Increment generation for this slot
        self.generations[slot_index] = self.generations[slot_index].wrapping_add(1);

        let id = ProjectileId {
            index: slot_index as u16,
            generation: self.generations[slot_index],
        };

        let projectile = Projectile {
            id,
            owner,
            position,
            velocity,
            damage,
            expires_at: Tick(current_tick.0.saturating_add(ttl_ticks)),
            active: true,
        };

        self.projectiles[slot_index] = Some(projectile);
        self.active_count += 1;

        Some(id)
    }

    /// Spawn a projectile using weapon definition.
    pub fn spawn_from_weapon(
        &mut self,
        owner: PlayerId,
        position: Vec3,
        direction: Vec3,
        weapon: &WeaponDef,
        current_tick: Tick,
    ) -> Option<ProjectileId> {
        let velocity = direction * (weapon.projectile_speed / 60.0); // Convert to blocks/tick
        self.spawn(
            owner,
            position,
            velocity,
            weapon.damage,
            weapon.projectile_ttl_ticks,
            current_tick,
        )
    }

    /// Remove a projectile by ID.
    pub fn remove(&mut self, id: ProjectileId) -> bool {
        let slot = id.index as usize;
        if slot >= self.projectiles.len() {
            return false;
        }

        if let Some(ref projectile) = self.projectiles[slot] {
            if projectile.id == id {
                self.projectiles[slot] = None;
                self.active_count = self.active_count.saturating_sub(1);
                return true;
            }
        }

        false
    }

    /// Remove all projectiles owned by a player.
    pub fn remove_by_owner(&mut self, owner: PlayerId) -> Vec<ProjectileId> {
        let mut removed = Vec::new();

        for slot in &mut self.projectiles {
            if let Some(ref projectile) = slot {
                if projectile.owner == owner {
                    removed.push(projectile.id);
                    *slot = None;
                    self.active_count = self.active_count.saturating_sub(1);
                }
            }
        }

        removed
    }

    /// Tick all projectiles, moving them and checking for collisions.
    ///
    /// Returns lists of impacts and despawns that occurred this tick.
    pub fn tick<W: WorldCollision>(
        &mut self,
        targets: &[ProjectileTarget],
        world: &W,
        current_tick: Tick,
    ) -> (
        Vec<ProjectileImpact>,
        Vec<(ProjectileId, ProjectileDespawnReason)>,
    ) {
        let mut impacts = Vec::new();
        let mut despawns = Vec::new();

        for slot in &mut self.projectiles {
            let projectile = match slot {
                Some(ref mut p) if p.active => p,
                _ => continue,
            };

            // Check expiration
            if current_tick.0 >= projectile.expires_at.0 {
                despawns.push((projectile.id, ProjectileDespawnReason::Timeout));
                *slot = None;
                self.active_count = self.active_count.saturating_sub(1);
                continue;
            }

            // Move projectile
            let old_pos = projectile.position;
            let new_pos = projectile.position + projectile.velocity;

            // Check for block collision along path (discrete stepping)
            if let Some(block_pos) = Self::check_block_collision(old_pos, new_pos, world) {
                let impact_pos = Self::find_impact_point(old_pos, new_pos, block_pos);
                impacts.push(ProjectileImpact {
                    id: projectile.id,
                    owner: projectile.owner,
                    position: impact_pos,
                    impact_type: ProjectileImpactType::Block(block_pos),
                    damage: projectile.damage,
                });
                *slot = None;
                self.active_count = self.active_count.saturating_sub(1);
                continue;
            }

            // Check for player collision
            if let Some(target) =
                Self::check_player_collision(old_pos, new_pos, projectile.owner, targets)
            {
                impacts.push(ProjectileImpact {
                    id: projectile.id,
                    owner: projectile.owner,
                    position: new_pos,
                    impact_type: ProjectileImpactType::Player(target.id),
                    damage: projectile.damage,
                });
                *slot = None;
                self.active_count = self.active_count.saturating_sub(1);
                continue;
            }

            // Update position
            projectile.position = new_pos;
        }

        (impacts, despawns)
    }

    /// Get iterator over all active projectiles.
    pub fn iter(&self) -> impl Iterator<Item = &Projectile> {
        self.projectiles.iter().filter_map(|p| p.as_ref())
    }

    /// Check for block collision along projectile path.
    fn check_block_collision<W: WorldCollision>(
        start: Vec3,
        end: Vec3,
        world: &W,
    ) -> Option<BlockPos> {
        // Use discrete stepping along the path
        let delta = end - start;
        let distance = delta.length();

        if distance < 0.001 {
            let block_pos = BlockPos::from_vec3(end);
            if world.is_solid(block_pos) {
                return Some(block_pos);
            }
            return None;
        }

        let step_size = 0.25; // Check every 0.25 blocks
        let steps = (distance / step_size).ceil() as usize;
        let step_delta = delta / steps as f32;

        let mut pos = start;
        for _ in 0..=steps {
            let block_pos = BlockPos::from_vec3(pos);
            if world.is_solid(block_pos) {
                return Some(block_pos);
            }
            pos += step_delta;
        }

        None
    }

    /// Check for player collision along projectile path.
    fn check_player_collision(
        start: Vec3,
        end: Vec3,
        owner: PlayerId,
        targets: &[ProjectileTarget],
    ) -> Option<ProjectileTarget> {
        for target in targets {
            // Don't hit owner
            if target.id == owner {
                continue;
            }

            // Sphere-vs-line segment collision (simplified)
            if Self::line_intersects_capsule(start, end, target) {
                return Some(*target);
            }
        }
        None
    }

    /// Check if a line segment intersects a capsule.
    fn line_intersects_capsule(start: Vec3, end: Vec3, target: &ProjectileTarget) -> bool {
        // Use closest point on line segment to capsule axis
        let capsule_bottom = target.position - Vec3::Y * target.half_height;
        let capsule_top = target.position + Vec3::Y * target.half_height;

        // Find closest points on line segment and capsule axis
        let (_, closest_on_capsule) =
            Self::closest_points_between_segments(start, end, capsule_bottom, capsule_top);

        // Check distance to projectile path
        let closest_on_line = Self::closest_point_on_segment(start, end, closest_on_capsule);
        let distance = (closest_on_line - closest_on_capsule).length();

        distance <= target.radius
    }

    /// Find closest point on line segment to a point.
    fn closest_point_on_segment(start: Vec3, end: Vec3, point: Vec3) -> Vec3 {
        let line = end - start;
        let len_sq = line.length_squared();

        if len_sq < 0.0001 {
            return start;
        }

        let t = ((point - start).dot(line) / len_sq).clamp(0.0, 1.0);
        start + line * t
    }

    /// Find closest points between two line segments.
    fn closest_points_between_segments(
        a_start: Vec3,
        a_end: Vec3,
        b_start: Vec3,
        b_end: Vec3,
    ) -> (Vec3, Vec3) {
        let d1 = a_end - a_start;
        let d2 = b_end - b_start;
        let r = a_start - b_start;

        let a = d1.dot(d1);
        let e = d2.dot(d2);
        let f = d2.dot(r);

        let (s, t) = if a < 0.0001 && e < 0.0001 {
            (0.0, 0.0)
        } else if a < 0.0001 {
            (0.0, (f / e).clamp(0.0, 1.0))
        } else {
            let c = d1.dot(r);
            if e < 0.0001 {
                ((-c / a).clamp(0.0, 1.0), 0.0)
            } else {
                let b = d1.dot(d2);
                let denom = a * e - b * b;

                let s = if denom.abs() > 0.0001 {
                    ((b * f - c * e) / denom).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                let t = (b * s + f) / e;
                let t = t.clamp(0.0, 1.0);

                let s = if t < 0.0 {
                    (-c / a).clamp(0.0, 1.0)
                } else if t > 1.0 {
                    ((b - c) / a).clamp(0.0, 1.0)
                } else {
                    s
                };

                (s, t)
            }
        };

        (a_start + d1 * s, b_start + d2 * t)
    }

    /// Find the exact impact point on a block.
    fn find_impact_point(_start: Vec3, end: Vec3, _block_pos: BlockPos) -> Vec3 {
        // For simplicity, use the end position as impact point
        // A more accurate implementation would ray-march to find exact intersection
        end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWorld {
        solid_blocks: std::collections::HashSet<BlockPos>,
    }

    impl WorldCollision for TestWorld {
        fn is_solid(&self, pos: BlockPos) -> bool {
            self.solid_blocks.contains(&pos)
        }
    }

    #[test]
    fn test_projectile_manager_spawn() {
        let mut mgr = ProjectileManager::new();

        let id = mgr
            .spawn(
                PlayerId(1),
                Vec3::ZERO,
                Vec3::new(0.0, 0.0, 0.5),
                15,
                180,
                Tick(0),
            )
            .unwrap();

        assert!(id.is_valid());
        assert_eq!(mgr.active_count(), 1);
        assert!(!mgr.is_full());
    }

    #[test]
    fn test_projectile_manager_limit() {
        let mut mgr = ProjectileManager::new();

        // Fill to capacity
        for i in 0..MAX_PROJECTILES {
            let result = mgr.spawn(
                PlayerId(1),
                Vec3::ZERO,
                Vec3::new(0.0, 0.0, 0.5),
                15,
                180,
                Tick(0),
            );
            assert!(result.is_some(), "Failed to spawn projectile {}", i);
        }

        assert!(mgr.is_full());
        assert_eq!(mgr.active_count(), MAX_PROJECTILES);

        // Should fail to spawn more
        let result = mgr.spawn(
            PlayerId(1),
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 0.5),
            15,
            180,
            Tick(0),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_projectile_expiration() {
        let mut mgr = ProjectileManager::new();
        let world = TestWorld {
            solid_blocks: std::collections::HashSet::new(),
        };

        mgr.spawn(
            PlayerId(1),
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 0.5),
            15,
            10, // 10 tick lifetime
            Tick(0),
        );

        assert_eq!(mgr.active_count(), 1);

        // Tick until expiration
        for tick in 1..=10 {
            let (impacts, despawns) = mgr.tick(&[], &world, Tick(tick));
            if tick < 10 {
                assert!(despawns.is_empty());
            } else {
                assert_eq!(despawns.len(), 1);
                assert_eq!(despawns[0].1, ProjectileDespawnReason::Timeout);
            }
        }

        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn test_projectile_block_collision() {
        let mut mgr = ProjectileManager::new();
        let mut world = TestWorld {
            solid_blocks: std::collections::HashSet::new(),
        };

        // Add a solid block at z=5
        world.solid_blocks.insert(BlockPos::new(0, 0, 5));

        // Spawn projectile moving toward the block
        mgr.spawn(
            PlayerId(1),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(0.0, 0.0, 1.0), // 1 block/tick
            15,
            180,
            Tick(0),
        );

        // Tick until collision
        let mut hit_block = false;
        for tick in 1..=10 {
            let (impacts, _) = mgr.tick(&[], &world, Tick(tick));
            if !impacts.is_empty() {
                hit_block = true;
                assert!(matches!(
                    impacts[0].impact_type,
                    ProjectileImpactType::Block(_)
                ));
                break;
            }
        }

        assert!(hit_block);
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn test_projectile_player_collision() {
        let mut mgr = ProjectileManager::new();
        let world = TestWorld {
            solid_blocks: std::collections::HashSet::new(),
        };

        // Spawn projectile moving toward player 2
        mgr.spawn(
            PlayerId(1),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0), // 1 block/tick
            15,
            180,
            Tick(0),
        );

        let targets = vec![ProjectileTarget {
            id: PlayerId(2),
            position: Vec3::new(0.0, 1.0, 3.0),
            radius: 0.4,
            half_height: 0.9,
        }];

        // Tick until collision
        let mut hit_player = false;
        for tick in 1..=10 {
            let (impacts, _) = mgr.tick(&targets, &world, Tick(tick));
            if !impacts.is_empty() {
                hit_player = true;
                assert!(matches!(
                    impacts[0].impact_type,
                    ProjectileImpactType::Player(PlayerId(2))
                ));
                assert_eq!(impacts[0].damage, 15);
                break;
            }
        }

        assert!(hit_player);
    }

    #[test]
    fn test_projectile_no_self_hit() {
        let mut mgr = ProjectileManager::new();
        let world = TestWorld {
            solid_blocks: std::collections::HashSet::new(),
        };

        // Spawn projectile starting inside player 1 (owner)
        mgr.spawn(
            PlayerId(1),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 0.5),
            15,
            10,
            Tick(0),
        );

        let targets = vec![ProjectileTarget {
            id: PlayerId(1), // Same as owner
            position: Vec3::new(0.0, 1.0, 0.0),
            radius: 0.4,
            half_height: 0.9,
        }];

        // Tick and verify no self-hit
        for tick in 1..=10 {
            let (impacts, _) = mgr.tick(&targets, &world, Tick(tick));
            for impact in impacts {
                if let ProjectileImpactType::Player(id) = impact.impact_type {
                    assert_ne!(id, PlayerId(1), "Projectile hit its owner!");
                }
            }
        }
    }

    #[test]
    fn test_remove_by_owner() {
        let mut mgr = ProjectileManager::new();

        // Spawn projectiles for different owners
        for _ in 0..5 {
            mgr.spawn(PlayerId(1), Vec3::ZERO, Vec3::Z, 15, 180, Tick(0));
        }
        for _ in 0..3 {
            mgr.spawn(PlayerId(2), Vec3::ZERO, Vec3::Z, 15, 180, Tick(0));
        }

        assert_eq!(mgr.active_count(), 8);

        // Remove player 1's projectiles
        let removed = mgr.remove_by_owner(PlayerId(1));
        assert_eq!(removed.len(), 5);
        assert_eq!(mgr.active_count(), 3);
    }

    #[test]
    fn test_generation_increments() {
        let mut mgr = ProjectileManager::new();

        // Spawn and remove, then spawn again in same slot
        let id1 = mgr
            .spawn(PlayerId(1), Vec3::ZERO, Vec3::Z, 15, 180, Tick(0))
            .unwrap();
        mgr.remove(id1);

        let id2 = mgr
            .spawn(PlayerId(1), Vec3::ZERO, Vec3::Z, 15, 180, Tick(0))
            .unwrap();

        // Should be same slot, different generation
        assert_eq!(id1.index, id2.index);
        assert_ne!(id1.generation, id2.generation);
    }
}
