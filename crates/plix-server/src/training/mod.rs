//! Training mode implementation
//!
//! Sandbox training mode allowing players to practice against basic bots
//! with configurable behaviors and permissive settings.

pub mod bot;
pub mod config;
pub mod coordinator;
pub mod stats;

pub use bot::{BotBehavior, TrainingBot};
pub use config::{BotBehaviorType, TrainingConfig};
pub use coordinator::{TrainingCoordinator, TrainingEvent};
pub use stats::TrainingStats;
