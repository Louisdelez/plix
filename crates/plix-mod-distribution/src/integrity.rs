//! SHA-256 integrity verification for mod bundles

use crate::errors::DistributionError;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

/// Calculate SHA-256 hash of a file (streaming, doesn't load entire file into memory)
pub fn calculate_hash(path: &Path) -> Result<String, DistributionError> {
    let file = std::fs::File::open(path).map_err(|e| {
        DistributionError::internal(&format!(
            "Failed to open file '{}' for hashing: {}",
            path.display(),
            e
        ))
    })?;

    let mut reader = std::io::BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192]; // 8KB buffer

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| {
            DistributionError::internal(&format!(
                "Failed to read file '{}' for hashing: {}",
                path.display(),
                e
            ))
        })?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Calculate SHA-256 hash of bytes
pub fn calculate_hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Verify that a bundle's hash matches the expected value
pub fn verify_bundle_hash(path: &Path, expected_hash: &str) -> Result<(), DistributionError> {
    let actual_hash = calculate_hash(path)?;

    if actual_hash.to_lowercase() != expected_hash.to_lowercase() {
        // Delete the corrupted/tampered bundle
        if let Err(e) = std::fs::remove_file(path) {
            tracing::warn!(
                "Failed to delete bundle with hash mismatch at {}: {}",
                path.display(),
                e
            );
        }

        return Err(DistributionError::hash_mismatch(
            &path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
            expected_hash,
            &actual_hash,
        ));
    }

    tracing::debug!(
        "Hash verification passed for {}: {}",
        path.display(),
        actual_hash
    );

    Ok(())
}

/// Validate SHA-256 hash format (64 hex characters)
pub fn is_valid_hash(hash: &str) -> bool {
    hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
}

// Minimal hex encoding (to avoid adding a dependency)
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    #[allow(dead_code)]
    pub fn decode(s: &str) -> Result<Vec<u8>, String> {
        if s.len() % 2 != 0 {
            return Err("Invalid hex string length".to_string());
        }

        (0..s.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&s[i..i + 2], 16)
                    .map_err(|e| format!("Invalid hex character: {}", e))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_calculate_hash_empty_file() {
        let temp = tempdir().unwrap();
        let file_path = temp.path().join("empty.txt");
        std::fs::write(&file_path, b"").unwrap();

        let hash = calculate_hash(&file_path).unwrap();
        // SHA-256 of empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_calculate_hash_known_content() {
        let temp = tempdir().unwrap();
        let file_path = temp.path().join("test.txt");
        std::fs::write(&file_path, b"hello world").unwrap();

        let hash = calculate_hash(&file_path).unwrap();
        // SHA-256 of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_calculate_hash_bytes() {
        let hash = calculate_hash_bytes(b"hello world");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_verify_bundle_hash_success() {
        let temp = tempdir().unwrap();
        let file_path = temp.path().join("bundle.plixmod");
        std::fs::write(&file_path, b"bundle content").unwrap();

        let expected_hash = calculate_hash(&file_path).unwrap();
        let result = verify_bundle_hash(&file_path, &expected_hash);

        assert!(result.is_ok());
        assert!(file_path.exists()); // File should still exist
    }

    #[test]
    fn test_verify_bundle_hash_mismatch() {
        let temp = tempdir().unwrap();
        let file_path = temp.path().join("bundle.plixmod");
        std::fs::write(&file_path, b"bundle content").unwrap();

        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        let result = verify_bundle_hash(&file_path, wrong_hash);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, crate::errors::ErrorCode::HashMismatch);

        // File should be deleted on mismatch
        assert!(!file_path.exists());
    }

    #[test]
    fn test_verify_bundle_hash_case_insensitive() {
        let temp = tempdir().unwrap();
        let file_path = temp.path().join("bundle.plixmod");
        std::fs::write(&file_path, b"test").unwrap();

        let hash = calculate_hash(&file_path).unwrap();
        let upper_hash = hash.to_uppercase();

        // Should accept uppercase hash
        let result = verify_bundle_hash(&file_path, &upper_hash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_valid_hash() {
        // Valid hashes
        assert!(is_valid_hash(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        ));
        assert!(is_valid_hash(
            "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"
        ));
        assert!(is_valid_hash(
            "0000000000000000000000000000000000000000000000000000000000000000"
        ));
        assert!(is_valid_hash(
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        ));

        // Invalid hashes
        assert!(!is_valid_hash("")); // Empty
        assert!(!is_valid_hash("abc")); // Too short
        assert!(!is_valid_hash(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b85" // 63 chars
        ));
        assert!(!is_valid_hash(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8555" // 65 chars
        ));
        assert!(!is_valid_hash(
            "g3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" // Invalid char 'g'
        ));
    }

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex::encode(&[]), "");
        assert_eq!(hex::encode(&[0]), "00");
        assert_eq!(hex::encode(&[255]), "ff");
        assert_eq!(hex::encode(&[0, 1, 2, 255]), "000102ff");
    }

    #[test]
    fn test_hex_decode() {
        assert_eq!(hex::decode("").unwrap(), Vec::<u8>::new());
        assert_eq!(hex::decode("00").unwrap(), vec![0u8]);
        assert_eq!(hex::decode("ff").unwrap(), vec![255u8]);
        assert_eq!(hex::decode("FF").unwrap(), vec![255u8]);
        assert_eq!(hex::decode("000102ff").unwrap(), vec![0u8, 1, 2, 255]);

        // Invalid
        assert!(hex::decode("0").is_err()); // Odd length
        assert!(hex::decode("gg").is_err()); // Invalid char
    }
}
