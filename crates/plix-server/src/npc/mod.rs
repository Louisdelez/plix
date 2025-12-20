//! NPC System (Feature 043, Phase 6)
//!
//! Manages NPC instances, dialogue, and quest interactions.
//!
//! Modules:
//! - `registry`: NPC instance management and lookup
//! - `dialogue`: Dialogue state machine and quest integration

mod registry;
mod dialogue;

pub use registry::NpcRegistry;
pub use dialogue::{DialogueHandler, DialogueSession, DialogueResponse};
