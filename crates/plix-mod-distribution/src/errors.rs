//! Distribution error types with structured error codes (EMREG001-008)

use std::collections::HashMap;
use std::fmt;

/// Error codes per spec
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// EMREG001: Registry unreachable
    RegistryUnreachable,

    /// EMREG002: Invalid registry index
    InvalidIndex,

    /// EMREG003: Download failed
    DownloadFailed,

    /// EMREG004: Hash mismatch
    HashMismatch,

    /// EMREG005: Signature invalid
    SignatureInvalid,

    /// EMREG006: Dependency conflict
    DependencyConflict,

    /// EMREG007: Version incompatible
    VersionIncompatible,

    /// EMREG008: Dependency cycle detected
    CycleDetected,

    /// Internal error (not part of spec)
    Internal,

    /// Configuration error
    ConfigError,

    /// Invalid mod ID
    InvalidModId,
}

impl ErrorCode {
    /// Get the string code (e.g., "EMREG001")
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RegistryUnreachable => "EMREG001",
            Self::InvalidIndex => "EMREG002",
            Self::DownloadFailed => "EMREG003",
            Self::HashMismatch => "EMREG004",
            Self::SignatureInvalid => "EMREG005",
            Self::DependencyConflict => "EMREG006",
            Self::VersionIncompatible => "EMREG007",
            Self::CycleDetected => "EMREG008",
            Self::Internal => "EINTERNAL",
            Self::ConfigError => "ECONFIG",
            Self::InvalidModId => "EINVALID_ID",
        }
    }

    /// Get a human-readable description of the error code
    pub fn description(&self) -> &'static str {
        match self {
            Self::RegistryUnreachable => "Registry is unreachable",
            Self::InvalidIndex => "Invalid registry index format",
            Self::DownloadFailed => "Bundle download failed",
            Self::HashMismatch => "Bundle hash does not match expected value",
            Self::SignatureInvalid => "Bundle signature is invalid or missing",
            Self::DependencyConflict => "Conflicting dependency requirements",
            Self::VersionIncompatible => "Mod version is incompatible with engine/API",
            Self::CycleDetected => "Circular dependency detected",
            Self::Internal => "Internal error",
            Self::ConfigError => "Configuration error",
            Self::InvalidModId => "Invalid mod identifier format",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Distribution error with structured code
#[derive(Debug, Clone)]
pub struct DistributionError {
    /// Error code (EMREG001-008)
    pub code: ErrorCode,

    /// Human-readable message
    pub message: String,

    /// Affected mod (if applicable)
    pub mod_id: Option<String>,

    /// Additional context
    pub context: HashMap<String, String>,
}

impl DistributionError {
    /// Create a new error with the given code and message
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            mod_id: None,
            context: HashMap::new(),
        }
    }

    /// Add a mod ID to the error
    pub fn with_mod_id(mut self, mod_id: impl Into<String>) -> Self {
        self.mod_id = Some(mod_id.into());
        self
    }

    /// Add context key-value pair
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    // ============ Error constructors ============

    /// EMREG001: Registry unreachable
    pub fn registry_unreachable(registry_name: &str, url: &str, reason: &str) -> Self {
        Self::new(
            ErrorCode::RegistryUnreachable,
            format!("Cannot reach registry '{}': {}", registry_name, reason),
        )
        .with_context("registry", registry_name)
        .with_context("url", url)
        .with_context("reason", reason)
    }

    /// EMREG002: Invalid index format
    pub fn invalid_index(registry_name: &str, reason: &str) -> Self {
        Self::new(
            ErrorCode::InvalidIndex,
            format!("Invalid index in registry '{}': {}", registry_name, reason),
        )
        .with_context("registry", registry_name)
        .with_context("reason", reason)
    }

    /// EMREG003: Download failed
    pub fn download_failed(url: &str, reason: &str) -> Self {
        Self::new(ErrorCode::DownloadFailed, format!("Download failed: {}", reason))
            .with_context("url", url)
            .with_context("reason", reason)
    }

    /// EMREG003: Bundle too large
    pub fn bundle_too_large(url: &str, size: u64, max_size: u64) -> Self {
        Self::new(
            ErrorCode::DownloadFailed,
            format!(
                "Bundle exceeds size limit: {} bytes > {} bytes max",
                size, max_size
            ),
        )
        .with_context("url", url)
        .with_context("size", size.to_string())
        .with_context("max_size", max_size.to_string())
    }

    /// EMREG004: Hash mismatch
    pub fn hash_mismatch(mod_id: &str, expected: &str, actual: &str) -> Self {
        Self::new(
            ErrorCode::HashMismatch,
            format!(
                "Hash mismatch for '{}': expected {}, got {}",
                mod_id, expected, actual
            ),
        )
        .with_mod_id(mod_id)
        .with_context("expected", expected)
        .with_context("actual", actual)
    }

    /// EMREG005: Signature invalid
    pub fn signature_invalid(mod_id: &str, reason: &str) -> Self {
        Self::new(
            ErrorCode::SignatureInvalid,
            format!("Invalid signature for '{}': {}", mod_id, reason),
        )
        .with_mod_id(mod_id)
        .with_context("reason", reason)
    }

    /// EMREG005: Signature missing when required
    pub fn signature_missing(mod_id: &str) -> Self {
        Self::new(
            ErrorCode::SignatureInvalid,
            format!("Signature required but missing for '{}'", mod_id),
        )
        .with_mod_id(mod_id)
        .with_context("reason", "missing")
    }

    /// EMREG005: Signature key not in allowed list
    pub fn signature_key_not_allowed(mod_id: &str, key_id: &str) -> Self {
        Self::new(
            ErrorCode::SignatureInvalid,
            format!(
                "Signature key '{}' for '{}' is not in allowed keys list",
                key_id, mod_id
            ),
        )
        .with_mod_id(mod_id)
        .with_context("key_id", key_id)
        .with_context("reason", "key_not_allowed")
    }

    /// EMREG006: Dependency conflict
    pub fn dependency_conflict(
        mod_a: &str,
        mod_b: &str,
        dependency: &str,
        req_a: &str,
        req_b: &str,
    ) -> Self {
        Self::new(
            ErrorCode::DependencyConflict,
            format!(
                "Conflicting requirements for '{}': '{}' requires {}, '{}' requires {}",
                dependency, mod_a, req_a, mod_b, req_b
            ),
        )
        .with_context("dependency", dependency)
        .with_context("mod_a", mod_a)
        .with_context("mod_b", mod_b)
        .with_context("requirement_a", req_a)
        .with_context("requirement_b", req_b)
    }

    /// EMREG007: API version incompatible
    pub fn api_version_incompatible(mod_id: &str, required: u32, available: u32) -> Self {
        Self::new(
            ErrorCode::VersionIncompatible,
            format!(
                "Mod '{}' requires API version {}, but engine only supports version {}",
                mod_id, required, available
            ),
        )
        .with_mod_id(mod_id)
        .with_context("required_api", required.to_string())
        .with_context("available_api", available.to_string())
    }

    /// EMREG007: Engine version incompatible
    pub fn engine_version_incompatible(
        mod_id: &str,
        current: &str,
        min: Option<&str>,
        max: Option<&str>,
    ) -> Self {
        let constraint = match (min, max) {
            (Some(min), Some(max)) => format!("{} <= version <= {}", min, max),
            (Some(min), None) => format!("version >= {}", min),
            (None, Some(max)) => format!("version <= {}", max),
            (None, None) => "any".to_string(),
        };
        Self::new(
            ErrorCode::VersionIncompatible,
            format!(
                "Mod '{}' requires engine {}, but current version is {}",
                mod_id, constraint, current
            ),
        )
        .with_mod_id(mod_id)
        .with_context("current_version", current)
        .with_context("constraint", constraint)
    }

    /// EMREG008: Cycle detected
    pub fn cycle_detected(cycle: &[String]) -> Self {
        Self::new(
            ErrorCode::CycleDetected,
            format!("Dependency cycle detected: {}", cycle.join(" → ")),
        )
        .with_context("cycle", cycle.join(" → "))
    }

    /// Invalid mod ID
    pub fn invalid_mod_id(id: &str) -> Self {
        Self::new(
            ErrorCode::InvalidModId,
            format!(
                "Invalid mod ID '{}': must be lowercase alphanumeric with hyphens, 1-64 chars",
                id
            ),
        )
        .with_context("id", id)
    }

    /// Configuration error
    pub fn config_error(reason: &str) -> Self {
        Self::new(ErrorCode::ConfigError, format!("Configuration error: {}", reason))
            .with_context("reason", reason)
    }

    /// Internal error
    pub fn internal(reason: &str) -> Self {
        Self::new(ErrorCode::Internal, format!("Internal error: {}", reason))
            .with_context("reason", reason)
    }

    /// Mod not found in any registry
    pub fn mod_not_found(mod_id: &str) -> Self {
        Self::new(
            ErrorCode::InvalidIndex,
            format!("Mod '{}' not found in any configured registry", mod_id),
        )
        .with_mod_id(mod_id)
    }

    /// Version not found for mod
    pub fn version_not_found(mod_id: &str, version_req: &str) -> Self {
        Self::new(
            ErrorCode::InvalidIndex,
            format!(
                "No version of '{}' matches requirement '{}'",
                mod_id, version_req
            ),
        )
        .with_mod_id(mod_id)
        .with_context("requirement", version_req)
    }
}

impl fmt::Display for DistributionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(ref mod_id) = self.mod_id {
            write!(f, " (mod: {})", mod_id)?;
        }
        Ok(())
    }
}

impl std::error::Error for DistributionError {}

// Conversion from common error types
impl From<std::io::Error> for DistributionError {
    fn from(err: std::io::Error) -> Self {
        Self::internal(&err.to_string())
    }
}

impl From<serde_json::Error> for DistributionError {
    fn from(err: serde_json::Error) -> Self {
        Self::invalid_index("unknown", &err.to_string())
    }
}

impl From<toml::de::Error> for DistributionError {
    fn from(err: toml::de::Error) -> Self {
        Self::config_error(&err.to_string())
    }
}

impl From<reqwest::Error> for DistributionError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_connect() || err.is_timeout() {
            Self::new(
                ErrorCode::RegistryUnreachable,
                format!("Network error: {}", err),
            )
        } else {
            Self::new(ErrorCode::DownloadFailed, format!("HTTP error: {}", err))
        }
    }
}

impl From<zip::result::ZipError> for DistributionError {
    fn from(err: zip::result::ZipError) -> Self {
        Self::internal(&format!("Zip error: {}", err))
    }
}

impl From<semver::Error> for DistributionError {
    fn from(err: semver::Error) -> Self {
        Self::config_error(&format!("Invalid version: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_as_str() {
        assert_eq!(ErrorCode::RegistryUnreachable.as_str(), "EMREG001");
        assert_eq!(ErrorCode::InvalidIndex.as_str(), "EMREG002");
        assert_eq!(ErrorCode::DownloadFailed.as_str(), "EMREG003");
        assert_eq!(ErrorCode::HashMismatch.as_str(), "EMREG004");
        assert_eq!(ErrorCode::SignatureInvalid.as_str(), "EMREG005");
        assert_eq!(ErrorCode::DependencyConflict.as_str(), "EMREG006");
        assert_eq!(ErrorCode::VersionIncompatible.as_str(), "EMREG007");
        assert_eq!(ErrorCode::CycleDetected.as_str(), "EMREG008");
    }

    #[test]
    fn test_error_creation() {
        let err = DistributionError::hash_mismatch("my-mod", "abc123", "def456");
        assert_eq!(err.code, ErrorCode::HashMismatch);
        assert!(err.message.contains("my-mod"));
        assert_eq!(err.mod_id, Some("my-mod".to_string()));
        assert_eq!(err.context.get("expected"), Some(&"abc123".to_string()));
        assert_eq!(err.context.get("actual"), Some(&"def456".to_string()));
    }

    #[test]
    fn test_error_display() {
        let err = DistributionError::registry_unreachable("official", "https://mods.example.com", "connection refused");
        let display = err.to_string();
        assert!(display.contains("EMREG001"));
        assert!(display.contains("official"));
    }

    #[test]
    fn test_cycle_detected() {
        let cycle = vec!["mod-a".to_string(), "mod-b".to_string(), "mod-a".to_string()];
        let err = DistributionError::cycle_detected(&cycle);
        assert_eq!(err.code, ErrorCode::CycleDetected);
        assert!(err.message.contains("mod-a → mod-b → mod-a"));
    }

    #[test]
    fn test_dependency_conflict() {
        let err = DistributionError::dependency_conflict(
            "mod-a",
            "mod-b",
            "shared-lib",
            "^1.0",
            "^2.0",
        );
        assert_eq!(err.code, ErrorCode::DependencyConflict);
        assert!(err.message.contains("shared-lib"));
        assert!(err.message.contains("^1.0"));
        assert!(err.message.contains("^2.0"));
    }
}
