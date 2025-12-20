//! Registry index schema (index.json) parsing and validation
//!
//! # Security (Feature 040)
//!
//! Index parsing includes size limits to prevent parser abuse:
//! - MAX_INDEX_JSON_BYTES: Maximum size for index.json (5 MB)

use crate::errors::DistributionError;
use crate::{EngineConstraint, ModDependency, ModId};
use plix_common::limits::MAX_INDEX_JSON_BYTES;
use semver::Version;
use serde::{Deserialize, Serialize};

/// Registry index containing all available mods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryIndex {
    /// Schema version for forward compatibility (must be 1)
    pub registry_version: u32,

    /// Registry name for display
    pub name: String,

    /// Registry base URL (for relative download URLs)
    #[serde(default)]
    pub base_url: Option<String>,

    /// Last update timestamp (RFC 3339)
    #[serde(default)]
    pub updated_at: Option<String>,

    /// All mods in this registry
    pub mods: Vec<RegistryMod>,
}

impl RegistryIndex {
    /// Parse index from JSON string
    ///
    /// # Security
    /// Validates size against MAX_INDEX_JSON_BYTES before parsing.
    pub fn from_json(json: &str) -> Result<Self, DistributionError> {
        // Security: Check size limit before parsing (T051)
        if json.len() > MAX_INDEX_JSON_BYTES {
            return Err(DistributionError::invalid_index(
                "unknown",
                &format!(
                    "Index size {} bytes exceeds limit {} bytes (EMREG002)",
                    json.len(),
                    MAX_INDEX_JSON_BYTES
                ),
            ));
        }
        let index: Self = serde_json::from_str(json)?;
        index.validate()?;
        Ok(index)
    }

    /// Parse index from JSON bytes
    ///
    /// # Security
    /// Validates size against MAX_INDEX_JSON_BYTES before parsing.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DistributionError> {
        // Security: Check size limit before parsing (T051)
        if bytes.len() > MAX_INDEX_JSON_BYTES {
            return Err(DistributionError::invalid_index(
                "unknown",
                &format!(
                    "Index size {} bytes exceeds limit {} bytes (EMREG002)",
                    bytes.len(),
                    MAX_INDEX_JSON_BYTES
                ),
            ));
        }
        let index: Self = serde_json::from_slice(bytes)?;
        index.validate()?;
        Ok(index)
    }

    /// Validate the index
    fn validate(&self) -> Result<(), DistributionError> {
        // Check version
        if self.registry_version != 1 {
            return Err(DistributionError::invalid_index(
                &self.name,
                &format!(
                    "Unsupported registry version {}, expected 1",
                    self.registry_version
                ),
            ));
        }

        // Validate each mod
        for mod_entry in &self.mods {
            mod_entry.validate(&self.name)?;
        }

        Ok(())
    }

    /// Find a mod by ID
    pub fn find_mod(&self, id: &ModId) -> Option<&RegistryMod> {
        self.mods.iter().find(|m| &m.id == id)
    }

    /// Find a specific version of a mod
    pub fn find_version(&self, id: &ModId, version: &Version) -> Option<&ModVersionEntry> {
        self.find_mod(id)
            .and_then(|m| m.versions.iter().find(|v| &v.version == version))
    }

    /// Resolve the full download URL for a version
    pub fn resolve_download_url(&self, version: &ModVersionEntry) -> String {
        if version.download_url.starts_with("http://")
            || version.download_url.starts_with("https://")
            || version.download_url.starts_with("file://")
        {
            // Absolute URL
            version.download_url.clone()
        } else if let Some(ref base) = self.base_url {
            // Relative URL - join with base
            let base = base.trim_end_matches('/');
            let path = version.download_url.trim_start_matches('/');
            format!("{}/{}", base, path)
        } else {
            // No base URL, use as-is
            version.download_url.clone()
        }
    }
}

/// A mod in the registry with all its versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMod {
    /// Mod ID
    pub id: ModId,

    /// Display name
    pub name: String,

    /// Short description
    #[serde(default)]
    pub description: Option<String>,

    /// Author name
    #[serde(default)]
    pub author: Option<String>,

    /// Homepage URL
    #[serde(default)]
    pub homepage: Option<String>,

    /// All available versions (newest first recommended)
    pub versions: Vec<ModVersionEntry>,
}

impl RegistryMod {
    /// Validate this mod entry
    fn validate(&self, registry_name: &str) -> Result<(), DistributionError> {
        if self.versions.is_empty() {
            return Err(DistributionError::invalid_index(
                registry_name,
                &format!("Mod '{}' has no versions", self.id),
            ));
        }

        for version in &self.versions {
            version.validate(registry_name, &self.id)?;
        }

        Ok(())
    }

    /// Get the latest non-yanked version
    pub fn latest_version(&self) -> Option<&ModVersionEntry> {
        self.versions.iter().filter(|v| !v.yanked).max_by(|a, b| a.version.cmp(&b.version))
    }

    /// Find versions matching a requirement
    pub fn matching_versions(&self, req: &semver::VersionReq) -> Vec<&ModVersionEntry> {
        self.versions
            .iter()
            .filter(|v| !v.yanked && req.matches(&v.version))
            .collect()
    }

    /// Get the latest version matching a requirement
    pub fn latest_matching(&self, req: &semver::VersionReq) -> Option<&ModVersionEntry> {
        self.matching_versions(req)
            .into_iter()
            .max_by(|a, b| a.version.cmp(&b.version))
    }
}

/// A specific mod version entry in the registry index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersionEntry {
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

impl ModVersionEntry {
    /// Validate this version entry
    fn validate(&self, registry_name: &str, mod_id: &ModId) -> Result<(), DistributionError> {
        // Validate SHA-256 format (64 hex chars)
        if self.sha256.len() != 64 || !self.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(DistributionError::invalid_index(
                registry_name,
                &format!(
                    "Invalid SHA-256 hash for {} v{}: must be 64 hex characters",
                    mod_id, self.version
                ),
            ));
        }

        // Validate signature format if present (128 hex chars)
        if let Some(ref sig) = self.signature {
            if sig.len() != 128 || !sig.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(DistributionError::invalid_index(
                    registry_name,
                    &format!(
                        "Invalid signature for {} v{}: must be 128 hex characters",
                        mod_id, self.version
                    ),
                ));
            }
        }

        // Validate signer format if present (16 hex chars)
        if let Some(ref signer) = self.signer {
            if signer.len() != 16 || !signer.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(DistributionError::invalid_index(
                    registry_name,
                    &format!(
                        "Invalid signer for {} v{}: must be 16 hex characters",
                        mod_id, self.version
                    ),
                ));
            }
        }

        // Validate download URL is not empty
        if self.download_url.is_empty() {
            return Err(DistributionError::invalid_index(
                registry_name,
                &format!("Empty download URL for {} v{}", mod_id, self.version),
            ));
        }

        Ok(())
    }
}

// Serde helper for Version
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

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_INDEX: &str = r#"{
        "registry_version": 1,
        "name": "Test Registry",
        "base_url": "https://mods.example.com/",
        "mods": [
            {
                "id": "core-lib",
                "name": "Core Library",
                "description": "Shared utilities",
                "versions": [
                    {
                        "version": "1.2.0",
                        "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                        "download_url": "bundles/core-lib-1.2.0.plixmod",
                        "size": 102400,
                        "api_version": 1
                    },
                    {
                        "version": "1.1.0",
                        "sha256": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
                        "download_url": "bundles/core-lib-1.1.0.plixmod",
                        "size": 98000,
                        "api_version": 1
                    }
                ]
            },
            {
                "id": "weapons-pack",
                "name": "Weapons Pack",
                "versions": [
                    {
                        "version": "2.0.0",
                        "sha256": "b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3",
                        "download_url": "bundles/weapons-pack-2.0.0.plixmod",
                        "size": 512000,
                        "dependencies": [
                            {"id": "core-lib", "version_req": "^1.0"}
                        ],
                        "api_version": 1,
                        "engine": {"min": "0.36.0"}
                    }
                ]
            }
        ]
    }"#;

    #[test]
    fn test_parse_valid_index() {
        let index = RegistryIndex::from_json(VALID_INDEX).unwrap();
        assert_eq!(index.registry_version, 1);
        assert_eq!(index.name, "Test Registry");
        assert_eq!(index.mods.len(), 2);
    }

    #[test]
    fn test_find_mod() {
        let index = RegistryIndex::from_json(VALID_INDEX).unwrap();
        let mod_id = ModId::new("core-lib").unwrap();
        let found = index.find_mod(&mod_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Core Library");

        let not_found = index.find_mod(&ModId::new("nonexistent").unwrap());
        assert!(not_found.is_none());
    }

    #[test]
    fn test_latest_version() {
        let index = RegistryIndex::from_json(VALID_INDEX).unwrap();
        let mod_id = ModId::new("core-lib").unwrap();
        let core_lib = index.find_mod(&mod_id).unwrap();
        let latest = core_lib.latest_version().unwrap();
        assert_eq!(latest.version, Version::new(1, 2, 0));
    }

    #[test]
    fn test_matching_versions() {
        let index = RegistryIndex::from_json(VALID_INDEX).unwrap();
        let mod_id = ModId::new("core-lib").unwrap();
        let core_lib = index.find_mod(&mod_id).unwrap();

        let req = semver::VersionReq::parse("^1.0").unwrap();
        let matching = core_lib.matching_versions(&req);
        assert_eq!(matching.len(), 2);

        let req = semver::VersionReq::parse("=1.1.0").unwrap();
        let matching = core_lib.matching_versions(&req);
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].version, Version::new(1, 1, 0));
    }

    #[test]
    fn test_resolve_download_url() {
        let index = RegistryIndex::from_json(VALID_INDEX).unwrap();
        let mod_id = ModId::new("core-lib").unwrap();
        let version = index
            .find_mod(&mod_id)
            .unwrap()
            .latest_version()
            .unwrap();

        let url = index.resolve_download_url(version);
        assert_eq!(
            url,
            "https://mods.example.com/bundles/core-lib-1.2.0.plixmod"
        );
    }

    #[test]
    fn test_invalid_version() {
        let json = r#"{
            "registry_version": 2,
            "name": "Test",
            "mods": []
        }"#;
        let result = RegistryIndex::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_sha256() {
        let json = r#"{
            "registry_version": 1,
            "name": "Test",
            "mods": [{
                "id": "test-mod",
                "name": "Test",
                "versions": [{
                    "version": "1.0.0",
                    "sha256": "invalid",
                    "download_url": "test.plixmod",
                    "size": 100,
                    "api_version": 1
                }]
            }]
        }"#;
        let result = RegistryIndex::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_yanked_version() {
        let json = r#"{
            "registry_version": 1,
            "name": "Test",
            "mods": [{
                "id": "test-mod",
                "name": "Test",
                "versions": [
                    {
                        "version": "2.0.0",
                        "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                        "download_url": "test-2.0.0.plixmod",
                        "size": 100,
                        "api_version": 1,
                        "yanked": true
                    },
                    {
                        "version": "1.0.0",
                        "sha256": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
                        "download_url": "test-1.0.0.plixmod",
                        "size": 100,
                        "api_version": 1
                    }
                ]
            }]
        }"#;
        let index = RegistryIndex::from_json(json).unwrap();
        let test_mod = index.find_mod(&ModId::new("test-mod").unwrap()).unwrap();

        // Latest should skip yanked version
        let latest = test_mod.latest_version().unwrap();
        assert_eq!(latest.version, Version::new(1, 0, 0));
    }
}
