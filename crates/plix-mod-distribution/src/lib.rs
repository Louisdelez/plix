//! Mod Distribution System for Plix
//!
//! This crate provides a production-ready mod distribution system enabling servers to:
//! - Read mod configuration (registries + required mods with version constraints)
//! - Resolve dependencies using SemVer constraints and generate reproducible lockfiles
//! - Download mod bundles from remote/local registries with caching
//! - Verify integrity (SHA-256 mandatory) and signatures (optional)
//! - Install/extract mods into local cache
//!
//! # Example
//!
//! ```ignore
//! use plix_mod_distribution::{DistributionConfig, resolve_and_install};
//!
//! let config = DistributionConfig::load("server_mods.toml")?;
//! let result = resolve_and_install(&config).await?;
//! println!("Installed {} mods", result.mods.len());
//! ```

pub mod bundle;
pub mod config;
pub mod downloader;
pub mod errors;
pub mod index;
pub mod installer;
pub mod integrity;
pub mod lockfile;
pub mod registry;
pub mod resolver;

#[cfg(feature = "signatures")]
pub mod signatures;

// Re-exports for public API
pub use config::{
    CacheSettings, DistributionConfig, DownloadSettings, JoinPolicy, ModRequirement,
    RegistryConfig, SyncConfig, TrustPolicy,
};
pub use errors::{DistributionError, ErrorCode};
pub use index::{ModVersionEntry, RegistryIndex, RegistryMod};
pub use lockfile::{LockedMod, Lockfile};
pub use resolver::{ResolvedGraph, ResolvedMod, ResolutionStats};

// Feature 037: Server Mods + Client Sync
pub use bundle::ModManifest;
pub use crate::RuntimeMode;

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Runtime mode for a mod: where it executes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMode {
    /// Server-only execution (default) - mod runs only on server
    #[default]
    Server,
    /// Client-only execution (future) - mod runs only on client
    Client,
    /// Both server and client execution (future) - mod runs on both
    Both,
}

impl RuntimeMode {
    /// Check if this mode requires server execution
    pub fn runs_on_server(&self) -> bool {
        matches!(self, RuntimeMode::Server | RuntimeMode::Both)
    }

    /// Check if this mode requires client execution
    pub fn runs_on_client(&self) -> bool {
        matches!(self, RuntimeMode::Client | RuntimeMode::Both)
    }

    /// Check if this is a server-only mod
    pub fn is_server_only(&self) -> bool {
        matches!(self, RuntimeMode::Server)
    }
}

impl fmt::Display for RuntimeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeMode::Server => write!(f, "server"),
            RuntimeMode::Client => write!(f, "client"),
            RuntimeMode::Both => write!(f, "both"),
        }
    }
}

/// Unique mod identifier (lowercase alphanumeric + hyphens)
/// Examples: "my-mod", "core-lib", "weapons-pack-v2"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModId(String);

impl ModId {
    /// Create a new ModId, validating format
    pub fn new(id: impl Into<String>) -> Result<Self, DistributionError> {
        let id = id.into();
        if Self::is_valid(&id) {
            Ok(Self(id))
        } else {
            Err(DistributionError::invalid_mod_id(&id))
        }
    }

    /// Create a ModId without validation (for internal use)
    pub(crate) fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Validate mod ID format: [a-z0-9][a-z0-9-]*[a-z0-9] or single char [a-z0-9]
    pub fn is_valid(id: &str) -> bool {
        if id.is_empty() || id.len() > 64 {
            return false;
        }
        if id.len() == 1 {
            return id.chars().next().map_or(false, |c| c.is_ascii_lowercase() || c.is_ascii_digit());
        }
        id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !id.starts_with('-')
            && !id.ends_with('-')
    }

    /// Get the string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ModId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Dependency on another mod with version constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    /// Mod ID of the dependency
    pub id: ModId,

    /// SemVer version requirement (e.g., "^1.0", ">=2.0, <3.0")
    #[serde(with = "version_req_serde")]
    pub version_req: VersionReq,

    /// Whether this dependency is optional
    #[serde(default)]
    pub optional: bool,
}

/// Engine version constraints
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngineConstraint {
    /// Minimum engine version (inclusive)
    #[serde(default, with = "option_version_serde")]
    pub min: Option<Version>,

    /// Maximum engine version (inclusive)
    #[serde(default, with = "option_version_serde")]
    pub max: Option<Version>,
}

impl EngineConstraint {
    /// Check if an engine version is compatible with this constraint
    pub fn is_compatible(&self, engine_version: &Version) -> bool {
        if let Some(ref min) = self.min {
            if engine_version < min {
                return false;
            }
        }
        if let Some(ref max) = self.max {
            if engine_version > max {
                return false;
            }
        }
        true
    }
}

/// A specific mod version in a registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersion {
    /// SemVer version
    #[serde(with = "version_serde")]
    pub version: Version,

    /// SHA-256 hash of the bundle (hex-encoded, 64 chars)
    pub sha256: String,

    /// Download URL (absolute or relative to registry base)
    pub download_url: String,

    /// Bundle size in bytes
    pub size: u64,

    /// Dependencies with version constraints
    #[serde(default)]
    pub dependencies: Vec<ModDependency>,

    /// Required API version (from plix-mod-core)
    pub api_version: u32,

    /// Optional engine version constraints
    #[serde(default)]
    pub engine: Option<EngineConstraint>,

    /// Optional Ed25519 signature (hex-encoded, 128 chars)
    #[serde(default)]
    pub signature: Option<String>,

    /// Signing key ID (hex-encoded public key prefix, 16 chars)
    #[serde(default)]
    pub signer: Option<String>,

    /// Publication timestamp (RFC 3339)
    #[serde(default)]
    pub published_at: Option<String>,

    /// Yanked flag (version should not be installed)
    #[serde(default)]
    pub yanked: bool,
}

// Serde helpers for semver types
mod version_serde {
    use semver::Version;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(version: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&version.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::parse(&s).map_err(serde::de::Error::custom)
    }
}

mod option_version_serde {
    use semver::Version;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(version: &Option<Version>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match version {
            Some(v) => serializer.serialize_some(&v.to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Version>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => Version::parse(&s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

mod version_req_serde {
    use semver::VersionReq;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(req: &VersionReq, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&req.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<VersionReq, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        VersionReq::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// Current API version supported by this distribution system
pub const CURRENT_API_VERSION: u32 = 1;

/// Current engine version (should match plix version)
pub fn current_engine_version() -> Version {
    Version::new(0, 36, 0)
}

/// Check API version compatibility
pub fn check_api_compatibility(mod_api_version: u32, engine_api_version: u32) -> bool {
    mod_api_version <= engine_api_version
}

/// Check engine version compatibility
pub fn check_engine_compatibility(
    constraint: &Option<EngineConstraint>,
    current_version: &Version,
) -> bool {
    match constraint {
        Some(c) => c.is_compatible(current_version),
        None => true,
    }
}

/// Main entry point: resolve dependencies and install mods
///
/// This orchestrates the full installation pipeline:
/// 1. Load config
/// 2. Fetch registry indexes
/// 3. Resolve dependencies
/// 4. Generate/update lockfile
/// 5. Download bundles
/// 6. Verify integrity (and signatures if enabled)
/// 7. Extract to cache
pub async fn resolve_and_install(
    config: &DistributionConfig,
    server_root: &std::path::Path,
) -> Result<ResolvedGraph, DistributionError> {
    use crate::downloader::Downloader;
    use crate::installer::Installer;
    use crate::integrity::verify_bundle_hash;
    use crate::lockfile::Lockfile;
    use crate::registry::RegistryManager;
    use crate::resolver::Resolver;

    tracing::info!("Starting mod distribution pipeline");

    // 1. Initialize registry manager and fetch indexes
    let registry_manager = RegistryManager::new(config)?;
    tracing::info!("Fetching registry indexes...");
    let indexes = registry_manager.fetch_all_indexes().await?;

    // 2. Check for existing lockfile
    let lockfile_path = server_root.join("mods.lock");
    let existing_lockfile = if lockfile_path.exists() {
        tracing::info!("Found existing lockfile at {}", lockfile_path.display());
        Some(Lockfile::load(&lockfile_path)?)
    } else {
        None
    };

    // 3. Resolve dependencies
    tracing::info!("Resolving dependencies...");
    let resolver = Resolver::new(&indexes, config);
    let resolved = resolver.resolve(existing_lockfile.as_ref())?;
    tracing::info!(
        "Resolved {} mods (total size: {} bytes)",
        resolved.mods.len(),
        resolved.stats.total_size
    );

    // 4. Generate/update lockfile
    let lockfile = Lockfile::from_resolved(&resolved);
    lockfile.write(&lockfile_path)?;
    tracing::info!("Wrote lockfile to {}", lockfile_path.display());

    // 5. Download and verify bundles
    let downloader = Downloader::new(config);
    let installer = Installer::new(config)?;

    for resolved_mod in &resolved.mods {
        let bundle_path = installer.bundle_path(&resolved_mod.sha256);

        // Check if already cached
        if bundle_path.exists() {
            tracing::debug!(
                "Bundle {} already cached at {}",
                resolved_mod.id,
                bundle_path.display()
            );
        } else {
            tracing::info!(
                "Downloading {} v{} from {}...",
                resolved_mod.id,
                resolved_mod.version,
                resolved_mod.download_url
            );
            downloader
                .download_bundle(&resolved_mod.download_url, &bundle_path)
                .await?;
        }

        // 6. Verify integrity
        tracing::debug!("Verifying hash for {}...", resolved_mod.id);
        verify_bundle_hash(&bundle_path, &resolved_mod.sha256)?;

        // 7. Extract to installed directory
        let install_dir = installer.install_path(&resolved_mod.id, &resolved_mod.version);
        if !install_dir.exists() {
            tracing::info!(
                "Extracting {} v{} to {}...",
                resolved_mod.id,
                resolved_mod.version,
                install_dir.display()
            );
            installer.extract_bundle(&bundle_path, &install_dir)?;
        } else {
            tracing::debug!(
                "{} v{} already installed at {}",
                resolved_mod.id,
                resolved_mod.version,
                install_dir.display()
            );
        }
    }

    tracing::info!(
        "Mod distribution complete: {} mods installed",
        resolved.mods.len()
    );
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_id_valid() {
        assert!(ModId::is_valid("my-mod"));
        assert!(ModId::is_valid("a"));
        assert!(ModId::is_valid("mod123"));
        assert!(ModId::is_valid("my-cool-mod"));
        assert!(ModId::is_valid("a1b2c3"));
    }

    #[test]
    fn test_mod_id_invalid() {
        assert!(!ModId::is_valid(""));
        assert!(!ModId::is_valid("-mod"));
        assert!(!ModId::is_valid("mod-"));
        assert!(!ModId::is_valid("My-Mod")); // uppercase
        assert!(!ModId::is_valid("my_mod")); // underscore
        assert!(!ModId::is_valid("my mod")); // space
    }

    #[test]
    fn test_mod_id_new() {
        assert!(ModId::new("my-mod").is_ok());
        assert!(ModId::new("-invalid").is_err());
    }

    #[test]
    fn test_engine_constraint_compatible() {
        let constraint = EngineConstraint {
            min: Some(Version::new(0, 30, 0)),
            max: Some(Version::new(1, 0, 0)),
        };

        assert!(constraint.is_compatible(&Version::new(0, 35, 0)));
        assert!(constraint.is_compatible(&Version::new(0, 30, 0)));
        assert!(constraint.is_compatible(&Version::new(1, 0, 0)));
        assert!(!constraint.is_compatible(&Version::new(0, 29, 0)));
        assert!(!constraint.is_compatible(&Version::new(1, 1, 0)));
    }

    #[test]
    fn test_api_compatibility() {
        assert!(check_api_compatibility(1, 1));
        assert!(check_api_compatibility(1, 2));
        assert!(!check_api_compatibility(2, 1));
    }
}
