//! {{mod_name}} - A chat filter mod for Plix
//!
//! This mod filters chat messages containing blocked words.

#![no_std]

extern crate alloc;

use alloc::format;
use plix_mod_sdk::prelude::*;
use plix_mod_sdk_macros::{on_event, plix_mod};

/// Blocked words list (customize as needed)
static BLOCKED_WORDS: &[&str] = &["badword", "spam", "banned"];

#[plix_mod]
struct ChatFilter;

#[plix_mod]
impl ChatFilter {
    fn init(&self) {
        info("ChatFilter mod initialized!");

        // Subscribe to chat events
        if let Err(e) = subscribe(EventType::PlayerChat) {
            error(&format!("Failed to subscribe to chat events: {:?}", e));
        }
    }

    fn shutdown(&self) {
        info("ChatFilter mod shutting down");
    }

    #[on_event("on_player_chat")]
    fn handle_chat(&self, ctx: &EventContext, payload: PlayerChatPayload) {
        let text_lower = payload.text.to_lowercase();

        for blocked in BLOCKED_WORDS {
            if text_lower.contains(blocked) {
                info(&format!(
                    "Blocked message from player {}: contained '{}'",
                    payload.player_id, blocked
                ));

                // Cancel the chat message
                if let Err(e) = ctx.cancel() {
                    error(&format!("Failed to cancel chat: {:?}", e));
                }
                return;
            }
        }
    }
}
