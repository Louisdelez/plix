//! Anti-cheat subsystem for the plix server.
//!
//! This module provides:
//! - Input validation (NaN/INF rejection, bounds checking)
//! - Rate limiting (per-action fixed-window counters)
//! - Movement sanity checks (speed/teleport detection)
//! - Sanction management (warning → kick → ban escalation)

pub mod ban_list;
pub mod config;
pub mod sanctions;
pub mod state;
pub mod validate;

pub use ban_list::BanList;
pub use config::AntiCheatConfig;
pub use sanctions::SanctionType;
pub use state::AntiCheatState;

/// Types of infractions that can be recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfractionType {
    // Input validation
    /// NaN or INF value in input
    InvalidFloat,
    /// Position/rotation outside valid range
    OutOfBounds,
    /// Out-of-order or duplicate input sequence
    InvalidSequence,

    // Rate limiting
    /// Movement input rate exceeded
    InputRateExceeded,
    /// Attack rate exceeded
    AttackRateExceeded,
    /// Block edit rate exceeded
    BlockEditRateExceeded,
    /// Ready toggle rate exceeded
    ReadyToggleRateExceeded,
    /// Training reset rate exceeded
    TrainingResetRateExceeded,

    // Physics sanity
    /// Speed exceeded maximum allowed
    SpeedExceeded,
    /// Acceleration exceeded maximum allowed
    AccelerationExceeded,
}

/// Types of actions that can be rate-limited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    /// Movement input
    Input,
    /// Attack action
    Attack,
    /// Block edit (place/remove)
    BlockEdit,
    /// Ready toggle
    ReadyToggle,
    /// Training reset (once per second)
    TrainingReset,
    /// Hotbar slot selection
    SlotSelect,
    /// Inventory item use
    InventoryUse,
    /// Shop purchase (5 req/sec)
    ShopBuy,
}
