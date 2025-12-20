//! {{mod_name}} - A world query mod for Plix
//!
//! This mod demonstrates how to query the world:
//! - Reading blocks at specific positions
//! - Performing raycasts
//! - Querying entities in an area

#![no_std]

extern crate alloc;

use alloc::format;
use plix_mod_sdk::prelude::*;
use plix_mod_sdk_macros::{on_event, plix_mod};

#[plix_mod]
struct WorldQuery;

#[plix_mod]
impl WorldQuery {
    fn init(&self) {
        info("WorldQuery mod initialized!");

        // Subscribe to player join events
        if let Err(e) = subscribe(EventType::PlayerJoin) {
            error(&format!("Failed to subscribe to player join: {:?}", e));
        }

        // Subscribe to block placed events
        if let Err(e) = subscribe(EventType::BlockPlaced) {
            error(&format!("Failed to subscribe to block placed: {:?}", e));
        }
    }

    fn shutdown(&self) {
        info("WorldQuery mod shutting down");
    }

    #[on_event("on_player_join")]
    fn handle_player_join(&self, _ctx: &EventContext, payload: PlayerJoinPayload) {
        info(&format!("Player {} joined", payload.player_id));

        // Query blocks around spawn (0, 0, 0)
        let spawn = IVec3::new(0, 64, 0);

        match get_block(spawn) {
            Ok(block_id) => {
                info(&format!("Block at spawn: {}", block_id));
            }
            Err(e) => {
                debug(&format!("Could not read spawn block: {:?}", e));
            }
        }
    }

    #[on_event("on_block_placed")]
    fn handle_block_placed(&self, _ctx: &EventContext, payload: BlockPlacedPayload) {
        info(&format!(
            "Block {} placed at ({}, {}, {})",
            payload.block_id, payload.pos.x, payload.pos.y, payload.pos.z
        ));

        // Demonstrate raycast from above the placed block
        let origin = Vec3::new(
            payload.pos.x as f32,
            payload.pos.y as f32 + 10.0,
            payload.pos.z as f32,
        );
        let direction = Vec3::new(0.0, -1.0, 0.0);
        let max_distance = 20.0;

        match raycast(origin, direction, max_distance) {
            Ok(Some(hit)) => {
                info(&format!(
                    "Raycast hit block {} at distance {:.2}",
                    hit.block_id, hit.distance
                ));
            }
            Ok(None) => {
                debug("Raycast hit nothing");
            }
            Err(e) => {
                error(&format!("Raycast failed: {:?}", e));
            }
        }
    }
}
