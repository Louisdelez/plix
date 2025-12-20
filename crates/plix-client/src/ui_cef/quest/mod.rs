//! Quest UI bridges for CEF (Feature 043)
//!
//! This module provides bridges between the Rust game client and the
//! CEF-based quest UI pages:
//! - Quest Log: Full quest list with active/completed tabs
//! - Quest Tracker: HUD overlay for pinned quest

pub mod log;
pub mod tracker;

pub use log::QuestLogBridge;
pub use tracker::QuestTrackerBridge;
