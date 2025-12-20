//! Content loading and validation for Adventure Mode
//!
//! This module handles loading TOML content files from `assets/content/` and
//! validating references and constraints.

pub mod loader;
pub mod validator;

pub use loader::{ContentLoader, ContentRegistry, LoadError};
pub use validator::{ContentValidator, ValidationError, ValidationMode};

/// Content schema version for content file compatibility.
///
/// This version is checked when loading content files to ensure compatibility.
/// - Major: Breaking schema changes (content files must be migrated)
/// - Minor: Additive schema changes (backward compatible)
///
/// Content files created for v1.0 schema will work on any v1.x engine.
pub const CONTENT_SCHEMA_VERSION: (u8, u8) = (1, 0);

/// Check if content with the given schema version is compatible with this engine.
#[inline]
pub fn is_content_compatible(content_major: u8, content_minor: u8) -> bool {
    let (engine_major, engine_minor) = CONTENT_SCHEMA_VERSION;
    content_major == engine_major && content_minor <= engine_minor
}
