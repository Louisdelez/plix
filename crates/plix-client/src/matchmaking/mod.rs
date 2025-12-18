//! Quick Join matchmaking module (Feature 027)
//!
//! Provides automatic server selection based on game mode, region,
//! and server scoring criteria. Uses the master server (Feature 026)
//! for server discovery and profile (Feature 025) for preference storage.
//!
//! # Components
//!
//! - `request`: QuickJoinRequest and QuickJoinResult types
//! - `preferences`: User preference persistence
//! - `filtering`: Server filtering (mode, region, version, capacity)
//! - `scoring`: Server scoring algorithm
//! - `selection`: Best server selection with tie-breaking
//! - `retry`: Auto-retry logic on connection failure

pub mod filtering;
pub mod preferences;
pub mod request;
pub mod retry;
pub mod scoring;
pub mod selection;

// Re-export main types for convenient access
pub use filtering::filter_servers;
pub use preferences::{
    load_preferences, save_preferences, update_preference_mode, update_preference_region,
    MatchmakingPreferences,
};
pub use request::{select_server, QuickJoinRequest, QuickJoinResult};
pub use retry::RetryState;
pub use scoring::{calculate_score, score_servers, ScoreBreakdown, ServerScore};
pub use selection::select_best_server;
