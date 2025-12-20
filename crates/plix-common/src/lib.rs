//! Plix Common - Shared types, protocol definitions, and math utilities
//!
//! This crate provides the foundational types used across all Plix components:
//! - Math types (Vec3, Rotation, AABB)
//! - Identifiers (PlayerId, EntityId, Tick, InputSeq)
//! - Protocol messages (ClientMessage, ServerMessage)
//! - Physics (MovementConfig, MovementState)
//! - Block Physics (BlockPhysicsConfig, BlockPhysicsEvent, BlockPhysicsQueue)
//! - Combat (CombatConfig)
//! - Metrics (RollingWindow, Stats)
//! - Server Browser (ServerEntry, ServerListResponse)
//! - Chat (ChatMessageKind, ChatMessageData) - Feature 032
//! - Build Info (BuildInfo) - Feature 041
//! - Time utilities

pub mod block_physics;
pub mod build_info;
pub mod chat;
pub mod chunk;
pub mod combat;
pub mod content;
pub mod economy;
pub mod identity;
pub mod inventory;
pub mod limits;
pub mod math;
pub mod metrics;
pub mod migration;
pub mod persist;
pub mod physics;
pub mod protocol;
pub mod server_browser;
pub mod time;
pub mod types;
pub mod version;
pub mod world;
pub mod worldgen;

pub use build_info::BuildInfo;
pub use chat::{ChatMessageData, ChatMessageKind, ChatValidationError};
pub use combat::CombatConfig;
pub use version::{
    ContentSchemaVersion, ModApiVersion, ProtocolVersion, CONTENT_SCHEMA_VERSION,
    PROTOCOL_VERSION,
};
pub use math::{Rotation, Vec3, AABB};
pub use physics::{CollisionResult, MovementConfig, MovementState};
pub use protocol::{
    ClientMessage, GameEvent, InventorySnapshot, MatchState, PlayerSnapshot, ServerMessage,
    SlotSnapshot, WorldSnapshot,
};
pub use time::Tick;
pub use types::{
    BlockPos, BlockType, ChunkCoord, EntityId, GameMode, InputSeq, ItemId, LootEntityId, PlayerId,
    TeamId,
};

// Re-export commonly used chunk items
pub use chunk::{Chunk, CHUNK_SIZE};
pub use world::ChunkedWorld;
