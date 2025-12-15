//! Integration test: Two clients see each other moving
//! T070 [US2]

use std::net::SocketAddr;

use plix_arena::format::{Arena, ArenaMetadata, BlockDefinitions, LoadedArena};
use plix_common::math::Vec3;
use plix_common::protocol::PlayerInput;
use plix_common::time::Tick;
use plix_common::types::{InputSeq, PlayerId, TeamId};
use plix_server::session::{ServerPlayer, SessionManager};
use plix_server::sim::movement::MovementSystem;

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
        .add_player("Player1".into(), TeamId::TEAM_0, addr1)
        .unwrap();
    let id2 = sessions
        .add_player("Player2".into(), TeamId::TEAM_1, addr2)
        .unwrap();

    // Spawn players at different positions
    sessions
        .get_mut(id1)
        .unwrap()
        .spawn(Vec3::new(10.0, 1.0, 10.0), 0.0);
    sessions
        .get_mut(id2)
        .unwrap()
        .spawn(Vec3::new(20.0, 1.0, 20.0), 0.0);

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
        .add_player("Player1".into(), TeamId::TEAM_0, addr1)
        .unwrap();
    let id2 = sessions
        .add_player("Player2".into(), TeamId::TEAM_1, addr2)
        .unwrap();

    // Spawn at specific positions
    sessions
        .get_mut(id1)
        .unwrap()
        .spawn(Vec3::new(10.0, 1.0, 10.0), 0.0);
    sessions
        .get_mut(id2)
        .unwrap()
        .spawn(Vec3::new(20.0, 1.0, 20.0), std::f32::consts::PI);

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
        .add_player("Player".into(), TeamId::TEAM_0, addr)
        .unwrap();
    sessions
        .get_mut(id)
        .unwrap()
        .spawn(Vec3::new(32.0, 1.0, 32.0), 0.0);

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

    // Should have moved approximately 5 blocks forward (5 b/s * 1 second)
    let distance = (final_pos - initial_pos).length();
    assert!(
        distance > 4.0 && distance < 6.0,
        "Expected ~5 blocks movement, got {} blocks",
        distance
    );
}
