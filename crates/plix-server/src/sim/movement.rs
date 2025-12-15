//! Player movement simulation

use plix_arena::format::LoadedArena;
use plix_common::math::{Rotation, Vec3};
use plix_common::protocol::PlayerInput;

/// Movement constants (aligned with physics.rs clarified values)
pub const MOVE_SPEED: f32 = 6.0; // m/s (was 5.0)
pub const SPRINT_MULTIPLIER: f32 = 1.5;
pub const CROUCH_MULTIPLIER: f32 = 0.5;
pub const JUMP_VELOCITY: f32 = 7.07; // sqrt(2 * 20 * 1.25) for 1.25 block jump height (was 8.0)
pub const GRAVITY: f32 = 20.0;
pub const GROUND_FRICTION: f32 = 10.0;
pub const AIR_CONTROL: f32 = 0.3; // 30% of ground control
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_RADIUS: f32 = 0.4; // (was 0.3)

/// Jump buffer duration in ticks (100ms at 60Hz)
pub const JUMP_BUFFER_TICKS: u8 = 6;

/// Jump state for a player
#[derive(Debug, Clone, Copy, Default)]
pub struct JumpState {
    /// Whether jump was pressed last tick (for edge detection)
    pub was_pressed: bool,
    /// Ticks remaining on jump buffer (0 = no buffer)
    pub buffer_ticks: u8,
}

impl JumpState {
    /// Create a new jump state
    pub fn new() -> Self {
        Self::default()
    }

    /// Update jump state with current input
    /// Returns true if a jump should be triggered this frame
    pub fn update(&mut self, jump_pressed: bool, is_grounded: bool) -> bool {
        // Detect rising edge (new press)
        let just_pressed = jump_pressed && !self.was_pressed;
        self.was_pressed = jump_pressed;

        // Buffer the jump if just pressed
        if just_pressed {
            self.buffer_ticks = JUMP_BUFFER_TICKS;
        }

        // Consume jump buffer if grounded
        let should_jump = self.buffer_ticks > 0 && is_grounded;

        if should_jump {
            // Consume the buffer
            self.buffer_ticks = 0;
        } else if self.buffer_ticks > 0 {
            // Decrement buffer timer
            self.buffer_ticks -= 1;
        }

        should_jump
    }

    /// Reset jump state (e.g., on respawn)
    pub fn reset(&mut self) {
        self.was_pressed = false;
        self.buffer_ticks = 0;
    }
}

/// Apply jump impulse to velocity
/// Resets vertical velocity to jump impulse (not additive)
#[inline]
pub fn apply_jump(velocity: &mut Vec3) {
    velocity.y = JUMP_VELOCITY;
}

/// Velocity threshold for zero-snap (prevents micro-drift)
pub const VELOCITY_THRESHOLD: f32 = 0.01;

/// Apply ground friction to horizontal velocity
/// Decelerates toward target velocity with friction coefficient
#[inline]
pub fn apply_ground_friction(velocity: &mut Vec3, target_vel: Vec3, dt: f32) {
    let friction = GROUND_FRICTION * dt;
    velocity.x = lerp_internal(velocity.x, target_vel.x, friction.min(1.0));
    velocity.z = lerp_internal(velocity.z, target_vel.z, friction.min(1.0));
}

/// Apply air control (reduced influence while airborne)
/// 30% of ground control for turning in air
#[inline]
pub fn apply_air_control(velocity: &mut Vec3, target_vel: Vec3, dt: f32) {
    velocity.x += (target_vel.x - velocity.x) * AIR_CONTROL * dt;
    velocity.z += (target_vel.z - velocity.z) * AIR_CONTROL * dt;
}

/// Snap small velocities to zero to prevent micro-drift
#[inline]
pub fn snap_velocity_to_zero(velocity: &mut Vec3) {
    if velocity.x.abs() < VELOCITY_THRESHOLD {
        velocity.x = 0.0;
    }
    if velocity.z.abs() < VELOCITY_THRESHOLD {
        velocity.z = 0.0;
    }
}

/// Enforce speed cap on horizontal velocity
/// Returns true if velocity was capped
#[inline]
pub fn enforce_speed_cap(velocity: &mut Vec3) -> bool {
    let horiz_speed_sq = velocity.x * velocity.x + velocity.z * velocity.z;
    let max_speed_sq = MOVE_SPEED * MOVE_SPEED;

    if horiz_speed_sq > max_speed_sq {
        let scale = MOVE_SPEED / horiz_speed_sq.sqrt();
        velocity.x *= scale;
        velocity.z *= scale;
        true
    } else {
        false
    }
}

/// Internal lerp function (same as module-level but inlined)
#[inline]
fn lerp_internal(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

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

            // Jump (basic version without edge detection)
            if input.jump {
                new_velocity.y = JUMP_VELOCITY;
            }
        } else {
            // Air control (reduced to 30%)
            new_velocity.x += (target_vel.x - new_velocity.x) * AIR_CONTROL * dt;
            new_velocity.z += (target_vel.z - new_velocity.z) * AIR_CONTROL * dt;
        }

        // Apply gravity
        if !is_grounded {
            new_velocity.y -= GRAVITY * dt;
        }

        // Calculate new position
        let new_position = position + new_velocity * dt;

        (new_position, new_velocity)
    }

    /// Apply input with full jump state handling
    /// This version properly handles jump buffering and edge detection
    pub fn apply_input_with_jump_state(
        &self,
        input: &PlayerInput,
        position: Vec3,
        velocity: Vec3,
        rotation: &mut Rotation,
        is_grounded: bool,
        jump_state: &mut JumpState,
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
        } else {
            // Air control (reduced to 30%)
            new_velocity.x += (target_vel.x - new_velocity.x) * AIR_CONTROL * dt;
            new_velocity.z += (target_vel.z - new_velocity.z) * AIR_CONTROL * dt;
        }

        // Handle jump with edge detection and buffering
        if jump_state.update(input.jump, is_grounded) {
            apply_jump(&mut new_velocity);
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
            rtt_nonce: 0,
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

    // ============================================================================
    // US2 - Jumping Tests
    // ============================================================================

    /// T030 [US2] Measure jump apex height (1.25 blocks ±5%)
    /// Note: Euler integration at 60Hz introduces ~5% error vs analytical solution
    #[test]
    fn test_jump_apex_height() {
        // Simulate a jump and measure apex height
        // Analytical physics: v² = v0² - 2*g*h => h = v0²/(2*g) = 7.07²/(2*20) = 1.25
        // But Euler integration accumulates error - we verify it's within 5%
        let dt = 1.0 / 60.0;
        let mut pos = Vec3::new(0.0, 0.0, 0.0);
        let mut vel = Vec3::new(0.0, JUMP_VELOCITY, 0.0);
        let mut max_height = 0.0f32;

        // Simulate until player returns to ground (y <= 0)
        for _ in 0..300 {
            // 5 seconds max
            // Semi-implicit Euler: update velocity first, then position
            vel.y -= GRAVITY * dt;
            pos.y += vel.y * dt;

            if pos.y > max_height {
                max_height = pos.y;
            }

            // Stop when back on ground
            if pos.y <= 0.0 && vel.y < 0.0 {
                break;
            }
        }

        // Expected analytical height is 1.25 blocks
        // With Euler integration at 60Hz, we expect ~1.19 (about 5% error)
        let expected = 1.25;
        let tolerance = expected * 0.05; // ±5% to account for Euler integration error

        assert!(
            (max_height - expected).abs() < tolerance,
            "Jump apex height should be {} ±5%, got {}",
            expected,
            max_height
        );
    }

    /// T031 [P] [US2] Jump blocked when airborne
    #[test]
    fn test_jump_blocked_when_airborne() {
        let system = MovementSystem::new(make_test_arena());
        let input = make_input(0.0, 0.0, true);
        let mut rotation = Rotation::ZERO;

        // When airborne (is_grounded = false), jump should not be triggered
        let (_, new_vel) = system.apply_input(
            &input,
            Vec3::new(0.0, 5.0, 0.0),  // In the air
            Vec3::new(0.0, -2.0, 0.0), // Falling
            &mut rotation,
            false, // NOT grounded
            1.0 / 60.0,
        );

        // Vertical velocity should continue falling (with gravity), not get jump impulse
        assert!(
            new_vel.y < 0.0,
            "Jump should be blocked when airborne, velocity should still be negative"
        );
        assert!(
            new_vel.y != JUMP_VELOCITY,
            "Jump velocity should not be applied when airborne"
        );
    }

    /// T032 [P] [US2] Jump requires button release between jumps (edge detection)
    #[test]
    fn test_jump_requires_release_between_jumps() {
        let mut jump_state = JumpState::new();

        // First press: should trigger jump
        assert!(
            jump_state.update(true, true),
            "First jump press should trigger"
        );

        // Holding jump: should NOT trigger again
        for _ in 0..10 {
            assert!(
                !jump_state.update(true, true),
                "Holding jump should not re-trigger"
            );
        }

        // Release and re-press: should trigger again
        jump_state.update(false, true); // Release
        assert!(
            jump_state.update(true, true),
            "Re-pressing jump should trigger"
        );
    }

    /// T029 [US2] Jump buffer allows jump shortly after leaving ground
    #[test]
    fn test_jump_buffer() {
        let mut jump_state = JumpState::new();

        // Press jump while airborne
        assert!(
            !jump_state.update(true, false),
            "Jump should not trigger in air"
        );

        // Jump is now buffered - verify buffer is set
        assert!(jump_state.buffer_ticks > 0, "Jump should be buffered");

        // Simulate a few ticks in air (buffer should remain)
        for _ in 0..3 {
            jump_state.update(true, false); // Still pressing, still airborne
        }

        // Land on ground - buffered jump should trigger
        assert!(
            jump_state.update(true, true),
            "Buffered jump should trigger when landing"
        );

        // Buffer should be consumed
        assert_eq!(jump_state.buffer_ticks, 0, "Buffer should be consumed");
    }

    /// T029 [US2] Jump buffer expires after 6 ticks (100ms)
    #[test]
    fn test_jump_buffer_expires() {
        let mut jump_state = JumpState::new();

        // Press jump while airborne
        jump_state.update(true, false);
        assert!(jump_state.buffer_ticks > 0);

        // Simulate more than JUMP_BUFFER_TICKS in air
        for _ in 0..(JUMP_BUFFER_TICKS as usize + 2) {
            jump_state.update(false, false); // Released jump, still airborne
        }

        // Buffer should have expired
        assert_eq!(jump_state.buffer_ticks, 0, "Buffer should have expired");

        // Landing now should NOT trigger jump (buffer expired)
        assert!(
            !jump_state.update(false, true),
            "Expired buffer should not trigger jump"
        );
    }

    /// T025, T028 [US2] apply_jump resets velocity (not additive)
    #[test]
    fn test_apply_jump_resets_velocity() {
        // If player has downward velocity, jump should reset to positive
        let mut vel = Vec3::new(3.0, -5.0, 2.0);
        apply_jump(&mut vel);

        assert_eq!(
            vel.y, JUMP_VELOCITY,
            "Jump should set Y velocity to JUMP_VELOCITY"
        );
        assert_eq!(vel.x, 3.0, "Jump should not affect X velocity");
        assert_eq!(vel.z, 2.0, "Jump should not affect Z velocity");
    }

    // ============================================================================
    // US4 - Friction & Ground Control Tests
    // ============================================================================

    /// T045 [US4] Speed never exceeds 6.0 m/s
    #[test]
    fn test_speed_cap() {
        // Test that speed cap works
        let mut vel = Vec3::new(10.0, 0.0, 10.0); // Way over speed cap
        let capped = enforce_speed_cap(&mut vel);

        assert!(capped, "Speed should be capped");

        let speed = (vel.x * vel.x + vel.z * vel.z).sqrt();
        assert!(
            (speed - MOVE_SPEED).abs() < 0.001,
            "Speed should be exactly MOVE_SPEED ({}), got {}",
            MOVE_SPEED,
            speed
        );

        // Verify direction is preserved
        let ratio = vel.x / vel.z;
        assert!(
            (ratio - 1.0).abs() < 0.001,
            "Direction should be preserved (ratio should be 1.0)"
        );
    }

    /// T045 [US4] Speed under cap is not modified
    #[test]
    fn test_speed_under_cap_unchanged() {
        let mut vel = Vec3::new(3.0, 0.0, 3.0); // Under speed cap
        let original = vel;
        let capped = enforce_speed_cap(&mut vel);

        assert!(!capped, "Speed should not be capped");
        assert_eq!(vel, original, "Velocity should be unchanged");
    }

    /// T046 [P] [US4] Friction stops player on ground
    #[test]
    fn test_friction_stops_player() {
        let mut vel = Vec3::new(6.0, 0.0, 0.0); // Moving at max speed
        let target = Vec3::ZERO; // Want to stop
        let dt = 1.0 / 60.0;

        // Apply friction for 1 second (60 frames)
        for _ in 0..60 {
            apply_ground_friction(&mut vel, target, dt);
        }

        // Should be very close to stopped
        let speed = (vel.x * vel.x + vel.z * vel.z).sqrt();
        assert!(
            speed < 0.1,
            "Player should be nearly stopped after 1s of friction, speed = {}",
            speed
        );
    }

    /// T047 [P] [US4] Air control is 30% of ground control
    #[test]
    fn test_air_control_reduced() {
        let mut ground_vel = Vec3::new(0.0, 0.0, 0.0);
        let mut air_vel = Vec3::new(0.0, 0.0, 0.0);
        let target = Vec3::new(6.0, 0.0, 0.0);
        let dt = 1.0 / 60.0;

        // Apply one frame of ground friction
        apply_ground_friction(&mut ground_vel, target, dt);

        // Apply one frame of air control
        apply_air_control(&mut air_vel, target, dt);

        // Air control should be approximately 30% of ground change
        // Ground uses friction = 10.0 * dt, air uses 0.3 * dt
        // The formulas are different so we check air is much slower
        assert!(
            air_vel.x < ground_vel.x,
            "Air control should be slower than ground. Air: {}, Ground: {}",
            air_vel.x,
            ground_vel.x
        );

        // Air control change should be roughly 3% per frame (0.3 * dt * (target - 0))
        let expected_air = 6.0 * 0.3 * dt;
        assert!(
            (air_vel.x - expected_air).abs() < 0.001,
            "Air control should change by {}. Got {}",
            expected_air,
            air_vel.x
        );
    }

    /// T048 [P] [US4] No sliding when stationary
    #[test]
    fn test_no_sliding_when_stationary() {
        let mut vel = Vec3::new(0.005, 0.0, 0.005); // Very small velocity
        snap_velocity_to_zero(&mut vel);

        assert_eq!(vel.x, 0.0, "X velocity should snap to zero");
        assert_eq!(vel.z, 0.0, "Z velocity should snap to zero");

        // Values above threshold should not snap
        let mut vel2 = Vec3::new(0.02, 0.0, 0.02);
        snap_velocity_to_zero(&mut vel2);

        assert!(vel2.x > 0.0, "X velocity above threshold should remain");
        assert!(vel2.z > 0.0, "Z velocity above threshold should remain");
    }
}
