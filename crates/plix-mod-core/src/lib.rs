//! Mod API Core: Stable contract between game engine and mod runtimes
//!
//! This crate provides the host API that mod runtimes (WASM) use to interact
//! with the game engine.
//!
//! # API Stability
//!
//! This crate follows a stability policy for v1.x:
//!
//! - **Stable**: Guaranteed for the v1.x lifetime. No breaking changes.
//! - **Experimental**: May change in minor versions. Use at your own risk.
//! - **Deprecated**: Will be removed in v2.0. Migration path provided.
//!
//! ## Stable APIs (v1.0+)
//!
//! The following are stable and guaranteed for all v1.x releases:
//!
//! - [`api::EntityApi`] - Entity queries and access
//! - [`api::NetApi`] - Network messaging
//! - [`api::TimerApi`] - Timer scheduling
//! - [`events::EventBus`] - Event subscription
//! - [`events::GameEvent`] - All event types
//! - [`capabilities::Capability`] - Capability system
//! - [`manifest::ModManifest`] - Mod manifest format
//! - [`registry::ModContext`] - Mod execution context
//! - [`MOD_API_VERSION`] - Version constant
//! - [`is_mod_compatible`] - Compatibility check
//!
//! ## Experimental APIs
//!
//! The following may change in minor releases:
//!
//! - [`api::WorldApi`] - Direct world modification (block placement)
//! - HTTP/external networking (planned, not yet implemented)
//!
//! ## Features
//!
//! - Event subscription and dispatch
//! - Entity access with capability checks
//! - Network messaging with rate limiting
//! - Timer scheduling with bounds
//!
//! # Architecture
//!
//! The `ModHost` trait defines the interface that mod runtimes will call.
//! The game engine implements this trait to provide actual functionality.

pub mod api;
pub mod capabilities;
pub mod errors;
pub mod events;
pub mod manifest;
pub mod observability;
pub mod registry;
pub mod stability;

// Re-exports for convenience
pub use capabilities::Capability;
pub use errors::{ErrorCode, ModApiError};
pub use events::{EventBus, EventType, GameEvent};
pub use manifest::ModManifest;
pub use registry::{ModContext, ModRegistry, ModState};
pub use stability::{Stability, STABLE, EXPERIMENTAL, DEPRECATED};

/// Current engine API version (MVP = 1)
pub const ENGINE_API_VERSION: u32 = 1;

/// Engine version string
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Mod API version for compatibility checking.
///
/// This version follows semantic versioning:
/// - Major: Breaking API changes (mods must be recompiled)
/// - Minor: Additive API changes (backward compatible)
///
/// Mods built for v1.0 will work on any v1.x engine.
/// Mods built for v1.x will NOT work on v2.0+ engine.
pub const MOD_API_VERSION: (u8, u8) = (1, 0);

/// Check if a mod with the given required version can run on this engine.
///
/// Returns true if the mod is compatible with the current engine API version.
#[inline]
pub fn is_mod_compatible(required_major: u8, required_minor: u8) -> bool {
    let (engine_major, engine_minor) = MOD_API_VERSION;
    required_major == engine_major && required_minor <= engine_minor
}

/// Returns the current API version
#[inline]
pub fn get_api_version() -> u32 {
    ENGINE_API_VERSION
}

/// Returns the engine version string
#[inline]
pub fn get_engine_version() -> &'static str {
    ENGINE_VERSION
}
