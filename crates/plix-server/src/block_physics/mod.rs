//! Block Physics System - Server-side Implementation
//!
//! This module provides the server-side physics simulation for blocks:
//! - Gravity for falling blocks (sand)
//! - Liquid spreading (water)
//! - Budget-bounded event processing
//!
//! The physics system is server-authoritative - clients receive block updates
//! via the existing replication system.

pub mod gravity;
pub mod liquid;
pub mod system;

pub use system::BlockPhysicsSystem;
