//! Build Information
//!
//! Compile-time embedded metadata for version tracking and reproducibility.
//! Uses shadow-rs to provide comprehensive build information at compile time.

use serde::{Deserialize, Serialize};

/// Compile-time embedded build metadata.
///
/// This struct contains information about when and how the binary was built,
/// which is useful for version tracking, debugging, and reproducibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    /// Semantic version from Cargo.toml (e.g., "0.1.0")
    pub version: String,
    /// Full git commit hash (40 characters)
    pub commit_sha: String,
    /// Short git commit hash (7 characters)
    pub commit_sha_short: String,
    /// ISO 8601 UTC timestamp of build time
    pub build_timestamp: String,
    /// Date only (YYYY-MM-DD)
    pub build_date: String,
    /// Rust target triple (e.g., "x86_64-unknown-linux-gnu")
    pub target_triple: String,
    /// Rust compiler version used for build
    pub rust_version: String,
    /// Git branch name at build time
    pub branch: String,
    /// True if there were uncommitted changes at build time
    pub is_dirty: bool,
}

impl BuildInfo {
    /// Create BuildInfo from shadow-rs constants.
    ///
    /// This should be called from the binary's main module where shadow-rs
    /// constants are available.
    ///
    /// # Example
    ///
    /// ```ignore
    /// mod shadow { include!(concat!(env!("OUT_DIR"), "/shadow.rs")); }
    ///
    /// let build_info = BuildInfo::from_shadow(
    ///     shadow::PKG_VERSION,
    ///     shadow::COMMIT_HASH,
    ///     shadow::SHORT_COMMIT,
    ///     shadow::BUILD_TIME,
    ///     shadow::BUILD_RUST_CHANNEL,
    ///     shadow::BRANCH,
    ///     shadow::GIT_CLEAN,
    /// );
    /// ```
    pub fn from_shadow(
        version: &str,
        commit_sha: &str,
        commit_sha_short: &str,
        build_timestamp: &str,
        rust_version: &str,
        branch: &str,
        is_clean: bool,
        target_triple: &str,
    ) -> Self {
        // Extract date from timestamp (format: "2025-12-19 14:30:00 UTC")
        let build_date = build_timestamp
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string();

        Self {
            version: version.to_string(),
            commit_sha: commit_sha.to_string(),
            commit_sha_short: commit_sha_short.to_string(),
            build_timestamp: build_timestamp.to_string(),
            build_date,
            target_triple: target_triple.to_string(),
            rust_version: rust_version.to_string(),
            branch: branch.to_string(),
            is_dirty: !is_clean,
        }
    }

    /// Display version string for CLI output.
    ///
    /// Format: "0.1.0 (abc1234) built 2025-12-19"
    pub fn display_version(&self) -> String {
        let dirty_suffix = if self.is_dirty { " (dirty)" } else { "" };
        format!(
            "{} ({}) built {}{}",
            self.version, self.commit_sha_short, self.build_date, dirty_suffix
        )
    }

    /// Serialize to JSON for bundle metadata.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serialize to compact JSON (single line).
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl std::fmt::Display for BuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_version())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_shadow() {
        let info = BuildInfo::from_shadow(
            "0.1.0",
            "abc1234567890def1234567890abc1234567890ab",
            "abc1234",
            "2025-12-19 14:30:00 UTC",
            "1.83.0",
            "main",
            true,
            "x86_64-unknown-linux-gnu",
        );

        assert_eq!(info.version, "0.1.0");
        assert_eq!(info.commit_sha_short, "abc1234");
        assert_eq!(info.build_date, "2025-12-19");
        assert!(!info.is_dirty);
    }

    #[test]
    fn test_display_version() {
        let info = BuildInfo::from_shadow(
            "0.1.0",
            "abc1234567890def1234567890abc1234567890ab",
            "abc1234",
            "2025-12-19 14:30:00 UTC",
            "1.83.0",
            "main",
            true,
            "x86_64-unknown-linux-gnu",
        );

        assert_eq!(info.display_version(), "0.1.0 (abc1234) built 2025-12-19");
    }

    #[test]
    fn test_display_version_dirty() {
        let info = BuildInfo::from_shadow(
            "0.1.0",
            "abc1234567890def1234567890abc1234567890ab",
            "abc1234",
            "2025-12-19 14:30:00 UTC",
            "1.83.0",
            "main",
            false, // not clean = dirty
            "x86_64-unknown-linux-gnu",
        );

        assert!(info.display_version().contains("(dirty)"));
    }

    #[test]
    fn test_to_json() {
        let info = BuildInfo::from_shadow(
            "0.1.0",
            "abc1234567890def1234567890abc1234567890ab",
            "abc1234",
            "2025-12-19 14:30:00 UTC",
            "1.83.0",
            "main",
            true,
            "x86_64-unknown-linux-gnu",
        );

        let json = info.to_json().unwrap();
        assert!(json.contains("\"version\": \"0.1.0\""));
        assert!(json.contains("\"commit_sha_short\": \"abc1234\""));
    }
}
