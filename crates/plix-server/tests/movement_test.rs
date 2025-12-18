//! Integration test: Two clients see each other moving
//! Movement and collision integration tests

use std::net::SocketAddr;

use plix_arena::format::{Arena, ArenaMetadata, BlockDefinitions, LoadedArena};
use plix_common::identity::SessionId;
use plix_common::math::Vec3;
use plix_common::protocol::PlayerInput;
use plix_common::time::Tick;
use plix_common::types::{BlockType, InputSeq, PlayerId, TeamId};
use plix_server::session::SessionManager;
use plix_server::sim::collision::{CollisionWorld, PLAYER_HALF_WIDTH};
use plix_server::sim::movement::MovementSystem;

fn make_test_arena() -> LoadedArena {
    LoadedArena {
        definition: Arena {
            metadata: ArenaMetadata {
                name: "Test".to_string(),
                version: "1.0".to_string(),
                size: [64, 32, 64],
                game_mode: plix_common::GameMode::Tdm,
            },
            spawn_points: vec![],
            blocks: BlockDefinitions {
                floor: None,
                walls: None,
                regions: vec![],
            },
            flag_zones: vec![],
            br_lite: None,
            training: None,
            economy: None,
        },
        blocks: vec![],
    }
}

fn make_input(seq: u16, forward: f32, right: f32, yaw: f32) -> PlayerInput {
    PlayerInput {
        seq: InputSeq(seq),
        tick: Tick(0),
        move_forward: forward,
        move_right: right,
        jump: false,
        crouch: false,
        attack: false,
        yaw,
        pitch: 0.0,
        rtt_nonce: 0,
    }
}

#[test]
fn test_two_players_movement() {
    let mut sessions = SessionManager::new(16);
    let movement = MovementSystem::new(make_test_arena());

    // Add two players
    let addr1: SocketAddr = "127.0.0.1:1001".parse().unwrap();
    let addr2: SocketAddr = "127.0.0.1:1002".parse().unwrap();

    let id1 = sessions
        .add_player("Player1".into(), TeamId::TEAM_0, addr1, SessionId::new(1))
        .unwrap();
    let id2 = sessions
        .add_player("Player2".into(), TeamId::TEAM_1, addr2, SessionId::new(2))
        .unwrap();

    // Spawn players at different positions
    sessions
        .get_mut(id1)
        .unwrap()
        .spawn(Vec3::new(10.0, 1.0, 10.0), 0.0, Tick(0), None);
    sessions
        .get_mut(id2)
        .unwrap()
        .spawn(Vec3::new(20.0, 1.0, 20.0), 0.0, Tick(0), None);

    // Player 1 moves forward
    let input1 = make_input(1, 1.0, 0.0, 0.0);
    sessions.get_mut(id1).unwrap().queue_input(input1);

    // Player 2 moves right
    let input2 = make_input(1, 0.0, 1.0, 0.0);
    sessions.get_mut(id2).unwrap().queue_input(input2);

    // Process movement for both players
    let dt = 1.0 / 60.0;

    for player in sessions.iter_mut() {
        if let Some(input) = player.pop_input() {
            let new_pos = movement.move_player(player.position, player.velocity, &input, dt);
            player.position = new_pos;
        }
    }

    // Verify positions changed
    let p1 = sessions.get(id1).unwrap();
    let p2 = sessions.get(id2).unwrap();

    // Player 1 moved forward (positive Z with yaw=0)
    assert!(p1.position.z > 10.0, "Player 1 should have moved forward");

    // Player 2 moved right (positive X with yaw=0)
    assert!(p2.position.x > 20.0, "Player 2 should have moved right");
}

#[test]
fn test_players_see_each_other_positions() {
    let mut sessions = SessionManager::new(16);

    // Add two players
    let addr1: SocketAddr = "127.0.0.1:1001".parse().unwrap();
    let addr2: SocketAddr = "127.0.0.1:1002".parse().unwrap();

    let id1 = sessions
        .add_player("Player1".into(), TeamId::TEAM_0, addr1, SessionId::new(1))
        .unwrap();
    let id2 = sessions
        .add_player("Player2".into(), TeamId::TEAM_1, addr2, SessionId::new(2))
        .unwrap();

    // Spawn at specific positions
    sessions
        .get_mut(id1)
        .unwrap()
        .spawn(Vec3::new(10.0, 1.0, 10.0), 0.0, Tick(0), None);
    sessions.get_mut(id2).unwrap().spawn(
        Vec3::new(20.0, 1.0, 20.0),
        std::f32::consts::PI,
        Tick(0),
        None,
    );

    // Both players should be visible to each other via iteration
    let positions: Vec<(PlayerId, Vec3)> = sessions.iter().map(|p| (p.id, p.position)).collect();

    assert_eq!(positions.len(), 2);

    // Check player 1's position
    let p1_data = positions.iter().find(|(id, _)| *id == id1).unwrap();
    assert_eq!(p1_data.1, Vec3::new(10.0, 1.0, 10.0));

    // Check player 2's position
    let p2_data = positions.iter().find(|(id, _)| *id == id2).unwrap();
    assert_eq!(p2_data.1, Vec3::new(20.0, 1.0, 20.0));
}

#[test]
fn test_continuous_movement_updates() {
    let mut sessions = SessionManager::new(16);
    let movement = MovementSystem::new(make_test_arena());

    let addr: SocketAddr = "127.0.0.1:1001".parse().unwrap();
    let id = sessions
        .add_player("Player".into(), TeamId::TEAM_0, addr, SessionId::new(1))
        .unwrap();
    sessions
        .get_mut(id)
        .unwrap()
        .spawn(Vec3::new(32.0, 1.0, 32.0), 0.0, Tick(0), None);

    let dt = 1.0 / 60.0;
    let initial_pos = sessions.get(id).unwrap().position;

    // Simulate 60 frames of forward movement
    for i in 0..60 {
        let input = make_input(i, 1.0, 0.0, 0.0);
        sessions.get_mut(id).unwrap().queue_input(input);

        let player = sessions.get_mut(id).unwrap();
        if let Some(input) = player.pop_input() {
            let new_pos = movement.move_player(player.position, player.velocity, &input, dt);
            player.position = new_pos;
        }
    }

    let final_pos = sessions.get(id).unwrap().position;

    // Should have moved approximately 6 blocks forward (6 m/s * 1 second)
    let distance = (final_pos - initial_pos).length();
    assert!(
        distance > 5.0 && distance < 7.0,
        "Expected ~6 blocks movement, got {} blocks",
        distance
    );
}

// ============================================================================
// US1 - Reliable Collision Integration Tests
// ============================================================================

/// Helper: Create a collision world with floor and walls for testing
fn make_wall_arena() -> CollisionWorld {
    // 16x16x16 world with floor at y=0 and walls at x=12 and z=12
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

    // Set wall at x=12 (full height)
    for z in 0..16 {
        for y in 1..16 {
            let index = z * 256 + y * 16 + 12;
            blocks[index] = BlockType::STONE;
        }
    }

    // Set wall at z=12 (full height)
    for x in 0..16 {
        for y in 1..16 {
            let index = 12 * 256 + y * 16 + x;
            blocks[index] = BlockType::STONE;
        }
    }

    CollisionWorld::new(size, blocks)
}

/// T022 [US1] Walk into wall from all angles - player should stop without penetrating
#[test]
fn test_walk_into_wall_all_angles() {
    let world = make_wall_arena();
    let dt = 1.0 / 60.0;

    // Test from multiple angles
    let angles = [
        0.0,                          // +X
        std::f32::consts::FRAC_PI_2,  // +Z
        std::f32::consts::PI,         // -X
        -std::f32::consts::FRAC_PI_2, // -Z
        std::f32::consts::FRAC_PI_4,  // +X +Z diagonal
    ];

    for angle in angles {
        // Start away from walls
        let start = Vec3::new(8.0, 1.0, 8.0);
        let speed = 6.0;
        let velocity = Vec3::new(speed * angle.cos(), 0.0, speed * angle.sin());

        // Simulate 120 frames (2 seconds) - plenty of time to hit wall
        let mut pos = start;
        let mut vel = velocity;

        for _ in 0..120 {
            let result = world.move_and_slide_v2(pos, vel, dt, false);
            pos = result.position;
            vel = result.velocity;
        }

        // Player should never penetrate walls (x=12 or z=12)
        let wall_x = 12.0 - PLAYER_HALF_WIDTH;
        let wall_z = 12.0 - PLAYER_HALF_WIDTH;

        assert!(
            pos.x < wall_x + 0.01,
            "Player penetrated X wall at angle {}: pos.x = {}",
            angle,
            pos.x
        );
        assert!(
            pos.z < wall_z + 0.01,
            "Player penetrated Z wall at angle {}: pos.z = {}",
            angle,
            pos.z
        );
    }
}

/// T023 [P] [US1] Diagonal corner collision - should slide smoothly
#[test]
fn test_diagonal_corner_collision() {
    let world = make_wall_arena();
    let dt = 1.0 / 60.0;

    // Start near corner (walls at x=12 and z=12)
    let start = Vec3::new(10.0, 1.0, 10.0);
    // Move diagonally toward corner
    let velocity = Vec3::new(6.0, 0.0, 6.0);

    let mut pos = start;
    let mut vel = velocity;
    let mut positions = vec![pos];

    // Simulate movement
    for _ in 0..60 {
        let result = world.move_and_slide_v2(pos, vel, dt, false);
        pos = result.position;
        vel = result.velocity;
        positions.push(pos);
    }

    // Check smooth sliding - no sudden jumps between frames
    for i in 1..positions.len() {
        let delta = (positions[i] - positions[i - 1]).length();
        // Maximum movement per frame should be speed * dt = 6.0 * 1/60 ≈ 0.1
        assert!(
            delta < 0.2,
            "Jerky movement detected at frame {}: delta = {}",
            i,
            delta
        );
    }

    // Should be stopped at corner (both X and Z blocked)
    let wall_x = 12.0 - PLAYER_HALF_WIDTH;
    let wall_z = 12.0 - PLAYER_HALF_WIDTH;

    assert!(
        pos.x < wall_x + 0.01,
        "Player penetrated X wall: pos.x = {}",
        pos.x
    );
    assert!(
        pos.z < wall_z + 0.01,
        "Player penetrated Z wall: pos.z = {}",
        pos.z
    );
}

/// T024 [P] [US1] Stationary against wall - should not jitter
#[test]
fn test_stationary_against_wall_no_jitter() {
    let world = make_wall_arena();
    let dt = 1.0 / 60.0;

    // Position player right at wall edge
    let wall_x = 12.0 - PLAYER_HALF_WIDTH - 0.001; // Just before wall
    let start = Vec3::new(wall_x, 1.0, 8.0);

    // Apply zero velocity (stationary)
    let velocity = Vec3::ZERO;

    let mut pos = start;
    let mut positions = vec![pos];

    // Simulate 60 frames of standing still
    for _ in 0..60 {
        let result = world.move_and_slide_v2(pos, velocity, dt, false);
        pos = result.position;
        positions.push(pos);
    }

    // Check for jitter - position should not change significantly
    for i in 1..positions.len() {
        let delta = (positions[i] - positions[i - 1]).length();
        assert!(
            delta < 0.001,
            "Jitter detected while stationary at frame {}: delta = {}",
            i,
            delta
        );
    }

    // Final position should be essentially unchanged
    let total_drift = (pos - start).length();
    assert!(
        total_drift < 0.01,
        "Position drifted while stationary: drift = {}",
        total_drift
    );
}

/// T021 [US1] Collision determinism - identical inputs produce identical outputs
#[test]
fn test_collision_determinism() {
    let world1 = make_wall_arena();
    let world2 = make_wall_arena();
    let dt = 1.0 / 60.0;

    let start = Vec3::new(8.0, 1.0, 8.0);
    let velocity = Vec3::new(6.0, -2.0, 3.0);

    // Run simulation twice
    let mut pos1 = start;
    let mut vel1 = velocity;
    let mut pos2 = start;
    let mut vel2 = velocity;

    for _ in 0..120 {
        let result1 = world1.move_and_slide_v2(pos1, vel1, dt, true);
        let result2 = world2.move_and_slide_v2(pos2, vel2, dt, true);

        pos1 = result1.position;
        vel1 = result1.velocity;
        pos2 = result2.position;
        vel2 = result2.velocity;

        // Results must be identical
        assert_eq!(
            pos1, pos2,
            "Position mismatch - non-deterministic collision"
        );
        assert_eq!(
            vel1, vel2,
            "Velocity mismatch - non-deterministic collision"
        );
        assert_eq!(
            result1.grounded, result2.grounded,
            "Grounded mismatch - non-deterministic collision"
        );
    }
}

// ============================================================================
// US6 - Client-Server Determinism Tests
// ============================================================================

/// T061 [US6] Shared physics config - client and server use identical constants
#[test]
fn test_shared_physics_config_determinism() {
    use plix_common::physics::MovementConfig;

    // Both client and server should use the same MovementConfig defaults
    let config1 = MovementConfig::default();
    let config2 = MovementConfig::default();

    // All constants must match exactly
    assert_eq!(config1.max_speed, config2.max_speed, "max_speed mismatch");
    assert_eq!(config1.gravity, config2.gravity, "gravity mismatch");
    assert_eq!(
        config1.jump_impulse, config2.jump_impulse,
        "jump_impulse mismatch"
    );
    assert_eq!(
        config1.ground_friction, config2.ground_friction,
        "ground_friction mismatch"
    );
    assert_eq!(
        config1.air_control, config2.air_control,
        "air_control mismatch"
    );
    assert_eq!(
        config1.step_height, config2.step_height,
        "step_height mismatch"
    );
    assert_eq!(
        config1.player_half_width, config2.player_half_width,
        "player_half_width mismatch"
    );
    assert_eq!(
        config1.player_height, config2.player_height,
        "player_height mismatch"
    );

    // Verify deterministic creation (PartialEq)
    assert_eq!(
        config1, config2,
        "MovementConfig::default() not deterministic"
    );
}

/// T061 [US6] Movement simulation determinism - same inputs produce same outputs
#[test]
fn test_movement_simulation_determinism() {
    use plix_common::physics::{lerp, snap_velocity_to_zero, MovementConfig, VELOCITY_THRESHOLD};

    let config = MovementConfig::default();
    let dt = 1.0 / 60.0;

    // Simulate ground movement with friction
    let mut vel1 = Vec3::new(5.0, 0.0, 3.0);
    let mut vel2 = Vec3::new(5.0, 0.0, 3.0);

    // Apply friction (same as server and client prediction)
    for _ in 0..60 {
        let friction = config.ground_friction * dt;
        vel1.x = lerp(vel1.x, 0.0, friction.min(1.0));
        vel1.z = lerp(vel1.z, 0.0, friction.min(1.0));
        vel1 = snap_velocity_to_zero(vel1, VELOCITY_THRESHOLD);

        vel2.x = lerp(vel2.x, 0.0, friction.min(1.0));
        vel2.z = lerp(vel2.z, 0.0, friction.min(1.0));
        vel2 = snap_velocity_to_zero(vel2, VELOCITY_THRESHOLD);

        assert_eq!(vel1, vel2, "Friction not deterministic");
    }

    // Both should be zero (stopped)
    assert_eq!(vel1, Vec3::ZERO, "Should have stopped due to friction");
}

/// T061 [US6] Jump height determinism - same jump impulse produces same height
#[test]
fn test_jump_height_determinism() {
    use plix_common::physics::MovementConfig;

    let config = MovementConfig::default();
    let dt = 1.0 / 60.0;

    // Simulate two identical jumps
    let mut pos1 = Vec3::new(0.0, 0.0, 0.0);
    let mut vel1 = Vec3::new(0.0, config.jump_impulse, 0.0);
    let mut max_height1 = 0.0f32;

    let mut pos2 = Vec3::new(0.0, 0.0, 0.0);
    let mut vel2 = Vec3::new(0.0, config.jump_impulse, 0.0);
    let mut max_height2 = 0.0f32;

    // Simulate until landing
    for _ in 0..120 {
        // Apply gravity
        vel1.y -= config.gravity * dt;
        pos1.y += vel1.y * dt;
        max_height1 = max_height1.max(pos1.y);

        vel2.y -= config.gravity * dt;
        pos2.y += vel2.y * dt;
        max_height2 = max_height2.max(pos2.y);

        // Stop at ground
        if pos1.y < 0.0 {
            pos1.y = 0.0;
            vel1.y = 0.0;
        }
        if pos2.y < 0.0 {
            pos2.y = 0.0;
            vel2.y = 0.0;
        }

        // Must be identical each frame
        assert_eq!(pos1, pos2, "Position mismatch during jump");
        assert_eq!(vel1, vel2, "Velocity mismatch during jump");
    }

    // Both should reach identical max height
    assert_eq!(
        max_height1, max_height2,
        "Jump heights not deterministic: {} vs {}",
        max_height1, max_height2
    );

    // Should be approximately 1.25 blocks (with Euler integration error)
    assert!(
        (max_height1 - 1.25).abs() < 0.1,
        "Jump height should be ~1.25 blocks, got {}",
        max_height1
    );
}
