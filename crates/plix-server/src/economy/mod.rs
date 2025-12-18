//! Economy system for currency and shop management.
//!
//! This module provides a minimalist server-authoritative economy system that allows
//! players to earn coins from in-match actions and spend them in a static shop.
//!
//! Features:
//! - Per-player wallet with coin balance (reset per match)
//! - Configurable earning rules (kills, CTF captures, BR placements)
//! - Static shop registry with purchasable offers
//! - Atomic purchase validation (all-or-nothing)
//! - Game mode configuration (enabled/disabled per mode)
//! - Rate limiting on purchase requests

pub mod config;
pub mod earnings;
pub mod metrics;
pub mod purchase;
pub mod shop;
pub mod wallet;

pub use config::{from_arena_config, get_economy_config, EconomyConfig};
pub use earnings::{award_coins, EarningEvent};
pub use metrics::EconomyMetrics;
pub use purchase::{try_purchase, PurchaseResult};
pub use shop::{ShopOffer, ShopRegistry};
pub use wallet::PlayerWallet;
