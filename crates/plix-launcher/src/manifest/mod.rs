//! Manifest types and operations

mod fetch;
mod validate;

pub use fetch::*;
pub use validate::*;

use serde::{Deserialize, Serialize};

/// Remote manifest describing a release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Manifest format version (for future compatibility)
    pub manifest_version: u32,

    /// Game version (semver format)
    pub version: String,

    /// Optional protocol version for server compatibility
    #[serde(default)]
    pub protocol_version: Option<u8>,

    /// Release timestamp (Unix epoch seconds)
    pub release_date: u64,

    /// Files included in this release
    pub files: Vec<ManifestFile>,

    /// Optional release notes URL
    #[serde(default)]
    pub release_notes_url: Option<String>,
}

/// A file entry in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFile {
    /// Relative path within installation directory
    pub path: String,

    /// Download URL for this file
    pub url: String,

    /// File size in bytes
    pub size: u64,

    /// SHA256 checksum (hex-encoded, lowercase)
    pub sha256: String,

    /// Whether file is executable (for chmod on Unix)
    #[serde(default)]
    pub executable: bool,
}
