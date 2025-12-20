//! Bundle (.plixmod) format handling
//!
//! # Security (Feature 040)
//!
//! Bundle parsing includes safety checks:
//! - MAX_MOD_TOML_BYTES: Maximum size for mod.toml (256 KB)
//! - MAX_ZIP_FILES: Maximum files in bundle (10,000)
//! - MAX_ZIP_RATIO: Maximum decompression ratio (20:1) for zip bomb detection
//! - Path traversal prevention (no `../` or absolute paths)

use crate::errors::DistributionError;
use crate::{ModId, RuntimeMode};
use plix_common::limits::{MAX_BUNDLE_BYTES, MAX_MOD_TOML_BYTES, MAX_ZIP_FILES, MAX_ZIP_RATIO};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

/// Mod manifest (mod.toml) structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    /// Mod identifier
    pub id: String,

    /// Mod display name
    #[serde(default)]
    pub name: String,

    /// Mod version (SemVer)
    pub version: String,

    /// Required API version
    #[serde(default = "default_api_version")]
    pub api_version: u32,

    /// Mod description
    #[serde(default)]
    pub description: String,

    /// Mod author
    #[serde(default)]
    pub author: String,

    /// Runtime mode: where this mod executes (default: "server")
    #[serde(default)]
    pub runtime: RuntimeMode,

    /// Whether this mod has client payload data to sync
    #[serde(default)]
    pub client_payload: bool,

    /// Files to include in client payload (relative paths within bundle)
    #[serde(default)]
    pub client_payload_files: Vec<String>,

    /// Network configuration for mod channels
    #[serde(default)]
    pub network: ModNetworkConfig,
}

fn default_api_version() -> u32 {
    1
}

/// Network configuration for mod channels
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModNetworkConfig {
    /// Channels client is allowed to send messages on
    /// Format: subchannel names (without mod:{id}: prefix)
    #[serde(default)]
    pub allowed_client_channels: Vec<String>,
}

impl ModManifest {
    /// Validate the manifest
    pub fn validate(&self) -> Result<(), DistributionError> {
        // Validate mod ID
        if !ModId::is_valid(&self.id) {
            return Err(DistributionError::invalid_index(
                "manifest",
                &format!("Invalid mod ID: '{}'", self.id),
            ));
        }

        // Validate version
        if Version::parse(&self.version).is_err() {
            return Err(DistributionError::invalid_index(
                "manifest",
                &format!("Invalid version: '{}'", self.version),
            ));
        }

        // Validate client_payload_files if client_payload is true
        if self.client_payload && self.client_payload_files.is_empty() {
            return Err(DistributionError::invalid_index(
                "manifest",
                "client_payload is true but client_payload_files is empty",
            ));
        }

        // Validate channel names
        for channel in &self.network.allowed_client_channels {
            if channel.len() > 64 {
                return Err(DistributionError::invalid_index(
                    "manifest",
                    &format!("Channel name too long: '{}'", channel),
                ));
            }
        }

        if self.network.allowed_client_channels.len() > 32 {
            return Err(DistributionError::invalid_index(
                "manifest",
                "Too many allowed_client_channels (max 32)",
            ));
        }

        Ok(())
    }

    /// Parse from TOML string
    ///
    /// # Security
    /// Validates size against MAX_MOD_TOML_BYTES before parsing.
    pub fn from_toml(toml_str: &str) -> Result<Self, DistributionError> {
        // Security: Check size limit before parsing (T052)
        if toml_str.len() > MAX_MOD_TOML_BYTES {
            return Err(DistributionError::invalid_index(
                "manifest",
                &format!(
                    "mod.toml size {} bytes exceeds limit {} bytes",
                    toml_str.len(),
                    MAX_MOD_TOML_BYTES
                ),
            ));
        }
        let manifest: ModManifest = toml::from_str(toml_str).map_err(|e| {
            DistributionError::invalid_index("manifest", &format!("Invalid TOML: {}", e))
        })?;
        manifest.validate()?;
        Ok(manifest)
    }
}

/// A mod bundle in memory
#[derive(Debug)]
pub struct ModBundle {
    /// Mod ID from manifest
    pub id: ModId,

    /// Version from manifest
    pub version: Version,

    /// Parsed manifest (as TOML string for now)
    pub manifest_toml: String,

    /// WASM module bytes (if present)
    pub wasm: Option<Vec<u8>>,

    /// Asset files: relative path -> contents
    pub assets: HashMap<String, Vec<u8>>,

    /// Computed SHA-256 hash
    pub sha256: String,
}

/// Check if a path is safe (no traversal, no absolute)
///
/// # Security
/// Prevents path traversal attacks in zip extraction.
fn is_safe_path(path: &str) -> bool {
    // Check for path traversal
    if path.contains("..") {
        return false;
    }

    // Check for absolute paths (Unix and Windows)
    if path.starts_with('/') || path.starts_with('\\') {
        return false;
    }

    // Check for Windows drive letters
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        return false;
    }

    true
}

impl ModBundle {
    /// Load a bundle from a .plixmod file
    ///
    /// # Security
    /// Validates:
    /// - File count against MAX_ZIP_FILES
    /// - Compression ratio against MAX_ZIP_RATIO
    /// - Path safety (no traversal)
    pub fn from_file(path: &Path) -> Result<Self, DistributionError> {
        let file = std::fs::File::open(path).map_err(|e| {
            DistributionError::internal(&format!(
                "Failed to open bundle '{}': {}",
                path.display(),
                e
            ))
        })?;

        // Security: Check file size (T053)
        let metadata = file.metadata().map_err(|e| {
            DistributionError::internal(&format!(
                "Failed to get metadata for '{}': {}",
                path.display(),
                e
            ))
        })?;
        if metadata.len() as usize > MAX_BUNDLE_BYTES {
            return Err(DistributionError::internal(&format!(
                "Bundle size {} bytes exceeds limit {} bytes",
                metadata.len(),
                MAX_BUNDLE_BYTES
            )));
        }

        // Calculate hash
        let sha256 = crate::integrity::calculate_hash(path)?;

        let mut archive = zip::ZipArchive::new(file)?;

        // Security: Check file count (T054)
        if archive.len() > MAX_ZIP_FILES {
            return Err(DistributionError::internal(&format!(
                "Bundle has {} files, exceeds limit {}",
                archive.len(),
                MAX_ZIP_FILES
            )));
        }

        // Security: Check compression ratio and paths (T053, T055)
        let compressed_size = metadata.len();
        let mut total_uncompressed: u64 = 0;
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name();

            // Security: Path traversal check (T053)
            if !is_safe_path(name) {
                return Err(DistributionError::internal(&format!(
                    "Unsafe path in bundle: '{}'",
                    name
                )));
            }

            total_uncompressed += file.size();
        }

        // Security: Zip bomb check (T055)
        if compressed_size > 0 {
            let ratio = total_uncompressed as f64 / compressed_size as f64;
            if ratio > MAX_ZIP_RATIO {
                return Err(DistributionError::internal(&format!(
                    "Bundle compression ratio {:.1}:1 exceeds limit {:.1}:1 (possible zip bomb)",
                    ratio, MAX_ZIP_RATIO
                )));
            }
        }

        // Re-open for reading content (archive was consumed above)
        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Read manifest
        let manifest_toml = {
            let mut manifest_file = archive.by_name("mod.toml").map_err(|_| {
                DistributionError::invalid_index(
                    "bundle",
                    &format!("Bundle '{}' missing mod.toml", path.display()),
                )
            })?;
            let mut content = String::new();
            manifest_file.read_to_string(&mut content)?;
            content
        };

        // Parse manifest to get id and version
        let manifest: toml::Value = toml::from_str(&manifest_toml).map_err(|e| {
            DistributionError::invalid_index(
                "bundle",
                &format!("Invalid mod.toml in '{}': {}", path.display(), e),
            )
        })?;

        let id = manifest
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                DistributionError::invalid_index(
                    "bundle",
                    &format!("mod.toml in '{}' missing 'id' field", path.display()),
                )
            })?;

        let version_str = manifest
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                DistributionError::invalid_index(
                    "bundle",
                    &format!("mod.toml in '{}' missing 'version' field", path.display()),
                )
            })?;

        let id = ModId::new(id)?;
        let version = Version::parse(version_str).map_err(|e| {
            DistributionError::invalid_index(
                "bundle",
                &format!(
                    "Invalid version '{}' in '{}': {}",
                    version_str,
                    path.display(),
                    e
                ),
            )
        })?;

        // Read WASM if present
        let wasm = {
            let mut archive = zip::ZipArchive::new(std::fs::File::open(path)?)?;
            let wasm_result = if let Ok(mut wasm_file) = archive.by_name("mod.wasm") {
                let mut bytes = Vec::new();
                wasm_file.read_to_end(&mut bytes)?;
                Some(bytes)
            } else {
                None
            };
            wasm_result
        };

        // Read assets
        let mut assets = HashMap::new();
        let mut archive = zip::ZipArchive::new(std::fs::File::open(path)?)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            // Skip non-asset files
            if name == "mod.toml" || name == "mod.wasm" {
                continue;
            }

            // Skip directories
            if name.ends_with('/') {
                continue;
            }

            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            assets.insert(name, bytes);
        }

        Ok(Self {
            id,
            version,
            manifest_toml,
            wasm,
            assets,
            sha256,
        })
    }
}

/// Quick metadata extraction without full bundle load
#[derive(Debug, Clone)]
pub struct BundleMetadata {
    /// Mod ID from manifest
    pub id: ModId,

    /// Version from manifest
    pub version: Version,

    /// Has WASM module
    pub has_wasm: bool,

    /// Total uncompressed size
    pub uncompressed_size: u64,

    /// File count
    pub file_count: usize,
}

impl BundleMetadata {
    /// Extract metadata from a bundle without loading all contents
    pub fn from_bundle(path: &Path) -> Result<Self, DistributionError> {
        let file = std::fs::File::open(path).map_err(|e| {
            DistributionError::internal(&format!(
                "Failed to open bundle '{}': {}",
                path.display(),
                e
            ))
        })?;

        let mut archive = zip::ZipArchive::new(file)?;

        // Read manifest
        let (id, version) = {
            let mut manifest_file = archive.by_name("mod.toml").map_err(|_| {
                DistributionError::invalid_index(
                    "bundle",
                    &format!("Bundle '{}' missing mod.toml", path.display()),
                )
            })?;
            let mut content = String::new();
            manifest_file.read_to_string(&mut content)?;

            let manifest: toml::Value = toml::from_str(&content).map_err(|e| {
                DistributionError::invalid_index(
                    "bundle",
                    &format!("Invalid mod.toml in '{}': {}", path.display(), e),
                )
            })?;

            let id = manifest
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    DistributionError::invalid_index(
                        "bundle",
                        &format!("mod.toml missing 'id' field"),
                    )
                })?;

            let version_str = manifest
                .get("version")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    DistributionError::invalid_index(
                        "bundle",
                        &format!("mod.toml missing 'version' field"),
                    )
                })?;

            let id = ModId::new(id)?;
            let version = Version::parse(version_str)?;

            (id, version)
        };

        // Check for WASM and calculate stats
        let mut has_wasm = false;
        let mut uncompressed_size = 0u64;
        let file_count = archive.len();

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            if file.name() == "mod.wasm" {
                has_wasm = true;
            }
            uncompressed_size += file.size();
        }

        Ok(Self {
            id,
            version,
            has_wasm,
            uncompressed_size,
            file_count,
        })
    }
}

/// Create a deterministic bundle from files
///
/// Files are sorted alphabetically and given a fixed timestamp for reproducibility.
pub fn create_bundle(
    output_path: &Path,
    manifest: &str,
    wasm: Option<&[u8]>,
    assets: &HashMap<String, Vec<u8>>,
) -> Result<String, DistributionError> {
    use std::io::Write;
    use zip::write::FileOptions;
    use zip::DateTime;

    let file = std::fs::File::create(output_path).map_err(|e| {
        DistributionError::internal(&format!(
            "Failed to create bundle '{}': {}",
            output_path.display(),
            e
        ))
    })?;

    let mut zip = zip::ZipWriter::new(file);

    // Fixed timestamp for determinism (2020-01-01 00:00:00)
    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(DateTime::from_date_and_time(2020, 1, 1, 0, 0, 0).unwrap());

    // Write manifest first
    zip.start_file("mod.toml", options)?;
    zip.write_all(manifest.as_bytes())?;

    // Write WASM if present
    if let Some(wasm_bytes) = wasm {
        zip.start_file("mod.wasm", options)?;
        zip.write_all(wasm_bytes)?;
    }

    // Write assets (sorted for determinism)
    let mut sorted_assets: Vec<_> = assets.iter().collect();
    sorted_assets.sort_by_key(|(name, _)| *name);

    for (name, content) in sorted_assets {
        zip.start_file(name, options)?;
        zip.write_all(content)?;
    }

    zip.finish()?;

    // Calculate hash of created bundle
    crate::integrity::calculate_hash(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_bundle(temp_dir: &Path) -> std::path::PathBuf {
        let bundle_path = temp_dir.join("test-mod-1.0.0.plixmod");

        let manifest = r#"
id = "test-mod"
name = "Test Mod"
version = "1.0.0"
api_version = 1
"#;

        let mut assets = HashMap::new();
        assets.insert("assets/texture.png".to_string(), vec![1, 2, 3, 4]);

        create_bundle(&bundle_path, manifest, None, &assets).unwrap();

        bundle_path
    }

    #[test]
    fn test_bundle_metadata() {
        let temp = tempdir().unwrap();
        let bundle_path = create_test_bundle(temp.path());

        let metadata = BundleMetadata::from_bundle(&bundle_path).unwrap();

        assert_eq!(metadata.id.as_str(), "test-mod");
        assert_eq!(metadata.version, Version::new(1, 0, 0));
        assert!(!metadata.has_wasm);
        assert_eq!(metadata.file_count, 2); // mod.toml + asset
    }

    #[test]
    fn test_full_bundle_load() {
        let temp = tempdir().unwrap();
        let bundle_path = create_test_bundle(temp.path());

        let bundle = ModBundle::from_file(&bundle_path).unwrap();

        assert_eq!(bundle.id.as_str(), "test-mod");
        assert_eq!(bundle.version, Version::new(1, 0, 0));
        assert!(bundle.wasm.is_none());
        assert_eq!(bundle.assets.len(), 1);
        assert!(bundle.assets.contains_key("assets/texture.png"));
    }

    #[test]
    fn test_bundle_with_wasm() {
        let temp = tempdir().unwrap();
        let bundle_path = temp.path().join("wasm-mod.plixmod");

        let manifest = r#"
id = "wasm-mod"
name = "WASM Mod"
version = "2.0.0"
api_version = 1
"#;

        let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6d]; // WASM magic bytes
        let assets = HashMap::new();

        create_bundle(&bundle_path, manifest, Some(&wasm_bytes), &assets).unwrap();

        let metadata = BundleMetadata::from_bundle(&bundle_path).unwrap();
        assert!(metadata.has_wasm);

        let bundle = ModBundle::from_file(&bundle_path).unwrap();
        assert!(bundle.wasm.is_some());
        assert_eq!(bundle.wasm.unwrap(), wasm_bytes);
    }

    #[test]
    fn test_bundle_determinism() {
        let temp = tempdir().unwrap();
        let bundle1_path = temp.path().join("bundle1.plixmod");
        let bundle2_path = temp.path().join("bundle2.plixmod");

        let manifest = r#"
id = "test-mod"
version = "1.0.0"
api_version = 1
"#;

        let mut assets = HashMap::new();
        assets.insert("z_file.txt".to_string(), vec![1, 2, 3]);
        assets.insert("a_file.txt".to_string(), vec![4, 5, 6]);

        let hash1 = create_bundle(&bundle1_path, manifest, None, &assets).unwrap();
        let hash2 = create_bundle(&bundle2_path, manifest, None, &assets).unwrap();

        // Bundles created with same inputs should have same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_bundle_missing_manifest() {
        let temp = tempdir().unwrap();
        let bundle_path = temp.path().join("bad.plixmod");

        // Create a zip without mod.toml
        let file = std::fs::File::create(&bundle_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        use std::io::Write;
        zip.start_file::<_, ()>("other.txt", Default::default())
            .unwrap();
        zip.write_all(b"test").unwrap();
        zip.finish().unwrap();

        let result = BundleMetadata::from_bundle(&bundle_path);
        assert!(result.is_err());
    }

    // Feature 037: Manifest parsing tests

    #[test]
    fn test_manifest_basic_valid() {
        let toml = r#"
id = "my-mod"
name = "My Mod"
version = "1.0.0"
api_version = 1
"#;
        let manifest = ModManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.id, "my-mod");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.runtime, RuntimeMode::Server);
        assert!(!manifest.client_payload);
    }

    #[test]
    fn test_manifest_with_runtime() {
        let toml = r#"
id = "server-mod"
version = "1.0.0"
runtime = "server"
"#;
        let manifest = ModManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.runtime, RuntimeMode::Server);

        let toml = r#"
id = "client-mod"
version = "1.0.0"
runtime = "client"
"#;
        let manifest = ModManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.runtime, RuntimeMode::Client);

        let toml = r#"
id = "both-mod"
version = "1.0.0"
runtime = "both"
"#;
        let manifest = ModManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.runtime, RuntimeMode::Both);
    }

    #[test]
    fn test_manifest_with_client_payload() {
        let toml = r#"
id = "payload-mod"
version = "1.0.0"
client_payload = true
client_payload_files = ["client/config.json", "client/items.toml"]
"#;
        let manifest = ModManifest::from_toml(toml).unwrap();
        assert!(manifest.client_payload);
        assert_eq!(manifest.client_payload_files.len(), 2);
    }

    #[test]
    fn test_manifest_client_payload_empty_files_error() {
        let toml = r#"
id = "bad-mod"
version = "1.0.0"
client_payload = true
"#;
        let result = ModManifest::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_with_network_config() {
        let toml = r#"
id = "network-mod"
version = "1.0.0"

[network]
allowed_client_channels = ["input", "config", "request"]
"#;
        let manifest = ModManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.network.allowed_client_channels.len(), 3);
    }

    #[test]
    fn test_manifest_invalid_runtime() {
        let toml = r#"
id = "bad-mod"
version = "1.0.0"
runtime = "invalid"
"#;
        let result = ModManifest::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_invalid_id() {
        let toml = r#"
id = "Invalid-Mod"
version = "1.0.0"
"#;
        let result = ModManifest::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_too_many_channels() {
        let channels: Vec<String> = (0..33).map(|i| format!("channel{}", i)).collect();
        let channels_toml = channels
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(", ");

        let toml = format!(
            r#"
id = "bad-mod"
version = "1.0.0"

[network]
allowed_client_channels = [{}]
"#,
            channels_toml
        );
        let result = ModManifest::from_toml(&toml);
        assert!(result.is_err());
    }

    // Security tests (Feature 040)

    #[test]
    fn test_safe_path_valid() {
        assert!(is_safe_path("mod.toml"));
        assert!(is_safe_path("assets/texture.png"));
        assert!(is_safe_path("deep/nested/path/file.txt"));
    }

    #[test]
    fn test_safe_path_traversal() {
        assert!(!is_safe_path("../etc/passwd"));
        assert!(!is_safe_path("assets/../../../etc/passwd"));
        assert!(!is_safe_path(".."));
    }

    #[test]
    fn test_safe_path_absolute() {
        assert!(!is_safe_path("/etc/passwd"));
        assert!(!is_safe_path("\\Windows\\System32"));
        assert!(!is_safe_path("C:\\Windows"));
    }

    #[test]
    fn test_manifest_size_limit() {
        // Create a TOML string that exceeds MAX_MOD_TOML_BYTES
        let large_toml = format!(
            r#"id = "test-mod"
version = "1.0.0"
description = "{}"
"#,
            "x".repeat(MAX_MOD_TOML_BYTES)
        );
        let result = ModManifest::from_toml(&large_toml);
        assert!(result.is_err());
    }
}
