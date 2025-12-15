//! Collision detection and resolution
//!
//! Implements capsule-equivalent AABB collision with voxel geometry.
//! Resolution order: Y → X → Z (vertical first for proper ground detection).

use plix_common::math::{Vec3, AABB};
use plix_common::types::BlockType;

/// Player collision box half-extents (aligned with physics.rs clarified values)
pub const PLAYER_HALF_WIDTH: f32 = 0.4; // radius (was 0.3)
pub const PLAYER_HALF_HEIGHT: f32 = 0.9; // 1.8m / 2

/// Epsilon for floating-point comparisons
pub const COLLISION_EPSILON: f32 = 0.001;

/// Ground detection probe distance (2cm)
pub const GROUND_PROBE_DISTANCE: f32 = 0.02;

/// Step-up maximum height (1 block for voxel terrain)
pub const STEP_HEIGHT: f32 = 1.0;

/// Maximum movement per collision step (for tunneling prevention)
pub const MAX_MOVE_PER_STEP: f32 = 0.5;

/// Result of collision resolution
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionResult {
    /// Final resolved position
    pub position: Vec3,
    /// Final velocity (may be zeroed on collision axes)
    pub velocity: Vec3,
    /// Whether player is on ground after resolution
    pub grounded: bool,
    /// Whether step-up occurred during this frame
    pub stepped: bool,
    /// Collision normal (if any collision occurred)
    pub hit_normal: Option<Vec3>,
}

impl Default for CollisionResult {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            grounded: false,
            stepped: false,
            hit_normal: None,
        }
    }
}

/// Collision world for arena
#[derive(Debug)]
pub struct CollisionWorld {
    /// Arena size
    size: [u32; 3],
    /// Block data
    blocks: Vec<BlockType>,
}

impl CollisionWorld {
    /// Create from arena data
    pub fn new(size: [u32; 3], blocks: Vec<BlockType>) -> Self {
        Self { size, blocks }
    }

    /// Get arena size
    pub fn size(&self) -> [u32; 3] {
        self.size
    }

    /// Get block at position
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> BlockType {
        if x < 0 || y < 0 || z < 0 {
            return BlockType::AIR;
        }

        let [sx, sy, sz] = self.size;
        let (ux, uy, uz) = (x as u32, y as u32, z as u32);

        if ux >= sx || uy >= sy || uz >= sz {
            return BlockType::AIR;
        }

        let index = (uz * sy * sx + uy * sx + ux) as usize;
        self.blocks.get(index).copied().unwrap_or(BlockType::AIR)
    }

    /// Check if position is solid
    pub fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
        self.get_block(x, y, z).is_solid()
    }

    /// Get player AABB at position (feet position)
    pub fn player_aabb(position: Vec3) -> AABB {
        AABB {
            min: Vec3::new(
                position.x - PLAYER_HALF_WIDTH,
                position.y,
                position.z - PLAYER_HALF_WIDTH,
            ),
            max: Vec3::new(
                position.x + PLAYER_HALF_WIDTH,
                position.y + PLAYER_HALF_HEIGHT * 2.0,
                position.z + PLAYER_HALF_WIDTH,
            ),
        }
    }

    /// Get player AABB with custom half-width and height
    pub fn player_aabb_custom(position: Vec3, half_width: f32, height: f32) -> AABB {
        AABB {
            min: Vec3::new(position.x - half_width, position.y, position.z - half_width),
            max: Vec3::new(
                position.x + half_width,
                position.y + height,
                position.z + half_width,
            ),
        }
    }

    /// Check if player AABB collides with any solid blocks
    pub fn check_collision(&self, position: Vec3) -> bool {
        let aabb = Self::player_aabb(position);
        self.check_aabb_collision(&aabb)
    }

    /// Check if an AABB collides with any solid blocks
    pub fn check_aabb_collision(&self, aabb: &AABB) -> bool {
        let min_x = aabb.min.x.floor() as i32;
        let min_y = aabb.min.y.floor() as i32;
        let min_z = aabb.min.z.floor() as i32;
        let max_x = aabb.max.x.ceil() as i32;
        let max_y = aabb.max.y.ceil() as i32;
        let max_z = aabb.max.z.ceil() as i32;

        for z in min_z..=max_z {
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if self.is_solid(x, y, z) {
                        // Check precise AABB intersection
                        let block_aabb = AABB {
                            min: Vec3::new(x as f32, y as f32, z as f32),
                            max: Vec3::new((x + 1) as f32, (y + 1) as f32, (z + 1) as f32),
                        };
                        if aabb.intersects(&block_aabb) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Check if a position is valid (not inside any solid blocks)
    pub fn is_position_valid(&self, position: Vec3) -> bool {
        !self.check_collision(position)
    }

    /// Check if player is on ground (using probe distance)
    pub fn is_grounded(&self, position: Vec3) -> bool {
        // Check slightly below feet
        let check_pos = Vec3::new(position.x, position.y - GROUND_PROBE_DISTANCE, position.z);
        self.check_collision(check_pos)
    }

    /// Legacy move and slide collision resolution (X → Y → Z order)
    /// Kept for backward compatibility
    pub fn move_and_slide(&self, position: Vec3, velocity: Vec3, dt: f32) -> (Vec3, Vec3, bool) {
        let result = self.move_and_slide_v2(position, velocity, dt, false);
        (result.position, result.velocity, result.grounded)
    }

    /// New move and slide collision resolution with Y → X → Z order
    /// This is the correct order for proper ground detection
    pub fn move_and_slide_v2(
        &self,
        position: Vec3,
        velocity: Vec3,
        dt: f32,
        allow_step_up: bool,
    ) -> CollisionResult {
        // Calculate total movement
        let total_move = velocity * dt;
        let move_length = total_move.length();

        // Subdivide movement if too fast (tunneling prevention)
        let steps = if move_length > MAX_MOVE_PER_STEP {
            (move_length / MAX_MOVE_PER_STEP).ceil() as u32
        } else {
            1
        };

        let sub_dt = dt / steps as f32;
        let mut current_pos = position;
        let mut current_vel = velocity;
        let mut grounded = false;
        let mut stepped = false;
        let mut hit_normal: Option<Vec3> = None;

        for _ in 0..steps {
            let step_result = self.resolve_single_step(
                current_pos,
                current_vel,
                sub_dt,
                allow_step_up && !stepped, // Only allow one step-up per frame
            );

            current_pos = step_result.position;
            current_vel = step_result.velocity;

            if step_result.grounded {
                grounded = true;
            }
            if step_result.stepped {
                stepped = true;
            }
            if step_result.hit_normal.is_some() {
                hit_normal = step_result.hit_normal;
            }
        }

        // Clamp to epsilon to prevent floating-point drift
        current_pos = self.clamp_position_epsilon(current_pos);
        current_vel = self.clamp_velocity_epsilon(current_vel);

        CollisionResult {
            position: current_pos,
            velocity: current_vel,
            grounded,
            stepped,
            hit_normal,
        }
    }

    /// Resolve a single collision step (internal)
    fn resolve_single_step(
        &self,
        position: Vec3,
        velocity: Vec3,
        dt: f32,
        allow_step_up: bool,
    ) -> CollisionResult {
        let mut new_pos = position;
        let mut new_vel = velocity;
        let mut grounded = false;
        let mut stepped = false;
        let mut hit_normal: Option<Vec3> = None;

        // Phase 1: Y-axis resolution (FIRST - for proper ground detection)
        let y_delta = velocity.y * dt;
        if y_delta.abs() > COLLISION_EPSILON {
            let test_y = Vec3::new(new_pos.x, position.y + y_delta, new_pos.z);
            if !self.check_collision(test_y) {
                new_pos.y = test_y.y;
            } else {
                // Collision on Y axis
                if velocity.y < 0.0 {
                    grounded = true;
                    hit_normal = Some(Vec3::new(0.0, 1.0, 0.0));
                } else {
                    hit_normal = Some(Vec3::new(0.0, -1.0, 0.0));
                }
                new_vel.y = 0.0;
            }
        }

        // Phase 2: X-axis resolution
        let x_delta = velocity.x * dt;
        let mut x_blocked = false;
        if x_delta.abs() > COLLISION_EPSILON {
            let test_x = Vec3::new(position.x + x_delta, new_pos.y, new_pos.z);
            if !self.check_collision(test_x) {
                new_pos.x = test_x.x;
            } else {
                x_blocked = true;
                hit_normal = Some(Vec3::new(-x_delta.signum(), 0.0, 0.0));
                new_vel.x = 0.0;
            }
        }

        // Phase 3: Z-axis resolution
        let z_delta = velocity.z * dt;
        let mut z_blocked = false;
        if z_delta.abs() > COLLISION_EPSILON {
            let test_z = Vec3::new(new_pos.x, new_pos.y, position.z + z_delta);
            if !self.check_collision(test_z) {
                new_pos.z = test_z.z;
            } else {
                z_blocked = true;
                hit_normal = Some(Vec3::new(0.0, 0.0, -z_delta.signum()));
                new_vel.z = 0.0;
            }
        }

        // Phase 4: Step-up attempt (only if horizontal collision and grounded)
        if allow_step_up && (x_blocked || z_blocked) && self.is_grounded(new_pos) {
            if let Some(step_result) = self.try_step_up(
                new_pos,
                Vec3::new(
                    if x_blocked { velocity.x } else { 0.0 },
                    0.0,
                    if z_blocked { velocity.z } else { 0.0 },
                ),
                dt,
            ) {
                new_pos = step_result;
                stepped = true;
                // Restore blocked velocities after step-up
                if x_blocked {
                    new_vel.x = velocity.x;
                }
                if z_blocked {
                    new_vel.z = velocity.z;
                }
                hit_normal = None; // Clear hit normal after successful step-up
            }
        }

        CollisionResult {
            position: new_pos,
            velocity: new_vel,
            grounded,
            stepped,
            hit_normal,
        }
    }

    /// Try to step up over an obstacle
    /// Returns Some(new_position) if step-up succeeded, None otherwise
    pub fn try_step_up(&self, position: Vec3, velocity: Vec3, dt: f32) -> Option<Vec3> {
        // Only attempt if there's horizontal movement intent
        let horiz_vel = Vec3::new(velocity.x, 0.0, velocity.z);
        if horiz_vel.length_squared() < COLLISION_EPSILON {
            return None;
        }

        // Try elevations from 0.1 to STEP_HEIGHT in increments
        let increments = 5;
        for i in 1..=increments {
            let height = (i as f32 / increments as f32) * STEP_HEIGHT;
            let elevated = Vec3::new(position.x, position.y + height, position.z);

            // Check no collision at elevated position (head clearance)
            if self.check_collision(elevated) {
                continue;
            }

            // Check can move horizontally at this height
            let h_delta = horiz_vel * dt;
            let forward = Vec3::new(elevated.x + h_delta.x, elevated.y, elevated.z + h_delta.z);

            if !self.check_collision(forward) {
                // Success - can step up and move forward
                return Some(forward);
            }
        }

        None
    }

    /// Clamp small position values to epsilon to prevent floating-point drift
    fn clamp_position_epsilon(&self, position: Vec3) -> Vec3 {
        Vec3::new(
            self.clamp_epsilon(position.x),
            self.clamp_epsilon(position.y),
            self.clamp_epsilon(position.z),
        )
    }

    /// Clamp small velocity values to zero
    fn clamp_velocity_epsilon(&self, velocity: Vec3) -> Vec3 {
        Vec3::new(
            if velocity.x.abs() < COLLISION_EPSILON {
                0.0
            } else {
                velocity.x
            },
            if velocity.y.abs() < COLLISION_EPSILON {
                0.0
            } else {
                velocity.y
            },
            if velocity.z.abs() < COLLISION_EPSILON {
                0.0
            } else {
                velocity.z
            },
        )
    }

    /// Clamp a value to avoid sub-epsilon precision issues
    fn clamp_epsilon(&self, value: f32) -> f32 {
        // Round to 3 decimal places to avoid floating-point drift
        (value * 1000.0).round() / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_floor_world() -> CollisionWorld {
        // 4x4x4 world with floor at y=0
        let size = [4, 4, 4];
        let mut blocks = vec![BlockType::AIR; 64];

        // Set floor (y=0 layer)
        for z in 0..4 {
            for x in 0..4 {
                let y = 0;
                let index = z * 16 + y * 4 + x;
                blocks[index as usize] = BlockType::STONE;
            }
        }

        CollisionWorld::new(size, blocks)
    }

    fn make_wall_world() -> CollisionWorld {
        // 8x8x8 world with floor and a wall at x=4
        let size = [8, 8, 8];
        let mut blocks = vec![BlockType::AIR; 512];

        // Set floor (y=0 layer)
        let y = 0;
        for z in 0..8 {
            for x in 0..8 {
                let index = z * 64 + y * 8 + x;
                blocks[index as usize] = BlockType::STONE;
            }
        }

        // Set wall at x=4 (all y and z)
        for z in 0..8 {
            for y in 1..8 {
                let index = z * 64 + y * 8 + 4;
                blocks[index as usize] = BlockType::STONE;
            }
        }

        CollisionWorld::new(size, blocks)
    }

    fn make_step_world() -> CollisionWorld {
        // 16x16x16 world with floor and a 1-block step at x>=8
        // With STEP_HEIGHT = 1.0, player can step up one full block
        let size = [16, 16, 16];
        let mut blocks = vec![BlockType::AIR; 16 * 16 * 16];

        // Set floor (y=0 layer)
        let floor_y = 0;
        for z in 0..16 {
            for x in 0..16 {
                let index = z * 256 + floor_y * 16 + x;
                blocks[index] = BlockType::STONE;
            }
        }

        // Create a raised platform at x>=8, y=1
        // Player standing at y=1 (on floor surface at y=0-1)
        // Step surface at y=2 (on block at y=1)
        // Step height is 1.0 block = within STEP_HEIGHT
        let step_y = 1;
        for z in 0..16 {
            for x in 8..16 {
                let index = z * 256 + step_y * 16 + x;
                blocks[index] = BlockType::STONE;
            }
        }

        CollisionWorld::new(size, blocks)
    }

    #[test]
    fn test_floor_collision() {
        let world = make_floor_world();

        // Above floor - no collision
        assert!(!world.check_collision(Vec3::new(2.0, 1.0, 2.0)));

        // In floor - collision
        assert!(world.check_collision(Vec3::new(2.0, 0.0, 2.0)));
    }

    #[test]
    fn test_grounded() {
        let world = make_floor_world();

        // On floor (just above it)
        assert!(world.is_grounded(Vec3::new(2.0, 1.0, 2.0)));

        // In air (well above floor)
        assert!(!world.is_grounded(Vec3::new(2.0, 2.0, 2.0)));
    }

    #[test]
    fn test_wall_collision() {
        let world = make_wall_world();

        // Before wall - no collision
        assert!(!world.check_collision(Vec3::new(3.0, 1.0, 4.0)));

        // At wall - collision (player extends into wall)
        assert!(world.check_collision(Vec3::new(3.7, 1.0, 4.0)));
    }

    #[test]
    fn test_move_and_slide_v2_y_first() {
        let world = make_floor_world();

        // Player just above ground falling - should collide and be grounded
        // Start at y=1.05 (just slightly above ground level y=1)
        let result = world.move_and_slide_v2(
            Vec3::new(2.0, 1.05, 2.0),
            Vec3::new(0.0, -10.0, 0.0),
            1.0 / 60.0,
            false,
        );

        // Should detect collision when trying to move into floor
        // Y velocity should be zeroed (collision detected)
        assert_eq!(result.velocity.y, 0.0);
        // Should be grounded since we tried to move down and hit floor
        assert!(result.grounded);
    }

    #[test]
    fn test_corner_collision_smooth() {
        let world = make_wall_world();

        // Player moving diagonally into corner (wall at x=4)
        let start = Vec3::new(3.0, 1.0, 4.0);
        let velocity = Vec3::new(5.0, 0.0, 5.0); // Moving +X and +Z

        let result = world.move_and_slide_v2(start, velocity, 1.0 / 60.0, false);

        // X movement should be blocked (wall at x=4)
        // Z movement should succeed (sliding along wall)
        assert!(result.position.x < 3.7); // Blocked before wall
        assert!(result.position.z > start.z); // Slid along wall
    }

    #[test]
    fn test_falling_onto_surface() {
        let world = make_floor_world();

        // Player falling fast from close to floor (within one frame's travel)
        // At -20 m/s and 1/60s, player moves 0.33 units per frame
        // Start at y=1.1 (just above floor), should collide and stop
        let result = world.move_and_slide_v2(
            Vec3::new(2.0, 1.1, 2.0),
            Vec3::new(0.0, -20.0, 0.0),
            1.0 / 60.0,
            false,
        );

        // Should not pass through floor (floor is y=0-1, player stands at y=1+)
        assert!(result.position.y >= 1.0 - COLLISION_EPSILON);
        // Should be grounded since collision was detected while moving down
        assert!(result.grounded);
    }

    #[test]
    fn test_tunneling_prevention() {
        let world = make_wall_world();

        // Player moving very fast toward wall
        let result = world.move_and_slide_v2(
            Vec3::new(2.0, 1.0, 4.0),
            Vec3::new(100.0, 0.0, 0.0), // Very fast
            1.0 / 60.0,
            false,
        );

        // Should not tunnel through wall
        assert!(result.position.x < 4.0 - PLAYER_HALF_WIDTH + COLLISION_EPSILON);
    }

    #[test]
    fn test_collision_result_struct() {
        let result = CollisionResult::default();
        assert_eq!(result.position, Vec3::ZERO);
        assert_eq!(result.velocity, Vec3::ZERO);
        assert!(!result.grounded);
        assert!(!result.stepped);
        assert!(result.hit_normal.is_none());
    }

    #[test]
    fn test_position_valid() {
        let world = make_floor_world();

        assert!(world.is_position_valid(Vec3::new(2.0, 1.0, 2.0)));
        assert!(!world.is_position_valid(Vec3::new(2.0, 0.0, 2.0)));
    }

    // ============================================================================
    // US3 - Step-Up Tests
    // ============================================================================

    /// Helper: Create a world with ceiling for step-up tests
    fn make_step_world_with_ceiling() -> CollisionWorld {
        // 16x16x16 world with floor, step, and low ceiling
        let size = [16, 16, 16];
        let mut blocks = vec![BlockType::AIR; 16 * 16 * 16];

        // Set floor (y=0 layer)
        let floor_y = 0;
        for z in 0..16 {
            for x in 0..16 {
                let index = z * 256 + floor_y * 16 + x;
                blocks[index] = BlockType::STONE;
            }
        }

        // Set step at x>=8, y=1 (1 block step)
        let step_y = 1;
        for z in 0..16 {
            for x in 8..16 {
                let index = z * 256 + step_y * 16 + x;
                blocks[index] = BlockType::STONE;
            }
        }

        // Set low ceiling at y=3 for x<8
        // Player at y=1 (1.8m tall) has head at y=2.8
        // Stepping up to y=2 would put head at y=3.8, blocked by ceiling at y=3
        let ceiling_y = 3;
        for z in 0..16 {
            for x in 0..8 {
                let index = z * 256 + ceiling_y * 16 + x;
                blocks[index] = BlockType::STONE;
            }
        }

        CollisionWorld::new(size, blocks)
    }

    /// Helper: Create a world with a tall wall (not steppable)
    fn make_tall_wall_world() -> CollisionWorld {
        // 16x16x16 world with floor and a tall wall (2+ blocks, beyond STEP_HEIGHT)
        let size = [16, 16, 16];
        let mut blocks = vec![BlockType::AIR; 16 * 16 * 16];

        // Set floor (y=0 layer)
        let floor_y = 0;
        for z in 0..16 {
            for x in 0..16 {
                let index = z * 256 + floor_y * 16 + x;
                blocks[index] = BlockType::STONE;
            }
        }

        // Set tall wall at x=8 (y=1 to y=4, 4 blocks tall - way beyond STEP_HEIGHT of 1.0)
        for z in 0..16 {
            for y in 1..5 {
                let index = z * 256 + y * 16 + 8;
                blocks[index] = BlockType::STONE;
            }
        }

        CollisionWorld::new(size, blocks)
    }

    /// T037 [US3] Single block step-up works
    #[test]
    fn test_step_up_single_block() {
        let world = make_step_world();

        // Player on floor, positioned near step edge (step starts at x=8)
        // Player half-width is 0.4, so player at x=7.55 would be touching step at x=8
        let start = Vec3::new(7.55, 1.0, 8.0); // Close to step edge
        let velocity = Vec3::new(6.0, 0.0, 0.0);

        // Simulate several frames to allow step-up to occur
        let mut pos = start;
        let mut vel = velocity;
        let mut stepped = false;

        for _ in 0..20 {
            let result = world.move_and_slide_v2(pos, vel, 1.0 / 60.0, true);
            pos = result.position;
            vel = result.velocity;
            if result.stepped {
                stepped = true;
            }
        }

        // Should have stepped up - check if player is at higher Y
        // After step-up, player should be on y=2 (on top of step at y=1)
        assert!(
            stepped || pos.y > start.y,
            "Player should step up. Position: {:?}, stepped: {}",
            pos,
            stepped
        );

        // Should have made progress in X direction
        assert!(pos.x > start.x, "Player should move forward after step-up");
    }

    /// T038 [P] [US3] Step-up blocked when airborne
    #[test]
    fn test_step_up_blocked_when_airborne() {
        let world = make_step_world();

        // Player in air (not grounded), moving toward step
        // Step-up should NOT occur when airborne
        let start = Vec3::new(7.0, 2.5, 8.0); // Above floor level (in air)
        let velocity = Vec3::new(6.0, -5.0, 0.0); // Moving +X and falling

        let result = world.move_and_slide_v2(start, velocity, 1.0 / 60.0, true);

        // Should not have stepped - player was airborne
        assert!(
            !result.stepped,
            "Step-up should not occur when player is airborne"
        );
    }

    /// T039 [P] [US3] Step-up blocked by ceiling
    #[test]
    fn test_step_up_blocked_by_ceiling() {
        let world = make_step_world_with_ceiling();

        // Player on floor under low ceiling, trying to step up
        // Ceiling at y=3 means player (1.8m tall) at y=1 has head at y=2.8
        // Step-up to y=2 would put head at y=3.8 - blocked by ceiling at y=3
        let start = Vec3::new(7.0, 1.0, 8.0);
        let velocity = Vec3::new(6.0, 0.0, 0.0);

        // Simulate several frames
        let mut pos = start;
        let mut vel = velocity;
        let mut stepped = false;

        for _ in 0..10 {
            let result = world.move_and_slide_v2(pos, vel, 1.0 / 60.0, true);
            pos = result.position;
            vel = result.velocity;
            if result.stepped {
                stepped = true;
            }
        }

        // With ceiling blocking, step-up should fail
        assert!(
            !stepped,
            "Step-up should be blocked when ceiling prevents it"
        );
    }

    /// T040 [P] [US3] Step-up blocked for tall walls
    #[test]
    fn test_step_up_blocked_for_tall_walls() {
        let world = make_tall_wall_world();

        // Player moving toward a tall wall (4 blocks, not steppable)
        let start = Vec3::new(7.0, 1.0, 8.0);
        let velocity = Vec3::new(6.0, 0.0, 0.0);

        // Simulate several frames
        let mut pos = start;
        let mut vel = velocity;
        let mut stepped = false;

        for _ in 0..20 {
            let result = world.move_and_slide_v2(pos, vel, 1.0 / 60.0, true);
            pos = result.position;
            vel = result.velocity;
            if result.stepped {
                stepped = true;
            }
        }

        // Should not have stepped - wall is too tall (4 blocks > STEP_HEIGHT of 1.0)
        assert!(
            !stepped,
            "Step-up should not occur for tall walls (height > STEP_HEIGHT)"
        );

        // Should be blocked near wall (x=8 - half_width = 7.6)
        assert!(
            pos.x < 7.7,
            "Player should be blocked by tall wall. X: {}",
            pos.x
        );
    }
}
