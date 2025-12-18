//! Plix Launcher Library
//!
//! Provides game update and launch functionality.

pub mod config;
pub mod download;
pub mod error;
pub mod install;
pub mod launch;
pub mod manifest;
pub mod ui;
pub mod version;

pub use config::LauncherConfig;
pub use error::LauncherError;
pub use manifest::{Manifest, ManifestFile};
pub use version::{LocalState, UpdateStatus};
