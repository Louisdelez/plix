//! Player movement simulation

use plix_arena::format::LoadedArena;
use plix_common::math::{Rotation, Vec3};
use plix_common::protocol::PlayerInput;

/// Movement constants
pub const MOVE_SPEED: f32 = 5.0; // blocks per second
pub const SPRINT_MULTIPLIER: f32 = 1.5;
pub const CROUCH_MULTIPLIER: f32 = 0.5;
pub const JUMP_VELOCITY: f32 = 8.0;
pub const GRAVITY: f32 = 20.0;
pub const GROUND_FRICTION: f32 = 10.0;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_RADIUS: f32 = 0.3;

/// Movement system for processing player inputs
#[derive(Debug)]
pub struct MovementSystem {
    arena: LoadedArena,
}

impl MovementSystem {
    /// Create a new movement system with arena collision
    pub fn new(arena: LoadedArena) -> Self {
        Self { arena }
    }

    /// Get immutable reference to arena
    pub fn arena(&self) -> &LoadedArena {
        &self.arena
    }

    /// Get mutable reference to arena
    pub fn arena_mut(&mut self) -> &mut LoadedArena {
        &mut self.arena
    }

    /// Move a player with collision detection
    pub fn move_player(
        &self,
        position: Vec3,
        velocity: Vec3,
        input: &PlayerInput,
        dt: f32,
    ) -> Vec3 {
        // Calculate movement direction based on input rotation
        let yaw = input.yaw;
        let forward = Vec3::new(-yaw.sin(), 0.0, yaw.cos());
        let right = Vec3::new(yaw.cos(), 0.0, yaw.sin());

        let mut move_dir = forward * input.move_forward + right * input.move_right;
        if move_dir.length_squared() > 1.0 {
            move_dir = move_dir.normalize();
        }

        let speed = if input.crouch {
            MOVE_SPEED * CROUCH_MULTIPLIER
        } else {
            MOVE_SPEED
        };

        let target_vel = move_dir * speed;
        let mut new_position = position + target_vel * dt;

        // Simple ground collision (y = floor level + 1)
        let floor_y = 1.0; // Assuming floor at y=0, spawn at y=1
        if new_position.y < floor_y {
            new_position.y = floor_y;
        }

        // Clamp to arena bounds
        let [sx, _sy, sz] = self.arena.size();
        new_position.x = new_position
            .x
            .clamp(PLAYER_RADIUS, sx as f32 - PLAYER_RADIUS);
        new_position.z = new_position
            .z
            .clamp(PLAYER_RADIUS, sz as f32 - PLAYER_RADIUS);

        new_position
    }

    /// Apply input to get new position and velocity (full physics)
    pub fn apply_input(
        &self,
        input: &PlayerInput,
        position: Vec3,
        velocity: Vec3,
        rotation: &mut Rotation,
        is_grounded: bool,
        dt: f32,
    ) -> (Vec3, Vec3) {
        // Update rotation from input
        rotation.yaw = input.yaw;
        rotation.pitch = input.pitch;

        // Calculate movement direction
        let forward = rotation.forward();
        let right = rotation.right();

        // Only use horizontal components for movement
        let forward_flat = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let right_flat = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

        let mut move_dir = forward_flat * input.move_forward + right_flat * input.move_right;

        if move_dir.length_squared() > 1.0 {
            move_dir = move_dir.normalize();
        }

        // Calculate speed modifier
        let speed = if input.crouch {
            MOVE_SPEED * CROUCH_MULTIPLIER
        } else {
            MOVE_SPEED
        };

        // Calculate target velocity
        let target_vel = move_dir * speed;

        // Apply velocity with friction
        let mut new_velocity = velocity;

        if is_grounded {
            // Ground movement with friction
            let friction = GROUND_FRICTION * dt;
            new_velocity.x = lerp(new_velocity.x, target_vel.x, friction.min(1.0));
            new_velocity.z = lerp(new_velocity.z, target_vel.z, friction.min(1.0));

            // Jump
            if input.jump {
                new_velocity.y = JUMP_VELOCITY;
            }
        } else {
            // Air control (reduced)
            let air_control = 0.3;
            new_velocity.x += (target_vel.x - new_velocity.x) * air_control * dt;
            new_velocity.z += (target_vel.z - new_velocity.z) * air_control * dt;
        }

        // Apply gravity
        if !is_grounded {
            new_velocity.y -= GRAVITY * dt;
        }

        // Calculate new position
        let new_position = position + new_velocity * dt;

        (new_position, new_velocity)
    }
}

/// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_arena::format::{Arena, ArenaMetadata, BlockDefinitions};
    use plix_common::time::Tick;
    use plix_common::types::InputSeq;

    fn make_test_arena() -> LoadedArena {
        LoadedArena {
            definition: Arena {
                metadata: ArenaMetadata {
                    name: "Test".to_string(),
                    version: "1.0".to_string(),
                    size: [64, 32, 64],
                },
                spawn_points: vec![],
                blocks: BlockDefinitions {
                    floor: None,
                    walls: None,
                    regions: vec![],
                },
            },
            blocks: vec![],
        }
    }

    fn make_input(forward: f32, right: f32, jump: bool) -> PlayerInput {
        PlayerInput {
            seq: InputSeq(0),
            tick: Tick(0),
            move_forward: forward,
            move_right: right,
            jump,
            crouch: false,
            attack: false,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    #[test]
    fn test_forward_movement() {
        let system = MovementSystem::new(make_test_arena());
        let input = make_input(1.0, 0.0, false);
        let mut rotation = Rotation::ZERO;

        let (new_pos, _) = system.apply_input(
            &input,
            Vec3::ZERO,
            Vec3::ZERO,
            &mut rotation,
            true,
            1.0 / 60.0,
        );

        // Should move forward (positive Z with zero yaw)
        assert!(new_pos.z > 0.0);
    }

    #[test]
    fn test_jump() {
        let system = MovementSystem::new(make_test_arena());
        let input = make_input(0.0, 0.0, true);
        let mut rotation = Rotation::ZERO;

        let (_, new_vel) = system.apply_input(
            &input,
            Vec3::ZERO,
            Vec3::ZERO,
            &mut rotation,
            true,
            1.0 / 60.0,
        );

        assert_eq!(new_vel.y, JUMP_VELOCITY);
    }

    #[test]
    fn test_gravity() {
        let system = MovementSystem::new(make_test_arena());
        let input = make_input(0.0, 0.0, false);
        let mut rotation = Rotation::ZERO;

        let (_, new_vel) = system.apply_input(
            &input,
            Vec3::ZERO,
            Vec3::ZERO,
            &mut rotation,
            false, // Not grounded
            1.0 / 60.0,
        );

        assert!(new_vel.y < 0.0); // Should fall
    }
}
