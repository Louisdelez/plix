//! Optional Ed25519 signature verification for mod bundles
//!
//! This module provides signature verification using Ed25519.
//! It is feature-gated behind the `signatures` feature.

use crate::errors::DistributionError;
use ed25519_dalek::{PublicKey, Signature, Verifier};
use std::collections::HashMap;

/// Public key fingerprint (first 8 bytes of public key, hex-encoded = 16 chars)
pub type KeyId = String;

/// Trusted key store for signature verification
#[derive(Debug, Default)]
pub struct TrustedKeys {
    /// Mapping from key ID to full public key
    keys: HashMap<KeyId, PublicKey>,
}

impl TrustedKeys {
    /// Create an empty trusted key store
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    /// Add a trusted public key
    pub fn add_key(&mut self, public_key_hex: &str) -> Result<KeyId, DistributionError> {
        let key_bytes = hex_decode(public_key_hex).map_err(|e| {
            DistributionError::signature_invalid("key", &format!("Invalid hex encoding: {}", e))
        })?;

        if key_bytes.len() != 32 {
            return Err(DistributionError::signature_invalid(
                "key",
                &format!("Expected 32 bytes, got {}", key_bytes.len()),
            ));
        }

        let key_array: [u8; 32] = key_bytes.try_into().unwrap();
        let public_key = PublicKey::from_bytes(&key_array).map_err(|e| {
            DistributionError::signature_invalid("key", &format!("Invalid Ed25519 key: {}", e))
        })?;

        let key_id = hex_encode(&key_array[..8]);
        self.keys.insert(key_id.clone(), public_key);

        Ok(key_id)
    }

    /// Get a public key by its ID
    pub fn get_key(&self, key_id: &str) -> Option<&PublicKey> {
        self.keys.get(key_id)
    }

    /// Check if a key ID is trusted
    pub fn is_trusted(&self, key_id: &str) -> bool {
        self.keys.contains_key(key_id)
    }

    /// Number of trusted keys
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

/// Verify an Ed25519 signature
///
/// # Arguments
/// * `data` - The data that was signed (typically the SHA-256 hash of the bundle)
/// * `signature_hex` - The signature in hex format (128 chars = 64 bytes)
/// * `key_id` - The key ID that supposedly signed this
/// * `trusted_keys` - Store of trusted public keys
pub fn verify_signature(
    data: &[u8],
    signature_hex: &str,
    key_id: &str,
    trusted_keys: &TrustedKeys,
) -> Result<(), DistributionError> {
    // Look up the key
    let public_key = trusted_keys.get_key(key_id).ok_or_else(|| {
        DistributionError::signature_invalid(key_id, "Key not in trusted keys store")
    })?;

    // Decode signature
    let signature_bytes = hex_decode(signature_hex).map_err(|e| {
        DistributionError::signature_invalid(key_id, &format!("Invalid signature hex: {}", e))
    })?;

    if signature_bytes.len() != 64 {
        return Err(DistributionError::signature_invalid(
            key_id,
            &format!("Expected 64 byte signature, got {}", signature_bytes.len()),
        ));
    }

    let signature_array: [u8; 64] = signature_bytes.try_into().unwrap();
    #[allow(deprecated)]
    let signature = Signature::new(signature_array);

    // Verify
    public_key.verify(data, &signature).map_err(|e| {
        DistributionError::signature_invalid(key_id, &format!("Signature verification failed: {}", e))
    })?;

    tracing::debug!("Signature verified for key {}", key_id);
    Ok(())
}

/// Verify a bundle signature using its hash
pub fn verify_bundle_signature(
    sha256_hash: &str,
    signature_hex: &str,
    key_id: &str,
    trusted_keys: &TrustedKeys,
) -> Result<(), DistributionError> {
    let hash_bytes = hex_decode(sha256_hash).map_err(|e| {
        DistributionError::signature_invalid(key_id, &format!("Invalid hash hex: {}", e))
    })?;

    verify_signature(&hash_bytes, signature_hex, key_id, trusted_keys)
}

/// Check if signature verification is required based on trust policy
///
/// Returns `true` if signature verification should be performed.
/// Returns `Err` if signature is required but missing or key is not allowed.
pub fn check_signature_requirement(
    trust_policy: &crate::config::TrustPolicy,
    mod_id: &str,
    signature: Option<&str>,
    signer: Option<&str>,
) -> Result<bool, crate::errors::DistributionError> {
    // If signatures are not required, skip verification
    if !trust_policy.require_signature {
        return Ok(false);
    }

    // Signatures are required - check if we have one
    match (signature, signer) {
        (Some(_sig), Some(key_id)) => {
            // Check if key is in allowed list (empty list = any key allowed)
            if !trust_policy.is_key_allowed(key_id) {
                return Err(crate::errors::DistributionError::signature_key_not_allowed(
                    mod_id, key_id,
                ));
            }
            Ok(true) // Verification required
        }
        _ => {
            // Signature required but missing
            Err(crate::errors::DistributionError::signature_missing(mod_id))
        }
    }
}

// Helper functions for hex encoding/decoding
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Keypair, SecretKey, Signer};

    fn generate_test_keypair() -> Keypair {
        // Use rand 0.8's random bytes and create keypair from secret
        use rand::RngCore;
        let mut secret_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut secret_bytes);
        let secret = SecretKey::from_bytes(&secret_bytes).unwrap();
        let public = (&secret).into();
        Keypair { secret, public }
    }

    #[test]
    fn test_trusted_keys_add_and_get() {
        let keypair = generate_test_keypair();
        let public_key_hex = hex_encode(keypair.public.as_bytes());

        let mut store = TrustedKeys::new();
        let key_id = store.add_key(&public_key_hex).unwrap();

        assert!(!key_id.is_empty());
        assert_eq!(key_id.len(), 16); // 8 bytes = 16 hex chars
        assert!(store.is_trusted(&key_id));
        assert!(store.get_key(&key_id).is_some());
    }

    #[test]
    fn test_verify_signature_valid() {
        let keypair = generate_test_keypair();
        let public_key_hex = hex_encode(keypair.public.as_bytes());

        let mut store = TrustedKeys::new();
        let key_id = store.add_key(&public_key_hex).unwrap();

        // Sign some data
        let data = b"test data to sign";
        let signature = keypair.sign(data);
        let signature_hex = hex_encode(&signature.to_bytes());

        // Verify
        let result = verify_signature(data, &signature_hex, &key_id, &store);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_signature_invalid() {
        let keypair = generate_test_keypair();
        let public_key_hex = hex_encode(keypair.public.as_bytes());

        let mut store = TrustedKeys::new();
        let key_id = store.add_key(&public_key_hex).unwrap();

        // Use wrong signature (64 zeros)
        let wrong_signature = "0".repeat(128);
        let result = verify_signature(b"test data", &wrong_signature, &key_id, &store);

        assert!(result.is_err());
    }

    #[test]
    fn test_verify_signature_unknown_key() {
        let store = TrustedKeys::new();
        let result = verify_signature(b"data", &"0".repeat(128), "unknown-key", &store);

        assert!(result.is_err());
    }

    #[test]
    fn test_hex_encode_decode() {
        let data = vec![0x00, 0x11, 0x22, 0xff];
        let encoded = hex_encode(&data);
        assert_eq!(encoded, "001122ff");

        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_hex_decode_invalid() {
        assert!(hex_decode("0").is_err()); // Odd length
        assert!(hex_decode("gg").is_err()); // Invalid chars
    }
}
