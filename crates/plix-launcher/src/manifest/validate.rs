//! Manifest validation

use crate::error::LauncherError;
use crate::manifest::Manifest;

/// Validate a manifest according to the schema rules
pub fn validate_manifest(manifest: &Manifest) -> Result<(), LauncherError> {
    // Check manifest version
    if manifest.manifest_version != 1 {
        return Err(LauncherError::Manifest(format!(
            "Unsupported manifest version: {} (expected 1)",
            manifest.manifest_version
        )));
    }

    // Check version format (basic semver check)
    if !is_valid_semver(&manifest.version) {
        return Err(LauncherError::Manifest(format!(
            "Invalid version format: {}",
            manifest.version
        )));
    }

    // Check release date
    if manifest.release_date == 0 {
        return Err(LauncherError::Manifest("Invalid release_date: 0".into()));
    }

    // Check files array is not empty
    if manifest.files.is_empty() {
        return Err(LauncherError::Manifest("Manifest contains no files".into()));
    }

    // Validate each file entry
    for file in &manifest.files {
        validate_file(file)?;
    }

    // Check for duplicate paths
    let mut seen_paths = std::collections::HashSet::new();
    for file in &manifest.files {
        if !seen_paths.insert(&file.path) {
            return Err(LauncherError::Manifest(format!(
                "Duplicate file path: {}",
                file.path
            )));
        }
    }

    Ok(())
}

fn validate_file(file: &crate::manifest::ManifestFile) -> Result<(), LauncherError> {
    // Check path is relative (no leading / or ..)
    if file.path.starts_with('/') || file.path.starts_with("..") || file.path.contains("/../") {
        return Err(LauncherError::Manifest(format!(
            "Invalid file path (must be relative): {}",
            file.path
        )));
    }

    // Check path is not empty
    if file.path.is_empty() {
        return Err(LauncherError::Manifest("Empty file path".into()));
    }

    // Check URL is valid HTTP/HTTPS
    if !file.url.starts_with("http://") && !file.url.starts_with("https://") {
        return Err(LauncherError::Manifest(format!(
            "Invalid URL (must be HTTP/HTTPS): {}",
            file.url
        )));
    }

    // Check size is positive
    if file.size == 0 {
        return Err(LauncherError::Manifest(format!(
            "Invalid file size (must be > 0) for: {}",
            file.path
        )));
    }

    // Check sha256 is 64 hex characters
    if file.sha256.len() != 64 || !file.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(LauncherError::Manifest(format!(
            "Invalid checksum for file: {} (expected 64 hex characters)",
            file.path
        )));
    }

    Ok(())
}

/// Basic semver validation
fn is_valid_semver(version: &str) -> bool {
    semver::Version::parse(version).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Manifest, ManifestFile};

    fn valid_manifest() -> Manifest {
        Manifest {
            manifest_version: 1,
            version: "1.0.0".to_string(),
            protocol_version: None,
            release_date: 1734480000,
            files: vec![ManifestFile {
                path: "plix-client".to_string(),
                url: "https://example.com/plix-client".to_string(),
                size: 1000,
                sha256: "a".repeat(64),
                executable: true,
            }],
            release_notes_url: None,
        }
    }

    #[test]
    fn test_valid_manifest() {
        let manifest = valid_manifest();
        assert!(validate_manifest(&manifest).is_ok());
    }

    #[test]
    fn test_invalid_manifest_version() {
        let mut manifest = valid_manifest();
        manifest.manifest_version = 2;
        let result = validate_manifest(&manifest);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported manifest version"));
    }

    #[test]
    fn test_invalid_version_format() {
        let mut manifest = valid_manifest();
        manifest.version = "not-semver".to_string();
        let result = validate_manifest(&manifest);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid version format"));
    }

    #[test]
    fn test_empty_files() {
        let mut manifest = valid_manifest();
        manifest.files.clear();
        let result = validate_manifest(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no files"));
    }

    #[test]
    fn test_invalid_path_absolute() {
        let mut manifest = valid_manifest();
        manifest.files[0].path = "/absolute/path".to_string();
        let result = validate_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_path_parent_dir() {
        let mut manifest = valid_manifest();
        manifest.files[0].path = "../escape".to_string();
        let result = validate_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_url() {
        let mut manifest = valid_manifest();
        manifest.files[0].url = "ftp://invalid.example".to_string();
        let result = validate_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_checksum_length() {
        let mut manifest = valid_manifest();
        manifest.files[0].sha256 = "tooshort".to_string();
        let result = validate_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_checksum_chars() {
        let mut manifest = valid_manifest();
        manifest.files[0].sha256 = "g".repeat(64); // 'g' is not hex
        let result = validate_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_paths() {
        let mut manifest = valid_manifest();
        manifest.files.push(manifest.files[0].clone());
        let result = validate_manifest(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }
}
