//! Block Physics System - Common Types
//!
//! This module provides shared types for the block physics system:
//! - `BlockPhysicsConfig`: Configuration for gravity and liquid physics
//! - `BlockPhysicsEvent`: Events queued for physics processing
//! - `BlockPhysicsQueue`: Bounded FIFO queue with deduplication
//! - `BlockPhysicsMetrics`: Observable counters for monitoring
//! - `BlockWorld`: Trait for abstracting block world access
//!
//! The actual physics simulation runs server-side only (in plix-server).
//! Clients receive block updates via the existing replication system.

pub mod config;
pub mod event;
pub mod metrics;
pub mod queue;
pub mod world_trait;

pub use config::BlockPhysicsConfig;
pub use event::{BlockPhysicsEvent, BlockPhysicsEventKind};
pub use metrics::BlockPhysicsMetrics;
pub use queue::BlockPhysicsQueue;
pub use world_trait::BlockWorld;
