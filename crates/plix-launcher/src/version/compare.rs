//! Version comparison utilities

use crate::error::LauncherError;
use std::cmp::Ordering;

/// Wrapper around semver::Version for convenience
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version(semver::Version);

impl Version {
    /// Parse version from string
    pub fn parse(s: &str) -> Result<Self, LauncherError> {
        semver::Version::parse(s)
            .map(Version)
            .map_err(|e| LauncherError::Version(format!("Invalid version '{}': {}", s, e)))
    }

    /// Compare two versions
    pub fn compare(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }

    /// Check if self is newer than other
    pub fn is_newer_than(&self, other: &Self) -> bool {
        self.0 > other.0
    }

    /// Check if versions are equal
    pub fn is_equal(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    /// Get the inner semver::Version
    pub fn inner(&self) -> &semver::Version {
        &self.0
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_versions() {
        assert!(Version::parse("1.0.0").is_ok());
        assert!(Version::parse("1.2.3").is_ok());
        assert!(Version::parse("10.20.30").is_ok());
        assert!(Version::parse("1.0.0-alpha").is_ok());
        assert!(Version::parse("1.0.0-beta.1").is_ok());
        assert!(Version::parse("1.0.0+build").is_ok());
    }

    #[test]
    fn test_parse_invalid_versions() {
        assert!(Version::parse("").is_err());
        assert!(Version::parse("1").is_err());
        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("not-a-version").is_err());
        assert!(Version::parse("v1.0.0").is_err()); // no v prefix
    }

    #[test]
    fn test_version_comparison() {
        let v1_0_0 = Version::parse("1.0.0").unwrap();
        let v1_0_1 = Version::parse("1.0.1").unwrap();
        let v1_1_0 = Version::parse("1.1.0").unwrap();
        let v2_0_0 = Version::parse("2.0.0").unwrap();

        assert_eq!(v1_0_0.compare(&v1_0_0), Ordering::Equal);
        assert_eq!(v1_0_0.compare(&v1_0_1), Ordering::Less);
        assert_eq!(v1_0_1.compare(&v1_0_0), Ordering::Greater);
        assert_eq!(v1_0_0.compare(&v1_1_0), Ordering::Less);
        assert_eq!(v1_0_0.compare(&v2_0_0), Ordering::Less);
    }

    #[test]
    fn test_is_newer_than() {
        let v1_0_0 = Version::parse("1.0.0").unwrap();
        let v1_0_1 = Version::parse("1.0.1").unwrap();
        let v2_0_0 = Version::parse("2.0.0").unwrap();

        assert!(!v1_0_0.is_newer_than(&v1_0_0)); // equal
        assert!(!v1_0_0.is_newer_than(&v1_0_1)); // older
        assert!(v1_0_1.is_newer_than(&v1_0_0)); // newer
        assert!(v2_0_0.is_newer_than(&v1_0_0)); // much newer
    }

    #[test]
    fn test_prerelease_versions() {
        let alpha = Version::parse("1.0.0-alpha").unwrap();
        let beta = Version::parse("1.0.0-beta").unwrap();
        let release = Version::parse("1.0.0").unwrap();

        assert!(alpha < beta);
        assert!(beta < release);
        assert!(release.is_newer_than(&alpha));
        assert!(release.is_newer_than(&beta));
    }

    #[test]
    fn test_version_display() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(format!("{}", v), "1.2.3");
    }
}
