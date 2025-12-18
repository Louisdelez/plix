//! SHA256 checksum verification

use crate::error::LauncherError;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Calculate SHA256 checksum of a file
pub fn calculate_file_checksum(path: &Path) -> Result<String, LauncherError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();

    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Calculate SHA256 checksum of bytes
pub fn calculate_checksum(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Verify file matches expected checksum
pub fn verify_file_checksum(path: &Path, expected: &str) -> Result<bool, LauncherError> {
    let actual = calculate_file_checksum(path)?;
    Ok(actual.to_lowercase() == expected.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_calculate_checksum() {
        let data = b"hello world\n";
        let checksum = calculate_checksum(data);
        // SHA256 checksums are always 64 hex characters
        assert_eq!(checksum.len(), 64);
        assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_calculate_file_checksum() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");

        let mut file = File::create(&path).unwrap();
        file.write_all(b"test content").unwrap();

        let checksum = calculate_file_checksum(&path).unwrap();
        assert_eq!(checksum.len(), 64);
        assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_verify_file_checksum() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");

        let content = b"verify me";
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();

        let expected = calculate_checksum(content);
        assert!(verify_file_checksum(&path, &expected).unwrap());

        // Wrong checksum should fail
        assert!(!verify_file_checksum(&path, &"0".repeat(64)).unwrap());
    }

    #[test]
    fn test_checksum_case_insensitive() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");

        let content = b"case test";
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();

        let expected = calculate_checksum(content);
        let expected_upper = expected.to_uppercase();

        // Should match regardless of case
        assert!(verify_file_checksum(&path, &expected_upper).unwrap());
    }

    #[test]
    fn test_calculate_file_checksum_missing() {
        let result = calculate_file_checksum(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }
}
