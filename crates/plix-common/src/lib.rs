//! Plix Common - Shared types, protocol definitions, and math utilities
//!
//! This crate provides the foundational types used across all Plix components:
//! - Math types (Vec3, Rotation, AABB)
//! - Identifiers (PlayerId, EntityId, Tick, InputSeq)
//! - Protocol messages (ClientMessage, ServerMessage)
//! - Physics (MovementConfig, MovementState)
//! - Combat (CombatConfig)
//! - Metrics (RollingWindow, Stats)
//! - Time utilities

pub mod chunk;
pub mod combat;
pub mod math;
pub mod metrics;
pub mod physics;
pub mod protocol;
pub mod time;
pub mod types;
pub mod world;
pub mod worldgen;

pub use combat::CombatConfig;
pub use math::{Rotation, Vec3, AABB};
pub use physics::{CollisionResult, MovementConfig, MovementState};
pub use protocol::{
    ClientMessage, GameEvent, MatchState, PlayerSnapshot, ServerMessage, WorldSnapshot,
};
pub use time::Tick;
pub use types::{BlockPos, BlockType, ChunkCoord, EntityId, InputSeq, PlayerId, TeamId};

// Re-export commonly used chunk items
pub use chunk::{Chunk, CHUNK_SIZE};
pub use world::ChunkedWorld;
