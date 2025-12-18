//! Version management

mod compare;
mod local;

pub use compare::*;
pub use local::*;

use crate::error::LauncherError;
use crate::manifest::{Manifest, ManifestFile};

/// Result of checking for updates
#[derive(Debug, Clone)]
pub enum UpdateStatus {
    /// Already up to date
    UpToDate { version: String },

    /// Update available
    UpdateAvailable {
        current: String,
        latest: String,
        files_to_update: Vec<ManifestFile>,
        total_download_size: u64,
    },

    /// Fresh installation required
    FreshInstall {
        version: String,
        files: Vec<ManifestFile>,
        total_download_size: u64,
    },

    /// Offline mode (manifest unreachable but local version exists)
    Offline { local_version: String },

    /// Cannot proceed (no local version and offline)
    CannotProceed { reason: String },
}

impl UpdateStatus {
    /// Check if we can launch the game with this status
    pub fn can_launch(&self) -> bool {
        matches!(
            self,
            UpdateStatus::UpToDate { .. }
                | UpdateStatus::UpdateAvailable { .. }
                | UpdateStatus::Offline { .. }
        )
    }

    /// Check if an update/install is required before launch
    pub fn needs_download(&self) -> bool {
        matches!(
            self,
            UpdateStatus::UpdateAvailable { .. } | UpdateStatus::FreshInstall { .. }
        )
    }
}

/// Determine update status by comparing local state with remote manifest
pub fn check_for_updates(
    local_state: &LocalState,
    manifest: &Manifest,
) -> Result<UpdateStatus, LauncherError> {
    // Fresh install case
    if local_state.installed_version.is_none() {
        let total_size: u64 = manifest.files.iter().map(|f| f.size).sum();
        return Ok(UpdateStatus::FreshInstall {
            version: manifest.version.clone(),
            files: manifest.files.clone(),
            total_download_size: total_size,
        });
    }

    let local_version_str = local_state.installed_version.as_ref().unwrap();
    let local_version = Version::parse(local_version_str)?;
    let remote_version = Version::parse(&manifest.version)?;

    // Up to date
    if !remote_version.is_newer_than(&local_version) {
        return Ok(UpdateStatus::UpToDate {
            version: local_version_str.clone(),
        });
    }

    // Update available - determine which files need updating
    let files_to_update = find_files_to_update(local_state, manifest);
    let total_size: u64 = files_to_update.iter().map(|f| f.size).sum();

    Ok(UpdateStatus::UpdateAvailable {
        current: local_version_str.clone(),
        latest: manifest.version.clone(),
        files_to_update,
        total_download_size: total_size,
    })
}

/// Find files that need to be downloaded (missing or checksum mismatch)
pub fn find_files_to_update(local_state: &LocalState, manifest: &Manifest) -> Vec<ManifestFile> {
    manifest
        .files
        .iter()
        .filter(|file| {
            // File needs update if:
            // 1. Not in local checksums (new file)
            // 2. Checksum doesn't match (modified file)
            match local_state.file_checksums.get(&file.path) {
                None => true,
                Some(local_checksum) => local_checksum != &file.sha256,
            }
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manifest(version: &str) -> Manifest {
        Manifest {
            manifest_version: 1,
            version: version.to_string(),
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
    fn test_fresh_install() {
        let state = LocalState::default();
        let manifest = test_manifest("1.0.0");

        let status = check_for_updates(&state, &manifest).unwrap();
        match status {
            UpdateStatus::FreshInstall {
                version,
                files,
                total_download_size,
            } => {
                assert_eq!(version, "1.0.0");
                assert_eq!(files.len(), 1);
                assert_eq!(total_download_size, 1000);
            }
            _ => panic!("Expected FreshInstall, got {:?}", status),
        }
    }

    #[test]
    fn test_up_to_date() {
        let mut state = LocalState::default();
        state.installed_version = Some("1.0.0".to_string());
        state
            .file_checksums
            .insert("plix-client".to_string(), "a".repeat(64));

        let manifest = test_manifest("1.0.0");

        let status = check_for_updates(&state, &manifest).unwrap();
        match status {
            UpdateStatus::UpToDate { version } => {
                assert_eq!(version, "1.0.0");
            }
            _ => panic!("Expected UpToDate, got {:?}", status),
        }
    }

    #[test]
    fn test_update_available() {
        let mut state = LocalState::default();
        state.installed_version = Some("1.0.0".to_string());
        state
            .file_checksums
            .insert("plix-client".to_string(), "b".repeat(64));

        let manifest = test_manifest("1.1.0");

        let status = check_for_updates(&state, &manifest).unwrap();
        match status {
            UpdateStatus::UpdateAvailable {
                current,
                latest,
                files_to_update,
                ..
            } => {
                assert_eq!(current, "1.0.0");
                assert_eq!(latest, "1.1.0");
                assert_eq!(files_to_update.len(), 1);
            }
            _ => panic!("Expected UpdateAvailable, got {:?}", status),
        }
    }

    #[test]
    fn test_downgrade_not_allowed() {
        let mut state = LocalState::default();
        state.installed_version = Some("2.0.0".to_string());

        let manifest = test_manifest("1.0.0");

        let status = check_for_updates(&state, &manifest).unwrap();
        match status {
            UpdateStatus::UpToDate { version } => {
                assert_eq!(version, "2.0.0");
            }
            _ => panic!("Expected UpToDate (no downgrade), got {:?}", status),
        }
    }

    #[test]
    fn test_find_files_to_update_new_file() {
        let state = LocalState::default();
        let manifest = test_manifest("1.0.0");

        let files = find_files_to_update(&state, &manifest);
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_find_files_to_update_checksum_match() {
        let mut state = LocalState::default();
        state
            .file_checksums
            .insert("plix-client".to_string(), "a".repeat(64));

        let manifest = test_manifest("1.0.0");

        let files = find_files_to_update(&state, &manifest);
        assert_eq!(files.len(), 0); // No files need update
    }

    #[test]
    fn test_can_launch() {
        assert!(UpdateStatus::UpToDate {
            version: "1.0.0".into()
        }
        .can_launch());
        assert!(UpdateStatus::Offline {
            local_version: "1.0.0".into()
        }
        .can_launch());
        assert!(!UpdateStatus::CannotProceed {
            reason: "test".into()
        }
        .can_launch());
    }

    #[test]
    fn test_needs_download() {
        assert!(!UpdateStatus::UpToDate {
            version: "1.0.0".into()
        }
        .needs_download());
        assert!(UpdateStatus::FreshInstall {
            version: "1.0.0".into(),
            files: vec![],
            total_download_size: 0
        }
        .needs_download());
    }
}
