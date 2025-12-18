//! Menu handlers for CEF UI (Feature 031)
//!
//! This module provides handlers for menu-related bridge messages:
//! - GetConfig/SetConfig - Settings management
//! - FetchServers/Connect - Server browser
//! - ToggleFavorite - Favorites management

pub mod config;
pub mod favorites;
pub mod servers;

pub use config::{handle_get_config, handle_set_config};
pub use favorites::{handle_toggle_favorite, Favorites};
pub use servers::{handle_connect, handle_fetch_servers};
