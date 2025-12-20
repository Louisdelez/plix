//! {{mod_name}} - A timers and networking mod for Plix
//!
//! This mod demonstrates:
//! - Setting up periodic timers (intervals)
//! - One-shot timers (timeouts)
//! - Sending messages to players
//! - Broadcasting messages to all players

#![no_std]

extern crate alloc;

use alloc::format;
use plix_mod_sdk::prelude::*;
use plix_mod_sdk_macros::{on_event, plix_mod};

/// Interval for periodic announcements (5 minutes in milliseconds)
const ANNOUNCEMENT_INTERVAL_MS: u64 = 5 * 60 * 1000;

/// Welcome message delay (3 seconds)
const WELCOME_DELAY_MS: u64 = 3000;

#[plix_mod]
struct TimersNet;

#[plix_mod]
impl TimersNet {
    fn init(&self) {
        info("TimersNet mod initialized!");

        // Subscribe to player join events
        if let Err(e) = subscribe(EventType::PlayerJoin) {
            error(&format!("Failed to subscribe to player join: {:?}", e));
        }

        // Subscribe to server start to set up periodic announcements
        if let Err(e) = subscribe(EventType::ServerStart) {
            error(&format!("Failed to subscribe to server start: {:?}", e));
        }
    }

    fn shutdown(&self) {
        info("TimersNet mod shutting down");
    }

    #[on_event("on_server_start")]
    fn handle_server_start(&self, _ctx: &EventContext, _payload: ServerStartPayload) {
        info("Server started, setting up periodic announcements");

        // Set up a periodic announcement every 5 minutes
        match set_interval(ANNOUNCEMENT_INTERVAL_MS) {
            Ok(handle) => {
                info(&format!("Periodic announcement timer set (handle: {})", handle.0));
            }
            Err(e) => {
                error(&format!("Failed to set interval: {:?}", e));
            }
        }
    }

    #[on_event("on_player_join")]
    fn handle_player_join(&self, _ctx: &EventContext, payload: PlayerJoinPayload) {
        let player_id = payload.player_id;
        info(&format!("Player {} joined, scheduling welcome message", player_id));

        // Send an immediate notification
        let join_msg = format!("Player {} has joined the server!", player_id);
        if let Err(e) = broadcast(&join_msg) {
            error(&format!("Failed to broadcast join message: {:?}", e));
        }

        // Schedule a delayed personal welcome message
        match set_timeout(WELCOME_DELAY_MS) {
            Ok(_handle) => {
                // Note: In a real implementation, you'd store the player_id
                // with the timer handle to send the message when the timer fires
                debug(&format!("Welcome timer scheduled for player {}", player_id));
            }
            Err(e) => {
                error(&format!("Failed to set welcome timeout: {:?}", e));
            }
        }
    }

    // Timer callback would be invoked here
    // In a full implementation, timer events would trigger this
    #[allow(dead_code)]
    fn on_timer(&self, _timer_id: u64) {
        // Periodic announcement
        let announcement = "Remember to have fun and play fair!";
        if let Err(e) = broadcast(announcement) {
            error(&format!("Failed to broadcast announcement: {:?}", e));
        }
    }
}
