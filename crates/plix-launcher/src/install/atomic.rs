//! Atomic file installation

use crate::error::LauncherError;
use crate::manifest::ManifestFile;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Atomically install files from temp directory to version directory
pub fn install_files(
    temp_dir: &Path,
    version_dir: &Path,
    files: &[ManifestFile],
) -> Result<HashMap<String, String>, LauncherError> {
    tracing::info!(
        "Installing {} files to {}",
        files.len(),
        version_dir.display()
    );

    // Create version directory
    fs::create_dir_all(version_dir)?;

    let mut checksums = HashMap::new();

    for file in files {
        let src = temp_dir.join(&file.path);
        let dest = version_dir.join(&file.path);

        // Ensure parent directory exists
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        // Move file from temp to version dir (atomic on same filesystem)
        if src.exists() {
            // Try rename first (atomic if same filesystem)
            if fs::rename(&src, &dest).is_err() {
                // Fall back to copy + delete
                fs::copy(&src, &dest)?;
                fs::remove_file(&src)?;
            }

            // Set executable permission on Unix
            #[cfg(unix)]
            if file.executable {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&dest)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&dest, perms)?;
            }

            checksums.insert(file.path.clone(), file.sha256.clone());
            tracing::debug!("Installed: {}", file.path);
        } else {
            tracing::warn!("Source file not found: {}", src.display());
        }
    }

    Ok(checksums)
}

/// Update the "current" symlink/directory to point to a version
pub fn update_current_link(current_dir: &Path, version_dir: &Path) -> Result<(), LauncherError> {
    tracing::debug!(
        "Updating current link: {} -> {}",
        current_dir.display(),
        version_dir.display()
    );

    // Remove existing current
    if current_dir.exists() {
        if current_dir.is_symlink() || current_dir.is_file() {
            fs::remove_file(current_dir)?;
        } else {
            fs::remove_dir_all(current_dir)?;
        }
    }

    // Create symlink on Unix, copy on Windows
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(version_dir, current_dir)?;
    }

    #[cfg(windows)]
    {
        // On Windows, copy files instead of symlink (symlinks require admin)
        copy_dir_recursive(version_dir, current_dir)?;
    }

    Ok(())
}

#[cfg(windows)]
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), LauncherError> {
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if entry_path.is_dir() {
            copy_dir_recursive(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
        }
    }

    Ok(())
}

/// Clean up temp directory
pub fn cleanup_temp(temp_dir: &Path) -> Result<(), LauncherError> {
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir)?;
        tracing::debug!("Cleaned up temp directory: {}", temp_dir.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_test_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_install_files() {
        let temp = tempdir().unwrap();
        let version = tempdir().unwrap();

        let temp_dir = temp.path();
        let version_dir = version.path();

        // Create test file in temp
        create_test_file(&temp_dir.join("game"), "game binary");
        create_test_file(&temp_dir.join("data/config.toml"), "config");

        let files = vec![
            ManifestFile {
                path: "game".to_string(),
                url: "".to_string(),
                size: 11,
                sha256: "a".repeat(64),
                executable: true,
            },
            ManifestFile {
                path: "data/config.toml".to_string(),
                url: "".to_string(),
                size: 6,
                sha256: "b".repeat(64),
                executable: false,
            },
        ];

        let checksums = install_files(temp_dir, version_dir, &files).unwrap();

        assert!(version_dir.join("game").exists());
        assert!(version_dir.join("data/config.toml").exists());
        assert_eq!(checksums.len(), 2);
    }

    #[test]
    #[cfg(unix)]
    fn test_update_current_link() {
        let version = tempdir().unwrap();
        let current = tempdir().unwrap();

        let version_dir = version.path();
        let current_link = current.path().join("current");

        create_test_file(&version_dir.join("game"), "binary");

        update_current_link(&current_link, version_dir).unwrap();

        assert!(current_link.is_symlink());
        assert!(current_link.join("game").exists());
    }

    #[test]
    fn test_cleanup_temp() {
        let temp = tempdir().unwrap();
        let temp_dir = temp.path().join("to_clean");

        fs::create_dir_all(&temp_dir).unwrap();
        create_test_file(&temp_dir.join("file.txt"), "data");

        assert!(temp_dir.exists());
        cleanup_temp(&temp_dir).unwrap();
        assert!(!temp_dir.exists());
    }
}
