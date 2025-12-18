//! Local player profile management
//!
//! This module provides client-side profile persistence:
//! - `PlayerProfile`: Local profile stored in ~/.config/plix/profile.toml
//! - Load/save functionality with atomic writes

mod player_profile;

pub use player_profile::{load_profile, profile_path, save_profile, PlayerProfile, ProfileError};
