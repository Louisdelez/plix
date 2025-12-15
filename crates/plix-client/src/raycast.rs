//! DDA Raycasting for block targeting
//! T029-T030 [US1]

use plix_arena::format::LoadedArena;
use plix_common::math::Vec3;
use plix_common::types::{BlockPos, BlockType};

/// Maximum raycast distance in blocks
pub const MAX_RAYCAST_DISTANCE: f32 = 5.0;

/// Result of a raycast against the voxel grid
#[derive(Debug, Clone)]
pub struct RaycastHit {
    /// Position of the hit block
    pub block_pos: BlockPos,
    /// The type of block that was hit
    pub block_type: BlockType,
    /// Distance from ray origin to hit point
    pub distance: f32,
    /// Normal of the face that was hit (integer vector)
    pub face_normal: [i32; 3],
}

impl RaycastHit {
    /// Get the adjacent block position (for placing blocks)
    pub fn adjacent_pos(&self) -> BlockPos {
        BlockPos {
            x: self.block_pos.x + self.face_normal[0],
            y: self.block_pos.y + self.face_normal[1],
            z: self.block_pos.z + self.face_normal[2],
        }
    }
}

/// Perform a DDA raycast through the voxel grid
///
/// Uses the Digital Differential Analyzer algorithm for efficient voxel traversal.
/// Returns the first solid block hit within max_distance.
pub fn raycast_blocks(
    arena: &LoadedArena,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> Option<RaycastHit> {
    // Normalize direction
    let dir = direction.normalize();
    if dir.length_squared() < 0.001 {
        return None;
    }

    // Current voxel position
    let mut current_x = origin.x.floor() as i32;
    let mut current_y = origin.y.floor() as i32;
    let mut current_z = origin.z.floor() as i32;

    // Step direction for each axis
    let step_x = if dir.x >= 0.0 { 1 } else { -1 };
    let step_y = if dir.y >= 0.0 { 1 } else { -1 };
    let step_z = if dir.z >= 0.0 { 1 } else { -1 };

    // Distance to next voxel boundary for each axis
    let next_boundary_x = if dir.x >= 0.0 {
        (current_x + 1) as f32 - origin.x
    } else {
        origin.x - current_x as f32
    };
    let next_boundary_y = if dir.y >= 0.0 {
        (current_y + 1) as f32 - origin.y
    } else {
        origin.y - current_y as f32
    };
    let next_boundary_z = if dir.z >= 0.0 {
        (current_z + 1) as f32 - origin.z
    } else {
        origin.z - current_z as f32
    };

    // t values for stepping (how far along ray to next boundary)
    let mut t_max_x = if dir.x.abs() > f32::EPSILON {
        next_boundary_x / dir.x.abs()
    } else {
        f32::MAX
    };
    let mut t_max_y = if dir.y.abs() > f32::EPSILON {
        next_boundary_y / dir.y.abs()
    } else {
        f32::MAX
    };
    let mut t_max_z = if dir.z.abs() > f32::EPSILON {
        next_boundary_z / dir.z.abs()
    } else {
        f32::MAX
    };

    // Delta t for stepping one voxel
    let t_delta_x = if dir.x.abs() > f32::EPSILON {
        1.0 / dir.x.abs()
    } else {
        f32::MAX
    };
    let t_delta_y = if dir.y.abs() > f32::EPSILON {
        1.0 / dir.y.abs()
    } else {
        f32::MAX
    };
    let t_delta_z = if dir.z.abs() > f32::EPSILON {
        1.0 / dir.z.abs()
    } else {
        f32::MAX
    };

    let mut traveled = 0.0;
    let mut face_normal = [0i32; 3];

    while traveled < max_distance {
        // Check current block
        let pos = BlockPos {
            x: current_x,
            y: current_y,
            z: current_z,
        };

        // Check if in bounds and solid
        if arena.is_in_bounds(pos) {
            let block_type = arena.get_block_at(pos);
            if block_type != BlockType::AIR {
                return Some(RaycastHit {
                    block_pos: pos,
                    block_type,
                    distance: traveled,
                    face_normal,
                });
            }
        }

        // Step to next voxel (DDA algorithm)
        if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                current_x += step_x;
                traveled = t_max_x;
                t_max_x += t_delta_x;
                face_normal = [-step_x, 0, 0];
            } else {
                current_z += step_z;
                traveled = t_max_z;
                t_max_z += t_delta_z;
                face_normal = [0, 0, -step_z];
            }
        } else if t_max_y < t_max_z {
            current_y += step_y;
            traveled = t_max_y;
            t_max_y += t_delta_y;
            face_normal = [0, -step_y, 0];
        } else {
            current_z += step_z;
            traveled = t_max_z;
            t_max_z += t_delta_z;
            face_normal = [0, 0, -step_z];
        }

        // Check bounds
        if current_x < 0 || current_y < 0 || current_z < 0 {
            // Out of bounds, stop raycasting
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_arena::format::{Arena, ArenaMetadata, BlockDefinitions};

    fn make_test_arena() -> LoadedArena {
        let definition = Arena {
            metadata: ArenaMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
                size: [16, 16, 16],
            },
            spawn_points: vec![],
            blocks: BlockDefinitions {
                floor: None,
                walls: None,
                regions: vec![],
            },
        };

        // Create arena with floor at y=0
        let mut blocks = vec![BlockType::AIR; 16 * 16 * 16];
        // Fill floor (y=0) with stone
        for z in 0..16 {
            for x in 0..16 {
                let y = 0;
                let index = z * 16 * 16 + y * 16 + x;
                blocks[index] = BlockType::STONE;
            }
        }

        LoadedArena { definition, blocks }
    }

    #[test]
    fn test_raycast_hits_floor() {
        let arena = make_test_arena();
        // Player at (8, 2, 8) looking down
        let origin = Vec3::new(8.5, 2.0, 8.5);
        let direction = Vec3::new(0.0, -1.0, 0.0);

        let hit = raycast_blocks(&arena, origin, direction, 5.0);
        assert!(hit.is_some());

        let hit = hit.unwrap();
        assert_eq!(hit.block_pos.y, 0);
        assert_eq!(hit.face_normal, [0, 1, 0]); // Top face
    }

    #[test]
    fn test_raycast_misses_air() {
        let arena = make_test_arena();
        // Player at (8, 2, 8) looking up (only air above)
        let origin = Vec3::new(8.5, 2.0, 8.5);
        let direction = Vec3::new(0.0, 1.0, 0.0);

        let hit = raycast_blocks(&arena, origin, direction, 5.0);
        assert!(hit.is_none());
    }

    #[test]
    fn test_raycast_respects_max_distance() {
        let arena = make_test_arena();
        // Player at (8, 10, 8) looking down - floor is 10 blocks away
        let origin = Vec3::new(8.5, 10.0, 8.5);
        let direction = Vec3::new(0.0, -1.0, 0.0);

        // Max distance of 5 shouldn't reach floor
        let hit = raycast_blocks(&arena, origin, direction, 5.0);
        assert!(hit.is_none());

        // Max distance of 15 should reach floor
        let hit = raycast_blocks(&arena, origin, direction, 15.0);
        assert!(hit.is_some());
    }

    #[test]
    fn test_adjacent_pos() {
        let hit = RaycastHit {
            block_pos: BlockPos { x: 5, y: 0, z: 5 },
            block_type: BlockType::STONE,
            distance: 2.0,
            face_normal: [0, 1, 0], // Top face
        };

        let adjacent = hit.adjacent_pos();
        assert_eq!(adjacent, BlockPos { x: 5, y: 1, z: 5 });
    }
}
