//! Block edit validation system

use plix_arena::format::LoadedArena;
use plix_common::math::Vec3;
use plix_common::protocol::{BlockEditKind, BlockEditRejectReason, BlockEditRequest, MatchPhase};
use plix_common::time::Tick;
use plix_common::types::{BlockPos, BlockType};

use crate::session::ServerPlayer;

/// Maximum distance from player to target block (in blocks)
pub const MAX_EDIT_RANGE: f32 = 5.0;

/// Cooldown between edits in ticks (15 ticks = 4 edits/sec at 60Hz)
pub const EDIT_COOLDOWN_TICKS: u32 = 15;

/// Block edit validation system
pub struct BlockEditSystem;

impl BlockEditSystem {
    /// Validate a remove block request
    pub fn validate_remove(
        request: &BlockEditRequest,
        player: &ServerPlayer,
        arena: &LoadedArena,
        current_tick: Tick,
        match_phase: MatchPhase,
    ) -> Result<(), BlockEditRejectReason> {
        // Check common validation rules
        Self::validate_common(request.target_pos, player, arena, current_tick, match_phase)?;

        // Check that target cell is not air (can't remove air)
        if arena.get_block_at(request.target_pos) == BlockType::AIR {
            return Err(BlockEditRejectReason::CellEmpty);
        }

        Ok(())
    }

    /// Validate a place block request
    pub fn validate_place(
        request: &BlockEditRequest,
        player: &ServerPlayer,
        arena: &LoadedArena,
        all_players: &[&ServerPlayer],
        current_tick: Tick,
        match_phase: MatchPhase,
    ) -> Result<(), BlockEditRejectReason> {
        // Check common validation rules
        Self::validate_common(request.target_pos, player, arena, current_tick, match_phase)?;

        // Check that target cell is air (can only place in empty cells)
        if arena.get_block_at(request.target_pos) != BlockType::AIR {
            return Err(BlockEditRejectReason::CellNotEmpty);
        }

        // Check that no player would be trapped
        if Self::would_collide_with_player(request.target_pos, all_players) {
            return Err(BlockEditRejectReason::PlayerCollision);
        }

        Ok(())
    }

    /// Validate a block edit request (dispatches to remove or place)
    pub fn validate(
        request: &BlockEditRequest,
        player: &ServerPlayer,
        arena: &LoadedArena,
        all_players: &[&ServerPlayer],
        current_tick: Tick,
        match_phase: MatchPhase,
    ) -> Result<(), BlockEditRejectReason> {
        match request.kind {
            BlockEditKind::Remove => {
                Self::validate_remove(request, player, arena, current_tick, match_phase)
            }
            BlockEditKind::Place => Self::validate_place(
                request,
                player,
                arena,
                all_players,
                current_tick,
                match_phase,
            ),
        }
    }

    /// Common validation checks for all edit types
    fn validate_common(
        target_pos: BlockPos,
        player: &ServerPlayer,
        arena: &LoadedArena,
        current_tick: Tick,
        match_phase: MatchPhase,
    ) -> Result<(), BlockEditRejectReason> {
        // Check match phase
        if match_phase != MatchPhase::Playing {
            return Err(BlockEditRejectReason::InvalidPhase);
        }

        // Check if player is dead
        if player.is_dead {
            return Err(BlockEditRejectReason::PlayerDead);
        }

        // Check bounds
        if !arena.is_in_bounds(target_pos) {
            return Err(BlockEditRejectReason::OutOfBounds);
        }

        // Check range
        if !Self::is_in_range(target_pos, player.position) {
            return Err(BlockEditRejectReason::OutOfRange);
        }

        // Check rate limit
        if Self::is_rate_limited(player, current_tick) {
            return Err(BlockEditRejectReason::RateLimited);
        }

        Ok(())
    }

    /// Check if target is within range of player
    fn is_in_range(target_pos: BlockPos, player_pos: Vec3) -> bool {
        let block_center = target_pos.to_vec3();
        let distance = player_pos.distance(block_center);
        distance <= MAX_EDIT_RANGE
    }

    /// Check if player is rate limited (cooldown not expired)
    fn is_rate_limited(player: &ServerPlayer, current_tick: Tick) -> bool {
        match player.last_edit_tick {
            Some(last_tick) => current_tick.diff(last_tick) < EDIT_COOLDOWN_TICKS as i32,
            None => false, // No previous edit, not rate limited
        }
    }

    /// Check if placing a block at position would collide with any player
    fn would_collide_with_player(pos: BlockPos, players: &[&ServerPlayer]) -> bool {
        // Block occupies pos.x..pos.x+1, pos.y..pos.y+1, pos.z..pos.z+1
        // Player AABB is approximately: x±0.4, y+0..y+1.8, z±0.4
        let bx = pos.x as f32;
        let by = pos.y as f32;
        let bz = pos.z as f32;

        for player in players {
            if player.is_dead {
                continue;
            }

            let p = player.position;

            // Player AABB bounds
            let player_min_x = p.x - 0.4;
            let player_max_x = p.x + 0.4;
            let player_min_y = p.y;
            let player_max_y = p.y + 1.8;
            let player_min_z = p.z - 0.4;
            let player_max_z = p.z + 0.4;

            // Block bounds
            let block_min_x = bx;
            let block_max_x = bx + 1.0;
            let block_min_y = by;
            let block_max_y = by + 1.0;
            let block_min_z = bz;
            let block_max_z = bz + 1.0;

            // Check AABB overlap
            let x_overlap = player_min_x < block_max_x && player_max_x > block_min_x;
            let y_overlap = player_min_y < block_max_y && player_max_y > block_min_y;
            let z_overlap = player_min_z < block_max_z && player_max_z > block_min_z;

            if x_overlap && y_overlap && z_overlap {
                return true;
            }
        }

        false
    }
}
