//! Payload receiver for chunked transfer reception
//!
//! Handles receiving payload chunks from the server, reassembly, and verification.

use sha2::{Digest, Sha256};
use std::collections::HashSet;

/// State of a payload transfer in progress
pub struct PayloadReceiver {
    /// Expected SHA-256 hash of complete payload
    expected_hash: String,
    /// Total size in bytes
    total_size: u64,
    /// Size of each chunk
    chunk_size: u32,
    /// Total number of chunks expected
    num_chunks: u32,
    /// Chunks received so far
    received_chunks: Vec<Option<Vec<u8>>>,
    /// Set of received chunk indices
    received_indices: HashSet<u32>,
}

/// Result of payload verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyResult {
    /// Payload verified successfully
    Ok,
    /// Hash mismatch - expected vs actual
    HashMismatch { expected: String, actual: String },
    /// Missing chunks that need to be requested
    MissingChunks(Vec<u32>),
}

impl PayloadReceiver {
    /// Create a new receiver for a payload transfer
    pub fn new(expected_hash: String, total_size: u64, chunk_size: u32, num_chunks: u32) -> Self {
        Self {
            expected_hash,
            total_size,
            chunk_size,
            num_chunks,
            received_chunks: vec![None; num_chunks as usize],
            received_indices: HashSet::new(),
        }
    }

    /// Get the expected hash
    pub fn expected_hash(&self) -> &str {
        &self.expected_hash
    }

    /// Check if all chunks have been received
    pub fn is_complete(&self) -> bool {
        self.received_indices.len() == self.num_chunks as usize
    }

    /// Get the number of chunks received
    pub fn received_count(&self) -> u32 {
        self.received_indices.len() as u32
    }

    /// Receive a chunk of payload data
    pub fn receive_chunk(&mut self, index: u32, data: Vec<u8>) -> Result<(), String> {
        if index >= self.num_chunks {
            return Err(format!(
                "Chunk index {} out of range (max {})",
                index,
                self.num_chunks - 1
            ));
        }

        // Validate chunk size (last chunk may be smaller)
        let expected_size = if index == self.num_chunks - 1 {
            let remaining = self.total_size as u32 % self.chunk_size;
            if remaining == 0 {
                self.chunk_size
            } else {
                remaining
            }
        } else {
            self.chunk_size
        };

        if data.len() as u32 != expected_size {
            return Err(format!(
                "Chunk {} has wrong size: expected {}, got {}",
                index,
                expected_size,
                data.len()
            ));
        }

        self.received_chunks[index as usize] = Some(data);
        self.received_indices.insert(index);

        Ok(())
    }

    /// Get indices of missing chunks
    pub fn missing_indices(&self) -> Vec<u32> {
        (0..self.num_chunks)
            .filter(|i| !self.received_indices.contains(i))
            .collect()
    }

    /// Reassemble the complete payload from chunks
    pub fn reassemble(&self) -> Result<Vec<u8>, String> {
        if !self.is_complete() {
            return Err(format!(
                "Cannot reassemble: {} chunks missing",
                self.num_chunks as usize - self.received_indices.len()
            ));
        }

        let mut payload = Vec::with_capacity(self.total_size as usize);
        for chunk in &self.received_chunks {
            if let Some(data) = chunk {
                payload.extend_from_slice(data);
            }
        }

        Ok(payload)
    }

    /// Verify the reassembled payload against the expected hash
    pub fn verify(&self) -> VerifyResult {
        if !self.is_complete() {
            return VerifyResult::MissingChunks(self.missing_indices());
        }

        match self.reassemble() {
            Ok(payload) => {
                let mut hasher = Sha256::new();
                hasher.update(&payload);
                let actual_hash = format!("{:x}", hasher.finalize());

                if actual_hash == self.expected_hash {
                    VerifyResult::Ok
                } else {
                    VerifyResult::HashMismatch {
                        expected: self.expected_hash.clone(),
                        actual: actual_hash,
                    }
                }
            }
            Err(_) => VerifyResult::MissingChunks(self.missing_indices()),
        }
    }

    /// Clear all received data (for retry or cleanup)
    pub fn clear(&mut self) {
        self.received_chunks = vec![None; self.num_chunks as usize];
        self.received_indices.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    #[test]
    fn test_receive_and_verify() {
        let data = b"Hello, World! This is test payload data.";
        let hash = compute_hash(data);
        let chunk_size = 16u32;
        let num_chunks = (data.len() as u32 + chunk_size - 1) / chunk_size;

        let mut receiver = PayloadReceiver::new(hash, data.len() as u64, chunk_size, num_chunks);

        assert!(!receiver.is_complete());
        assert_eq!(receiver.received_count(), 0);

        // Receive chunks
        for i in 0..num_chunks {
            let start = (i * chunk_size) as usize;
            let end = std::cmp::min(start + chunk_size as usize, data.len());
            let chunk = data[start..end].to_vec();
            receiver.receive_chunk(i, chunk).unwrap();
        }

        assert!(receiver.is_complete());
        assert_eq!(receiver.verify(), VerifyResult::Ok);
    }

    #[test]
    fn test_missing_chunks() {
        let data = b"Hello, World! This is test payload data.";
        let hash = compute_hash(data);
        let chunk_size = 16u32;
        let num_chunks = (data.len() as u32 + chunk_size - 1) / chunk_size;

        let mut receiver = PayloadReceiver::new(hash, data.len() as u64, chunk_size, num_chunks);

        // Only receive first chunk
        receiver
            .receive_chunk(0, data[0..16].to_vec())
            .unwrap();

        assert!(!receiver.is_complete());
        let missing = receiver.missing_indices();
        assert!(!missing.is_empty());
        assert!(!missing.contains(&0));
    }

    #[test]
    fn test_hash_mismatch() {
        let data = b"Hello, World!";
        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();

        let mut receiver = PayloadReceiver::new(wrong_hash.clone(), data.len() as u64, 16, 1);
        receiver.receive_chunk(0, data.to_vec()).unwrap();

        match receiver.verify() {
            VerifyResult::HashMismatch { expected, actual: _ } => {
                assert_eq!(expected, wrong_hash);
            }
            _ => panic!("Expected hash mismatch"),
        }
    }
}
