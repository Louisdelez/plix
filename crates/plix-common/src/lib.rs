//! Plix Common - Shared types, protocol definitions, and math utilities
//!
//! This crate provides the foundational types used across all Plix components:
//! - Math types (Vec3, Rotation, AABB)
//! - Identifiers (PlayerId, EntityId, Tick, InputSeq)
//! - Protocol messages (ClientMessage, ServerMessage)
//! - Time utilities

pub mod math;
pub mod protocol;
pub mod time;
pub mod types;

pub use math::{Rotation, Vec3, AABB};
pub use protocol::{
    ClientMessage, GameEvent, MatchState, PlayerSnapshot, ServerMessage, WorldSnapshot,
};
pub use time::Tick;
pub use types::{BlockPos, BlockType, EntityId, InputSeq, PlayerId, TeamId};
