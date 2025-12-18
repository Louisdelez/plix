//! Atomic file write utilities.
//!
//! Provides crash-safe file operations using the temp file + rename pattern.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

/// Write data to a file atomically.
///
/// Uses the temp file + rename pattern to ensure the file is either
/// fully written or not modified at all in case of crash.
///
/// # Arguments
/// * `path` - Target file path.
/// * `data` - Data to write.
///
/// # Returns
/// * `Ok(())` on success.
/// * `Err(io::Error)` on failure.
///
/// # Safety
/// - Creates parent directories if needed.
/// - Writes to a temp file first.
/// - Syncs the file to disk.
/// - Atomically renames to the target path.
/// - Syncs the parent directory (POSIX durability).
pub fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write to temp file
    let temp_path = path.with_extension("tmp");
    let mut file = File::create(&temp_path)?;
    file.write_all(data)?;
    file.sync_all()?;

    // Atomic rename
    fs::rename(&temp_path, path)?;

    // Sync parent directory for durability (POSIX)
    if let Some(parent) = path.parent() {
        if let Ok(dir) = File::open(parent) {
            let _ = dir.sync_all();
        }
    }

    Ok(())
}

/// Write data to a file atomically using a custom temp suffix.
///
/// Like `atomic_write` but allows specifying the temp file suffix.
/// Useful when multiple atomic writes might happen concurrently.
///
/// # Arguments
/// * `path` - Target file path.
/// * `data` - Data to write.
/// * `suffix` - Suffix for the temp file (e.g., process ID or timestamp).
pub fn atomic_write_with_suffix(path: &Path, data: &[u8], suffix: &str) -> io::Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Build temp path with custom suffix
    let temp_filename = format!(
        "{}.tmp{}",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("file"),
        suffix
    );
    let temp_path = path.with_file_name(temp_filename);

    // Write to temp file
    let mut file = File::create(&temp_path)?;
    file.write_all(data)?;
    file.sync_all()?;

    // Atomic rename
    fs::rename(&temp_path, path)?;

    // Sync parent directory for durability (POSIX)
    if let Some(parent) = path.parent() {
        if let Ok(dir) = File::open(parent) {
            let _ = dir.sync_all();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_atomic_write_basic() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bin");

        let data = b"hello world";
        atomic_write(&file_path, data).unwrap();

        let read_data = fs::read(&file_path).unwrap();
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_atomic_write_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nested").join("dir").join("test.bin");

        let data = b"nested data";
        atomic_write(&file_path, data).unwrap();

        assert!(file_path.exists());
        let read_data = fs::read(&file_path).unwrap();
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_atomic_write_overwrites() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bin");

        atomic_write(&file_path, b"first").unwrap();
        atomic_write(&file_path, b"second").unwrap();

        let read_data = fs::read(&file_path).unwrap();
        assert_eq!(read_data, b"second");
    }

    #[test]
    fn test_atomic_write_no_temp_file_left() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bin");
        let temp_path = file_path.with_extension("tmp");

        atomic_write(&file_path, b"data").unwrap();

        // Temp file should be renamed, not left behind
        assert!(!temp_path.exists());
        assert!(file_path.exists());
    }

    #[test]
    fn test_atomic_write_with_suffix() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bin");

        let data = b"suffixed data";
        atomic_write_with_suffix(&file_path, data, ".12345").unwrap();

        let read_data = fs::read(&file_path).unwrap();
        assert_eq!(read_data, data);
    }
}
