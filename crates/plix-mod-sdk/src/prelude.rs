//! Prelude module - re-exports commonly used types and functions
//!
//! Import with `use plix_mod_sdk::prelude::*;` for convenience.

// Version constants
pub use crate::version::{check_compatibility, SDK_ABI_VERSION, SDK_API_VERSION};

// Error types
pub use crate::error::{ErrorCode, ModError, Result as ModResult};

// Logging functions
pub use crate::log::{debug, error, info, trace, warn};

// Capabilities
pub use crate::caps::{has_capability, Capability};

// Events
pub use crate::events::{
    subscribe, BlockBrokenPayload, BlockPlacedPayload, EntityDespawnedPayload,
    EntitySpawnedPayload, EventContext, EventType, LeaveReason, PlayerChatPayload,
    PlayerJoinPayload, PlayerLeavePayload, ServerStartPayload, ServerStopPayload,
};

// World API
pub use crate::world::{get_block, query_aabb, raycast, set_block, BlockFace, RaycastHit};

// Entity API
pub use crate::entities::{apply_damage, apply_impulse, get_transform, EntityHandle, Transform};

// Network API
pub use crate::net::{broadcast, send, MessageTarget};

// Timers API
pub use crate::timers::{clear_timer, set_interval, set_timeout, TimerHandle};

// Math types from glam
pub use glam::{IVec3, Quat, Vec3};
