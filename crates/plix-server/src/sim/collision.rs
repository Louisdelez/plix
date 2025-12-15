//! Collision detection

use plix_common::math::{Vec3, AABB};
use plix_common::types::BlockType;

/// Player collision box half-extents
pub const PLAYER_HALF_WIDTH: f32 = 0.3;
pub const PLAYER_HALF_HEIGHT: f32 = 0.9;

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

    /// Get player AABB at position
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

    /// Check if player AABB collides with any solid blocks
    pub fn check_collision(&self, position: Vec3) -> bool {
        let aabb = Self::player_aabb(position);

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

    /// Check if player is on ground
    pub fn is_grounded(&self, position: Vec3) -> bool {
        // Check slightly below feet
        let check_pos = Vec3::new(position.x, position.y - 0.01, position.z);
        self.check_collision(check_pos)
    }

    /// Move and slide collision resolution
    pub fn move_and_slide(&self, position: Vec3, velocity: Vec3, dt: f32) -> (Vec3, Vec3, bool) {
        let mut new_pos = position;
        let mut new_vel = velocity;
        let mut grounded = false;

        // Try X movement
        let test_x = Vec3::new(position.x + velocity.x * dt, position.y, position.z);
        if !self.check_collision(test_x) {
            new_pos.x = test_x.x;
        } else {
            new_vel.x = 0.0;
        }

        // Try Y movement
        let test_y = Vec3::new(new_pos.x, position.y + velocity.y * dt, position.z);
        if !self.check_collision(test_y) {
            new_pos.y = test_y.y;
        } else {
            if velocity.y < 0.0 {
                grounded = true;
            }
            new_vel.y = 0.0;
        }

        // Try Z movement
        let test_z = Vec3::new(new_pos.x, new_pos.y, position.z + velocity.z * dt);
        if !self.check_collision(test_z) {
            new_pos.z = test_z.z;
        } else {
            new_vel.z = 0.0;
        }

        (new_pos, new_vel, grounded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_floor_world() -> CollisionWorld {
        // 4x4x4 world with floor at y=0
        let size = [4, 4, 4];
        let mut blocks = vec![BlockType::AIR; 64];

        // Set floor
        for z in 0..4 {
            for x in 0..4 {
                blocks[(z * 16 + x) as usize] = BlockType::STONE;
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

        // On floor
        assert!(world.is_grounded(Vec3::new(2.0, 1.0, 2.0)));

        // In air
        assert!(!world.is_grounded(Vec3::new(2.0, 2.0, 2.0)));
    }
}
