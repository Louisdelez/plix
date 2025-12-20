//! Plix Mod SDK - Ergonomic API for WASM mod development
//!
//! This crate provides safe, ergonomic wrappers around the Plix mod runtime ABI.
//! Mod authors use this SDK instead of making raw host function calls.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use plix_mod_sdk::prelude::*;
//!
//! #[plix_mod]
//! struct MyChatFilter;
//!
//! #[plix_mod]
//! impl MyChatFilter {
//!     fn init(&self) {
//!         info!("Chat filter initialized!");
//!         subscribe(EventType::PlayerChat).unwrap();
//!     }
//!
//!     fn shutdown(&self) {
//!         info!("Chat filter shutting down");
//!     }
//!
//!     #[on_event("on_player_chat")]
//!     fn handle_chat(&self, ctx: &EventContext, payload: PlayerChatPayload) {
//!         if payload.text.contains("badword") {
//!             ctx.cancel().unwrap();
//!         }
//!     }
//! }
//! ```
//!
//! # Modules
//!
//! - [`version`] - SDK version constants and compatibility checks
//! - [`error`] - Error types and EMOD error code mapping
//! - [`log`] - Logging macros (info!, warn!, error!, debug!)
//! - [`caps`] - Capability checking
//! - [`events`] - Event subscription and handling
//! - [`world`] - World API (get_block, set_block, raycast, query_aabb)
//! - [`entities`] - Entity API (get_transform, apply_damage, apply_impulse)
//! - [`net`] - Network API (send, broadcast)
//! - [`timers`] - Timer API (set_timeout, set_interval, clear)
//! - [`prelude`] - Common re-exports for convenience

#![no_std]

// We need alloc for Vec, String, etc.
extern crate alloc;

pub mod abi;
pub mod caps;
pub mod codec;
pub mod entities;
pub mod error;
pub mod events;
pub mod log;
pub mod net;
pub mod prelude;
pub mod timers;
pub mod version;
pub mod world;

// Re-export macros when the feature is enabled
// #[cfg(feature = "macros")]
// pub use plix_mod_sdk_macros::{on_event, plix_mod};
