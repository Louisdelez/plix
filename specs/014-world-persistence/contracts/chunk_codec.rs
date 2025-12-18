//! ChunkCodec API Contract
//!
//! This file defines the public API for chunk serialization.
//! Implementation in: crates/plix-common/src/persist/chunk_codec.rs

use crate::chunk::{Chunk, ChunkCoord};
use crate::persist::PersistError;

/// Current chunk data format version.
pub const CHUNK_FORMAT_VERSION: u8 = 1;

/// Expected size of encoded chunk data (approximate).
/// Used for buffer pre-allocation and validation.
pub const CHUNK_DATA_SIZE_APPROX: usize = 4109;

/// Serializable chunk data structure.
///
/// This is the on-disk format, separate from runtime Chunk struct.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChunkData {
    /// Format version for this chunk file.
    pub version: u8,

    /// Chunk coordinates.
    pub coord: ChunkCoord,

    /// Block data (4096 blocks = 16×16×16).
    pub blocks: [crate::types::BlockType; crate::chunk::CHUNK_BLOCK_COUNT],
}

impl ChunkData {
    /// Create ChunkData from a runtime Chunk.
    ///
    /// # Arguments
    /// * `chunk` - Runtime chunk to serialize
    ///
    /// # Returns
    /// ChunkData ready for encoding.
    pub fn from_chunk(chunk: &Chunk) -> Self;

    /// Convert ChunkData to a runtime Chunk.
    ///
    /// # Returns
    /// Runtime Chunk with data from this ChunkData.
    ///
    /// # Note
    /// The returned chunk has dirty=false (no mesh rebuild needed on load).
    pub fn into_chunk(self) -> Chunk;
}

/// Codec for chunk serialization/deserialization.
///
/// Stateless - all methods are associated functions.
pub struct ChunkCodec;

impl ChunkCodec {
    // =========================================================================
    // ENCODING
    // =========================================================================

    /// Encode a chunk to bytes.
    ///
    /// # Arguments
    /// * `chunk` - Runtime chunk to encode
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Encoded bytes
    /// * `Err(PersistError::Codec)` - If serialization fails (shouldn't happen)
    ///
    /// # Format
    /// bincode-encoded ChunkData struct.
    ///
    /// # Size
    /// Approximately 4109 bytes per chunk.
    pub fn encode(chunk: &Chunk) -> Result<Vec<u8>, PersistError>;

    /// Encode chunk data directly.
    ///
    /// # Arguments
    /// * `data` - ChunkData struct to encode
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Encoded bytes
    /// * `Err(PersistError::Codec)` - If serialization fails
    pub fn encode_data(data: &ChunkData) -> Result<Vec<u8>, PersistError>;

    // =========================================================================
    // DECODING
    // =========================================================================

    /// Decode bytes to a chunk.
    ///
    /// # Arguments
    /// * `bytes` - Encoded chunk data
    ///
    /// # Returns
    /// * `Ok(Chunk)` - Decoded runtime chunk
    /// * `Err(PersistError::Codec)` - If deserialization fails
    /// * `Err(PersistError::ChunkCorrupted)` - If validation fails
    ///
    /// # Validation
    /// * Checks version is supported
    /// * Checks block array length
    pub fn decode(bytes: &[u8]) -> Result<Chunk, PersistError>;

    /// Decode bytes to ChunkData (for inspection/migration).
    ///
    /// # Arguments
    /// * `bytes` - Encoded chunk data
    ///
    /// # Returns
    /// * `Ok(ChunkData)` - Decoded data struct
    /// * `Err(PersistError::Codec)` - If deserialization fails
    pub fn decode_data(bytes: &[u8]) -> Result<ChunkData, PersistError>;

    // =========================================================================
    // VALIDATION
    // =========================================================================

    /// Validate encoded chunk data without full decode.
    ///
    /// # Arguments
    /// * `bytes` - Encoded chunk data
    ///
    /// # Returns
    /// * `Ok(ChunkCoord)` - Chunk coordinates if valid
    /// * `Err(PersistError)` - If invalid
    ///
    /// # Performance
    /// Faster than full decode - only reads header.
    ///
    /// # Checks
    /// * Minimum size for header
    /// * Version field is supported
    /// * Can extract coordinates
    pub fn validate(bytes: &[u8]) -> Result<ChunkCoord, PersistError>;

    /// Check if a version number is supported.
    ///
    /// # Arguments
    /// * `version` - Chunk format version
    ///
    /// # Returns
    /// * `true` if version can be loaded (possibly with migration)
    /// * `false` if version is unsupported
    pub fn is_version_supported(version: u8) -> bool;

    // =========================================================================
    // MIGRATION
    // =========================================================================

    /// Migrate chunk data from old version to current.
    ///
    /// # Arguments
    /// * `data` - ChunkData with old version
    ///
    /// # Returns
    /// * `Ok(ChunkData)` - Migrated data with current version
    /// * `Err(PersistError)` - If migration not possible
    ///
    /// # Note
    /// Currently no migrations defined (v1 only).
    /// Will be implemented when v2 is introduced.
    pub fn migrate(data: ChunkData) -> Result<ChunkData, PersistError>;
}

// =============================================================================
// FILE NAMING
// =============================================================================

/// Generate filename for a chunk coordinate.
///
/// # Arguments
/// * `coord` - Chunk coordinates
///
/// # Returns
/// Filename in format `<x>_<y>_<z>.bin`
///
/// # Examples
/// * (0, 0, 0) → "0_0_0.bin"
/// * (-5, 10, 3) → "-5_10_3.bin"
pub fn chunk_filename(coord: ChunkCoord) -> String;

/// Parse chunk coordinate from filename.
///
/// # Arguments
/// * `filename` - Filename to parse
///
/// # Returns
/// * `Some(ChunkCoord)` if valid chunk filename
/// * `None` if filename doesn't match pattern
///
/// # Pattern
/// `<x>_<y>_<z>.bin` where x, y, z are signed integers.
pub fn parse_chunk_filename(filename: &str) -> Option<ChunkCoord>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        // Create chunk with some blocks
        let mut chunk = Chunk::new(ChunkCoord::new(1, 2, 3));
        chunk.set_block(0, 0, 0, BlockType::STONE);
        chunk.set_block(15, 15, 15, BlockType::BRICK);

        // Encode
        let bytes = ChunkCodec::encode(&chunk).unwrap();

        // Decode
        let decoded = ChunkCodec::decode(&bytes).unwrap();

        // Verify
        assert_eq!(decoded.coord(), chunk.coord());
        assert_eq!(decoded.get_block(0, 0, 0), BlockType::STONE);
        assert_eq!(decoded.get_block(15, 15, 15), BlockType::BRICK);
    }

    #[test]
    fn test_filename_roundtrip() {
        let coord = ChunkCoord::new(-10, 5, 100);
        let filename = chunk_filename(coord);
        assert_eq!(filename, "-10_5_100.bin");

        let parsed = parse_chunk_filename(&filename).unwrap();
        assert_eq!(parsed, coord);
    }
}
