//! Performance profiling and optimization subsystem
//!
//! This module provides:
//! - Reproducible profiling scenarios (Idle, WorldChurn, NetStress)
//! - Tick budget tracking and backpressure mechanisms
//! - Structured performance reports (JSON format)
//! - Per-subsystem timing instrumentation
//! - Allocation tracking and buffer reuse utilities
//!
//! # Feature Flags
//! - `perf`: Enables the performance profiling system (default: off)
//!
//! # Usage
//! ```bash
//! cargo run --release --bin plix-perf -- --scenario idle --duration 60
//! ```

pub mod config;

#[cfg(feature = "perf")]
pub mod alloc;
#[cfg(feature = "perf")]
pub mod budgets;
#[cfg(feature = "perf")]
pub mod metrics;
#[cfg(feature = "perf")]
pub mod reporter;
#[cfg(feature = "perf")]
pub mod scenarios;

// Re-export commonly used types
pub use config::PerfConfig;

#[cfg(feature = "perf")]
pub use alloc::{AllocTracker, BufferPool, ReusableBuffer};
#[cfg(feature = "perf")]
pub use budgets::TickBudget;
#[cfg(feature = "perf")]
pub use metrics::{Histogram, NetMessageStats, SubsystemMetrics};
#[cfg(feature = "perf")]
pub use reporter::PerfReport;
#[cfg(feature = "perf")]
pub use scenarios::PerfScenario;
