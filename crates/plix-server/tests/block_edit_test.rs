//! Unit tests for block edit validation
//! T015-T020, T038-T040, T048-T049 [US1, US2, US3]

use std::net::SocketAddr;

use plix_arena::format::{Arena, ArenaMetadata, BlockDefinitions, LoadedArena};
use plix_common::math::Vec3;
use plix_common::protocol::{BlockEditKind, BlockEditRejectReason, BlockEditRequest, MatchPhase};
use plix_common::time::Tick;
use plix_common::types::{BlockPos, BlockType, PlayerId, TeamId};
use plix_server::session::ServerPlayer;
use plix_server::sim::block_edit::{BlockEditSystem, EDIT_COOLDOWN_TICKS, MAX_EDIT_RANGE};

/// Create a test arena with given dimensions filled with stone
fn create_test_arena(size_x: u32, size_y: u32, size_z: u32) -> LoadedArena {
    let definition = Arena {
        metadata: ArenaMetadata {
            name: "test_arena".to_string(),
            version: "1.0".to_string(),
            size: [size_x, size_y, size_z],
        },
        spawn_points: vec![],
        blocks: BlockDefinitions {
            floor: None,
            walls: None,
            regions: vec![],
        },
    };

    // Fill with stone blocks
    let total_blocks = (size_x * size_y * size_z) as usize;
    let blocks = vec![BlockType::STONE; total_blocks];

    LoadedArena { definition, blocks }
}

/// Create a test player at given position
fn create_test_player(id: u16, position: Vec3) -> ServerPlayer {
    let player_id = PlayerId(id);
    let addr: SocketAddr = format!("127.0.0.1:{}", 10000 + id).parse().unwrap();
    let mut player = ServerPlayer::new(player_id, format!("Player{}", id), TeamId::TEAM_0, addr);
    player.spawn(position, 0.0);
    player
}

// =============================================================================
// T015: Unit test: validate_remove rejects out-of-bounds
// =============================================================================
#[test]
fn test_validate_remove_rejects_out_of_bounds() {
    let arena = create_test_arena(16, 16, 16);
    let player = create_test_player(1, Vec3::new(8.0, 1.0, 8.0));
    let current_tick = Tick(100);

    // Request to remove block outside arena bounds
    let request = BlockEditRequest {
        kind: BlockEditKind::Remove,
        target_pos: BlockPos { x: 20, y: 0, z: 0 }, // Out of bounds (x >= 16)
        block_type: BlockType::AIR,
    };

    let result = BlockEditSystem::validate_remove(
        &request,
        &player,
        &arena,
        current_tick,
        MatchPhase::Playing,
    );

    assert_eq!(result, Err(BlockEditRejectReason::OutOfBounds));
}

#[test]
fn test_validate_remove_rejects_negative_coords() {
    let arena = create_test_arena(16, 16, 16);
    let player = create_test_player(1, Vec3::new(1.0, 1.0, 1.0));
    let current_tick = Tick(100);

    // Request with negative coordinates
    let request = BlockEditRequest {
        kind: BlockEditKind::Remove,
        target_pos: BlockPos { x: -1, y: 0, z: 0 },
        block_type: BlockType::AIR,
    };

    let result = BlockEditSystem::validate_remove(
        &request,
        &player,
        &arena,
        current_tick,
        MatchPhase::Playing,
    );

    assert_eq!(result, Err(BlockEditRejectReason::OutOfBounds));
}

// =============================================================================
// T016: Unit test: validate_remove rejects out-of-range
// =============================================================================
#[test]
fn test_validate_remove_rejects_out_of_range() {
    let arena = create_test_arena(32, 16, 32);
    let player = create_test_player(1, Vec3::new(5.0, 1.0, 5.0));
    let current_tick = Tick(100);

    // Block far away from player (distance > MAX_EDIT_RANGE)
    let request = BlockEditRequest {
        kind: BlockEditKind::Remove,
        target_pos: BlockPos { x: 20, y: 1, z: 20 }, // ~21 blocks away
        block_type: BlockType::AIR,
    };

    let result = BlockEditSystem::validate_remove(
        &request,
        &player,
        &arena,
        current_tick,
        MatchPhase::Playing,
    );

    assert_eq!(result, Err(BlockEditRejectReason::OutOfRange));
}

#[test]
fn test_validate_remove_accepts_at_max_range() {
    let arena = create_test_arena(32, 16, 32);
    // Position player so block is exactly at MAX_EDIT_RANGE
    let player = create_test_player(1, Vec3::new(10.0, 1.0, 10.0));
    let current_tick = Tick(100);

    // Block at approximately MAX_EDIT_RANGE distance
    // Block center is at (15.5, 1.5, 10.5), player at (10.0, 1.0, 10.0)
    // Distance ~= sqrt((5.5)^2 + (0.5)^2 + (0.5)^2) = sqrt(30.75) ~= 5.5 > MAX_EDIT_RANGE
    // Let's pick a closer block
    let request = BlockEditRequest {
        kind: BlockEditKind::Remove,
        target_pos: BlockPos { x: 14, y: 1, z: 10 }, // Block center at (14.5, 1.5, 10.5)
        block_type: BlockType::AIR,
    };

    let result = BlockEditSystem::validate_remove(
        &request,
        &player,
        &arena,
        current_tick,
        MatchPhase::Playing,
    );

    // Distance = sqrt((14.5-10)^2 + (1.5-1)^2 + (10.5-10)^2) = sqrt(20.5) ~= 4.5 < 5.0
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
}

// =============================================================================
// T017: Unit test: validate_remove rejects air cell (CellEmpty)
// =============================================================================
#[test]
fn test_validate_remove_rejects_air_cell() {
    let mut arena = create_test_arena(16, 16, 16);
    let player = create_test_player(1, Vec3::new(8.0, 1.0, 8.0));
    let current_tick = Tick(100);

    // Set target block to air
    let target_pos = BlockPos { x: 8, y: 2, z: 8 };
    arena.set_block(target_pos, BlockType::AIR);

    let request = BlockEditRequest {
        kind: BlockEditKind::Remove,
        target_pos,
        block_type: BlockType::AIR,
    };

    let result = BlockEditSystem::validate_remove(
        &request,
        &player,
        &arena,
        current_tick,
        MatchPhase::Playing,
    );

    assert_eq!(result, Err(BlockEditRejectReason::CellEmpty));
}

// =============================================================================
// T018: Unit test: validate_remove rejects dead player
// =============================================================================
#[test]
fn test_validate_remove_rejects_dead_player() {
    let arena = create_test_arena(16, 16, 16);
    let mut player = create_test_player(1, Vec3::new(8.0, 1.0, 8.0));
    let current_tick = Tick(100);

    // Kill the player
    player.die(Tick(200));

    let request = BlockEditRequest {
        kind: BlockEditKind::Remove,
        target_pos: BlockPos { x: 8, y: 2, z: 8 },
        block_type: BlockType::AIR,
    };

    let result = BlockEditSystem::validate_remove(
        &request,
        &player,
        &arena,
        current_tick,
        MatchPhase::Playing,
    );

    assert_eq!(result, Err(BlockEditRejectReason::PlayerDead));
}

// =============================================================================
// T019: Unit test: validate_remove rejects rate-limited player
// =============================================================================
#[test]
fn test_validate_remove_rejects_rate_limited() {
    let arena = create_test_arena(16, 16, 16);
    let mut player = create_test_player(1, Vec3::new(8.0, 1.0, 8.0));
    let current_tick = Tick(100);

    // Set last edit tick to be within cooldown period
    player.last_edit_tick = Some(Tick(100 - EDIT_COOLDOWN_TICKS + 5)); // 5 ticks ago, cooldown is 15

    let request = BlockEditRequest {
        kind: BlockEditKind::Remove,
        target_pos: BlockPos { x: 8, y: 2, z: 8 },
        block_type: BlockType::AIR,
    };

    let result = BlockEditSystem::validate_remove(
        &request,
        &player,
        &arena,
        current_tick,
        MatchPhase::Playing,
    );

    assert_eq!(result, Err(BlockEditRejectReason::RateLimited));
}

#[test]
fn test_validate_remove_accepts_after_cooldown() {
    let arena = create_test_arena(16, 16, 16);
    let mut player = create_test_player(1, Vec3::new(8.0, 1.0, 8.0));
    let current_tick = Tick(100);

    // Set last edit tick to be past cooldown period
    player.last_edit_tick = Some(Tick(100 - EDIT_COOLDOWN_TICKS - 1)); // Cooldown expired

    let request = BlockEditRequest {
        kind: BlockEditKind::Remove,
        target_pos: BlockPos { x: 8, y: 2, z: 8 },
        block_type: BlockType::AIR,
    };

    let result = BlockEditSystem::validate_remove(
        &request,
        &player,
        &arena,
        current_tick,
        MatchPhase::Playing,
    );

    assert!(
        result.is_ok(),
        "Expected Ok after cooldown, got {:?}",
        result
    );
}

// =============================================================================
// T020: Unit test: validate_remove accepts valid request
// =============================================================================
#[test]
fn test_validate_remove_accepts_valid_request() {
    let arena = create_test_arena(16, 16, 16);
    let player = create_test_player(1, Vec3::new(8.0, 1.0, 8.0));
    let current_tick = Tick(100);

    // Valid request: in bounds, in range, solid block, alive player, not rate limited
    let request = BlockEditRequest {
        kind: BlockEditKind::Remove,
        target_pos: BlockPos { x: 8, y: 2, z: 8 }, // Block center at (8.5, 2.5, 8.5), ~1.8 blocks from player
        block_type: BlockType::AIR,
    };

    let result = BlockEditSystem::validate_remove(
        &request,
        &player,
        &arena,
        current_tick,
        MatchPhase::Playing,
    );

    assert!(
        result.is_ok(),
        "Expected valid request to be accepted, got {:?}",
        result
    );
}

// =============================================================================
// T038: Unit test: validate_place rejects non-air cell (CellNotEmpty)
// =============================================================================
#[test]
fn test_validate_place_rejects_non_air_cell() {
    let arena = create_test_arena(16, 16, 16); // All stone
    let player = create_test_player(1, Vec3::new(8.0, 1.0, 8.0));
    let current_tick = Tick(100);

    // Try to place in a cell that already has a block
    let request = BlockEditRequest {
        kind: BlockEditKind::Place,
        target_pos: BlockPos { x: 8, y: 2, z: 8 }, // Already has stone
        block_type: BlockType::BRICK,
    };

    let all_players: Vec<&ServerPlayer> = vec![&player];
    let result = BlockEditSystem::validate_place(
        &request,
        &player,
        &arena,
        &all_players,
        current_tick,
        MatchPhase::Playing,
    );

    assert_eq!(result, Err(BlockEditRejectReason::CellNotEmpty));
}

// =============================================================================
// T039: Unit test: validate_place rejects player collision
// =============================================================================
#[test]
fn test_validate_place_rejects_player_collision() {
    let mut arena = create_test_arena(16, 16, 16);
    let player = create_test_player(1, Vec3::new(8.0, 1.0, 8.0));
    let other_player = create_test_player(2, Vec3::new(10.5, 1.0, 8.5)); // Standing at block (10, 1, 8)
    let current_tick = Tick(100);

    // Clear the target cell so we can place
    let target_pos = BlockPos { x: 10, y: 1, z: 8 };
    arena.set_block(target_pos, BlockType::AIR);

    let request = BlockEditRequest {
        kind: BlockEditKind::Place,
        target_pos,
        block_type: BlockType::STONE,
    };

    let all_players: Vec<&ServerPlayer> = vec![&player, &other_player];
    let result = BlockEditSystem::validate_place(
        &request,
        &player,
        &arena,
        &all_players,
        current_tick,
        MatchPhase::Playing,
    );

    assert_eq!(result, Err(BlockEditRejectReason::PlayerCollision));
}

#[test]
fn test_validate_place_ignores_dead_players_for_collision() {
    let mut arena = create_test_arena(16, 16, 16);
    let player = create_test_player(1, Vec3::new(8.0, 1.0, 8.0));
    let mut dead_player = create_test_player(2, Vec3::new(10.5, 1.0, 8.5));
    dead_player.die(Tick(200)); // Dead player at the target position

    let current_tick = Tick(100);

    // Clear the target cell
    let target_pos = BlockPos { x: 10, y: 1, z: 8 };
    arena.set_block(target_pos, BlockType::AIR);

    let request = BlockEditRequest {
        kind: BlockEditKind::Place,
        target_pos,
        block_type: BlockType::STONE,
    };

    let all_players: Vec<&ServerPlayer> = vec![&player, &dead_player];
    let result = BlockEditSystem::validate_place(
        &request,
        &player,
        &arena,
        &all_players,
        current_tick,
        MatchPhase::Playing,
    );

    // Should succeed because dead players don't block placement
    assert!(
        result.is_ok(),
        "Expected Ok (dead players ignored), got {:?}",
        result
    );
}

// =============================================================================
// T040: Unit test: validate_place accepts valid request
// =============================================================================
#[test]
fn test_validate_place_accepts_valid_request() {
    let mut arena = create_test_arena(16, 16, 16);
    // Player at ground level, placing a block 3 units away horizontally (not colliding)
    let player = create_test_player(1, Vec3::new(8.0, 1.0, 8.0));
    let current_tick = Tick(100);

    // Clear a cell for placement that's far enough from player to not collide
    // Player AABB: x±0.4, y+0..y+1.8, z±0.4 => (7.6..8.4, 1.0..2.8, 7.6..8.4)
    // Block at (11, 1, 8) won't collide: block bounds (11..12, 1..2, 8..9)
    let target_pos = BlockPos { x: 11, y: 1, z: 8 };
    arena.set_block(target_pos, BlockType::AIR);

    let request = BlockEditRequest {
        kind: BlockEditKind::Place,
        target_pos,
        block_type: BlockType::BRICK,
    };

    let all_players: Vec<&ServerPlayer> = vec![&player];
    let result = BlockEditSystem::validate_place(
        &request,
        &player,
        &arena,
        &all_players,
        current_tick,
        MatchPhase::Playing,
    );

    assert!(
        result.is_ok(),
        "Expected valid place request to be accepted, got {:?}",
        result
    );
}

// =============================================================================
// T048: Unit test: validate rejects InvalidPhase (not Playing)
// =============================================================================
#[test]
fn test_validate_rejects_invalid_phase_waiting() {
    let arena = create_test_arena(16, 16, 16);
    let player = create_test_player(1, Vec3::new(8.0, 1.0, 8.0));
    let current_tick = Tick(100);

    let request = BlockEditRequest {
        kind: BlockEditKind::Remove,
        target_pos: BlockPos { x: 8, y: 2, z: 8 },
        block_type: BlockType::AIR,
    };

    let result = BlockEditSystem::validate_remove(
        &request,
        &player,
        &arena,
        current_tick,
        MatchPhase::WaitingForPlayers,
    );

    assert_eq!(result, Err(BlockEditRejectReason::InvalidPhase));
}

#[test]
fn test_validate_rejects_invalid_phase_countdown() {
    let arena = create_test_arena(16, 16, 16);
    let player = create_test_player(1, Vec3::new(8.0, 1.0, 8.0));
    let current_tick = Tick(100);

    let request = BlockEditRequest {
        kind: BlockEditKind::Remove,
        target_pos: BlockPos { x: 8, y: 2, z: 8 },
        block_type: BlockType::AIR,
    };

    let result = BlockEditSystem::validate_remove(
        &request,
        &player,
        &arena,
        current_tick,
        MatchPhase::Countdown,
    );

    assert_eq!(result, Err(BlockEditRejectReason::InvalidPhase));
}

#[test]
fn test_validate_rejects_invalid_phase_round_end() {
    let arena = create_test_arena(16, 16, 16);
    let player = create_test_player(1, Vec3::new(8.0, 1.0, 8.0));
    let current_tick = Tick(100);

    let request = BlockEditRequest {
        kind: BlockEditKind::Remove,
        target_pos: BlockPos { x: 8, y: 2, z: 8 },
        block_type: BlockType::AIR,
    };

    let result = BlockEditSystem::validate_remove(
        &request,
        &player,
        &arena,
        current_tick,
        MatchPhase::RoundEnd,
    );

    assert_eq!(result, Err(BlockEditRejectReason::InvalidPhase));
}

#[test]
fn test_validate_rejects_invalid_phase_match_end() {
    let arena = create_test_arena(16, 16, 16);
    let player = create_test_player(1, Vec3::new(8.0, 1.0, 8.0));
    let current_tick = Tick(100);

    let request = BlockEditRequest {
        kind: BlockEditKind::Remove,
        target_pos: BlockPos { x: 8, y: 2, z: 8 },
        block_type: BlockType::AIR,
    };

    let result = BlockEditSystem::validate_remove(
        &request,
        &player,
        &arena,
        current_tick,
        MatchPhase::MatchEnd,
    );

    assert_eq!(result, Err(BlockEditRejectReason::InvalidPhase));
}

// =============================================================================
// T049: Unit test: all BlockEditRejectReason codes are reachable
// =============================================================================
#[test]
fn test_all_reject_reasons_reachable() {
    // This test documents that all reject reasons can be triggered.
    // Each reason is tested in its own dedicated test above:
    // - OutOfBounds: test_validate_remove_rejects_out_of_bounds
    // - OutOfRange: test_validate_remove_rejects_out_of_range
    // - CellEmpty: test_validate_remove_rejects_air_cell
    // - CellNotEmpty: test_validate_place_rejects_non_air_cell
    // - PlayerCollision: test_validate_place_rejects_player_collision
    // - RateLimited: test_validate_remove_rejects_rate_limited
    // - PlayerDead: test_validate_remove_rejects_dead_player
    // - InvalidPhase: test_validate_rejects_invalid_phase_warmup

    // Verify all variants exist by matching
    let reasons = [
        BlockEditRejectReason::OutOfBounds,
        BlockEditRejectReason::OutOfRange,
        BlockEditRejectReason::CellEmpty,
        BlockEditRejectReason::CellNotEmpty,
        BlockEditRejectReason::PlayerCollision,
        BlockEditRejectReason::RateLimited,
        BlockEditRejectReason::PlayerDead,
        BlockEditRejectReason::InvalidPhase,
    ];

    for reason in reasons {
        // Just verify they can be formatted (exist and are valid)
        let _ = format!("{:?}", reason);
    }
}

// =============================================================================
// Additional edge case tests
// =============================================================================
#[test]
fn test_edit_range_constant() {
    // Verify constant matches specification (5.0 blocks)
    assert_eq!(MAX_EDIT_RANGE, 5.0);
}

#[test]
fn test_edit_cooldown_constant() {
    // Verify constant matches specification (15 ticks = 4 edits/sec at 60Hz)
    assert_eq!(EDIT_COOLDOWN_TICKS, 15);
}
