//! Version management for world persistence.
//!
//! Defines version constants and compatibility checking logic.

use super::error::VersionCheck;

/// Current save format version.
pub const CURRENT_VERSION: u32 = 1;

/// Minimum supported version for loading (inclusive).
pub const MIN_SUPPORTED_VERSION: u32 = 1;

/// Check version compatibility.
///
/// # Arguments
/// * `version` - The version to check.
///
/// # Returns
/// A `VersionCheck` indicating compatibility status.
pub fn check_version(version: u32) -> VersionCheck {
    if version == CURRENT_VERSION {
        VersionCheck::Current
    } else if version > CURRENT_VERSION {
        VersionCheck::TooNew(version)
    } else if version >= MIN_SUPPORTED_VERSION {
        VersionCheck::NeedsMigration(version)
    } else {
        VersionCheck::TooOld(version)
    }
}

/// Check if a version is supported for loading.
///
/// # Arguments
/// * `version` - The version to check.
///
/// # Returns
/// `true` if the version can be loaded (possibly with migration).
pub fn is_version_supported(version: u32) -> bool {
    version >= MIN_SUPPORTED_VERSION && version <= CURRENT_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_version() {
        assert_eq!(check_version(CURRENT_VERSION), VersionCheck::Current);
        assert!(is_version_supported(CURRENT_VERSION));
    }

    #[test]
    fn test_too_new_version() {
        let future = CURRENT_VERSION + 1;
        assert_eq!(check_version(future), VersionCheck::TooNew(future));
        assert!(!is_version_supported(future));
    }

    #[test]
    fn test_too_old_version() {
        if MIN_SUPPORTED_VERSION > 0 {
            let old = MIN_SUPPORTED_VERSION - 1;
            assert_eq!(check_version(old), VersionCheck::TooOld(old));
            assert!(!is_version_supported(old));
        }

        // Version 0 should always be too old (unless MIN is 0)
        if MIN_SUPPORTED_VERSION > 0 {
            assert_eq!(check_version(0), VersionCheck::TooOld(0));
            assert!(!is_version_supported(0));
        }
    }

    #[test]
    fn test_needs_migration() {
        // When MIN < CURRENT, versions in between need migration
        if MIN_SUPPORTED_VERSION < CURRENT_VERSION {
            let migrate = MIN_SUPPORTED_VERSION;
            assert_eq!(
                check_version(migrate),
                VersionCheck::NeedsMigration(migrate)
            );
            assert!(is_version_supported(migrate));
        }
    }

    #[test]
    fn test_version_constants_valid() {
        // MIN should not be greater than CURRENT
        assert!(MIN_SUPPORTED_VERSION <= CURRENT_VERSION);
        // CURRENT should be at least 1
        assert!(CURRENT_VERSION >= 1);
    }
}
