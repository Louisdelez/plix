//! Backup System for Migration Safety
//!
//! Creates timestamped backups before migrations and maintains a rolling window
//! of the 3 most recent backups.

use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Maximum number of backups to retain (rolling window)
pub const MAX_BACKUPS: usize = 3;

/// Backup metadata
#[derive(Debug, Clone)]
pub struct Backup {
    /// Path to backup file
    pub path: PathBuf,
    /// Original file path
    pub original_path: PathBuf,
    /// Timestamp string (ISO 8601 format)
    pub timestamp: String,
    /// Size in bytes
    pub size_bytes: u64,
    /// SHA-256 checksum of backup contents
    pub checksum: String,
}

/// Errors that can occur during backup operations
#[derive(Debug, Error)]
pub enum BackupError {
    #[error("Failed to create backup directory: {0}")]
    CreateDir(#[source] io::Error),

    #[error("Failed to read source file: {0}")]
    ReadSource(#[source] io::Error),

    #[error("Failed to write backup file: {0}")]
    WriteBackup(#[source] io::Error),

    #[error("Failed to delete old backup: {0}")]
    DeleteBackup(#[source] io::Error),

    #[error("Failed to list backup directory: {0}")]
    ListDir(#[source] io::Error),

    #[error("Source file does not exist: {0}")]
    SourceNotFound(PathBuf),

    #[error("Checksum verification failed")]
    ChecksumMismatch,
}

/// Create a backup of a file before migration.
///
/// The backup is stored in the specified backup directory with a timestamped name.
/// A SHA-256 checksum is computed to verify backup integrity.
///
/// # Arguments
///
/// * `source_path` - Path to the file to backup
/// * `backup_dir` - Directory where backups should be stored
///
/// # Returns
///
/// A `Backup` struct containing metadata about the created backup.
///
/// # Example
///
/// ```ignore
/// let backup = create_backup(Path::new("config.toml"), Path::new("./backups"))?;
/// println!("Backup created: {}", backup.path.display());
/// ```
pub fn create_backup(source_path: &Path, backup_dir: &Path) -> Result<Backup, BackupError> {
    // Ensure source exists
    if !source_path.exists() {
        return Err(BackupError::SourceNotFound(source_path.to_path_buf()));
    }

    // Create backup directory if needed
    fs::create_dir_all(backup_dir).map_err(BackupError::CreateDir)?;

    // Read source file
    let content = fs::read(source_path).map_err(BackupError::ReadSource)?;

    // Compute checksum
    let checksum = compute_sha256(&content);

    // Generate timestamp and backup filename
    let timestamp = generate_timestamp();
    let filename = source_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let backup_name = format!("{}.bak.{}", filename, timestamp);
    let backup_path = backup_dir.join(&backup_name);

    // Write backup
    fs::write(&backup_path, &content).map_err(BackupError::WriteBackup)?;

    // Verify backup was written correctly
    let verify_content = fs::read(&backup_path).map_err(BackupError::ReadSource)?;
    if compute_sha256(&verify_content) != checksum {
        return Err(BackupError::ChecksumMismatch);
    }

    tracing::info!(
        source = %source_path.display(),
        backup = %backup_path.display(),
        size = content.len(),
        checksum = %checksum,
        "Backup created successfully"
    );

    Ok(Backup {
        path: backup_path,
        original_path: source_path.to_path_buf(),
        timestamp,
        size_bytes: content.len() as u64,
        checksum,
    })
}

/// Rotate backups, keeping only the N most recent.
///
/// Deletes older backups to maintain the rolling window defined by `keep_count`.
///
/// # Arguments
///
/// * `backup_dir` - Directory containing backups
/// * `original_filename` - Name of the original file (used to find related backups)
/// * `keep_count` - Maximum number of backups to retain
///
/// # Returns
///
/// A list of paths to deleted backup files.
pub fn rotate_backups(
    backup_dir: &Path,
    original_filename: &str,
    keep_count: usize,
) -> Result<Vec<PathBuf>, BackupError> {
    let pattern = format!("{}.bak.", original_filename);

    // List all backups for this file
    let mut backups: Vec<_> = fs::read_dir(backup_dir)
        .map_err(BackupError::ListDir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map_or(false, |n| n.starts_with(&pattern))
        })
        .collect();

    // Sort by filename (which includes timestamp, so newer = later in sort)
    backups.sort_by_key(|e| e.file_name());

    // Delete oldest backups if we exceed the limit
    let mut deleted = Vec::new();
    while backups.len() > keep_count {
        if let Some(oldest) = backups.first() {
            let path = oldest.path();
            fs::remove_file(&path).map_err(BackupError::DeleteBackup)?;
            tracing::debug!(path = %path.display(), "Deleted old backup");
            deleted.push(path);
            backups.remove(0);
        }
    }

    if !deleted.is_empty() {
        tracing::info!(
            file = original_filename,
            deleted_count = deleted.len(),
            remaining = backups.len(),
            "Rotated backups"
        );
    }

    Ok(deleted)
}

/// List all backups for a specific file.
///
/// # Arguments
///
/// * `backup_dir` - Directory containing backups
/// * `original_filename` - Name of the original file
///
/// # Returns
///
/// A list of backup paths, sorted from oldest to newest.
pub fn list_backups(backup_dir: &Path, original_filename: &str) -> Result<Vec<PathBuf>, BackupError> {
    let pattern = format!("{}.bak.", original_filename);

    let mut backups: Vec<PathBuf> = fs::read_dir(backup_dir)
        .map_err(BackupError::ListDir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map_or(false, |n| n.starts_with(&pattern))
        })
        .map(|e| e.path())
        .collect();

    backups.sort();
    Ok(backups)
}

/// Restore from a backup file.
///
/// Copies the backup content back to the original location.
///
/// # Arguments
///
/// * `backup_path` - Path to the backup file
/// * `restore_path` - Path where the backup should be restored
pub fn restore_backup(backup_path: &Path, restore_path: &Path) -> Result<(), BackupError> {
    let content = fs::read(backup_path).map_err(BackupError::ReadSource)?;
    fs::write(restore_path, &content).map_err(BackupError::WriteBackup)?;

    tracing::info!(
        backup = %backup_path.display(),
        restore = %restore_path.display(),
        "Restored from backup"
    );

    Ok(())
}

/// Compute SHA-256 checksum of data.
fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Generate ISO 8601 timestamp for backup naming.
fn generate_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_backup() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("config.toml");
        let backup_dir = temp.path().join("backups");

        // Create source file
        fs::write(&source, "test content").unwrap();

        // Create backup
        let backup = create_backup(&source, &backup_dir).unwrap();

        assert!(backup.path.exists());
        assert_eq!(backup.original_path, source);
        assert_eq!(backup.size_bytes, 12);
        assert!(!backup.checksum.is_empty());

        // Verify backup content matches
        let backup_content = fs::read_to_string(&backup.path).unwrap();
        assert_eq!(backup_content, "test content");
    }

    #[test]
    fn test_create_backup_source_not_found() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("nonexistent.toml");
        let backup_dir = temp.path().join("backups");

        let result = create_backup(&source, &backup_dir);
        assert!(matches!(result, Err(BackupError::SourceNotFound(_))));
    }

    #[test]
    fn test_rotate_backups_keeps_three() {
        let temp = TempDir::new().unwrap();
        let backup_dir = temp.path();

        // Create 5 backups
        for i in 1..=5 {
            let backup_name = format!("config.toml.bak.2025-01-0{}T12-00-00", i);
            fs::write(backup_dir.join(&backup_name), format!("backup {}", i)).unwrap();
        }

        // Rotate to keep only 3
        let deleted = rotate_backups(backup_dir, "config.toml", 3).unwrap();

        // Should have deleted 2 oldest
        assert_eq!(deleted.len(), 2);

        // Should have 3 remaining
        let remaining = list_backups(backup_dir, "config.toml").unwrap();
        assert_eq!(remaining.len(), 3);

        // Remaining should be the 3 newest (03, 04, 05)
        let names: Vec<_> = remaining
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert!(names[0].contains("03"));
        assert!(names[1].contains("04"));
        assert!(names[2].contains("05"));
    }

    #[test]
    fn test_rotate_backups_no_deletion_needed() {
        let temp = TempDir::new().unwrap();
        let backup_dir = temp.path();

        // Create only 2 backups
        for i in 1..=2 {
            let backup_name = format!("config.toml.bak.2025-01-0{}T12-00-00", i);
            fs::write(backup_dir.join(&backup_name), format!("backup {}", i)).unwrap();
        }

        // Rotate to keep 3 (should delete nothing)
        let deleted = rotate_backups(backup_dir, "config.toml", 3).unwrap();
        assert!(deleted.is_empty());

        // Should still have 2
        let remaining = list_backups(backup_dir, "config.toml").unwrap();
        assert_eq!(remaining.len(), 2);
    }

    #[test]
    fn test_restore_backup() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("config.toml");
        let backup_dir = temp.path().join("backups");

        // Create and backup source
        fs::write(&source, "original content").unwrap();
        let backup = create_backup(&source, &backup_dir).unwrap();

        // Modify source
        fs::write(&source, "modified content").unwrap();
        assert_eq!(fs::read_to_string(&source).unwrap(), "modified content");

        // Restore from backup
        restore_backup(&backup.path, &source).unwrap();

        // Source should have original content again
        assert_eq!(fs::read_to_string(&source).unwrap(), "original content");
    }

    #[test]
    fn test_checksum_computation() {
        let data = b"test data";
        let checksum = compute_sha256(data);
        // Known SHA-256 of "test data"
        assert_eq!(
            checksum,
            "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9"
        );
    }
}
