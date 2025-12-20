//! Configuration parsing for server_mods.toml

use crate::errors::DistributionError;
use crate::ModId;
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Server mod distribution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionConfig {
    /// Configured registries in priority order
    #[serde(default)]
    pub registries: Vec<RegistryConfig>,

    /// Required mods with version constraints
    #[serde(default)]
    pub mods: Vec<ModRequirement>,

    /// Trust policy for signatures
    #[serde(default)]
    pub trust: TrustPolicy,

    /// Download settings
    #[serde(default)]
    pub download: DownloadSettings,

    /// Cache settings
    #[serde(default)]
    pub cache: CacheSettings,

    /// Join policy (Feature 037)
    #[serde(default)]
    pub join_policy: JoinPolicy,

    /// Sync configuration (Feature 037)
    #[serde(default)]
    pub sync: SyncConfig,
}

impl DistributionConfig {
    /// Load configuration from a TOML file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, DistributionError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            DistributionError::config_error(&format!(
                "Failed to read config file '{}': {}",
                path.display(),
                e
            ))
        })?;
        Self::from_toml(&content)
    }

    /// Parse configuration from TOML string
    pub fn from_toml(content: &str) -> Result<Self, DistributionError> {
        let config: Self = toml::from_str(content)?;
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration
    fn validate(&self) -> Result<(), DistributionError> {
        // Validate registries
        for reg in &self.registries {
            if reg.name.is_empty() {
                return Err(DistributionError::config_error(
                    "Registry name cannot be empty",
                ));
            }
            if reg.url.is_empty() {
                return Err(DistributionError::config_error(&format!(
                    "Registry '{}' has empty URL",
                    reg.name
                )));
            }
        }

        // Validate mod requirements
        for req in &self.mods {
            // ModId validation happens during deserialization via the serde impl
            // but we can do additional checks here
            if req.id.as_str().is_empty() {
                return Err(DistributionError::config_error("Mod ID cannot be empty"));
            }
        }

        // Validate join policy (Feature 037)
        self.join_policy.validate()?;

        // Validate sync config (Feature 037)
        self.sync.validate()?;

        Ok(())
    }

    /// Get enabled registries sorted by priority (lower = higher priority)
    pub fn enabled_registries(&self) -> Vec<&RegistryConfig> {
        let mut registries: Vec<_> = self.registries.iter().filter(|r| r.enabled).collect();
        registries.sort_by_key(|r| r.priority);
        registries
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> PathBuf {
        self.cache.path.clone().unwrap_or_else(default_cache_dir)
    }
}

impl Default for DistributionConfig {
    fn default() -> Self {
        Self {
            registries: Vec::new(),
            mods: Vec::new(),
            trust: TrustPolicy::default(),
            download: DownloadSettings::default(),
            cache: CacheSettings::default(),
            join_policy: JoinPolicy::default(),
            sync: SyncConfig::default(),
        }
    }
}

/// Registry source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Registry name (for logging/errors)
    pub name: String,

    /// Registry URL or local path
    /// - HTTP(S) URL: "https://mods.example.com/index.json"
    /// - Local path: "file:///path/to/registry" or "/path/to/registry"
    pub url: String,

    /// Priority (lower = higher priority, default 100)
    #[serde(default = "default_priority")]
    pub priority: u32,

    /// Whether this registry is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_priority() -> u32 {
    100
}

fn default_true() -> bool {
    true
}

impl RegistryConfig {
    /// Check if this is a local registry (file:// or absolute path)
    pub fn is_local(&self) -> bool {
        self.url.starts_with("file://") || self.url.starts_with('/')
    }

    /// Get the local path if this is a local registry
    pub fn local_path(&self) -> Option<PathBuf> {
        if self.url.starts_with("file://") {
            Some(PathBuf::from(self.url.strip_prefix("file://").unwrap()))
        } else if self.url.starts_with('/') {
            Some(PathBuf::from(&self.url))
        } else {
            None
        }
    }

    /// Get the index URL (for HTTP registries, append /index.json if needed)
    pub fn index_url(&self) -> String {
        if self.is_local() {
            // For local registries, return path to index.json
            let path = self.local_path().unwrap();
            path.join("index.json").to_string_lossy().to_string()
        } else {
            // For HTTP registries, use the URL as-is (should point to index.json)
            self.url.clone()
        }
    }
}

/// A mod requirement from server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModRequirement {
    /// Mod ID
    pub id: ModId,

    /// Version constraint (SemVer)
    #[serde(with = "version_req_serde")]
    pub version: VersionReq,

    /// Pin to exact version (ignores newer compatible versions)
    #[serde(default)]
    pub pinned: bool,

    /// Whether this mod is optional
    #[serde(default)]
    pub optional: bool,
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

/// Trust policy for mod signatures
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrustPolicy {
    /// Require valid signatures on all mods
    #[serde(default)]
    pub require_signature: bool,

    /// Allowed signing key IDs (hex-encoded public keys)
    /// Empty = allow any valid signature
    #[serde(default)]
    pub allowed_keys: Vec<String>,
}

impl TrustPolicy {
    /// Check if a key ID is allowed
    pub fn is_key_allowed(&self, key_id: &str) -> bool {
        self.allowed_keys.is_empty() || self.allowed_keys.contains(&key_id.to_string())
    }
}

/// Download behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSettings {
    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,

    /// Read timeout in seconds
    #[serde(default = "default_read_timeout")]
    pub read_timeout_secs: u64,

    /// Maximum retry attempts
    #[serde(default = "default_retries")]
    pub retries: u32,

    /// Maximum bundle size in bytes
    #[serde(default = "default_max_size")]
    pub max_bundle_size: u64,
}

fn default_connect_timeout() -> u64 {
    30
}

fn default_read_timeout() -> u64 {
    120
}

fn default_retries() -> u32 {
    3
}

fn default_max_size() -> u64 {
    50 * 1024 * 1024 // 50 MB
}

impl Default for DownloadSettings {
    fn default() -> Self {
        Self {
            connect_timeout_secs: default_connect_timeout(),
            read_timeout_secs: default_read_timeout(),
            retries: default_retries(),
            max_bundle_size: default_max_size(),
        }
    }
}

/// Cache directory settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheSettings {
    /// Cache directory path (default: ~/.local/share/plix/mods/)
    #[serde(default)]
    pub path: Option<PathBuf>,

    /// Maximum cache size in bytes (0 = unlimited)
    #[serde(default)]
    pub max_size: u64,
}

/// Get the default cache directory
fn default_cache_dir() -> PathBuf {
    dirs_next::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("plix")
        .join("mods")
}

/// Join policy for client connections (Feature 037: Server Mods + Client Sync)
///
/// Controls how the server handles clients with different mod sync capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinPolicy {
    /// Allow clients that only support server-only mods (no sync capability)
    #[serde(default = "default_true")]
    pub allow_server_only: bool,

    /// Allow clients to receive mod payloads via sync
    #[serde(default = "default_true")]
    pub allow_payload_sync: bool,

    /// Require clients to sync mod payloads (reject clients without sync support)
    #[serde(default)]
    pub require_payload_sync: bool,
}

impl Default for JoinPolicy {
    fn default() -> Self {
        Self {
            allow_server_only: true,
            allow_payload_sync: true,
            require_payload_sync: false,
        }
    }
}

impl JoinPolicy {
    /// Validate policy configuration
    pub fn validate(&self) -> Result<(), DistributionError> {
        // require_payload_sync implies !allow_server_only
        if self.require_payload_sync && self.allow_server_only {
            return Err(DistributionError::config_error(
                "require_payload_sync=true is incompatible with allow_server_only=true",
            ));
        }
        // require_payload_sync implies allow_payload_sync
        if self.require_payload_sync && !self.allow_payload_sync {
            return Err(DistributionError::config_error(
                "require_payload_sync=true requires allow_payload_sync=true",
            ));
        }
        Ok(())
    }
}

/// Sync configuration for client mod payload transfer (Feature 037)
///
/// Controls bandwidth and resource usage during mod payload synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Maximum payload size per mod in megabytes
    #[serde(default = "default_max_payload_mb")]
    pub max_payload_mb: u32,

    /// Chunk size for payload transfer in kilobytes
    #[serde(default = "default_chunk_size_kb")]
    pub chunk_size_kb: u32,

    /// Maximum number of in-flight (unacknowledged) chunks
    #[serde(default = "default_max_inflight")]
    pub max_inflight_chunks: u8,

    /// Transfer timeout in seconds (per payload)
    #[serde(default = "default_transfer_timeout")]
    pub transfer_timeout_secs: u32,
}

fn default_max_payload_mb() -> u32 {
    25
}

fn default_chunk_size_kb() -> u32 {
    256
}

fn default_max_inflight() -> u8 {
    8
}

fn default_transfer_timeout() -> u32 {
    300
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            max_payload_mb: default_max_payload_mb(),
            chunk_size_kb: default_chunk_size_kb(),
            max_inflight_chunks: default_max_inflight(),
            transfer_timeout_secs: default_transfer_timeout(),
        }
    }
}

impl SyncConfig {
    /// Validate sync configuration
    pub fn validate(&self) -> Result<(), DistributionError> {
        if self.max_payload_mb == 0 {
            return Err(DistributionError::config_error(
                "max_payload_mb must be greater than 0",
            ));
        }
        if self.max_payload_mb > 100 {
            return Err(DistributionError::config_error(
                "max_payload_mb cannot exceed 100 MB",
            ));
        }
        if self.chunk_size_kb == 0 {
            return Err(DistributionError::config_error(
                "chunk_size_kb must be greater than 0",
            ));
        }
        if self.chunk_size_kb > 1024 {
            return Err(DistributionError::config_error(
                "chunk_size_kb cannot exceed 1024 KB (1 MB)",
            ));
        }
        if self.max_inflight_chunks == 0 {
            return Err(DistributionError::config_error(
                "max_inflight_chunks must be greater than 0",
            ));
        }
        if self.transfer_timeout_secs < 30 {
            return Err(DistributionError::config_error(
                "transfer_timeout_secs must be at least 30 seconds",
            ));
        }
        Ok(())
    }

    /// Get the chunk size in bytes
    pub fn chunk_size_bytes(&self) -> usize {
        (self.chunk_size_kb as usize) * 1024
    }

    /// Get the maximum payload size in bytes
    pub fn max_payload_bytes(&self) -> usize {
        (self.max_payload_mb as usize) * 1024 * 1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"
[[registries]]
name = "official"
url = "https://mods.plix.dev/index.json"

[[mods]]
id = "my-mod"
version = "^1.0"
"#;
        let config = DistributionConfig::from_toml(toml).unwrap();
        assert_eq!(config.registries.len(), 1);
        assert_eq!(config.registries[0].name, "official");
        assert_eq!(config.registries[0].priority, 100); // default
        assert!(config.registries[0].enabled); // default
        assert_eq!(config.mods.len(), 1);
        assert_eq!(config.mods[0].id.as_str(), "my-mod");
    }

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
[[registries]]
name = "local"
url = "/srv/plix/mods"
priority = 1
enabled = true

[[registries]]
name = "official"
url = "https://mods.plix.dev/index.json"
priority = 100

[[mods]]
id = "core-lib"
version = "^1.0"
pinned = false

[[mods]]
id = "weapons-pack"
version = "=2.0.0"
pinned = true

[trust]
require_signature = true
allowed_keys = ["abc123def456abc1"]

[download]
connect_timeout_secs = 15
read_timeout_secs = 60
retries = 5
max_bundle_size = 104857600

[cache]
path = "/tmp/plix-mods"
max_size = 1073741824
"#;
        let config = DistributionConfig::from_toml(toml).unwrap();
        assert_eq!(config.registries.len(), 2);
        assert_eq!(config.mods.len(), 2);
        assert!(config.trust.require_signature);
        assert_eq!(config.trust.allowed_keys.len(), 1);
        assert_eq!(config.download.connect_timeout_secs, 15);
        assert_eq!(config.download.retries, 5);
        assert_eq!(
            config.cache.path,
            Some(PathBuf::from("/tmp/plix-mods"))
        );
    }

    #[test]
    fn test_enabled_registries_sorted() {
        let toml = r#"
[[registries]]
name = "low-priority"
url = "https://example.com/index.json"
priority = 100

[[registries]]
name = "high-priority"
url = "https://priority.com/index.json"
priority = 10

[[registries]]
name = "disabled"
url = "https://disabled.com/index.json"
priority = 1
enabled = false
"#;
        let config = DistributionConfig::from_toml(toml).unwrap();
        let enabled = config.enabled_registries();
        assert_eq!(enabled.len(), 2);
        assert_eq!(enabled[0].name, "high-priority");
        assert_eq!(enabled[1].name, "low-priority");
    }

    #[test]
    fn test_registry_is_local() {
        let file_reg = RegistryConfig {
            name: "local".to_string(),
            url: "file:///srv/mods".to_string(),
            priority: 1,
            enabled: true,
        };
        assert!(file_reg.is_local());
        assert_eq!(
            file_reg.local_path(),
            Some(PathBuf::from("/srv/mods"))
        );

        let path_reg = RegistryConfig {
            name: "local".to_string(),
            url: "/srv/mods".to_string(),
            priority: 1,
            enabled: true,
        };
        assert!(path_reg.is_local());
        assert_eq!(path_reg.local_path(), Some(PathBuf::from("/srv/mods")));

        let http_reg = RegistryConfig {
            name: "remote".to_string(),
            url: "https://example.com/index.json".to_string(),
            priority: 100,
            enabled: true,
        };
        assert!(!http_reg.is_local());
        assert_eq!(http_reg.local_path(), None);
    }

    #[test]
    fn test_invalid_config() {
        // Empty registry name
        let toml = r#"
[[registries]]
name = ""
url = "https://example.com"
"#;
        assert!(DistributionConfig::from_toml(toml).is_err());

        // Empty URL
        let toml = r#"
[[registries]]
name = "test"
url = ""
"#;
        assert!(DistributionConfig::from_toml(toml).is_err());
    }

    #[test]
    fn test_trust_policy() {
        let policy = TrustPolicy {
            require_signature: true,
            allowed_keys: vec!["key1".to_string(), "key2".to_string()],
        };
        assert!(policy.is_key_allowed("key1"));
        assert!(policy.is_key_allowed("key2"));
        assert!(!policy.is_key_allowed("key3"));

        // Empty allowed_keys means any key is allowed
        let open_policy = TrustPolicy {
            require_signature: true,
            allowed_keys: vec![],
        };
        assert!(open_policy.is_key_allowed("any-key"));
    }

    #[test]
    fn test_join_policy_default() {
        let policy = JoinPolicy::default();
        assert!(policy.allow_server_only);
        assert!(policy.allow_payload_sync);
        assert!(!policy.require_payload_sync);
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_join_policy_validation() {
        // Valid: require_payload_sync with allow_server_only=false
        let policy = JoinPolicy {
            allow_server_only: false,
            allow_payload_sync: true,
            require_payload_sync: true,
        };
        assert!(policy.validate().is_ok());

        // Invalid: require_payload_sync with allow_server_only=true
        let policy = JoinPolicy {
            allow_server_only: true,
            allow_payload_sync: true,
            require_payload_sync: true,
        };
        assert!(policy.validate().is_err());

        // Invalid: require_payload_sync without allow_payload_sync
        let policy = JoinPolicy {
            allow_server_only: false,
            allow_payload_sync: false,
            require_payload_sync: true,
        };
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert_eq!(config.max_payload_mb, 25);
        assert_eq!(config.chunk_size_kb, 256);
        assert_eq!(config.max_inflight_chunks, 8);
        assert_eq!(config.transfer_timeout_secs, 300);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_sync_config_validation() {
        // Invalid: zero max_payload_mb
        let config = SyncConfig {
            max_payload_mb: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid: max_payload_mb too large
        let config = SyncConfig {
            max_payload_mb: 101,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid: zero chunk_size_kb
        let config = SyncConfig {
            chunk_size_kb: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid: chunk_size_kb too large
        let config = SyncConfig {
            chunk_size_kb: 1025,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid: zero max_inflight_chunks
        let config = SyncConfig {
            max_inflight_chunks: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid: transfer_timeout too short
        let config = SyncConfig {
            transfer_timeout_secs: 29,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sync_config_size_helpers() {
        let config = SyncConfig {
            max_payload_mb: 10,
            chunk_size_kb: 128,
            ..Default::default()
        };
        assert_eq!(config.max_payload_bytes(), 10 * 1024 * 1024);
        assert_eq!(config.chunk_size_bytes(), 128 * 1024);
    }

    #[test]
    fn test_parse_config_with_sync_settings() {
        let toml = r#"
[[registries]]
name = "official"
url = "https://mods.plix.dev/index.json"

[join_policy]
allow_server_only = false
allow_payload_sync = true
require_payload_sync = true

[sync]
max_payload_mb = 50
chunk_size_kb = 512
max_inflight_chunks = 4
transfer_timeout_secs = 600
"#;
        let config = DistributionConfig::from_toml(toml).unwrap();
        assert!(!config.join_policy.allow_server_only);
        assert!(config.join_policy.allow_payload_sync);
        assert!(config.join_policy.require_payload_sync);
        assert_eq!(config.sync.max_payload_mb, 50);
        assert_eq!(config.sync.chunk_size_kb, 512);
        assert_eq!(config.sync.max_inflight_chunks, 4);
        assert_eq!(config.sync.transfer_timeout_secs, 600);
    }
}
